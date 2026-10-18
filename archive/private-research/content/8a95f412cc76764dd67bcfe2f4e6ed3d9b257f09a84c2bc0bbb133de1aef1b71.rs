//! Bounded neutral Mule callbacks. Ordinary scheduler gates and unsupported
//! pathfinding/AI branches remain separate from this owned transaction.
use super::*;
use crate::creature_world::{self,InitialMuleChoice,CreatureAnimationBinding,CommonSpeedInput,MuleStepContext,MuleAnimations};
use crate::physics_tables::PhysicsTables;

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct MuleCallbackReceipt {
    pub motion:creature_world::CommonPhysicsPlan<124>,
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
        self.step_mule_followup_inner(context,physics,trig,animations,false)
    }
    /// Same owned callback with source-backed code30 water factories enabled.
    /// Unsupported water damage/code15 callbacks still reject atomically.
    pub fn step_mule_followup_with_water(&mut self,context:MuleStepContext,physics:&PhysicsTables,trig:&TrigTables,animations:&MuleAnimations)->Result<MuleCallbackReceipt,ActorWorldError> {
        self.step_mule_followup_inner(context,physics,trig,animations,true)
    }
    fn step_mule_followup_inner(&mut self,context:MuleStepContext,physics:&PhysicsTables,trig:&TrigTables,animations:&MuleAnimations,admit_water:bool)->Result<MuleCallbackReceipt,ActorWorldError> {
        self.validate_links()?;
        let pool=self.creatures.as_ref().ok_or(CreatureError::UnsupportedMotionState)?;
        if pool.binding(0)?!=Some(CreatureAnimationBinding::MuleSelector4)||pool.records().iter().skip(1).any(|r|r[6]!=0){return Err(CreatureError::UnsupportedMotionState.into());}
        let before=*pool.record(0)?;
        if matches!(before[0x5e],1|7) {let target=self.entity(reference(&before,0x55))?;let expected=if before[0x5e]==7{1}else{0};if before[0x57]!=2||target[0x18]!=4||self.mule_plant_flags(word(&before,0x55))?&9!=expected {return Err(CreatureError::UnsupportedMotionState.into());}}
        let mut rng=self.rng;
        let (prefix,working_target)=self.mule_route_prefix(&before,context.frame_counter,physics,trig,&mut rng)?;
        let mut reserve_plant=None;let mut release_plant=None;let mut fed_plant=None;let mut pending_audio=Vec::new();let mut pending_plant_retirements=Vec::new();
        let(plan,water_effect)=creature_world::plan_mule_after_admitted_prefix_with_water_policy(&before,&prefix,working_target,self.map(),context,physics,trig,
            |record,normal,previous|self.mule_contact_speed(record,normal,previous).map_err(|_|CreatureError::MotionEnvironment),
            |point|self.move_obstacle_at(point).map_err(|_|CreatureError::MotionEnvironment),animations,|record|{
                if word(record,0x49)as u16&0x100!=0 {
                    self.check_mule_cell_entry(floor_cell([word(record,7),word(record,9)])).map_err(|_|CreatureError::UnsupportedCellEntry)?;
                    let choice=creature_world::plan_mule_retry_wait(record,rng)?;rng=choice.rng_after;return Ok(choice.after);
                }
                if record[0x5e]==7 {
                    let raw=reference(record,0x55);let id=word(&raw,0);let health=self.mule_plant_health(id).map_err(|_|CreatureError::MotionEnvironment)?;if raw[2]!=2||health<=0{return Err(CreatureError::UnsupportedMotionState);}
                    self.check_mule_neutral_neighborhood([word(record,7),word(record,9)]).map_err(|_|CreatureError::UnsupportedNeighborhood)?;
                    let l=self.layouts[1];let selector=self.backing[l.file_offset+id as usize*l.packed_stride+0x23]as i8;
                    if selector<0{return Err(CreatureError::UnsupportedMotionState);}
                    let params=self.plants.as_ref().ok_or(CreatureError::MotionEnvironment)?.parameters.slot(selector as usize).map_err(|_|CreatureError::MotionEnvironment)?;
                    let retired=health<=3;
                    if retired&&(params.retirement_delay_u8>=4||self.mule_plant_flags(id).map_err(|_|CreatureError::MotionEnvironment)?&0x88!=0){return Err(CreatureError::UnsupportedMotionState);}
                    let choice=if retired {creature_world::plan_mule_plant_feed_after_retirement(record,params.nutrition_u8,rng)?}else{creature_world::plan_mule_plant_feed(record,params.nutrition_u8,rng)?};rng=choice.rng_after;fed_plant=Some(id);let full=record[0x61]&0x40!=0&&choice.after[0x61]&0x40==0;pending_audio.push(if full{0x1b}else{0x12});if retired{pending_plant_retirements.push(id as u16);}else if choice.after[0x5e]==8{release_plant=Some(id);}return Ok(choice.after);
                }
                if record[0x5e]==1 {
                    let raw=reference(record,0x55);if raw[2]!=2||self.mule_plant_flags(word(&raw,0)).map_err(|_|CreatureError::MotionEnvironment)?&9!=0{return Err(CreatureError::UnsupportedMotionState);}
                    let mut after=*record;after[0x3f..0x41].copy_from_slice(&577i16.to_le_bytes());after[0x5e]=7;after[0x6c..0x6e].copy_from_slice(&32767i16.to_le_bytes());after[0x72..0x76].copy_from_slice(&record[7..11]);after[0x60]|=8;after[0x60]&=0xbf;reserve_plant=Some(word(&raw,0));return Ok(after);
                }
                let mut record=*record;if record[0x5e]==0 {record[0x5e]=8;}
                self.check_mule_neutral_neighborhood([word(&record,7),word(&record,9)]).map_err(|_|CreatureError::UnsupportedNeighborhood)?;
                let target=self.mule_neutral_food_target(&record,rng).map_err(|_|CreatureError::UnsupportedNeighborhood)?;
                let choice=if let Some((id,point))=target {creature_world::plan_mule_plant_acquisition(&record,id,point,rng)?}else{creature_world::plan_neutral_mule_rechoice(&record,rng,trig,|point|self.initial_mule_accepts(point).map_err(|_|CreatureError::MotionEnvironment))?};
                rng=choice.rng_after;Ok(choice.after)
            },admit_water)?;
        // 5B260's type1/state8 cell-entry predicate only remains false if
        // this is the sole type1 and no code11 type5 occupies the new cell.
        if plan.old_cell!=plan.physics_cell {self.check_mule_cell_entry(plan.physics_cell)?;}
        if plan.physics_cell!=plan.new_cell {self.check_mule_cell_entry(plan.new_cell)?;}
        let mut staged=self.clone();
        if plan.old_cell!=plan.physics_cell {staged.unlink_mule(plan.old_cell)?;}
        staged.creatures.as_mut().unwrap().replace_record_checked(0,&before,&plan.after)?;
        if plan.old_cell!=plan.physics_cell {staged.insert_mule_head(plan.physics_cell)?;}
        if plan.physics_cell!=plan.new_cell {staged.unlink_mule(plan.physics_cell)?;staged.insert_mule_head(plan.new_cell)?;}
        if let Some(id)=reserve_plant {let l=staged.layouts[1];let at=l.file_offset+id as usize*l.packed_stride+0x26;staged.backing[at]|=1;}
        if let Some(id)=release_plant {let l=staged.layouts[1];let at=l.file_offset+id as usize*l.packed_stride+0x26;staged.backing[at]&=0xfe;}
        if let Some(id)=fed_plant {let l=staged.layouts[1];let at=l.file_offset+id as usize*l.packed_stride+0x24;let old=word(&staged.backing,at);staged.backing[at..at+2].copy_from_slice(&old.wrapping_sub(3).to_le_bytes());}
        for id in &pending_plant_retirements {staged.retire_mule_food_plant(*id)?;}
        if let Some(request)=water_effect {
            if plan.physics_cell!=plan.new_cell {return Err(CreatureError::MotionEnvironment.into());}
            if request.code!=0x30 {return Err(CreatureError::MotionEnvironment.into());}
            if let Some((_slot,events))=staged.create_water_splash(request.position,request.physical_height)? {
                for event in events {if let crate::landing::LandingEvent::PlayAudio(index)=event {pending_audio.push(index.channel);}}
            }
        }
        staged.rng=rng;staged.validate_links()?;let mut result=plan;result.after=*staged.creatures.as_ref().unwrap().record(0)?;*self=staged;Ok(MuleCallbackReceipt{motion:result,pending_audio,pending_plant_retirements,pending_portrait_refresh:before[0x5e]==8&&prefix[0x5e]==0})
    }

    fn mule_route_prefix(&self,before:&CreatureRecord,frame:u32,physics:&PhysicsTables,trig:&TrigTables,rng:&mut BlueprintRng)->Result<(CreatureRecord,[i16;2]),ActorWorldError> {
        let own=[0,0,1];let current=[word(before,7),word(before,9)];let final_target=[word(before,11),word(before,13)];
        let mut a=*before;a[0x49]&=0x9f;
        if a[0x49]&3==3||a[0x4f]!=0||a[0x52]!=0||a[0x4a]&0x04!=0{return Err(CreatureError::UnsupportedMotionState.into());}
        if a[0x5e]<24&&a[0x49]&0x10==0&&frame&31==0&&!self.common_move_point_accepted(own,current)? {
            if let Some(point)=creature_world::find_common_idle_relocation(current,|point|self.common_move_point_accepted(own,point).map_err(|_|CreatureError::MotionEnvironment))? {a=creature_world::plan_mule_idle_relocation(&a,point)?;a[0x49]&=0xfe;}
        }
        let mut working=if a[0x49]&3!=0 {[word(&a,0x45),word(&a,0x47)]}else{final_target};
        let mut immediate_heading=false;
        if a[0x49]&0x10!=0&&a[0x49]&0x0c!=0 {
            if !self.common_move_point_accepted(own,final_target)? {return Err(CreatureError::TargetSearchRequired.into());}
            let from=if a[0x49]&3!=0 {working}else{current};
            let ray=self.mule_ray(from,final_target,physics,trig)?;
            if ray.clear {
                if a[0x49]&3!=0 {let flags=word(&a,0x49)as u16&0xeffc;a[0x49..0x4b].copy_from_slice(&flags.to_le_bytes());a[0x16]&=0x7f;immediate_heading=true;}
                a[0x49]&=0xf3;working=final_target;
            }else{
                if a[0x49]&3!=0 {
                    if a[0x49]&8!=0 {
                        let result=self.mule_waypoint(from,final_target,a[0x49]&1!=0,120,physics,trig)?;
                        working=result.point.unwrap_or(from);
                        if result.cost<120 {immediate_heading=true;}
                        else {a[0x54]=a[0x54].wrapping_sub(1);if a[0x54]==0 {let flags=(word(&a,0x49)as u16&0xeebc)|0x140;a[0x49..0x4b].copy_from_slice(&flags.to_le_bytes());a[0x16]&=0x7f;}}
                    }
                }else{
                    let blocked=ray.failure_point.ok_or(CreatureError::MotionEnvironment)?;
                    let first=self.mule_waypoint(blocked,final_target,false,120,physics,trig)?;
                    let second=self.mule_waypoint(blocked,final_target,true,if first.cost<120{first.cost+1}else{120},physics,trig)?;
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
        if a[0x49]&0x10!=0&&(immediate_heading||frame&1!=0){
            let dx=working[0].wrapping_sub(current[0]).wrapping_mul(2);let dy=working[1].wrapping_sub(current[1]).wrapping_mul(2);
            let angle=crate::actor_motion::angle(i32::from(dx),i32::from(dy),physics.angles());a[0x4b..0x4d].copy_from_slice(&angle.to_le_bytes());
        }
        a=creature_world::plan_common_ground_steering(&a,trig)?;Ok((a,working))
    }
    fn mule_ray(&self,from:[i16;2],to:[i16;2],physics:&PhysicsTables,trig:&TrigTables)->Result<crate::actor_motion::ActorStraightRay,ActorWorldError> {
        Ok(crate::actor_motion::actor_straight_ray::<CreatureError,_>(from,to,physics.angles(),trig.sine().try_into().unwrap(),trig.cosine().try_into().unwrap(),
            |point|self.common_move_point_accepted([0,0,1],point).map_err(|_|CreatureError::MotionEnvironment))?)
    }
    fn mule_waypoint(&self,start:[i16;2],target:[i16;2],side:bool,limit:u8,physics:&PhysicsTables,trig:&TrigTables)->Result<creature_world::CommonWaypoint,ActorWorldError> {
        let current=self.creatures.as_ref().unwrap().record(0)?;
        Ok(creature_world::plan_common_waypoint([word(current,7),word(current,9)],start,target,side,limit,physics,
            |point|self.common_move_point_accepted([0,0,1],point).map_err(|_|CreatureError::MotionEnvironment),
            |from,to|self.common_straight_move_ray([0,0,1],from,to,physics,trig).map_err(|_|CreatureError::MotionEnvironment))?)
    }

    /// Actual74D90 null-obstacle point admission for kind1 Mule or an owned
    /// kind3 actor. The source queries obstacle before terrain/blocker.
    pub fn common_move_point_accepted(&self,raw:[u8;3],point:[i16;2])->Result<bool,ActorWorldError> {
        self.entity(raw)?;
        if self.move_obstacle_at(point)? {return Ok(false);}
        match raw[2] {
            1 if raw==[0,0,1]=>self.initial_mule_accepts(point),
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
        let target=[word(record,11),word(record,13)];let distance=creature_world::common_distance_squared(self.map(),point,target)?;
        Ok(creature_world::plan_common_contact_speed(CommonSpeedInput{speed:word(record,0x41),desired_speed:desired,
            width:word(record,0x2b)as u16,flags_4a:record[0x4a],free_flight:false,ground_height:i32::from_le_bytes(record[0x11..0x15].try_into().unwrap()),
            previous_height_low:previous,surface_term:normal[2],distance_squared:distance})?)
    }

    fn check_mule_cell_entry(&self,cell:usize)->Result<(),ActorWorldError> {
        let mut raw=reference(self.map(),cell*14+6);
        while raw[2]!=0 {if raw[2]==1&&raw!=[0,0,1] {return Err(CreatureError::UnsupportedCellEntry.into());}
            if raw[2]==5 {let pool=self.effects.as_ref().ok_or(CreatureError::UnsupportedCellEntry)?;if pool.record(word(&raw,0)as usize)?.get(0x45)==Some(&0x11){return Err(CreatureError::UnsupportedCellEntry.into());}}
            raw=reference(self.entity(raw)?,3);
        }Ok(())
    }
    fn check_mule_neutral_neighborhood(&self,point:[i16;2])->Result<(),ActorWorldError> {
        let cell=floor_cell(point);self.check_mule_cell_entry(cell)?;
        let(cx,cy)=((cell%128)as i32,(cell/128)as i32);
        for dy in -4..=4 {for dx in -4..=4 {
            let index=((cy+dy)&127)as usize*128+((cx+dx)&127)as usize;let mut raw=reference(self.map(),index*14+6);
            while raw[2]!=0 {let entity=self.entity(raw)?;let ignored_effect=if raw[2]==5{self.effects.as_ref().ok_or(CreatureError::UnsupportedNeighborhood)?.record(word(&raw,0)as usize)?.get(0x45)!=Some(&0x11)}else{false};if raw!=[0,0,1]&&!ignored_effect&&(raw[2]!=2||entity[24]!=4){return Err(CreatureError::UnsupportedNeighborhood.into());}raw=reference(entity,3);}
        }}Ok(())
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
        if word(record,0x68)>word(record,0x62)>>1&&record[0x61]&0x40==0 {return Ok(None);}
        let first=rng.seed.wrapping_mul(0xe62d);let second=first.wrapping_mul(0xe62d);
        let sx=if (first>>8)&1==0{256i16}else{-256};let sy=if (second>>8)&1==0{256i16}else{-256};
        let point=[word(record,7),word(record,9)];let mut best=None;let mut score=i32::MIN+1;
        for x in -4i16..=4 {for y in -4i16..=4 {
            let sample=[point[0].wrapping_add(x*sx),point[1].wrapping_add(y*sy)];let d=genewars_core::cd_candidate::distance_squared_wrapping_15(point,sample);
            if d>1024*1024 {continue;}
            let value=1024-genewars_core::cd_candidate::integer_sqrt(d)as i32;let cell=floor_cell(sample);let mut raw=reference(self.map(),cell*14+6);
            while raw[2]!=0 {let entity=self.entity(raw)?;
                if raw[2]==5 {if self.effects.as_ref().ok_or(CreatureError::UnsupportedNeighborhood)?.record(word(&raw,0)as usize)?.get(0x45)==Some(&0x11){return Err(CreatureError::UnsupportedNeighborhood.into());}raw=reference(entity,3);continue;}
                if raw!=[0,0,1] {if raw[2]!=2||entity[0x18]!=4 {return Err(CreatureError::UnsupportedNeighborhood.into());}
                    if self.mule_plant_flags(word(&raw,0))?&9==0&&self.mule_plant_health(word(&raw,0))?>0&&value>score {score=value;best=Some((word(&raw,0),[word(entity,7),word(entity,9)]));}}
                raw=reference(entity,3);
            }
        }}Ok(best)
    }
    fn creature_replace_next(&mut self,raw:[u8;3],next:[u8;3])->Result<(),ActorWorldError> {
        self.entity(raw)?;let id=word(&raw,0)as usize;
        match raw[2] {
            1=>{let pool=self.creatures.as_mut().ok_or(CreatureError::UnsupportedMotionState)?;let before=*pool.record(id)?;let mut after=before;after[3..6].copy_from_slice(&next);pool.replace_record_checked(id,&before,&after)?;},
            3=>{let before=*self.actors.record(id)?;let mut after=before;after[3..6].copy_from_slice(&next);self.actors.replace_record_checked(id,&before,&after)?;},
            5 if self.effects.is_some()=>{let pool=self.effects.as_mut().unwrap();let before=*pool.record(id)?;let mut after=before;after[3..6].copy_from_slice(&next);pool.replace_record_checked(id,&before,&after)?;},
            _=>{let l=self.layouts[usize::from(raw[2]-1)];let at=l.file_offset+id*l.packed_stride+3;self.backing[at..at+3].copy_from_slice(&next);},
        }Ok(())
    }
    fn unlink_mule(&mut self,cell:usize)->Result<(),ActorWorldError> {
        let raw=[0,0,1];let record=self.entity(raw)?;let previous=reference(record,0);let next=reference(record,3);
        if previous[2]==0 {if word(&previous,0)as usize!=cell||reference(self.map(),cell*14+6)!=raw{return Err(ActorWorldError::StaleLink);}
            if next[2]!=0 {self.backing[MAP_START+cell*14+6..MAP_START+cell*14+9].copy_from_slice(&next);}else{self.backing[MAP_START+cell*14+8]=0;}
        }else{if reference(self.entity(previous)?,3)!=raw{return Err(ActorWorldError::StaleLink);}self.creature_replace_next(previous,next)?;}
        if next[2]!=0 {if reference(self.entity(next)?,0)!=raw{return Err(ActorWorldError::StaleLink);}self.replace_previous(next,previous)?;}Ok(())
    }
    fn insert_mule_head(&mut self,cell:usize)->Result<(),ActorWorldError> {
        let before=*self.creatures.as_ref().unwrap().record(0)?;let head=reference(self.map(),cell*14+6);let mut after=before;
        after[..2].copy_from_slice(&(cell as u16).to_le_bytes());after[2]=0;
        if head[2]!=0 {self.entity(head)?;after[3..6].copy_from_slice(&head);self.replace_previous(head,[0,0,1])?;}else{after[5]=0;}
        self.creatures.as_mut().unwrap().replace_record_checked(0,&before,&after)?;
        self.backing[MAP_START+cell*14+6..MAP_START+cell*14+9].copy_from_slice(&[0,0,1]);Ok(())
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
    fn initial_mule_accepts(&self,point:[i16;2])->Result<bool,ActorWorldError> {
        let class=self.terrain_class_at(point)?;
        if 1u8 & (1u8<<class)!=0 {return Ok(false);}
        let cell=floor_cell(point);
        //8B18E..8B242 exempts the entire selected plant cell, before
        //ordinary blocker traversal, for state1/7 or5F1.
        let own=self.creatures.as_ref().ok_or(CreatureError::MotionEnvironment)?.record(0)?;
        if (own[0x5f]==1||matches!(own[0x5e],1|7))&&own[0x57]==2 {
            let target=self.entity(reference(own,0x55))?;
            if floor_cell([word(target,7),word(target,9)])==cell {return Ok(true);}
        }
        let mut raw=reference(self.map(),cell*14+6);let mut previous=[cell as u8,(cell>>8)as u8,0];let mut seen=HashSet::new();let mut weight=0;
        while raw[2]!=0 {
            if !seen.insert(raw) {return Err(ActorWorldError::DuplicateLink(raw));}
            let entity=self.entity(raw)?;
            if reference(entity,0)!=previous {return Err(ActorWorldError::BrokenPrevious(raw));}
            if raw!=[0,0,1] {match raw[2] {
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
