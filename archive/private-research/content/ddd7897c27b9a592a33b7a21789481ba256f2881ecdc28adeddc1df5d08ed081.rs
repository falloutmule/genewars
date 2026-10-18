//! Fixed CD resource admission and projection of owned world state.
pub(super) mod creature_scene;
use std::{fs, io::Read, path::Path};
use genewars_assets::{AnbAnimationRecord, AnbFrameRecord, AnbSections, B32Atlas, CampaignHeader, ConqFastPrefix, ConqMapCells, ConqMissionSource, FontResource, Palette256, PaletteChannelMode, SpriteTable, TextStringTable};
use genewars_core::cd_candidate::{CampaignResourcePlan, camera_cells_for_target, entity_view_position,
    interpolate_entity_shade, plan_terrain_factor_raise_4096, queue_family1, family1_draw_packet, ProjectedFamily1};
use genewars_render::{RgbImage, PlantFrame, PlantColorInputs, PlantSceneRecord, draw_explicit_family1_scene,
    terrain::TerrainFadeTable, terrain_modes::MainGlaTable, terrain_view::{render_initial_terrain_view,compose_left_panel}};
use genewars_runtime::{actor_initialization::ActorResourceTableKey, actor_resources::{ActorAnimationDatabase, ActorResourceCatalog}, building_resources::BuildingResourceDatabase, creature_world::MuleAnimations, effect_pool::EffectAnimationBinding, identity::sha256_hex, physics_tables::PhysicsTables, world::MissionWorld};

use genewars_runtime::source_building_resources::{SourceBuildingBinding,SourceBuildingResourceDatabase};

pub enum BuildingResourceRef<'a> { Bturr(&'a BuildingResourceDatabase), Source(&'a SourceBuildingResourceDatabase) }

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
    font0_dat: Vec<u8>, font0_tab: Vec<u8>, text: Vec<u8>,
    palette: Palette256, plants: Vec<Vec<u8>>,
    pub actors: ActorResources,
    pub effects: EffectAnimationDatabase,
    /// Exact selector-1 BTURR database, admitted through the runtime catalog.
    /// Additional independently bound mission-one resources are stored below.
    pub bturr_selector1: BuildingResourceDatabase,
    pub source_buildings: Vec<SourceBuildingResourceDatabase>,
    /// Hash-validated CD angle and terrain bounce tables used by landing updates.
    pub physics: PhysicsTables,
    /// Hash-bound 8192-byte construction border lookup copied from validated
    /// mapped object3 VA 0x112D24. It is not a CD-disc asset.
    pub construction_border: Box<[u8; 8192]>,
    /// Exact CD selector-4 MULE animation database; its sprite codec remains
    /// explicitly selected by the creature draw path.
    pub mule: CreatureAnimationDatabase,
    /// Exact CD BONES database selected by the selector-4 state>=26 route.
    /// Its packed4 decoder is hash-bound separately from MULE; drawing policy
    /// remains the caller's responsibility.
    pub bones: BonesAnimationDatabase,
    /// Once-parsed animation hold/flag metadata from the exact same
    /// hash-admitted MULE.ANB bytes as `mule`; used by the owned world tick.
    pub mule_animations: MuleAnimations,
    pub ethereal: crate::family8_scene::EtherealAnimationDatabase,
    pub family8_animation: genewars_runtime::family8_motion::Family8Animation,
    pub family8_directions: genewars_runtime::family8_motion::Family8DirectionalAnimations,
}

pub struct CreatureAnimationDatabase { bytes: Vec<u8>, sprite_records: usize, animations: usize }

/// Independently hash-bound BONES.ANB data. It deliberately has its own type
/// so a MULE selector cannot accidentally decode these sprite records.
pub struct BonesAnimationDatabase { bytes: Vec<u8>, sprite_records: usize, animations: usize }

