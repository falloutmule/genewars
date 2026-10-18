use genewars_assets::ConqMissionSource;
use genewars_runtime::{actor_world::{ActorWorld,ActorWorldError},actor_pool::ActorHandleBindings,
 blueprint::BlueprintRng,player_state::PlayerState,building_resources::BuildingResourceDatabase,
 completed_building::QuietBuildingError};
use std::{fs,path::Path,io::Write};
fn snapshot(w:&ActorWorld)->Vec<u8>{let mut b=w.backing_bytes().to_vec();for r in w.actors().records(){b.extend(r)}for r in w.creatures().unwrap().records(){b.extend(r)}for r in w.buildings().unwrap().records(){b.extend(r)}for p in 0..4{b.extend(w.player(p).unwrap().bytes())}let r=w.rng();b.extend(r.seed.to_le_bytes());b.push(r.ring);b.extend(r.draw_count.to_le_bytes());for i in 0..256{b.push(w.buildings().unwrap().binding(i).unwrap().map(|v|v.selector+1).unwrap_or(0))}b}
fn main(){let a:Vec<_>=std::env::args().collect();let p=Path::new(&a[1]);let out=Path::new(&a[2]);let raw=fs::read(p.join("private/integration/game-runtime/terrain/CONQ001.DAT")).unwrap();let src=ConqMissionSource::parse(&raw).unwrap();let resource=BuildingResourceDatabase::load_selector1_bturr(fs::read(p.join("private/reference-run/run-006/drive-c/GAME/DATA/BUILDING/BTURR.ANB")).unwrap()).unwrap();let mut ledger=fs::File::create(out.join("cases.tsv")).unwrap();
 for case in 0..17u8 {
  let selected=case%4;let player_count=if case==16{4}else{[0,1,2,4][usize::from(case/4)]};
  let players=core::array::from_fn(|owner|{let mut r=*PlayerState::fresh_campaign(owner as u8,[0,0],b"fixture\0").unwrap().bytes();r[0x5e5]=[0,49,255][(usize::from(case)+owner)%3];r[0x4a7..0x4ab].copy_from_slice(&[-1i32,0,1,999][(usize::from(case)+owner)%4].to_le_bytes());for n in 0..8{r[0x5ea+n*2..0x5ec+n*2].copy_from_slice(&[0u16,1,999,1000,65535][(usize::from(case)+owner+n)%5].to_le_bytes());}PlayerState::from_record(&r).unwrap()});
  let mut w=ActorWorld::from_mission(&src,&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed:[0,1,5428,65535][usize::from(case%4)],ring:31,draw_count:u32::MAX}).unwrap();w.admit_creatures(&[[0;124];300]).unwrap();w.admit_buildings(&[[0;67];256],[None;256]).unwrap();
  for owner in 0..4u8 {for n in 0..(if case==16{1}else{(case+owner)%3+1}) {
   let at=if case==16&&owner==3{[16896,15616]}else{[19839+i16::from(n)*256,12927+i16::from(owner)*256]};let slot=w.create_building(owner,1,at).unwrap().unwrap().slot;
   let increment=if case==16||n&1==1{120}else{[0,1,62,119][usize::from((case+owner)%4)]};if increment!=0{w.progress_selector1_building(slot,increment,true).unwrap();}
  }}
  let name=format!("case{case}");let dir=out.join(&name);fs::create_dir(&dir).unwrap();fs::write(dir.join("before.bin"),snapshot(&w)).unwrap();let before=w.clone();let result=w.step_selector1_building_callbacks(player_count,selected,&resource);
  let callbacks=if case==16{assert!(matches!(result,Err(ActorWorldError::QuietBuilding(QuietBuildingError::AcceptedAttackTarget))));assert_eq!(w,before);0}else{result.unwrap().callbacks.len()};fs::write(dir.join("after.bin"),snapshot(&w)).unwrap();writeln!(ledger,"{name}\t{player_count}\t{selected}\t{}\t{callbacks}",u8::from(case==16)).unwrap();
 }println!("16 mixed passes and one late-target rollback");}
