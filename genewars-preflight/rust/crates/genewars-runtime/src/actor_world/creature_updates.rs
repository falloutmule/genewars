//! Bounded neutral Mule callbacks. Ordinary scheduler gates and unsupported
//! pathfinding/AI branches remain separate from this owned transaction.
use super::*;
use crate::creature_world::{self,InitialMuleChoice,CreatureAnimationBinding,CommonSpeedInput,MuleStepContext,MuleAnimations};
use crate::physics_tables::PhysicsTables;

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct MuleCallbackReceipt {
    pub motion:creature_world::CommonPhysicsPlan<124>,
    pub retirement:Option<MuleRetirementReceipt>,
    /// Requests made at the source virtual44 interface. Playback/mixer
    /// admission remains separate; no audio device is executed here.
    pub pending_audio:Vec<u8>,
    /// Source6C780 display erase and2C200 module cleanup requests. Numeric
    /// map/reference retirement is already committed, UI consumption is not.
    pub pending_plant_retirements:Vec<u16>,
    /// Optional5DF50 portrait/module refresh; no UI root prior is assumed.
    pub pending_portrait_refresh:bool,
}
impl core::ops::Deref for MuleCallbackReceipt {type Target=creature_world::CommonPhysicsPlan<124>;fn deref(&self)->&Self::Target{&self.motion}}

