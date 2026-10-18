//! Exact-source factory integration with explicit constructor-gap fixtures.
use genewars_assets::ConqMissionSource;
use genewars_runtime::{actor_world::ActorWorld,actor_pool::ActorHandleBindings,
    blueprint::BlueprintRng,player_state::PlayerState};
use std::{fs,io::{BufWriter,Write},path::Path};

fn snapshot(out: &mut impl Write, world: &ActorWorld) {
    out.write_all(world.backing_bytes()).unwrap();
    for actor in world.actors().records() { out.write_all(actor).unwrap(); }
    for player in 0..4 { out.write_all(world.player(player).unwrap().bytes()).unwrap(); }
    let rng=world.rng();
    out.write_all(&rng.seed.to_le_bytes()).unwrap();
    out.write_all(&[rng.ring]).unwrap();
    out.write_all(&rng.draw_count.to_le_bytes()).unwrap();
    for effect in world.effects().unwrap().records() { out.write_all(effect).unwrap(); }
    for slot in 0..200 { out.write_all(&[u8::from(world.effects().unwrap().binding(slot).unwrap().is_some())]).unwrap(); }
}
fn word(bytes: &[u8],offset:usize)->i16 { i16::from_le_bytes([bytes[offset],bytes[offset+1]]) }
fn main() {
    let directory=std::env::args().nth(1).unwrap();let directory=Path::new(&directory);
    let input=fs::read(directory.join("mission.bin")).unwrap();
    let source=ConqMissionSource::parse(&input).unwrap();
    let plants=source.groups()[1].bytes().chunks_exact(41).filter(|r|r[6]==2).collect::<Vec<_>>();
    let mut out=BufWriter::new(fs::File::create(directory.join("actual.bin")).unwrap());
    for case in 0..32_usize {
        let point=if case<16 { [20736+case as i16*127,13568-case as i16*71] }
            else { let record=plants[case-16];[word(record,7),word(record,9)] };
        let players=std::array::from_fn(|id|PlayerState::fresh_campaign(id as u8,point,b"fixture\0").unwrap());
        let priors=std::array::from_fn(|id|std::array::from_fn(|offset|(case+id*37+offset*19)as u8));
        let mut world=ActorWorld::from_mission(&source,&priors,[ActorHandleBindings::empty();20],players,
            BlueprintRng{seed:case as u16*193,ring:case as u8,draw_count:case as u32}).unwrap();
        let effect_priors=std::array::from_fn(|id|std::array::from_fn(|offset|(case*3+id*7+offset*29)as u8));
        world.admit_effects(&effect_priors,[None;200]).unwrap();
        snapshot(&mut out,&world);
        for player in 0..4 {
            let receipt=world.create_landing_core(player,(case%4)as u8).unwrap();
            assert_eq!(receipt.slot,u16::from(player));
            assert_eq!(receipt.pending_events.len(),if player==(case%4)as u8{2}else{1});
            snapshot(&mut out,&world);
            let before=world.clone();
            assert!(world.create_landing_core(player,(case%4)as u8).is_err());
            assert_eq!(world,before);
        }
    }
    out.flush().unwrap();
}
