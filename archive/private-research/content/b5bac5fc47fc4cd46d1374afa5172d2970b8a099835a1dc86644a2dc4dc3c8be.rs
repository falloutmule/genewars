//! Source-only mission metadata and projected family1 records. This does not
//! construct live objects or infer the selected player, visibility or sorting.
use genewars_assets::{ConqFastPrefix, ConqMissionSource};
use genewars_core::cd_candidate::{camera_cells_for_target, entity_view_position, interpolate_entity_shade};

pub fn inspect(args: &[String]) -> Result<(), String> {
    let bytes = super::read_input(&args[0])?;
    let source = ConqMissionSource::parse(&bytes).map_err(|e| e.to_string())?;
    let player: usize = args[1].parse().map_err(|_| "source player must be0..3")?;
    let target = source.target_words(player).map_err(|e| e.to_string())?.map(|word| word as i16);
    let prefix = ConqFastPrefix::parse(&bytes).map_err(|e| e.to_string())?;
    let map = prefix.opaque_14_byte_region();
    let camera = camera_cells_for_target(map, target[0], target[1]).map_err(|e| e.to_string())?;
    let mut groups = Vec::new();
    for group in source.groups() {
        let layout = group.layout();
        let mut nonzero = 0;
        for index in 0..layout.count {
            if group.raw_byte_6(index).map_err(|e| e.to_string())? != 0 { nonzero += 1; }
        }
        groups.push(format!("{{\"family\":{},\"file_offset\":{},\"count\":{},\"packed_stride\":{},\"nonzero_byte6_records\":{nonzero}}}",
            layout.family, layout.file_offset, layout.count, layout.packed_stride));
    }
    let mut records = Vec::new();
    let family1 = source.group(1).map_err(|e| e.to_string())?;
    for index in 0..family1.layout().count {
        if family1.raw_byte_6(index).map_err(|e| e.to_string())? == 0 { continue; }
        let fields = source.family1_sprite_fields(index).map_err(|e| e.to_string())?;
        let position = fields.signed_position_words;
        let projected = entity_view_position(map, position, camera, 128).map_err(|e| e.to_string())?;
        let shade = interpolate_entity_shade(map, position[0], position[1]).map_err(|e| e.to_string())?;
        records.push(format!("{{\"record\":{index},\"position\":{:?},\"local_projection\":{:?},\"source_shade\":{shade},\"resource_slot\":{},\"animation\":{},\"frame\":{}}}",
            position, projected, fields.resource_slot, fields.animation_selector, fields.frame_selector));
    }
    println!("{{\"layout\":\"{}\",\"source_bytes\":{},\"flags_byte\":{},\"active_count_byte\":{},\"source_player\":{player},\"target\":{:?},\"camera_words\":{:?},\"panel_margin\":128,\"groups\":[{}],\"family1_nonzero_byte6_records\":[{}],\"runtime_objects_constructed\":false,\"visibility_inferred\":false,\"sort_order_inferred\":false}}",
        source.layout_version(), source.bytes().len(), source.flags_byte(), source.active_count_byte(), target, camera,
        groups.join(","), records.join(","));
    Ok(())
}