impl BonesAnimationDatabase {
    fn load(root: &Path) -> Result<Self, String> {
        let bytes = read_bound(&root.join("creatures/BONES.ANB"))?;
        Self::from_catalog_bytes(bytes)
    }

    fn from_catalog_bytes(bytes: Vec<u8>) -> Result<Self, String> {
        // The exact selector-4 BONES input has five animation records, ten
        // frames/components, and eleven sprite records including record zero.
        // Keep these identity-specific extents strict; generic ANB structural
        // validity alone does not admit a BONES resource.
        let sections = AnbSections::parse(&bytes).map_err(|e| e.to_string())?;
        if sections.header_word() != 5 || sections.offsets() != [3721, 3787, 3887, 4027] {
            return Err("CD BONES ANB section layout differs from its pinned selector proof".into());
        }
        let metadata = sections.ui_metadata().map_err(|e| e.to_string())?;
        let table = SpriteTable::from_anb(&bytes).map_err(|e| e.to_string())?;
        if metadata.animation_count() != 5 || metadata.frame_count() != 10
            || metadata.component_count() != 10 || table.records().len() != 11 {
            return Err("CD BONES ANB record counts differ from its pinned selector proof".into());
        }
        let expected = [
            (0u8, 0u8), (73, 30), (73, 30), (55, 34), (55, 34),
            (65, 36), (65, 36), (56, 45), (56, 45), (72, 44), (72, 44),
        ];
        for (index, descriptor) in table.records().iter().enumerate() {
            if (descriptor.width, descriptor.height) != expected[index] {
                return Err(format!("CD BONES sprite record {index} differs from its pinned descriptor"));
            }
            if index != 0 {
                table.decode_packed4(index, &bytes, 0)
                    .map_err(|e| format!("CD BONES packed4 sprite record {index}: {e}"))?;
            }
        }
        for animation_index in 0..metadata.animation_count() {
            let animation = metadata.animation(animation_index).map_err(|e| e.to_string())?;
            if animation.first_frame_offset != (animation_index * 28) as u32
                || animation.last_frame_offset != (animation_index * 28 + 14) as u32
                || animation.trailing_word != 2 {
                return Err(format!("CD BONES animation {animation_index} differs from its pinned frame range"));
            }
        }
        let sprite_records = table.records().len();
        let animations = metadata.animation_count();
        // These views borrow `bytes`; release them before retaining ownership.
        drop(table);
        Ok(Self { bytes, sprite_records, animations })
    }

    pub fn bytes(&self) -> &[u8] { &self.bytes }
    pub fn sprite_record_count(&self) -> usize { self.sprite_records }
    pub fn animation_count(&self) -> usize { self.animations }
    pub fn frame_count(&self, animation_index: usize) -> Result<usize, String> {
        let sections = AnbSections::parse(&self.bytes).map_err(|e| e.to_string())?;
        let metadata = sections.ui_metadata().map_err(|e| e.to_string())?;
        let animation = metadata.animation(animation_index).map_err(|e| e.to_string())?;
        if animation.last_frame_offset < animation.first_frame_offset
            || (animation.last_frame_offset - animation.first_frame_offset) % 14 != 0 {
            return Err(format!("CD BONES animation {animation_index} has an invalid frame range"));
        }
        Ok(((animation.last_frame_offset - animation.first_frame_offset) / 14 + 1) as usize)
    }

