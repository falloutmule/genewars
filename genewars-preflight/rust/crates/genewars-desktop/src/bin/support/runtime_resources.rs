//! Fixed CD resource admission and projection of owned world state.
use std::{fs, io::Read, path::Path};
use genewars_assets::{AnbAnimationRecord, AnbFrameRecord, AnbSections, B32Atlas, CampaignHeader, ConqFastPrefix, ConqMapCells, ConqMissionSource, Palette256, PaletteChannelMode};
use genewars_core::cd_candidate::{CampaignResourcePlan, camera_cells_for_target, entity_view_position,
    interpolate_entity_shade, plan_terrain_factor_raise_4096, queue_family1, family1_draw_packet, ProjectedFamily1};
use genewars_render::{RgbImage, PlantFrame, PlantColorInputs, PlantSceneRecord, draw_explicit_family1_scene,
    terrain::TerrainFadeTable, terrain_modes::MainGlaTable, terrain_view::{render_initial_terrain_view,compose_left_panel}};
use genewars_runtime::{actor_initialization::ActorResourceTableKey, actor_resources::{ActorAnimationDatabase, ActorResourceCatalog}, effect_pool::EffectAnimationBinding, identity::sha256_hex, world::MissionWorld};

pub const CATALOG: &str = include_str!("../../../../../../research/integration/runtime-resource-catalog.tsv");

pub fn read_bound(path: &Path) -> Result<Vec<u8>, String> {
    // Full suffix matching admits only the fixed roles; expected hashes are
    // compiled into the application, never taken from a mutable bundle manifest.
    let role = CATALOG.lines().filter(|line| !line.starts_with('#')).find_map(|line| {
        let fields: Vec<_> = line.split('\t').collect();
        (fields.len() == 5 && path.ends_with(Path::new(fields[0]))).then_some(fields)
    }).ok_or_else(|| format!("unregistered runtime resource: {}", path.display()))?;
    let size: usize = role[2].parse().map_err(|_| "invalid compiled resource size")?;
    let file = fs::File::open(path).map_err(|e| format!("{}: {e}", role[0]))?;
    let mut data = Vec::with_capacity(size);
    file.take(size as u64 + 1).read_to_end(&mut data).map_err(|e| e.to_string())?;
    if data.len() != size || sha256_hex(&data) != role[1] {
        return Err(format!("{}: size/SHA-256 mismatch against CD resource catalog", role[0]));
    }
    Ok(data)
}

pub struct Resources {
    _header: Vec<u8>,
    pub title: Vec<u8>,
    atlas: Vec<u8>, fade: Vec<u8>, gla: Vec<u8>, panel: Vec<u8>,
    palette: Palette256, plants: Vec<Vec<u8>>,
    pub actors: ActorResources,
    pub effects: EffectAnimationDatabase,
}

/// The sixteen source-bound databases loaded through 56E20's style/category
/// table. Lookup always requires the actor's typed table key.
pub struct ActorResources {
    catalog: ActorResourceCatalog,
    databases: Vec<ActorAnimationDatabase>,
}

impl ActorResources {
    fn load(root: &Path) -> Result<Self, String> {
        let catalog = ActorResourceCatalog;
        let mut databases = Vec::with_capacity(16);
        for spec in catalog.specs() {
            let bytes = read_bound(&root.join(format!("actors/key-{:02}.anb", spec.key.flat_index)))?;
            databases.push(catalog.load(spec.key, bytes).map_err(|e| e.to_string())?);
        }
        if databases.len() != 16 { return Err("actor resource catalog did not resolve all sixteen keys".into()); }
        Ok(Self { catalog, databases })
    }

    pub fn database(&self, key: ActorResourceTableKey) -> Result<&ActorAnimationDatabase, String> {
        self.catalog.spec(key).map_err(|e| e.to_string())?;
        self.databases.get(usize::from(key.flat_index)).ok_or_else(|| format!("missing actor resource key {}", key.flat_index))
    }
}

/// EFF.ANB contains the two effect animation IDs used by the bounded landing
/// plans. This resolves metadata only; it does not claim effect timing or draw
/// dispatch beyond the separately traced EFF frame ranges.
pub struct EffectAnimationDatabase { bytes: Vec<u8> }

impl EffectAnimationDatabase {
    fn load(root: &Path) -> Result<Self, String> {
        let bytes = read_bound(&root.join("effects/EFF.ANB"))?;
        let value = Self { bytes };
        for (binding, expected_frames) in [(EffectAnimationBinding::EffAnimation49, 1), (EffectAnimationBinding::EffAnimation6, 13)] {
            if value.frame_count(binding)? != expected_frames {
                return Err(format!("EFF animation {:?} has an unexpected frame count", binding));
            }
        }
        Ok(value)
    }

