use genewars_assets::ConqMissionSource;
use genewars_runtime::{actor_world::ActorWorld,actor_pool::ActorHandleBindings,
    blueprint::BlueprintRng,player_state::PlayerState,family8_pool::AnimationBinding};
use std::{fs,path::Path,io::Write};
fn snapshot(world:&ActorWorld)->Vec<u8>{
    let mut bytes=world.backing_bytes().to_vec();
    for record in world.actors().records(){bytes.extend(record);}
    for record in world.creatures().unwrap().records(){bytes.extend(record);}
    for record in world.family8().unwrap().records(){bytes.extend(record);}
    for owner in 0..4{bytes.extend(world.player(owner).unwrap().bytes());}
    let rng=world.rng();bytes.extend(rng.seed.to_le_bytes());bytes.push(rng.ring);bytes.extend(rng.draw_count.to_le_bytes());
    for slot in 0..16{bytes.push(match world.family8().unwrap().binding(slot).unwrap(){None=>0,Some(AnimationBinding::Base)=>1,Some(AnimationBinding::Offset80)=>2});}
    bytes
}
fn main(){
    let args:Vec<_>=std::env::args().collect();let root=Path::new(&args[1]);let work=Path::new(&args[2]);
    let mission=fs::read(root.join("private/integration/game-runtime/terrain/CONQ001.DAT")).unwrap();
    let source=ConqMissionSource::parse(&mission).unwrap();
    let point=|offset|[i16::from_le_bytes(mission[offset..offset+2].try_into().unwrap()),i16::from_le_bytes(mission[offset+2..offset+4].try_into().unwrap())];
    let points=[point(277838+7),point(243038+7),[32767,-32768]];
    let mut ledger=fs::File::create(work.join("cases.tsv")).unwrap();
    for (selector,playing)in [(1u8,false),(0,false),(0,true)] {
    for(scenario,point)in points.into_iter().enumerate(){
        let players=std::array::from_fn(|owner|PlayerState::fresh_campaign(owner as u8,[20736,13568],b"fixture\0").unwrap());
        let mut world=ActorWorld::from_mission(&source,&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed:5428,ring:31,draw_count:u32::MAX}).unwrap();
        world.admit_creatures(&[[0;124];300]).unwrap();world.admit_family8(&[[0xa5;102];16],[None;16]).unwrap();
        for slot in 0..16{world.create_family8(selector,slot%4,point,25600).unwrap().unwrap();}
        // First unlink a tail, middle and head; then exhaust the remaining list.
        for slot in [0u16,8,15,7,1,2,3,4,5,6,9,10,11,12,13,14]{
            let name=format!("selector{selector}-playing{playing}-scenario{scenario}-retire{slot}");let dir=work.join(&name);fs::create_dir(&dir).unwrap();
            fs::write(dir.join("before.bin"),snapshot(&world)).unwrap();
            let result=if selector==1{world.retire_family8_child(slot)}else{world.retire_family8_parent(slot,playing)}.unwrap();
            assert_eq!(result.stop_audio,if selector==1{vec![0x53]}else{vec![0x5e,0x5f]});
            assert_eq!(result.fade_audio,if playing{Some((0x38,0))}else{None});
            assert_eq!(world.family8().unwrap().binding(slot as usize).unwrap(),None);
            fs::write(dir.join("after.bin"),snapshot(&world)).unwrap();writeln!(ledger,"{name}\t{slot}\t{selector}\t{}",u8::from(playing)).unwrap();
            let stable=world.clone();assert!(if selector==1{world.retire_family8_child(slot)}else{world.retire_family8_parent(slot,playing)}.is_err());assert_eq!(world,stable);
        }
        assert_eq!(world.backing_bytes()[0x3559],0);
        // Clearing the typed binding must permit first-inactive slot reuse.
        let reused=world.create_family8(selector,0,point,25600).unwrap().unwrap();assert_eq!(reused.slot,0);
    }
}
}
