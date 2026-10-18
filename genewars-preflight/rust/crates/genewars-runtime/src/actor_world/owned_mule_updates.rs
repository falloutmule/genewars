//! Bounded owned Mule callback transaction; separate from neutral admission.
use super::*;
use crate::creature_world::{self,MuleStepContext,MuleAnimations,CreatureAnimationBinding};
use crate::physics_tables::PhysicsTables;
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct OwnedMuleReceipt {pub motion:creature_world::CommonPhysicsPlan<124>,pub pending_reveal_vertices:Vec<[i16;2]>,pub pending_audio:Vec<u8>}
impl ActorWorld {
 pub fn step_owned_mule_slot(&mut self,slot:u16,selected:u8,global_flags:u16,context:MuleStepContext,physics:&PhysicsTables,trig:&TrigTables,animations:&MuleAnimations)->Result<OwnedMuleReceipt,ActorWorldError>{
  #[cfg(any(debug_assertions, feature = "callback-captures"))] self.capture_numeric_callback(usize::from(slot),context.frame_counter,"owned-before");
  let result=self.step_owned_mule_slot_inner(slot,selected,global_flags,context,physics,trig,animations);
  #[cfg(any(debug_assertions, feature = "callback-captures"))] match &result {Ok(_)=>self.capture_numeric_callback(usize::from(slot),context.frame_counter,"owned-after"),Err(_)=>self.capture_numeric_callback(usize::from(slot),context.frame_counter,"blocked-before-owned")};
  result
 }
 fn step_owned_mule_slot_inner(&mut self,slot:u16,selected:u8,global_flags:u16,context:MuleStepContext,physics:&PhysicsTables,trig:&TrigTables,animations:&MuleAnimations)->Result<OwnedMuleReceipt,ActorWorldError>{
  self.validate_links()?;if selected>=4||global_flags&1!=0{return Err(CreatureError::UnsupportedNeighborhood.into());}
  let slot=usize::from(slot);let pool=self.creatures.as_ref().ok_or(CreatureError::UnsupportedMotionState)?;
  if pool.binding(slot)?!=Some(CreatureAnimationBinding::MuleSelector4){return Err(CreatureError::UnsupportedMotionState.into());}
  let before=*pool.record(slot)?;if word(&before,15)!=slot as i16{return Err(CreatureError::UnsupportedMotionState.into());}
  let hostile_enemy=self.owned_mule_live_hostile_target(&before)?;
  if before[0x61]&1==0&&creature_world::mule_callback_phase(&before,context.frame_counter)&1==0 {
   //5ee25/5ee32 parity gate: 61 bit0 clear and even phase select the5ee38 light
   //tail. 5ee1f/6FB60 runs with the full-path-only association admission
   //skipped, then5EB30/5E640/7AC00 through the SAME admitted body and the SAME
   //staged cell-entry closure the full path uses. No full state machine, no AI
   //choice, no reveal, no6C decrement.
   let mut rng=self.rng;
   let(prefix,working_target)=self.mule_route_prefix(&before,context.frame_counter,physics,trig,&mut rng,true)?;
   let own=[before[15],before[16],1];
   let query_world=std::cell::RefCell::new(self.clone());let rng_state=std::cell::RefCell::new(rng);
   let links_moved=std::cell::Cell::new(false);let mut pending_audio=Vec::new();
   let(plan,water_effect)=creature_world::plan_mule_admitted_body_with_postphysics(&before,&prefix,working_target,self.map(),context,physics,trig,
    |record,normal,previous|query_world.borrow().mule_contact_speed(record,normal,previous).map_err(|_|CreatureError::MotionEnvironment),
    |point|query_world.borrow().move_obstacle_at(point).map_err(|_|CreatureError::MotionEnvironment),animations,
    //5EB30 choice callback never runs on even source parity.
    |_|Err(CreatureError::UnsupportedMotionState),true,|record,old_cell,new_cell|{
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
      if phase&1!=0 {if phase&3==3&&32767-i32::from(word(record,0x6c))>256{return Err(CreatureError::UnsupportedMotionState);}}
     }
     Ok(after)
    },hostile_enemy.is_some())?;
   let rng=rng_state.into_inner();let mut staged=query_world.into_inner();
   if plan.old_cell!=plan.physics_cell&&!links_moved.get() {self.check_mule_cell_entry(own,plan.physics_cell)?;staged.unlink_mule_slot(slot,plan.old_cell)?;}
   if plan.physics_cell!=plan.new_cell {staged.check_mule_cell_entry(own,plan.new_cell)?;}
   let prior=*staged.creatures.as_ref().unwrap().record(slot)?;
   staged.creatures.as_mut().unwrap().replace_record_checked(slot,&prior,&plan.after)?;
   if plan.old_cell!=plan.physics_cell&&!links_moved.get() {staged.insert_mule_head(slot,plan.physics_cell)?;}
   if plan.physics_cell!=plan.new_cell {staged.unlink_mule_slot(slot,plan.physics_cell)?;staged.insert_mule_head(slot,plan.new_cell)?;}
   if let Some(request)=water_effect {
    if plan.physics_cell!=plan.new_cell {return Err(CreatureError::MotionEnvironment.into());}
    match request.code {
     0x30=>if let Some((_slot,events))=staged.create_water_splash(request.position,request.physical_height)? {
      for event in events {if let crate::landing::LandingEvent::PlayAudio(index)=event {pending_audio.push(index.channel);}}
     },
     0x15=>{staged.create_water_effect(0x15,0,request.position,request.physical_height,request.angles,trig)?;},
     _=>return Err(CreatureError::MotionEnvironment.into()),
    }
   }
   staged.rng=rng;staged.validate_links()?;
   let mut result=plan;result.after=*staged.creatures.as_ref().unwrap().record(slot)?;*self=staged;
   return Ok(OwnedMuleReceipt{motion:result,pending_reveal_vertices:Vec::new(),pending_audio});
  }
  let command_retained=before[0x5e]==8&&before[0x61]==0x80&&before[0x60]==0x18&&before[0x78]==7
   &&before[0x52]==1&&before[0x57]==0&&before[0x5a]==1;
  //5968 measured state-1 pursuit continuation over a DEGRADED target: the
  //enemy (neutral Mule slot 5) reached state 8 (0x60=0x88, 0x61=0x40, 0x78=0,
  //0x55-ref back at the owned mule) while the owned mule keeps pursuit
  //(0x5e=1, 0x60=0xc8, 0x61=0, 0x78=7, 0x52/0x57/0x5a=1, 0x6c=0x7ffe). The
  //source full path 0x5ed20->0x5ed3b(JZ 0x5ee1c)->0x6fb60 virtual94 movement
  //prefix->0x5ee25..0x5ee32 parity gate (odd) ->0x5eed5..0x5ef1f->0x5ef22
  //CALL 0x5e0b0->0x5ef32 CALL 0x5eb30->0x5ef37..0x5ef40 (word(0x6c) fresh side)
  //->0x5ef49->0x6a22b..0x6a2dd->0x65743..0x65de6 (state-1 movement
  //continuation)->tail 0x6a290..0x6a53e (0x6a4a0 DEC word(0x6c))->0x7ac00
  //epilogue (2542 instructions, 0 draws) NEVER reads the enemy record
  //(private/review/attempt28-5968-probe-2a44a5da; oracle output
  //owned-mule-captured-width/22da7a68d74f407b80611ca51ad602d0). The enemy
  //state is admission-only on this path, so the pursuit admission stands
  //without target liveness; everything outside this shape keeps refusing.
  let degraded_pursuit=before[0x5e]==1&&matches!(before[0x60],0x48|0xc8)&&before[0x61]==0
   &&before[0x78]==7&&before[0x4f]==0&&before[0x5d]==0&&before[0x52]==1&&before[0x57]==1&&before[0x5a]==1
   &&reference(&before,0x50)==reference(&before,0x55)&&reference(&before,0x58)==reference(&before,0x55)
   &&before[0x76]<=90&&word(&before,0x49)as u16&0x100==0;
  if !command_retained&&!degraded_pursuit&&((matches!(before[0x5e],1|21)||before[0x5e]==8&&before[0x52]!=0))&&hostile_enemy.is_none(){return Err(CreatureError::UnsupportedMotionState.into());}
  let mut rng=self.rng;let(prefix,target)=self.mule_route_prefix(&before,context.frame_counter,physics,trig,&mut rng,false)?;
  let stage=std::cell::RefCell::new(self.clone());let random=std::cell::RefCell::new(rng);let reveals=std::cell::RefCell::new(Vec::new());let audio=std::cell::RefCell::new(Vec::new());
  let(plan,_water)=crate::owned_mule_callback::plan_owned_after_prefix(&before,&prefix,target,self.map(),context,physics,trig,
   |r,n,p|stage.borrow().mule_contact_speed(r,n,p).map_err(|_|CreatureError::MotionEnvironment),
   |record| {
    let world=stage.borrow();let point=[word(record,7),word(record,9)];let height=i32::from_le_bytes(record[39..43].try_into().unwrap());
    match world.airborne_building_at(point,height).map_err(|_|CreatureError::MotionEnvironment)? {
     None=>Ok(*record),Some(building)=>crate::airborne_building_collision::plan_collision(record,[word(&building,7),word(&building,9)],i32::from_le_bytes(building[17..21].try_into().unwrap()),physics,trig)
        .map(|plan|plan.after).map_err(|_|CreatureError::MotionEnvironment)
    }
   },animations,
   |record|{
    if record[0x5e]==20&&word(record,0x6c)==0&&record[0x60]==0x18&&record[0x61]==0x80 {
     //5946 measured owned state-20 EXPIRED-timer completion at5F4C4/6029A: the
     //jump-table dispatch (0x65384 entry 0x14 -> 5F4C4) runs the completion
     //block 0005f532-0005f569 (0005f564 CALL 0x81c60 -> 0005f5bc JNZ 0x6029a ->
     //000602a2 MOV byte[0x5e],0x8; 00060291 MOV word[0x6c],0) and clears the
     //retained pair target57 (measured 0x57 1->0) before the shared tail
     //0006a290-0006a53e selects animation 0x19 0x10->0x05 via 5E640 (0006a4a0
     //DEC word(0x6c) skipped: 0006a49b JLE). 0x60/0x61 keep the explicit move
     //command shape; no RNG, no reattack. The first triple 0x50..0x53 is
     //admission-only and is NOT pinned here (measured mismatch 05 00 00).
     if record[24]!=0||record[0x78]!=7||record[0x7a]!=4||record[0x4f]!=0||record[0x5d]!=0
      ||record[0x52]!=0||record[0x57]!=1||record[0x5a]!=1
      ||record[0x58..0x5b]!=record[0x55..0x58]
      ||i16::from_le_bytes([record[0x6a],record[0x6b]])<=0{return Err(CreatureError::UnsupportedMotionState);}
     let mut after=*record;after[0x5e]=8;after[0x57]=0;return Ok(after);
    }
    if record[0x5e]==20&&word(record,0x6c)==0 {
     //5922 measured owned hit-reaction completion at5E0B0/5CC60: the expired
     //0x6c timer returns the mule to state8 with the explicit move command
 //shape (0x61=0x80,0x60=0x18) and clears the retained pair target57.
     //No RNG, no target reattack, no animation writes here.
     if record[24]!=0||record[0x60]!=8||record[0x61]!=0||record[0x78]!=7||record[0x52]!=1||record[0x57]!=1||record[0x5a]!=1||record[0x5d]!=0
      ||record[0x50..0x53]!=record[0x55..0x58]||record[0x58..0x5b]!=record[0x55..0x58]{return Err(CreatureError::UnsupportedMotionState);}
     let mut after=*record;after[0x5e]=8;after[0x60]=0x18;after[0x61]=0x80;after[0x57]=0;return Ok(after);
    }
    if let Some(enemy)=hostile_enemy {
     if record[0x5e]==8&&word(record,0x6c)==0&&record[0x60]==0x88&&record[0x61]==0&&record[0x78]==7
      &&record[0x52]==1&&record[0x57]==1&&record[0x5a]==1&&record[0x5d]==0
      &&record[0x50..0x53]==record[0x55..0x58]&&record[0x58..0x5b]==record[0x55..0x58]{
      //5940 measured state8 attack-ready resume: 624B8 dispatches the retained
      //pair straight back into state21 through 5B580/5CC60 (5 draws, enemy
      //untouched; see mule_owned_reattack_resume for the measured deltas).
      let plan=crate::mule_owned_reattack_resume::plan_resume(record,&enemy,*random.borrow(),physics).map_err(|_|CreatureError::UnsupportedMotionState)?;
      *random.borrow_mut()=plan.rng_after;return Ok(plan.after);
     }
     if record[0x5e]==21{if word(record,0x6c)<=0{return stage.borrow_mut().owned_mule_completed_attack(record,&enemy,&mut random.borrow_mut(),physics,&mut audio.borrow_mut());}let mut after=*record;if creature_world::mule_callback_phase(record,context.frame_counter)&3==3{
      let dx=word(&enemy,7).wrapping_mul(2).wrapping_sub(word(record,7).wrapping_mul(2));let dy=word(&enemy,9).wrapping_mul(2).wrapping_sub(word(record,9).wrapping_mul(2));
      after[0x4b..0x4d].copy_from_slice(&crate::actor_motion::angle(i32::from(dx),i32::from(dy),physics.angles()).to_le_bytes());
     }return Ok(after);}
     if record[0x5e]==1 {
     let xy=genewars_core::cd_candidate::distance_squared_wrapping_15([word(record,7),word(record,9)],[word(&enemy,7),word(&enemy,9)])as i32;
     let z=(i32::from_le_bytes(enemy[39..43].try_into().unwrap()).wrapping_sub(i32::from_le_bytes(record[39..43].try_into().unwrap())))>>3;
     if xy.wrapping_add(z.wrapping_mul(z))<=16384 {
      match crate::mule_hostile_contact::plan_false_contact(record,&enemy,physics){
       Ok(after)=>return Ok(after),Err(crate::mule_hostile_contact::HostileContactError::UnsupportedAttack)=>{
        let entry=crate::mule_hostile_attack_entry::plan_entry(record,&enemy,*random.borrow(),physics).map_err(|_|CreatureError::UnsupportedMotionState)?;
        *random.borrow_mut()=entry.rng_after;if let Some(channel)=entry.pending_audio{audio.borrow_mut().push(channel);}let mut after=entry.after;after[0x60]&=0xbf;return Ok(after);
       },Err(_)=>return Err(CreatureError::UnsupportedMotionState),
      }
     }return Ok(*record);
    }}
    //5968 measured degraded-target pursuit continuation: with a live enemy the
    //state-1 arm above returns the record unchanged beyond contact distance;
    //the measured source path (5ef32 CALL 5eb30 -> 5eb36..5ec98 -> 5ef37
    //JLE NOT taken, word(0x6c)=0x7ffe -> 5ef49 -> movement continuation ->
    //tail) likewise runs NO 624B8 AI and NO 6B700 neighborhood scan - the
    //trace never reaches 624B8 (private/review/attempt28-5968-probe-2a44a5da).
    //Pin exactly that arm for the degraded shape; expired timers (word(0x6c)==0)
    //and other kinds keep refusing below.
    if hostile_enemy.is_none()&&record[0x5e]==1&&matches!(record[0x60],0x48|0xc8)&&word(record,0x6c)>0{return Ok(*record);}
    //6008 measured arrival side of the SAME degraded-target pursuit: the
    //source5EB30 body ran its zeroing block (0005eb9b AND byte[0x60],0x7f:
    //0xc8->0x48; 0005ebd9 MOV word[0x6c],0), then 0005ef3a/0005ef3e/0005ef40
    //reads the FRESH word(0x6c) and JLEs to 00065384, whose state dispatch
    //(0006538a CMP al,0x18 / 00065397 JMP [eax*4+0x5eca0]) takes entry 1 ->
    //0005ef46 - the same continuation as the fresh side above; it runs NO
    //624B8 AI and NO 6B700 walk (trace private/review/owned-mule-captured-
    //width/b3ac36ab78474896ab7e200a37250f8b, 2818 instructions, 0 draws).
    //The former word(record,0x6c)>0 freshness test on the POST-prefix record
    //was port-authored: the prefix itself produces the zeroed timer. A pre-
    //prefix expired timer (word(&before,0x6c)==0) and other 0x60 sides keep
    //refusing below.
    if hostile_enemy.is_none()&&record[0x5e]==1&&record[0x60]==0x48&&word(record,0x6c)==0&&word(&before,0x6c)>0{
     //65d80..65dce continuation near-target block, degraded state-1 arrival:
     //the record has reached its own route waypoint (word 0x0b/0x0d == the
     //position the continuation context point carries), so 73970's doubled
     //delta is zero and AE028 yields yaw0; 7a6c0's shortest delta against
     //word(0x39) exceeds 0x100 (measured 551) so the FALSE side runs and
     //writes word(0x4b)=0 (measured oracle 0x4b 1024->0). The aligned side
     //(virtual38/5C840, 65da7..65db5) is unmeasured and refuses.
     //F35 (attempt35) moved the block into the SHARED helper
     //mule_owned_expired_replan::near_target so the B11 ring-gate SHORT side
     //(65c7f JMP 65d4d, measured at frame 6060) and this arrival side run ONE
     //implementation; the refusal conditions and the word(0x4b)=0 write are
     //unchanged from the F17 arm that R30 proved byte-identical.
     return crate::mule_owned_expired_replan::near_target(record);
    }
    //F21/attempt34 B11 CONVERSION of the R31 negative row to POSITIVE in this
    //same change set, per the family policy's "source-proved and intentionally
    //expanded" condition. The PRE-prefix EXPIRED-timer side of the same B9
    //degraded-pursuit continuation (word(&before,0x6c)==0 ALREADY before the
    //callback; fixture attempt31-6010-measure, frame 6010): the source reaches
    //the 65c60 ring gate on the state-1 continuation 65743..65de6 and takes
    //the LONG side ((word(0xf)+[0x11877e])&3)==3 = (6010+1)&3==3, the 65cb2
    //timer gate (0x7fff-0 > 0x100), then 65cbe activation write 0x9aa/
    //0x10e4d8, 65cfe draw 1, 65d42 CALL 5cc60 -> 5cca0 zeroing block ->
    //65d58 JNZ 65deb taken, 65deb activation write 0xa03/0x10e4e5 + draw 2,
    //65e68 byte[0x5e]=8, 65e7a CALL [vtable+0x8c]=8b780 (measured AL!=0),
    //65e88 CALL 73c20 (activation writes 0x59c/0x10e6b7 and 0x59d/0x10e6c0,
    //draws 3-4, waypoint dword 0xb replanned from the 0x107ee8/0x1086e8 slope
    //tables), 0x54=0x14, word(0x49)|0x10|0x4 &0xeffc, byte[0x4f]=0 (13eb0
    //+0x4d), byte[0x52]=0 (13eb0 +0x50), 65ed0 CALL 5a990 -> word(0x3f),
    //word(0x6c)=0x7fff, byte[0x60]|=0x80, 65eeb CALL 6bdd0 (species byte
    //0x104a64+2*0x7a = 0x04 != 3 -> 65ef2 JZ 6a290) and the shared tail
    //6a4a0 DEC word(0x6c) -> 0x7ffe. Oracle: source_prepared, 2431
    //instructions, 4 draws, output
    //private/review/owned-mule-captured-width/59dc2afb22b64ab6bf89e1ccb85df79b
    //(slot-1-only deltas match the pinned arm). The machinery lives in
    //mule_owned_expired_replan; every unmeasured side of this block (ring
    //short side, timer-gate short side, draw-1 flag JG side, 8b780 AL==0 side,
    //6bdd0 true side/6c190) stays fail-closed there. The activation-table
    //consumers are a separate unproved family and are NOT guessed.
    if hostile_enemy.is_none()&&record[0x5e]==1&&record[0x60]==0x48&&word(record,0x6c)==0&&word(&before,0x6c)==0{
     return crate::mule_owned_expired_replan::plan_expired_reacquisition(record,&mut random.borrow_mut(),context.frame_counter);
    }
    let world=stage.borrow();let mut r=*record;if r[0x5e]==0{r[0x5e]=8;}

    if let Some(enemy)=world.owned_mule_single_hostile(&r).map_err(|_|CreatureError::UnsupportedNeighborhood)? {
     let hostile=crate::mule_owned_hostile_acquisition::plan_acquisition(&r,&enemy,*random.borrow()).map_err(|_|CreatureError::UnsupportedNeighborhood)?;
     *random.borrow_mut()=hostile.rng_after;return Ok(hostile.after);
    }
    //5924 measured 0x61=0x80 dispatch: source 624B8 runs the ordinary AI

    //without the 6BA47 food-guard neighborhood scan (measured call order

    //5EB30->624B8->6B150). Skip it for the retained explicit-command domain.

    if r[0x61]&0x80==0{world.owned_mule_neighborhood_no_action(&r).map_err(|_|CreatureError::UnsupportedNeighborhood)?;}
    let mut rng=random.borrow_mut();for _ in 0..2{rng.seed=rng.seed.wrapping_mul(0xe62d);rng.ring=(rng.ring+1)&31;rng.draw_count=rng.draw_count.wrapping_add(1);}
    if r[0x61]&0x80!=0{
     return crate::owned_mule_ai::plan_explicit_target(&r,true,|p,_|world.mule_accepts(slot,p).map_err(|_|crate::owned_mule_ai::OwnedIdleError::QueryUnavailable)).map(|p|p.after).map_err(|_|CreatureError::UnsupportedMotionState);
    }
    let current=world.mule_cell_entry_predicate(&r,floor_cell([word(&r,7),word(&r,9)])).map_err(|_|CreatureError::UnsupportedCellEntry)?;
    //63EA8..63EB1 skips the owned idle decision for retained command78.
    //Arrival can clear61bit80 while command7 still reaches common fallback.
    let after=if r[0x78]==0 {
     let ai=crate::owned_mule_ai::plan_idle_without_building_action(&r,*rng,crate::owned_mule_ai::OwnedIdleContext{neighborhood_eligible:true,current_cell_requires_escape:current},|mask,flags,p|world.owned_mule_nearest_building(r[24],mask,flags,p).map_err(|_|crate::owned_mule_ai::OwnedIdleError::QueryUnavailable)).map_err(|_|CreatureError::UnsupportedNeighborhood)?;
     *rng=ai.rng_after;ai.after
    }else if r[0x78]==7 {r}else{return Err(CreatureError::UnsupportedMotionState);};
    let choice=crate::owned_mule_common_choice::plan_choice(&after,*rng,trig,crate::owned_mule_common_choice::ChoiceContext{common_choice_eligible:true,virtual_b0:word(&after,0x49)as u16&0x100!=0,current_cell_requires_escape:current},|p,_|world.mule_accepts(slot,p).map_err(|_|crate::owned_mule_common_choice::ChoiceError::QueryUnavailable)).map_err(|_|CreatureError::TargetSearchRequired)?;
    *rng=choice.rng_after;Ok(choice.after)
   },
   |record,old,new|{
    let mut world=stage.borrow_mut();world.unlink_mule_slot(slot,old).map_err(|_|CreatureError::UnsupportedCellEntry)?;
    let prior=*world.creatures.as_ref().unwrap().record(slot)?;let mut r=*record;r[..6].copy_from_slice(&prior[..6]);world.creatures.as_mut().unwrap().replace_record_checked(slot,&prior,&r)?;world.insert_mule_head(slot,new).map_err(|_|CreatureError::UnsupportedCellEntry)?;
    let linked=*world.creatures.as_ref().unwrap().record(slot)?;let after=world.plan_mule_cell_entry(slot,&linked,new,&mut random.borrow_mut(),trig)?;world.creatures.as_mut().unwrap().replace_record_checked(slot,&linked,&after)?;Ok(after)
   },
   |record,water|{
    let mut world=stage.borrow_mut();let prior=*world.creatures.as_ref().unwrap().record(slot)?;world.creatures.as_mut().unwrap().replace_record_checked(slot,&prior,record)?;
    //6E8xx factories precede6E9AF owned reveal and subsequent AI. The
    //factory can prepend an effect and change this creature's map links.
    if let Some(request)=water {
     world.rng=*random.borrow();
     match request.code {
      0x30=>if let Some((_,events))=world.create_water_splash(request.position,request.physical_height).map_err(|_|CreatureError::MotionEnvironment)? {
       for event in events {if let crate::landing::LandingEvent::PlayAudio(index)=event{audio.borrow_mut().push(index.channel);}}
      },
      0x15=>{world.create_water_effect(0x15,0,request.position,request.physical_height,request.angles,trig).map_err(|_|CreatureError::MotionEnvironment)?;},
      _=>return Err(CreatureError::MotionEnvironment),
     }
     *random.borrow_mut()=world.rng;
    }
    let mut linked=*world.creatures.as_ref().unwrap().record(slot)?;
    if context.frame_counter.wrapping_add(i32::from(word(&linked,15))as u32)&3==3
        &&i32::from_le_bytes(linked[39..43].try_into().unwrap())==0&&linked[0x53]&1!=0 {
     let class=world.terrain_class_at([word(&linked,7),word(&linked,9)]).map_err(|_|CreatureError::MotionEnvironment)?;
     if creature_world::common_water_damage_required(&linked,context.frame_counter,context.water_height,class){
      let damage=crate::mule_self_water_damage::plan(&linked).map_err(|_|CreatureError::MotionEnvironment)?;
      world.creatures.as_mut().unwrap().replace_record_checked(slot,&linked,&damage.after)?;linked=damage.after;
     }
    }
    if let Some(enemy)=hostile_enemy.filter(|_|before[0x5e]==1) {
     let phase=creature_world::mule_callback_phase(&before,context.frame_counter);
     if phase&1!=0 {
      if phase&3==3&&32767-i32::from(word(&linked,0x6c))>256{return Err(CreatureError::UnsupportedMotionState);}
      let delta=[word(&linked,7),word(&linked,9)];let other=[word(&enemy,7),word(&enemy,9)];
      let z=(i32::from_le_bytes(enemy[39..43].try_into().unwrap()).wrapping_sub(i32::from_le_bytes(linked[39..43].try_into().unwrap())))>>3;
      let distance=(genewars_core::cd_candidate::distance_squared_wrapping_15(delta,other)as i32).wrapping_add(z.wrapping_mul(z));
      //Contact disposition runs later at its original AI boundary.
      //5D360 owned phase3 requires the species4 player genetics predicate.
      //Only its actual zero/no-action branch is reconstructed here.
      if distance>16384&&phase&3==3&&(linked[0x61]&4!=0||world.player(0).map_err(|_|CreatureError::UnsupportedMotionState)?.bytes()[0x6cb]!=0){return Err(CreatureError::UnsupportedMotionState);}
     }
    }
    let record=&linked;
    let phase=context.frame_counter.wrapping_add(i32::from(word(record,15))as u32);
    let height=i32::from_le_bytes(record[39..43].try_into().unwrap());let ground=i32::from_le_bytes(record[17..21].try_into().unwrap());
    if record[0x16]&2!=0&&phase&3==3&&record[24]==selected&&(word(record,0x3d)>32||height>ground){let v=world.owned_mule_reveal([word(record,7),word(record,9)]).map_err(|_|CreatureError::MotionEnvironment)?;reveals.borrow_mut().extend(v);}
    Ok(*record)
   })?;
  let mut world=stage.into_inner();if plan.physics_cell!=plan.new_cell{return Err(CreatureError::UnsupportedCellEntry.into());}
  let prior=*world.creatures.as_ref().unwrap().record(slot)?;world.creatures.as_mut().unwrap().replace_record_checked(slot,&prior,&plan.after)?;world.rng=random.into_inner();world.validate_links()?;*self=world;
  Ok(OwnedMuleReceipt{motion:plan,pending_reveal_vertices:reveals.into_inner(),pending_audio:audio.into_inner()})
 }
 /// Source5D700 returns false when every actual roster blueprint category
 ///differs from3. Actor C0 is an opaque port pointer; selector74 identifies the
 ///same player blueprint as original4FF2C. Operator support stays explicit.
 fn owned_attack_no_operator(&self)->Result<(),CreatureError>{
  let player=self.player(0).map_err(|_|CreatureError::UnsupportedNeighborhood)?;let p=player.bytes();let count=usize::from(p[3]);
  if count>5{return Err(CreatureError::UnsupportedNeighborhood);}let mut seen=HashSet::new();
  for &id in &p[4..4+count]{let slot=usize::from(id);if slot>=5||!seen.insert(slot){return Err(CreatureError::UnsupportedNeighborhood);}
   let actor=self.actors.record(slot).map_err(|_|CreatureError::UnsupportedNeighborhood)?;
   if actor[6]!=3||actor[24]!=0||word(actor,15)!=slot as i16||actor[0xc0..0xc4]!=[0;4]{return Err(CreatureError::UnsupportedNeighborhood);}
   let blueprint=player.blueprint(usize::from(actor[0x74])).map_err(|_|CreatureError::UnsupportedNeighborhood)?;
   if blueprint[0x16]==3{return Err(CreatureError::UnsupportedNeighborhood);}
  }Ok(())
 }
 ///60332 actual owned21 completion, one nonlethal hit and immediate reattack.
 ///Every stage is in the existing cloned transaction; late rejection restores
 ///the target, player/backing/map, all RNG and pending presentation requests.
 fn owned_mule_completed_attack(&mut self,record:&CreatureRecord,enemy:&CreatureRecord,rng:&mut BlueprintRng,physics:&PhysicsTables,audio:&mut Vec<u8>)->Result<CreatureRecord,CreatureError>{
  if record[24]!=0||record[0x5e]!=21||record[0x61]!=0||record[0x60]!=0x88||record[0x78]!=7||word(record,0x6c)!=0||word(record,0x68)<0||record[0x76]>90{return Err(CreatureError::UnsupportedMotionState);}
  let xy=genewars_core::cd_candidate::distance_squared_wrapping_15([word(record,7),word(record,9)],[word(enemy,7),word(enemy,9)])as i32;
  let z=(i32::from_le_bytes(enemy[39..43].try_into().unwrap()).wrapping_sub(i32::from_le_bytes(record[39..43].try_into().unwrap())))>>3;
  if (xy.wrapping_add(z.wrapping_mul(z))as u32)>16384{
   //603C0->6152C: a miss keeps references/reserve and waits oneBBD draw.
   let mut waiting=*record;waiting[0x5e]=8;
   let escape=self.mule_cell_entry_predicate(&waiting,floor_cell([word(&waiting,7),word(&waiting,9)])).map_err(|_|CreatureError::UnsupportedCellEntry)?;
   let miss=crate::mule_owned_attack_miss::plan(record,*rng,escape).map_err(|_|CreatureError::UnsupportedMotionState)?;
   *rng=miss.rng_after;return Ok(miss.after);
  }
  if word(record,0x68)==0{return Err(CreatureError::UnsupportedMotionState);}
  self.owned_attack_no_operator()?;
  let mut after=*record;let trait_value=10*(50+50*i32::from(record[0x76])/100)/100;
  let cost=(trait_value>>2).min(i32::from(word(record,0x68)));after[0x68..0x6a].copy_from_slice(&(word(record,0x68)-cost as i16).to_le_bytes());
  let draw=|r:&mut BlueprintRng|{r.seed=r.seed.wrapping_mul(0xe62d);r.ring=r.ring.wrapping_add(1)&31;r.draw_count=r.draw_count.wrapping_add(1);(r.seed>>8)as u8};
  if rng.ring>=32{return Err(CreatureError::UnsupportedMotionState);}let incoming=((trait_value>>1).max(cost*2)+i32::from(draw(rng)%10))as i16; //B66
  let target_id=word(enemy,15)as usize;let prior=*self.creatures.as_ref().unwrap().record(target_id)?;if prior!=*enemy{return Err(CreatureError::UnsupportedMotionState);}
  let damage=crate::mule_nonlethal_hostile_damage::plan(&prior,&after,incoming,*rng).map_err(|_|CreatureError::UnsupportedMotionState)?;
  self.creatures.as_mut().unwrap().replace_record_checked(target_id,&prior,&damage.target_after)?;*rng=damage.rng_after;audio.push(damage.pending_audio);
  if draw(rng)%100>90{return Err(CreatureError::UnsupportedNeighborhood);} //BA4; alternate branch unported.
  let sole=self.owned_mule_single_hostile(&after).map_err(|_|CreatureError::UnsupportedNeighborhood)?.ok_or(CreatureError::UnsupportedNeighborhood)?;
  if word(&sole,15)!=target_id as i16||self.mule_cell_entry_predicate(&after,floor_cell([word(&after,7),word(&after,9)])).map_err(|_|CreatureError::UnsupportedNeighborhood)?{return Err(CreatureError::UnsupportedNeighborhood);}
  let query=crate::mule_owned_attack_reacquire::plan_reacquire(&after,&sole,*rng).map_err(|_|CreatureError::UnsupportedNeighborhood)?;*rng=query.rng_after;
  let entry=crate::mule_hostile_attack_entry::plan_entry(&query.after,&sole,*rng,physics).map_err(|_|CreatureError::UnsupportedMotionState)?;*rng=entry.rng_after;if let Some(channel)=entry.pending_audio{audio.push(channel);}
  after=entry.after;after[0x60]&=0xbf;Ok(after)
 }
 /// Source74270 mask0 query preserving candidate order and current-frame height.
 fn airborne_building_at(&self,point:[i16;2],height:i32)->Result<Option<[u8;67]>,ActorWorldError> {
  crate::building_obstacle::building_at_height(self.map(),point,height,0,|cell| {
   let mut raw=reference(self.map(),cell*14+6);let mut seen=HashSet::new();
   while raw[2]!=0 {
    if !seen.insert(raw){return Err(ActorWorldError::DuplicateLink(raw));}
    let record=self.entity(raw)?;
    if raw[2]==4{return Ok(Some(record.try_into().map_err(|_|ActorWorldError::UnsupportedBuildingQuery(cell))?));}
    raw=reference(record,3);
   }Ok(None)
  },|record| {
   self.building_collision_heights.get(&record[45]).and_then(|animations|animations.get(usize::from(record[25])))
    .and_then(|frames|frames.get(usize::from(record[27]))).copied().ok_or(ActorWorldError::UnsupportedBuildingQuery(floor_cell(point)))
  }).map_err(|error|match error {crate::building_obstacle::BuildingObstacleError::Query(error)=>error,_=>ActorWorldError::UnsupportedBuildingQuery(floor_cell(point))})
 }
 fn owned_mule_nearest_building(&self,owner:u8,mask:u8,flags:u8,at:[i16;2])->Result<Option<u16>,ActorWorldError>{
  if flags!=0x30{return Err(CreatureError::UnsupportedNeighborhood.into());}let p=self.player(owner)?.bytes();if p[0x4a1]!=owner{return Err(CreatureError::UnsupportedNeighborhood.into());}
  let pool=self.buildings.as_ref().ok_or(ActorWorldError::BuildingsNotAdmitted)?;let mut result=None;let mut distance=i32::MAX;
  for slot in usize::from(owner)*64..usize::from(owner+1)*64{let b=pool.record(slot)?;if b[6]==0{continue;}if b[24]!=p[1]&&(u32::from(p[0x6e5])&(1u32<<u32::from(b[24]&31)))==0{continue;}
   if u32::from(mask)&(1u32<<u32::from(b[0x2d]&31))==0||word(b,0x31)as u16&0x141!=1{continue;}
   let d=creature_world::common_distance_squared(self.map(),at,[word(b,7),word(b,9)])?;if d<distance{distance=d;result=Some(slot as u16);}
  }Ok(result)
 }
 /// Completed owned order's sole hostile query, before disposition draws.
 ///6B7C6/6B7EA require player0's neutral-alliance bit clear. Only a reciprocal
 ///live neutral Mule pursuit and source-skipped kind8/non11 effects are admitted;
 ///this proves no friendly count, no food candidates and no extra query RNG.
 fn owned_mule_single_hostile(&self,r:&CreatureRecord)->Result<Option<CreatureRecord>,ActorWorldError>{
  let attack_query=r[0x5e]==21&&r[0x60]==0x88&&word(r,0x6c)==0;
  if r[24]!=0||!(r[0x5e]==8&&r[0x60]==8||attack_query)||r[0x61]!=0||r[0x78]!=7{return Ok(None);}
  let own=[r[15],r[16],1];let cell=floor_cell([word(r,7),word(r,9)]);let(cx,cy)=((cell%128)as i32,(cell/128)as i32);
  let mut hostile=None;let mut unsupported=false;
  for dy in -4..=4{for dx in -4..=4{if dx*dx+dy*dy>16{continue;}let c=(((cy+dy)&127)*128+((cx+dx)&127))as usize;let mut raw=reference(self.map(),c*14+6);
   while raw[2]!=0{let e=self.entity(raw)?;if raw!=own{match raw[2]{
    1=>{let pool=self.creatures.as_ref().ok_or(CreatureError::UnsupportedNeighborhood)?;let other=*pool.record(word(&raw,0)as usize)?;
     //F14 hostile kind-1 classification (scorer 6B700, owned idle context):
     //6B7DF..6B7FE computes the self owner's alliance mask [owner*0x70d+0x6e5],
     //EAX=1<<cand_owner, TEST EDX,EAX; the VALIDATED bytes 75 04 (JNZ 0x6B804;
     //the listing's 0x6B800 render is wrong) skip ALIGNED candidates, so a
     //candidate is hostile only when the mask bit is clear. Kind dispatch
     //6B81E..6B830 -> jump table 6BD70[0]=0x6B9B0; 6B9B0..0x6B9C3 requires
     //state!=0xF; 0x6BA8A..0x6BAF3 requires candidate state<0x1A and
     //[ESI+0x60]&1==0. Liveness (word(0x6a)>0, word(0x49)bit0x400 clear) and
     //the typed MuleSelector4 binding are the caller-proved admission.
     let allied=(u32::from(self.player(r[24])?.bytes()[0x6e5])>>u32::from(other[24]&31))&1!=0;
     if other[24]==4&&other[0x7a]==4&&(other[0x5e]==1||attack_query&&other[0x5e]==20)&&pool.binding(word(&raw,0)as usize)?==Some(CreatureAnimationBinding::MuleSelector4){
      if hostile.is_some(){return Err(CreatureError::UnsupportedNeighborhood.into());}hostile=Some(other);
     }else if allied{}
     else if !attack_query&&other[0x7a]==4&&other[0x5e]<0x1a&&other[0x5e]!=0xf&&other[0x61]&1==0
      &&word(&other,0x6a)>0&&word(&other,0x49)as u16&0x400==0
      &&pool.binding(word(&raw,0)as usize)?==Some(CreatureAnimationBinding::MuleSelector4){
      if hostile.is_some(){return Err(CreatureError::UnsupportedNeighborhood.into());}hostile=Some(other);
     }else{unsupported=true;}
    },
    5=>{let pool=self.effects.as_ref().ok_or(CreatureError::UnsupportedNeighborhood)?;if pool.record(word(&raw,0)as usize)?[69]==0x11||pool.binding(word(&raw,0)as usize)?.is_none(){unsupported=true;}},
    8=>{},_=>unsupported=true,
   }}raw=reference(e,3);}
  }}
  if hostile.is_some()&&(unsupported||self.player(0)?.bytes()[0x6e5]&16!=0){return Err(CreatureError::UnsupportedNeighborhood.into());}
  Ok(hostile)
 }
 /// Typed reciprocal owned task7 pursuit, separate from plant/corpse food.
