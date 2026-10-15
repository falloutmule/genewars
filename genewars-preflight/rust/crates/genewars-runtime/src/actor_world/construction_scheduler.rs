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


#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ActorDispatchSnapshot {
 pub state:u8,pub restore_state:u8,pub category:u8,pub style:u8,
 pub task:u8,pub task_kind:u8,pub selector:u8,pub blueprint_prior:u8,pub blueprint_current:u8,
}
impl ActorDispatchSnapshot {
 pub fn blueprint_equal(&self)->bool{self.blueprint_prior==self.blueprint_current}
 pub fn capture(world:&ActorWorld,id:u16)->Result<Self,ActorWorldError>{
  let r=world.actors.record(id as usize)?;let binding=world.actors.bindings(id as usize)?.blueprint.ok_or(ActorWorldError::MissingBlueprintBinding(id as i16))?;
  let b=world.player(binding.player_id)?.blueprint(binding.blueprint_index as usize)?;
  Ok(Self{state:r[0x55],restore_state:r[0x56],category:b[0x16],style:b[0x15],task:r[0x7e],task_kind:r[0x82],selector:r[0x83],blueprint_prior:b[0x18],blueprint_current:b[0x1b]})
 }
}
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum ActorObservedBranch { Ordinary{source_dispatch:u32,idle_scorer_empty:Option<bool>}, Architect{source_dispatch:u32,setup_after_state:Option<u8>,rejected_site:bool} }
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ActorDispatchObservation {pub frame:u32,pub actor_id:u16,pub before:ActorDispatchSnapshot,pub after:ActorDispatchSnapshot,pub branch:ActorObservedBranch}

//57F92 selects the action from state55 AFTER pending setup/cancellation.
//Receipt boundaries are pre-action/pre-tail evidence; final actor state is not.
//Zero means unknown, not a claimed source call.
fn architect_observed_dispatch(motion_state:u8,setup_state:Option<u8>,rejected:bool,cancellation_state:Option<u8>)->u32 {
 let state=if rejected { cancellation_state } else { setup_state.or(Some(motion_state)) };
 if state==Some(9){0x534d0}else{0}
}

