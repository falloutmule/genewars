//! Explicit ActorWorld compositor over the shared terrain/plant indexed view.
//!
//! The terrain and family1 layers use the existing runtime renderer. Family2
//! actors are drawn only in caller order; no actor admission queue is known.
//! Active family4 effects are reported as pending because their draw wrapper
//! has not been admitted here.

use std::collections::HashSet;

use genewars_core::cd_candidate::entity_view_position;
use genewars_render::{
    entity_shadow::draw_normal_component_shadow, terrain_modes::MainGlaTable,
};
use genewars_runtime::{
    actor_render::{decode_actor_components_indexed8, draw_actor_components, extract_actor_render_record,
        plan_actor_draw, ActorDrawReceipt, ActorWrapperKind},
    actor_world::ActorWorld,
    identity::sha256_hex,
};

use super::runtime_resources::creature_scene::{draw_supported_bones_record, draw_supported_mule_record, CreatureDrawError, CreatureFrameReceipt};
use super::effect_scene::{draw_effect_slots, EffectSceneReceipt};
use super::building_scene::{draw_bturr_completed_common, draw_bturr_incomplete_common, BuildingSceneReceipt};
use super::runtime_resources::{Frame, Resources};

#[path = "actor_panel_text.rs"]
pub mod panel_text;