    /// Decode a caller-selected animation/frame using the independently
    /// admitted BONES high-first packed4 representation. `base` remains an
    /// explicit color input; this API does not claim the runtime palette path.
    pub fn decode_frame_packed4(
        &self, animation_index: usize, frame_index: usize, base: u8,
    ) -> Result<CreatureFramePixels, String> {
        let sections = AnbSections::parse(&self.bytes).map_err(|e| e.to_string())?;
        let metadata = sections.ui_metadata().map_err(|e| e.to_string())?;
        let animation = metadata.animation(animation_index).map_err(|e| e.to_string())?;
        let frame_count = self.frame_count(animation_index)?;
        if frame_index >= frame_count {
            return Err(format!("CD BONES animation {animation_index} frame {frame_index} is out of range"));
        }
        let offset = usize::try_from(animation.first_frame_offset).map_err(|e| e.to_string())?
            .checked_add(frame_index.checked_mul(14).ok_or("BONES frame offset overflow")?)
            .ok_or("BONES frame offset overflow")?;
        let frame = metadata.frame_at(u32::try_from(offset).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        let table = SpriteTable::from_anb(&self.bytes).map_err(|e| e.to_string())?;
        let mut components = Vec::with_capacity(usize::from(frame.component_count));
        for index in 0..usize::from(frame.component_count) {
            let component = metadata.frame_component(frame, index).map_err(|e| e.to_string())?;
            let sprite_record = usize::try_from(component.sprite_offset).map_err(|e| e.to_string())? / 6;
            let sprite = table.decode_packed4(sprite_record, &self.bytes, base)
                .map_err(|e| format!("CD BONES sprite record {sprite_record}: {e}"))?;
            components.push(CreatureFrameComponent {
                local_position: component.local_position(), flags: component.flags,
                sprite_record, sprite,
            });
        }
        Ok(CreatureFramePixels { animation: animation_index, frame_index, frame, components })
    }
}

/// Decoded pixels and source component metadata for one explicitly selected
/// packed-four-bit creature frame. This exposes asset contents only; a
/// caller still has to supply the palette base and the proved draw wrapper.
pub struct CreatureFrameComponent {
    pub local_position: (i16, i16),
    pub flags: u8,
    pub sprite_record: usize,
    pub sprite: genewars_assets::IndexedSprite,
}

pub struct CreatureFramePixels {
    pub animation: usize,
    pub frame_index: usize,
    pub frame: AnbFrameRecord,
    pub components: Vec<CreatureFrameComponent>,
}

impl CreatureAnimationDatabase {
    fn load(root: &Path) -> Result<Self, String> {
        let bytes = read_bound(&root.join("creatures/MULE.ANB"))?;
        let sections = AnbSections::parse(&bytes).map_err(|e| e.to_string())?;
        // These values are pinned for the exact CD MULE input. Parsing also
        // checks all section ranges before the runtime keeps the resource.
        if sections.header_word() != 24 || sections.offsets() != [51836, 52526, 54306, 56798] {
            return Err("CD MULE ANB section layout differs from its pinned resource proof".into());
        }
        let metadata = sections.ui_metadata().map_err(|e| e.to_string())?;
        let table = SpriteTable::from_anb(&bytes).map_err(|e| e.to_string())?;
        if metadata.animation_count() != 24 || table.records().len() != 115 {
            return Err("CD MULE ANB animation or sprite-table count differs from its pinned proof".into());
        }
        let sprite_records = table.records().len();
        let animations = metadata.animation_count();
        drop((sections, metadata, table));
        Ok(Self { bytes, sprite_records, animations })
    }

    pub fn bytes(&self) -> &[u8] { &self.bytes }
    pub fn sprite_record_count(&self) -> usize { self.sprite_records }
    pub fn animation_count(&self) -> usize { self.animations }

