use genewars_assets::ConqMissionSource;
use genewars_runtime::{actor_world::{ActorWorld,ActorWorldError},actor_pool::ActorHandleBindings,
 blueprint::BlueprintRng,player_state::PlayerState,building_resources::BuildingResourceDatabase,
 source_building_resources::{SourceBuildingResourceDatabase,SourceBuildingBinding},physics_tables::PhysicsTables,
 building_callback::BuildingCallbackError};
use std::{fs,path::Path};
fn main(){
 let a:Vec<_>=std::env::args().collect();let root=Path::new(&a[1]);let load=|s:&str|fs::read(root.join(s)).unwrap();
 let raw=load("private/integration/game-runtime/terrain/CONQ001.DAT");let mission=ConqMissionSource::parse(&raw).unwrap();
 let players=core::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,[20736,13568],b"fixture\0").unwrap());
 let mut world=ActorWorld::from_mission(&mission,&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed:5428,ring:31,draw_count:0}).unwrap();
 world.admit_creatures(&[[0;124];300]).unwrap();world.admit_buildings(&[[0;67];256],[None;256]).unwrap();
 let slot=world.create_building(0,0,[19839,12927]).unwrap().unwrap().slot;
 let resource=SourceBuildingResourceDatabase::load(SourceBuildingBinding::CdBgeneSelector0,load("private/integration/game-runtime/buildings/BGENE.ANB")).unwrap();
 let turret=BuildingResourceDatabase::load_selector1_bturr(load("private/reference-run/run-006/drive-c/GAME/DATA/BUILDING/BTURR.ANB")).unwrap();
 let physics=PhysicsTables::from_mapped_bytes(&load("private/integration/game-runtime/physics/angle.bin"),&load("private/integration/game-runtime/physics/bounce.bin")).unwrap();
 assert!(world.progress_admitted_grade0_building(slot,280,false).unwrap().completed);
 let before=world.clone();let error=world.step_admitted_building_callbacks(1,0,&turret,&[resource],1,&physics,|_|false).unwrap_err();
 assert!(matches!(error,ActorWorldError::BuildingCallback(BuildingCallbackError::UnsupportedBuilding)),"{error:?}");
 assert_eq!(world,before);println!("passed: completed Genepod callback is explicitly unsupported; entire world unchanged");
}
