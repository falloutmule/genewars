//! Ordered47F80 callbacks for currently recovered building behavior.
//! Completed production branches outside admission reject the whole pass.
use super::*;
use crate::building_resources::BuildingResourceDatabase;
use crate::source_building_resources::SourceBuildingResourceDatabase;

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum AdmittedBuildingCallback {
    GenepodIdle{slot:u16,plan:crate::genepod_idle::GenepodIdlePlan},
    GenepodWork{slot:u16,plan:crate::genepod_work_callback::GenepodWorkPlan},
    GenepodTerminal{receipt:GenepodProductionReceipt},
    Incomplete{slot:u16,plan:crate::building_callback::IncompleteBuildingPlan},
    Turret{slot:u16,plan:crate::completed_building::TurretTrackingPlan},
    Extractor{slot:u16,plan:crate::extractor_production::ExtractorIdlePlan},
    ProducingExtractor{receipt:ExtractorWorldReceipt},
    PowerProducer{receipt:PowerBuildingReceipt},
}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct AdmittedBuildingPass {
    pub callbacks:Vec<AdmittedBuildingCallback>,
    pub notices:Vec<(u8,crate::bookkeeping::BookkeepingNotice)>,
}
impl ActorWorld {
    pub fn step_admitted_building_callbacks(&mut self,player_count:u8,selected:u8,
        turret:&BuildingResourceDatabase,resources:&[SourceBuildingResourceDatabase],
        frame:u32,physics:&crate::physics_tables::PhysicsTables,
        work_audio_active:impl Fn(u16)->bool)->Result<AdmittedBuildingPass,ActorWorldError> {
        self.step_admitted_buildings_inner(player_count,selected,turret,resources,frame,physics,work_audio_active,None)
    }
    pub fn step_admitted_buildings_with_production(&mut self,player_count:u8,selected:u8,
        turret:&BuildingResourceDatabase,resources:&[SourceBuildingResourceDatabase],
        frame:u32,physics:&crate::physics_tables::PhysicsTables,
        work_audio_active:impl Fn(u16)->bool,context:LandingPhysicsContext,trig:&TrigTables,table:&[u8;8192])
        ->Result<AdmittedBuildingPass,ActorWorldError>{
        self.step_admitted_buildings_inner(player_count,selected,turret,resources,frame,physics,work_audio_active,Some((context,trig,table)))
    }
    fn step_admitted_buildings_inner(&mut self,player_count:u8,selected:u8,
        turret:&BuildingResourceDatabase,resources:&[SourceBuildingResourceDatabase],
        frame:u32,physics:&crate::physics_tables::PhysicsTables,
        work_audio_active:impl Fn(u16)->bool,production:Option<(LandingPhysicsContext,&TrigTables,&[u8;8192])>)
        ->Result<AdmittedBuildingPass,ActorWorldError>{
        if player_count>4||selected>=4{return Err(ActorWorldError::InvalidPlayer(player_count.max(selected)));}
        self.validate_links()?;self.validate_building_counts()?;
        let period=u16::from_le_bytes(self.backing[0x1c7a..0x1c7c].try_into().unwrap());
        let mut staged=self.clone();let mut callbacks=Vec::new();let mut notices=Vec::new();
        for owner in 0..player_count {
            let start=usize::from(owner)*64;
            crate::bookkeeping::plan_admitted_building_player(staged.players[usize::from(owner)].bytes(),
                &staged.buildings.as_ref().unwrap().records()[start..start+64],selected,period)?;
            for slot in start..start+64 {
                let record=*staged.buildings.as_ref().unwrap().record(slot)?;
                if record[6]==0{continue;}
                let selector=record[45];
                if selector!=1 {
                    let resource=resources.iter().find(|r|r.selector()==selector)
                        .ok_or(ActorWorldError::EnvironmentUnavailable)?;
                    if staged.buildings.as_ref().unwrap().binding(slot)?!=Some(crate::building::BuildingAnimationBinding{selector:resource.selector()}) {
                        return Err(crate::building_pool::BuildingPoolError::InvalidBinding(slot).into());
                    }
                }
                let callback=if record[49]&1==0 {
                    let plan=if selector==1{staged.step_incomplete_selector1_building(slot as u16,turret)?}
                    else{staged.step_incomplete_source_building(slot as u16,resources.iter().find(|r|r.selector()==selector).unwrap())?};
                    AdmittedBuildingCallback::Incomplete{slot:slot as u16,plan}
                }else if selector==0 && i32::from_le_bytes(record[54..58].try_into().unwrap())==1 && production.is_some(){
                    let (_,trig,_)=production.unwrap();
                    let receipt=staged.step_genepod_terminal(slot as u16,frame,trig).map_err(|e|match e{
                        GenepodProductionError::World(e)=>e,
                        _=>ActorWorldError::UnsupportedGenepodProduction("terminal planner admission"),
                    })?;
                    #[cfg(any(debug_assertions, feature = "callback-captures"))] staged.capture_numeric_callback(slot,frame,"genepod-terminal-after");
                    AdmittedBuildingCallback::GenepodTerminal{receipt}
                }else if selector==0 && i32::from_le_bytes(record[54..58].try_into().unwrap())>0 {
                    let players=core::array::from_fn(|p|*staged.players[p].bytes());
                    let plan=crate::genepod_work_callback::plan_completed_work(&record,&players,frame)
                        .map_err(|_|ActorWorldError::BuildingCallback(crate::building_callback::BuildingCallbackError::UnsupportedBuilding))?;
                    staged.buildings.as_mut().unwrap().replace_record_checked(slot,&record,&plan.building_after)?;
                    for p in 0..4{staged.players[p].replace_bookkeeping_checked(&players[p],&plan.players_after[p])?;}
                    AdmittedBuildingCallback::GenepodWork{slot:slot as u16,plan}
                }else if selector==0 {
                    let player=*staged.players[usize::from(owner)].bytes();
                    let plan=crate::genepod_idle::plan_completed_idle(&record,&player)
                        .map_err(|_|ActorWorldError::BuildingCallback(crate::building_callback::BuildingCallbackError::UnsupportedBuilding))?;
                    staged.buildings.as_mut().unwrap().replace_record_checked(slot,&record,&plan.building_after)?;
                    staged.players[usize::from(owner)].replace_bookkeeping_checked(&player,&plan.player_after)?;
                    AdmittedBuildingCallback::GenepodIdle{slot:slot as u16,plan}
                }else if selector==1 {
                    AdmittedBuildingCallback::Turret{slot:slot as u16,plan:staged.step_tracking_completed_selector1_building(slot as u16,turret,frame,physics)?}
                }else if matches!(selector,3|6) && production.is_some(){
                    let (_,trig,_)=production.unwrap();
                    let resource=resources.iter().find(|r|r.selector()==selector).ok_or(ActorWorldError::EnvironmentUnavailable)?;
                    let solar_generation=word(staged.backing_bytes(),0x1c5f);
                    AdmittedBuildingCallback::PowerProducer{receipt:staged.step_completed_power_building(slot as u16,resource,frame,solar_generation,physics,trig)?}
                }else if selector==11 && production.is_some(){
                    let (context,trig,table)=production.unwrap();
                    let resource=resources.iter().find(|r|r.selector()==11).ok_or(ActorWorldError::EnvironmentUnavailable)?;
                    AdmittedBuildingCallback::ProducingExtractor{receipt:staged.step_completed_extractor(slot as u16,resource,frame,work_audio_active(slot as u16),context,trig,table)?}
                }else if selector==11 {
                    let player=*staged.players[usize::from(owner)].bytes();
                    let plan=crate::extractor_production::plan_completed_idle(&record,&player,work_audio_active(slot as u16))
                        .map_err(ActorWorldError::ExtractorCallback)?;
                    staged.buildings.as_mut().unwrap().replace_record_checked(slot,&record,&plan.building_after)?;
                    staged.players[usize::from(owner)].replace_bookkeeping_checked(&player,&plan.player_after)?;
                    AdmittedBuildingCallback::Extractor{slot:slot as u16,plan}
                }else{return Err(ActorWorldError::BuildingCallback(crate::building_callback::BuildingCallbackError::UnsupportedBuilding));};
                callbacks.push(callback);
            }
            let player=*staged.players[usize::from(owner)].bytes();
            let plan=crate::bookkeeping::plan_admitted_building_player(&player,
                &staged.buildings.as_ref().unwrap().records()[start..start+64],selected,period)?;
            for (index,after) in plan.records_after.iter().enumerate(){
                let before=*staged.buildings.as_ref().unwrap().record(start+index)?;
                staged.buildings.as_mut().unwrap().replace_record_checked(start+index,&before,after)?;
            }
            staged.players[usize::from(owner)].replace_bookkeeping_checked(&player,&plan.player.after)?;
            notices.extend(plan.player.notices.into_iter().map(|notice|(owner,notice)));
        }
        staged.validate_links()?;staged.validate_building_counts()?;*self=staged;
        Ok(AdmittedBuildingPass{callbacks,notices})
    }
}

