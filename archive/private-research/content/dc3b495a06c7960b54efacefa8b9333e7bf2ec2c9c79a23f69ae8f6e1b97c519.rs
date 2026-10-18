use super::*;
use crate::mission_tail::{self,MissionTailError,TailContext,TailPlan};
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct MissionTailRuntimeContext{pub runtime_flags:u8,pub runtime_mode:u8,pub active_player_count:u8,pub selected_player:u8,pub frame:u32}
#[derive(Debug)]
pub enum MissionTailWorldError{Plan(MissionTailError<ActorWorldError>),PlayerCommit(crate::player_state::PlayerStateError)}
impl ActorWorld{
 /// Exact bounded72ED0 nonterminal return. Runtime globals are explicit;
 /// raw backing fields and live player records come from this owned world.
 /// Terminal/deferred branches reject before any player mutation commits.
 pub fn step_mission_tail_nonterminal(&mut self,r:MissionTailRuntimeContext)->Result<TailPlan,MissionTailWorldError>{
  let b=&self.backing;let word=|p|u16::from_le_bytes(b[p..p+2].try_into().unwrap());let dword=|p|u32::from_le_bytes(b[p..p+4].try_into().unwrap());
  let context=TailContext{flags:r.runtime_flags,mode:r.runtime_mode,active:r.active_player_count,selected:r.selected_player,frame:r.frame,mission_counter:dword(0x1ec2),target_count:word(0x1ea6),mission_limit:word(0x1c7c),mission_timer:word(0x355c),terrain_count:b[0x34b8],terrain_limit:dword(0x1eaa)as i32,field_34b9:b[0x34b9]};
  let before=std::array::from_fn(|i|*self.players[i].bytes());
  let plan=mission_tail::plan_full(&before,&context,|id|self.construction_capability_exhausted(id)).map_err(MissionTailWorldError::Plan)?;
  let mut staged=self.clone();
  for i in 0..4{staged.players[i].replace_bookkeeping_checked(&before[i],&plan.players[i]).map_err(MissionTailWorldError::PlayerCommit)?;}
  *self=staged;Ok(plan)
 }
}

#[cfg(test)]mod tests{
 use super::*;
 fn players()->[PlayerState;4]{std::array::from_fn(|i|PlayerState::fresh_campaign(i as u8,[20736,13568],b"fixture\0").unwrap())}
 fn record(p:&mut PlayerState,f:impl FnOnce(&mut[u8;1805])){let mut b=*p.bytes();f(&mut b);*p=PlayerState::from_record(&b).unwrap();}
 fn world(players:[PlayerState;4])->ActorWorld{let mut raw=std::fs::read(std::path::Path::new(&std::env::var("GENEWARS_REVIEW_ROOT").unwrap()).join("private/integration/game-runtime/terrain/CONQ001.DAT")).unwrap();raw[0x1ec2..0x1ec6].fill(0);raw[0x1ea6..0x1ea8].fill(0);raw[0x34b8..0x34ba].fill(0);ActorWorld::from_parts(&genewars_assets::ConqMissionSource::parse(&raw).unwrap(),&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed:5428,ring:1,draw_count:64}).unwrap()}
 fn context(selected:u8)->MissionTailRuntimeContext{MissionTailRuntimeContext{runtime_flags:0x20,runtime_mode:0,active_player_count:1,selected_player:selected,frame:3}}
 #[test]fn nonterminal_commits_only_exact_elimination_byte(){let mut p=players();record(&mut p[0],|b|b[0x49d..0x4a1].copy_from_slice(&1i32.to_le_bytes()));let mut w=world(p);let before=w.clone();let q=w.step_mission_tail_nonterminal(context(3)).unwrap();assert_eq!(q.players[0][0x6e3],1);let mut expected=before;let prior=*expected.players[0].bytes();let mut after=prior;after[0x6e3]=1;expected.players[0].replace_bookkeeping_checked(&prior,&after).unwrap();assert_eq!(w,expected);}
 #[test]fn late_capability_error_rolls_back_prior_player_elimination(){let mut p=players();record(&mut p[0],|b|b[0x49d..0x4a1].copy_from_slice(&1i32.to_le_bytes()));record(&mut p[1],|b|{b[0x49d..0x4a1].copy_from_slice(&1i32.to_le_bytes());b[0x4a2]=1;b[3]=1;b[4]=0;});let mut w=world(p);let before=w.clone();let mut c=context(1);c.active_player_count=2;assert!(matches!(w.step_mission_tail_nonterminal(c),Err(MissionTailWorldError::Plan(MissionTailError::Capability(_)))));assert_eq!(w,before);}
 #[test]fn terminal_and_deferred_branches_roll_back(){let mut p=players();record(&mut p[0],|b|b[0x49d..0x4a1].copy_from_slice(&1i32.to_le_bytes()));let mut w=world(p);let before=w.clone();assert!(matches!(w.step_mission_tail_nonterminal(context(0)),Err(MissionTailWorldError::Plan(MissionTailError::TerminalLossRequired{..}))));assert_eq!(w,before);w.backing[0x1ec2..0x1ec6].copy_from_slice(&3u32.to_le_bytes());let before=w.clone();assert!(matches!(w.step_mission_tail_nonterminal(context(0)),Err(MissionTailWorldError::Plan(MissionTailError::DeferredLandingRequired{..}))));assert_eq!(w,before);}
 #[test]fn terminal_earlyreturn_and_capability_shortcircuit_do_not_resolve_handles(){let mut p=players();record(&mut p[0],|b|{b[0x49d..0x4a1].copy_from_slice(&1i32.to_le_bytes());b[0x4a2]=1;b[3]=127;b[0x616..0x61a].copy_from_slice(&1i32.to_le_bytes());});let mut w=world(p);let before=w.clone();let q=w.step_mission_tail_nonterminal(context(0)).unwrap();assert_eq!(q.capability_queries,[0]);assert_eq!(w,before);let mut c=context(0);c.runtime_flags=0x40;let q=w.step_mission_tail_nonterminal(c).unwrap();assert!(q.already_terminal);assert!(q.capability_queries.is_empty());assert_eq!(w,before);}
}
