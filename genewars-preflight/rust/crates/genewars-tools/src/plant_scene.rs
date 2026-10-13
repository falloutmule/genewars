//! A strict, explicit post-admission snapshot for bounded scene diagnostics.
//! This format is authored for these tools; it is not an original game file.
use genewars_core::cd_candidate::{family1_draw_packet, queue_family1, ProjectedFamily1};
use genewars_render::{plant::{PlantColorInputs, PlantFrame},
    plant_scene::{draw_explicit_family1_scene, PlantSceneRecord, PlantSceneRecordStatus},
    terrain::TerrainFadeTable, terrain_modes::MainGlaTable};
use std::{collections::BTreeSet, path::Path, str::FromStr};

#[derive(Debug)]
struct Plant {
    point: ProjectedFamily1,
    slot: u8,
    animation: u8,
    frame: u8,
    jitter: u8,
}

#[derive(Debug)]
pub struct Snapshot {
    active: bool,
    initial_count: i16,
    margin: u16,
    gate: bool,
    mode: u8,
    context_flags: u8,
    plants: Vec<Plant>,
}

fn number<T: FromStr>(word: &str) -> Result<T, String> {
    word.parse().map_err(|_| format!("invalid or out-of-range snapshot number: {word}"))
}

fn bit(word: &str) -> Result<bool, String> {
    match word { "0" => Ok(false), "1" => Ok(true), _ => Err("snapshot booleans must be0 or1".into()) }
}

impl Snapshot {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > 1024 * 1024 { return Err("snapshot exceeds1 MiB diagnostic limit".into()); }
        let text = std::str::from_utf8(bytes).map_err(|_| "snapshot must be UTF-8")?;
        let mut lines = text.lines().filter(|line| !line.trim().is_empty());
        if lines.next() != Some("GWPLANT1") { return Err("snapshot must start with GWPLANT1".into()); }
        let words: Vec<_> = lines.next().ok_or("missing snapshot context")?.split_whitespace().collect();
        if words.len() != 7 || words[0] != "context" {
            return Err("expected context ACTIVE INITIAL_COUNT MARGIN GATE MODE CONTEXT_FLAGS".into());
        }
        let mut snapshot = Self { active: bit(words[1])?, initial_count: number(words[2])?,
            margin: number(words[3])?, gate: bit(words[4])?, mode: number(words[5])?,
            context_flags: number(words[6])?, plants: Vec::new() };
        if !(0..=256).contains(&snapshot.initial_count) || snapshot.margin != 128 {
            return Err("scene snapshot requires count0..256 and margin128".into());
        }
        let mut ids = BTreeSet::new();
        for line in lines {
            if snapshot.plants.len() == 4096 { return Err("snapshot exceeds4096 diagnostic candidates".into()); }
            let words: Vec<_> = line.split_whitespace().collect();
            if words.len() != 10 || words[0] != "plant" {
                return Err("expected plant ID SLOT ANIM FRAME X Y SHADE FLAGS JITTER".into());
            }
            let id = number(words[1])?;
            if !ids.insert(id) { return Err("duplicate snapshot ID".into()); }
            let slot: u8 = number(words[2])?;
            if slot >= 12 { return Err("snapshot slot must be0..11".into()); }
            snapshot.plants.push(Plant { point: ProjectedFamily1 { id, x: number(words[5])?,
                y: number(words[6])?, shade: number(words[7])?, flags: number(words[8])? },
                slot, animation: number(words[3])?, frame: number(words[4])?, jitter: number(words[9])? });
        }
        Ok(snapshot)
    }

    pub fn draw(&self, directory: &str, indices: &mut [u8], fade: &TerrainFadeTable<'_>,
                gla: &MainGlaTable<'_>) -> Result<String, String> {
        let candidates: Vec<_> = self.plants.iter().map(|plant| plant.point).collect();
        let queue = queue_family1(self.active, self.initial_count, self.margin, &candidates)
            .map_err(|error| error.to_string())?;
        let mut frames = Vec::new();
        let mut selected = Vec::new();
        let mut packets = Vec::new();
        for entry in &queue.draw_order {
            // IDs were checked unique; queue entries can only originate here.
            let plant = self.plants.iter().find(|plant| plant.point.id == entry.id).unwrap();
            let path = Path::new(directory).join(format!("slot-{:02}.anb", plant.slot));
            let bytes = super::read_input(path.to_str().ok_or("plant resource path is not UTF-8")?)?;
            frames.push(PlantFrame::parse_indexed8(&bytes, plant.animation, plant.frame)
                .map_err(|error| format!("plant{}: {error}", entry.id))?);
            selected.push(plant);
            packets.push(family1_draw_packet(self.gate, self.mode, entry.flags));
        }
        let records: Vec<_> = queue.draw_order.iter().enumerate().map(|(i, entry)| PlantSceneRecord {
            id: entry.id, frame: &frames[i], origin: [i32::from(entry.x), i32::from(entry.draw_y)],
            inputs: PlantColorInputs { context_flags_at_16e0c4: self.context_flags,
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
            self.plants.len(), self.initial_count, queue.final_count, self.active, self.gate, self.mode,
            self.context_flags, self.margin, rows.join(","), receipt.foreground_pixels_written,
            receipt.foreground_pixels_clipped, receipt.shadow_pixels_written, receipt.shadow_pixels_clipped,
            receipt.pixels_written, receipt.pixels_clipped))
    }
}