impl ActorWorld {
    /// One subsequently admitted Mule callback. Its record, raw map links
    /// and neighbor references commit together. Timer0 rechoice threads the
    /// shared RNG; rendering is a pending request.
    pub fn step_initial_mule_followup(&mut self,context:MuleStepContext,physics:&PhysicsTables,trig:&TrigTables,
        animations:&MuleAnimations)->Result<MuleCallbackReceipt,ActorWorldError> {
        self.step_mule_followup_inner(0,context,physics,trig,animations,false)
    }
    /// Same owned callback with source-backed code30 splash and code15
    /// deep-water effect factories enabled. Physical damage remains bounded.
    pub fn step_mule_followup_with_water(&mut self,context:MuleStepContext,physics:&PhysicsTables,trig:&TrigTables,animations:&MuleAnimations)->Result<MuleCallbackReceipt,ActorWorldError> {
        self.step_mule_followup_inner(0,context,physics,trig,animations,true)
    }
    pub fn step_neutral_mule_slot_with_water(&mut self,slot:u16,context:MuleStepContext,physics:&PhysicsTables,trig:&TrigTables,animations:&MuleAnimations)->Result<MuleCallbackReceipt,ActorWorldError>{
        self.step_mule_followup_inner(usize::from(slot),context,physics,trig,animations,true)
    }
    fn step_mule_followup_inner(&mut self,slot:usize,context:MuleStepContext,physics:&PhysicsTables,trig:&TrigTables,animations:&MuleAnimations,admit_water:bool)->Result<MuleCallbackReceipt,ActorWorldError> {
        #[cfg(debug_assertions)] self.capture_mule_callback(slot,context.frame_counter,"before");
        self.validate_links()?;
        let pool=self.creatures.as_ref().ok_or(CreatureError::UnsupportedMotionState)?;
        let binding=if pool.record(slot)?[0x5e]>=26{CreatureAnimationBinding::BonesSelector4}else{CreatureAnimationBinding::MuleSelector4};
        if pool.binding(slot)?!=Some(binding){return Err(CreatureError::UnsupportedMotionState.into());}
        let before=*pool.record(slot)?;let own=[slot as u8,(slot>>8)as u8,1];
        if before[6]!=1||before[24]!=4||word(&before,15)!=slot as i16{return Err(CreatureError::UnsupportedMotionState.into());}
        let null_food_wait=before[0x5e]==1&&before[0x57]==0&&before[0x49]&3==3&&before[0x60]&0x42==0x40;
        if matches!(before[0x5e],1|7)&&!null_food_wait {
            if before[0x57]==0{return Err(CreatureError::UnsupportedMotionState.into());}
            let target=self.entity(reference(&before,0x55))?;let id=word(&before,0x55);let flags=self.mule_plant_flags(id)?&9;
            //659A3 tests live health;659BF idempotently reserves on arrival.
            //Another feeder's reservation is not source exclusivity.
            if before[0x57]!=2||target[0x18]!=4||self.mule_plant_health(id)?<=0||if before[0x5e]==7{flags!=1}else{flags&8!=0}{return Err(CreatureError::UnsupportedMotionState.into());}
        }
        if before[0x5e]==3 {
            let raw=reference(&before,0x55);let association=reference(&before,0x50);
            let approach=before[0x61]&0x20==0&&association==raw;
            let rendezvous=before[0x61]&0x20!=0&&association[2]==0;
            if raw[2]!=1||!(approach||rendezvous){return Err(CreatureError::UnsupportedMotionState.into());}
            let peer=self.creatures.as_ref().unwrap().record(word(&raw,0)as usize)?;
            if peer[0x5e]!=3||peer[0x18]!=4||peer[0x7a]!=4||reference(peer,0x55)!=own{return Err(CreatureError::UnsupportedMotionState.into());}
        }
        let mut rng=self.rng;
        let (prefix,working_target)=self.mule_route_prefix(&before,context.frame_counter,physics,trig,&mut rng)?;
        if before[0x5e]==27&&creature_world::mule_callback_phase(&before,context.frame_counter)&1!=0&&word(&before,0x6c)&15==0&&(0..=1).contains(&word(&before,0x68)) {
            let plan=creature_world::plan_mule_bones_retirement_after_prefix(&before,&prefix,self.map(),context,physics,trig,animations)?;
            let mut staged=self.clone();let retirement=staged.retire_mule_bones_after_callback(&before,&plan.motion)?;
            staged.rng=rng;let mut motion=plan.motion;motion.after=retirement.after;*self=staged;
            return Ok(MuleCallbackReceipt{motion,retirement:Some(retirement),pending_audio:Vec::new(),pending_plant_retirements:Vec::new(),pending_portrait_refresh:false});
        }
        let query_world=std::cell::RefCell::new(self.clone());let rng_state=std::cell::RefCell::new(rng);
        let links_moved=std::cell::Cell::new(false);
        let mut pending_pair=None;
        let mut reserve_plant=None;let mut release_plant=None;let mut fed_plant=None;let mut pending_audio=Vec::new();let mut pending_plant_retirements=Vec::new();
        let(plan,water_effect)=creature_world::plan_mule_after_admitted_prefix_with_cell_entry(&before,&prefix,working_target,self.map(),context,physics,trig,
            |record,normal,previous|query_world.borrow().mule_contact_speed(record,normal,previous).map_err(|_|CreatureError::MotionEnvironment),
            |point|query_world.borrow().move_obstacle_at(point).map_err(|_|CreatureError::MotionEnvironment),animations,|record|{
                let world=query_world.borrow();let mut rng=rng_state.borrow_mut();
                if word(record,0x49)as u16&0x100!=0 {
                    world.check_mule_cell_entry(own,floor_cell([word(record,7),word(record,9)])).map_err(|_|CreatureError::UnsupportedCellEntry)?;
                    let choice=creature_world::plan_mule_retry_wait(record,*rng)?;*rng=choice.rng_after;return Ok(choice.after);
                }
                if record[0x5e]==3{
                    let raw=reference(record,0x55);
                    let peer=world.creatures.as_ref().unwrap().record(word(&raw,0)as usize)?;
                    if record[0x61]&0x20!=0{
                        //69BE2..69C2B: wait/mark this arrival only. Starting
                        //the next atomic center movement remains bounded out.
                        if (word(record,0x60)as u16&word(peer,0x60)as u16&0x2004)!=0x2000{return Err(CreatureError::UnsupportedMotionState);}
                        let mut after=*record;if word(record,0x49)as u16&0x200!=0{after[0x60]|=4;}
                        if (word(&after,0x60)as u16&word(peer,0x60)as u16&0x2004)==0x2004{return Err(CreatureError::UnsupportedMotionState);}
                        return Ok(after);
                    }
                    let point=[word(peer,7),word(peer,9)];let saved_cell=floor_cell([word(record,7),word(record,9)]);
                    if floor_cell(point)==saved_cell{
                        let raw=reference(record,0x55);let id=word(&raw,0)as u16;
                        let other=*world.creatures.as_ref().unwrap().record(id as usize)?;
                        let pair=crate::mule_pair_rendezvous::plan_rendezvous(record,&other,saved_cell as u16).map_err(|_|CreatureError::UnsupportedMotionState)?;
                        pending_pair=Some((id,other,pair.peer_after,Vec::new()));
                        return Ok(pair.current_after);
                    }
                    let mut after=*record;
                    if creature_world::mule_callback_phase(&before,context.frame_counter)&3==3{after[11..13].copy_from_slice(&point[0].to_le_bytes());after[13..15].copy_from_slice(&point[1].to_le_bytes());after[0x49]|=4;}
                    return Ok(after);
                }
                if record[0x5e]==7 {
                    let raw=reference(record,0x55);let id=word(&raw,0);let health=world.mule_plant_health(id).map_err(|_|CreatureError::MotionEnvironment)?;if raw[2]!=2||health<=0{return Err(CreatureError::UnsupportedMotionState);}
                    // The feeding query may choose a real same-species pair.
                    // Its winner is selected in source randomized cell order.
                    let pair_query=record[0x77]==255&&record[0x79]>0;
                    if !pair_query{world.check_mule_neutral_neighborhood(record,own,[word(record,7),word(record,9)]).map_err(|_|CreatureError::UnsupportedNeighborhood)?;}
                    let l=world.layouts[1];let selector=world.backing[l.file_offset+id as usize*l.packed_stride+0x23]as i8;
                    if selector<0{return Err(CreatureError::UnsupportedMotionState);}
                    let params=world.plants.as_ref().ok_or(CreatureError::MotionEnvironment)?.parameters.slot(selector as usize).map_err(|_|CreatureError::MotionEnvironment)?;
                    let retired=health<=3;
                    if retired&&(params.retirement_delay_u8>=4||world.mule_plant_flags(id).map_err(|_|CreatureError::MotionEnvironment)?&0x88!=0){return Err(CreatureError::UnsupportedMotionState);}
                    let choice=if retired {creature_world::plan_mule_plant_feed_after_retirement(record,params.nutrition_u8,*rng)?}else{creature_world::plan_mule_plant_feed(record,params.nutrition_u8,*rng)?};*rng=choice.rng_after;fed_plant=Some(id);
                    if pair_query{
                        // This bounded branch excludes retirement/full feeding;
                        // those source fallbacks need their own continuation.
                        if retired||choice.after[0x5e]!=7{return Err(CreatureError::UnsupportedNeighborhood);}
                        let peer=world.mule_feeding_pair_candidate(&choice.after,choice.rng_before).map_err(|_|CreatureError::UnsupportedNeighborhood)?;
                        if let Some(peer_id)=peer{
                            if world.creatures.as_ref().unwrap().binding(peer_id as usize)?!=Some(CreatureAnimationBinding::MuleSelector4){return Err(CreatureError::UnsupportedNeighborhood);}
                            let other=*world.creatures.as_ref().unwrap().record(peer_id as usize)?;
                            let pair=crate::mule_pair_init::plan_pair_init(&choice.after,&other).map_err(|_|CreatureError::UnsupportedNeighborhood)?;
                            for &plant in &pair.release_plant_reservations{if world.entity([plant as u8,(plant>>8)as u8,2]).map_err(|_|CreatureError::UnsupportedNeighborhood)?[6]!=2{return Err(CreatureError::UnsupportedNeighborhood);}}
                            pending_pair=Some((peer_id,other,pair.peer_after,pair.release_plant_reservations));
                            pending_audio.push(0x12);return Ok(pair.current_after);
                        }
                    }
                    let full=record[0x61]&0x40!=0&&choice.after[0x61]&0x40==0;pending_audio.push(if full{0x1b}else{0x12});if retired{pending_plant_retirements.push(id as u16);}else if choice.after[0x5e]==8{release_plant=Some(id);}return Ok(choice.after);
                }
                if record[0x5e]==1 {
                    let raw=reference(record,0x55);
                    if raw[2]==0 {
                        let choice=crate::mule_lost_food::plan_lost_food(record,*rng).map_err(|_|CreatureError::UnsupportedMotionState)?;
                        let saved_cell=floor_cell([word(record,7),word(record,9)]);
                        if world.mule_cell_entry_predicate(&choice.record_after,saved_cell).map_err(|_|CreatureError::UnsupportedCellEntry)?{return Err(CreatureError::UnsupportedCellEntry);}
                        *rng=choice.rng_after;return Ok(choice.record_after);
                    }
                    if raw[2]!=2||world.mule_plant_flags(word(&raw,0)).map_err(|_|CreatureError::MotionEnvironment)?&8!=0||world.mule_plant_health(word(&raw,0)).map_err(|_|CreatureError::MotionEnvironment)?<=0{return Err(CreatureError::UnsupportedMotionState);}
                    let mut after=*record;after[0x3f..0x41].copy_from_slice(&creature_world::mule_requested_speed(record).to_le_bytes());after[0x5e]=7;after[0x6c..0x6e].copy_from_slice(&32767i16.to_le_bytes());after[0x72..0x76].copy_from_slice(&record[7..11]);after[0x60]|=8;after[0x60]&=0xbf;reserve_plant=Some(word(&raw,0));return Ok(after);
                }
                let mut record=*record;if record[0x5e]==0 {record[0x5e]=8;}
                world.check_mule_neutral_neighborhood(&record,own,[word(&record,7),word(&record,9)]).map_err(|_|CreatureError::UnsupportedNeighborhood)?;
                let target=world.mule_neutral_food_target(&record,*rng).map_err(|_|CreatureError::UnsupportedNeighborhood)?;
                let choice=if let Some((id,point))=target {creature_world::plan_mule_plant_acquisition(&record,id,point,*rng)?}else{creature_world::plan_neutral_mule_rechoice(&record,*rng,trig,|point|world.mule_accepts(slot,point).map_err(|_|CreatureError::MotionEnvironment))?};
                *rng=choice.rng_after;Ok(choice.after)
            },admit_water,|record,old_cell,new_cell|{
                let mut world=query_world.borrow_mut();
                world.unlink_mule_slot(slot,old_cell).map_err(|_|CreatureError::UnsupportedCellEntry)?;
                let prior=*world.creatures.as_ref().unwrap().record(slot)?;
                world.creatures.as_mut().unwrap().replace_record_checked(slot,&prior,record)?;
                world.insert_mule_head(slot,new_cell).map_err(|_|CreatureError::UnsupportedCellEntry)?;
                let entered=*world.creatures.as_ref().unwrap().record(slot)?;
                let after=world.plan_mule_cell_entry(slot,&entered,new_cell,&mut rng_state.borrow_mut(),trig)?;
                world.creatures.as_mut().unwrap().replace_record_checked(slot,&entered,&after)?;
                links_moved.set(true);Ok(after)
            })?;
        let rng=rng_state.into_inner();let mut staged=query_world.into_inner();
        // The entry hook already applied source unlink/prepend and virtual88.
        // Mortality uses the legacy bounded planner and still rejects its
        // unsupported cell callback; arrival's second cell remains guarded.
        if plan.old_cell!=plan.physics_cell&&!links_moved.get() {self.check_mule_cell_entry(own,plan.physics_cell)?;staged.unlink_mule_slot(slot,plan.old_cell)?;}
        if plan.physics_cell!=plan.new_cell {staged.check_mule_cell_entry(own,plan.new_cell)?;}
        let prior=*staged.creatures.as_ref().unwrap().record(slot)?;
        staged.creatures.as_mut().unwrap().replace_record_checked(slot,&prior,&plan.after)?;
        if plan.old_cell!=plan.physics_cell&&!links_moved.get() {staged.insert_mule_head(slot,plan.physics_cell)?;}
        if plan.physics_cell!=plan.new_cell {staged.unlink_mule_slot(slot,plan.physics_cell)?;staged.insert_mule_head(slot,plan.new_cell)?;}
        if let Some(id)=reserve_plant {let l=staged.layouts[1];let at=l.file_offset+id as usize*l.packed_stride+0x26;staged.backing[at]|=1;}
        if let Some(id)=release_plant {let l=staged.layouts[1];let at=l.file_offset+id as usize*l.packed_stride+0x26;staged.backing[at]&=0xfe;}
        if let Some(id)=fed_plant {let l=staged.layouts[1];let at=l.file_offset+id as usize*l.packed_stride+0x24;let old=word(&staged.backing,at);staged.backing[at..at+2].copy_from_slice(&old.wrapping_sub(3).to_le_bytes());}
        for id in &pending_plant_retirements {staged.retire_mule_food_plant(*id)?;}
        if let Some((peer_id,before,after,releases))=pending_pair{
            staged.creatures.as_mut().unwrap().replace_record_checked(peer_id as usize,&before,&after)?;
            for id in releases{let l=staged.layouts[1];staged.backing[l.file_offset+id as usize*l.packed_stride+0x26]&=0xfe;}
        }
        if let Some(request)=water_effect {
            if plan.physics_cell!=plan.new_cell {return Err(CreatureError::MotionEnvironment.into());}
            match request.code {
                0x30=>if let Some((_slot,events))=staged.create_water_splash(request.position,request.physical_height)? {
                    for event in events {if let crate::landing::LandingEvent::PlayAudio(index)=event {pending_audio.push(index.channel);}}
                },
                //6E8CE..6E914 pushes four zero arguments: factory owner0.
                0x15=>{staged.create_water_effect(0x15,0,request.position,request.physical_height,request.angles,trig)?;},
                _=>return Err(CreatureError::MotionEnvironment.into()),
            }
        }
        staged.rng=rng;
        #[cfg(debug_assertions)] staged.capture_mule_callback(slot,context.frame_counter,"after");
        staged.validate_links()?;let mut result=plan;result.after=*staged.creatures.as_ref().unwrap().record(slot)?;*self=staged;Ok(MuleCallbackReceipt{motion:result,retirement:None,pending_audio,pending_plant_retirements,pending_portrait_refresh:before[0x5e]==8&&prefix[0x5e]==0})
    }

