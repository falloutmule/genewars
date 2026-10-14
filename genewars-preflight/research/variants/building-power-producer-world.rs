use super::*;use std::{fs,path::Path};
fn dump(world:&ActorWorld,w:&Path,label:&str){
 fs::write(w.join(format!("{label}-map.bin")),world.map()).unwrap();
 fs::write(w.join(format!("{label}-buildings.bin")),world.buildings.as_ref().unwrap().records().concat()).unwrap();
 fs::write(w.join(format!("{label}-effects.bin")),world.effects.as_ref().unwrap().records().concat()).unwrap();
 fs::write(w.join(format!("{label}-players.bin")),world.players.iter().flat_map(|p|p.bytes().iter().copied()).collect::<Vec<_>>()).unwrap();
 let mut rng=world.rng.seed.to_le_bytes().to_vec();rng.push(world.rng.ring);rng.extend(world.rng.draw_count.to_le_bytes());fs::write(w.join(format!("{label}-rng.bin")),rng).unwrap();
 fs::write(w.join(format!("{label}-count.bin")),&world.backing[0x3554..0x3556]).unwrap();
}
pub fn review_power_producers(p:&Path,w:&Path){
 let trig=TrigTables::generate_pinned().unwrap();let t=p.join("private/review/family8-state4");let physics=crate::physics_tables::PhysicsTables::from_mapped_bytes(&fs::read(t.join("angles.bin")).unwrap(),&fs::read(t.join("bounce.bin")).unwrap()).unwrap();
 for case in 0..32u32{
 let folder=w.join(case.to_string());fs::create_dir(&folder).unwrap();
 let mut bytes=vec![0;343958];bytes[..4].copy_from_slice(&0x55336u32.to_le_bytes());bytes[337054..337058].copy_from_slice(&0x6beu32.to_le_bytes());for c in 0..16384{bytes[MAP_START+c*14+2..MAP_START+c*14+4].copy_from_slice(&3136i16.to_le_bytes());bytes[MAP_START+c*14+5]=7;}
 let src=ConqMissionSource::parse(&bytes).unwrap();let players=std::array::from_fn(|i|PlayerState::fresh_campaign(i as u8,[4096,4096],b"fixture\0").unwrap());let mut world=ActorWorld::from_parts(&src,&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed:[65535,5428,0,1][case as usize%4],ring:31,draw_count:99}).unwrap();world.admit_effects(&[[0;81];200],[None;200]).unwrap();world.admit_buildings(&[[0;67];256],[None;256]).unwrap();
 let owner=(case/2%4)as u8;let selector=if case%2==0{3}else{6};let slot=world.create_building(owner,selector,[4096,4096]).unwrap().unwrap().slot;
 for (i,x) in [4352,4608,5120].iter().enumerate(){let target=if selector==3{11}else{3};let id=world.create_building(owner,target,[*x,4096]).unwrap().unwrap().slot;let old=*world.buildings.as_ref().unwrap().record(id as usize).unwrap();let mut new=old;new[49]=if case%8==0{0}else{1};new[46]=0;new[61..63].copy_from_slice(&(10+i as i16).to_le_bytes());world.buildings.as_mut().unwrap().replace_record_checked(id as usize,&old,&new).unwrap();}
 let before=*world.buildings.as_ref().unwrap().record(slot as usize).unwrap();let mut b=before;b[49]=5;b[27]=[0,1,6,8,9,255,1,6][case as usize%8];b[26]=[0,1,255,2][case as usize%4];b[29]=[0,5,128,255][case as usize%4];b[30]=[0,1,127,255][case as usize%4];b[61..63].copy_from_slice(&[0i16,199,200,1999][case as usize%4].to_le_bytes());if case==4{b[61..63].copy_from_slice(&199i16.to_le_bytes());}world.buildings.as_mut().unwrap().replace_record_checked(slot as usize,&before,&b).unwrap();
 let root=p.join("private/reference-run/run-006/drive-c/GAME/DATA/BUILDING");let(name,binding)=if selector==3{("BPOWE",crate::source_building_resources::SourceBuildingBinding::CdBpoweSelector3)}else{("BSOLA",crate::source_building_resources::SourceBuildingBinding::CdBsolaSelector6)};let resource=crate::source_building_resources::SourceBuildingResourceDatabase::load(binding,fs::read(root.join(format!("{name}.ANB"))).unwrap()).unwrap();
 let frame=if case%4<2{0}else{1};let solar=4;dump(&world,&folder,"before");let original=world.clone();let receipt=world.step_completed_power_building(slot,&resource,frame,solar,&physics,&trig).unwrap();assert_eq!(world.actors,original.actors);world.validate_links().unwrap();dump(&world,&folder,"after");fs::write(folder.join("parameters.txt"),format!("{slot} {selector} {frame} {solar} {}",receipt.created_effects.len())).unwrap();
 if case==4{let mut full=original.clone();let req=receipt.numeric.transfers.first().unwrap();for _ in 0..199{full.create_power_transfer33(req,&trig).unwrap();}let frozen=full.clone();assert!(full.step_completed_power_building(slot,&resource,frame,solar,&physics,&trig).is_err());assert_eq!(full,frozen);}
 }
 let mut pool=[[0;81];200];for r in &mut pool{r[6]=5;r[69]=33;}
 for case in 0..4{if case==1{pool[0][69]=6;pool[0][21]=1;}if case==2{pool[1][69]=6;}if case==3{pool[2][69]=0x2f;pool[3][69]=0x2f;}
 let selected=landing::water_splash_replacement(&pool).unwrap();assert_eq!(selected,[None,None,Some(1),Some(2)][case]);fs::write(w.join(format!("replacement{case}.bin")),pool.concat()).unwrap();}
 println!("passed32producerworldcases/lateallocationrollback/4replacementscans");
}
