#[path = "../rust/crates/genewars-runtime/src/family8_creation.rs"]
mod family8_creation;
use family8_creation::{plan_creation, CreationInput, CellHeadIdentity, CreationError};
use std::{fs, path::Path};
fn main() {
    let path = std::env::args().nth(1).unwrap(); let work = Path::new(&path);
    let bytes = fs::read(work.join("input.bin")).unwrap();
    let map = fs::read(work.join("map.bin")).unwrap();
    let mut output = Vec::new();
    for case in bytes.chunks_exact(117) {
        let before = case[..102].try_into().unwrap();
        let slot = u16::from_le_bytes(case[102..104].try_into().unwrap());
        let position = [i16::from_le_bytes(case[106..108].try_into().unwrap()), i16::from_le_bytes(case[108..110].try_into().unwrap())];
        let old_head = (case[116] != 0).then_some(CellHeadIdentity { index: u16::from_le_bytes(case[114..116].try_into().unwrap()), kind: case[116] });
        let input = CreationInput { slot, owner: case[104], selector: case[105], position,
            ground_height: genewars_core::cd_candidate::interpolate_height(&map, position[0], position[1]).unwrap(),
            shade: genewars_core::cd_candidate::interpolate_entity_shade(&map, position[0], position[1]).unwrap(),
            initial_height: i32::from_le_bytes(case[110..114].try_into().unwrap()), old_head };
        let plan = plan_creation(&before, input).unwrap();
        output.extend(plan.after); output.push(plan.animation_table_offset);
        output.extend(plan.cell_head_after.index.to_le_bytes()); output.push(plan.cell_head_after.kind);
        if let Some((_, previous)) = plan.old_head_previous { output.extend(previous.index.to_le_bytes()); output.push(previous.kind); }
        else { output.extend([0; 3]); }
        assert_eq!(plan.cell, u16::from_le_bytes(plan.after[..2].try_into().unwrap()));
        assert_eq!(plan_creation(&before, CreationInput { slot: 16, ..input }), Err(CreationError::InvalidSlot));
        assert_eq!(plan_creation(&before, CreationInput { owner: 4, ..input }), Err(CreationError::InvalidOwner));
        assert_eq!(plan_creation(&before, CreationInput { selector: 3, ..input }), Err(CreationError::InvalidSelector));
        let mut active = before; active[6] = 8;
        assert_eq!(plan_creation(&active, input), Err(CreationError::ActiveSlot));
    }
    fs::write(work.join("actual.bin"), output).unwrap();
}