    /// Opt-in immutable callback fixture for source comparison; absent in release.
    #[cfg(debug_assertions)]
    fn capture_mule_callback(&self,slot:usize,frame:u32,phase:&str){
        let extra=std::env::var("GENEWARS_MULE_CALLBACK_FRAME").ok().and_then(|v|v.parse::<u32>().ok());
        if extra.map_or(frame>1,|requested|frame!=requested){return;} // Explicit frame or default first two.
        let Some(root)=std::env::var_os("GENEWARS_MULE_CALLBACK_FIXTURE") else{return;};
        let dir=std::path::PathBuf::from(root).join(format!("frame-{frame}-slot-{slot}-{phase}"));
        std::fs::create_dir(&dir).expect("new immutable Mule callback fixture");
        for(name,bytes)in [("backing",self.backing_bytes().to_vec()),("players",(0..4u8).flat_map(|p|self.player(p).unwrap().bytes().iter().copied()).collect()),("actors",self.actors().records().concat()),("creatures",self.creatures().unwrap().records().concat()),("effects",self.effects().unwrap().records().concat()),("family8",self.family8().unwrap().records().concat()),("buildings",self.buildings().unwrap().records().concat())]{std::fs::write(dir.join(format!("{name}.bin")),bytes).unwrap();}
        let mut rng=self.rng.seed.to_le_bytes().to_vec();rng.push(self.rng.ring);rng.extend(self.rng.draw_count.to_le_bytes());std::fs::write(dir.join("rng.bin"),rng).unwrap();
    }
    fn mule_route_prefix(&self,before:&CreatureRecord,frame:u32,physics:&PhysicsTables,trig:&TrigTables,rng:&mut BlueprintRng)->Result<(CreatureRecord,[i16;2]),ActorWorldError> {
        let phase=creature_world::mule_callback_phase(before,frame);
        let own=[before[15],before[16],1];let current=[word(before,7),word(before,9)];
        let associated_pair=before[0x5e]==3&&before[0x52]==1&&reference(before,0x50)==reference(before,0x55);
        let mut final_target=[word(before,11),word(before,13)];
        let mut a=*before;a[0x49]&=0x9f;
        if a[0x4f]!=0||(a[0x52]!=0&&!associated_pair)||a[0x4a]&0x04!=0{return Err(CreatureError::UnsupportedMotionState.into());}
        if a[0x49]&3==3 {
            let wait=crate::mule_waypoint_wait::plan_wait(&a,frame,physics,|goal|self.mule_cell_entry_predicate(&a,floor_cell(goal))).map_err(|_|CreatureError::UnsupportedMotionState)?;
            return Ok((creature_world::plan_common_ground_steering(&wait.record_after,trig)?,final_target));
        }
        if a[0x5e]<24&&a[0x49]&0x10==0&&phase&31==0&&!self.common_move_point_accepted(own,current)? {
            if let Some(point)=creature_world::find_common_idle_relocation(current,|point|self.common_move_point_accepted(own,point).map_err(|_|CreatureError::MotionEnvironment))? {a=creature_world::plan_mule_idle_relocation(&a,point)?;a[0x49]&=0xfe;}
        }
        let mut working=if a[0x49]&3!=0 {[word(&a,0x45),word(&a,0x47)]}else{final_target};
        let mut immediate_heading=false;
        if a[0x49]&0x10!=0&&(a[0x49]&0x0c!=0||(associated_pair&&phase&3==3&&a[0x49]&3==0)) {
            if associated_pair{let peer=self.entity(reference(before,0x50))?;final_target=[word(peer,7),word(peer,9)];a[11..13].copy_from_slice(&final_target[0].to_le_bytes());a[13..15].copy_from_slice(&final_target[1].to_le_bytes());if a[0x49]&3==0{working=final_target;}}
            let accepted=if associated_pair{self.mule_accepts(word(before,15)as usize,final_target)?}else{self.common_move_point_accepted(own,final_target)?};
            if !accepted {
                if !associated_pair{return Err(CreatureError::TargetSearchRequired.into());}
                final_target=crate::target_resolver::resolve_target(current,final_target,physics,trig,|point|self.common_move_point_accepted(own,point)).map_err(|_|CreatureError::TargetSearchRequired)?.point;
                a[11..13].copy_from_slice(&final_target[0].to_le_bytes());a[13..15].copy_from_slice(&final_target[1].to_le_bytes());
                if a[0x49]&3==0{working=final_target;}
            }
            let from=if a[0x49]&3!=0 {working}else{current};
            let ray=self.mule_ray(own,from,final_target,physics,trig)?;
            if ray.clear {
                if a[0x49]&3!=0 {let flags=word(&a,0x49)as u16&0xeffc;a[0x49..0x4b].copy_from_slice(&flags.to_le_bytes());a[0x16]&=0x7f;immediate_heading=true;}
                a[0x49]&=0xf3;working=final_target;
            }else{
                if a[0x49]&3!=0 {
                    if a[0x49]&8!=0 {
                        let result=self.mule_waypoint(own,from,final_target,a[0x49]&1!=0,120,physics,trig)?;
                        working=result.point.unwrap_or(from);
                        if result.cost<120 {immediate_heading=true;}
                        else {a[0x54]=a[0x54].wrapping_sub(1);if a[0x54]==0 {let flags=(word(&a,0x49)as u16&0xeebc)|0x140;a[0x49..0x4b].copy_from_slice(&flags.to_le_bytes());a[0x16]&=0x7f;}}
                    }
                }else{
                    let blocked=ray.failure_point.ok_or(CreatureError::MotionEnvironment)?;
                    let first=self.mule_waypoint(own,blocked,final_target,false,120,physics,trig)?;
                    let second=self.mule_waypoint(own,blocked,final_target,true,if first.cost<120{first.cost+1}else{120},physics,trig)?;
                    let flags=word(&a,0x49)as u16&0xeffc;a[0x49..0x4b].copy_from_slice(&flags.to_le_bytes());a[0x16]&=0x7f;
                    let failed=first.cost>=120&&second.cost>=120;
                    if failed {a[0x54]=a[0x54].wrapping_sub(1);}
                    if failed&&a[0x54]==0 {let flags=word(&a,0x49)as u16|0x140;a[0x49..0x4b].copy_from_slice(&flags.to_le_bytes());working=[word(&a,0x45),word(&a,0x47)];}
                    else {
                        if failed&&first.cost==121&&second.cost==121 {a[0x16]|=0x80;}
                        let choose_first=if failed&&first.cost==second.cost {
                            rng.seed=rng.seed.wrapping_mul(0xe62d);rng.ring=rng.ring.wrapping_add(1)&31;rng.draw_count=rng.draw_count.wrapping_add(1);(rng.seed>>8)%10<5
                        }else{first.cost<second.cost};
                        let(result,flag)=if choose_first{(first,0x1002u16)}else{(second,0x1001u16)};working=result.point.ok_or(CreatureError::MotionEnvironment)?;
                        let flags=word(&a,0x49)as u16|flag;a[0x49..0x4b].copy_from_slice(&flags.to_le_bytes());immediate_heading=true;
                    }
                }
                a[0x45..0x47].copy_from_slice(&working[0].to_le_bytes());a[0x47..0x49].copy_from_slice(&working[1].to_le_bytes());a[0x49]&=0xf3;
            }
        }
        if a[0x49]&0x10!=0&&(immediate_heading||phase&1!=0){
            let dx=working[0].wrapping_sub(current[0]).wrapping_mul(2);let dy=working[1].wrapping_sub(current[1]).wrapping_mul(2);
            let angle=crate::actor_motion::angle(i32::from(dx),i32::from(dy),physics.angles());a[0x4b..0x4d].copy_from_slice(&angle.to_le_bytes());
        }
        a=creature_world::plan_common_ground_steering(&a,trig)?;Ok((a,working))
    }
    fn mule_ray(&self,own:[u8;3],from:[i16;2],to:[i16;2],physics:&PhysicsTables,trig:&TrigTables)->Result<crate::actor_motion::ActorStraightRay,ActorWorldError> {
        Ok(crate::actor_motion::actor_straight_ray::<CreatureError,_>(from,to,physics.angles(),trig.sine().try_into().unwrap(),trig.cosine().try_into().unwrap(),
            |point|self.common_move_point_accepted(own,point).map_err(|_|CreatureError::MotionEnvironment))?)
    }
    fn mule_waypoint(&self,own:[u8;3],start:[i16;2],target:[i16;2],side:bool,limit:u8,physics:&PhysicsTables,trig:&TrigTables)->Result<creature_world::CommonWaypoint,ActorWorldError> {
        let current=self.creatures.as_ref().unwrap().record(word(&own,0)as usize)?;
        Ok(creature_world::plan_common_waypoint([word(current,7),word(current,9)],start,target,side,limit,physics,
            |point|self.common_move_point_accepted(own,point).map_err(|_|CreatureError::MotionEnvironment),
            |from,to|self.common_straight_move_ray(own,from,to,physics,trig).map_err(|_|CreatureError::MotionEnvironment))?)
    }