#[cfg(test)]
mod genepod_idle_tests {
 use super::*;
 use crate::{actor_pool::ActorHandleBindings,blueprint::BlueprintRng,player_state::PlayerState,source_building_resources::SourceBuildingBinding,physics_tables::PhysicsTables};
 use genewars_assets::ConqMissionSource;
 #[test]
 fn genepod_idle_then_late_production_rolls_back_entire_pass(){
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
 let load=|s:&str|std::fs::read(root.join(s)).unwrap();
 let raw=load("private/integration/game-runtime/terrain/CONQ001.DAT");let mission=ConqMissionSource::parse(&raw).unwrap();
 let players=core::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,[20736,13568],b"fixture\0").unwrap());
 let mut world=ActorWorld::from_mission(&mission,&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed:5428,ring:31,draw_count:0}).unwrap();
 world.admit_creatures(&[[0;124];300]).unwrap();world.admit_buildings(&[[0;67];256],[None;256]).unwrap();
 let slot=world.create_building(0,0,[19839,12927]).unwrap().unwrap().slot;
 let resource=SourceBuildingResourceDatabase::load(SourceBuildingBinding::CdBgeneSelector0,load("private/integration/game-runtime/buildings/BGENE.ANB")).unwrap();
 let turret=BuildingResourceDatabase::load_selector1_bturr(load("private/reference-run/run-006/drive-c/GAME/DATA/BUILDING/BTURR.ANB")).unwrap();
 let physics=PhysicsTables::from_mapped_bytes(&load("private/integration/game-runtime/physics/angle.bin"),&load("private/integration/game-runtime/physics/bounce.bin")).unwrap();
 assert!(world.progress_admitted_grade0_building(slot,280,false).unwrap().completed);

 let first_before=world.clone();
 let pass=world.step_admitted_building_callbacks(1,0,&turret,&[resource.clone()],1,&physics,|_|false).unwrap();
 assert!(matches!(pass.callbacks[0],AdmittedBuildingCallback::GenepodIdle{..}));
 assert_eq!(world.buildings().unwrap().record(slot as usize).unwrap()[25],1);
 assert_eq!(world.rng(),first_before.rng());
 let second=world.create_building(0,0,[21000,14000]).unwrap().unwrap().slot;
 world.progress_admitted_grade0_building(second,280,false).unwrap();
 let before=*world.buildings().unwrap().record(second as usize).unwrap();let mut active=before;active[54..58].copy_from_slice(&75i32.to_le_bytes());
 world.buildings.as_mut().unwrap().replace_record_checked(second as usize,&before,&active).unwrap();
 let snapshot=world.clone();
 let error=world.step_admitted_building_callbacks(1,0,&turret,&[resource],2,&physics,|_|false).unwrap_err();
 assert!(matches!(error,ActorWorldError::BuildingCallback(crate::building_callback::BuildingCallbackError::UnsupportedBuilding)));
 assert_eq!(world,snapshot);
 }
}
