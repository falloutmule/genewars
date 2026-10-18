use genewars_assets::ConqMissionSource;
use genewars_runtime::{actor_world::{ActorWorld,ActorWorldError},actor_pool::ActorHandleBindings,
 blueprint::BlueprintRng,player_state::PlayerState,building_resources::BuildingResourceDatabase,
 completed_building::QuietBuildingError};
use std::{fs,path::Path,io::Write};
fn snapshot(w:&ActorWorld)->Vec<u8>{let mut b=w.backing_bytes().to_vec();for r in w.actors().records(){b.extend(r)}for r in w.creatures().unwrap().records(){b.extend(r)}for r in w.buildings().unwrap().records(){b.extend(r)}for p in 0..4{b.extend(w.player(p).unwrap().bytes())}let r=w.rng();b.extend(r.seed.to_le_bytes());b.push(r.ring);b.extend(r.draw_count.to_le_bytes());for i in 0..256{b.push(w.buildings().unwrap().binding(i).unwrap().map(|v|v.selector+1).unwrap_or(0))}b}
fn main(){let a:Vec<_>=std::env::args().collect();let p=Path::new(&a[1]);let out=Path::new(&a[2]);let raw=fs::read(p.join("private/integration/game-runtime/terrain/CONQ001.DAT")).unwrap();let src=ConqMissionSource::parse(&raw).unwrap();let resource=BuildingResourceDatabase::load_selector1_bturr(fs::read(p.join("private/reference-run/run-006/drive-c/GAME/DATA/BUILDING/BTURR.ANB")).unwrap()).unwrap();let mut ledger=fs::File::create(out.join("cases.tsv")).unwrap();
 let resources=[("BPOWE",genewars_runtime::source_building_resources::SourceBuildingBinding::CdBpoweSelector3),("BSOLA",genewars_runtime::source_building_resources::SourceBuildingBinding::CdBsolaSelector6),("BMINE",genewars_runtime::source_building_resources::SourceBuildingBinding::CdBmineSelector11)].into_iter().map(|(n,b)|genewars_runtime::source_building_resources::SourceBuildingResourceDatabase::load(b,fs::read(p.join(format!("private/integration/game-runtime/buildings/{n}.ANB"))).unwrap()).unwrap()).collect::<Vec<_>>();
 let physics=genewars_runtime::physics_tables::PhysicsTables::from_mapped_bytes(&fs::read(p.join("private/integration/game-runtime/physics/angle.bin")).unwrap(),&fs::read(p.join("private/integration/game-runtime/physics/bounce.bin")).unwrap()).unwrap();
 for case in 0..9u8 {
  let selected=case%4;let player_count=if case==8{4}else{[1,4][usize::from(case/4)]};let audio_active=case%2!=0;
  let players=core::array::from_fn(|owner|{let mut r=*PlayerState::fresh_campaign(owner as u8,[0,0],b"fixture\0").unwrap().bytes();r[0x5e5]=[0,49,255][(usize::from(case)+owner)%3];r[0x4a7..0x4ab].copy_from_slice(&[0i32,601,999,1800][(usize::from(case)+owner)%4].to_le_bytes());PlayerState::from_record(&r).unwrap()});
  let mut w=ActorWorld::from_mission(&src,&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed:[0,1,5428,65535][usize::from(case%4)],ring:31,draw_count:u32::MAX}).unwrap();w.admit_creatures(&[[0;124];300]).unwrap();w.admit_buildings(&[[0;67];256],[None;256]).unwrap();
  for owner in 0..4u8 {for (n,selector) in [1,3,6,11].into_iter().enumerate(){
   let slot=w.create_building(owner,selector,[19839+n as i16*256,12927+i16::from(owner)*256]).unwrap().unwrap().slot;
   w.progress_admitted_grade0_building(slot,match selector{1=>120,11=>280,_=>20},true).unwrap();
  }}
  if case==8{let slot=w.create_building(3,3,[21119,13695]).unwrap().unwrap().slot;w.progress_admitted_grade0_building(slot,240,true).unwrap();}
  let name=format!("case{case}");let dir=out.join(&name);fs::create_dir(&dir).unwrap();fs::write(dir.join("before.bin"),snapshot(&w)).unwrap();let before=w.clone();
  let result=w.step_admitted_building_callbacks(player_count,selected,&resource,&resources,0,&physics,|_|audio_active);
  let callbacks=if case==8{assert!(result.is_err());assert_eq!(w,before);0}else{let pass=result.unwrap();let audio=pass.callbacks.iter().filter_map(|c|if let genewars_runtime::actor_world::AdmittedBuildingCallback::Extractor{plan,..}=c{Some(plan.audio.len())}else{None}).sum::<usize>();assert_eq!(audio,if audio_active{usize::from(player_count)*2}else{0});pass.callbacks.len()};
  fs::write(dir.join("after.bin"),snapshot(&w)).unwrap();writeln!(ledger,"{name}\t{player_count}\t{selected}\t{}\t{}\t{callbacks}",u8::from(case==8),u8::from(audio_active)).unwrap();
 }println!("8 mixed passes and one late unsupported Distributor rollback");}
