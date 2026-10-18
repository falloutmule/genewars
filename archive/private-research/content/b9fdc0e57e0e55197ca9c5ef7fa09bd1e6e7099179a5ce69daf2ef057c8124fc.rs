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
    Stage4(crate::construction_stage4::ConstructionStage4Error),
    BuildingEffect(crate::building_effect::BuildingEffectError),
    EffectReplacementRequired,EffectPositionUnsupported,BuildingFactoryReturnedNull,
    TaskMovement(crate::construction_motion::ConstructionMotionError),
    Wait(crate::construction_wait::TerraformWaitError),
    Resume(crate::construction_terraform_resume::TerraformResumeError),
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
    fn stage0_callback_allocates_and_late_tail_failure_rolls_back(){
        let mut world=world();let old=*world.actors.record(0).unwrap();let mut a=old;
        a[0x55]=9;a[0x57]=0;a[0x58]=1;a[0x7e]=0;a[0x82]=1;a[0x83]=1;
        a[0x88..0x8a].fill(0);a[0x84..0x86].copy_from_slice(&7i16.to_le_bytes());
        a[0x8a..0x8c].copy_from_slice(&3136i16.to_le_bytes());a[0x8c..0x90].copy_from_slice(&old[7..11]);
        a[0x70]=0x11;a[0x71]=0;a[0x16]&=!0x68;a[11..15].copy_from_slice(&old[7..11]);
        a[17..21].copy_from_slice(&6272i32.to_le_bytes());a[39..43].copy_from_slice(&6272i32.to_le_bytes());
        a[0x2b..0x2d].copy_from_slice(&64i16.to_le_bytes());a[0x2d..0x33].fill(0);
        a[0x39..0x3f].fill(0);a[0x3f..0x41].copy_from_slice(&960i16.to_le_bytes());
        a[0x41..0x45].fill(0);a[0x49]=16;a[0x4a]=0;a[0x4f]=0;a[0x52]=0;
        world.actors.replace_record_checked(0,&old,&a).unwrap();
        let key=world.actors.bindings(0).unwrap().animation_database.unwrap();
        let mut jobs=crate::terrain_job_pool::TerrainJobPool::admit(std::array::from_fn(|_|{let mut r=[0;176];r[12..14].copy_from_slice(&(-1i16).to_le_bytes());r}),0).unwrap();
        let snapshot=world.clone();let jobs_before=jobs.clone();let physics=crate::physics_tables::PhysicsTables::fixture();let trig=TrigTables::generate_pinned().unwrap();
        let context=LandingPhysicsContext{gravity:1024,water_height:1632};
        assert!(world.step_architect_stage0_inner(0,1,context,&physics,&trig,&mut jobs,&[0;8192],key,(false,false),
            |_|Ok((Vec::new(),0))).is_err());
        assert_eq!(world,snapshot);assert_eq!(jobs,jobs_before);
        let funds=world.players[0].bytes()[0x4a3..0x4ab].to_vec();let rng=world.rng;
        let receipt=world.step_architect_stage0_inner(0,1,context,&physics,&trig,&mut jobs,&[0;8192],key,(false,false),
            |animation|{assert_eq!(animation,13);Ok((vec![0;8],0))}).unwrap();
        assert!(receipt.motion.arrived);assert!(receipt.terraform.as_ref().unwrap().job.is_some());
        assert_eq!(jobs.count(),1);assert_eq!(world.backing[0x2ff7],1);
        assert_eq!(world.actors.record(0).unwrap()[0x88],1);
        assert_eq!(world.players[0].bytes()[0x4a3..0x4ab],funds);assert_eq!(world.rng,rng);
        assert!(receipt.footstep.is_none());
        let wait=world.step_architect_stage0_inner(0,2,context,&physics,&trig,&mut jobs,&[0;8192],key,(false,false),
            |_|Ok((vec![0;8],0))).unwrap();
        assert!(wait.terraform_wait.is_some());assert!(wait.terraform_completion.is_none());
        let environment=crate::construction_maintenance::TerrainEnvironment{climate_center:[0,0],raw_water_height:816,climate:106,light:[181,0,-181]};
        for _ in 0..8{if jobs.count()==0{break;}world.step_terrain_jobs_without_height_changes(&mut jobs,environment,&[0;8192]).unwrap();}
        assert_eq!(jobs.count(),0);assert_eq!(world.backing[0x2ff7],0);
        let rng=world.rng;
        let completed=world.step_architect_stage0_inner(0,3,context,&physics,&trig,&mut jobs,&[0;8192],key,(false,false),
            |_|Ok((vec![0;8],0))).unwrap();
        assert!(completed.terraform_completion.is_some());assert!(completed.terraform_wait.is_none());
        assert_eq!(world.actors.record(0).unwrap()[0x88],3);
        assert_eq!(world.players[0].bytes()[0x4a3..0x4ab],funds);
        assert_eq!(world.rng.draw_count,rng.draw_count.wrapping_add(2));
        assert_eq!(world.rng.seed,rng.seed.wrapping_mul(0xe62d).wrapping_mul(0xe62d));
    }
    #[test]
    fn assigned_centered_architect_completes_flat_terraform_cycle(){
        let mut world=world();
        //Explicit centered idle fixture; preserve zero desired speed/readiness.
        let before=*world.actors.record(0).unwrap();let mut centered=before;
        centered[7..9].copy_from_slice(&19839i16.to_le_bytes());centered[9..11].copy_from_slice(&12927i16.to_le_bytes());
        let point=centered[7..11].to_vec();centered[11..15].copy_from_slice(&point);
        world.commit_actor_motion_record(0,&before,&centered).unwrap();
        assert_eq!(word(&centered,0x3f),0);
        world.assign_architect_construction(0,0,0x100,false,false).unwrap();
        world.begin_pending_architect_task(0,&[0;8192]).unwrap();
        let key=world.actors.bindings(0).unwrap().animation_database.unwrap();
        let records=std::array::from_fn(|_|{let mut r=[0;176];r[12..14].copy_from_slice(&(-1i16).to_le_bytes());r});
        let mut jobs=crate::terrain_job_pool::TerrainJobPool::admit(records,0).unwrap();
        let physics=crate::physics_tables::PhysicsTables::fixture();let trig=TrigTables::generate_pinned().unwrap();
        let context=LandingPhysicsContext{gravity:1024,water_height:1632};
        let environment=crate::construction_maintenance::TerrainEnvironment{climate_center:[0,0],raw_water_height:816,climate:106,light:[181,0,-181]};
        let funds=world.players[0].bytes()[0x4a3..0x4ab].to_vec();let initial_rng=world.rng;
        let mut allocated=false;let mut completed=false;
        for frame in 1..=300{
            let receipt=world.step_architect_stage0_inner(0,frame,context,&physics,&trig,&mut jobs,&[0;8192],key,(false,false),|_|Ok((vec![0;8],0))).unwrap();
            allocated|=receipt.terraform.as_ref().is_some_and(|p|p.job.is_some());
            completed|=receipt.terraform_completion.is_some();
            if completed{break;}
            world.step_terrain_jobs_without_height_changes(&mut jobs,environment,&[0;8192]).unwrap();
        }
        assert!(allocated&&completed);assert_eq!(jobs.count(),0);
        assert_eq!(world.actors.record(0).unwrap()[0x88],3);
        assert_eq!(world.players[0].bytes()[0x4a3..0x4ab],funds);
        assert_eq!(world.rng.draw_count,initial_rng.draw_count.wrapping_add(2));
    }
    #[test]
    fn stage3_payment_and_cancellation_roll_back_on_late_animation_failure(){
        for funds in [0i32,1000]{
            let mut world=world();let original=*world.actors.record(0).unwrap();let mut a=original;
            a[0x55]=9;a[0x57]=0;a[0x58]=1;a[0x7e]=0;a[0x82]=1;a[0x83]=1;a[0x59]=13;
            a[0x88..0x8a].copy_from_slice(&3i16.to_le_bytes());a[0x70]=0x11;a[0x71]=0;a[0x16]&=!0x68;
            a[11..15].copy_from_slice(&original[7..11]);a[0x8c..0x90].copy_from_slice(&original[7..11]);
            a[17..21].copy_from_slice(&6272i32.to_le_bytes());a[39..43].copy_from_slice(&6272i32.to_le_bytes());
            a[0x2b..0x2d].copy_from_slice(&64i16.to_le_bytes());a[0x2d..0x33].fill(0);
            a[0x39..0x3f].fill(0);a[0x3f..0x41].copy_from_slice(&960i16.to_le_bytes());a[0x41..0x45].fill(0);a[0x49]=16;a[0x4a]=8;a[0x4f]=0;a[0x52]=0;
            world.actors.replace_record_checked(0,&original,&a).unwrap();
            let mut player=*world.players[0].bytes();player[0x4a3..0x4a7].copy_from_slice(&funds.to_le_bytes());world.players[0]=PlayerState::from_record(&player).unwrap();
            let key=world.actors.bindings(0).unwrap().animation_database.unwrap();
            let mut jobs=crate::terrain_job_pool::TerrainJobPool::admit(std::array::from_fn(|_|{let mut r=[0;176];r[12..14].copy_from_slice(&(-1i16).to_le_bytes());r}),0).unwrap();
            let saved=world.clone();let saved_jobs=jobs.clone();let rng=world.rng;
            let physics=crate::physics_tables::PhysicsTables::fixture();let trig=TrigTables::generate_pinned().unwrap();let context=LandingPhysicsContext{gravity:1024,water_height:1632};
            assert!(world.step_architect_stage0_inner(0,1,context,&physics,&trig,&mut jobs,&[0;8192],key,(false,false),|_|Ok((Vec::new(),0))).is_err());
            assert_eq!(world,saved);assert_eq!(jobs,saved_jobs);
            let receipt=world.step_architect_stage0_inner(0,1,context,&physics,&trig,&mut jobs,&[0;8192],key,(false,false),|_|Ok((vec![0;23],0))).unwrap();
            if funds==0{assert!(receipt.cancellation.is_some());assert_eq!(world.actors.record(0).unwrap()[0x82],0);assert_eq!(world.actors.record(0).unwrap()[0x7e],255);}
            else{assert_eq!(receipt.payment.unwrap().cost,688);assert_eq!(world.actors.record(0).unwrap()[0x19],14);}
            assert_eq!(i32::from_le_bytes(world.players[0].bytes()[0x4a3..0x4a7].try_into().unwrap()),if funds==0{0}else{312});assert_eq!(world.rng,rng);
        }
    }
    #[test]
    fn stage4_factory_progress_and_late_tail_rollback(){
            let mut world=world();let original=*world.actors.record(0).unwrap();let mut a=original;
            a[0x55]=9;a[0x57]=0;a[0x58]=1;a[0x7e]=0;a[0x82]=1;a[0x83]=1;a[0x59]=14;
            a[0x88..0x8a].copy_from_slice(&4i16.to_le_bytes());a[0x70]=0x11;a[0x71]=0;a[0x16]&=!0x68;
            a[11..15].copy_from_slice(&original[7..11]);a[0x8c..0x90].copy_from_slice(&original[7..11]);
            a[17..21].copy_from_slice(&6272i32.to_le_bytes());a[39..43].copy_from_slice(&6272i32.to_le_bytes());
            a[0x2b..0x2d].copy_from_slice(&64i16.to_le_bytes());a[0x2d..0x33].fill(0);
            a[0x39..0x3f].fill(0);a[0x3f..0x41].copy_from_slice(&960i16.to_le_bytes());a[0x41..0x45].fill(0);a[0x49]=16;a[0x4a]=8;a[0x4f]=0;a[0x52]=0;
            world.actors.replace_record_checked(0,&original,&a).unwrap();
            let before=*world.actors.record(0).unwrap();let mut a=before;a[0x19]=14;a[0x1a]=0;a[0x1b]=22;a[0x1e]=0;
            world.actors.replace_record_checked(0,&before,&a).unwrap();
            world.admit_creatures(&[[0;124];300]).unwrap();world.admit_buildings(&[[0xa5;67];256],[None;256]).unwrap();world.admit_effects(&[[0;81];200],[None;200]).unwrap();
            let key=world.actors.bindings(0).unwrap().animation_database.unwrap();
            let mut jobs=crate::terrain_job_pool::TerrainJobPool::admit(std::array::from_fn(|_|{let mut r=[0;176];r[12..14].copy_from_slice(&(-1i16).to_le_bytes());r}),0).unwrap();
            let saved=world.clone();let saved_jobs=jobs.clone();let rng=world.rng;
            let physics=crate::physics_tables::PhysicsTables::fixture();let trig=TrigTables::generate_pinned().unwrap();let context=LandingPhysicsContext{gravity:1024,water_height:1632};
            let mut metadata_calls=0;
            assert!(world.step_architect_stage0_inner(0,1,context,&physics,&trig,&mut jobs,&[0;8192],key,(false,false),|_|{metadata_calls+=1;Ok((if metadata_calls==2{Vec::new()}else{vec![0;23]},0))}).is_err());
            assert_eq!(world,saved);assert_eq!(jobs,saved_jobs);
            let result=world.step_architect_stage0_inner(0,1,context,&physics,&trig,&mut jobs,&[0;8192],key,(false,false),|animation|Ok((vec![0;if animation==15{8}else{23}],0))).unwrap();
            let created=result.building_effect.unwrap();assert_eq!(created.slot,0);assert_eq!(created.building.slot,0);
            assert_eq!(word(world.actors.record(0).unwrap(),0x88),5);assert_eq!(world.actors.record(0).unwrap()[0x59],15);assert!(result.selected_animation<8);
            assert_eq!(word(world.buildings.as_ref().unwrap().record(0).unwrap(),0x2b),1);assert_eq!(world.rng,rng);
    }
    #[test]
    fn code2c_factory_links_native_child_and_rolls_back_late_stale_binding(){
        for stale in [false,true]{
            let mut world=world();world.admit_creatures(&[[0;124];300]).unwrap();world.admit_buildings(&[[0xa5;67];256],[None;256]).unwrap();
            let mut bindings=[None;200];if stale{bindings[0]=Some(crate::effect_pool::EffectAnimationBinding::EffAnimation44);}
            world.admit_effects(&[[0;81];200],bindings).unwrap();let saved=world.clone();let rng=world.rng;
            let result=world.create_building_effect(0,1,[19839,12927],6272);
            if stale{assert!(result.is_err());assert_eq!(world,saved);continue;}
            let receipt=result.unwrap().unwrap();assert_eq!(receipt.slot,0);assert_eq!(receipt.building.slot,0);
            assert_eq!(world.effects.as_ref().unwrap().binding(0).unwrap(),Some(crate::effect_pool::EffectAnimationBinding::EffAnimation44));
            assert_eq!(&world.effects.as_ref().unwrap().record(0).unwrap()[71..74],&[0,0,4]);
            assert_eq!(world.player(0).unwrap().bytes()[0x4a2],1);assert_eq!(word(&world.backing,0x3554),1);assert_eq!(world.rng,rng);
            let saved=world.clone();assert!(matches!(world.create_building_effect(0,1,[19966,12927],6272),Err(ConstructionWorldError::EffectPositionUnsupported)));assert_eq!(world,saved);
        }
    }
    #[test]
    fn code2c_full_pool_requires_source_priority_replacement_or_returns_null(){
        let mut world=world();world.admit_effects(&[[0;81];200],[None;200]).unwrap();
        let trig=TrigTables::generate_pinned().unwrap();let context=LandingPhysicsContext{gravity:1024,water_height:1632};
        for _ in 0..200{world.create_unmarked_smoke([19839,12927],6272,[0,0,0],context,&trig).unwrap().unwrap();}
        let saved=world.clone();assert!(matches!(world.create_building_effect(0,1,[19839,12927],6272),Err(ConstructionWorldError::EffectReplacementRequired)));assert_eq!(world,saved);
        for slot in 0..200{let before=*world.effects.as_ref().unwrap().record(slot).unwrap();let mut after=before;after[21]|=1;world.effects.as_mut().unwrap().replace_record_checked(slot,&before,&after).unwrap();}
        let saved=world.clone();assert!(world.create_building_effect(0,1,[19839,12927],6272).unwrap().is_none());assert_eq!(world,saved);
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
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ArchitectStage0Receipt {
    pub motion:ArchitectMotionReceipt,
    pub setup:Option<ArchitectSetupPlan>,
    pub terraform:Option<crate::actor_construction_lifecycle::TerraformStartPlan>,
    pub payment:Option<crate::construction_payment::ConstructionPaymentPlan>,
    pub cancellation:Option<crate::construction_cancellation::ConstructionCancellationPlan>,
    pub building_effect:Option<BuildingEffectCreationReceipt>,
    pub progress:Option<crate::building_progress::BuildingProgressPlan>,
    pub terraform_wait:Option<crate::construction_wait::TerraformWaitPlan>,
    pub terraform_completion:Option<crate::construction_terraform_completion::TerraformCompletionPlan>,
    pub selected_animation:u8,
    pub footstep:Option<crate::actor_motion::ActorFootstep>,
    pub created_effect:Option<u16>,
    pub factory_draw:Option<crate::landing::LandingDraw>,
    pub queue_requested:bool,
}
impl ActorWorld {
    pub fn step_architect_stage0_callback<B:AsRef<[u8]>>(&mut self,id:u16,frame:u32,
        context:LandingPhysicsContext,physics:&crate::physics_tables::PhysicsTables,trig:&TrigTables,
        jobs:&mut crate::terrain_job_pool::TerrainJobPool,border:&[u8;8192],resource:&crate::actor_resources::ActorAnimationDatabase<B>)
        ->Result<ArchitectStage0Receipt,ConstructionWorldError>{
        let before=self.actors.record(usize::from(id)).map_err(ActorWorldError::from)?;
        if word(before,0x88)!=0{return Err(ConstructionWorldError::PendingTaskRequired);}
        self.step_architect_terraform_callback(id,frame,context,physics,trig,jobs,border,resource)
    }
    /// Strict terraform-only entry; payment/cancellation requires the explicit
    /// construction callback with its selected-UI context.
    pub fn step_architect_terraform_callback<B:AsRef<[u8]>>(&mut self,id:u16,frame:u32,
        context:LandingPhysicsContext,physics:&crate::physics_tables::PhysicsTables,trig:&TrigTables,
        jobs:&mut crate::terrain_job_pool::TerrainJobPool,border:&[u8;8192],resource:&crate::actor_resources::ActorAnimationDatabase<B>)
        ->Result<ArchitectStage0Receipt,ConstructionWorldError>{
        let before=self.actors.record(usize::from(id)).map_err(ActorWorldError::from)?;
        if !(0..=2).contains(&word(before,0x88)){return Err(ConstructionWorldError::PendingTaskRequired);}
        self.step_architect_construction_callback(id,frame,context,physics,trig,jobs,border,resource,false,false)
    }
    /// Ordinary construction stages0..4 with actual movement/readiness. Later
    /// construction stages reject before mutation. The world and job pool commit together.
    pub fn step_architect_construction_callback<B:AsRef<[u8]>>(&mut self,id:u16,frame:u32,
        context:LandingPhysicsContext,physics:&crate::physics_tables::PhysicsTables,
        trig:&TrigTables,jobs:&mut crate::terrain_job_pool::TerrainJobPool,border:&[u8;8192],
        resource:&crate::actor_resources::ActorAnimationDatabase<B>,owner_selected:bool,actor_selected:bool)
        ->Result<ArchitectStage0Receipt,ConstructionWorldError>{
        self.step_architect_stage0_inner(id,frame,context,physics,trig,jobs,border,resource.key(),(owner_selected,actor_selected),|animation|{
            let count=resource.frame_count(usize::from(animation)).map_err(|_|ActorWorldError::ActorMotion(crate::actor_motion::ActorMotionError::InvalidAnimation))?;
            let holds=(0..count).map(|i|resource.frame(usize::from(animation),i).map(|f|f.byte_10))
                .collect::<Result<Vec<_>,_>>().map_err(|_|ActorWorldError::ActorMotion(crate::actor_motion::ActorMotionError::InvalidAnimation))?;
            let flags=(resource.animation(usize::from(animation)).map_err(|_|ActorWorldError::ActorMotion(crate::actor_motion::ActorMotionError::InvalidAnimation))?.trailing_word>>8)as u8;
            Ok((holds,flags))
        })
    }
    /// Whole pending state1 callback: common motion, source setup, stage0 and tail.
    /// World and external terrain jobs commit together after every dependency succeeds.
    pub fn step_pending_architect_callback<B:AsRef<[u8]>>(&mut self,id:u16,frame:u32,
        context:LandingPhysicsContext,physics:&crate::physics_tables::PhysicsTables,trig:&TrigTables,
        jobs:&mut crate::terrain_job_pool::TerrainJobPool,border:&[u8;8192],
        resource:&crate::actor_resources::ActorAnimationDatabase<B>,owner_selected:bool,actor_selected:bool)
        ->Result<ArchitectStage0Receipt,ConstructionWorldError>{
        let before=self.actors.record(usize::from(id)).map_err(ActorWorldError::from)?;
        if before[0x55]!=1||before[0x58]!=0||before[0x57]&3!=1||before[0x7e]!=0||before[0x82]!=1||before[0x83]!=1{
            return Err(ConstructionWorldError::PendingTaskRequired);
        }
        let mut staged=self.clone();let mut staged_jobs=jobs.clone();
        let motion=staged.step_architect_motion(id,frame,context,physics,trig)?;
        let setup=staged.begin_pending_architect_task(id,border)?;
        let mut result=staged.step_architect_after_motion(id,frame,context,physics,trig,&mut staged_jobs,border,resource.key(),(owner_selected,actor_selected),Some(motion),|animation|{
            let count=resource.frame_count(usize::from(animation)).map_err(|_|ActorWorldError::ActorMotion(crate::actor_motion::ActorMotionError::InvalidAnimation))?;
            let holds=(0..count).map(|i|resource.frame(usize::from(animation),i).map(|f|f.byte_10)).collect::<Result<Vec<_>,_>>().map_err(|_|ActorWorldError::ActorMotion(crate::actor_motion::ActorMotionError::InvalidAnimation))?;
            let flags=(resource.animation(usize::from(animation)).map_err(|_|ActorWorldError::ActorMotion(crate::actor_motion::ActorMotionError::InvalidAnimation))?.trailing_word>>8)as u8;
            Ok((holds,flags))
        })?;
        result.setup=Some(setup);*self=staged;*jobs=staged_jobs;Ok(result)
    }
    fn step_architect_stage0_inner<F>(&mut self,id:u16,frame:u32,context:LandingPhysicsContext,
        physics:&crate::physics_tables::PhysicsTables,trig:&TrigTables,
        jobs:&mut crate::terrain_job_pool::TerrainJobPool,border:&[u8;8192],
        key:crate::actor_initialization::ActorResourceTableKey,selection:(bool,bool),metadata:F)
        ->Result<ArchitectStage0Receipt,ConstructionWorldError>
        where F:FnMut(u8)->Result<(Vec<u8>,u8),ActorWorldError>{
        self.step_architect_after_motion(id,frame,context,physics,trig,jobs,border,key,selection,None,metadata)
    }
    fn step_architect_after_motion<F>(&mut self,id:u16,frame:u32,context:LandingPhysicsContext,
        physics:&crate::physics_tables::PhysicsTables,trig:&TrigTables,
        jobs:&mut crate::terrain_job_pool::TerrainJobPool,border:&[u8;8192],
        key:crate::actor_initialization::ActorResourceTableKey,selection:(bool,bool),motion:Option<ArchitectMotionReceipt>,mut metadata:F)
        ->Result<ArchitectStage0Receipt,ConstructionWorldError>
        where F:FnMut(u8)->Result<(Vec<u8>,u8),ActorWorldError>{
        let before=*self.actors.record(usize::from(id)).map_err(ActorWorldError::from)?;
        let blueprint=self.construction_blueprint(id)?;
        let bindings=self.actors.bindings(usize::from(id)).map_err(ActorWorldError::from)?;
        if before[0x55]!=9||before[0x7e]!=0||before[0x82]!=1||!(0..=4).contains(&word(&before,0x88))
            ||blueprint[0x15]>=3||blueprint[0x16]!=0||bindings.animation_database!=Some(key){
            return Err(ConstructionWorldError::PendingTaskRequired);
        }
        let handle=bindings.blueprint.ok_or(ActorWorldError::MissingBlueprintBinding(id as i16))?;
        let mut staged=self.clone();let mut staged_jobs=jobs.clone();
        let motion=match motion{Some(value)=>value,None=>staged.step_architect_motion(id,frame,context,physics,trig)?};
        //57A10's periodic blueprint recovery precedes the action dispatch.
        let mut blueprint_after=blueprint;
        let mask=if blueprint[0x15]==2{127}else{255};
        if frame.wrapping_add(i32::from(word(&before,15))as u32)&mask==0
            &&(blueprint[0x1a]as i8)<(blueprint[0x19]as i8){
            blueprint_after[0x1a]=blueprint_after[0x1a].wrapping_add(1);
        }
        staged.players[usize::from(handle.player_id)].commit_activation_blueprint(
            usize::from(handle.blueprint_index),&blueprint,&blueprint_after).map_err(ActorWorldError::from)?;
        let current=*staged.actors.record(usize::from(id)).map_err(ActorWorldError::from)?;
        if staged.backing[0x2ff7]!=staged_jobs.count(){return Err(ConstructionWorldError::JobCountMismatch);}
        let mut terraform_wait=None;let mut terraform_completion=None;let mut payment=None;let mut cancellation=None;let mut building_effect=None;let mut progress=None;
        let terraform=if word(&current,0x88)==0{
            let start=if current[0x49]&32!=0{
            Some(staged.start_architect_terraform(id,&mut staged_jobs,border)?)
        }else{
            let plan=crate::construction_motion::plan_architect_task_movement(&current,&blueprint_after,physics.angles())
                .map_err(ConstructionWorldError::TaskMovement)?;
            staged.actors.replace_record_checked(usize::from(id),&current,&plan.after).map_err(ActorWorldError::from)?;
            None
            };
            start

        }else if word(&current,0x88)<=2{
            let slot=usize::from(current[0x90]);
            let job=staged_jobs.records().get(slot).ok_or(ConstructionWorldError::Jobs(crate::terrain_job_pool::TerrainJobPoolError::InvalidSlot))?;
            if i16::from_le_bytes(job[12..14].try_into().unwrap())>=0{
                let distance=crate::creature_world::common_distance_squared(staged.map(),[word(&current,7),word(&current,9)],[word(&current,11),word(&current,13)]).map_err(ActorWorldError::from)? as u32;
                terraform_wait=Some(crate::construction_wait::plan_architect_terraform_wait(&current,job,distance).map_err(ConstructionWorldError::Wait)?);
            }else{
                let speed=[960i16,576,1088][usize::from(blueprint_after[0x15])];
                let plan=crate::construction_terraform_resume::plan_architect_terraform_resume(&current,&blueprint_after,job,staged.rng.seed,speed,physics.angles()).map_err(ConstructionWorldError::Resume)?;
                staged.actors.replace_record_checked(usize::from(id),&current,&plan.after).map_err(ActorWorldError::from)?;
                staged.rng.seed=plan.seed_after;staged.rng.ring=staged.rng.ring.wrapping_add(2)&31;staged.rng.draw_count=staged.rng.draw_count.wrapping_add(2);
                terraform_completion=Some(plan);
            }
            None
        }else if word(&current,0x88)==3{
            if current[0x49]&32!=0{
                match staged.pay_architect_construction(id,physics){
                    Ok(plan)=>payment=Some(plan),
                    Err(ConstructionWorldError::Payment(crate::construction_payment::ConstructionPaymentError::InsufficientCancellationRequired))=>{
                        cancellation=Some(staged.cancel_insufficient_architect_selector1(id,selection.0,selection.1)?);
                    },
                    Err(error)=>return Err(error),
                }
            }else{
                let plan=crate::construction_motion::plan_architect_task_movement(&current,&blueprint_after,physics.angles()).map_err(ConstructionWorldError::TaskMovement)?;
                staged.actors.replace_record_checked(usize::from(id),&current,&plan.after).map_err(ActorWorldError::from)?;
            }
            None
        }else{
            let (holds,flags)=metadata(14)?;
            let count=u8::try_from(holds.len()).map_err(|_|ConstructionWorldError::Stage4(crate::construction_stage4::ConstructionStage4Error::InvalidAnimation))?;
            let plan=crate::construction_stage4::plan_architect_stage4(&current,count,flags).map_err(ConstructionWorldError::Stage4)?;
            staged.actors.replace_record_checked(usize::from(id),&current,&plan.after).map_err(ActorWorldError::from)?;
            if let Some(request)=&plan.request{
                let (height,_)=staged.effect_ground_sample(request.position)?;
                if let Some(created)=staged.create_building_effect(request.owner,request.selector,request.position,height)?{
                    let building=*staged.buildings.as_ref().unwrap().record(usize::from(created.building.slot)).map_err(ActorWorldError::from)?;
                    let speed=[960i16,576,1088][usize::from(blueprint_after[0x15])];
                    let mut after=crate::construction_stage4::finish_architect_stage4(&plan,&building,speed).map_err(ConstructionWorldError::Stage4)?;
                    let linked=*staged.actors.record(usize::from(id)).map_err(ActorWorldError::from)?;
                    after[..6].copy_from_slice(&linked[..6]);
                    progress=Some(staged.progress_selector1_building(created.building.slot,1,false)?);
                    staged.actors.replace_record_checked(usize::from(id),&linked,&after).map_err(ActorWorldError::from)?;
                    building_effect=Some(created);
                }
            }
            None
        };
        let current=*staged.actors.record(usize::from(id)).map_err(ActorWorldError::from)?;
        let animation=crate::actor_construction_tail::architect_tail_animation(&current).map_err(ActorWorldError::from)?;
        let(holds,flags)=metadata(animation)?;
        let tail=crate::actor_construction_tail::plan_architect_callback_tail(&current,frame,&holds,flags).map_err(ActorWorldError::from)?;
        staged.actors.replace_record_checked(usize::from(id),&current,&tail.after).map_err(ActorWorldError::from)?;
        let mut footstep=None;let mut created_effect=None;let mut factory_draw=None;
        if let Some(point)=tail.footstep_query{
            let class=staged.terrain_class_at(point)?;
            if let Some(request)=crate::actor_motion::plan_actor_footstep(&tail.after,frame,class).map_err(ActorWorldError::from)?{
                if let Some((slot,draw))=staged.create_unmarked_smoke(request.position,request.physical_height,request.angles,context,trig)?{
                    created_effect=Some(slot);factory_draw=Some(draw);
                }
                footstep=Some(request);
            }
        }
        staged.validate_links()?;if staged.effects.is_some(){staged.validate_effect_count()?;}
        *self=staged;*jobs=staged_jobs;
        Ok(ArchitectStage0Receipt{motion,setup:None,terraform,payment,cancellation,building_effect,progress,terraform_wait,terraform_completion,selected_animation:animation,footstep,created_effect,factory_draw,queue_requested:true})
    }

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

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct BuildingEffectCreationReceipt {
    pub slot:u16,
    pub building:crate::building::BuildingFactoryPlan,
    pub effect:crate::building_effect::BuildingEffectPlan,
}
impl ActorWorld {
    ///48790 code2C free-slot branch, actual42FA0 and source map insertion.
    ///Eligible full-pool replacement and unbound building results are explicit.
    pub fn create_building_effect(&mut self,owner:u8,selector:u8,position:[i16;2],physical_height:i32)
        ->Result<Option<BuildingEffectCreationReceipt>,ConstructionWorldError>{
        self.validate_effect_count()?;self.validate_links()?;
        let pool=self.effects.as_ref().ok_or(ActorWorldError::EffectsNotAdmitted)?;
        if word(&self.backing,0x3554)==200{
            for record in pool.records(){
                let priority=match record[69]{6=>27,0x15=>19,0x16=>20,0x1a=>35,0x2c=>5,0x2f=>35,0x30=>20,0x31=>1,_=>return Err(ActorWorldError::UnsupportedEffectEvent.into())};
                if priority>5&&record[21]&1==0{return Err(ConstructionWorldError::EffectReplacementRequired);}
            }
            return Ok(None);
        }
        let slot=pool.records().iter().position(|record|record[6]==0).ok_or(ActorWorldError::UnsupportedEffectEvent)? as u16;
        let before=*pool.record(usize::from(slot)).map_err(ActorWorldError::from)?;
        //The source factory inserts at position+64 without moving record7.
        //Construction's aligned0/127 points satisfy this bounded shared-cell domain.
        if floor_cell(position)!=floor_cell(position.map(|v|v.wrapping_add(64))){return Err(ConstructionWorldError::EffectPositionUnsupported);}
        let(ground,shade)=self.effect_ground_sample(position)?;
        let mut staged=self.clone();
        let building=staged.create_building(owner,selector,position)?.ok_or(ConstructionWorldError::BuildingFactoryReturnedNull)?;
        let effect=crate::building_effect::plan_building_effect_numeric(&before,slot,position,ground,shade,physical_height,selector,owner,Some(building.slot))
            .map_err(ConstructionWorldError::BuildingEffect)?;
        staged.effects.as_mut().unwrap().commit_building_factory(&effect).map_err(ActorWorldError::from)?;
        let count=word(&staged.backing,0x3554).wrapping_add(1);staged.backing[0x3554..0x3556].copy_from_slice(&count.to_le_bytes());
        staged.insert_effect_head(slot)?;
        staged.validate_links()?;staged.validate_effect_count()?;staged.validate_building_counts()?;
        *self=staged;Ok(Some(BuildingEffectCreationReceipt{slot,building,effect}))
    }
}
