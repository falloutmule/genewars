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
  let mut rng=self.rng;let(prefix,target)=self.mule_route_prefix(&before,context.frame_counter,physics,trig,&mut rng)?;
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
    let world=stage.borrow();let mut r=*record;if r[0x5e]==0{r[0x5e]=8;}
    world.owned_mule_neighborhood_no_action(&r).map_err(|_|CreatureError::UnsupportedNeighborhood)?;
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
 /// Bounded no-action neighborhood: full reserve, only own Genepods and
 /// neutral plants, same-owner specialists and Genepod/Extractor buildings.
 /// Source6BA47 kinds3/4 bypass food/pair scoring; building-action queries
 /// still run separately. Same-owner water effects also take6BA47 kind>2.
 ///6BD3D->6B72A skips code1A (all effects except11) before classification;
 /// this adapter additionally proves same-owner/EFF26. Other candidates require the scorer.
 fn owned_mule_neighborhood_no_action(&self,r:&CreatureRecord)->Result<(),ActorWorldError>{
  if word(r,0x68)<=word(r,0x62)>>1||r[0x61]&0x40!=0{return Err(CreatureError::UnsupportedNeighborhood.into());}
  let own=[r[15],r[16],1];let point=[word(r,7),word(r,9)];let cell=floor_cell(point);let(cx,cy)=((cell%128)as i32,(cell/128)as i32);
  //8D930 visits sample cells within radius1024, including its boundary.
  for dy in -4..=4{for dx in -4..=4{if dx*dx+dy*dy>16{continue;}let c=(((cy+dy)&127)*128+((cx+dx)&127))as usize;let mut raw=reference(self.map(),c*14+6);
   while raw[2]!=0{let e=self.entity(raw)?;if raw!=own{match raw[2]{2 if e[24]==4=>{},3 if e[24]==r[24]=>{},4 if e[24]==r[24]&&matches!(self.buildings.as_ref().unwrap().record(word(&raw,0)as usize)?[0x2d],0|11)=>{},5 if e[24]==r[24]&&(matches!(self.effects.as_ref().unwrap().record(word(&raw,0)as usize)?[69],0x15|0x30)||(e[69]==0x1a&&self.effects.as_ref().unwrap().binding(word(&raw,0)as usize)?==Some(EffectAnimationBinding::EffAnimation26)))=>{},8=>{},_=>return Err(CreatureError::UnsupportedNeighborhood.into())}}raw=reference(e,3);}
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
