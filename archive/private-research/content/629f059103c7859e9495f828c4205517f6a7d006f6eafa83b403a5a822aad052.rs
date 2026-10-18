//!58780 player-list dispatch with bounded construction callbacks.
use super::*;
use crate::actor_motion::{self,ActorIdleContext,ActorMotionError};
use crate::actor_resources::ActorAnimationDatabase;
use crate::physics_tables::PhysicsTables;
use crate::terrain_job_pool::TerrainJobPool;

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum ConstructionAwareActorCallback {
    Ordinary(ActorMovingReceipt),
    Architect{id:u16,receipt:ArchitectStage0Receipt},
}

impl ActorWorld {
    /// The live signed-byte player list is reread after every callback. A
    /// later failure rolls back the entire actor family and terrain-job pool.
    pub fn step_actor_callbacks_with_construction<B:AsRef<[u8]>>(&mut self,player_count:u8,
        frame:u32,context:LandingPhysicsContext,physics:&PhysicsTables,trig:&TrigTables,
        resources:&[ActorAnimationDatabase<B>],jobs:&mut TerrainJobPool,border:&[u8;8192],
        selected_owner:u8,selected_actor:Option<u16>,runtime_mode:u16,override_180d80:u8)
        ->Result<Vec<ConstructionAwareActorCallback>,ConstructionWorldError> {
        if player_count>4||selected_owner>=4 {
            return Err(ActorWorldError::InvalidPlayer(player_count.max(selected_owner)).into())
        }
        let mut staged=self.clone();let mut staged_jobs=jobs.clone();let mut receipts=Vec::new();
        for owner in 0..player_count {
            let mut index=0;
            while let Some(id)=actor_motion::next_player_actor_id(staged.player(owner)?.bytes(),index)
                .map_err(ActorWorldError::from)? {
                index+=1;
                if !(0..20).contains(&id){return Err(ActorWorldError::from(ActorMotionError::UnsupportedState).into())}
                let actor=*staged.actors.record(id as usize).map_err(ActorWorldError::from)?;
                if actor[6]!=3||actor[24]!=owner{return Err(ActorWorldError::ActorOwner(id).into())}
                if actor[0x70]&1==0{continue}
                let key=staged.actors.bindings(id as usize).map_err(ActorWorldError::from)?.animation_database
                    .ok_or(ActorWorldError::MissingBlueprintBinding(id))?;
                let resource=resources.iter().find(|r|r.key()==key).ok_or(ActorWorldError::MissingBlueprintBinding(id))?;
                let callback=if actor[0x55]==9 {
                    ConstructionAwareActorCallback::Architect{id:id as u16,receipt:staged.step_architect_construction_callback(
                        id as u16,frame,context,physics,trig,&mut staged_jobs,border,resource,
                        owner==selected_owner,selected_actor==Some(id as u16))?}
                }else if actor[0x55]==1&&actor[0x57]&3==1 {
                    ConstructionAwareActorCallback::Architect{id:id as u16,receipt:staged.step_pending_architect_callback(
                        id as u16,frame,context,physics,trig,&mut staged_jobs,border,resource,
                        owner==selected_owner,selected_actor==Some(id as u16))?}
                }else{
                    let idle=ActorIdleContext{runtime_mode,override_180d80,
                        runtime_count_11c2bc:word(&staged.backing,0x355c)as u16,
                        player_mode:staged.player(owner)?.bytes()[2]};
                    ConstructionAwareActorCallback::Ordinary(staged.step_actor_callback_with_idle(
                        id as u16,frame,context,physics,trig,resource,idle)?)
                };
                receipts.push(callback);
            }
        }
        *self=staged;*jobs=staged_jobs;Ok(receipts)
    }
}