/// Draw the selected actor's source-bound generated name and category using
/// the catalog-admitted FONT0, English TEXT table, and MAIN.PAL owned by
/// `Resources`. Selection policy and panel visibility remain caller-owned.
pub fn draw_selected_actor_text(
    resources: &Resources,
    world: &ActorWorld,
    actor_slot: usize,
    image: &mut genewars_render::RgbImage,
) -> Result<panel_text::SelectedActorTextReceipt, String> {
    let font = resources.panel_font0()?;
    let text = resources.panel_text()?;
    panel_text::draw_selected_actor_details(image, &font, &text, resources.panel_palette(), world, actor_slot)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorSceneActorReceipt {
    pub actor_slot: usize,
    pub drawn: bool,
    pub projected_origin: Option<[i16; 2]>,
    pub virtual_height: i32,
    pub visible_by_wrapper: bool,
    pub shadow_required: bool,
    pub shadow_pixels_written: usize,
    pub shadow_pixels_clipped: usize,
    pub color: Option<ActorDrawReceipt>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MuleSceneGlobals {
    /// The byte at the wrapper's global shadow gate (115B5C).
    pub global_shadow_mode: u8,
    /// Caller-provided inherited 19CFAC flags. The complete draw scheduler is
    /// not modeled here, so this is never inferred from another queued item.
    pub inherited_wrapper_flags: u8,
    /// The caller-provided low-bit seed used by the packed-pixel alternation.
    pub clip_flag_bit0: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorSceneMuleReceipt {
    pub creature_slot: usize,
    pub drawn: bool,
    pub projected_ground_origin: Option<[i16; 2]>,
    pub virtual_height: i32,
    pub depth: Option<u8>,
    pub animation: u8,
    pub frame: u8,
    pub composition: Option<CreatureFrameReceipt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorSceneBuildingReceipt {
    pub building_slot: usize,
    pub projected_origin: [i16; 2],
    pub main_image: BuildingSceneReceipt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedActorPortraitReceipt {
    pub actor_slot: usize,
    pub origin: [i16; 2],
    pub visible_by_wrapper: bool,
    pub color: Option<ActorDrawReceipt>,
}

#[derive(Debug)]
pub struct ActorSceneFrame {
    pub frame: Frame,
    pub camera: [u16; 2],
    pub actor_order: Vec<usize>,
    pub actor_receipts: Vec<ActorSceneActorReceipt>,
    pub mule_receipts: Vec<ActorSceneMuleReceipt>,
    pub pending_effect_slots: Vec<usize>,
    pub effect_scene: Option<EffectSceneReceipt>,
    pub building_receipts: Vec<ActorSceneBuildingReceipt>,
    pub building_layer_policy: &'static str,
    pub family8_receipts: Vec<super::family8_scene::Family8DrawReceipt>,
    pub family8_layer_policy: &'static str,
    /// Explicit selected-actor panel portrait. Its caller controls whether a
    /// selection is active; the source-backed draw origin is the panel call's
    /// signed coordinate pair (25,42), on the full 640Ãƒâ€”480 framebuffer.
    pub selected_portrait: Option<SelectedActorPortraitReceipt>,
    pub actor_order_policy: &'static str,
    pub interfamily_layer_policy: &'static str,
}

/// Build terrain, queued plants, and then family2 type3 actors on one indexed
/// framebuffer. `actor_order` is explicit caller policy because no family2
/// queue ordering contract has been proved. Effects are retained in the world
/// and surfaced by slot; they are not drawn by this function.
pub fn render_actor_world(
    resources: &Resources,
    world: &ActorWorld,
    target: [i16; 2],
    selected_player: u8,
    frame_counter: u8,
    actor_order: &[usize],
) -> Result<ActorSceneFrame, String> {
    render_actor_world_layers(resources, world, target, selected_player, frame_counter, actor_order, None, None, None, None)
}

/// Render the caller-selected family-2 order, followed by the caller-selected
/// family-4 order. This ordering is an explicit development policy; the game
/// scheduler's interfamily draw ordering remains unproved.
pub fn render_actor_world_with_effects(
    resources: &Resources,
    world: &ActorWorld,
    target: [i16; 2],
    selected_player: u8,
    frame_counter: u8,
    actor_order: &[usize],
    effect_order: &[usize],
) -> Result<ActorSceneFrame, String> {
    render_actor_world_layers(resources, world, target, selected_player, frame_counter,
        actor_order, None, None, Some(effect_order), None)
}

/// Render the caller-selected family-2 actor order, then the selected MULE
/// order. The latter is an explicit preview policy; no family-2/family-0 game
/// queue ordering is inferred. Wrapper globals are supplied by the caller.
pub fn render_actor_world_with_creatures(
    resources: &Resources,
    world: &ActorWorld,
    target: [i16; 2],
    selected_player: u8,
    frame_counter: u8,
    actor_order: &[usize],
    creature_order: &[usize],
    mule_globals: MuleSceneGlobals,
) -> Result<ActorSceneFrame, String> {
    render_actor_world_layers(resources, world, target, selected_player, frame_counter,
        actor_order, Some(creature_order), Some(mule_globals), None, None)
}

/// As above, with a caller-selected family-4 order after the MULE order. This
/// layer sequence is an explicit development policy, not the game's
/// interfamily draw ordering.
pub fn render_actor_world_with_creatures_and_effects(
    resources: &Resources,
    world: &ActorWorld,
    target: [i16; 2],
    selected_player: u8,
    frame_counter: u8,
    actor_order: &[usize],
    creature_order: &[usize],
    mule_globals: MuleSceneGlobals,
    effect_order: &[usize],
) -> Result<ActorSceneFrame, String> {
    render_actor_world_layers(resources, world, target, selected_player, frame_counter,
        actor_order, Some(creature_order), Some(mule_globals), Some(effect_order), None)
}

/// As above, with an explicitly selected type-3 actor drawn through the
/// verified direct portrait wrapper after world layers. Selection is supplied
/// by the caller; this does not invent panel state or draw ordering.
pub fn render_actor_world_with_selected_portrait(
    resources: &Resources,
    world: &ActorWorld,
    target: [i16; 2],
    selected_player: u8,
    frame_counter: u8,
    actor_order: &[usize],
    creature_order: &[usize],
    mule_globals: MuleSceneGlobals,
    effect_order: &[usize],
    selected_actor_slot: Option<usize>,
) -> Result<ActorSceneFrame, String> {
    render_actor_world_layers(resources, world, target, selected_player, frame_counter,
        actor_order, Some(creature_order), Some(mule_globals), Some(effect_order), selected_actor_slot)
}

fn render_actor_world_layers(
    resources: &Resources,
    world: &ActorWorld,
    target: [i16; 2],
    selected_player: u8,
    frame_counter: u8,
    actor_order: &[usize],
    creature_order: Option<&[usize]>,
    mule_globals: Option<MuleSceneGlobals>,
    effect_order: Option<&[usize]>,
    selected_actor_slot: Option<usize>,
) -> Result<ActorSceneFrame, String> {
    if selected_player >= 4 { return Err("selected player must be in 0..4".into()); }
    if actor_order.len() > 20 { return Err("actor draw order exceeds the twenty-slot pool".into()); }
    if creature_order.is_some_and(|order| order.len() > 300) { return Err("creature draw order exceeds the three-hundred-slot pool".into()); }
    if effect_order.is_some_and(|order| order.len() > 200) { return Err("effect draw order exceeds the two-hundred-slot pool".into()); }
    let mut unique = HashSet::with_capacity(actor_order.len());
    for &slot in actor_order {
        if slot >= 20 { return Err(format!("actor slot {slot} is out of range")); }
        if !unique.insert(slot) { return Err(format!("actor slot {slot} occurs more than once in draw order")); }
    }
    let mut unique_creatures = HashSet::with_capacity(creature_order.map_or(0, <[usize]>::len));
    for &slot in creature_order.unwrap_or_default() {
        if slot >= 300 { return Err(format!("creature slot {slot} is out of range")); }
        if !unique_creatures.insert(slot) { return Err(format!("creature slot {slot} occurs more than once in draw order")); }
    }
    if creature_order.is_some() != mule_globals.is_some() {
        return Err("MULE draw order and explicit wrapper globals must be supplied together".into());
    }

    let plants = world.plant_render_records().map_err(|e| e.to_string())?;
    let mut indexed = resources.render_indexed(world.backing_bytes(), &plants, target)?;
    let camera = indexed.camera;
    let gla = MainGlaTable::parse(resources.main_gla_bytes()).map_err(|e| e.to_string())?;
    let mut actor_receipts = Vec::with_capacity(actor_order.len());
    for &slot in actor_order {
        let record_bytes = world.actors().record(slot).map_err(|e| e.to_string())?;
        let actor_type = record_bytes[6];
        if actor_type == 0 {
            actor_receipts.push(ActorSceneActorReceipt {
                actor_slot: slot, drawn: false, projected_origin: None, virtual_height: 0,
                visible_by_wrapper: false, shadow_required: false, shadow_pixels_written: 0,
                shadow_pixels_clipped: 0, color: None,
            });
            continue;
        }
        if actor_type != 3 {
            return Err(format!("active actor slot {slot} has unsupported family/type {actor_type}"));
        }
        let key = world.actors().bindings(slot).map_err(|e| e.to_string())?
            .animation_database.ok_or_else(|| format!("actor slot {slot} has no animation database binding"))?;
        let database = resources.actors.database(key)?;
        let record = extract_actor_render_record(world.actors(), slot, &genewars_runtime::actor_resources::ActorResourceCatalog, database)
            .map_err(|e| format!("actor slot {slot}: {e}"))?;
        let player = world.player(record.owner).map_err(|e| e.to_string())?;
        let point = entity_view_position(world.map(), record.world_position, camera, 128).map_err(|e| e.to_string())?;
        let screen_origin = [
            i16::try_from(point[0]).map_err(|_| format!("actor slot {slot} projected X is outside signed-word draw range"))?,
            i16::try_from(point[1]).map_err(|_| format!("actor slot {slot} projected Y is outside signed-word draw range"))?,
        ];
        let policy = plan_actor_draw(&record, player, selected_player, frame_counter, screen_origin,
            record.virtual_height, ActorWrapperKind::World).map_err(|e| format!("actor slot {slot}: {e}"))?;
        if !policy.visible {
            actor_receipts.push(ActorSceneActorReceipt {
                actor_slot: slot, drawn: false, projected_origin: Some(screen_origin),
                virtual_height: record.virtual_height, visible_by_wrapper: false,
                shadow_required: policy.shadow_required, shadow_pixels_written: 0,
                shadow_pixels_clipped: 0, color: None,
            });
            continue;
        }

        let decoded = decode_actor_components_indexed8(&record, database)
            .map_err(|e| format!("actor slot {slot}: {e}"))?;
        let mut shadow_pixels_written = 0usize;
        let mut shadow_pixels_clipped = 0usize;
        if policy.shadow_required {
            for component in &decoded {
                let (x, y) = component.component.local_position();
                let receipt = draw_normal_component_shadow(&component.sprite, &gla, screen_origin, [x, y], 0,
                    [512, 480], 640, &mut indexed.indices[128..]).map_err(|e| format!("actor slot {slot} shadow: {e}"))?;
                shadow_pixels_written = shadow_pixels_written.checked_add(receipt.pixels_written)
                    .ok_or("actor shadow pixel count overflow")?;
                shadow_pixels_clipped = shadow_pixels_clipped.checked_add(receipt.clipped_opaque_samples)
                    .ok_or("actor shadow clipped count overflow")?;
            }
        }
        let fade_row = if let Some(row) = policy.fade_row {
            let start = usize::try_from(row).map_err(|_| format!("actor slot {slot} has negative fade row"))?
                .checked_mul(256).ok_or("actor fade row offset overflow")?;
            Some(resources.fade_bytes().get(start..start + 256)
                .ok_or_else(|| format!("actor slot {slot} fade row {row} is outside FADE.DAT"))?)
        } else { None };
        let color = draw_actor_components(&record, &decoded, policy.context, fade_row,
            Some(resources.main_gla_bytes()), &mut indexed.indices[128..], 512, 480, 640)
            .map_err(|e| format!("actor slot {slot} color: {e}"))?;
        actor_receipts.push(ActorSceneActorReceipt {
            actor_slot: slot, drawn: true, projected_origin: Some(screen_origin),
            virtual_height: record.virtual_height, visible_by_wrapper: true,
            shadow_required: policy.shadow_required, shadow_pixels_written,
            shadow_pixels_clipped, color: Some(color),
        });
    }

    let mut mule_receipts = Vec::with_capacity(creature_order.map_or(0, <[usize]>::len));
    if let Some(order) = creature_order {
        let pool = world.creatures().ok_or("family0 creature pool has not been admitted")?;
        let globals = mule_globals.expect("validated with creature order");
        let shadow_color = black_palette_index(resources)?;
        for &slot in order {
            let record = pool.record(slot).map_err(|e| e.to_string())?;
            let animation = record[0x19];
            let frame_index = record[0x1b];
            if record[6] == 0 {
                mule_receipts.push(ActorSceneMuleReceipt {
                    creature_slot: slot, drawn: false, projected_ground_origin: None,
                    virtual_height: 0, depth: None, animation, frame: frame_index,
                    composition: None,
                });
                continue;
            }
            let binding = pool.binding(slot).map_err(|e| format!("{e:?}"))?;
            let virtual_height = i32::from_le_bytes(record[0x27..0x2b].try_into().unwrap())
                .wrapping_sub(i32::from_le_bytes(record[0x11..0x15].try_into().unwrap()));
            let depth = if binding == Some(genewars_runtime::creature_world::CreatureAnimationBinding::MuleSelector4) {
                let health = i32::from(i16::from_le_bytes([record[0x6a], record[0x6b]]));
                let maximum = i32::from(i16::from_le_bytes([record[0x64], record[0x65]]));
                if maximum == 0 { return Err(format!("creature slot {slot} has zero health maximum for wrapper depth")); }
                let quotient = health.checked_mul(8).ok_or("MULE depth numerator overflow")? / maximum;
                let depth_value = 8i32.checked_sub(quotient).ok_or("MULE wrapper depth overflow")?;
                u8::try_from(depth_value).map_err(|_| format!("creature slot {slot} wrapper depth {depth_value} is outside byte range"))?
            } else if binding == Some(genewars_runtime::creature_world::CreatureAnimationBinding::BonesSelector4) {
                if !matches!(record[0x5e], 26 | 27) { return Err(format!("creature slot {slot} BONES binding is outside state 26/27")); }
                0
            } else {
                return Err(format!("creature slot {slot} has no admitted MULE or BONES animation binding"));
            };
            let world_position = [
                i16::from_le_bytes([record[7], record[8]]),
                i16::from_le_bytes([record[9], record[10]]),
            ];
            let point = entity_view_position(world.map(), world_position, camera, 128)
                .map_err(|e| format!("creature slot {slot} projection: {e}"))?;
            let ground_origin = [
                i16::try_from(point[0]).map_err(|_| format!("creature slot {slot} projected X is outside signed-word range"))?,
                i16::try_from(point[1]).map_err(|_| format!("creature slot {slot} projected Y is outside signed-word range"))?,
            ];
            let projected = [i32::from(ground_origin[0]), i32::from(ground_origin[1])];
            let composition = match binding {
                Some(genewars_runtime::creature_world::CreatureAnimationBinding::MuleSelector4) =>
                    draw_supported_mule_record(&resources.mule, record, projected,
                        virtual_height, globals.global_shadow_mode, globals.inherited_wrapper_flags,
                        globals.clip_flag_bit0, shadow_color, depth, resources.fade_bytes(),
                        resources.main_gla_bytes(), [512, 480], 640, &mut indexed.indices[128..]),
                Some(genewars_runtime::creature_world::CreatureAnimationBinding::BonesSelector4) =>
                    draw_supported_bones_record(&resources.bones, record, projected,
                        virtual_height, globals.global_shadow_mode, globals.inherited_wrapper_flags,
                        globals.clip_flag_bit0, shadow_color, resources.fade_bytes(),
                        resources.main_gla_bytes(), [512, 480], 640, &mut indexed.indices[128..]),
                None => unreachable!("binding admitted above"),
            }
                .map_err(|error| format!("creature slot {slot}: {}", mule_draw_error(error)))?;
            mule_receipts.push(ActorSceneMuleReceipt {
                creature_slot: slot, drawn: true, projected_ground_origin: Some(ground_origin),
                virtual_height, depth: Some(depth), animation, frame: frame_index,
                composition: Some(composition),
            });
        }
    }

    let effect_scene = if let Some(order) = effect_order {
        let pool = world.effects().ok_or("family4 effect pool has not been admitted")?;
        let mut unique = HashSet::with_capacity(order.len());
        let mut origins = vec![[0i16; 2]; pool.records().len()];
        for &slot in order {
            if slot >= pool.records().len() { return Err(format!("effect slot {slot} is out of range")); }
            if !unique.insert(slot) { return Err(format!("effect slot {slot} occurs more than once in draw order")); }
            let record = pool.record(slot).map_err(|e| e.to_string())?;
            let position = [
                i16::from_le_bytes([record[7], record[8]]),
                i16::from_le_bytes([record[9], record[10]]),
            ];
            let point = entity_view_position(world.map(), position, camera, 128).map_err(|e| e.to_string())?;
            origins[slot] = [
                i16::try_from(point[0]).map_err(|_| format!("effect slot {slot} projected X is outside signed-word draw range"))?,
                i16::try_from(point[1]).map_err(|_| format!("effect slot {slot} projected Y is outside signed-word draw range"))?,
            ];
        }
        Some(draw_effect_slots(resources, pool, order, &origins, frame_counter, 0,
            &mut indexed.indices[128..], [512,480], 640).map_err(|e| e.to_string())?)
    } else { None };
    let drawn_effects: HashSet<usize> = effect_order.unwrap_or_default().iter().copied().collect();
    let pending_effect_slots = world.effects().map(|pool| pool.records().iter().enumerate()
        .filter_map(|(slot, record)| (record[6] != 0 && !drawn_effects.contains(&slot)).then_some(slot)).collect()).unwrap_or_default();
    let mut family8_receipts = Vec::new();
    if let Some(pool) = world.family8() {
        let fade = genewars_render::terrain::TerrainFadeTable::parse(resources.fade_bytes()).map_err(|e| e.to_string())?;
        for (slot, record) in pool.records().iter().enumerate().filter(|(_, r)| r[6] != 0) {
            if pool.binding(slot).map_err(|e| format!("{e:?}"))?
                != Some(if record[0x55]==1 {genewars_runtime::family8_pool::AnimationBinding::Base}else{genewars_runtime::family8_pool::AnimationBinding::Offset80}) {
                return Err(format!("family8 slot {slot} has an unsupported animation binding"));
            }
            let position = [i16::from_le_bytes([record[7],record[8]]),i16::from_le_bytes([record[9],record[10]])];
            let projected = entity_view_position(world.map(), position, camera, 128).map_err(|e| e.to_string())?;
            let origin = [i16::try_from(projected[0]).map_err(|_| "family8 projected X exceeds signed word")?,
                i16::try_from(projected[1]).map_err(|_| "family8 projected Y exceeds signed word")?];
            let draw=match (record[0x55],record[0x56]) {
                (0,4)=>super::family8_scene::draw_family8_selector0_state4,
                (0,5)=>super::family8_scene::draw_family8_selector0_state5,
                (0,8)=>super::family8_scene::draw_family8_selector0_state8,
                (0,9)=>super::family8_scene::draw_family8_selector0_state9,
                (1,0)=>super::family8_scene::draw_family8_selector1_state0,
                (1,1)=>super::family8_scene::draw_family8_selector1_state1,
                (1,3)=>super::family8_scene::draw_family8_selector1_state3,
                state=>return Err(format!("family8 slot {slot}: unsupported draw state {state:?}")),
            };
            family8_receipts.push(draw(&resources.ethereal,record,origin,&fade,&gla,&mut indexed.indices[128..],[512,480],640)
                .map_err(|e|format!("family8 slot {slot}: {e}"))?);
        }
    }
    // Selector-1 animations 0 and 1 use the common source 43A40 main-image
    // path. Completion flags control later overlays, not this image pass.
    // Projection and ascending pool-slot
    // order are explicit development policies; shadow and post-image owner/
    // status overlays remain omitted. Empty pools add no pixels.
    let mut building_receipts = Vec::new();
    if let Some(pool) = world.buildings() {
        for (slot, record) in pool.records().iter().enumerate().filter(|(_, r)| r[6] != 0) {
            let binding = pool.binding(slot).map_err(|e| format!("{e:?}"))?;
            if binding.map(|v| v.selector) != Some(1) || record[0x2d] != 1 {
                return Err(format!("building slot {slot} has unsupported selector binding {binding:?}"));
            }
            let position = [
                i16::from_le_bytes([record[7], record[8]]),
                i16::from_le_bytes([record[9], record[10]]),
            ];
            let projected = entity_view_position(world.map(), position, camera, 128)
                .map_err(|e| format!("building slot {slot} projection: {e}"))?;
            let origin = [
                i16::try_from(projected[0]).map_err(|_| format!("building slot {slot} projected X is outside signed-word range"))?,
                i16::try_from(projected[1]).map_err(|_| format!("building slot {slot} projected Y is outside signed-word range"))?,
            ];
            let fade_row = if record[0x15] & 0x02 == 0 && record[0x17] < 30 {
                let start = usize::from(record[0x17]) * 256;
                Some(resources.fade_bytes().get(start..start + 256)
                    .ok_or_else(|| format!("building slot {slot} fade row is outside FADE.DAT"))?)
            } else { None };
            let main_image = if record[0x19] == 1 {
                draw_bturr_completed_common(
                    &resources.bturr_selector1, record, origin, 0, fade_row,
                    Some(resources.main_gla_bytes()), &mut indexed.indices[128..], 512, 480, 640,
                )
            } else {
                draw_bturr_incomplete_common(
                    &resources.bturr_selector1, record, origin, 0, fade_row,
                    Some(resources.main_gla_bytes()), &mut indexed.indices[128..], 512, 480, 640,
                )
            }.map_err(|e| format!("building slot {slot}: {e}; candidate_record={}",record.iter().map(|b|format!("{b:02x}")).collect::<String>()))?;
            building_receipts.push(ActorSceneBuildingReceipt { building_slot: slot, projected_origin: origin, main_image });
        }
    }
    let selected_portrait = if let Some(slot) = selected_actor_slot {
        Some(draw_selected_actor_portrait(resources, world, slot, selected_player, frame_counter,
            &mut indexed.indices)?)
    } else { None };
    let snapshot_sha256 = actor_world_snapshot_sha256(world);
    let frame = resources.finish_indexed(indexed, snapshot_sha256)?;
    Ok(ActorSceneFrame { frame, camera, actor_order: actor_order.to_vec(), actor_receipts, mule_receipts,
        pending_effect_slots, effect_scene, building_receipts,
        building_layer_policy: "active selector-1 buildings ascending slot after family8; admitted BTURR common main images for animation 0 and 1, shadow/overlays omitted",
        selected_portrait, family8_receipts,
        family8_layer_policy: "active native slots ascending after effects; source interfamily order remains unproved",
        actor_order_policy: "caller-supplied; family2 queue order unproved",
        interfamily_layer_policy: match (creature_order.is_some(), effect_order.is_some()) {
            (true, true) => "caller-supplied family2 order, then family0 MULE order, then family4 order; game interfamily order unproved",
            (true, false) => "caller-supplied family2 order, then family0 MULE order; game interfamily order unproved",
            (false, true) => "caller-supplied family2 order, then family4 order; game interfamily order unproved",
            (false, false) => "family0/family4 not drawn",
        } })
}

fn draw_selected_actor_portrait(
    resources: &Resources,
    world: &ActorWorld,
    slot: usize,
    selected_player: u8,
    frame_counter: u8,
    framebuffer: &mut [u8],
) -> Result<SelectedActorPortraitReceipt, String> {
    let record_bytes = world.actors().record(slot).map_err(|e| e.to_string())?;
    if record_bytes[6] != 3 {
        return Err(format!("selected portrait slot {slot} is not an admitted type-3 actor"));
    }
    let key = world.actors().bindings(slot).map_err(|e| e.to_string())?
        .animation_database.ok_or_else(|| format!("selected portrait slot {slot} has no animation database binding"))?;
    let database = resources.actors.database(key)?;
    let record = extract_actor_render_record(world.actors(), slot,
        &genewars_runtime::actor_resources::ActorResourceCatalog, database)
        .map_err(|e| format!("selected portrait slot {slot}: {e}"))?;
    let player = world.player(record.owner).map_err(|e| e.to_string())?;
    // In 265F0/27F20 the caller loads BX=0x2A and DX=0x19. The 57050
    // wrapper forwards DX as X and BX as Y and disables the world shadow/Y
    // displacement while calling the actor's virtual +0x24 draw.
    let origin = [25, 42];
    let policy = plan_actor_draw(&record, player, selected_player, frame_counter,
        origin, record.virtual_height, ActorWrapperKind::PortraitDirect)
        .map_err(|e| format!("selected portrait slot {slot}: {e}"))?;
    if !policy.visible {
        return Ok(SelectedActorPortraitReceipt { actor_slot: slot, origin,
            visible_by_wrapper: false, color: None });
    }
    let decoded = decode_actor_components_indexed8(&record, database)
        .map_err(|e| format!("selected portrait slot {slot}: {e}"))?;
    let fade = if let Some(row) = policy.fade_row {
        let start = usize::try_from(row).map_err(|_| "selected portrait has a negative fade row")?
            .checked_mul(256).ok_or("selected portrait fade offset overflow")?;
        Some(resources.fade_bytes().get(start..start + 256)
            .ok_or_else(|| format!("selected portrait fade row {row} is outside FADE.DAT"))?)
    } else { None };
    let color = draw_actor_components(&record, &decoded, policy.context, fade,
        Some(resources.main_gla_bytes()), framebuffer, 640, 480, 640)
        .map_err(|e| format!("selected portrait slot {slot}: {e}"))?;
    Ok(SelectedActorPortraitReceipt { actor_slot: slot, origin,
        visible_by_wrapper: true, color: Some(color) })
}

fn black_palette_index(resources: &Resources) -> Result<u8, String> {
    resources.palette_entries().iter().position(|rgb| *rgb == [0, 0, 0])
        .map(|index| index as u8)
        .ok_or_else(|| "MAIN.PAL has no exact black entry for the source nearest-black shadow color".into())
}

fn mule_draw_error(error: CreatureDrawError) -> String { format!("{error:?}") }

fn actor_world_snapshot_sha256(world: &ActorWorld) -> String {
    let mut bytes = Vec::with_capacity(world.backing_bytes().len() + 20 * 200 + 4 * 0x70d + 200 * 81 + 300 * 125 + 32);
    bytes.extend_from_slice(world.backing_bytes());
    for slot in 0..20 {
        if let Ok(record) = world.actors().record(slot) { bytes.extend_from_slice(record); }
        if let Ok(bindings) = world.actors().bindings(slot) {
            if let Some(handle) = bindings.blueprint {
                bytes.extend_from_slice(&[1, handle.player_id, handle.blueprint_index]);
            } else { bytes.extend_from_slice(&[0, 0, 0]); }
            for key in [bindings.resource_database, bindings.animation_database] {
                if let Some(key) = key { bytes.extend_from_slice(&[1, key.style, key.category, key.flat_index]); }
                else { bytes.extend_from_slice(&[0, 0, 0, 0]); }
            }
            if let Some(offset) = bindings.internal_actor_offset { bytes.extend_from_slice(&[1, offset]); }
            else { bytes.extend_from_slice(&[0, 0]); }
        } else { bytes.extend_from_slice(&[0; 13]); }
    }
    for player in 0..4 {
        if let Ok(record) = world.player(player) {
            bytes.extend_from_slice(record.bytes());
            bytes.push(match record.origin() {
                genewars_runtime::player_state::PlayerStateOrigin::ExplicitRecord => 0,
                genewars_runtime::player_state::PlayerStateOrigin::FreshCampaign => 1,
                genewars_runtime::player_state::PlayerStateOrigin::RestoredForMission => 2,
                genewars_runtime::player_state::PlayerStateOrigin::PreparedFreshActors => 3,
            });
        }
    }
    if let Some(effects) = world.effects() {
        bytes.push(1);
        for (slot, record) in effects.records().iter().enumerate() {
            bytes.extend_from_slice(record);
            bytes.push(match effects.binding(slot).ok().flatten() {
                None => 0,
                Some(genewars_runtime::effect_pool::EffectAnimationBinding::EffAnimation49) => 1,
                Some(genewars_runtime::effect_pool::EffectAnimationBinding::EffAnimation6) => 2,
                Some(genewars_runtime::effect_pool::EffectAnimationBinding::EffAnimation47) => 3,
                Some(genewars_runtime::effect_pool::EffectAnimationBinding::EffAnimation48) => 4,
                Some(genewars_runtime::effect_pool::EffectAnimationBinding::EffAnimation26) => 5,
                Some(genewars_runtime::effect_pool::EffectAnimationBinding::EffAnimation21) => 6,
                Some(genewars_runtime::effect_pool::EffectAnimationBinding::EffAnimation22) => 7,
                Some(genewars_runtime::effect_pool::EffectAnimationBinding::EffAnimation44) => 8,
            });
        }
    } else { bytes.push(0); }
    if let Some(buildings) = world.buildings() {
        bytes.push(1);
        for (slot, record) in buildings.records().iter().enumerate() {
            bytes.extend_from_slice(record);
            bytes.push(match buildings.binding(slot).ok().flatten() {
                None => 0,
                Some(genewars_runtime::building::BuildingAnimationBinding { selector: 1 }) => 1,
                Some(genewars_runtime::building::BuildingAnimationBinding { selector }) => selector.wrapping_add(2),
            });
        }
    } else { bytes.push(0); }
    // Family8 is also an owned overlay once admitted. Persist its logical
    // animation binding in the snapshot identity, never a host address.
    if let Some(pool) = world.family8() {
        bytes.push(1);
        for (slot, record) in pool.records().iter().enumerate() {
            bytes.extend_from_slice(record);
            bytes.push(match pool.binding(slot).ok().flatten() {
                None => 0,
                Some(genewars_runtime::family8_pool::AnimationBinding::Base) => 1,
                Some(genewars_runtime::family8_pool::AnimationBinding::Offset80) => 2,
            });
        }
    } else { bytes.push(0); }
    // Packed source bytes stop being authoritative after a creature pool is
    // admitted. Include both native after-images and each logical animation
    // binding so distinct runtime states cannot share a snapshot identity.
    if let Some(creatures) = world.creatures() {
        bytes.push(1);
        for (slot, record) in creatures.records().iter().enumerate() {
            bytes.extend_from_slice(record);
            bytes.push(match creatures.binding(slot).ok().flatten() {
                None => 0,
                Some(genewars_runtime::creature_world::CreatureAnimationBinding::MuleSelector4) => 1,
                Some(genewars_runtime::creature_world::CreatureAnimationBinding::BonesSelector4) => 9,
            });
        }
    } else { bytes.push(0); }
    let rng = world.rng();
    bytes.extend_from_slice(&rng.seed.to_le_bytes());
    bytes.push(rng.ring);
    bytes.extend_from_slice(&rng.draw_count.to_le_bytes());
    if let Some(tick) = world.plant_tick_number() { bytes.push(1); bytes.extend_from_slice(&tick.to_le_bytes()); }
    else { bytes.push(0); }
    sha256_hex(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use genewars_assets::ConqMissionSource;
    use super::super::shared_scene::SharedMission;
    use genewars_runtime::{
        actor_activation::TrigTables,
        actor_pool::ActorHandleBindings,
        actor_world::{ActorWorld, PlacementEnvironment},
        blueprint::BlueprintRng,
        player_state::{ActorOriginSample, PlayerState},
    };
    use std::{fs, path::PathBuf};

    fn bundle() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .join("private/integration/game-runtime")
    }

    fn direct_color_oracle(
        record: &genewars_runtime::actor_render::ActorRenderRecord,
        components: &[genewars_runtime::actor_render::ActorDecodedComponent],
        context: genewars_runtime::actor_render::ActorDrawContext,
        fade_row: Option<&[u8]>,
        gla: &[u8],
        initial: &[u8],
    ) -> Vec<u8> {
        let mut expected = initial.to_vec();
        let bias = record.owner.wrapping_mul(16).wrapping_sub(64);
        let use_fade = record.actor_flags_15 & 2 == 0 && record.shade < 30;
        for item in components {
            let sprite = &item.sprite;
            let mut component_flags = if context.global_flags & 3 == 0 {
                context.global_flags | item.component.flags
            } else {
                (context.global_flags & 0xfc) | item.component.flags
            };
            let (component_x, component_y) = item.component.local_position();
            let mut ox = i32::from(component_x);
            let mut oy = i32::from(component_y);
            if context.global_flags & 1 != 0 {
                ox = -i32::from(sprite.width) - ox;
                component_flags ^= 1;
            }
            if context.global_flags & 2 != 0 {
                oy = -i32::from(sprite.height) - oy;
                component_flags ^= 2;
            }
            let left = i64::from(context.origin[0]) + i64::from(ox);
            let top = i64::from(context.origin[1]) + i64::from(oy);
            for dy in 0..usize::from(sprite.height) {
                for dx in 0..usize::from(sprite.width) {
                    let sx = if component_flags & 1 != 0 { usize::from(sprite.width) - 1 - dx } else { dx };
                    let sy = if component_flags & 2 != 0 { usize::from(sprite.height) - 1 - dy } else { dy };
                    let source_at = sy * usize::from(sprite.width) + sx;
                    if !sprite.opaque[source_at] { continue; }
                    let x = left + dx as i64;
                    let y = top + dy as i64;
                    if !(0..640).contains(&x) || !(0..480).contains(&y) { continue; }
                    let mut source = sprite.indices[source_at];
                    if (0xc0..=0xcf).contains(&source) {
                        source = bias.wrapping_add(source & 0x0f);
                    }
                    if use_fade { source = fade_row.expect("source fade row")[source as usize]; }
                    let at = y as usize * 640 + x as usize;
                    match component_flags & 0x0c {
                        0 => expected[at] = source,
                        4 => expected[at] = gla[usize::from(expected[at]) * 256 + usize::from(source)],
                        8 | 12 => expected[at] = gla[usize::from(source) * 256 + usize::from(expected[at])],
                        other => panic!("unexpected admitted blend flags {other:#x}"),
                    }
                }
            }
        }
        expected
    }

    fn explicit_fixture_world(resources_dir: &std::path::Path) -> (ActorWorld, [i16; 2]) {
        let mission_bytes = fs::read(resources_dir.join("terrain/CONQ001.DAT")).unwrap();
        let source = ConqMissionSource::parse(&mission_bytes).unwrap();
        let target = source.target_words(0).unwrap().map(|v| v as i16);
        let mut players = core::array::from_fn(|id| {
            PlayerState::fresh_campaign(id as u8, target, b"fixture\0").unwrap()
        });
        let mut player = *players[0].bytes();
        for (index, category) in [(0usize, 0u8), (10, 2)] {
            let at = 0x16 + index * 36;
            player[at] = b'X';
            player[at + 0x15] = (index as u8) & 3;
            player[at + 0x16] = category;
            player[at + 0x18] = index as u8;
            player[at + 0x1B] = 0xA5;
            player[at + 0x1C..at + 0x20].copy_from_slice(&u32::MAX.to_le_bytes());
        }
        players[0] = PlayerState::from_record(&player).unwrap();
        let prior = core::array::from_fn(|slot| {
            core::array::from_fn(|byte| ((slot * 37 + byte * 19) & 255) as u8)
        });
        let mut world = ActorWorld::from_mission(
            &source,
            &prior,
            [ActorHandleBindings::empty(); 20],
            players,
            BlueprintRng { seed: 5428, ring: 7, draw_count: 0 },
        ).unwrap();
        world.prepare_fresh_player(0, 5, ActorOriginSample { position: [0, 0], height: 0, shade: 0 }, target).unwrap();
        let tables = TrigTables::generate_pinned().unwrap();
        let mut query_counts = [0usize; 5];
        let batch = world.activate_without_associated_unit(0, &tables, |_, slot, _| {
            query_counts[slot] += 1;
            Ok(PlacementEnvironment {
                terrain_class: if query_counts[slot] == 1 { 0 } else { 1 },
                blocker_weight: 9,
            })
        }).unwrap();
        assert!(!batch.activations.is_empty(), "fixture must produce active actor records");
        (world, target)
    }

    #[test]
    fn real_terrain_plant_actor_pipeline_matches_legacy_without_actors_then_draws_fixture_actors() {
        let root = bundle();
        let (resources, legacy_world, target) = Resources::load(&root).unwrap();
        let legacy_frame = resources.render(&legacy_world, target).unwrap();
        let (actor_world, actor_target) = explicit_fixture_world(&root);
        assert_eq!(actor_target, target);

        let empty = render_actor_world(&resources, &actor_world, target, 0, 0, &[]).unwrap();
        assert_eq!(empty.frame.image, legacy_frame.image, "an empty actor order must preserve the shared legacy terrain/plant pixels");
        assert!(empty.actor_receipts.is_empty());

        let active = actor_world.actors().records().iter().enumerate()
            .filter_map(|(slot, record)| (record[6] == 3).then_some(slot)).collect::<Vec<_>>();
        assert!(!active.is_empty());
        let drawn = render_actor_world(&resources, &actor_world, target, 0, 0, &active).unwrap();
        assert!(drawn.actor_receipts.iter().any(|receipt| receipt.drawn && receipt.color.as_ref().is_some_and(|color| color.pixels_written > 0)),
            "at least one active actor must exercise its indexed8 color path against the real map");
        assert_ne!(drawn.frame.image, legacy_frame.image, "the explicit actor layer should alter rendered pixels");
        assert_eq!(actor_world.actors().records().iter().filter(|record| record[6] == 3).count(), active.len());
    }

    #[test]
    fn active_selector1_building_is_projected_and_drawn_as_main_image_only() {
        let root = bundle();
        let (resources, _, target) = Resources::load(&root).unwrap();
        let mission_bytes = fs::read(root.join("terrain/CONQ001.DAT")).unwrap();
        let mut shared = SharedMission::load(&root, &mission_bytes, 5428).unwrap();

        let empty = render_actor_world(&resources, &shared.world, target, 0, 0, &[]).unwrap();
        assert!(empty.building_receipts.is_empty());
        let factory = shared.world.create_building(0, 1, [19839, 12927]).unwrap()
            .expect("source factory admits the selector-1 building at the proved site");
        let built = render_actor_world(&resources, &shared.world, target, 0, 0, &[]).unwrap();
        assert_eq!(built.building_receipts.len(), 1);
        let receipt = &built.building_receipts[0];
        assert_eq!(receipt.building_slot, usize::from(factory.slot));
        assert!(receipt.main_image.main_image.color.pixels_written > 0);
        assert!(!receipt.main_image.omissions.shadow_pass_included);
        assert!(!receipt.main_image.omissions.post_image_overlays_included);
        assert_eq!(built.building_layer_policy,
            "active selector-1 buildings ascending slot after family8; admitted BTURR common main images for animation 0 and 1, shadow/overlays omitted");
        assert_ne!(built.frame.image.rgb_bytes(), empty.frame.image.rgb_bytes(),
            "an on-screen factory record must change the actual indexed-to-RGB frame");
    }

    #[test]
    fn selected_actor_portrait_uses_fullframe_panel_origin_and_direct_wrapper() {
        let root = bundle();
        let (resources, _, target) = Resources::load(&root).unwrap();
        let (world, _) = explicit_fixture_world(&root);
        let slot = world.actors().records().iter().enumerate()
            .find_map(|(slot, record)| (record[6] == 3 && record[24] == 0).then_some(slot))
            .expect("fixture has a player-zero actor");
        let plants = world.plant_render_records().unwrap();
        let mut indexed = resources.render_indexed(world.backing_bytes(), &plants, target).unwrap();
        let before = indexed.indices.clone();
        let key = world.actors().bindings(slot).unwrap().animation_database.unwrap();
        let database = resources.actors.database(key).unwrap();
        let record = extract_actor_render_record(world.actors(), slot,
            &genewars_runtime::actor_resources::ActorResourceCatalog, database).unwrap();
        let player = world.player(record.owner).unwrap();
        let origin = [25, 42];
        let policy = plan_actor_draw(&record, player, 0, 0, origin,
            record.virtual_height, ActorWrapperKind::PortraitDirect).unwrap();
        assert!(policy.visible);
        assert!(!policy.shadow_required);
        let decoded = decode_actor_components_indexed8(&record, database).unwrap();
        let fade = policy.fade_row.map(|row| {
            let start = usize::try_from(row).unwrap() * 256;
            &resources.fade_bytes()[start..start + 256]
        });
        let expected = direct_color_oracle(&record, &decoded, policy.context, fade,
            resources.main_gla_bytes(), &before);
        let portrait = draw_selected_actor_portrait(&resources, &world, slot, 0, 0,
            &mut indexed.indices).unwrap();
        assert_eq!(portrait.actor_slot, slot);
        assert_eq!(portrait.origin, [25, 42]);
        assert!(portrait.visible_by_wrapper);
        let color = portrait.color.expect("visible portrait is composited");
        assert!(color.pixels_written > 0);
        assert_eq!(indexed.indices, expected, "complete portrait framebuffer must match independent pixel walk");
        assert_eq!(indexed.indices.len(), 640 * 480);
        let changed_left = (0..480).any(|y| (0..128).any(|x| {
            let at = y * 640 + x;
            before[at] != indexed.indices[at]
        }));
        assert!(changed_left, "portrait must affect the left panel");
        for y in 0..480 {
            for x in 128..640 {
                let at = y * 640 + x;
                assert_eq!(before[at], indexed.indices[at], "portrait must not alter world viewport at ({x},{y})");
            }
        }

        // Exercise the same component path against an independently walked
        // clipped placement. This second origin is a synthetic edge case, not
        // a claim about another game callsite.
        let clipped_origin = [-20, -12];
        let clipped_context = genewars_runtime::actor_render::ActorDrawContext {
            origin: clipped_origin.map(i32::from),
            virtual_height: record.virtual_height,
            global_flags: policy.context.global_flags,
        };
        let clipped_before = vec![0x5a; 640 * 480];
        let clipped_expected = direct_color_oracle(&record, &decoded, clipped_context,
            fade, resources.main_gla_bytes(), &clipped_before);
        let mut clipped_actual = clipped_before;
        let clipped_receipt = draw_actor_components(&record, &decoded, clipped_context, fade,
            Some(resources.main_gla_bytes()), &mut clipped_actual, 640, 480, 640).unwrap();
        assert!(clipped_receipt.pixels_clipped > 0);
        assert_eq!(clipped_actual, clipped_expected, "clipped fullframe must match independent pixel walk");
    }

    #[test]
    fn selected_portrait_receipt_on_shared_landing_contact_frame() {
        let root = bundle();
        let mission = fs::read(root.join("terrain/CONQ001.DAT")).unwrap();
        let mut shared = super::super::shared_scene::SharedMission::load(&root, &mission, 5428).unwrap();
        let resources = Resources::load(&root).unwrap().0;
        for _ in 0..189 { shared.step(&resources).unwrap(); }
        shared.select(0).unwrap();
        let active_actors = shared.world.actors().records().iter().enumerate()
            .filter_map(|(slot, record)| (record[6] == 3 && record[0x70] & 1 != 0).then_some(slot))
            .collect::<Vec<_>>();
        let active_effects = shared.world.effects().unwrap().records().iter().enumerate()
            .filter_map(|(slot, record)| (record[6] != 0).then_some(slot))
            .collect::<Vec<_>>();
        let rendered = render_actor_world_with_selected_portrait(
            &resources, &shared.world, [20736, 13568], 0, shared.frame_counter as u8,
            &active_actors, &[0], MuleSceneGlobals { global_shadow_mode: 0, inherited_wrapper_flags: 0, clip_flag_bit0: false },
            &active_effects, shared.selected,
        ).unwrap();
        let receipt = rendered.selected_portrait.expect("selection should request one portrait");
        eprintln!("shared contact portrait receipt: {receipt:?}");
        let color = receipt.color.expect("wrapper-visible selection should have a color receipt");
        assert!(color.pixels_written > 0, "the actual shared contact frame must write portrait pixels");
        let plants = shared.world.plant_render_records().unwrap();
        let baseline_indexed = resources.render_indexed(shared.world.backing_bytes(), &plants, [20736, 13568]).unwrap();
        let baseline = resources.finish_indexed(baseline_indexed,
            genewars_runtime::identity::sha256_hex(shared.world.backing_bytes())).unwrap();
        let baseline_rgb = baseline.image.rgb_bytes();
        let selected_rgb = rendered.frame.image.rgb_bytes();
        let mut changed = 0usize;
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (128usize, 480usize, 0usize, 0usize);
        for y in 0..480 {
            for x in 0..128 {
                let at = (y * 640 + x) * 3;
                if baseline_rgb[at..at + 3] != selected_rgb[at..at + 3] {
                    changed += 1;
                    min_x = min_x.min(x); min_y = min_y.min(y);
                    max_x = max_x.max(x); max_y = max_y.max(y);
                }
            }
        }
        eprintln!("shared contact portrait changed {changed} RGB pixels in [{min_x},{min_y}]..=[{max_x},{max_y}], rendered RGB SHA-256 {}",
            rendered.frame.rgb_sha256);
        assert!(changed > 0, "the source wrapper should visibly change the shared panel framebuffer");
    }

    #[test]
    fn invalid_actor_order_rejects_without_mutating_explicit_world() {
        let root = bundle();
        let (resources, _, target) = Resources::load(&root).unwrap();
        let (world, _) = explicit_fixture_world(&root);
        let before = world.clone();
        assert!(render_actor_world(&resources, &world, target, 0, 0, &[0, 0]).is_err());
        assert!(render_actor_world(&resources, &world, target, 0, 0, &[20]).is_err());
        assert!(render_actor_world(&resources, &world, target, 4, 0, &[]).is_err());
        assert_eq!(world, before);
    }

    #[test]
    fn family8_overlay_changes_snapshot_and_composes_ground_shadow() {
        let root=bundle();let (resources,_,_)=Resources::load(&root).unwrap();
        let (mut world,target)=explicit_fixture_world(&root);
        let before=actor_world_snapshot_sha256(&world);
        world.admit_family8(&[[0;102];16],[None;16]).unwrap();
        assert_ne!(before,actor_world_snapshot_sha256(&world));
        let ground=genewars_core::cd_candidate::interpolate_height(world.map(),target[0],target[1]).unwrap();
        world.create_family8(0,0,target,ground+16000).unwrap();
        let scene=render_actor_world(&resources,&world,target,0,0,&[]).unwrap();
        assert_eq!(scene.family8_receipts.len(),1);
        let receipt=&scene.family8_receipts[0];
        assert_eq!(receipt.virtual_height,16000);
        assert_eq!(receipt.color_origin[1],receipt.ground_origin[1].wrapping_sub(250));
        assert!(receipt.shadow_pixels>0);
        assert_eq!(scene.frame.world_sha256,actor_world_snapshot_sha256(&world));
    }

    #[test]
    fn snapshot_hash_covers_admitted_creature_overlay() {
        let root = bundle();
        let (world, _) = explicit_fixture_world(&root);
        let before = actor_world_snapshot_sha256(&world);
        let mut admitted = world.clone();
        let prior = core::array::from_fn(|slot| {
            core::array::from_fn(|byte| ((slot * 53 + byte * 29) & 255) as u8)
        });
        admitted.admit_creatures(&prior).unwrap();
        assert_ne!(actor_world_snapshot_sha256(&admitted), before,
            "admitting a mutable native creature overlay must change snapshot identity");
    }

    #[test]
    fn explicit_initial_mule_order_draws_source_bound_frame_on_shared_view() {
        let root = bundle();
        let (resources, _, _target) = Resources::load(&root).unwrap();
        let (mut world, _) = explicit_fixture_world(&root);
        world.admit_creatures(&[[0u8; 124]; 300]).unwrap();
        let first = world.creatures().unwrap().record(0).unwrap();
        assert_eq!(first[6], 1);
        assert_eq!(first[0x19], 6);
        assert_eq!(first[0x1b], 0);
        let mule_target = [
            i16::from_le_bytes([first[7], first[8]]),
            i16::from_le_bytes([first[9], first[10]]),
        ];

        let rendered = render_actor_world_with_creatures(
            &resources, &world, mule_target, 0, 0, &[], &[0],
            MuleSceneGlobals {
                global_shadow_mode: 0,
                inherited_wrapper_flags: 0,
                clip_flag_bit0: false,
            },
        ).unwrap();
        let receipt = rendered.mule_receipts.first().unwrap();
        assert!(receipt.drawn);
        assert_eq!(receipt.depth, Some(0));
        assert_eq!(receipt.virtual_height, 0);
        let composition = receipt.composition.as_ref().unwrap();
        assert!(!composition.shadow_pending);
        assert_eq!(composition.animation, 6);
        assert_eq!(composition.frame_index, 0);
        assert!(composition.components.iter().any(|item| item.pixels_written > 0));
        assert!(rendered.frame.image.rgb_bytes().iter().any(|pixel| *pixel != 0));
        assert!(rendered.interfamily_layer_policy.contains("MULE order"));
    }

    #[test]
    fn seeded_full_landing_frame_composes_live_effect_bindings() {
        use genewars_runtime::{landing_effect_lifecycle::RetirementUiState, effect_pool::EffectAnimationBinding};

        let root = bundle();
        let source_bytes = fs::read(root.join("terrain/CONQ001.DAT")).unwrap();
        let source = ConqMissionSource::parse(&source_bytes).unwrap();
        let target = source.target_words(0).unwrap().map(|value| value as i16);
        let players = core::array::from_fn(|id| PlayerState::fresh_campaign(id as u8, target, b"fixture\0").unwrap());
        let prior_actors = [[0u8; 200]; 20];
        let mut world = ActorWorld::from_mission(
            &source, &prior_actors, [ActorHandleBindings::empty(); 20], players,
            BlueprintRng { seed: 5428, ring: 0, draw_count: 0 },
        ).unwrap();
        world.admit_effects(&[[0u8; 81]; 200], [None; 200]).unwrap();
        world.create_landing_core(0, 0).unwrap();
        let resources = Resources::load(&root).unwrap().0;
        let trig = TrigTables::generate_pinned().unwrap();
        let context = world.mission_loading_physics();
        let ui = RetirementUiState::native_empty();
        let mut saw_eff6_pixels = false;
        let mut saw_eff47_pixels = false;
        for pass in 0..450 {
            world.step_landing_effect_callbacks(context, &resources.physics, &trig, &ui)
                .unwrap_or_else(|error| panic!("landing callback failed at pass {}: {error:?}", pass + 1));
            let effects = world.effects().unwrap();
            let order = effects.records().iter().enumerate()
                .filter_map(|(slot, record)| (record[6] == 5).then_some(slot)).collect::<Vec<_>>();
            if !order.is_empty() {
                let frame = render_actor_world_with_effects(&resources, &world, target, 0, (pass + 1) as u8, &[], &order)
                    .unwrap_or_else(|error| panic!("effect composition failed at pass {}: {error}", pass + 1));
                let receipt = frame.effect_scene.expect("explicit effect pass returns a receipt");
                saw_eff47_pixels |= receipt.items.iter().any(|item| item.binding == EffectAnimationBinding::EffAnimation47 && item.color_pixels_written > 0);
                saw_eff6_pixels |= receipt.items.iter().any(|item| item.binding == EffectAnimationBinding::EffAnimation6 && item.color_pixels_written > 0);
            }
        }
        assert!(saw_eff47_pixels, "the seeded 450-pass fixture must draw EFF47 at least once");
        assert!(saw_eff6_pixels, "the seeded 450-pass fixture must draw EFF6 at least once");
    }

}