    /// Actual74D90 null-obstacle point admission for kind1 Mule or an owned
    /// kind3 actor. The source queries obstacle before terrain/blocker.
    pub fn common_move_point_accepted(&self,raw:[u8;3],point:[i16;2])->Result<bool,ActorWorldError> {
        self.entity(raw)?;
        if self.move_obstacle_at(point)? {return Ok(false);}
        match raw[2] {
            1=>self.mule_accepts(word(&raw,0)as usize,point),
            3=>{let env=self.placement_environment(word(&raw,0),point)?;let actor=self.entity(raw)?;Ok(actor[0x53]&(1<<env.terrain_class)==0&&env.blocker_weight<10)},
            _=>Err(CreatureError::UnsupportedMotionState.into()),
        }
    }

    /// Source743C0's fixed-point straight ray; it recomputes heading each64
    /// iterations and snaps when squared signed15 distance is<=4096. A
    /// blocked ray is exposed to the caller rather than inventing waypoints.
    pub fn common_straight_move_ray(&self,raw:[u8;3],from:[i16;2],to:[i16;2],physics:&PhysicsTables,trig:&TrigTables)->Result<bool,ActorWorldError> {
        let ray=crate::actor_motion::actor_straight_ray::<CreatureError,_>(from,to,physics.angles(),trig.sine().try_into().unwrap(),trig.cosine().try_into().unwrap(),
            |point|self.common_move_point_accepted(raw,point).map_err(|_|CreatureError::MotionEnvironment))?;
        Ok(ray.clear)
    }

