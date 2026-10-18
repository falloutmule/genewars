//! Atomic construction assignment/site preparation; later construction callbacks stay separate.
use super::*;
use crate::actor_construction::{self,ConstructionCommandError,ArchitectAssignmentPlan};
use crate::construction_setup::{self,ArchitectSetupError,ArchitectSetupPlan};
use crate::construction_site::ConstructionSiteError;
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum ConstructionWorldError {
    World(ActorWorldError), Assignment(ConstructionCommandError), Setup(ArchitectSetupError),
    Lifecycle(crate::actor_construction_lifecycle::ConstructionLifecycleError),
    Jobs(crate::terrain_job_pool::TerrainJobPoolError), JobCountMismatch,
    Payment(crate::construction_payment::ConstructionPaymentError),
    TerraformCompletion(crate::construction_terraform_completion::TerraformCompletionError),
    PendingTaskRequired,
    Cancellation(crate::construction_cancellation::ConstructionCancellationError),
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::BlueprintNames;
    fn world()->ActorWorld {
        let mut bytes=vec![0;343_958];bytes[..4].copy_from_slice(&0x55336u32.to_le_bytes());bytes[337_054..337_058].copy_from_slice(&0x6beu32.to_le_bytes());
        for c in 0..16384 {let p=MAP_START+c*14;bytes[p..p+2].copy_from_slice(&8192u16.to_le_bytes());bytes[p+2..p+4].copy_from_slice(&3136i16.to_le_bytes());bytes[p+5]=7;}
        let players=core::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,[4096,4096],b"fixture\0").unwrap());
        let mut world=ActorWorld::from_parts(&ConqMissionSource::parse(&bytes).unwrap(),&[[0;200];20],core::array::from_fn(|_|ActorHandleBindings::default()),players,BlueprintRng{seed:5428,ring:0,draw_count:0}).unwrap();
        let strings:Vec<_>=(0..128).map(|i|format!("name{i}")).collect();let borrowed:Vec<_>=strings.iter().map(|s|s.as_bytes()).collect();let names=BlueprintNames::new(&borrowed).unwrap();
        world.regenerate_player_blueprints(0,[0;36],&names,true).unwrap();world.prepare_fresh_player(0,5,ActorOriginSample{position:[19839,12927],height:6272,shade:0},[19839,12927]).unwrap();
        world.activate_without_associated_unit(0,&TrigTables::generate_pinned().unwrap(),|_,_,_|Ok(PlacementEnvironment{terrain_class:7,blocker_weight:0})).unwrap();
        let before=*world.actors.record(0).unwrap();let mut after=before;after[0x55]=1;world.actors.replace_record_checked(0,&before,&after).unwrap();world
    }
    #[test]
    fn construction_arrival_preserves_task_and_seed(){
        let mut world=world();let before=*world.actors.record(0).unwrap();let mut a=before;
        a[0x55]=9;a[0x57]=0;a[0x58]=1;a[0x7e]=0;a[0x82]=1;a[0x83]=1;a[0x70]=0x91;a[0x71]=0;a[0x16]&=!0x68;
        a[11..15].copy_from_slice(&before[7..11]);a[17..21].copy_from_slice(&6272i32.to_le_bytes());a[39..43].copy_from_slice(&6272i32.to_le_bytes());
        a[0x2b..0x2d].copy_from_slice(&64i16.to_le_bytes());a[0x2d..0x33].fill(0);a[0x39..0x3f].fill(0);a[0x3f..0x41].copy_from_slice(&960i16.to_le_bytes());a[0x41..0x45].fill(0);a[0x49]=16;a[0x4a]=0;a[0x4f]=0;a[0x52]=0;
        world.actors.replace_record_checked(0,&before,&a).unwrap();let rng=world.rng;
        let receipt=world.step_architect_motion(0,1,LandingPhysicsContext{gravity:1024,water_height:1632},&crate::physics_tables::PhysicsTables::fixture(),&TrigTables::generate_pinned().unwrap()).unwrap();
        assert!(receipt.arrived);assert_eq!(receipt.after[0x55],9);assert_eq!(receipt.after[0x82],1);assert_ne!(receipt.after[0x49]&32,0);assert_eq!(world.rng,rng);
    }
    #[test]
    fn assignment_site_setup_preserves_funds_and_rejected_repeat_is_atomic(){
        let mut world=world();let funds=world.players[0].bytes()[0x4a3..0x4ab].to_vec();
        let assigned=world.assign_architect_construction(0,0,0x100,true,true).unwrap();assert!(assigned.panel_refresh_pending);
        let snapshot=world.clone();assert!(world.assign_architect_construction(0,0,0x100,true,true).is_err());assert_eq!(world,snapshot);
        let plan=world.prepare_architect_construction_site(0,&[0;8192]).unwrap();assert_eq!(plan.after[0x55],9);assert_eq!(plan.after[0x83],1);
        assert_eq!(world.players[0].bytes()[0x4a3..0x4ab],funds);assert_ne!(world.map(),snapshot.map());
        let records=core::array::from_fn(|_|{let mut r=[0;176];r[12..14].copy_from_slice(&(-1i16).to_le_bytes());r});
        let mut jobs=crate::terrain_job_pool::TerrainJobPool::admit(records,0).unwrap();
        let before=*world.actors.record(0).unwrap();let mut unready=before;unready[0x49]&=!32;
        world.actors.replace_record_checked(0,&before,&unready).unwrap();let snapshot=world.clone();let jobs_before=jobs.clone();
        assert!(matches!(world.start_architect_terraform(0,&mut jobs,&[0;8192]),Err(ConstructionWorldError::Lifecycle(crate::actor_construction_lifecycle::ConstructionLifecycleError::MovementRequired))));
        assert_eq!(world,snapshot);assert_eq!(jobs,jobs_before);
    }
}
impl From<ActorWorldError> for ConstructionWorldError { fn from(e:ActorWorldError)->Self{Self::World(e)} }
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ArchitectMotionReceipt{pub before:[u8;200],pub after:[u8;200],pub arrived:bool,pub changed_cell:bool,pub water_effect:Option<u16>,pub pending_events:Vec<LandingEvent>}
impl ActorWorld {
    /// The actual6FB60 motion/arrival slice for construction. It does not run
    /// state8 arrival-to-idle controls;534D0 must see this actor's ready bit.
    pub fn step_architect_motion(&mut self,id:u16,frame:u32,context:LandingPhysicsContext,physics:&crate::physics_tables::PhysicsTables,trig:&TrigTables)
        ->Result<ArchitectMotionReceipt,ConstructionWorldError>{
        self.validate_links()?;
        let before=*self.actors.record(usize::from(id)).map_err(ActorWorldError::from)?;
        let blueprint=self.construction_blueprint(id)?;
        if before[6]!=3||before[0x70]&1==0||before[0x16]&0x20!=0||blueprint[0x16]!=0
            ||!(before[0x55]==9||(before[0x55]==1&&before[0x57]&3==1)){
            return Err(ActorWorldError::ActorMotion(crate::actor_motion::ActorMotionError::UnsupportedState).into());
        }
        let raw=[id as u8,(id>>8)as u8,3];let target=[word(&before,11),word(&before,13)];
        let counter=frame.wrapping_add(i32::from(word(&before,15))as u32);
        let idle_suppressed=if before[0x49]&16==0&&counter&31==0{(blueprint[0x1a]as i8)<=0||self.common_move_point_accepted(raw,[word(&before,7),word(&before,9)])?}else{false};
        let(prefix,working)=self.actor_route_prefix(raw,&before,counter,physics,trig,idle_suppressed)?;
        let(motion,water)=crate::creature_world::plan_common_nonflight_physics_with_water_tail(&prefix,self.map(),frame,context.gravity,context.water_height,physics,trig,
            |r,n,p|self.actor_contact_speed(r,n,p).map_err(|_|CreatureError::MotionEnvironment),
            |point|self.move_obstacle_at(point).map_err(|_|CreatureError::MotionEnvironment),true).map_err(ActorWorldError::from)?;
        let mut moved=motion.after;
        if moved[0x49]&3!=0&&crate::creature_world::common_distance_squared(self.map(),[word(&moved,7),word(&moved,9)],working).map_err(ActorWorldError::from)? as u32<=4096{moved[0x49]|=8;}
        let arrived=prefix[0x49]&3==0&&(word(&prefix,0x3f)>=1||prefix[0x49]&16!=0)&&word(&moved,0x3d)<=256
            &&i32::from_le_bytes(moved[0x27..0x2b].try_into().unwrap())<=i32::from_le_bytes(moved[0x11..0x15].try_into().unwrap())
            &&crate::creature_world::common_distance_squared(self.map(),[word(&moved,7),word(&moved,9)],target).map_err(ActorWorldError::from)? as u32<=1024;
        let mut staged=self.clone();staged.commit_actor_motion_record(id,&before,&moved)?;
        let mut water_effect=None;let mut pending_events=Vec::new();
        if let Some(request)=water{if request.code!=0x30{return Err(ActorWorldError::UnsupportedEffectEvent.into());}
            if let Some((slot,events))=staged.create_water_splash(request.position,request.physical_height)?{water_effect=Some(slot);pending_events.extend(events);}}
        if arrived{
            let current=*staged.actors.record(usize::from(id)).map_err(ActorWorldError::from)?;
            let ground=genewars_core::cd_candidate::interpolate_height(staged.map(),target[0],target[1]).map_err(|e|ActorWorldError::Terrain(e.to_string()))?;
            let shade=genewars_core::cd_candidate::interpolate_entity_shade(staged.map(),target[0],target[1]).map_err(|e|ActorWorldError::Terrain(e.to_string()))?;
            let snap=crate::actor_motion::plan_actor_arrival_snap(&current,target,ground,shade).map_err(ActorWorldError::from)?;
            staged.commit_actor_motion_record(id,&current,&snap)?;
        }
        staged.validate_links()?;let after=*staged.actors.record(usize::from(id)).map_err(ActorWorldError::from)?;
        let changed_cell=floor_cell([word(&before,7),word(&before,9)])!=floor_cell([word(&after,7),word(&after,9)]);
        *self=staged;Ok(ArchitectMotionReceipt{before,after,arrived,changed_cell,water_effect,pending_events})
    }
    pub fn cancel_insufficient_architect_selector1(&mut self,id:u16,owner_selected:bool,actor_selected:bool)
        ->Result<crate::construction_cancellation::ConstructionCancellationPlan,ConstructionWorldError>{
        let before=*self.actors.record(usize::from(id)).map_err(ActorWorldError::from)?;
        let blueprint=self.construction_blueprint(id)?;let player=self.player(before[24])?.bytes();
        let primary=i32::from_le_bytes(player[0x4a3..0x4a7].try_into().unwrap());
        let secondary=i32::from_le_bytes(player[0x4a7..0x4ab].try_into().unwrap());
        let plan=crate::construction_cancellation::plan_insufficient_selector1_cancellation(self.map(),&before,&blueprint,primary,secondary,owner_selected,actor_selected)
            .map_err(ConstructionWorldError::Cancellation)?;
        let mut staged=self.clone();
        staged.actors.replace_record_checked(usize::from(id),&before,&plan.after).map_err(ActorWorldError::from)?;
        staged.backing[MAP_START..MAP_START+MAP_BYTES].copy_from_slice(&plan.map_after);
        *self=staged;Ok(plan)
    }
    ///57F62 pending construction entry from an idle, enabled architect.
    /// Run after this callback's real movement and blueprint periodic update.
    pub fn begin_pending_architect_task(&mut self,id:u16,table:&[u8;8192])
        ->Result<ArchitectSetupPlan,ConstructionWorldError>{
        let before=*self.actors.record(usize::from(id)).map_err(ActorWorldError::from)?;
        if before[0x55]!=1||before[0x58]!=0||before[0x57]&3!=1||before[0x70]&1==0||before[0x71]&16!=0{
            return Err(ConstructionWorldError::PendingTaskRequired);
        }
        let mut staged=self.clone();let mut prefix=before;
        prefix[0x57]&=!1;prefix[0x58]|=1;prefix[0x7a]=0;
        staged.actors.replace_record_checked(usize::from(id),&before,&prefix).map_err(ActorWorldError::from)?;
        let plan=staged.prepare_architect_construction_site(id,table)?;
        *self=staged;Ok(plan)
    }
    pub fn complete_architect_terraform(&mut self,id:u16,jobs:&crate::terrain_job_pool::TerrainJobPool)
        ->Result<crate::construction_terraform_completion::TerraformCompletionPlan,ConstructionWorldError>{
        if self.backing[0x2ff7]!=jobs.count(){return Err(ConstructionWorldError::JobCountMismatch);}
        let actor=*self.actors.record(usize::from(id)).map_err(ActorWorldError::from)?;
        let blueprint=self.construction_blueprint(id)?;
        let job=jobs.records().get(usize::from(actor[0x90])).ok_or(ConstructionWorldError::Jobs(crate::terrain_job_pool::TerrainJobPoolError::InvalidSlot))?;
        let speeds=[960i16,576,1088,832];
        let speed=*speeds.get(usize::from(blueprint[0x15])).ok_or(ConstructionWorldError::TerraformCompletion(crate::construction_terraform_completion::TerraformCompletionError::UnsupportedActor))?;
        let plan=crate::construction_terraform_completion::plan_architect_completed_terraform(&actor,&blueprint,job,self.rng.seed,speed)
            .map_err(ConstructionWorldError::TerraformCompletion)?;
        let mut staged=self.clone();
        staged.actors.replace_record_checked(usize::from(id),&actor,&plan.after).map_err(ActorWorldError::from)?;
        staged.rng.seed=plan.seed_after;staged.rng.ring=staged.rng.ring.wrapping_add(2)&31;staged.rng.draw_count=staged.rng.draw_count.wrapping_add(2);
        *self=staged;Ok(plan)
    }
    /// Source53A33 paid transaction. Insufficiency leaves both stores untouched
    /// until51E80's terrain/reference cleanup is admitted.
    pub fn pay_architect_construction(&mut self,id:u16,physics:&crate::physics_tables::PhysicsTables)
        ->Result<crate::construction_payment::ConstructionPaymentPlan,ConstructionWorldError>{
        let actor=*self.actors.record(usize::from(id)).map_err(ActorWorldError::from)?;
        let blueprint=self.construction_blueprint(id)?;
        let player=*self.player(actor[24])?.bytes();
        let resource_owner=player[0x4a1];
        let refill=self.player(resource_owner)?.bytes()[2]!=0;
        let dx=word(&actor,0x8c).wrapping_sub(word(&actor,7)).wrapping_mul(2);
        let dy=word(&actor,0x8e).wrapping_sub(word(&actor,9)).wrapping_mul(2);
        let yaw=crate::actor_motion::angle(i32::from(dx),i32::from(dy),physics.angles());
        let plan=crate::construction_payment::plan_funded_construction_payment(&actor,&blueprint,&player,refill,yaw)
            .map_err(ConstructionWorldError::Payment)?;
        let mut staged=self.clone();
        staged.actors.replace_record_checked(usize::from(id),&actor,&plan.actor_after).map_err(ActorWorldError::from)?;
        staged.players[usize::from(actor[24])].replace_bookkeeping_checked(&player,&plan.player_after).map_err(ActorWorldError::from)?;
        *self=staged;Ok(plan)
    }
    /// Start the actual stage0 job using a separately admitted native job store.
    /// Both stores commit together; movement readiness comes from the actor itself.
    pub fn start_architect_terraform(&mut self,id:u16,jobs:&mut crate::terrain_job_pool::TerrainJobPool,table:&[u8;8192])
        ->Result<crate::actor_construction_lifecycle::TerraformStartPlan,ConstructionWorldError>{
        const JOB_COUNT:usize=0x2ff7; // 11BD57 - mission backing base118D60.
        if self.backing[JOB_COUNT]!=jobs.count(){return Err(ConstructionWorldError::JobCountMismatch);}
        let before=*self.actors.record(usize::from(id)).map_err(ActorWorldError::from)?;
        self.construction_blueprint(id)?;
        let plan=crate::actor_construction_lifecycle::plan_architect_terraform_start(self.map(),&before,jobs.records(),jobs.count(),table)
            .map_err(ConstructionWorldError::Lifecycle)?;
        let mut staged=self.clone();let mut staged_jobs=jobs.clone();
        if let Some(allocation)=&plan.job{staged_jobs.apply_allocation(allocation).map_err(ConstructionWorldError::Jobs)?;}
        staged.actors.replace_record_checked(usize::from(id),&before,&plan.after).map_err(ActorWorldError::from)?;
        staged.backing[MAP_START..MAP_START+MAP_BYTES].copy_from_slice(&plan.map_after);
        staged.backing[JOB_COUNT]=staged_jobs.count();
        *self=staged;*jobs=staged_jobs;Ok(plan)
    }
    fn construction_blueprint(&self,id:u16)->Result<[u8;36],ConstructionWorldError>{
        let actor=self.actors.record(usize::from(id)).map_err(ActorWorldError::from)?;
        let handle=self.actors.bindings(usize::from(id)).map_err(ActorWorldError::from)?.blueprint
            .ok_or(ActorWorldError::MissingBlueprintBinding(id as i16))?;
        if handle.player_id!=actor[24] { return Err(ActorWorldError::ActorOwner(id as i16).into()); }
        Ok(*self.player(handle.player_id)?.blueprint(usize::from(handle.blueprint_index)).map_err(ActorWorldError::from)?)
    }
    /// Source opcode5 numeric assignment; selected UI refresh is a receipt request.
    pub fn assign_architect_construction(&mut self,id:u16,mode:u16,argument:u16,owner_selected:bool,actor_selected:bool)
        ->Result<ArchitectAssignmentPlan,ConstructionWorldError>{
        let before=*self.actors.record(usize::from(id)).map_err(ActorWorldError::from)?;
        let blueprint=self.construction_blueprint(id)?;
        let plan=actor_construction::plan_architect_assignment(&before,&blueprint,mode,argument,owner_selected,actor_selected)
            .map_err(ConstructionWorldError::Assignment)?;
        let mut staged=self.clone();
        staged.actors.replace_record_checked(usize::from(id),&before,&plan.after).map_err(ActorWorldError::from)?;
        *self=staged;Ok(plan)
    }
    /// Source507D0 site/task setup. The query walks actual intrusive cell links:
    /// 8B340 with mask0 admits the first kind2 plant (virtual68 false).
    pub fn prepare_architect_construction_site(&mut self,id:u16,table:&[u8;8192])
        ->Result<ArchitectSetupPlan,ConstructionWorldError>{
        let before=*self.actors.record(usize::from(id)).map_err(ActorWorldError::from)?;
        let blueprint=self.construction_blueprint(id)?;
        let plan=construction_setup::plan_architect_setup(self.map(),&before,&blueprint,table,|cell|{
            let mut raw=reference(self.map(),usize::from(cell)*14+6);
            let mut visited=std::collections::BTreeSet::new();
            while raw[2]!=0 {
                if !visited.insert(raw){return Err(ConstructionSiteError::QueryFailed);}
                let entity=self.entity(raw).map_err(|_|ConstructionSiteError::QueryFailed)?;
                if raw[2]==2{return Ok(true);}
                raw=reference(entity,3);
            }
            Ok(false)
        }).map_err(ConstructionWorldError::Setup)?;
        let mut staged=self.clone();
        staged.actors.replace_record_checked(usize::from(id),&before,&plan.after).map_err(ActorWorldError::from)?;
        staged.backing[MAP_START..MAP_START+MAP_BYTES].copy_from_slice(&plan.map_after);
        *self=staged;Ok(plan)
    }
}
