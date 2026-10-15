use genewars_assets::ConqMissionSource;
use genewars_runtime::{actor_world::ActorWorld,actor_pool::ActorHandleBindings,
    blueprint::BlueprintRng,player_state::PlayerState,building::BuildingAnimationBinding};
use std::{fs,path::Path,io::Write};
fn snapshot(world:&ActorWorld)->Vec<u8>{
    let mut bytes=world.backing_bytes().to_vec();
    for record in world.actors().records(){bytes.extend(record);}
    for record in world.creatures().unwrap().records(){bytes.extend(record);}
    for record in world.buildings().unwrap().records(){bytes.extend(record);}
    for player in 0..4 {bytes.extend(world.player(player).unwrap().bytes());}
    let rng=world.rng();bytes.extend(rng.seed.to_le_bytes());bytes.push(rng.ring);bytes.extend(rng.draw_count.to_le_bytes());
    for slot in 0..256{bytes.push(world.buildings().unwrap().binding(slot).unwrap().map(|BuildingAnimationBinding{selector}|selector+1).unwrap_or(0));}
    bytes
}
fn main(){
    let args:Vec<_>=std::env::args().collect();let root=Path::new(&args[1]);let work=Path::new(&args[2]);
    let mission=fs::read(root.join("private/integration/game-runtime/terrain/CONQ001.DAT")).unwrap();
    let source=ConqMissionSource::parse(&mission).unwrap();
    let mut ledger=fs::File::create(work.join("cases.tsv")).unwrap();
    for (selector,name,maximum,binding) in [
        (0,"BGENE",280,genewars_runtime::source_building_resources::SourceBuildingBinding::CdBgeneSelector0)] {
    let resource=genewars_runtime::source_building_resources::SourceBuildingResourceDatabase::load(binding,fs::read(root.join(format!("private/integration/game-runtime/buildings/{name}.ANB"))).unwrap()).unwrap();
    let wrong_binding=if selector==3{genewars_runtime::source_building_resources::SourceBuildingBinding::CdBsolaSelector6}else{genewars_runtime::source_building_resources::SourceBuildingBinding::CdBpoweSelector3};
    let wrong_name=if selector==3{"BSOLA"}else{"BPOWE"};
    let wrong=genewars_runtime::source_building_resources::SourceBuildingResourceDatabase::load(wrong_binding,fs::read(root.join(format!("private/integration/game-runtime/buildings/{wrong_name}.ANB"))).unwrap()).unwrap();
    for owner in 0..4u8 {for loading in [false,true]{
        let players=core::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,[20736,13568],b"fixture\0").unwrap());
        let mut world=ActorWorld::from_mission(&source,&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed:5428,ring:31,draw_count:u32::MAX}).unwrap();
        world.admit_creatures(&[[0;124];300]).unwrap();world.admit_buildings(&[[0xA5;67];256],[None;256]).unwrap();
        let slot=world.create_building(owner,selector,[19839,12927]).unwrap().unwrap().slot;
        for selector in [0,4,11,13]{world.create_building(owner,selector,[19839,12927]).unwrap().unwrap();}
        let before=world.clone();assert!(world.step_incomplete_source_building(slot,&wrong).is_err());assert_eq!(world,before);
        for (step,increment)in [1,maximum/2,maximum/2-2,1].into_iter().enumerate(){
            let name=format!("selector{selector}-owner{owner}-loading{loading}-callback{step}");let dir=work.join(&name);fs::create_dir(&dir).unwrap();
            fs::write(dir.join("before.bin"),snapshot(&world)).unwrap();
            let callback=world.step_incomplete_source_building(slot,&resource).unwrap();
            assert_eq!(callback.stop_audio_23,selector==6);assert_eq!(callback.stop_audio_2f,selector==11);assert_eq!(callback.queue_requested,step!=0);
            fs::write(dir.join("after.bin"),snapshot(&world)).unwrap();
            writeln!(ledger,"{name}\t{selector}\t{owner}\t{slot}\t0\t{}\t1",u8::from(loading)).unwrap();
            let name=format!("selector{selector}-owner{owner}-loading{loading}-step{step}");let dir=work.join(&name);fs::create_dir(&dir).unwrap();
            fs::write(dir.join("before.bin"),snapshot(&world)).unwrap();
            let result=world.progress_admitted_grade0_building(slot,increment,loading).unwrap();
            assert_eq!(result.completed,step==3);assert_eq!(result.pending_audio_4d,step==3&&!loading&&selector!=6);
            fs::write(dir.join("after.bin"),snapshot(&world)).unwrap();
            writeln!(ledger,"{name}\t{selector}\t{owner}\t{slot}\t{increment}\t{}\t0",u8::from(loading)).unwrap();
        }
        let before=world.clone();assert!(world.progress_admitted_grade0_building(slot,1,loading).is_err());assert_eq!(world,before);
    }}}
}