    fn mule_contact_speed(&self,record:&CreatureRecord,normal:[i32;3],previous:i16)->Result<i16,ActorWorldError> {
        let point=[word(record,7),word(record,9)];let cell=floor_cell(point);let mut raw=reference(self.map(),cell*14+6);
        //8B340 returns the first type2; its virtual68 is1F410(false).
        let mut nearby_plant=None;
        while raw[2]!=0 {let entity=self.entity(raw)?;if raw[2]==2 {nearby_plant=Some((
            [word(entity,7),word(entity,9)],i32::from_le_bytes(entity[0x11..0x15].try_into().unwrap()),entity[0x19]));break;}raw=reference(entity,3);}
        let class=self.terrain_class_at(point)?;let desired=word(record,0x3f);
        let mut desired=match class {2=>(i32::from(desired)*4/5)as i16,5=>(i32::from(desired)*5/6)as i16,_=>desired};
        if let Some((at,height,animation))=nearby_plant {
            //73770 is squared distance, not sqrt: virtual0 is the Mule's
            // physical27 and the plant's ground11, then signed heightdelta>>3.
            let physical=i32::from_le_bytes(record[0x27..0x2b].try_into().unwrap());let z=height.wrapping_sub(physical)>>3;
            let distance=(genewars_core::cd_candidate::distance_squared_wrapping_15(point,at)as i32).wrapping_add(z.wrapping_mul(z));
            if distance<=i32::from(animation)*32+64 {desired=(i32::from(desired)*3/4)as i16;}
        }
        let target=[word(record,11),word(record,13)];
        let distance=if record[0x5e]==3&&record[0x52]==1{
            //5AD70 virtualB8 uses the associated Mule's physical xyz,
            //even when74A50 repaired the ordinary goal back to self.
            let peer=self.creatures.as_ref().ok_or(CreatureError::MotionEnvironment)?.record(word(record,0x50)as usize)?;
            let other=[word(peer,7),word(peer,9)];
            let z=i32::from_le_bytes(peer[0x27..0x2b].try_into().unwrap()).wrapping_sub(i32::from_le_bytes(record[0x27..0x2b].try_into().unwrap()))>>3;
            (genewars_core::cd_candidate::distance_squared_wrapping_15(point,other)as i32).wrapping_add(z.wrapping_mul(z))
        }else{creature_world::common_distance_squared(self.map(),point,target)?};
        Ok(creature_world::plan_common_contact_speed(CommonSpeedInput{speed:word(record,0x41),desired_speed:desired,
            width:word(record,0x2b)as u16,flags_4a:record[0x4a],free_flight:false,ground_height:i32::from_le_bytes(record[0x11..0x15].try_into().unwrap()),
            previous_height_low:previous,surface_term:normal[2],distance_squared:distance})?)
    }

