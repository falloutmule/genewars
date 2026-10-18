//! A strict, explicit post-admission snapshot for bounded scene diagnostics.
//! This format is authored for these tools; it is not an original game file.
use genewars_core::cd_candidate::{family1_draw_packet, queue_family1, ProjectedFamily1};
use genewars_render::{plant::{PlantColorInputs, PlantFrame},
    plant_scene::{draw_explicit_family1_scene, PlantSceneRecord, PlantSceneRecordStatus},
    terrain::TerrainFadeTable, terrain_modes::MainGlaTable, PlantSceneSnapshot};
use std::path::Path;

#[derive(Debug)]
pub struct Snapshot(PlantSceneSnapshot);

impl Snapshot {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        PlantSceneSnapshot::parse(bytes).map(Self)
    }

    pub fn draw(&self, directory: &str, indices: &mut [u8], fade: &TerrainFadeTable<'_>,
                gla: &MainGlaTable<'_>) -> Result<String, String> {
        let snapshot = &self.0;
        let candidates: Vec<_> = snapshot.plants.iter().map(|plant| ProjectedFamily1 {
            id: plant.id, x: plant.x, y: plant.y, shade: plant.shade, flags: plant.flags,
        }).collect();
        let queue = queue_family1(snapshot.active, snapshot.initial_count, snapshot.margin, &candidates)
            .map_err(|error| error.to_string())?;
        let mut frames = Vec::new();
        let mut selected = Vec::new();
        let mut packets = Vec::new();
        for entry in &queue.draw_order {
            // IDs were checked unique; queue entries can only originate here.
            let plant = snapshot.plants.iter().find(|plant| plant.id == entry.id).unwrap();
            let path = Path::new(directory).join(format!("slot-{:02}.anb", plant.slot));
            let bytes = super::read_input(path.to_str().ok_or("plant resource path is not UTF-8")?)?;
            frames.push(PlantFrame::parse_indexed8(&bytes, plant.animation, plant.frame)
                .map_err(|error| format!("plant{}: {error}", entry.id))?);
            selected.push(plant);
            packets.push(family1_draw_packet(snapshot.gate, snapshot.mode, entry.flags));
        }
        let records: Vec<_> = queue.draw_order.iter().enumerate().map(|(i, entry)| PlantSceneRecord {
            id: entry.id, frame: &frames[i], origin: [i32::from(entry.x), i32::from(entry.draw_y)],
            inputs: PlantColorInputs { context_flags_at_16e0c4: snapshot.context_flags,
                entity_flags_at_15: entry.flags, native_flags_at_2e: selected[i].jitter, shade_at_17: entry.shade },
            draw: packets[i].draws,
        }).collect();
        let receipt = draw_explicit_family1_scene(&records, indices, [512, 480], 640, Some(fade), gla)
            .map_err(|error| error.to_string())?;
        let mut rows = Vec::new();
        for (i, entry) in queue.draw_order.iter().enumerate() {
            let (fw, fc, sw, sc) = match &receipt.records[i].status {
                PlantSceneRecordStatus::SkippedByCaller => (0, 0, 0, 0),
                PlantSceneRecordStatus::Drawn(draw) => (draw.foreground_pixels_written,
                    draw.foreground_pixels_clipped, draw.shadow_pixels_written, draw.shadow_pixels_clipped),
            };
            let plant = selected[i];
            let packet = packets[i];
            let class = packet.dirty_class.map(|value| value.to_string()).unwrap_or("null".to_owned());
            rows.push(format!("{{\"id\":{},\"projected\":[{},{}],\"origin\":[{},{}],\"bucket\":{},\"residual\":{},\"slot\":{},\"animation\":{},\"frame\":{},\"shade\":{},\"flags_before\":{},\"flags_after\":{},\"dirty_class\":{class},\"drawn\":{},\"foreground_pixels\":{fw},\"foreground_clipped\":{fc},\"shadow_pixels\":{sw},\"shadow_clipped\":{sc}}}",
                entry.id, entry.x, entry.projected_y, entry.x, entry.draw_y, entry.bucket, entry.residual,
                plant.slot, plant.animation, plant.frame, entry.shade, entry.flags, packet.flags_after, packet.draws));
        }
        let order: Vec<_> = queue.draw_order.iter().map(|entry| entry.id).collect();
        Ok(format!("{{\"policy\":\"explicit-post-admission-snapshot\",\"candidate_count\":{},\"initial_count\":{},\"final_count\":{},\"active_module\":{},\"context_gate\":{},\"draw_mode\":{},\"context_flags\":{},\"margin\":{},\"queue_order\":{order:?},\"records\":[{}],\"foreground_pixels\":{},\"foreground_clipped\":{},\"shadow_pixels\":{},\"shadow_clipped\":{},\"pixels_written\":{},\"pixels_clipped\":{},\"admission_callbacks_applied\":false,\"dirty_prepass_applied\":false,\"initial_state_inferred\":false}}",
            snapshot.plants.len(), snapshot.initial_count, queue.final_count, snapshot.active, snapshot.gate, snapshot.mode,
            snapshot.context_flags, snapshot.margin, rows.join(","), receipt.foreground_pixels_written,
            receipt.foreground_pixels_clipped, receipt.shadow_pixels_written, receipt.shadow_pixels_clipped,
            receipt.pixels_written, receipt.pixels_clipped))
    }
}