    /// Decode one caller-selected frame using the explicitly proved packed4
    /// codec for this MULE database. `palette_base` is deliberately supplied
    /// by the caller: this method does not choose a creature owner/fade path.
    pub fn decode_frame_packed4(
        &self,
        animation_index: usize,
        frame_index: usize,
        palette_base: u8,
    ) -> Result<CreatureFramePixels, String> {
        let sections = AnbSections::parse(&self.bytes).map_err(|e| e.to_string())?;
        let metadata = sections.ui_metadata().map_err(|e| e.to_string())?;
        let animation = metadata.animation(animation_index).map_err(|e| e.to_string())?;
        if animation.last_frame_offset < animation.first_frame_offset
            || (animation.last_frame_offset - animation.first_frame_offset) % 14 != 0 {
            return Err(format!("MULE animation {animation_index} has an invalid frame range"));
        }
        let frame_count = ((animation.last_frame_offset - animation.first_frame_offset) / 14 + 1) as usize;
        if frame_index >= frame_count {
            return Err(format!("MULE animation {animation_index} frame {frame_index} is out of range"));
        }
        let offset = usize::try_from(animation.first_frame_offset).map_err(|e| e.to_string())?
            .checked_add(frame_index.checked_mul(14).ok_or("MULE frame offset overflow")?)
            .ok_or("MULE frame offset overflow")?;
        let frame = metadata.frame_at(u32::try_from(offset).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        let table = SpriteTable::from_anb(&self.bytes).map_err(|e| e.to_string())?;
        let mut components = Vec::with_capacity(usize::from(frame.component_count));
        for index in 0..usize::from(frame.component_count) {
            let component = metadata.frame_component(frame, index).map_err(|e| e.to_string())?;
            let sprite_record = usize::try_from(component.sprite_offset).map_err(|e| e.to_string())? / 6;
            let sprite = table.decode_packed4(sprite_record, &self.bytes, palette_base)
                .map_err(|e| format!("MULE sprite record {sprite_record}: {e}"))?;
            components.push(CreatureFrameComponent {
                local_position: component.local_position(),
                flags: component.flags,
                sprite_record,
                sprite,
            });
        }
        Ok(CreatureFramePixels { animation: animation_index, frame_index, frame, components })
    }
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

    /// Read-only view of the sixteen already hash-validated source databases.
    pub fn databases(&self) -> &[ActorAnimationDatabase] { &self.databases }
}

/// EFF.ANB contains the animation IDs used by the bounded landing plans. The
/// typed bindings validate the resource ranges; draw behavior is implemented
/// separately from its traced wrapper contract.
pub struct EffectAnimationDatabase { bytes: Vec<u8> }

impl EffectAnimationDatabase {
    fn load(root: &Path) -> Result<Self, String> {
        let bytes = read_bound(&root.join("effects/EFF.ANB"))?;
        let value = Self { bytes };
        for (binding, expected_frames) in [
            (EffectAnimationBinding::EffAnimation49, 1),
            (EffectAnimationBinding::EffAnimation6, 13),
            (EffectAnimationBinding::EffAnimation48, 12),
            (EffectAnimationBinding::EffAnimation47, 6),
            (EffectAnimationBinding::EffAnimation26, 5),
            (EffectAnimationBinding::EffAnimation21, 2),
            (EffectAnimationBinding::EffAnimation22, 3),
            (EffectAnimationBinding::EffAnimation44, 9),
        ] {
            if value.frame_count(binding)? != expected_frames {
                return Err(format!("EFF animation {:?} has an unexpected frame count", binding));
            }
        }
        Ok(value)
    }

    pub fn bytes(&self) -> &[u8] { &self.bytes }

