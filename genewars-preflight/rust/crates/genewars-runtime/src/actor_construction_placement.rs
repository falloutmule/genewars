//! CD opcode16 (811A0): combined movement and new architect assignment.
//! The caller supplies both ordered 74A50 resolver results; neither is a speed override.
use crate::actor_construction::{plan_architect_assignment,ArchitectAssignmentPlan,ConstructionActorRecord,ConstructionCommandError};

pub fn plan_architect_placement(before:&ConstructionActorRecord,blueprint:&[u8;36],
    mode:u16,argument:u16,resolved:[[i16;2];2],owner_selected:bool,actor_selected:bool)
    ->Result<ArchitectAssignmentPlan,ConstructionCommandError>{
    let mut plan=plan_architect_assignment(before,blueprint,mode,argument,owner_selected,actor_selected)?;
    // 8127D resets the first movement discriminator; 812B2 resets the second.
    plan.after[0x5b]=0;plan.after[0x5a]=0;
    for (offset,value) in [(0x5c,resolved[1][0]),(0x5e,resolved[1][1]),(0x0b,resolved[1][0]),(0x0d,resolved[1][1])]{
        plan.after[offset..offset+2].copy_from_slice(&value.to_le_bytes());
    }
    plan.after[0x57]|=2;plan.after[0x58]|=2;
    Ok(plan)
}