impl ActorWorld {
    /// The live signed-byte player list is reread after every callback. A
    /// later failure rolls back the entire actor family and terrain-job pool.
    pub fn step_actor_callbacks_with_construction<B:AsRef<[u8]>>(&mut self,player_count:u8,
        frame:u32,context:LandingPhysicsContext,physics:&PhysicsTables,trig:&TrigTables,
        resources:&[ActorAnimationDatabase<B>],jobs:&mut TerrainJobPool,border:&[u8;8192],
        selected_owner:u8,selected_actor:Option<u16>,runtime_mode:u16,override_180d80:u8)
        ->Result<Vec<ConstructionAwareActorCallback>,ConstructionWorldError> {
        self.step_actor_callbacks_with_construction_observed(player_count,frame,context,physics,trig,resources,jobs,border,selected_owner,selected_actor,runtime_mode,override_180d80,|_|{})
    }
    /// Read-only audit notifications are flushed only after the complete pass commits.
    pub fn step_actor_callbacks_with_construction_observed<B:AsRef<[u8]>>(&mut self,player_count:u8,
        frame:u32,context:LandingPhysicsContext,physics:&PhysicsTables,trig:&TrigTables,
        resources:&[ActorAnimationDatabase<B>],jobs:&mut TerrainJobPool,border:&[u8;8192],
        selected_owner:u8,selected_actor:Option<u16>,runtime_mode:u16,override_180d80:u8,
        mut observe:impl FnMut(&ActorDispatchObservation))
        ->Result<Vec<ConstructionAwareActorCallback>,ConstructionWorldError> {
        let mut observations=Vec::new();
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
                let observed_before=ActorDispatchSnapshot::capture(&staged,id as u16)?;
                let cancelled=actor[0x55]==9&&actor[0x7e]==0xff&&actor[0x82]==0;
                let attempt=|staged:&mut ActorWorld,staged_jobs:&mut TerrainJobPool|->Result<ConstructionAwareActorCallback,ConstructionWorldError>{Ok(if actor[0x55]==9&&!cancelled {
                    ConstructionAwareActorCallback::Architect{id:id as u16,receipt:staged.step_architect_construction_callback(
                        id as u16,frame,context,physics,trig,staged_jobs,border,resource,
                        owner==selected_owner,selected_actor==Some(id as u16))?}
                }else if actor[0x55]==1&&actor[0x57]&3==1 {
                    ConstructionAwareActorCallback::Architect{id:id as u16,receipt:staged.step_pending_architect_callback(
                        id as u16,frame,context,physics,trig,staged_jobs,border,resource,
                        owner==selected_owner,selected_actor==Some(id as u16))?}
                }else if actor[0x55]==8&&actor[0x57]&3==1&&actor[0x58]==2&&actor[0x82]==1&&actor[0x7e]==0 {
                    // Preview common motion on an owned clone to select the
                    // same-callback arrival continuation. Only one result is
                    // committed; no external clock, RNG or audio is consumed.
                    let mut arrival_world=staged.clone();
                    let motion=arrival_world.step_architect_motion(id as u16,frame,context,physics,trig)?;
                    if motion.arrived {
                        let receipt=arrival_world.finish_arrived_architect_callback(id as u16,frame,context,physics,trig,
                            staged_jobs,border,resource,(owner==selected_owner,selected_actor==Some(id as u16)),motion)?;
                        *staged=arrival_world;ConstructionAwareActorCallback::Architect{id:id as u16,receipt}
                    }else{
                        let idle=ActorIdleContext{runtime_mode,override_180d80,runtime_count_11c2bc:word(&staged.backing,0x355c)as u16,player_mode:staged.player(owner)?.bytes()[2]};
                        ConstructionAwareActorCallback::Ordinary(staged.step_actor_callback_with_idle(id as u16,frame,context,physics,trig,resource,idle)?)
                    }
                }else{
                    let idle=ActorIdleContext{runtime_mode,override_180d80,
                        runtime_count_11c2bc:word(&staged.backing,0x355c)as u16,
                        player_mode:staged.player(owner)?.bytes()[2]};
                    ConstructionAwareActorCallback::Ordinary(staged.step_actor_callback_with_idle(
                        id as u16,frame,context,physics,trig,resource,idle)?)
                })};
                let callback=match attempt(&mut staged,&mut staged_jobs){
                    Err(ConstructionWorldError::Setup(crate::construction_setup::ArchitectSetupError::UnsupportedSite(class @ (crate::construction_site::ConstructionSiteClass::Blocked|crate::construction_site::ConstructionSiteClass::Water))))=>{
                        let rejection=if class==crate::construction_site::ConstructionSiteClass::Water{crate::construction_cancellation::SiteRejection::Water}else{crate::construction_cancellation::SiteRejection::Blocked};
                        let idle=ActorIdleContext{runtime_mode,override_180d80,runtime_count_11c2bc:word(&staged.backing,0x355c)as u16,player_mode:staged.player(owner)?.bytes()[2]};
                        ConstructionAwareActorCallback::Architect{id:id as u16,receipt:staged.step_rejected_site_callback(id as u16,frame,context,physics,trig,border,resource,idle,(owner==selected_owner,selected_actor==Some(id as u16)),rejection)?}
                    },other=>other?,
                };
                let branch=match &callback {
                    ConstructionAwareActorCallback::Ordinary(r)=>ActorObservedBranch::Ordinary{source_dispatch:r.source_dispatch,idle_scorer_empty:r.idle_scorer_empty},
                    ConstructionAwareActorCallback::Architect{receipt:r,..}=>ActorObservedBranch::Architect{source_dispatch:architect_observed_dispatch(r.motion.after[0x55],r.setup.as_ref().map(|s|s.after[0x55]),r.rejected_site_message_index.is_some(),r.cancellation.as_ref().map(|c|c.after[0x55])),setup_after_state:r.setup.as_ref().map(|s|s.after[0x55]),rejected_site:r.rejected_site_message_index.is_some()},
                };
                observations.push(ActorDispatchObservation{frame,actor_id:id as u16,before:observed_before,after:ActorDispatchSnapshot::capture(&staged,id as u16)?,branch});
                receipts.push(callback);
            }
        }
        *self=staged;*jobs=staged_jobs;for observation in &observations{observe(observation);}Ok(receipts)
    }
}

#[cfg(test)]
mod observer_tests {
 use super::*;
 #[test]
 fn architect_observer_requires_actual_pre_dispatch_state(){
  //Ordinary construction, pending setup and arriving setup select state9.
  assert_eq!(architect_observed_dispatch(9,None,false,None),0x534d0);
  assert_eq!(architect_observed_dispatch(1,Some(9),false,None),0x534d0);
  assert_eq!(architect_observed_dispatch(8,Some(9),false,None),0x534d0);
  //Literal2523: rejected507D0/51E80 keeps9, then534D0 returns on taskFF.
  assert_eq!(architect_observed_dispatch(8,None,true,Some(9)),0x534d0);
  //Missing or non9 postsetup evidence never inherits the callback's label.
  assert_eq!(architect_observed_dispatch(9,None,true,None),0);
  assert_eq!(architect_observed_dispatch(9,None,true,Some(1)),0);
  assert_eq!(architect_observed_dispatch(9,Some(1),false,None),0);
  assert_eq!(architect_observed_dispatch(1,None,false,None),0);
  assert_eq!(architect_observed_dispatch(8,None,false,None),0);
 }

