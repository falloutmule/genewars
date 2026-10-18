//! Atomic numeric7DBC0 entry after explicit host presentation completion.
use super::*;
use crate::{mission_context::{MissionRuntimeContext,CallbackGate},mission_outcome::MissionOutcome,result_transition::{self,VictoryEntryPlan,ResultTransitionError}};
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct VictoryEntryTicket { context_before:MissionRuntimeContext, selected:u8, plan:VictoryEntryPlan }
impl VictoryEntryTicket { pub fn plan(&self)->&VictoryEntryPlan{&self.plan} }
/// Host-owned service port. Evidence strings identify actual receipts; the
/// engine never synthesizes BIOWIN/audio completion from a numeric plan.
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum VictoryPresentationStatus {
    Pending,
    Unsupported(String),
    Completed{ordered_service_calls:Vec<u32>,biowin_receipt:String,audio_receipt:String},
}
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum OwnedVictoryEntryError {
    InvalidWinner,ContextNotReady,Source(ResultTransitionError),PresentationPending,
    PresentationUnsupported(String),InvalidPresentationEvidence,StaleContext,StalePlayers,
    Player(crate::player_state::PlayerStateError),Context(&'static str),
}
impl ActorWorld {
    pub fn prepare_victory_entry(&self,context:&MissionRuntimeContext,selected:u8,video_lock_depth:i8)
        ->Result<VictoryEntryTicket,OwnedVictoryEntryError>{
        if selected>=context.participating_players()||context.outcome()!=(MissionOutcome::Won{player:selected}){return Err(OwnedVictoryEntryError::InvalidWinner);}
        if context.callback_gate()!=CallbackGate::Ready||context.runtime_mode().is_none_or(|m|m&3!=0){return Err(OwnedVictoryEntryError::ContextNotReady);}
        let players=core::array::from_fn(|id|*self.players[id].bytes());
        let plan=result_transition::enter_victory(&players,selected,context.known_runtime_flags(),context.runtime_mode().unwrap(),context.frame(),video_lock_depth).map_err(OwnedVictoryEntryError::Source)?;
        Ok(VictoryEntryTicket{context_before:context.clone(),selected,plan})
    }
    /// Commit numeric state only. Returned stage/saved-frame/cleanup obligation
    /// remain host responsibilities; this does not advance the campaign.
    pub fn commit_victory_entry(&mut self,context:&mut MissionRuntimeContext,ticket:&VictoryEntryTicket,
        presentation:&VictoryPresentationStatus)->Result<VictoryEntryPlan,OwnedVictoryEntryError>{
        if *context!=ticket.context_before{return Err(OwnedVictoryEntryError::StaleContext);}
        if (0..4).any(|id|self.players[id].bytes()!=&ticket.plan.players_before[id]){return Err(OwnedVictoryEntryError::StalePlayers);}
        match presentation {
            VictoryPresentationStatus::Pending=>return Err(OwnedVictoryEntryError::PresentationPending),
            VictoryPresentationStatus::Unsupported(reason)=>return Err(OwnedVictoryEntryError::PresentationUnsupported(reason.clone())),
            VictoryPresentationStatus::Completed{ordered_service_calls,biowin_receipt,audio_receipt}=>{
                if ordered_service_calls!=&ticket.plan.service_calls||biowin_receipt.trim().is_empty()||audio_receipt.trim().is_empty(){return Err(OwnedVictoryEntryError::InvalidPresentationEvidence);}
            },
        }
        let mut players=self.players.clone();let mut after_context=context.clone();
        for id in 0..4 {players[id].replace_bookkeeping_checked(&ticket.plan.players_before[id],&ticket.plan.players_after[id]).map_err(OwnedVictoryEntryError::Player)?;}
        after_context.apply_victory_entry_checked(&ticket.context_before,ticket.selected,&ticket.plan).map_err(OwnedVictoryEntryError::Context)?;
        self.players=players;*context=after_context;Ok(ticket.plan.clone())
    }
}
#[cfg(test)]mod tests {
    use super::*;use crate::mission_context::DevelopmentContext;
    fn fixture()->(ActorWorld,MissionRuntimeContext){
        let mut bytes=vec![0;343_958];bytes[..4].copy_from_slice(&0x55336u32.to_le_bytes());bytes[337_054..337_058].copy_from_slice(&0x6BEu32.to_le_bytes());
        let mut world=ActorWorld::from_parts(&ConqMissionSource::parse(&bytes).unwrap(),&[[0;200];20],core::array::from_fn(|_|ActorHandleBindings::default()),core::array::from_fn(|id|PlayerState::fresh_campaign(id as u8,[4096,4096],b"fixture\0").unwrap()),BlueprintRng{seed:0,ring:0,draw_count:0}).unwrap();
        let mut player=*world.players[0].bytes();player[0x49d..0x4a1].copy_from_slice(&1i32.to_le_bytes());player[3]=1;player[0x606]=3;world.players[0]=PlayerState::from_record(&player).unwrap();
        let mut context=MissionRuntimeContext::development(DevelopmentContext{participating_players:1,runtime_flags:0x24,runtime_mode:0,countdown:0,frame:300,replay_seed:0}).unwrap();
        assert_eq!(world.step_mission_outcome(&mut context,0).unwrap().outcome,MissionOutcome::Won{player:0});(world,context)
    }
    fn completed(ticket:&VictoryEntryTicket)->VictoryPresentationStatus{VictoryPresentationStatus::Completed{ordered_service_calls:ticket.plan().service_calls.clone(),biowin_receipt:"synthetic unit service port only".into(),audio_receipt:"synthetic unit service port only".into()}}
    #[test]fn victory_entry_requires_services_and_commits_once(){
        let(mut w,mut c)=fixture();let ticket=w.prepare_victory_entry(&c,0,0).unwrap();let before=w.clone();let old=c.clone();
        for evidence in [VictoryPresentationStatus::Pending,VictoryPresentationStatus::Unsupported("BIOWIN pending".into()),VictoryPresentationStatus::Completed{ordered_service_calls:vec![],biowin_receipt:"unit".into(),audio_receipt:"unit".into()}]{assert!(w.commit_victory_entry(&mut c,&ticket,&evidence).is_err());assert_eq!(w,before);assert_eq!(c,old);}
        let plan=w.commit_victory_entry(&mut c,&ticket,&completed(&ticket)).unwrap();assert_eq!(c.frame(),0);assert_eq!(c.runtime_mode(),Some(2));assert_eq!(c.known_runtime_flags(),0x60);assert_eq!(w.players[0].bytes()[0x6e7],1);assert_eq!(c.outcome(),MissionOutcome::Won{player:0});assert_eq!(plan.saved_frame,300);assert!(plan.exit_cleanup_72e50_required);
        let after=w.clone();let new=c.clone();assert_eq!(w.commit_victory_entry(&mut c,&ticket,&completed(&ticket)),Err(OwnedVictoryEntryError::StaleContext));assert!(w.prepare_victory_entry(&c,0,0).is_err());assert_eq!(w,after);assert_eq!(c,new);
    }
    #[test]fn victory_entry_rejects_stale_context_and_players_atomically(){
        let(mut w,mut c)=fixture();let ticket=w.prepare_victory_entry(&c,0,0).unwrap();c.finish_outer_frame();let old=c.clone();let before=w.clone();assert_eq!(w.commit_victory_entry(&mut c,&ticket,&completed(&ticket)),Err(OwnedVictoryEntryError::StaleContext));assert_eq!(w,before);assert_eq!(c,old);
        let(mut w,mut c)=fixture();let ticket=w.prepare_victory_entry(&c,0,0).unwrap();let mut player=*w.players[1].bytes();player[0x6e7]=2;w.players[1]=PlayerState::from_record(&player).unwrap();let before=w.clone();let old=c.clone();assert_eq!(w.commit_victory_entry(&mut c,&ticket,&completed(&ticket)),Err(OwnedVictoryEntryError::StalePlayers));assert_eq!(w,before);assert_eq!(c,old);
    }
    #[test]fn victory_entry_rejects_wrong_selected_and_locked_video(){let(w,c)=fixture();assert_eq!(w.prepare_victory_entry(&c,1,0),Err(OwnedVictoryEntryError::InvalidWinner));assert_eq!(w.prepare_victory_entry(&c,0,1),Err(OwnedVictoryEntryError::Source(ResultTransitionError::VideoLockState)));}
}
