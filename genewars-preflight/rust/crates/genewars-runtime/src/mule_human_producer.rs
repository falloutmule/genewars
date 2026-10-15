//! Bounded MAINPANL action5 continuation2CC87..2CE90, single selected Mule.
//! The caller has already reached source action5 and resolved the hover CL and
//! terrain projection. This module establishes no hardware button mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProducerInput {
    pub selected_player_index: u8,
    pub selected_player_owner: u8,
    /// Parent2C965 suppression byte19FF71, not an invented button identity.
    pub suppression_flags: u8,
    pub pause_flags: u8,
    pub hover_consumed_cl: u8,
    pub projected_point: [i16; 2],
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandPacket { pub words: [i16; 5], pub opcode: u8 }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProducerError { Player, Identity, UnsupportedGroup, UnsupportedActorKind }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IgnoreReason { Suppressed, Paused, HoverConsumed, NoSelection, Ownership, EntityKind }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProducerOutcome { Ignored(IgnoreReason), Issued }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProducerPlan {
    pub packet_after: CommandPacket,
    pub manager_after: [u8; 0x59d],
    pub outcome: ProducerOutcome,
}
/// Selected pointer is an explicit resolved native record. Null remains None.
/// Native manager bytes and all untouched packet words are retained, including
/// packet word4 (+8). Source group opcode12 and actor opcode3 are not admitted.
/// No command handler, movement, RNG, audio or16988 group pointer write occurs.
pub fn plan_action5(selected: Option<&[u8; 124]>, packet: CommandPacket,
    manager: &[u8; 0x59d], input: ProducerInput) -> Result<ProducerPlan, ProducerError> {
    if input.selected_player_index >= 4 || input.selected_player_owner >= 4 { return Err(ProducerError::Player); }
    let mut plan = ProducerPlan { packet_after: packet, manager_after: *manager, outcome: ProducerOutcome::Issued };
    let ignored = |mut p: ProducerPlan, reason| { p.outcome=ProducerOutcome::Ignored(reason); Ok(p) };
    // Parent2C965..2C979 guards, deliberately separate from the literal
    //2CC87 validator entry. They must not be mistaken for mouse mapping.
    if input.suppression_flags & 0x0a != 0 { return ignored(plan, IgnoreReason::Suppressed); }
    if input.pause_flags & 2 != 0 { return ignored(plan, IgnoreReason::Paused); }
    if input.hover_consumed_cl != 0 { return ignored(plan, IgnoreReason::HoverConsumed); }
    if u16::from_le_bytes(manager[0x526..0x528].try_into().unwrap()) != 0 || manager[0x52f] != 0 { return Err(ProducerError::UnsupportedGroup); }
    let Some(selected)=selected else { return ignored(plan, IgnoreReason::NoSelection); };
    if selected[24] != input.selected_player_owner { return ignored(plan, IgnoreReason::Ownership); }
    if selected[6] == 3 { return Err(ProducerError::UnsupportedActorKind); }
    if selected[6] != 1 { return ignored(plan, IgnoreReason::EntityKind); }
    let slot=i16::from_le_bytes(selected[15..17].try_into().unwrap());
    if !(0..300).contains(&slot) || selected[122] != 4 { return Err(ProducerError::Identity); }
    plan.packet_after.opcode=0x11;
    plan.packet_after.words[0]=slot;
    plan.packet_after.words[1]=input.projected_point[0];
    plan.packet_after.words[2]=input.projected_point[1];
    plan.packet_after.words[3]=0;
    plan.manager_after[0x58c]=0;
    plan.manager_after[0x58d]=1;
    plan.manager_after[0x58e]=1;
    for (i,v) in input.projected_point.into_iter().enumerate() { plan.manager_after[0x599+2*i..0x59b+2*i].copy_from_slice(&v.to_le_bytes()); }
    Ok(plan)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn input()->ProducerInput{ProducerInput{selected_player_index:0,selected_player_owner:2,suppression_flags:0,pause_flags:0,hover_consumed_cl:0,projected_point:[-32768,32767]}}
    fn mule()->[u8;124]{let mut m=[0;124];m[6]=1;m[24]=2;m[122]=4;m[15..17].copy_from_slice(&299i16.to_le_bytes());m}
    #[test]fn issued_packet_preserves_unused_word_and_manager_bytes(){let mut manager=[0xa5;0x59d];manager[0x526..0x528].fill(0);manager[0x52f]=0;let packet=CommandPacket{words:[1,2,3,4,-1234],opcode:99};let p=plan_action5(Some(&mule()),packet,&manager,input()).unwrap();assert_eq!(p.packet_after,CommandPacket{words:[299,-32768,32767,0,-1234],opcode:17});assert_eq!(&p.manager_after[0x58c..0x58f],&[0,1,1]);for i in 0..manager.len(){if !matches!(i,0x58c..=0x58e|0x599..=0x59c){assert_eq!(p.manager_after[i],manager[i]);}}}
    #[test]fn parent_guards_and_hover_preserve_packet_and_manager(){let mut manager=[0xa5;0x59d];manager[0x526..0x528].fill(0);manager[0x52f]=0;let packet=CommandPacket{words:[1,2,3,4,5],opcode:99};for case in 0..3{let mut i=input();if case==0{i.suppression_flags=2;}else if case==1{i.pause_flags=2;}else{i.hover_consumed_cl=255;}let p=plan_action5(Some(&mule()),packet,&manager,i).unwrap();assert!(matches!(p.outcome,ProducerOutcome::Ignored(_)));assert_eq!(p.packet_after,packet);assert_eq!(p.manager_after,manager);}}
    #[test]fn group_and_foreign_owner_cannot_issue_single_creature_packet(){let mut manager=[0;0x59d];let packet=CommandPacket{words:[0;5],opcode:0};let mut i=input();manager[0x526..0x528].fill(255);assert_eq!(plan_action5(Some(&mule()),packet,&manager,i),Err(ProducerError::UnsupportedGroup));manager[0x526..0x528].fill(0);i.selected_player_owner=1;assert_eq!(plan_action5(Some(&mule()),packet,&manager,i).unwrap().outcome,ProducerOutcome::Ignored(IgnoreReason::Ownership));}
}