    fn plan_mule_cell_entry(&self,slot:usize,record:&CreatureRecord,cell:usize,rng:&mut BlueprintRng,trig:&TrigTables)->Result<CreatureRecord,CreatureError>{
        if !self.mule_cell_entry_predicate(record,cell).map_err(|_|CreatureError::UnsupportedCellEntry)?{return Ok(*record);}
        if record[0x53]!=1||record[0x4f]!=0{return Err(CreatureError::UnsupportedCellEntry);}
        let plan=crate::mule_cell_entry::plan_waypoint(record,cell as u16,*rng,trig,|point|self.mule_accepts(slot,point))
            .map_err(|_|CreatureError::UnsupportedCellEntry)?;
        *rng=plan.rng_after;Ok(plan.record_after)
    }
    /// Bounded5B100 predicate shared by cell entry and waypoint waiting.
    fn mule_cell_entry_predicate(&self,record:&CreatureRecord,cell:usize)->Result<bool,ActorWorldError>{
        let own=[record[15],record[16],1];
        let approach_plant=record[0x5e]==1&&record[0x57]==2&&self.entity(reference(record,0x55))?[6]==2;
        if !matches!(record[0x5e],3|8)&&!approach_plant{self.check_mule_cell_entry(own,cell)?;return Ok(false);}
        if word(record,0x49)as u16&0x0400!=0{return Err(CreatureError::UnsupportedCellEntry.into());}
        let mut raw=reference(self.map(),cell*14+6);let mut last=None;
        while raw[2]!=0{
            let entity=self.entity(raw)?;
            if raw[2]==1{let candidate=self.creatures.as_ref().unwrap().record(word(&raw,0)as usize)?;if candidate[0x5e]<24{last=Some(raw);}}
            if raw[2]==5&&self.effects.as_ref().ok_or(CreatureError::UnsupportedCellEntry)?.record(word(&raw,0)as usize)?[0x45]==0x11{return Err(CreatureError::UnsupportedCellEntry.into());}
            raw=reference(entity,3);
        }
        Ok(last!=Some(own))
    }
    fn check_mule_cell_entry(&self,own:[u8;3],cell:usize)->Result<(),ActorWorldError> {
        let mut raw=reference(self.map(),cell*14+6);
        while raw[2]!=0 {if raw[2]==1&&raw!=own {return Err(CreatureError::UnsupportedCellEntry.into());}
            if raw[2]==5 {let pool=self.effects.as_ref().ok_or(CreatureError::UnsupportedCellEntry)?;if pool.record(word(&raw,0)as usize)?.get(0x45)==Some(&0x11){return Err(CreatureError::UnsupportedCellEntry.into());}}
            raw=reference(self.entity(raw)?,3);
        }Ok(())
    }
    fn check_mule_neutral_neighborhood(&self,record:&CreatureRecord,own:[u8;3],point:[i16;2])->Result<(),ActorWorldError> {
        let cell=floor_cell(point);self.check_mule_cell_entry(own,cell)?;
        let(cx,cy)=((cell%128)as i32,(cell/128)as i32);
        for dy in -4..=4 {for dx in -4..=4 {
            let index=((cy+dy)&127)as usize*128+((cx+dx)&127)as usize;let mut raw=reference(self.map(),index*14+6);
            while raw[2]!=0 {let entity=self.entity(raw)?;let ignored_effect=if raw[2]==8{true}else if raw[2]==5{self.effects.as_ref().ok_or(CreatureError::UnsupportedNeighborhood)?.record(word(&raw,0)as usize)?.get(0x45)!=Some(&0x11)}else{false};if raw!=own&&!ignored_effect&&(raw[2]!=2||entity[24]!=4)&&!self.mule_friendly_candidate_is_no_action(record,raw)?{return Err(CreatureError::UnsupportedNeighborhood.into());}raw=reference(entity,3);}
        }}Ok(())
    }
    ///6BA65/6BBAE: same-species live friend contributes no food candidate.
    /// Friendly direction is unused only with the separately enforced absence
    /// of enemies (62949..52); full field77 can select a paired action.
    fn mule_friendly_candidate_is_no_action(&self,record:&CreatureRecord,raw:[u8;3])->Result<bool,ActorWorldError>{
        if raw[2]!=1||record[0x18]!=4||record[0x7a]!=4||record[0x76]>90||record[0x77]==255{return Ok(false);}
        let other=self.creatures.as_ref().ok_or(CreatureError::UnsupportedNeighborhood)?.record(word(&raw,0)as usize)?;
        Ok(other[6]==1&&other[0x18]==4&&other[0x7a]==4&&other[0x5e]<24&&other[0x5e]!=15)
    }
    fn retire_mule_food_plant(&mut self,id:u16)->Result<(),ActorWorldError> {
        let dead=[id as u8,(id>>8)as u8,2];let record=self.entity(dead)?.to_vec();if record[24]!=4{return Err(CreatureError::UnsupportedMotionState.into());}
        let cell=floor_cell([word(&record,7),word(&record,9)]);let previous=reference(&record,0);let next=reference(&record,3);
        if previous[2]==0 {
            if word(&previous,0)as usize!=cell||reference(self.map(),cell*14+6)!=dead{return Err(ActorWorldError::StaleLink);}
            if next[2]!=0 {self.backing[MAP_START+cell*14+6..MAP_START+cell*14+9].copy_from_slice(&next);}else{self.backing[MAP_START+cell*14+8]=0;}
        }else{if reference(self.entity(previous)?,3)!=dead{return Err(ActorWorldError::StaleLink);}self.creature_replace_next(previous,next)?;}
        if next[2]!=0 {if reference(self.entity(next)?,0)!=dead{return Err(ActorWorldError::StaleLink);}self.replace_previous(next,previous)?;}
        let l=self.layouts[1];let at=l.file_offset+usize::from(id)*l.packed_stride;
        self.backing[at+0x19]=self.backing[at+0x19].wrapping_add(6);self.backing[at+0x24..at+0x26].fill(0);self.backing[at+0x26]|=0x80;
        let selector=self.backing[at+0x23]as i8;if selector<0{return Err(CreatureError::UnsupportedMotionState.into());}
        let delay=self.plants.as_ref().ok_or(CreatureError::MotionEnvironment)?.parameters.slot(selector as usize)?.retirement_delay_u8;self.backing[at+0x28]=delay;
        let population=self.backing[MAP_START+cell*14+13];if population<64 {self.backing[MAP_START+cell*14+13]=population.wrapping_add(2);}
        let target=crate::landing::LandingReference{index:id as i16,kind:2};
        for(kind,slot,bytes)in self.effect_cleanup_records(){for patch in crate::landing_effect_lifecycle::plan_reference_cleanup(kind,slot,&bytes,target)?{self.apply_cleanup_byte(patch)?;}}
        self.backing[at+6]=0;let count=word(&self.backing,0x3552).wrapping_sub(1);self.backing[0x3552..0x3554].copy_from_slice(&count.to_le_bytes());
        Ok(())
    }
    fn mule_plant_health(&self,id:i16)->Result<i16,ActorWorldError> {
        if id<0||id as usize>=self.layouts[1].count {return Err(CreatureError::UnsupportedNeighborhood.into());}
        let l=self.layouts[1];Ok(word(&self.backing,l.file_offset+id as usize*l.packed_stride+0x24))
    }
    fn mule_plant_flags(&self,id:i16)->Result<u8,ActorWorldError> {
        if id<0||id as usize>=self.layouts[1].count {return Err(CreatureError::UnsupportedNeighborhood.into());}
        //entity() deliberately exposes only the common31-byte prefix.
        //Loader native2E is packed26 after the8-byte pointer gap.
        let l=self.layouts[1];Ok(self.backing[l.file_offset+id as usize*l.packed_stride+0x26])
    }
    ///6B700 neutral type2 food score over8D930's radius1024 cell samples.
    /// The two live RNG values choose X/Y traversal signs, preserving ties.
    fn mule_neutral_food_target(&self,record:&CreatureRecord,rng:BlueprintRng)->Result<Option<(i16,[i16;2])>,ActorWorldError> {
        let own=[record[15],record[16],1];
        if word(record,0x68)>word(record,0x62)>>1&&record[0x61]&0x40==0 {return Ok(None);}
        let first=rng.seed.wrapping_mul(0xe62d);let second=first.wrapping_mul(0xe62d);
        let sx=if (first>>8)&1==0{256i16}else{-256};let sy=if (second>>8)&1==0{256i16}else{-256};
        let point=[word(record,7),word(record,9)];let mut best=None;let mut score=i32::MIN+1;
        for x in -4i16..=4 {for y in -4i16..=4 {
            let sample=[point[0].wrapping_add(x*sx),point[1].wrapping_add(y*sy)];let d=genewars_core::cd_candidate::distance_squared_wrapping_15(point,sample);
            if d>1024*1024 {continue;}
            let value=1024-genewars_core::cd_candidate::integer_sqrt(d)as i32;let cell=floor_cell(sample);let mut raw=reference(self.map(),cell*14+6);
            while raw[2]!=0 {let entity=self.entity(raw)?;
                //6BD4F..58 skips kind8 before scoring, regardless of owner.
                if raw[2]==8{raw=reference(entity,3);continue;}
                if raw[2]==5 {if self.effects.as_ref().ok_or(CreatureError::UnsupportedNeighborhood)?.record(word(&raw,0)as usize)?.get(0x45)==Some(&0x11){return Err(CreatureError::UnsupportedNeighborhood.into());}raw=reference(entity,3);continue;}
                if raw!=own {if self.mule_friendly_candidate_is_no_action(record,raw)?{raw=reference(entity,3);continue;}if raw[2]!=2||entity[0x18]!=4 {return Err(CreatureError::UnsupportedNeighborhood.into());}
                    if self.mule_plant_flags(word(&raw,0))?&9==0&&self.mule_plant_health(word(&raw,0))?>0&&value>score {score=value;best=Some((word(&raw,0),[word(entity,7),word(entity,9)]));}}
                raw=reference(entity,3);
            }
        }}Ok(best)
    }
    ///6B700/69334: bounded neutral same-Mule feeding query and pair winner.
    /// The supplied RNG is BEFORE the two query-initialization draws.
    fn mule_feeding_pair_candidate(&self,record:&CreatureRecord,rng:BlueprintRng)->Result<Option<u16>,ActorWorldError>{
        if record[0x18]!=4||record[0x7a]!=4||record[0x5e]!=7||record[0x77]!=255||record[0x79]==0{return Err(CreatureError::UnsupportedNeighborhood.into());}
        let own=[record[15],record[16],1];let point=[word(record,7),word(record,9)];
        let first=rng.seed.wrapping_mul(0xe62d);let second=first.wrapping_mul(0xe62d);
        let sx=if(first>>8)&1==0{256i16}else{-256};let sy=if(second>>8)&1==0{256i16}else{-256};
        let mut best=None;let mut score=0xf0000001u32 as i32;
        for x in -4i16..=4{for y in -4i16..=4{
            let sample=[point[0].wrapping_add(x*sx),point[1].wrapping_add(y*sy)];let d=genewars_core::cd_candidate::distance_squared_wrapping_15(point,sample);
            if d>1024*1024{continue;}let weight=1024-genewars_core::cd_candidate::integer_sqrt(d)as i32;
            let mut raw=reference(self.map(),floor_cell(sample)*14+6);
            while raw[2]!=0{let entity=self.entity(raw)?;
                if raw!=own{match raw[2]{
                    1=>{let other=self.creatures.as_ref().unwrap().record(word(&raw,0)as usize)?;
                        if other[0x18]!=4||other[0x7a]!=4||other[0x5e]>=24{return Err(CreatureError::UnsupportedNeighborhood.into());}
                        if !matches!(other[0x5e],3|10|15)&&other[0x77]==255&&word(other,0x49)as u16&0x400==0{
                            let value=i32::from(word(other,0x68))+weight;if value>score{score=value;best=Some(word(&raw,0)as u16);}
                        }
                    },
                    2=>if entity[24]!=4{return Err(CreatureError::UnsupportedNeighborhood.into());},
                    5=>if self.effects.as_ref().unwrap().record(word(&raw,0)as usize)?[0x45]==0x11{return Err(CreatureError::UnsupportedNeighborhood.into());},
                    8=>{},_=>return Err(CreatureError::UnsupportedNeighborhood.into()),
                }}
                raw=reference(entity,3);
            }
        }}
        if word(record,0x68)<=word(record,0x62)>>1{return Ok(None);}Ok(best)
    }
    fn creature_replace_next(&mut self,raw:[u8;3],next:[u8;3])->Result<(),ActorWorldError>{self.replace_next(raw,next)}
    pub(super) fn unlink_mule_slot(&mut self,slot:usize,cell:usize)->Result<(),ActorWorldError> {
        let raw=[slot as u8,(slot>>8)as u8,1];let record=self.entity(raw)?;let previous=reference(record,0);let next=reference(record,3);
        if previous[2]==0 {if word(&previous,0)as usize!=cell||reference(self.map(),cell*14+6)!=raw{return Err(ActorWorldError::StaleLink);}
            if next[2]!=0 {self.backing[MAP_START+cell*14+6..MAP_START+cell*14+9].copy_from_slice(&next);}else{self.backing[MAP_START+cell*14+8]=0;}
        }else{if reference(self.entity(previous)?,3)!=raw{return Err(ActorWorldError::StaleLink);}self.creature_replace_next(previous,next)?;}
        if next[2]!=0 {if reference(self.entity(next)?,0)!=raw{return Err(ActorWorldError::StaleLink);}self.replace_previous(next,previous)?;}Ok(())
    }
    fn insert_mule_head(&mut self,slot:usize,cell:usize)->Result<(),ActorWorldError> {
        let before=*self.creatures.as_ref().unwrap().record(slot)?;let head=reference(self.map(),cell*14+6);let mut after=before;
        after[..2].copy_from_slice(&(cell as u16).to_le_bytes());after[2]=0;
        if head[2]!=0 {self.entity(head)?;after[3..6].copy_from_slice(&head);self.replace_previous(head,[slot as u8,(slot>>8)as u8,1])?;}else{after[5]=0;}
        self.creatures.as_mut().unwrap().replace_record_checked(slot,&before,&after)?;
        self.backing[MAP_START+cell*14+6..MAP_START+cell*14+9].copy_from_slice(&[slot as u8,(slot>>8)as u8,1]);Ok(())
    }

