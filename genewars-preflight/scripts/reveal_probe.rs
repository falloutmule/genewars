//! Exact mission fixture with explicit native priors; reveal numeric afterimage.
use genewars_assets::ConqMissionSource;
use genewars_runtime::{actor_world::ActorWorld,actor_pool::ActorHandleBindings,
    blueprint::BlueprintRng,player_state::PlayerState};
use std::{fs,path::Path};
fn main(){
    let directory=std::env::args().nth(1).unwrap();let directory=Path::new(&directory);
    let input=fs::read(directory.join("mission.bin")).unwrap();let source=ConqMissionSource::parse(&input).unwrap();
    let targets=fs::read(directory.join("targets.bin")).unwrap();let mut out=Vec::new();
    for target in targets.chunks_exact(4) {
        let target=[i16::from_le_bytes(target[..2].try_into().unwrap()),i16::from_le_bytes(target[2..].try_into().unwrap())];
        let players=std::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,target,b"fixture\0").unwrap());
        let mut world=ActorWorld::from_mission(&source,&[[0xa5;200];20],[ActorHandleBindings::empty();20],players,
            BlueprintRng{seed:5428,ring:31,draw_count:12345}).unwrap();
        world.admit_creatures(&[[0x5a;124];300]).unwrap();
        let before=world.clone();let receipt=world.apply_reveal_4096(target).unwrap();
        assert_eq!(world.actors(),before.actors());assert_eq!(world.creatures(),before.creatures());assert_eq!(world.rng(),before.rng());
        for p in 0..4 {assert_eq!(world.player(p).unwrap(),before.player(p).unwrap());}
        assert_eq!(receipt.cell_visits,4*receipt.changes.len());
        out.extend(world.backing_bytes());
        for n in [receipt.changes.len(),receipt.cell_visits,receipt.plant_shade_writes] {out.extend((n as u32).to_le_bytes());}
        let stable=world.clone();assert!(world.apply_reveal_4096(target).unwrap().changes.is_empty());assert_eq!(world,stable);
    }
    fs::write(directory.join("native.bin"),out).unwrap();
}