 #[test]
 fn observer_preserves_two_actor_world_and_rolls_back_notifications(){
  let mut bytes=vec![0;343958];bytes[..4].copy_from_slice(&0x55336u32.to_le_bytes());bytes[337054..337058].copy_from_slice(&0x6beu32.to_le_bytes());
  for cell in 0..16384{let at=MAP_START+cell*14;bytes[at..at+2].copy_from_slice(&8192u16.to_le_bytes());bytes[at+2..at+4].copy_from_slice(&3136i16.to_le_bytes());bytes[at+5]=7;}
  let players=core::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,[4096,4096],b"fixture\0").unwrap());
  let mut initial=ActorWorld::from_parts(&ConqMissionSource::parse(&bytes).unwrap(),&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed:5428,ring:0,draw_count:0}).unwrap();
  let strings:Vec<_>=(0..128).map(|i|format!("name{i}")).collect();let borrowed:Vec<_>=strings.iter().map(|s|s.as_bytes()).collect();let names=crate::blueprint::BlueprintNames::new(&borrowed).unwrap();
  initial.regenerate_player_blueprints(0,[0;36],&names,true).unwrap();initial.prepare_fresh_player(0,5,ActorOriginSample{position:[19839,12927],height:6272,shade:0},[19839,12927]).unwrap();
  let trig=TrigTables::generate_pinned().unwrap();let mut resources=Vec::new();
  for id in 0..2{
   initial.activate_without_associated_unit(id,&trig,|_,_,_|Ok(PlacementEnvironment{terrain_class:7,blocker_weight:0})).unwrap();
   let before=*initial.actors.record(id as usize).unwrap();let mut a=before;a[0x55]=1;a[0x57]=0;a[0x58]=0;a[0x70]=1;a[0x71]=0;a[0x16]&=!0x68;a[0x49]=0;a[0x4a]=0;a[0x4f]=0;a[0x52]=0;a[0x39..0x45].fill(0);a[0x19]=8;a[0x1a..0x1f].fill(0);a[0x1c]=1;
   initial.actors.replace_record_checked(id as usize,&before,&a).unwrap();
   let key=initial.actors.bindings(id as usize).unwrap().animation_database.unwrap();let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../../private/integration/game-runtime/actors/key-{:02}.anb",key.flat_index));resources.push(ActorAnimationDatabase::load(key,std::fs::read(path).unwrap()).unwrap());
  }
  initial.admit_creatures(&[[0;124];300]).unwrap();
  let records=core::array::from_fn(|_|{let mut r=[0;176];r[12..14].copy_from_slice(&(-1i16).to_le_bytes());r});let jobs=TerrainJobPool::admit(records,0).unwrap();
  let context=LandingPhysicsContext{gravity:1024,water_height:1632};let physics=PhysicsTables::fixture();
  let mut off=initial.clone();let mut off_jobs=jobs.clone();let mut on=initial.clone();let mut on_jobs=jobs.clone();let mut observations=Vec::new();
  let a=off.step_actor_callbacks_with_construction(1,1,context,&physics,&trig,&resources,&mut off_jobs,&[0;8192],0,Some(0),0,0).unwrap();
  let b=on.step_actor_callbacks_with_construction_observed(1,1,context,&physics,&trig,&resources,&mut on_jobs,&[0;8192],0,Some(0),0,0,|o|observations.push(o.clone())).unwrap();
  assert_eq!(off,on);assert_eq!(off_jobs,on_jobs);assert_eq!(a,b);assert_eq!(observations.len(),2);assert_eq!(observations.iter().map(|o|o.before.category).collect::<Vec<_>>(),vec![0,2]);
  assert!(observations.iter().all(|o|o.before.blueprint_equal()&&o.after.blueprint_equal()&&matches!(o.branch,ActorObservedBranch::Ordinary{source_dispatch:0x52780,..})));
  let mut scorer_seen=false;
  for frame in 2..=128 {
   let off_receipts=off.step_actor_callbacks_with_construction(1,frame,context,&physics,&trig,&resources,&mut off_jobs,&[0;8192],0,Some(0),0,0).unwrap();
   let on_receipts=on.step_actor_callbacks_with_construction_observed(1,frame,context,&physics,&trig,&resources,&mut on_jobs,&[0;8192],0,Some(0),0,0,|o|{if matches!(o.branch,ActorObservedBranch::Ordinary{idle_scorer_empty:Some(true),..}){scorer_seen=true;}}).unwrap();
   assert_eq!(off,on);assert_eq!(off_jobs,on_jobs);assert_eq!(off_receipts,on_receipts);if scorer_seen{break;}
  }
  assert!(scorer_seen,"fixture must reach an actual empty-scorer discharge");
  let before=*initial.actors.record(1).unwrap();let mut bad=before;bad[0x55]=5;bad[0x56]=10;initial.actors.replace_record_checked(1,&before,&bad).unwrap();let saved=initial.clone();let mut late_jobs=jobs.clone();let mut seen=0;
  assert!(initial.step_actor_callbacks_with_construction_observed(1,1,context,&physics,&trig,&resources,&mut late_jobs,&[0;8192],0,Some(0),0,0,|_|seen+=1).is_err());assert_eq!(seen,0);assert_eq!(initial,saved);assert_eq!(late_jobs,jobs);
 }
}
