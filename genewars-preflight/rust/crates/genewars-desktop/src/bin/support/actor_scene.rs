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

use super::runtime_resources::{Frame, Resources};

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

#[derive(Debug)]
pub struct ActorSceneFrame {
    pub frame: Frame,
    pub camera: [u16; 2],
    pub actor_order: Vec<usize>,
    pub actor_receipts: Vec<ActorSceneActorReceipt>,
    pub pending_effect_slots: Vec<usize>,
    pub actor_order_policy: &'static str,
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
    if selected_player >= 4 { return Err("selected player must be in 0..4".into()); }
    if actor_order.len() > 20 { return Err("actor draw order exceeds the twenty-slot pool".into()); }
    let mut unique = HashSet::with_capacity(actor_order.len());
    for &slot in actor_order {
        if slot >= 20 { return Err(format!("actor slot {slot} is out of range")); }
        if !unique.insert(slot) { return Err(format!("actor slot {slot} occurs more than once in draw order")); }
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

    let pending_effect_slots = world.effects().map(|pool| pool.records().iter().enumerate()
        .filter_map(|(slot, record)| (record[6] != 0).then_some(slot)).collect()).unwrap_or_default();
    let snapshot_sha256 = actor_world_snapshot_sha256(world);
    let frame = resources.finish_indexed(indexed, snapshot_sha256)?;
    Ok(ActorSceneFrame { frame, camera, actor_order: actor_order.to_vec(), actor_receipts,
        pending_effect_slots, actor_order_policy: "caller-supplied; family2 queue order unproved" })
}

fn actor_world_snapshot_sha256(world: &ActorWorld) -> String {
    let mut bytes = Vec::with_capacity(world.backing_bytes().len() + 20 * 200 + 4 * 0x70d + 200 * 81 + 32);
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
        for record in effects.records() { bytes.extend_from_slice(record); }
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
}
