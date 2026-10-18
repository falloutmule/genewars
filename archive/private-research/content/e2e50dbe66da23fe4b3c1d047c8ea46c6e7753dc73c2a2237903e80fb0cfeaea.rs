use genewars_assets::ConqMissionSource;
use genewars_runtime::{actor_world::ActorWorld,actor_pool::ActorHandleBindings,
    blueprint::BlueprintRng,player_state::PlayerState,family8_pool::AnimationBinding};
use std::{fs,path::Path,io::Write};
fn snapshot(world:&ActorWorld)->Vec<u8>{
    let mut bytes=world.backing_bytes().to_vec();
    for record in world.actors().records(){bytes.extend(record);}
    for record in world.creatures().unwrap().records(){bytes.extend(record);}
    for record in world.family8().unwrap().records(){bytes.extend(record);}
    for player in 0..4 {bytes.extend(world.player(player).unwrap().bytes());}
    let rng=world.rng();bytes.extend(rng.seed.to_le_bytes());bytes.push(rng.ring);bytes.extend(rng.draw_count.to_le_bytes());
    for slot in 0..16 {bytes.push(match world.family8().unwrap().binding(slot).unwrap(){None=>0,Some(AnimationBinding::Base)=>1,Some(AnimationBinding::Offset80)=>2});}
    bytes
}
fn main(){
    let args:Vec<_>=std::env::args().collect();let root=Path::new(&args[1]);let work=Path::new(&args[2]);
    let mission=fs::read(root.join("private/integration/game-runtime/terrain/CONQ001.DAT")).unwrap();
    let source=ConqMissionSource::parse(&mission).unwrap();
    let point=|offset|[i16::from_le_bytes(mission[offset..offset+2].try_into().unwrap()),i16::from_le_bytes(mission[offset+2..offset+4].try_into().unwrap())];
    let points=[point(277838+7),point(243038+7),[32767,-32768]];
    let mut ledger=fs::File::create(work.join("cases.tsv")).unwrap();
    for (scenario,point)in points.into_iter().enumerate(){
        let players=core::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,[20736,13568],b"fixture\0").unwrap());
        let mut world=ActorWorld::from_mission(&source,&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed:5428,ring:31,draw_count:u32::MAX}).unwrap();
        world.admit_creatures(&[[0;124];300]).unwrap();world.admit_family8(&[[0xA5;102];16],[None;16]).unwrap();
        for attempt in 0..17u8 {
            let name=format!("scenario{scenario}-attempt{attempt}");let dir=work.join(&name);fs::create_dir(&dir).unwrap();
            fs::write(dir.join("before.bin"),snapshot(&world)).unwrap();
            let height=[i32::MIN,-1,25600,i32::MAX][attempt as usize%4];
            let result=world.create_family8(attempt%3,attempt%4,point,height).unwrap();
            assert_eq!(result.is_some(),attempt<16);
            fs::write(dir.join("after.bin"),snapshot(&world)).unwrap();
            writeln!(ledger,"{name}\t{}\t{}\t{}\t{}\t{height}",attempt%3,attempt%4,point[0],point[1]).unwrap();
        }
    }
}
