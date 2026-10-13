use genewars_assets::ConqMissionSource;
use genewars_runtime::{actor_world::ActorWorld,actor_pool::ActorHandleBindings,blueprint::BlueprintRng,player_state::{PlayerState,ActorOriginSample}};
use std::{fs,io::{BufWriter,Write},path::Path};
fn main(){
 let dir=std::env::args().nth(1).unwrap();let dir=Path::new(&dir);let bytes=fs::read(dir.join("mission.bin")).unwrap();let source=ConqMissionSource::parse(&bytes).unwrap();
 let mut players=core::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,[20736,13568],b"review\0").unwrap());
 let mut record=*players[0].bytes();for i in 0..4{let at=0x16+i*36;record[at]=b'A'+i as u8;record[at+21]=0;record[at+22]=i as u8;}
 players[0]=PlayerState::from_record(&record).unwrap();let priors=core::array::from_fn(|id|core::array::from_fn(|i|((i*37+id*19)&255)as u8));
 let mut world=ActorWorld::from_mission(&source,&priors,[ActorHandleBindings::empty();20],players,BlueprintRng{seed:5428,ring:31,draw_count:u32::MAX}).unwrap();
 world.prepare_fresh_player(0,15,ActorOriginSample{position:[0,0],height:1234,shade:67},[20736,13568]).unwrap();
 let before=world.clone();let mut out=BufWriter::new(fs::File::create(dir.join("query-output.bin")).unwrap());
 for y in 0..128 {for x in 0..128 {
  for [fx,fy]in [[0,0],[127,127],[128,0],[0,128],[128,128],[255,255],[127,128],[128,127],[255,0]] {
   let point=[(x*256+fx)as i16,(y*256+fy)as i16];out.write_all(&[world.terrain_class_at(point).unwrap(),world.move_obstacle_at(point).unwrap()as u8]).unwrap();
  }
  for id in 0..2 {let point=[(x*256+128)as i16,(y*256+127)as i16];let direct=world.cell_plant_blocker(id,point).unwrap();let env=world.placement_environment(id,point).unwrap();out.write_all(&direct.to_le_bytes()).unwrap();out.write_all(&[env.terrain_class]).unwrap();out.write_all(&env.blocker_weight.to_le_bytes()).unwrap();}
 }}
 for raw in 0..65536u32 {let point=[raw as i16,(raw^0xa55a)as i16];out.write_all(&[world.terrain_class_at(point).unwrap(),world.move_obstacle_at(point).unwrap()as u8]).unwrap();}
 for id in [-32768,-1,2,19,20,32767]{assert!(world.cell_plant_blocker(id,[0,0]).is_err());}
 assert_eq!(world,before);out.flush().unwrap();
}