    pub fn bytes(&self) -> &[u8] { &self.bytes }

    pub fn animation_index(binding: EffectAnimationBinding) -> usize {
        match binding { EffectAnimationBinding::EffAnimation49 => 49, EffectAnimationBinding::EffAnimation6 => 6 }
    }

    pub fn animation(&self, binding: EffectAnimationBinding) -> Result<AnbAnimationRecord, String> {
        let sections = AnbSections::parse(&self.bytes).map_err(|e| e.to_string())?;
        sections.ui_metadata().map_err(|e| e.to_string())?
            .animation(Self::animation_index(binding)).map_err(|e| e.to_string())
    }

    pub fn frame_count(&self, binding: EffectAnimationBinding) -> Result<usize, String> {
        let animation = self.animation(binding)?;
        if animation.last_frame_offset < animation.first_frame_offset
            || (animation.last_frame_offset - animation.first_frame_offset) % 14 != 0 {
            return Err(format!("EFF animation {} has an invalid frame range", Self::animation_index(binding)));
        }
        Ok(((animation.last_frame_offset - animation.first_frame_offset) / 14 + 1) as usize)
    }

    pub fn frame(&self, binding: EffectAnimationBinding, index: usize) -> Result<AnbFrameRecord, String> {
        if index >= self.frame_count(binding)? {
            return Err(format!("EFF animation {} frame {index} is out of range", Self::animation_index(binding)));
        }
        let animation = self.animation(binding)?;
        let offset = usize::try_from(animation.first_frame_offset).map_err(|e| e.to_string())?
            .checked_add(index.checked_mul(14).ok_or("EFF frame offset overflow")?).ok_or("EFF frame offset overflow")?;
        let sections = AnbSections::parse(&self.bytes).map_err(|e| e.to_string())?;
        sections.ui_metadata().map_err(|e| e.to_string())?.frame_at(u32::try_from(offset).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
    }
}

#[derive(Debug)]
pub struct Frame {
    pub image: RgbImage,
    pub queue: Vec<u32>,
    pub rgb_sha256: String,
    pub world_sha256: String,
}

pub struct IndexedFrame {
    pub indices: Vec<u8>,
    pub queue: Vec<u32>,
    pub camera: [u16; 2],
}

impl Resources {
    pub fn load(root: &Path) -> Result<(Self, MissionWorld, [i16;2]), String> {
        let plan = CampaignResourcePlan::new(0,0).map_err(|e| format!("{e:?}"))?;
        if plan.mission != "levels/conq001.dat" || plan.header != "LEVELS/hdrco1.dat" {
            return Err("initial campaign binding differs".into());
        }
        let mission = read_bound(&root.join("terrain/CONQ001.DAT"))?;
        let source = ConqMissionSource::parse(&mission).map_err(|e| e.to_string())?;
        let target = source.target_words(0).map_err(|e| e.to_string())?.map(|v| v as i16);
        let parameters = read_bound(&root.join("plant-parameters.bin"))?;
        let world = MissionWorld::from_source(&source, &parameters).map_err(|e| e.to_string())?;
        let header = read_bound(&root.join("HDRCO1.DAT"))?;
        let title = CampaignHeader::parse(&header).map_err(|e| e.to_string())?.title_bytes().to_vec();
        let atlas = read_bound(&root.join("terrain/B32A.DAT"))?;
        B32Atlas::parse(&atlas).map_err(|e| e.to_string())?;
        let fade = read_bound(&root.join("terrain/FADE.DAT"))?;
        TerrainFadeTable::parse(&fade).map_err(|e| e.to_string())?;
        let gla = read_bound(&root.join("terrain/MAIN.GLA"))?;
        MainGlaTable::parse(&gla).map_err(|e| e.to_string())?;
        let panel = read_bound(&root.join("terrain/GENEPANL.DAT"))?;
        let palette = Palette256::parse(&read_bound(&root.join("terrain/MAIN.PAL"))?, PaletteChannelMode::Rgb).map_err(|e| e.to_string())?;
        if palette.entries().iter().flatten().any(|&v| v>63) { return Err("MAIN.PAL is not RGB6".into()); }
        let plants = (0..12).map(|slot| read_bound(&root.join(format!("plants/slot-{slot:02}.anb")))).collect::<Result<Vec<_>,_>>()?;
        let actors = ActorResources::load(root)?;
        let effects = EffectAnimationDatabase::load(root)?;
        Ok((Self { _header: header, title, atlas, fade, gla, panel, palette, plants, actors, effects }, world, target))
    }

