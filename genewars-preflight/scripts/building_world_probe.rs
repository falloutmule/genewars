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
    let point=|offset|[i16::from_le_bytes(mission[offset..offset+2].try_into().unwrap()),i16::from_le_bytes(mission[offset+2..offset+4].try_into().unwrap())];
    let points=[point(277838+7),point(243038+7),[32767,-32768],[19839,12927]];
    let mut ledger=fs::File::create(work.join("cases.tsv")).unwrap();
    for (scenario,point)in points.into_iter().enumerate(){
        let players=core::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,[20736,13568],b"fixture\0").unwrap());
        let mut world=ActorWorld::from_mission(&source,&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed:5428,ring:31,draw_count:u32::MAX}).unwrap();
        world.admit_creatures(&[[0;124];300]).unwrap();world.admit_buildings(&[[0xA5;67];256],[None;256]).unwrap();
        for attempt in 0..=64u8 {
            let position=[point[0].wrapping_add(i16::from(attempt%4)*256),point[1]];
            let name=format!("owner{scenario}-attempt{attempt}");let dir=work.join(&name);
            let capture=attempt<14||attempt==64;
            if capture{fs::create_dir(&dir).unwrap();fs::write(dir.join("before.bin"),snapshot(&world)).unwrap();}
            let result=world.create_building(scenario as u8,attempt%14,position).unwrap();
            assert_eq!(result.is_some(),attempt<64);
            if capture{
                fs::write(dir.join("after.bin"),snapshot(&world)).unwrap();
                writeln!(ledger,"{name}\t{}\t{scenario}\t{}\t{}",attempt%14,position[0],position[1]).unwrap();
            }
        }
    }
}