    pub fn animation_index(binding: EffectAnimationBinding) -> usize {
        match binding {
            EffectAnimationBinding::EffAnimation49 => 49,
            EffectAnimationBinding::EffAnimation6 => 6,
            EffectAnimationBinding::EffAnimation48 => 48,
            EffectAnimationBinding::EffAnimation47 => 47,
            EffectAnimationBinding::EffAnimation26 => 26,
            EffectAnimationBinding::EffAnimation21 => 21,
            EffectAnimationBinding::EffAnimation22 => 22,
            EffectAnimationBinding::EffAnimation44 => 44,
        }
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
    pub fn building_resource(&self,selector:u8)->Result<BuildingResourceRef<'_>,String>{
        if selector==1{return Ok(BuildingResourceRef::Bturr(&self.bturr_selector1));}
        self.source_buildings.iter().find(|r|r.selector()==selector).map(BuildingResourceRef::Source)
            .ok_or_else(||format!("unadmitted building resource selector {selector}"))
    }
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
        let font0_dat = read_bound(&root.join("panel/font0.dat"))?;
        let font0_tab = read_bound(&root.join("panel/font0.tab"))?;
        let text = read_bound(&root.join("menu/text.dat"))?;
        // Parse these at load time so malformed or mismatched panel resources
        // fail admission before any selected-actor drawing is attempted.
        let font0 = FontResource::parse(&font0_dat, &font0_tab).map_err(|e| e.to_string())?;
        if font0.record_count() < 96 { return Err("FONT0 has fewer than 96 byte-addressable glyph records".into()); }
        let text_table = TextStringTable::parse(&text).map_err(|e| e.to_string())?;
        if text_table.strings().len() != 604 { return Err("English TEXT resource did not produce the pinned 604-entry index".into()); }
        let palette = Palette256::parse(&read_bound(&root.join("terrain/MAIN.PAL"))?, PaletteChannelMode::Rgb).map_err(|e| e.to_string())?;
        if palette.entries().iter().flatten().any(|&v| v>63) { return Err("MAIN.PAL is not RGB6".into()); }
        let plants = (0..12).map(|slot| read_bound(&root.join(format!("plants/slot-{slot:02}.anb")))).collect::<Result<Vec<_>,_>>()?;
        let actors = ActorResources::load(root)?;
        let effects = EffectAnimationDatabase::load(root)?;
        let bturr_bytes = read_bound(&root.join("buildings/BTURR.ANB"))?;
        let bturr_selector1 = BuildingResourceDatabase::load_selector1_bturr(bturr_bytes)
            .map_err(|e| format!("CD BTURR selector-1 database: {e}"))?;
        let source_buildings=[("BSOLA",SourceBuildingBinding::CdBsolaSelector6),("BMINE",SourceBuildingBinding::CdBmineSelector11),("BPOWE",SourceBuildingBinding::CdBpoweSelector3)]
            .into_iter().map(|(name,binding)|SourceBuildingResourceDatabase::load(binding,read_bound(&root.join(format!("buildings/{name}.ANB")))?)
                .map_err(|e|format!("CD {name} database: {e}"))).collect::<Result<Vec<_>,String>>()?;
        let angle = read_bound(&root.join("physics/angle.bin"))?;
        let bounce = read_bound(&root.join("physics/bounce.bin"))?;
        let corpse_friction = read_bound(&root.join("physics/corpse-friction.bin"))?;
        let physics = PhysicsTables::from_mapped_bytes(&angle, &bounce).map_err(|e| e.to_string())?
            .with_corpse_friction(&corpse_friction).map_err(|e| e.to_string())?;
        let construction_border: Box<[u8; 8192]> = read_bound(&root.join("construction/border.bin"))?
            .into_boxed_slice().try_into().map_err(|_| "construction border table must be exactly 8192 bytes")?;
        let mule = CreatureAnimationDatabase::load(root)?;
        let bones = BonesAnimationDatabase::load(root)?;
        let mule_animations = MuleAnimations::from_anb(mule.bytes())
            .and_then(|animations| animations.with_bones_anb(bones.bytes()))
            .map_err(|e| format!("CD MULE/BONES animation metadata: {e}"))?;
        let ethereal_bytes = read_bound(&root.join("countdown/ETHEREAL.ANB"))?;
        let ethereal = crate::family8_scene::EtherealAnimationDatabase::from_bytes(&ethereal_bytes).map_err(|e| e.to_string())?;
        let family8_animation = genewars_runtime::family8_motion::Family8Animation::from_ethereal_anb(&ethereal_bytes).map_err(|e| format!("{e:?}"))?;
        let family8_directions = genewars_runtime::family8_motion::Family8DirectionalAnimations::from_ethereal_anb(&ethereal_bytes).map_err(|e|format!("{e:?}"))?;
        Ok((Self { _header: header, title, atlas, fade, gla, panel, font0_dat, font0_tab, text, palette, plants, actors, effects, bturr_selector1, source_buildings, physics, construction_border, mule, bones, mule_animations, ethereal, family8_animation, family8_directions }, world, target))
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