    pub fn render(&self, world: &MissionWorld, target: [i16;2]) -> Result<Frame,String> {
        let records = world.render_records().map_err(|e| e.to_string())?;
        let indexed = self.render_indexed(world.source_bytes(), &records, target)?;
        self.finish_indexed(indexed, sha256_hex(world.source_bytes()))
    }

    /// Shared indexed terrain+family1 base used by the legacy world view and
    /// the newer owned ActorWorld path. Caller remains responsible for
    /// layering other admitted families before converting to RGB.
    pub(super) fn render_indexed(
        &self,
        source_bytes: &[u8],
        records: &[genewars_runtime::world::PlantRenderRecord],
        target: [i16;2],
    ) -> Result<IndexedFrame,String> {
        let prefix = ConqFastPrefix::parse(source_bytes).map_err(|e| e.to_string())?;
        let mut map = prefix.opaque_14_byte_region().to_vec();
        // Explicit development view policy. Does not mutate world visibility,
        // call original factor side effects, or assert its initial panel gate.
        for change in plan_terrain_factor_raise_4096(&map, target).map_err(|e| e.to_string())? {
            let at = (usize::from(change.source_cell[1])*128+usize::from(change.source_cell[0]))*14+12;
            if map[at] != change.previous { return Err("factor view mismatch".into()); }
            map[at] = change.next;
        }
        let camera = camera_cells_for_target(&map,target[0],target[1]).map_err(|e| e.to_string())?;
        let cells = ConqMapCells::parse(&map).map_err(|e| e.to_string())?;
        let fade = TerrainFadeTable::parse(&self.fade).map_err(|e| e.to_string())?;
        let gla = MainGlaTable::parse(&self.gla).map_err(|e| e.to_string())?;
        let mut rendered = render_initial_terrain_view(&cells, camera.map(|v|i32::from(v&127)), &self.atlas,&fade,None,0)?;
        compose_left_panel(&mut rendered,&self.panel)?;
        let candidates = records.iter().map(|r| {
            let position = entity_view_position(&map,r.position_words,camera,128).map_err(|e| e.to_string())?;
            let shade = interpolate_entity_shade(&map,r.position_words[0],r.position_words[1]).map_err(|e| e.to_string())?;
            Ok(ProjectedFamily1 { id:r.record_index as u32,x:position[0],y:position[1],shade,flags:r.flags_at_15 })
        }).collect::<Result<Vec<_>,String>>()?;
        // The plant-only runtime occupies an empty queue; other families are
        // suspended. Full redraw mode1/gate1 replaces absent dirty rectangles.
        let queue = queue_family1(true,0,128,&candidates).map_err(|e| e.to_string())?;
        let mut frames = Vec::new();
        for q in &queue.draw_order {
            let r = records.iter().find(|r| r.record_index as u32 == q.id).ok_or("unknown render ID")?;
            let resource = self.plants.get(r.resource_selector_i8 as usize).ok_or("unsupported plant resource")?;
            frames.push(PlantFrame::parse_indexed8(resource,r.animation_u8,r.frame_u8).map_err(|e| format!("plant {}: {e}", q.id))?);
        }
        let draw = queue.draw_order.iter().zip(&frames).map(|(q,frame)| {
            let r = records.iter().find(|r|r.record_index as u32==q.id).unwrap();
            PlantSceneRecord { id:q.id,frame,origin:[i32::from(q.x),i32::from(q.draw_y)],
                inputs:PlantColorInputs { context_flags_at_16e0c4:0, entity_flags_at_15:q.flags,native_flags_at_2e:r.flags_at_2e,shade_at_17:q.shade },
                draw:family1_draw_packet(true,1,q.flags).draws }
        }).collect::<Vec<_>>();
        draw_explicit_family1_scene(&draw,&mut rendered.indices[128..],[512,480],640,Some(&fade),&gla).map_err(|e|e.to_string())?;
        Ok(IndexedFrame { indices: rendered.indices, queue: queue.draw_order.iter().map(|q|q.id).collect(), camera })
    }

    pub(super) fn finish_indexed(&self, indexed: IndexedFrame, world_sha256: String) -> Result<Frame,String> {
        let rgb = indexed.indices.iter().flat_map(|&i|self.palette.entries()[usize::from(i)].map(|v|(u16::from(v)*255/63)as u8)).collect::<Vec<_>>();
        Ok(Frame { rgb_sha256:sha256_hex(&rgb),world_sha256,
            image:RgbImage::from_rgb24(640,480,rgb).map_err(|e|e.to_string())?,queue:indexed.queue })
    }

    pub(super) fn fade_bytes(&self) -> &[u8] { &self.fade }
    pub(super) fn main_gla_bytes(&self) -> &[u8] { &self.gla }
    pub(super) fn palette_entries(&self) -> &[[u8; 3]; 256] { self.palette.entries() }
}
