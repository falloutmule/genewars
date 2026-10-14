//! Ordered47F80 callbacks for currently recovered building behavior.
//! Completed production branches outside admission reject the whole pass.
use super::*;
use crate::building_resources::BuildingResourceDatabase;
use crate::source_building_resources::SourceBuildingResourceDatabase;

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum AdmittedBuildingCallback {
    Incomplete{slot:u16,plan:crate::building_callback::IncompleteBuildingPlan},
    Turret{slot:u16,plan:crate::completed_building::TurretTrackingPlan},
    Extractor{slot:u16,plan:crate::extractor_production::ExtractorIdlePlan},
    ProducingExtractor{receipt:ExtractorWorldReceipt},
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
                }else if selector==1 {
                    AdmittedBuildingCallback::Turret{slot:slot as u16,plan:staged.step_tracking_completed_selector1_building(slot as u16,turret,frame,physics)?}
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
