use genewars_runtime::creature_world::*;
use std::{fs,io::Write,path::Path};
fn main(){
 let arg=std::env::args().nth(1).unwrap();let dir=Path::new(&arg);let data=fs::read(dir.join("mission.bin")).unwrap();let source=genewars_assets::ConqMissionSource::parse(&data).unwrap();let priors=core::array::from_fn(|slot|core::array::from_fn(|i|(slot*37+i*19)as u8));let pool=CreaturePool::from_mission_source(&source,&priors).unwrap();let mut out=fs::File::create(dir.join("pool.bin")).unwrap();
 for i in 0..CREATURE_CAPACITY{out.write_all(pool.record(i).unwrap()).unwrap();out.write_all(&[pool.binding(i).unwrap().is_some()as u8]).unwrap();}
 assert_eq!(pool.binding(0).unwrap(),Some(CreatureAnimationBinding::MuleSelector4));assert!(pool.record(CREATURE_CAPACITY).is_err());
 let mut changed=data.clone();changed[0x3b55e+21]^=1;let source=genewars_assets::ConqMissionSource::parse(&changed).unwrap();assert!(matches!(CreaturePool::from_mission_source(&source,&priors),Err(CreatureError::MissionHash)));
}
#[test]
fn malformed_extent_and_active_binding_reject(){
 let priors=[[0xa5;124];300];let mut packed=vec![0;300*116];
 assert!(matches!(CreaturePool::from_packed(&packed[..packed.len()-1],&priors),Err(CreatureError::PackedLength(_))));
 packed[6]=3;assert!(matches!(CreaturePool::from_packed(&packed,&priors),Err(CreatureError::UnsupportedActiveRecord(0))));
}
#[test]
fn inactive_native_gap_is_preserved_without_binding(){
 let priors=[[0xa5;124];300];let packed=vec![0;300*116];let pool=CreaturePool::from_packed(&packed,&priors).unwrap();
 for i in 0..300{assert_eq!(&pool.record(i).unwrap()[31..39],&[0xa5;8]);assert_eq!(pool.binding(i).unwrap(),None);}
 assert!(pool.record(300).is_err());assert!(pool.binding(300).is_err());
}
