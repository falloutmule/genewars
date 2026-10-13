mod identity { pub use genewars_runtime::identity::sha256_hex; }
#[path="../../rust/crates/genewars-runtime/src/creature_world.rs"] mod creature_world;
use creature_world::*;
use std::{fs,io::Write,path::Path};
fn main(){
 let arg=std::env::args().nth(1).unwrap();let dir=Path::new(&arg);let data=fs::read(dir.join("mission.bin")).unwrap();let source=genewars_assets::ConqMissionSource::parse(&data).unwrap();let priors=core::array::from_fn(|slot|core::array::from_fn(|i|(slot*37+i*19)as u8));let pool=CreaturePool::from_mission_source(&source,&priors).unwrap();let mut out=fs::File::create(dir.join("pool.bin")).unwrap();
 for i in 0..CREATURE_CAPACITY{out.write_all(pool.record(i).unwrap()).unwrap();out.write_all(&[pool.binding(i).unwrap().is_some()as u8]).unwrap();}
 assert_eq!(pool.binding(0).unwrap(),Some(CreatureAnimationBinding::MuleSelector4));assert!(pool.record(CREATURE_CAPACITY).is_err());
 let mut changed=data.clone();changed[0x3b55e+21]^=1;let source=genewars_assets::ConqMissionSource::parse(&changed).unwrap();assert!(matches!(CreaturePool::from_mission_source(&source,&priors),Err(CreatureError::MissionHash)));
}
