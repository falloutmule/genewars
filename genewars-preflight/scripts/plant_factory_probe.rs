use genewars_assets::ConqMissionSource;
use genewars_runtime::{actor_world::ActorWorld,actor_pool::ActorHandleBindings,player_state::PlayerState,blueprint::BlueprintRng};
use std::{fs,path::Path,io::Write};
fn main(){
    let root=std::env::args().nth(1).unwrap();let root=Path::new(&root);let work=std::env::args().nth(2).unwrap();let work=Path::new(&work);
    let mission=fs::read(root.join("private/integration/game-runtime/terrain/CONQ001.DAT")).unwrap();let source=ConqMissionSource::parse(&mission).unwrap();
    let players=core::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,[20736,13568],b"fixture\0").unwrap());
    let mut base=ActorWorld::from_mission(&source,&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed:5428,ring:0,draw_count:0}).unwrap();
    base.admit_creatures(&[[0;124];300]).unwrap();base.admit_plants(&fs::read(root.join("private/integration/game-runtime/plant-parameters.bin")).unwrap()).unwrap();
    let mut out=fs::File::create(work.join("actual.bin")).unwrap();
    for c in fs::read(work.join("input.bin")).unwrap().chunks_exact(5){
        let mut world=base.clone();let point=[i16::from_le_bytes(c[1..3].try_into().unwrap()),i16::from_le_bytes(c[3..5].try_into().unwrap())];
        let slot=world.create_neutral_plant(c[0],point).unwrap().unwrap();assert_eq!(world.rng(),base.rng());
        out.write_all(&slot.to_le_bytes()).unwrap();out.write_all(world.backing_bytes()).unwrap();out.write_all(world.creatures().unwrap().record(0).unwrap()).unwrap();
    }
    let mut bad=base.clone();assert!(bad.create_neutral_plant(12,[0,0]).is_err());assert_eq!(bad,base);
}