    /// Parsed views over the exact catalog-bound FONT0 and English TEXT inputs.
    pub(super) fn panel_font0(&self) -> Result<FontResource<'_>, String> {
        FontResource::parse(&self.font0_dat, &self.font0_tab).map_err(|e| e.to_string())
    }
    pub(super) fn panel_text(&self) -> Result<TextStringTable<'_>, String> {
        TextStringTable::parse(&self.text).map_err(|e| e.to_string())
    }
    pub(super) fn panel_palette(&self) -> &Palette256 { &self.palette }
}

#[cfg(test)]
mod building_resource_tests {
    use super::*;

    #[test]
    fn runtime_bundle_admits_the_catalog_bound_selector1_database() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../private/integration/game-runtime");
        let bytes = read_bound(&root.join("buildings/BTURR.ANB"))
            .expect("BTURR is registered in the runtime resource catalog");
        let database = BuildingResourceDatabase::load_selector1_bturr(bytes)
            .expect("selector 1 resolves the pinned BTURR animation database");
        let animation = database.animation0();
        assert_eq!(animation.frame_count(), 3);
        assert_eq!(animation.holds().as_slice(), &[0, 0, 0]);
        assert_eq!(animation.frames()[0].display_dimensions, (33, 106));
        assert_eq!(animation.frames()[2].components.len(), 2);
    }

    #[test]
    fn runtime_bundle_admits_the_validated_construction_border_slice() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../private/integration/game-runtime");
        let bytes = read_bound(&root.join("construction/border.bin"))
            .expect("the mapped construction border table is catalog-bound");
        assert_eq!(bytes.len(), 8192);
        assert_eq!(sha256_hex(&bytes), "9f1dcbc35c350d6027f98be0f5c8b43b42ca52b7604459c0c42be3aa88913d47");
    }

    #[test]
    fn bones_selector_database_matches_all_pinned_raw_sprite_pixels_and_masks() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../private/integration/game-runtime");
        let bytes = read_bound(&root.join("creatures/BONES.ANB"))
            .expect("exact selector-4 BONES resource is catalog-bound");
        let database = BonesAnimationDatabase::from_catalog_bytes(bytes).unwrap();
        assert_eq!(database.animation_count(), 5);
        assert_eq!(database.sprite_record_count(), 11);
        let expected: [(usize, &str, &str); 10] = [
            (1, "adb2762e279b460f548c5d1ec166cd19cfa3b7365823498f72a3bbc12fbf08dc", "0c742d6101e72146710977acc8ad63d0f86f59fbe763a477176187276e7c7ed3"),
            (2, "d61ade207c337276a99f106f469db1aff9b29a445f03a5aa54c3fad7b84a8ffc", "14ccd3e3d8114831d765a4a820951a318e1b75f819d55cff74e974084f6b6de5"),
            (3, "946d9724e8c678728c90debe0ef3a7c309aa3cd4ebc94d2bb57867ae7b349443", "916fea846ee727a585371100317a8957882980d33772ea72064138a13e7f37f2"),
            (4, "b17dd6abf4174a66c9171506080784abc5c01283c92e32ee3981a76d9585a706", "97893f58a152b94fad600e5c389f65f4deffe211bc8efaaf0fb2d3a43fa70d9a"),
            (5, "53efdcfcd2593ed696d684ab6eb53ed4e6af9d738e6b897c228fe666be3b6996", "b164948123a83c86524e172994fc50720abe9ef1f4a3ef9c8635580dd7c21f46"),
            (6, "b21d332324bb0090e69af2feedaefbb955be45a3eef01cfde0e4263c059b2424", "206f50d3e105574ada6802aee17aac24cea681ab152fe41854d3915c4ec948d9"),
            (7, "388a8c3182e91b129a5a4d74024014005dc07f90ad0bc0ad4e62ee46ec62f797", "cc46c0fe15dc42d0fc2a415cb9f352c970f0eb997b59bf30f8380c63145ef969"),
            (8, "54076d35917730b2d2ec826a7e12ac0cbd26129dea9dbf93f61d2f8ed31163c5", "6aa5be339dfcc14ad7b35fe666a49cf29f3af271b2481add63541df4d7f10391"),
            (9, "c2d43f2268adbc20c151589f77291ba28cf6a6a6364c66a6b1d0c41fd2ab587d", "b4f6fa41e6c7cb9abde87824481cd89693ec6da79eb94db9b877971a4a10df22"),
            (10, "3de4568ab234a2f375408d229feb75feddf43e85d235009c4cfa4b92d5466fcc", "e9461a79c2bd19741922a37e60f2500307d77287fbd2b34656204f7f34edeea8"),
        ];
        let mut seen = std::collections::BTreeSet::new();
        for animation in 0..database.animation_count() {
            for frame_index in 0..database.frame_count(animation).unwrap() {
                let frame = database.decode_frame_packed4(animation, frame_index, 0).unwrap();
                assert_eq!(frame.components.len(), 1);
                let component = &frame.components[0];
                seen.insert(component.sprite_record);
                let (_, expected_pixels, expected_mask) = expected.iter()
                    .find(|(record, _, _)| *record == component.sprite_record).unwrap();
                let mask = component.sprite.opaque.iter().map(|&v| u8::from(v)).collect::<Vec<_>>();
                assert_eq!(sha256_hex(&component.sprite.indices), *expected_pixels, "sprite {} pixels", component.sprite_record);
                assert_eq!(sha256_hex(&mask), *expected_mask, "sprite {} mask", component.sprite_record);
            }
        }
        assert_eq!(seen, (1usize..=10).collect());
    }

    #[test]
    fn mule_state26_metadata_requires_the_hash_bound_bones_database() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../private/integration/game-runtime");
        let mule = CreatureAnimationDatabase::load(&root).unwrap();
        let bones = BonesAnimationDatabase::load(&root).unwrap();
        let mule_only = MuleAnimations::from_anb(mule.bytes()).unwrap();
        assert!(mule_only.frame_metadata(25, 4, 0).is_ok(), "pre-BONES selector remains in MULE");
        assert!(mule_only.frame_metadata(26, 4, 0).is_err(), "state26 must not fall back to MULE animation4");

        let bound = mule_only.with_bones_anb(bones.bytes()).unwrap();
        assert!(bound.frame_metadata(26, 4, 0).is_ok(), "state26 animation4 frame0 uses BONES");
        assert!(bound.frame_metadata(27, 4, 1).is_ok(), "state27 animation4 frame1 uses BONES");
        assert!(bound.frame_metadata(26, 4, 2).is_err(), "BONES animation4 has only the admitted two frames");
        assert!(MuleAnimations::from_anb(mule.bytes()).unwrap()
            .with_bones_anb(&bones.bytes()[..bones.bytes().len() - 1]).is_err(),
            "truncated BONES cannot satisfy the separate database binding");
    }

    #[test]
    fn runtime_load_keeps_the_hash_bound_bones_database_attached_to_mule_metadata() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../private/integration/game-runtime");
        let (resources, _, _) = Resources::load(&root)
            .expect("runtime load admits the catalog-bound MULE and BONES resources");
        let metadata = resources.mule_animations.frame_metadata(26, 4, 0)
            .expect("state 26 selector 4 resolves through BONES after production loading");
        assert_eq!(metadata, (0, 0));
        assert!(resources.mule_animations.frame_metadata(26, 4, 2).is_err(),
            "the loaded BONES binding still enforces its two-frame bound");
    }
}
