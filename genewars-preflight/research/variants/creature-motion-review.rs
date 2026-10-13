use genewars_runtime::{creature_world::*,actor_activation::TrigTables,physics_tables::PhysicsTables};
use std::{fs,path::PathBuf};
fn main(){
 let p=PathBuf::from(std::env::args().nth(1).unwrap());let mission=fs::read(p.join("mission.bin")).unwrap();
 let physics=PhysicsTables::from_mapped_bytes(&fs::read(p.join("angles.bin")).unwrap(),&fs::read(p.join("bounce.bin")).unwrap()).unwrap();
 let trig=TrigTables::generate_pinned().unwrap();let cases=fs::read(p.join("physics-cases.bin")).unwrap();
 for row in cases.chunks_exact(255){let n=u32::from_le_bytes(row[..4].try_into().unwrap());let present=row[4]!=0;let speed=i16::from_le_bytes(row[5..7].try_into().unwrap());
  let before:[u8;124]=row[7..131].try_into().unwrap();let expected=&row[131..255];let mut called=false;
  let got=plan_common_nonflight_physics(&before,&mission[0x355e..0x3b55e],n,1024,1632,&physics,&trig,|_,_,_|{called=true;Ok(speed)},|_|Ok(false)).unwrap();
  assert_eq!(called,present,"contact frame{n}");
  for i in 6..124 {assert_eq!(got.after[i],expected[i],"physics frame{n} offset{i:02x}");}
 }
 println!("138 commonphysics full numeric afterimages matched");
 let anb=fs::read(p.join("mule.anb")).unwrap();let frames=u32::from_le_bytes(anb[10..14].try_into().unwrap())as usize;let animations=u32::from_le_bytes(anb[14..18].try_into().unwrap())as usize;
 let mule_animations=MuleAnimations::from_anb(&anb).unwrap();
 let cases=fs::read(p.join("followup-cases.bin")).unwrap();
 for row in cases.chunks_exact(255){let n=u32::from_le_bytes(row[..4].try_into().unwrap());let present=row[4]!=0;let speed=i16::from_le_bytes(row[5..7].try_into().unwrap());let before:[u8;124]=row[7..131].try_into().unwrap();let expected=&row[131..255];
  let desc=animations+usize::from(expected[0x19])*10;let first=u32::from_le_bytes(anb[desc..desc+4].try_into().unwrap())as usize;let count=usize::from(anb[desc+8]);let holds=(0..count).map(|i|anb[frames+first+14*i+10]).collect::<Vec<_>>();let mut called=false;
  let got=plan_initial_mule_followup(&before,&mission[0x355e..0x3b55e],MuleStepContext{frame_counter:n,gravity:1024,water_height:1632,climate:106},&physics,&trig,true,true,false,|_,_,_|{called=true;Ok(speed)},|_|Ok(false),&mule_animations).unwrap();
  assert_eq!(called,present,"full contact frame{n}");for i in 6..124{assert_eq!(got.after[i],expected[i],"full frame{n} offset{i:02x}");}
 }
 println!("137 followup full numeric afterimages matched");
 use genewars_runtime::{actor_world::ActorWorld,actor_pool::ActorHandleBindings,player_state::PlayerState,blueprint::BlueprintRng};
 use std::io::{BufWriter,Write};
 let source=genewars_assets::ConqMissionSource::parse(&mission).unwrap();
 let mut world=ActorWorld::from_mission(&source,&[[0;200];20],[ActorHandleBindings::empty();20],core::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,[20736,13568],b"review\0").unwrap()),BlueprintRng{seed:5428,ring:31,draw_count:u32::MAX}).unwrap();world.admit_creatures(&[[0;124];300]).unwrap();
 let initial_actors=world.actors().clone();let initial_players=core::array::from_fn::<_,4,_>(|p|world.player(p as u8).unwrap().clone());let initial_pool=world.creatures().unwrap().clone();
 let mut out=BufWriter::new(fs::File::create(p.join("owned-afterimages.bin")).unwrap());
 for n in 1..=138 {if n==1 {world.step_initial_mule_choice(n,&trig).unwrap();}else {world.step_initial_mule_followup(MuleStepContext{frame_counter:n,gravity:1024,water_height:1632,climate:106},&physics,&trig,&mule_animations).unwrap();}
  out.write_all(world.creatures().unwrap().record(0).unwrap()).unwrap();out.write_all(world.map()).unwrap();
  assert_eq!(world.actors(),&initial_actors);for i in 0..4{assert_eq!(world.player(i as u8).unwrap(),&initial_players[i]);}
  for i in 1..300{assert_eq!(world.creatures().unwrap().record(i).unwrap(),initial_pool.record(i).unwrap());}
  assert_eq!(world.rng(),BlueprintRng{seed:64612,ring:4,draw_count:4});
 }
 out.flush().unwrap();println!("138 owned callbacks completed");
 // Adversarial late environmental failure must not commit the already
 // calculated steering, velocity, native record or shared RNG.
 let saved=world.clone();assert!(world.step_initial_mule_followup(MuleStepContext{frame_counter:139,gravity:1024,water_height:1632,climate:106},&physics,&trig,&mule_animations).is_err());assert_eq!(world,saved);
 let mut no_pool=world.clone();let saved_no_pool=no_pool.clone();assert!(no_pool.step_mule_followup_with_water(MuleStepContext{frame_counter:139,gravity:1024,water_height:1632,climate:106},&physics,&trig,&mule_animations).is_err());assert_eq!(no_pool,saved_no_pool);
 let mut wet=world.clone();wet.admit_effects(&[[0;81];200],[None;200]).unwrap();let request=wet.step_mule_followup_with_water(MuleStepContext{frame_counter:139,gravity:1024,water_height:1632,climate:106},&physics,&trig,&mule_animations).unwrap();assert_eq!(request.pending_audio,vec![0x3b]);assert_eq!(wet.rng(),world.rng());assert_eq!(wet.effects().unwrap().records().iter().filter(|r|r[6]!=0).count(),1);fs::write(p.join("water139-mule.bin"),wet.creatures().unwrap().record(0).unwrap()).unwrap();fs::write(p.join("water139-effect.bin"),wet.effects().unwrap().record(0).unwrap()).unwrap();
 let mut rejected=world.clone();let saved=rejected.clone();
 assert!(rejected.step_initial_mule_followup(MuleStepContext{frame_counter:191,gravity:1024,water_height:30000,climate:106},&physics,&trig,&mule_animations).is_err());assert_eq!(rejected,saved);
 assert!(MuleAnimations::from_anb(&anb[..anb.len()-1]).is_err());
 let mut changed=anb.clone();changed[0]^=1;assert!(MuleAnimations::from_anb(&changed).is_err());
 let mut live=ActorWorld::from_mission(&source,&[[0;200];20],[ActorHandleBindings::empty();20],core::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,[20736,13568],b"review\0").unwrap()),BlueprintRng{seed:10548,ring:31,draw_count:64}).unwrap();live.admit_creatures(&[[0;124];300]).unwrap();
 live.admit_effects(&[[0;81];200],[None;200]).unwrap();live.create_landing_core(0,0).unwrap();live.apply_reveal_4096([20736,13568]).unwrap();
 fs::write(p.join("startup-context.bin"),live.backing_bytes()).unwrap();let mut out=BufWriter::new(fs::File::create(p.join("startup-afterimages.bin")).unwrap());
 for n in 1..=314 {if n==1{live.step_initial_mule_choice(n,&trig).unwrap();}else{live.step_initial_mule_followup(MuleStepContext{frame_counter:n,gravity:1024,water_height:1632,climate:106},&physics,&trig,&mule_animations).unwrap_or_else(|e|{fs::write(p.join("native-failed-before.bin"),live.creatures().unwrap().record(0).unwrap()).unwrap();panic!("live frame{n}: {e:?}")});}
  out.write_all(live.creatures().unwrap().record(0).unwrap()).unwrap();out.write_all(live.map()).unwrap();}
 out.flush().unwrap();assert_eq!(live.rng(),BlueprintRng{seed:228,ring:12,draw_count:77});
 for (side,limit,cost,point) in [(false,120,24,[16384,14592]),(true,25,5,[16128,16000])] {
  let got=plan_common_waypoint([16384,15872],[16384,15872],[15225,15670],side,limit,&physics,
   |p|live.common_move_point_accepted([0,0,1],p).map_err(|_|CreatureError::MotionEnvironment),
   |a,b|live.common_straight_move_ray([0,0,1],a,b,&physics,&trig).map_err(|_|CreatureError::MotionEnvironment)).unwrap();
  assert_eq!(got,CommonWaypoint{cost,point:Some(point)});
 }
 let saved=live.clone();assert!(live.step_initial_mule_followup(MuleStepContext{frame_counter:315,gravity:1024,water_height:1632,climate:106},&physics,&trig,&mule_animations).is_err());assert_eq!(live,saved);
 println!("314 shared-startup-context callbacks plus5 rejection checks completed");
}
