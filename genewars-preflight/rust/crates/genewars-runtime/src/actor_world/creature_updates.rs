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
    /// Unmarked5A210 allocations; not source AL newborn-second-pass requests.
    pub newborn_slots:Vec<u16>,
    /// Source6C780 display erase and2C200 module cleanup requests. Numeric
    /// map/reference retirement is already committed, UI consumption is not.
    pub pending_plant_retirements:Vec<u16>,
    /// Optional5DF50 portrait/module refresh; no UI root prior is assumed.
    pub pending_portrait_refresh:bool,
}
impl core::ops::Deref for MuleCallbackReceipt {type Target=creature_world::CommonPhysicsPlan<124>;fn deref(&self)->&Self::Target{&self.motion}}

///73770 physical-Mule distance, shared by pursuit braking and pair AI.
fn mule_pair_distance(first:&CreatureRecord,second:&CreatureRecord)->i32{
    let a=[word(first,7),word(first,9)];let b=[word(second,7),word(second,9)];
    let z=i32::from_le_bytes(second[0x27..0x2b].try_into().unwrap()).wrapping_sub(i32::from_le_bytes(first[0x27..0x2b].try_into().unwrap()))>>3;
    (genewars_core::cd_candidate::distance_squared_wrapping_15(a,b)as i32).wrapping_add(z.wrapping_mul(z))
}

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
        let result=self.step_mule_followup_inner(usize::from(slot),context,physics,trig,animations,true);
        // Capture the exact first unsupported callback without another full replay.
        #[cfg(any(debug_assertions, feature = "callback-captures"))] if result.is_err(){self.capture_numeric_callback(usize::from(slot),context.frame_counter,"blocked-before");}
        result
    }
    fn step_mule_followup_inner(&mut self,slot:usize,context:MuleStepContext,physics:&PhysicsTables,trig:&TrigTables,animations:&MuleAnimations,admit_water:bool)->Result<MuleCallbackReceipt,ActorWorldError> {
        #[cfg(any(debug_assertions, feature = "callback-captures"))] self.capture_numeric_callback(slot,context.frame_counter,"before");
        self.validate_links()?;
        let pool=self.creatures.as_ref().ok_or(CreatureError::UnsupportedMotionState)?;
        let binding=if pool.record(slot)?[0x5e]>=26{CreatureAnimationBinding::BonesSelector4}else{CreatureAnimationBinding::MuleSelector4};
        if pool.binding(slot)?!=Some(binding){return Err(CreatureError::UnsupportedMotionState.into());}
        let before=*pool.record(slot)?;let own=[slot as u8,(slot>>8)as u8,1];
        if before[6]!=1||before[24]!=4||word(&before,15)!=slot as i16{return Err(CreatureError::UnsupportedMotionState.into());}
        let null_food_target=before[0x5e]==1&&before[0x57]==0&&before[0x60]&0x42==0x40;
        let hostile_enemy=self.mule_live_hostile_target(&before)?;
        let corpse_approach=matches!(before[0x5e],1|4|5)&&before[0x57]==1&&before[0x52]==1&&reference(&before,0x50)==reference(&before,0x55);
        if corpse_approach&&hostile_enemy.is_none() {
            let id=word(&before,0x55)as usize;let corpse=pool.record(id)?;
            let binding=if corpse[0x5e]>=26{CreatureAnimationBinding::BonesSelector4}else{CreatureAnimationBinding::MuleSelector4};
            if pool.binding(id)?!=Some(binding)||corpse[24]!=4||corpse[0x7a]!=4||!matches!(corpse[0x5e],24..=27){return Err(CreatureError::UnsupportedMotionState.into());}
        }
        let retained_food_association=before[0x5e]==8&&before[0x60]==8&&before[0x61]==0x40&&before[0x57]==0
            &&before[0x52]==1&&word(&before,0x50)==word(&before,0x55);
        if retained_food_association {
            let id=word(&before,0x50)as usize;let corpse=pool.record(id)?;
            let binding=if corpse[0x5e]>=26{CreatureAnimationBinding::BonesSelector4}else{CreatureAnimationBinding::MuleSelector4};
            if pool.binding(id)?!=Some(binding)||corpse[24]!=4||corpse[0x7a]!=4||!matches!(corpse[0x5e],24..=27){return Err(CreatureError::UnsupportedMotionState.into());}
        }
        if matches!(before[0x5e],1|7)&&!null_food_target&&!corpse_approach {
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
        let (prefix,working_target)=self.mule_route_prefix(&before,context.frame_counter,physics,trig,&mut rng,false)?;
        if before[0x5e]==27&&creature_world::mule_callback_phase(&before,context.frame_counter)&1!=0&&word(&before,0x6c)&15==0&&(0..=1).contains(&word(&before,0x68)) {
            let plan=creature_world::plan_mule_bones_retirement_after_prefix(&before,&prefix,self.map(),context,physics,trig,animations)?;
            let mut staged=self.clone();let retirement=staged.retire_mule_bones_after_callback(&before,&plan.motion)?;
            staged.rng=rng;let mut motion=plan.motion;motion.after=retirement.after;*self=staged;
            return Ok(MuleCallbackReceipt{motion,retirement:Some(retirement),newborn_slots:Vec::new(),pending_audio:Vec::new(),pending_plant_retirements:Vec::new(),pending_portrait_refresh:false});
        }
        let query_world=std::cell::RefCell::new(self.clone());let rng_state=std::cell::RefCell::new(rng);
        let links_moved=std::cell::Cell::new(false);
        let mut pending_pair=None;let mut newborn_slots=Vec::new();let mut failed_action_dispatched=false;let mut retired_during_feeding=false;
        let mut feed_bones=None;let mut reserve_corpse=None;let mut reserve_plant=None;let mut release_plant=None;let mut fed_plant=None;let mut pending_audio=Vec::new();let mut pending_plant_retirements=Vec::new();
        let(plan,water_effect)=creature_world::plan_mule_after_admitted_prefix_with_cell_entry_and_postphysics_policy(&before,&prefix,working_target,self.map(),context,physics,trig,
            |record,normal,previous|query_world.borrow().mule_contact_speed(record,normal,previous).map_err(|_|CreatureError::MotionEnvironment),
            |point|query_world.borrow().move_obstacle_at(point).map_err(|_|CreatureError::MotionEnvironment),animations,|record|{
                let mut world=query_world.borrow_mut();let mut rng=rng_state.borrow_mut();
                if word(record,0x49)as u16&0x100!=0&&!failed_action_dispatched {
                    //5E0B0 retry precedes5EB30 cleanup and ordinary624B8 AI.
                    //Flag100 survives untilthetail; do notdispatchretrytwice.
                    failed_action_dispatched=true;
                    if record[0x5e]==1{
                        //5CD1A releases plant2E bit4, distinct from reservationbit1.
                        //Bounded no-op release; nonclear/live-kind variants remain explicit.
                        if record[0x57]==2 {
                            if world.mule_plant_flags(word(record,0x55)).map_err(|_|CreatureError::UnsupportedMotionState)?&4!=0{return Err(CreatureError::UnsupportedMotionState);}
                        }else if record[0x57]==1 {
                            //State1 reset5CD1A only releases kind2 plants; kind1
                            //takes local5CCA0 reset, preserving corpse reservation.
                            let id=word(record,0x55)as u16;let corpse=world.creatures.as_ref().unwrap().record(id as usize)?;
                            if !matches!(corpse[0x5e],24..=27){return Err(CreatureError::UnsupportedMotionState);}
                        }else{return Err(CreatureError::UnsupportedMotionState);}
                    }
                    let choice=creature_world::plan_mule_retry_wait(record,*rng)?;*rng=choice.rng_after;
                    let cell=floor_cell([word(record,7),word(record,9)]);
                    if world.mule_cell_entry_predicate(&choice.after,cell).map_err(|_|CreatureError::UnsupportedCellEntry)?{
                        let relocation=crate::mule_retry_relocation::plan_relocation(&choice.after,*rng,trig).map_err(|_|CreatureError::UnsupportedCellEntry)?;
                        *rng=relocation.rng_after;return Ok(relocation.record_after);
                    }
                    return Ok(choice.after);
                }
                if record[0x5e]==4 {
                    if record[0x57]==0 {
                        let lost=crate::mule_lost_food::plan_lost_bones(record,*rng).map_err(|_|CreatureError::UnsupportedMotionState)?;
                        if world.mule_cell_entry_predicate(&lost.record_after,floor_cell([word(record,7),word(record,9)])).map_err(|_|CreatureError::UnsupportedCellEntry)?{return Err(CreatureError::UnsupportedCellEntry);}
                        *rng=lost.rng_after;return Ok(lost.record_after);
                    }
                    //679CE/67A22: nearby retained Bones, nonprocessing timer gate.
                    let bones=world.creatures.as_ref().unwrap().record(word(record,0x55)as usize)?;
                    if !matches!(bones[0x5e],26|27)||creature_world::common_distance_squared(world.map(),[word(record,7),word(record,9)],[word(bones,7),word(bones,9)])?>4096{return Err(CreatureError::UnsupportedMotionState);}
                    if word(record,0x6c)&15!=0{return Ok(*record);}
                    let feed=crate::mule_bones_feeding::plan_feed(record,bones,*rng).map_err(|_|CreatureError::UnsupportedMotionState)?;
                    //68445 skips food selection; low-half reserve prevents mating.
                    //Prove zero enemies from the actual circle, including reserved Bones.
                    world.check_mule_bones_feeding_no_action(&feed.current_after).map_err(|_|CreatureError::UnsupportedNeighborhood)?;
                    feed_bones=Some((word(record,0x55)as u16,word(bones,0x68),word(&feed.target_after,0x68)));
                    pending_audio.push(feed.pending_audio);*rng=feed.rng_after;return Ok(feed.current_after);
                }
                if record[0x5e]==5 {
                    let corpse=world.creatures.as_ref().unwrap().record(word(record,0x55)as usize)?;
                    //667EE/66834: still nearby; first food-processing gate remains bounded out.
                    if creature_world::common_distance_squared(world.map(),[word(record,7),word(record,9)],[word(corpse,7),word(corpse,9)])?>4096{return Err(CreatureError::UnsupportedMotionState);}
                    if word(record,0x6c)&15!=0{return Ok(*record);}
                    let mut predicate_record=*record;predicate_record[0x5e]=8;
                    let escape=world.mule_cell_entry_predicate(&predicate_record,floor_cell([word(record,7),word(record,9)])).map_err(|_|CreatureError::UnsupportedCellEntry)?;
                    let abandoned=crate::mule_corpse_abandon::plan_abandon(record,corpse,*rng,escape).map_err(|_|CreatureError::UnsupportedMotionState)?;
                    reserve_corpse=Some((word(record,0x55)as u16,corpse[0x60],abandoned.target_after[0x60]));
                    *rng=abandoned.rng_after;return Ok(abandoned.current_after);
                }
                if let Some(enemy)=hostile_enemy {
                    if record[0x5e]==20&&word(record,0x6c)==0 {
                        let sole=world.mule_hit_single_owned_enemy(record).map_err(|_|CreatureError::UnsupportedNeighborhood)?;
                        //8B7D0 only scans for kind5/code11; sole-enemy helper proves none in this cell.
                        if sole!=enemy{return Err(CreatureError::UnsupportedNeighborhood);}
                        let reacquire=crate::mule_neutral_hit_reacquire::plan_reacquire(record,&sole,*rng).map_err(|_|CreatureError::UnsupportedMotionState)?;
                        *rng=reacquire.rng_after;return Ok(reacquire.after);
                    }
                    if record[0x5e]==21{if word(record,0x6c)<=0{return world.mule_neutral_completed_attack(record,&enemy,&mut rng,physics,&mut pending_audio);}let mut after=*record;if creature_world::mule_callback_phase(record,context.frame_counter)&3==3{
                        let dx=word(&enemy,7).wrapping_mul(2).wrapping_sub(word(record,7).wrapping_mul(2));let dy=word(&enemy,9).wrapping_mul(2).wrapping_sub(word(record,9).wrapping_mul(2));
                        after[0x4b..0x4d].copy_from_slice(&crate::actor_motion::angle(i32::from(dx),i32::from(dy),physics.angles()).to_le_bytes());
                    }return Ok(after);}
                    if record[0x5e]==1 {
                    if mule_pair_distance(record,&enemy)<=16384 {
                        match crate::mule_hostile_contact::plan_false_contact(record,&enemy,physics){
                            Ok(after)=>return Ok(after),Err(crate::mule_hostile_contact::HostileContactError::UnsupportedAttack)=>{
                                let entry=crate::mule_hostile_attack_entry::plan_entry(record,&enemy,*rng,physics).map_err(|_|CreatureError::UnsupportedMotionState)?;
                                *rng=entry.rng_after;if let Some(channel)=entry.pending_audio{pending_audio.push(channel);}let mut after=entry.after;after[0x60]&=0xbf;return Ok(after);
                            },Err(_)=>return Err(CreatureError::UnsupportedMotionState),
                        }
                    }return Ok(*record);
                }}
                if record[0x5e]==1&&record[0x57]==1 {
                    let id=word(record,0x55)as u16;let corpse=world.creatures.as_ref().unwrap().record(id as usize)?;
                    let arrival=crate::mule_corpse_arrival::plan_arrival(record,corpse).map_err(|_|CreatureError::UnsupportedMotionState)?;
                    reserve_corpse=Some((id,corpse[0x60],arrival.target_after[0x60]));
                    //65B4D refreshes requested speed after both arrival branches.
                    let mut after=arrival.current_after;after[0x3f..0x41].copy_from_slice(&creature_world::mule_requested_speed(record).to_le_bytes());
                    return Ok(after);
                }
                if record[0x5e]==10{
                    //69EF2..69F84 positive-countdown wait. Missing/far peers
                    //require the separate source reset branch69F91.
                    let raw=reference(record,0x55);
                    if raw[2]!=1{return Err(CreatureError::UnsupportedMotionState);}
                    let peer=world.creatures.as_ref().unwrap().record(word(&raw,0)as usize).map_err(|_|CreatureError::UnsupportedMotionState)?;
                    if peer[6]!=1||peer[0x5e]!=10||peer[0x7a]!=4||reference(peer,0x55)!=own{return Err(CreatureError::UnsupportedMotionState);}
                    if word(record,0x6c)==0{
                        let birth=crate::mule_pair_birth::plan_birth(record,peer,*rng,peer[0x5e]>=24,genewars_core::cd_candidate::distance_squared_wrapping_15([word(record,7),word(record,9)],[word(peer,7),word(peer,9)])).map_err(|_|CreatureError::UnsupportedMotionState)?;
                        let peer_id=word(&raw,0)as usize;
                        let prior=*world.creatures.as_ref().unwrap().record(slot).map_err(|_|CreatureError::UnsupportedMotionState)?;
                        let peer_prior=*world.creatures.as_ref().unwrap().record(peer_id).map_err(|_|CreatureError::UnsupportedMotionState)?;
                        world.creatures.as_mut().unwrap().replace_record_checked(slot,&prior,&birth.current_after)?;
                        world.creatures.as_mut().unwrap().replace_record_checked(peer_id,&peer_prior,&birth.peer_after)?;
                        world.rng=birth.rng_after;
                        if let Some(newborn)=world.create_mule(birth.request.owner,birth.request.position,birth.request.quality,Some(&birth.current_after),trig).map_err(|_|CreatureError::UnsupportedInitialState)?{
                            newborn_slots.push(newborn.slot);pending_audio.push(0x44);
                            if birth.request.inherit{let old=*world.creatures.as_ref().unwrap().record(newborn.slot as usize)?;let mut after=old;after[0x61]|=4;world.creatures.as_mut().unwrap().replace_record_checked(newborn.slot as usize,&old,&after)?;}
                        }
                        let linked_current=*world.creatures.as_ref().unwrap().record(slot)?;let linked_peer=*world.creatures.as_ref().unwrap().record(peer_id)?;
                        let cell=floor_cell([word(record,7),word(record,9)])as u16;
                        let tail=crate::mule_pair_birth_tail::plan_tail(&linked_current,&linked_peer,cell,world.rng,trig,|r,c|world.mule_cell_entry_predicate(r,usize::from(c))).map_err(|_|CreatureError::UnsupportedCellEntry)?;
                        world.creatures.as_mut().unwrap().replace_record_checked(peer_id,&linked_peer,&tail.peer_after)?;
                        *rng=tail.rng_after;return Ok(tail.current_after);
                    }
                    if creature_world::mule_callback_phase(record,context.frame_counter)&3==3&&mule_pair_distance(record,peer)>16384{return Err(CreatureError::UnsupportedMotionState);}
                    return Ok(*record);
                }
                if record[0x5e]==3{
                    let raw=reference(record,0x55);
                    let peer=world.creatures.as_ref().unwrap().record(word(&raw,0)as usize)?;
                    if record[0x61]&0x20!=0{
                        //69BE2..69C2B: wait/mark this arrival only. Starting
                        //the next atomic center movement remains bounded out.
                        if (word(record,0x60)as u16&word(peer,0x60)as u16&0x2004)!=0x2000{
                            if mule_pair_distance(record,peer)>16384{return Ok(*record);}
                            let pair=crate::mule_pair_stop::plan_stop(record,peer).map_err(|_|CreatureError::UnsupportedMotionState)?;
                            pending_pair=Some((word(&raw,0)as u16,*peer,pair.peer_after,Vec::new()));
                            return Ok(pair.current_after);
                        }
                        let mut after=*record;if word(record,0x49)as u16&0x200!=0{after[0x60]|=4;}
                        if (word(&after,0x60)as u16&word(peer,0x60)as u16&0x2004)==0x2004{
                            let cell=floor_cell([word(record,7),word(record,9)]);
                            let pair=crate::mule_pair_center::plan_center(&after,peer,cell as u16).map_err(|_|CreatureError::UnsupportedMotionState)?;
                            pending_pair=Some((word(&raw,0)as u16,*peer,pair.peer_after,Vec::new()));
                            return Ok(pair.current_after);
                        }
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
                    let pair_query=record[0x77]==255&&(record[0x79]as i8)>0;
                    if !pair_query{world.check_mule_neutral_neighbors(record,own,[word(record,7),word(record,9)]).map_err(|_|CreatureError::UnsupportedNeighborhood)?;}
                    let l=world.layouts[1];let selector=world.backing[l.file_offset+id as usize*l.packed_stride+0x23]as i8;
                    if selector<0{return Err(CreatureError::UnsupportedMotionState);}
                    let params=world.plants.as_ref().ok_or(CreatureError::MotionEnvironment)?.parameters.slot(selector as usize).map_err(|_|CreatureError::MotionEnvironment)?;
                    let retired=health<=3;
                    if retired&&(params.retirement_delay_u8>=4||world.mule_plant_flags(id).map_err(|_|CreatureError::MotionEnvironment)?&0x88!=0){return Err(CreatureError::UnsupportedMotionState);}
                    let nutrition=params.nutrition_u8;let feeding_rng=*rng;
                    if retired{world.retire_mule_food_plant(id as u16).map_err(|_|CreatureError::UnsupportedMotionState)?;retired_during_feeding=true;pending_plant_retirements.push(id as u16);}else{fed_plant=Some(id);}
                    let mut choice=if pair_query {creature_world::plan_mule_feeding_query_prefix(record,nutrition,feeding_rng,retired)?}else if retired{creature_world::plan_mule_plant_feed_after_retirement(record,nutrition,feeding_rng)?}else{creature_world::plan_mule_plant_feed(record,nutrition,feeding_rng)?};*rng=choice.rng_after;
                    if retired{choice.after[..6].copy_from_slice(&world.creatures.as_ref().unwrap().record(slot)?[..6]);}
                    let pair_resources_ready=word(&choice.after,0x68)>(word(&choice.after,0x62)>>1);
                    if pair_query&&!pair_resources_ready{world.check_mule_neutral_neighbors(&choice.after,own,[word(&choice.after,7),word(&choice.after,9)]).map_err(|_|CreatureError::UnsupportedNeighborhood)?;}
                    if pair_query&&pair_resources_ready{
                        //69334 pair success precedes retirement/full fallback.
                        if choice.after[0x5e]!=7{return Err(CreatureError::UnsupportedNeighborhood);}
                        let peer=world.mule_neutral_pair_candidate(&choice.after,choice.rng_before).map_err(|_|CreatureError::UnsupportedNeighborhood)?;
                        if let Some(peer_id)=peer{
                            if world.creatures.as_ref().unwrap().binding(peer_id as usize)?!=Some(CreatureAnimationBinding::MuleSelector4){return Err(CreatureError::UnsupportedNeighborhood);}
                            let other=*world.creatures.as_ref().unwrap().record(peer_id as usize)?;
                            let pair=crate::mule_pair_init::plan_pair_init(&choice.after,&other).map_err(|_|CreatureError::UnsupportedNeighborhood)?;
                            for &plant in &pair.release_plant_reservations{if world.entity([plant as u8,(plant>>8)as u8,2]).map_err(|_|CreatureError::UnsupportedNeighborhood)?[6]!=2{return Err(CreatureError::UnsupportedNeighborhood);}}
                            pending_pair=Some((peer_id,other,pair.peer_after,pair.release_plant_reservations));
                            pending_audio.push(0x12);return Ok(pair.current_after);
                        }
                    }
                    if pair_query{choice=if retired{creature_world::plan_mule_plant_feed_after_retirement(record,nutrition,feeding_rng)?}else{creature_world::plan_mule_plant_feed(record,nutrition,feeding_rng)?};*rng=choice.rng_after;if retired{choice.after[..6].copy_from_slice(&world.creatures.as_ref().unwrap().record(slot)?[..6]);}}
                    if choice.after[0x5e]==8&&world.mule_cell_entry_predicate(&choice.after,floor_cell([word(&choice.after,7),word(&choice.after,9)])).map_err(|_|CreatureError::UnsupportedCellEntry)?{return Err(CreatureError::UnsupportedCellEntry);}
                    let full=record[0x61]&0x40!=0&&choice.after[0x61]&0x40==0;pending_audio.push(if full{0x1b}else{0x12});if !retired&&choice.after[0x5e]==8{release_plant=Some(id);}return Ok(choice.after);
                }
                if record[0x5e]==1 {
                    let raw=reference(record,0x55);
                    if raw[2]==0 {
                        let choice=crate::mule_lost_food::plan_lost_food(record,*rng).map_err(|_|CreatureError::UnsupportedMotionState)?;
                        let saved_cell=floor_cell([word(record,7),word(record,9)]);
                        if world.mule_cell_entry_predicate(&choice.record_after,saved_cell).map_err(|_|CreatureError::UnsupportedCellEntry)?{
                            //66120..66181 shares the exact radius256 relocation writes.
                            let relocation=crate::mule_retry_relocation::plan_relocation(&choice.record_after,choice.rng_after,trig).map_err(|_|CreatureError::UnsupportedCellEntry)?;
                            *rng=relocation.rng_after;return Ok(relocation.record_after);
                        }
                        *rng=choice.rng_after;return Ok(choice.record_after);
                    }
                    if raw[2]!=2||world.mule_plant_flags(word(&raw,0)).map_err(|_|CreatureError::MotionEnvironment)?&8!=0||world.mule_plant_health(word(&raw,0)).map_err(|_|CreatureError::MotionEnvironment)?<=0{return Err(CreatureError::UnsupportedMotionState);}
                    let mut after=*record;after[0x3f..0x41].copy_from_slice(&creature_world::mule_requested_speed(record).to_le_bytes());after[0x5e]=7;after[0x6c..0x6e].copy_from_slice(&32767i16.to_le_bytes());after[0x72..0x76].copy_from_slice(&record[7..11]);after[0x60]|=8;after[0x60]&=0xbf;reserve_plant=Some(word(&raw,0));return Ok(after);
                }
                let mut record=*record;if record[0x5e]==0 {record[0x5e]=8;}
                //62949..62E11: a real hostile query preempts food/mating.
                //Only the sole live owned-Mule candidate and direct acquisition
                //disposition are proved here. All alternate outcomes still reject.
                if let Some(enemy)=world.mule_single_owned_hostile(&record).map_err(|_|CreatureError::UnsupportedNeighborhood)? {
                    let hostile=crate::mule_hostile_acquisition::plan_acquisition(&record,&enemy,*rng).map_err(|_|CreatureError::UnsupportedNeighborhood)?;
                    *rng=hostile.rng_after;return Ok(hostile.after);
                }
                //62EA9..63045 idle mate action after the no-food-priority gates.
                //Neutral5D700 returns true without querying a specialist.
                if before[0x5e]==8&&record[0x5e]==8&&record[0x77]==255&&(record[0x79]as i8)>0
                    &&record[0x61]&0x40==0&&word(&record,0x68)>(word(&record,0x62)>>1){
                    if let Some(peer_id)=world.mule_neutral_pair_candidate(&record,*rng).map_err(|_|CreatureError::UnsupportedNeighborhood)?{
                        let other=*world.creatures.as_ref().unwrap().record(peer_id as usize)?;
                        if world.creatures.as_ref().unwrap().binding(peer_id as usize)?!=Some(CreatureAnimationBinding::MuleSelector4){return Err(CreatureError::UnsupportedNeighborhood);}
                        let pair=crate::mule_idle_pair_init::plan_pair_init(&record,&other).map_err(|_|CreatureError::UnsupportedNeighborhood)?;
                        for _ in 0..2{rng.seed=rng.seed.wrapping_mul(0xe62d);rng.ring=(rng.ring+1)&31;rng.draw_count=rng.draw_count.wrapping_add(1);}
                        //63045 falls through5EF46 into the newly selected state3.
                        //Different cells only: the same-cell rendezvous is a separate transaction.
                        let point=[word(&other,7),word(&other,9)];
                        if floor_cell(point)==floor_cell([word(&record,7),word(&record,9)]){return Err(CreatureError::UnsupportedMotionState);}
                        let mut after=pair.current_after;
                        if creature_world::mule_callback_phase(&before,context.frame_counter)&3==3{after[11..13].copy_from_slice(&point[0].to_le_bytes());after[13..15].copy_from_slice(&point[1].to_le_bytes());after[0x49]|=4;}
                        pending_pair=Some((peer_id,other,pair.peer_after,Vec::new()));
                        return Ok(after);
                    }
                }
                if record[24]==4&&record[0x5e]==21&&word(&record,0x6c)==0&&hostile_enemy.is_none()
                    &&reference(&record,0x55)[2]==1 {
                    //60332 resolves the pursued target from the attacker's own55
                    //link after the state dispatch; the reciprocal50/55 kind byte
                    //may already be cleared by13EB0 when the owned target entered
                    //its hit domain, so identity is the link id, not the kind
                    //byte. Only the measured live owned-Mule shape is admitted.
                    let id=word(&record,0x55) as usize;
                    let target=*world.creatures.as_ref().unwrap().record(id)?;
                    if world.creatures.as_ref().unwrap().binding(id)?==Some(CreatureAnimationBinding::MuleSelector4)
                        &&target[6]==1&&target[24]==0&&target[0x7a]==4&&word(&target,0x6a)>0
                        &&word(&target,0x49) as u16&0x400==0{
                        return world.mule_neutral_completed_attack(&record,&target,&mut rng,physics,&mut pending_audio);
                    }
                }
                //The food query validates every candidate, including edible corpses.
                let target=world.mule_neutral_food_target(&record,*rng).map_err(|_|CreatureError::UnsupportedNeighborhood)?;
                let choice=if let Some((raw,point))=target {if raw[2]==2{creature_world::plan_mule_plant_acquisition(&record,word(&raw,0),point,*rng)?}else{let corpse=world.creatures.as_ref().unwrap().record(word(&raw,0)as usize)?;creature_world::plan_mule_corpse_acquisition(&record,corpse,*rng)?}}else{
                    let forced=world.mule_cell_entry_predicate(&record,floor_cell([word(&record,7),word(&record,9)])).map_err(|_|CreatureError::UnsupportedNeighborhood)?;
                    creature_world::plan_neutral_mule_rechoice_with_cell_predicate(&record,*rng,trig,forced,|point|world.mule_accepts(slot,point).map_err(|_|CreatureError::MotionEnvironment))?
                };
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
            },|record|{
                let after=*record;
                if hostile_enemy.is_some()&&before[0x5e]==1 {
                    let phase=creature_world::mule_callback_phase(&before,context.frame_counter);
                    //65C60 runs on every odd callback, after physical motion.
                    //Only far pursuit with no abandonment draw is admitted.
                    if phase&1!=0 {
                        if phase&3==3&&32767-i32::from(word(record,0x6c))>256{return Err(CreatureError::UnsupportedMotionState);}
                        //Contact response is dispatched after energy/food and action cleanup.
                    }
                }
                Ok(after)
            },hostile_enemy.is_some())?;
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
        if let Some((id,expected,reserve))=feed_bones {
            let prior=*staged.creatures.as_ref().unwrap().record(id as usize)?;
            if word(&prior,0x68)!=expected{return Err(CreatureError::UnsupportedMotionState.into());}
            let mut after=prior;after[0x68..0x6a].copy_from_slice(&reserve.to_le_bytes());staged.creatures.as_mut().unwrap().replace_record_checked(id as usize,&prior,&after)?;
        }
        if let Some((id,expected,flags))=reserve_corpse {
            let prior=*staged.creatures.as_ref().unwrap().record(id as usize)?;
            if prior[0x60]!=expected{return Err(CreatureError::UnsupportedMotionState.into());}
            let mut after=prior;after[0x60]=flags;staged.creatures.as_mut().unwrap().replace_record_checked(id as usize,&prior,&after)?;
        }
        if let Some(id)=reserve_plant {let l=staged.layouts[1];let at=l.file_offset+id as usize*l.packed_stride+0x26;staged.backing[at]|=1;}
        if let Some(id)=release_plant {let l=staged.layouts[1];let at=l.file_offset+id as usize*l.packed_stride+0x26;staged.backing[at]&=0xfe;}
        if let Some(id)=fed_plant {let l=staged.layouts[1];let at=l.file_offset+id as usize*l.packed_stride+0x24;let old=word(&staged.backing,at);staged.backing[at..at+2].copy_from_slice(&old.wrapping_sub(3).to_le_bytes());}
        if !retired_during_feeding{for id in &pending_plant_retirements {staged.retire_mule_food_plant(*id)?;}}
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
        //5D011..5D02E: lethal health death requests virtual44 sound1F.
        //The numeric planner admits only the owner4 reason4/BL0 branch.
        if before[0x5e]==8&&plan.after[0x5e]==24&&word(&plan.after,0x6a)==0 {pending_audio.push(0x1f);}
        staged.rng=rng;
        #[cfg(any(debug_assertions, feature = "callback-captures"))] staged.capture_numeric_callback(slot,context.frame_counter,"after");
        staged.validate_links()?;let mut result=plan;result.after=*staged.creatures.as_ref().unwrap().record(slot)?;*self=staged;Ok(MuleCallbackReceipt{motion:result,retirement:None,newborn_slots,pending_audio,pending_plant_retirements,pending_portrait_refresh:before[0x5e]==8&&prefix[0x5e]==0})
    }

    ///60332 neutral owner4 completed attack: the expired attack timer hits the
    ///pursued owned target through the shared5B9F0 damage body (direction
    ///flipped from the proven owned packet) and immediately reattacks through
    ///the 6096A..612B6 sole-enemy scan, 5B580's local reset and the proven
    ///5C840/65DB5 entry planner. Mirrors ActorWorld::owned_mule_completed_attack
    ///with the neutral caller domain (attacker owner4/0x61=0x40/task0). Every
    ///stage runs inside the existing cloned transaction; late rejection
    ///discards the target write, both records and every RNG draw together.
    ///The 603C0 range miss (owner0-domain planner) and the5BC63 stop branch
    ///stay explicit unsupported boundaries.
    fn mule_neutral_completed_attack(&mut self,record:&CreatureRecord,enemy:&CreatureRecord,rng:&mut BlueprintRng,physics:&PhysicsTables,audio:&mut Vec<u8>)->Result<CreatureRecord,CreatureError>{
        if record[24]!=4||record[0x5e]!=21||record[0x61]!=0x40||record[0x60]!=0x88||record[0x78]!=0||word(record,0x6c)!=0||record[0x76]>90
            ||record[0x7a]!=4||record[0x7b]!=255||record[0x15]!=0x80||!matches!(record[0x16],2|0x82)||record[0x77]!=255
            ||record[0x4f]!=0||record[0x5d]!=0||record[0x52]!=1||record[0x57]!=1
            ||reference(record,0x50)!=reference(record,0x55)||word(record,0x49)as u16&0x100!=0||word(record,0x49)as u16&0x400!=0
            {return Err(CreatureError::UnsupportedMotionState);}
        let xy=genewars_core::cd_candidate::distance_squared_wrapping_15([word(record,7),word(record,9)],[word(enemy,7),word(enemy,9)])as i32;
        let z=(i32::from_le_bytes(enemy[39..43].try_into().unwrap()).wrapping_sub(i32::from_le_bytes(record[39..43].try_into().unwrap())))>>3;
        if (xy.wrapping_add(z.wrapping_mul(z))as u32)>16384{return Err(CreatureError::UnsupportedMotionState);}
        let mut after=*record;
        let trait_value=10*(50+50*i32::from(record[0x76])/100)/100;
        let cost=(trait_value>>2).min(i32::from(word(record,0x68)));
        after[0x68..0x6a].copy_from_slice(&(word(record,0x68)-cost as i16).to_le_bytes());
        let draw=|r:&mut BlueprintRng|{r.seed=r.seed.wrapping_mul(0xe62d);r.ring=r.ring.wrapping_add(1)&31;r.draw_count=r.draw_count.wrapping_add(1);(r.seed>>8)as u8};
        if rng.ring>=32{return Err(CreatureError::UnsupportedMotionState);}
        let incoming=((trait_value>>1).max(cost*2)+i32::from(draw(rng)%10))as i16;
        let target_id=word(enemy,15)as usize;
        let prior=*self.creatures.as_ref().unwrap().record(target_id)?;if prior!=*enemy{return Err(CreatureError::UnsupportedMotionState);}
        //5B9F0 shared nonlethal damage with the flipped direction: the target
        //is the owned task7 Mule; defense uses the target trait and the packet
        //keeps the proven nonlethal1..4/no-blood domain.
        let percent=50+50*i32::from(prior[0x76])/100;let defense=6*percent/100;
        let effective=(i32::from(incoming)-defense).max(1);
        let attack_ref=[target_id as u8,(target_id>>8)as u8,1];
        let attacker_ref=[record[15],record[16],1];
        if incoming<=0||effective>4||i32::from(word(&prior,0x6a))<=effective
            ||word(&prior,0x50)!=i16::from(word(record,15))||word(&prior,0x55)!=i16::from(word(record,15))
            ||word(&prior,0x49)as u16&0x400!=0||word(&prior,0x6a)<=0{return Err(CreatureError::UnsupportedMotionState);}
        let mut target_after=prior;
        target_after[0x15]&=0xdf;target_after[0x33..0x37].fill(0);
        target_after[0x60..0x62].copy_from_slice(&((word(&prior,0x60)as u16)&0xd71a).to_le_bytes());
        target_after[0x5d]=0;target_after[0x57]=1;
        target_after[0x6a..0x6c].copy_from_slice(&(word(&prior,0x6a)-effective as i16).to_le_bytes());
        target_after[94]=20;target_after[0x6c..0x6e].copy_from_slice(&32767i16.to_le_bytes());
        let mut damage_rng=*rng;
        let first=draw(&mut damage_rng);
        let damage_audio=if first%2!=0{8u8}else{28u8+draw(&mut damage_rng)%3};
        self.creatures.as_mut().unwrap().replace_record_checked(target_id,&prior,&target_after)?;
        *rng=damage_rng;audio.push(damage_audio);
        if draw(rng)%100>90{return Err(CreatureError::UnsupportedNeighborhood);} //BA4; alternate branch unported
        let sole=self.mule_neutral_attack_single_hostile(&after).map_err(|_|CreatureError::UnsupportedNeighborhood)?.ok_or(CreatureError::UnsupportedNeighborhood)?;
        if word(&sole,15)!=target_id as i16{return Err(CreatureError::UnsupportedNeighborhood);}
        //8B7D0 current-cell predicate: only a shared kind5/code11 effect forces
        //escape; kind1 links (the hit target included) do not.
        {let cell=floor_cell([word(&after,7),word(&after,9)]);let mut raw=reference(self.map(),cell*14+6);
         while raw[2]!=0{let e=self.entity(raw).map_err(|_|CreatureError::UnsupportedNeighborhood)?;if raw[2]==5&&self.effects.as_ref().ok_or(CreatureError::UnsupportedNeighborhood)?.record(word(&raw,0)as usize).map_err(|_|CreatureError::UnsupportedNeighborhood)?[0x45]==0x11{return Err(CreatureError::UnsupportedNeighborhood);}raw=reference(e,3);}}
        //60D91..60F53 query weight/confidence with four ordered draws, then
        //5B580's local reset. 5B5CC's task7 reference58 retention does not run
        //for the task0 neutral attacker; the shared entry planner re-enters21.
        let cell_delta=|a:i16,b:i16|{let a=i32::from((a as u16&0x7fff)>>8);let b=i32::from((b as u16&0x7fff)>>8);(b-a+64).rem_euclid(128)-64};
        let dx=cell_delta(word(&after,7),word(&sole,7));let dy=cell_delta(word(&after,9),word(&sole,9));
        let distance=(dx*dx+dy*dy)*256*256;
        if distance>=1024*1024||word(&after,0x64)<=0{return Err(CreatureError::UnsupportedNeighborhood);}
        let weight=1024-genewars_core::cd_candidate::integer_sqrt(distance as u32)as i32;
        let confidence=i32::from(word(&after,0x6a))*256/i32::from(word(&after,0x64))-((weight/512)<<5)+10;
        if rng.ring>=32{return Err(CreatureError::UnsupportedMotionState);}
        let draws=[draw(rng),draw(rng),draw(rng),draw(rng)];
        if i32::from(draws[2])%100>90||i32::from(draws[3])>=confidence{return Err(CreatureError::UnsupportedNeighborhood);}
        let mut reacquired=after;
        reacquired[0x15]&=0xdf;reacquired[0x33..0x37].fill(0);reacquired[0x5d]=0;reacquired[0x4f]=0;reacquired[0x57]=0;
        reacquired[0x60..0x62].copy_from_slice(&((word(&after,0x60)as u16&0xd71a)|0xc0).to_le_bytes());
        reacquired[0x55..0x58].copy_from_slice(&attack_ref);reacquired[0x57]=1;
        reacquired[0x16]&=0x3f;reacquired[0x49..0x4b].copy_from_slice(&((word(&after,0x49)as u16|0x14)&0xeffc).to_le_bytes());
        reacquired[0x54]=20;reacquired[0x50..0x53].copy_from_slice(&attack_ref);reacquired[0x52]=1;
        let speed=creature_world::mule_requested_speed(&reacquired);reacquired[0x3f..0x41].copy_from_slice(&speed.to_le_bytes());
        reacquired[0x6c..0x6e].copy_from_slice(&32767i16.to_le_bytes());reacquired[94]=1;
        let entry=crate::mule_hostile_attack_entry::plan_entry(&reacquired,&sole,*rng,physics).map_err(|_|CreatureError::UnsupportedMotionState)?;
        *rng=entry.rng_after;if let Some(channel)=entry.pending_audio{audio.push(channel);}
        let mut final_after=entry.after;final_after[0x60]&=0xbf;Ok(final_after)
    }
    ///Direction-flipped 6096A/8D930 sole-hostile circle scan behind the neutral
    ///completed attack: exactly one live owned MuleSelector4 creature (task7
    ///state8, or state20 hit reaction under this expired 0x88 query) and no
    ///friends. Owner4 regards owner0 as hostile independently of alliances, so
    ///no alliance gate applies; kind5 code11, plants and unknown kinds reject.
    fn mule_neutral_attack_single_hostile(&self,r:&CreatureRecord)->Result<Option<CreatureRecord>,ActorWorldError>{
        let attack_query=r[0x5e]==21&&r[0x60]==0x88&&word(r,0x6c)==0;
        if r[24]!=4||!(r[0x5e]==8&&r[0x60]==8||attack_query)||r[0x61]!=0x40||r[0x78]!=0{return Ok(None);}
        let own=[r[15],r[16],1];let cell=floor_cell([word(r,7),word(r,9)]);let(cx,cy)=((cell%128)as i32,(cell/128)as i32);
        let mut hostile=None;let mut unsupported=false;
        for dy in -4..=4{for dx in -4..=4{if dx*dx+dy*dy>16{continue;}let c=(((cy+dy)&127)*128+((cx+dx)&127))as usize;let mut raw=reference(self.map(),c*14+6);
            while raw[2]!=0{let e=self.entity(raw)?;if raw!=own{match raw[2]{
                1=>{let pool=self.creatures.as_ref().ok_or(CreatureError::UnsupportedNeighborhood)?;let other=*pool.record(word(&raw,0)as usize)?;
                    if other[24]==0&&other[0x7a]==4&&(other[0x5e]==8||attack_query&&other[0x5e]==20)&&pool.binding(word(&raw,0)as usize)?==Some(CreatureAnimationBinding::MuleSelector4){
                        if hostile.is_some(){return Err(CreatureError::UnsupportedNeighborhood.into());}hostile=Some(other);
                    }else{unsupported=true;}
                },
                5=>{let pool=self.effects.as_ref().ok_or(CreatureError::UnsupportedNeighborhood)?;if pool.record(word(&raw,0)as usize)?[69]==0x11||pool.binding(word(&raw,0)as usize)?.is_none(){unsupported=true;}},
                8=>{},_=>unsupported=true,
            }}raw=reference(e,3);}
        }}
        if hostile.is_some()&&unsupported{return Err(CreatureError::UnsupportedNeighborhood.into());}
        Ok(hostile)
    }
    /// Opt-in immutable callback fixture for source comparison; absent in release.
    #[cfg(any(debug_assertions, feature = "callback-captures"))]
    pub(super) fn capture_numeric_callback(&self,slot:usize,frame:u32,phase:&str){
        // A campaign replay resets frame numbers between missions. Filter
        // by the actual retained player progress to avoid cross-mission paths.
        if let Ok(value)=std::env::var("GENEWARS_MULE_CALLBACK_CAMPAIGN_PROGRESS") {
            let progress=value.parse::<u8>().expect("explicit capture campaign progress");
            if self.players[0].bytes()[0x6e7]!=progress{return;}
        }
        let extra=std::env::var("GENEWARS_MULE_CALLBACK_FRAME").ok().map(|v|v.split(',').map(str::parse::<u32>).collect::<Result<Vec<_>,_>>().expect("explicit numeric callback frames"));
        if !phase.starts_with("blocked-before")&&extra.as_ref().map_or(frame>1,|requested|!requested.contains(&frame)){return;} // Explicit frame or default first two.
        let Some(root)=std::env::var_os("GENEWARS_MULE_CALLBACK_FIXTURE") else{return;};
        let dir=std::path::PathBuf::from(root).join(format!("frame-{frame}-slot-{slot}-{phase}"));
        std::fs::create_dir(&dir).expect("new immutable Mule callback fixture");
        for(name,bytes)in [("backing",self.backing_bytes().to_vec()),("players",(0..4u8).flat_map(|p|self.player(p).unwrap().bytes().iter().copied()).collect()),("actors",self.actors().records().concat()),("creatures",self.creatures().unwrap().records().concat()),("effects",self.effects().unwrap().records().concat()),("family8",self.family8().unwrap().records().concat()),("buildings",self.buildings().unwrap().records().concat())]{std::fs::write(dir.join(format!("{name}.bin")),bytes).unwrap();}
        let mut rng=self.rng.seed.to_le_bytes().to_vec();rng.push(self.rng.ring);rng.extend(self.rng.draw_count.to_le_bytes());std::fs::write(dir.join("rng.bin"),rng).unwrap();
    }
    pub(super) fn mule_route_prefix(&self,before:&CreatureRecord,frame:u32,physics:&PhysicsTables,trig:&TrigTables,rng:&mut BlueprintRng,light:bool)->Result<(CreatureRecord,[i16;2]),ActorWorldError> {
        let phase=creature_world::mule_callback_phase(before,frame);
        let own=[before[15],before[16],1];let current=[word(before,7),word(before,9)];
        let retained_food_association=before[24]==4&&before[0x5e]==8&&before[0x60]==8&&before[0x61]==0x40
            &&before[0x57]==0&&before[0x52]==1&&word(before,0x50)==word(before,0x55);
        let associated_pair=matches!(before[0x5e],1|3|4|5|20|21)&&before[0x52]==1&&before[0x57]==1&&reference(before,0x50)==reference(before,0x55)||retained_food_association
                        ||before[24]==0&&before[0x5e]==8&&before[0x60]==0x88&&before[0x61]==0&&before[0x78]==7&&word(before,0x6c)>0&&before[0x52]==1&&before[0x57]==1&&before[0x5a]==1&&reference(before,0x50)==reference(before,0x55)&&reference(before,0x58)==reference(before,0x55)
            ||before[24]==0&&before[0x5e]==8&&before[0x60]==0x88&&before[0x61]==0&&before[0x78]==7
             &&word(before,0x6c)==0&&before[0x52]==1&&before[0x57]==1&&before[0x5a]==1
             &&reference(before,0x50)==reference(before,0x55)&&reference(before,0x58)==reference(before,0x55)
            //5924 measured retained command dispatch: state8, 0x61=0x80,
            //0x60=0x18, 0x6c==0, retained 0x52=1 with cleared57. The ordinary
            //AI choice machinery (624B8) performs the dispatch.
            ||before[24]==0&&before[0x5e]==8&&before[0x60]==0x18&&before[0x61]==0x80&&before[0x78]==7&&word(before,0x6c)==0&&before[0x52]==1&&before[0x57]==0&&before[0x5a]==1;        let mut final_target=[word(before,11),word(before,13)];
        let mut a=*before;a[0x49]&=0x9f;
        if a[0x4f]!=0||(!light&&(a[0x52]!=0&&!associated_pair))||a[0x4a]&0x04!=0{return Err(CreatureError::UnsupportedMotionState.into());}
        if a[0x49]&3==3 {
            let associated_goal=if a[0x52]!=0{let peer=self.entity(reference(&a,0x50))?;Some([word(peer,7),word(peer,9)])}else{None};
            if let Some(goal)=associated_goal{final_target=goal;}
            let wait=crate::mule_waypoint_wait::plan_wait_with_association(&a,frame,physics,associated_goal,|goal|self.mule_cell_entry_predicate(&a,floor_cell(goal))).map_err(|_|CreatureError::UnsupportedMotionState)?;
            return Ok((creature_world::plan_common_ground_steering(&wait.record_after,trig)?,final_target));
        }
        if a[0x5e]<24&&a[0x49]&0x10==0&&phase&31==0&&!self.common_move_point_accepted(own,current)? {
            if let Some(point)=creature_world::find_common_idle_relocation(current,|point|self.common_move_point_accepted(own,point).map_err(|_|CreatureError::MotionEnvironment))? {a=creature_world::plan_mule_idle_relocation(&a,point)?;a[0x49]&=0xfe;}
        }
        let mut working=if a[0x49]&3!=0 {[word(&a,0x45),word(&a,0x47)]}else{final_target};
        let mut immediate_heading=false;
        if a[0x49]&0x10!=0&&(a[0x49]&0x0c!=0||(associated_pair&&phase&3==3&&a[0x49]&3==0)) {
            if associated_pair{let peer=self.entity(reference(before,0x50))?;final_target=[word(peer,7),word(peer,9)];a[11..13].copy_from_slice(&final_target[0].to_le_bytes());a[13..15].copy_from_slice(&final_target[1].to_le_bytes());if a[0x49]&3==0{working=final_target;}}
            //6FEAB..6FEB7 kind1 uses74D90 obstacle plus terrain admission.
            let accepted=self.common_move_point_accepted(own,final_target)?;
            if !accepted {
                //6FEAB..6FED3 applies74A50 to rejected owned idle goals too;
                // association50 only refreshes a goal earlier at6FE8C.
                let owned_idle_target=before[24]<4&&before[122]==4&&before[0x5e]==8
                    &&before[0x60]==0x88&&before[0x61]==0&&before[0x78]==0
                    &&before[0x4f]==0&&before[0x52]==0&&before[0x57]==0;
                let owned_command_target=before[24]<4&&before[122]==4&&matches!(before[0x5e],0|8)
                    &&matches!(before[0x60],0x18|0x98)&&before[0x61]==0x80&&before[0x78]==7
                    &&before[0x4f]==0&&before[0x52]==0&&before[0x57]==0;
                if !associated_pair&&!owned_idle_target&&!owned_command_target{return Err(CreatureError::TargetSearchRequired.into());}
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

    pub(super) fn mule_contact_speed(&self,record:&CreatureRecord,normal:[i32;3],previous:i16)->Result<i16,ActorWorldError> {
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
        let distance=if (matches!(record[0x5e],1|3|4|5)||record[24]==4&&record[0x5e]==8)&&record[0x52]==1{
            //5AD70 virtualB8 uses the associated Mule's physical xyz,
            //even when74A50 repaired the ordinary goal back to self.
            let peer=self.creatures.as_ref().ok_or(CreatureError::MotionEnvironment)?.record(word(record,0x50)as usize)?;
            mule_pair_distance(record,peer)
        }else{creature_world::common_distance_squared(self.map(),point,target)?};
        Ok(creature_world::plan_common_contact_speed(CommonSpeedInput{speed:word(record,0x41),desired_speed:desired,
            width:word(record,0x2b)as u16,flags_4a:record[0x4a],free_flight:false,ground_height:i32::from_le_bytes(record[0x11..0x15].try_into().unwrap()),
            previous_height_low:previous,surface_term:normal[2],distance_squared:distance})?)
    }

    pub(super) fn plan_mule_cell_entry(&self,slot:usize,record:&CreatureRecord,cell:usize,rng:&mut BlueprintRng,trig:&TrigTables)->Result<CreatureRecord,CreatureError>{
        if !self.mule_cell_entry_predicate(record,cell).map_err(|_|CreatureError::UnsupportedCellEntry)?{return Ok(*record);}
        if record[0x53]!=1||record[0x4f]!=0{return Err(CreatureError::UnsupportedCellEntry);}
        let plan=crate::mule_cell_entry::plan_waypoint(record,cell as u16,*rng,trig,|point|self.common_move_point_accepted([slot as u8,(slot>>8)as u8,1],point))
            .map_err(|_|CreatureError::UnsupportedCellEntry)?;
        *rng=plan.rng_after;Ok(plan.record_after)
    }
    /// Bounded5B100 predicate shared by cell entry and waypoint waiting.
    /// Typed live hostile pursuit. Corpse feeding keeps its independent guard.
    ///5F5A9 completed hit query: exactly one live owned enemy, no friends.
    ///Owner4 classification treats owned0 as hostile independently of alliances.
    ///Known non11 effects/kind8 skip before scoring; all other candidates remain guarded.
    fn mule_hit_single_owned_enemy(&self,r:&CreatureRecord)->Result<CreatureRecord,ActorWorldError>{
        let own=[r[15],r[16],1];let cell=floor_cell([word(r,7),word(r,9)]);let(cx,cy)=((cell%128)as i32,(cell/128)as i32);let mut enemy=None;
        for dy in -4..=4{for dx in -4..=4{if dx*dx+dy*dy>16{continue;}let c=(((cy+dy)&127)*128+((cx+dx)&127))as usize;let mut raw=reference(self.map(),c*14+6);
            while raw[2]!=0{let e=self.entity(raw)?;if raw!=own{match raw[2]{
                1=>{let pool=self.creatures.as_ref().ok_or(CreatureError::UnsupportedNeighborhood)?;let other=*pool.record(word(&raw,0)as usize)?;
                    if other[24]!=0||other[0x5e]!=21||other[0x7a]!=4||pool.binding(word(&raw,0)as usize)?!=Some(CreatureAnimationBinding::MuleSelector4)||enemy.is_some(){return Err(CreatureError::UnsupportedNeighborhood.into());}enemy=Some(other);
                },
                5=>{let pool=self.effects.as_ref().ok_or(CreatureError::UnsupportedNeighborhood)?;if pool.record(word(&raw,0)as usize)?[69]==0x11||pool.binding(word(&raw,0)as usize)?.is_none(){return Err(CreatureError::UnsupportedNeighborhood.into());}},
                8=>{},_=>return Err(CreatureError::UnsupportedNeighborhood.into()),
            }}raw=reference(e,3);}
        }}enemy.ok_or(CreatureError::UnsupportedNeighborhood.into())
    }
    fn mule_live_hostile_target(&self,r:&CreatureRecord)->Result<Option<CreatureRecord>,ActorWorldError>{
        if r[6]!=1||r[24]!=4||r[0x7a]!=4||!matches!(r[0x5e],1|20|21)||r[0x61]!=0x40||!(if r[0x5e]==1{matches!(r[0x60],0x48|0xc8)}else{matches!(r[0x60],8|0x88)})
            ||r[0x78]!=0||r[0x76]>90||r[0x4f]!=0||r[0x5d]!=0||r[0x57]!=1||r[0x52]!=1
            ||reference(r,0x50)!=reference(r,0x55)||word(r,0x49)as u16&0x100!=0{return Ok(None);}
        let pool=self.creatures.as_ref().ok_or(CreatureError::UnsupportedMotionState)?;let id=word(r,0x55)as usize;let target=*pool.record(id)?;
        if target[6]==1&&target[24]==0&&target[0x7a]==4&&(target[0x5e]==8||(matches!(target[0x5e],1|20|21)&&target[0x78]==7&&target[0x61]==0&&(if target[0x5e]==1{matches!(target[0x60],0x48|0xc8)}else{matches!(target[0x60],8|0x88)})
            &&target[0x4f]==0&&target[0x5d]==0&&reference(&target,0x50)==[r[15],r[16],1]
            &&reference(&target,0x55)==[r[15],r[16],1]&&reference(&target,0x58)==[r[15],r[16],1]))&&word(&target,0x6a)>0
            &&word(&target,0x49)as u16&0x400==0&&pool.binding(id)?==Some(CreatureAnimationBinding::MuleSelector4){Ok(Some(target))}else{Ok(None)}
    }
    pub(super) fn mule_cell_entry_predicate(&self,record:&CreatureRecord,cell:usize)->Result<bool,ActorWorldError>{
        let own=[record[15],record[16],1];
        let approach_plant=record[0x5e]==1&&record[0x57]==2&&self.entity(reference(record,0x55))?[6]==2;
        let approach_corpse=record[0x5e]==1&&record[0x57]==1&&record[0x52]==1&&reference(record,0x50)==reference(record,0x55)
            &&matches!(self.creatures.as_ref().unwrap().record(word(record,0x55)as usize)?[0x5e],24..=27);
        if !matches!(record[0x5e],3|8)&&!approach_plant&&!approach_corpse&&self.mule_live_hostile_target(record)?.is_none()&&self.owned_mule_live_hostile_target(record)?.is_none(){self.check_mule_cell_entry(own,cell)?;return Ok(false);}
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
    pub(super) fn check_mule_cell_entry(&self,own:[u8;3],cell:usize)->Result<(),ActorWorldError> {
        let mut raw=reference(self.map(),cell*14+6);
        while raw[2]!=0 {if raw[2]==1&&raw!=own {return Err(CreatureError::UnsupportedCellEntry.into());}
            if raw[2]==5 {let pool=self.effects.as_ref().ok_or(CreatureError::UnsupportedCellEntry)?;if pool.record(word(&raw,0)as usize)?.get(0x45)==Some(&0x11){return Err(CreatureError::UnsupportedCellEntry.into());}}
            raw=reference(self.entity(raw)?,3);
        }Ok(())
    }
    ///6B7C6..6BA40: neutral owner4 regards owner0 as hostile, independent
    ///of alliances. One live kind1 Mule(state8) publishes both enemy scores;
    ///its count/pointer do not depend on score ties. Known neutral friends,
    ///plants and skipped effects consume no extra query RNG. Query direction
    ///and food/mate outputs are unused by the admitted direct629C0 branch.
    fn mule_single_owned_hostile(&self,record:&CreatureRecord)->Result<Option<CreatureRecord>,ActorWorldError>{
        let own=[record[15],record[16],1];let cell=floor_cell([word(record,7),word(record,9)]);
        let(cx,cy)=((cell%128)as i32,(cell/128)as i32);let mut enemy=None;let mut unsupported=false;
        for dy in -4..=4{for dx in -4..=4{if dx*dx+dy*dy>16{continue;}
            let index=((cy+dy)&127)as usize*128+((cx+dx)&127)as usize;let mut raw=reference(self.map(),index*14+6);
            while raw[2]!=0{let entity=self.entity(raw)?;
                if raw!=own{match raw[2]{
                    1=>{let pool=self.creatures.as_ref().ok_or(CreatureError::UnsupportedNeighborhood)?;let other=*pool.record(word(&raw,0)as usize)?;
                        let binding=if other[0x5e]>=26{CreatureAnimationBinding::BonesSelector4}else{CreatureAnimationBinding::MuleSelector4};
                        if other[6]!=1||other[0x7a]!=4||pool.binding(word(&raw,0)as usize)?!=Some(binding){unsupported=true;}
                        else if other[24]==4 {if !matches!(other[0x5e],0|1|3|4|5|7|8|10|15|24..=27){unsupported=true;}}
                        else if other[24]==0&&other[0x5e]==8&&binding==CreatureAnimationBinding::MuleSelector4 {
                            if enemy.is_some(){return Err(CreatureError::UnsupportedNeighborhood.into());}enemy=Some(other);
                        }else{unsupported=true;}
                    },
                    2=>if entity[24]!=4{unsupported=true;},
                    5=>{let effect=self.effects.as_ref().ok_or(CreatureError::UnsupportedNeighborhood)?.record(word(&raw,0)as usize)?;if effect[0x45]==0x11{unsupported=true;}},
                    8=>{},_=>unsupported=true,
                }}raw=reference(entity,3);
            }
        }}
        if enemy.is_some()&&unsupported{return Err(CreatureError::UnsupportedNeighborhood.into());}
        Ok(enemy)
    }
    fn check_mule_neutral_neighbors(&self,record:&CreatureRecord,own:[u8;3],point:[i16;2])->Result<(),ActorWorldError>{
        let cell=floor_cell(point);
        let(cx,cy)=((cell%128)as i32,(cell/128)as i32);
        //8D99A..8D9AA excludes samples beyond radius1024, retaining its boundary.
        for dy in -4..=4 {for dx in -4..=4 {
            if dx*dx+dy*dy>16{continue;}
            let index=((cy+dy)&127)as usize*128+((cx+dx)&127)as usize;let mut raw=reference(self.map(),index*14+6);
            while raw[2]!=0 {let entity=self.entity(raw)?;let ignored_effect=if raw[2]==8{true}else if raw[2]==5{self.effects.as_ref().ok_or(CreatureError::UnsupportedNeighborhood)?.record(word(&raw,0)as usize)?.get(0x45)!=Some(&0x11)}else{false};if raw!=own&&!ignored_effect&&(raw[2]!=2||entity[24]!=4)&&!self.mule_friendly_candidate_is_no_action(record,raw)?{return Err(CreatureError::UnsupportedNeighborhood.into());}raw=reference(entity,3);}
        }}Ok(())
    }
    ///679CE state4 query:68445 ignores food,684DB requires reserve>half for mating.
    /// Only same-owner/species kind1 neighbors are admitted, proving no enemies;
    /// reserved Bones skip6BAA6 scoring; other friend directions are unused.
    fn check_mule_bones_feeding_no_action(&self,record:&CreatureRecord)->Result<(),ActorWorldError>{
        if record[0x5e]!=4||record[24]!=4||record[0x7a]!=4||record[0x60]!=8||record[0x61]!=0x40
            ||word(record,0x68)>(word(record,0x62)>>1){return Err(CreatureError::UnsupportedNeighborhood.into());}
        let cell=floor_cell([word(record,7),word(record,9)]);let(cx,cy)=((cell%128)as i32,(cell/128)as i32);
        for dy in -4..=4{for dx in -4..=4{
            if dx*dx+dy*dy>16{continue;}
            let index=((cy+dy)&127)as usize*128+((cx+dx)&127)as usize;let mut raw=reference(self.map(),index*14+6);
            while raw[2]!=0{
                if raw[2]!=1{return Err(CreatureError::UnsupportedNeighborhood.into());}
                let id=word(&raw,0)as usize;let pool=self.creatures.as_ref().ok_or(CreatureError::UnsupportedNeighborhood)?;let other=pool.record(id)?;
                let binding=if other[0x5e]>=26{CreatureAnimationBinding::BonesSelector4}else{CreatureAnimationBinding::MuleSelector4};
                if other[6]!=1||other[24]!=4||other[0x7a]!=4||other[0x5e]>27||pool.binding(id)?!=Some(binding){return Err(CreatureError::UnsupportedNeighborhood.into());}
                raw=reference(other,3);
            }
        }}Ok(())
    }
    ///6BA65/6BBAE: same-species live friend contributes no food candidate.
    /// Friendly direction is unused only with the separately enforced absence
    /// of enemies (62949..52); full field77 can select a paired action.
    fn mule_friendly_candidate_is_no_action(&self,record:&CreatureRecord,raw:[u8;3])->Result<bool,ActorWorldError>{
        if raw[2]!=1||record[0x18]!=4||record[0x7a]!=4||record[0x76]>90{return Ok(false);}
        let other=self.creatures.as_ref().ok_or(CreatureError::UnsupportedNeighborhood)?.record(word(&raw,0)as usize)?;
        if other[6]!=1||other[0x18]!=4||other[0x7a]!=4{return Ok(false);}
        //6BAA6 reserved Bones skip scoring; unreserved6BAB6..6BAF3 only direction.
        //The surrounding no-pair/no-enemy domain does not consume that direction.
        if matches!(other[0x5e],26|27){return Ok(self.creatures.as_ref().unwrap().binding(word(&raw,0)as usize)?==Some(CreatureAnimationBinding::BonesSelector4));}
        if record[0x77]==255&&(record[0x79]as i8)>0&&word(record,0x68)>(word(record,0x62)>>1){return Ok(false);}
        //6BA8A..6BB57 still score friendly corpses as food. Their score and
        //direction are unused only in this satiated, no-pair, no-enemy domain.
        if other[0x5e]>=24{return Ok(record[0x61]&0x40==0&&word(record,0x68)>(word(record,0x62)>>1));}
        Ok(other[0x5e]!=15)
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
    fn mule_neutral_food_target(&self,record:&CreatureRecord,rng:BlueprintRng)->Result<Option<([u8;3],[i16;2])>,ActorWorldError> {
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
                if raw!=own {
                    if raw[2]==1 {let corpse=self.creatures.as_ref().unwrap().record(word(&raw,0)as usize)?;
                        if corpse[24]==4&&corpse[0x7a]==4&&matches!(corpse[0x5e],24|25)
                            &&self.creatures.as_ref().unwrap().binding(word(&raw,0)as usize)?==Some(CreatureAnimationBinding::MuleSelector4){
                            //625C3..626AC Mule query bonuses are zero;6BB32..57 doubles unreserved corpses.
                            let food=if corpse[0x60]&1==0{value*2}else{value};
                            if food>score{score=food;best=Some((raw,[word(corpse,7),word(corpse,9)]));}
                            raw=reference(entity,3);continue;
                        }
                    }
                    if self.mule_friendly_candidate_is_no_action(record,raw)?{raw=reference(entity,3);continue;}if raw[2]!=2||entity[0x18]!=4 {return Err(CreatureError::UnsupportedNeighborhood.into());}
                    if self.mule_plant_flags(word(&raw,0))?&9==0&&self.mule_plant_health(word(&raw,0))?>0&&value>score {score=value;best=Some((raw,[word(entity,7),word(entity,9)]));}}
                raw=reference(entity,3);
            }
        }}Ok(best)
    }
    ///6B700/69334/62EA9: bounded neutral same-Mule query and pair winner.
    /// The supplied RNG is BEFORE the two query-initialization draws.
    fn mule_neutral_pair_candidate(&self,record:&CreatureRecord,rng:BlueprintRng)->Result<Option<u16>,ActorWorldError>{
        if record[0x18]!=4||record[0x7a]!=4||!matches!(record[0x5e],7|8)||record[0x77]!=255||(record[0x79]as i8)<=0{return Err(CreatureError::UnsupportedNeighborhood.into());}
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
                        if other[0x18]!=4||other[0x7a]!=4{return Err(CreatureError::UnsupportedNeighborhood.into());}
                        //6BAF3 skips both best-mate and best-food publication.
                        if matches!(other[0x5e],26|27)
                            &&self.creatures.as_ref().unwrap().binding(word(&raw,0)as usize)?==Some(CreatureAnimationBinding::BonesSelector4){raw=reference(entity,3);continue;}
                        if other[0x5e]>=24{return Err(CreatureError::UnsupportedNeighborhood.into());}
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
    pub(super) fn insert_mule_head(&mut self,slot:usize,cell:usize)->Result<(),ActorWorldError> {
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
    pub(super) fn mule_accepts(&self,slot:usize,point:[i16;2])->Result<bool,ActorWorldError> {
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
                //Building112120+10=1F410(false);74D90 obstaclequery ran first.
                1|3|4|5|8=>(),
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
