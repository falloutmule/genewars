use genewars_assets::ConqMissionSource;
use genewars_runtime::{actor_world::ActorWorld,actor_pool::ActorHandleBindings,
    actor_activation::TrigTables,blueprint::BlueprintRng,player_state::PlayerState,
    physics_tables::PhysicsTables,creature_world::{MuleAnimations,MuleStepContext}};
use std::{fs,path::Path,io::Write};
fn main(){
    let root=std::env::args().nth(1).unwrap();let root=Path::new(&root);
    let work=root.join("private/review/family-scheduler");let inputs=fs::read(work.join("input.bin")).unwrap();
    let runtime=root.join("private/integration/game-runtime");
    let mission=fs::read(runtime.join("terrain/CONQ001.DAT")).unwrap();let source=ConqMissionSource::parse(&mission).unwrap();
    let physics=PhysicsTables::from_mapped_bytes(&fs::read(runtime.join("physics/angle.bin")).unwrap(),&fs::read(runtime.join("physics/bounce.bin")).unwrap()).unwrap();
    let trig=TrigTables::generate_pinned().unwrap();let animations=MuleAnimations::from_anb(&fs::read(runtime.join("creatures/MULE.ANB")).unwrap()).unwrap();
    let mut out=fs::File::create(work.join("actual.bin")).unwrap();
    for case in inputs.chunks_exact(3+4*1805) {
        let seed=u16::from_le_bytes([case[0],case[1]]);let player_count=case[2];
        let players=std::array::from_fn(|i|PlayerState::from_record(&case[3+i*1805..3+(i+1)*1805]).unwrap());
        let mut world=ActorWorld::from_mission(&source,&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed,ring:0,draw_count:0}).unwrap();
        world.admit_creatures(&[[0;124];300]).unwrap();
        let receipt=world.step_neutral_creature_callbacks(player_count,MuleStepContext{frame_counter:1,gravity:1024,water_height:1632,climate:106},&physics,&trig,&animations).unwrap();
        assert_eq!(receipt.primary_slots,vec![0]);world.step_empty_families_7_and_6().unwrap();
        for player in 0..4 {out.write_all(world.player(player).unwrap().bytes()).unwrap();}
        out.write_all(world.creatures().unwrap().record(0).unwrap()).unwrap();out.write_all(world.map()).unwrap();
        let rng=world.rng();out.write_all(&rng.seed.to_le_bytes()).unwrap();out.write_all(&[rng.ring]).unwrap();out.write_all(&rng.draw_count.to_le_bytes()).unwrap();
        for request in receipt.pending_ui_animations {out.write_all(&request.resource_global.to_le_bytes()).unwrap();out.write_all(&request.descriptor_offset.to_le_bytes()).unwrap();out.write_all(&request.state_global.to_le_bytes()).unwrap();}
    }
}
