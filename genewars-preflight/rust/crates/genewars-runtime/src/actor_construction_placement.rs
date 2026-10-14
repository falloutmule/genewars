//! CD opcode16 (811A0): combined movement and new architect assignment.
//! The caller supplies both ordered 74A50 resolver results; neither is a speed override.
use crate::actor_construction::{plan_architect_assignment,ArchitectAssignmentPlan,ConstructionActorRecord,ConstructionCommandError};

/// 26890 -> 75080 with footprint byte104DE0. Width2 aligns to a cell
/// corner; widths1/3 align to127 within the cell. ECX10 is preserved.
pub fn construction_placement_point(point:[i16;2],selector:u8)->Result<[i16;2],ConstructionCommandError>{
    let footprint=*[3u8,1,2,3,2,2,1,2,1,2,3,3,2,1].get(usize::from(selector)).ok_or(ConstructionCommandError::InvalidSelector(selector))?;
    Ok(point.map(|p|(p&!255)|if footprint==2{0}else{127}))
}

pub fn plan_architect_placement(before:&ConstructionActorRecord,blueprint:&[u8;36],
    mode:u16,argument:u16,resolved:[[i16;2];2],owner_selected:bool,actor_selected:bool)
    ->Result<ArchitectAssignmentPlan,ConstructionCommandError>{
    //811A0 accepts ordinary movement as well as idle. Reuse opcode5's other
    //new-task checks on a local validation copy, preserving the actual state.
    if !matches!(before[0x55],1|8){return Err(ConstructionCommandError::UnsupportedRunningState);}
    let mut validation=*before;validation[0x55]=1;
    let mut plan=plan_architect_assignment(&validation,blueprint,mode,argument,owner_selected,actor_selected)?;
    plan.before=*before;plan.after[0x55]=before[0x55];
    // 8127D resets the first movement discriminator; 812B2 resets the second.
    plan.after[0x5b]=0;plan.after[0x5a]=0;
    for (offset,value) in [(0x5c,resolved[1][0]),(0x5e,resolved[1][1]),(0x0b,resolved[1][0]),(0x0d,resolved[1][1])]{
        plan.after[offset..offset+2].copy_from_slice(&value.to_le_bytes());
    }
    plan.after[0x57]|=2;plan.after[0x58]|=2;
    Ok(plan)
}