/// 5940 measured extension: the expired state8 timer (0x6c==0) with the
/// retained pair still selects the live peer through 624B8; the 0x6c>0 arm
/// keeps the proven hold domain and the shared admission guard rejects 0x6c<0.
 pub(super) fn owned_mule_live_hostile_target(&self,r:&CreatureRecord)->Result<Option<CreatureRecord>,ActorWorldError>{
  if r[6]!=1||r[24]!=0||r[0x7a]!=4||!(matches!(r[0x5e],1|21)||r[0x5e]==8&&word(r,0x6c)>=0)||r[0x61]!=0||!(if r[0x5e]==1{matches!(r[0x60],0x48|0xc8)}else{matches!(r[0x60],8|0x88)})||r[0x78]!=7
   ||r[0x76]>90||r[0x4f]!=0||r[0x5d]!=0||r[0x52]!=1||r[0x57]!=1||r[0x5a]!=1||word(r,0x49)as u16&0x100!=0
   ||reference(r,0x50)!=reference(r,0x55)||reference(r,0x58)!=reference(r,0x55){return Ok(None);}
  let pool=self.creatures.as_ref().ok_or(CreatureError::UnsupportedMotionState)?;let id=word(r,0x55)as usize;let target=*pool.record(id)?;
  if target[6]==1&&target[24]==4&&target[0x7a]==4&&matches!(target[0x5e],1|20|21)&&target[0x61]==0x40&&(if target[0x5e]==1{matches!(target[0x60],0x48|0xc8)}else{matches!(target[0x60],8|0x88)})
   &&target[0x78]==0&&word(&target,0x6a)>0&&word(&target,0x49)as u16&0x400==0
   &&reference(&target,0x50)==[r[15],r[16],1]&&reference(&target,0x55)==[r[15],r[16],1]
   &&pool.binding(id)?==Some(CreatureAnimationBinding::MuleSelector4){Ok(Some(target))}else{Ok(None)}
 }
 /// Bounded no-action neighborhood: full reserve, only own Genepods and
 /// neutral plants, same-owner specialists and Genepod/Extractor buildings.
 /// Source6BA47 kinds3/4 bypass food/pair scoring; building-action queries
 /// still run separately. Same-owner water effects also take6BA47 kind>2.
 ///6BD3D->6B72A skips code1A (all effects except11) before classification;
 /// known effects except11 skip independently of ownership; other candidates require the scorer.
 fn owned_mule_neighborhood_no_action(&self,r:&CreatureRecord)->Result<(),ActorWorldError>{
  let food_guard=word(r,0x68)<=word(r,0x62)>>1||r[0x61]&0x40!=0;
  // Completed task7 skips owned idle AI. A hungry query is a proved no-op
  //only when every other candidate follows6BD3D/6B72A's skip branches.
  let completed_command=r[0x5e]==8&&r[0x61]==0&&matches!(r[0x60],8|0x88)&&r[0x78]==7;
  if food_guard&&!completed_command{return Err(CreatureError::UnsupportedNeighborhood.into());}
  let own=[r[15],r[16],1];let point=[word(r,7),word(r,9)];let cell=floor_cell(point);let(cx,cy)=((cell%128)as i32,(cell/128)as i32);
  //8D930 visits sample cells within radius1024, including its boundary.
  for dy in -4..=4{for dx in -4..=4{if dx*dx+dy*dy>16{continue;}let c=(((cy+dy)&127)*128+((cx+dx)&127))as usize;let mut raw=reference(self.map(),c*14+6);
   while raw[2]!=0{let e=self.entity(raw)?;if raw!=own{match raw[2]{2 if !food_guard&&e[24]==4=>{},3 if !food_guard&&e[24]==r[24]=>{},4 if !food_guard&&e[24]==r[24]&&matches!(self.buildings.as_ref().unwrap().record(word(&raw,0)as usize)?[0x2d],0|11)=>{},5 if self.effects.as_ref().unwrap().record(word(&raw,0)as usize)?[69]!=0x11&&self.effects.as_ref().unwrap().binding(word(&raw,0)as usize)?.is_some()=>{},8=>{},_=>return Err(CreatureError::UnsupportedNeighborhood.into())}}raw=reference(e,3);}
  }}Ok(())
 }
 fn owned_mule_reveal(&mut self,at:[i16;2])->Result<Vec<[i16;2]>,ActorWorldError>{
  let mut changed=Vec::new();let start=[at[0].wrapping_sub(1024)&!255,at[1].wrapping_sub(1024)&!255];let end=[(at[0].wrapping_add(1024)&!255).wrapping_add(256),(at[1].wrapping_add(1024)&!255).wrapping_add(256)];
  let mut y=start[1];while y!=end[1]{let mut x=start[0];while x!=end[0]{let point=[x,y];let cell=floor_cell(point);let old=self.map()[cell*14+12];if old<63{let d=creature_world::common_distance_squared(self.map(),at,point)?;let next=if d<768*768{63}else if d<1024*1024{(((1024-genewars_core::cd_candidate::integer_sqrt(d as u32)as i32)*64-64)/256)as u8}else{old};if next>old{self.backing[MAP_START+cell*14+12]=next;self.refresh_terrain_vertex(point,true)?;changed.push(point);}}x=x.wrapping_add(256);}y=y.wrapping_add(256);}Ok(changed)
 }
}
impl ActorWorld {
 pub fn dispatch_owned_mule_move(&mut self,slot:u16,target:[i16;2],context:crate::mule_owned_command::CommandContext)->Result<crate::mule_owned_command::CommandPlan,ActorWorldError>{
  let pool=self.creatures.as_ref().ok_or(CreatureError::UnsupportedMotionState)?;let before=*pool.record(usize::from(slot))?;
  let plan=crate::mule_owned_command::plan_move(&before,target,context).map_err(|_|CreatureError::UnsupportedMotionState)?;
  self.creatures.as_mut().unwrap().replace_record_checked(usize::from(slot),&before,&plan.after)?;Ok(plan)
 }
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn actual_terminal_owned_landing_and_command_fixture(){
  let p=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");let resources=p.join("private/integration/game-runtime");
  let raw=std::fs::read(resources.join("terrain/CONQ001.DAT")).unwrap();let mission=genewars_assets::ConqMissionSource::parse(&raw).unwrap();
  let players=core::array::from_fn(|i|{let p=PlayerState::fresh_campaign(i as u8,[20736,13568],b"fixture\0").unwrap();let mut a=*p.bytes();a[0x4a3..0x4a7].copy_from_slice(&2000i32.to_le_bytes());a[0x62a..0x62e].copy_from_slice(&(50i32<<11).to_le_bytes());PlayerState::from_record(&a).unwrap()});
  let mut w=ActorWorld::from_mission(&mission,&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed:5428,ring:31,draw_count:0}).unwrap();w.admit_effects(&[[0;81];200],[None;200]).unwrap();w.admit_creatures(&[[0;124];300]).unwrap();w.admit_buildings(&[[0;67];256],[None;256]).unwrap();
  let slot=w.create_building(0,0,[19839,12927]).unwrap().unwrap().slot;w.progress_admitted_grade0_building(slot,280,false).unwrap();let b=*w.buildings.as_ref().unwrap().record(slot as usize).unwrap();let mut a=b;a[51]=4;a[54..58].copy_from_slice(&1i32.to_le_bytes());a[58]=0;w.buildings.as_mut().unwrap().replace_record_checked(slot as usize,&b,&a).unwrap();
  let trig=TrigTables::generate_pinned().unwrap();let terminal=w.step_genepod_terminal(slot,0,&trig).unwrap();assert_eq!(terminal.newborn_slots,vec![1]);
  let physics=PhysicsTables::from_mapped_bytes(&std::fs::read(resources.join("physics/angle.bin")).unwrap(),&std::fs::read(resources.join("physics/bounce.bin")).unwrap()).unwrap();let animations=MuleAnimations::from_anb(&std::fs::read(resources.join("creatures/MULE.ANB")).unwrap()).unwrap();
  let export=|w:&ActorWorld,n:u32|{if let Some(root)=std::env::var_os("GENEWARS_OWNED_MULE_CAPTURE"){let d=std::path::PathBuf::from(root).join(format!("frame-{n}"));std::fs::create_dir_all(&d).unwrap();for(name,data)in[("backing",w.backing.clone()),("map",w.map().to_vec()),("creatures",w.creatures.as_ref().unwrap().records().concat()),("buildings",w.buildings.as_ref().unwrap().records().concat()),("players",w.players.iter().flat_map(|p|p.bytes().iter().copied()).collect())]{std::fs::write(d.join(format!("{name}.bin")),data).unwrap();}let mut r=w.rng.seed.to_le_bytes().to_vec();r.push(w.rng.ring);r.extend(w.rng.draw_count.to_le_bytes());std::fs::write(d.join("rng.bin"),r).unwrap();}};
  export(&w,0);
  for frame in 1..=18{
   if frame==18{let r=w.creatures.as_ref().unwrap().record(1).unwrap();let target=[word(r,7)+256,word(r,9)];let manager=[0;0x59d];let packet=crate::mule_human_producer::CommandPacket{words:[0;5],opcode:0};let mut input=crate::mule_human_producer::ProducerInput{selected_player_index:0,selected_player_owner:0,suppression_flags:0,pause_flags:2,hover_consumed_cl:0,projected_point:target};
    let saved=w.clone();let ignored=w.dispatch_owned_mule_action5(Some(1),packet,&manager,input,false).unwrap();assert!(ignored.command.is_none());assert_eq!(w,saved);assert_eq!(ignored.producer.manager_after,manager);
    input.pause_flags=0;let receipt=w.dispatch_owned_mule_action5(Some(1),packet,&manager,input,false).unwrap();assert_eq!(receipt.producer.packet_after.opcode,0x11);assert!(receipt.command.is_some());}
   let before=w.clone();let result=w.step_owned_mule_slot(1,0,0,MuleStepContext{frame_counter:frame,gravity:1024,water_height:i32::from(i16::from_le_bytes([raw[0x1c61],raw[0x1c62]])),climate:1632},&physics,&trig,&animations);
   if result.is_err(){assert_eq!(w,before);panic!("frame{frame}: {result:?}");}export(&w,frame);
  }
 }
}