    /// Apply the exact initial neutral Mule numeric callback as one owned
    /// transaction. Queries read this world's current map and native pools.
    /// The record after-image and shared RNG commit together; queue admission
    /// remains an explicit request in the receipt. Later callbacks reject
    /// until their movement/AI paths are separately admitted.
    pub fn step_initial_mule_choice(&mut self,frame_counter:u32,tables:&TrigTables)->Result<InitialMuleChoice,ActorWorldError> {
        if frame_counter!=1 {return Err(CreatureError::UnsupportedInitialState.into());}
        self.validate_links()?;
        let pool=self.creatures.as_ref().ok_or(CreatureError::UnsupportedInitialState)?;
        if pool.binding(0)?!=Some(CreatureAnimationBinding::MuleSelector4) || pool.records().iter().skip(1).any(|r|r[6]!=0) {return Err(CreatureError::UnsupportedInitialState.into());}
        let before=*pool.record(0)?;let point=[word(&before,7),word(&before,9)];let cell=floor_cell(point);let map=self.map();
        if reference(map,cell*14+6)!=[0,0,1] || before[5]!=0
            || genewars_core::cd_candidate::interpolate_height(map,point[0],point[1]).map_err(|e|ActorWorldError::Terrain(e.to_string()))?!=6272
            || genewars_core::cd_candidate::interpolate_entity_shade(map,point[0],point[1]).map_err(|e|ActorWorldError::Terrain(e.to_string()))?!=0
            || word(&self.backing,0x1c61).wrapping_add(word(map,cell*14+9))<=25 {
            return Err(CreatureError::InitialEnvironment.into());
        }
        // This conservative square includes every cell visited by the source
        // radius1024 scan. Neutral plants cannot trigger the initial full-
        // health hunger action; foreign creatures/actors/effects need their
        // scorer paths and are not silently omitted.
        let (cx,cy)=((cell%128)as i32,(cell/128)as i32);
        for dy in -4..=4 {for dx in -4..=4 {
            let index=((cy+dy)&127)as usize*128+((cx+dx)&127)as usize;
            let mut raw=reference(map,index*14+6);let mut seen=HashSet::new();
            while raw[2]!=0 {
                if !seen.insert(raw) {return Err(ActorWorldError::DuplicateLink(raw));}
                let entity=self.entity(raw)?;
                if raw!=[0,0,1] && (raw[2]!=2 || entity[24]!=4) {return Err(CreatureError::UnsupportedNeighborhood.into());}
                raw=reference(entity,3);
            }
        }}
        let plan=creature_world::plan_initial_mule_choice(&before,self.rng,tables,|target| {
            self.initial_mule_accepts(target).map_err(|_|CreatureError::InitialEnvironment)
        })?;
        let mut staged=self.clone();staged.creatures.as_mut().unwrap().replace_record_checked(0,&plan.before,&plan.after)?;
        staged.rng=plan.rng_after;staged.validate_links()?;*self=staged;Ok(plan)
    }

    /// 74CC0 type1 selector4 (kind-table value4, not the special value3).
    /// Avoided terrain short-circuits; 8B130 then counts only plant weights.
    fn initial_mule_accepts(&self,point:[i16;2])->Result<bool,ActorWorldError>{self.mule_accepts(0,point)}
    fn mule_accepts(&self,slot:usize,point:[i16;2])->Result<bool,ActorWorldError> {
        let class=self.terrain_class_at(point)?;
        if 1u8 & (1u8<<class)!=0 {return Ok(false);}
        let cell=floor_cell(point);
        //8B18E..8B242 exempts the entire selected plant cell, before
        //ordinary blocker traversal, for state1/7 or5F1.
        let own=self.creatures.as_ref().ok_or(CreatureError::MotionEnvironment)?.record(slot)?;
        if (own[0x5f]==1||matches!(own[0x5e],1|7))&&own[0x57]==2 {
            let target=self.entity(reference(own,0x55))?;
            if floor_cell([word(target,7),word(target,9)])==cell {return Ok(true);}
        }
        let mut raw=reference(self.map(),cell*14+6);let mut previous=[cell as u8,(cell>>8)as u8,0];let mut seen=HashSet::new();let mut weight=0;
        while raw[2]!=0 {
            if !seen.insert(raw) {return Err(ActorWorldError::DuplicateLink(raw));}
            let entity=self.entity(raw)?;
            if reference(entity,0)!=previous {return Err(ActorWorldError::BrokenPrevious(raw));}
            if raw!=[slot as u8,(slot>>8)as u8,1] {match raw[2] {
                1|3|5|8=>(),
                2=> {let layout=self.layouts[1];let at=layout.file_offset+word(&raw,0)as usize*layout.packed_stride;
                    if self.backing[at+0x26]&8==0 {weight+=10;}
                },
                kind=>return Err(ActorWorldError::UnsupportedBlockerFamily(kind)),
            }}
            previous=raw;raw=reference(entity,3);
        }
        Ok(weight<10)
    }
}
