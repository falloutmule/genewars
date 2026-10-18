//! Callback transaction fixtures on the exact map; not the full scheduler.
use genewars_assets::ConqMissionSource;
use genewars_runtime::{actor_world::ActorWorld,actor_pool::ActorHandleBindings,
    blueprint::BlueprintRng,player_state::PlayerState,actor_activation::TrigTables,
    physics_tables::PhysicsTables,effect_pool::EffectAnimationBinding};
use std::{fs,io::{BufWriter,Write},path::Path};
fn snapshot(out:&mut impl Write,world:&ActorWorld) {
    out.write_all(world.backing_bytes()).unwrap();
    for r in world.actors().records(){out.write_all(r).unwrap();}
    for p in 0..4{out.write_all(world.player(p).unwrap().bytes()).unwrap();}
    let r=world.rng();out.write_all(&r.seed.to_le_bytes()).unwrap();out.write_all(&[r.ring]).unwrap();out.write_all(&r.draw_count.to_le_bytes()).unwrap();
    for r in world.effects().unwrap().records(){out.write_all(r).unwrap();}
    for slot in 0..200 {
        out.write_all(&[match world.effects().unwrap().binding(slot).unwrap(){None=>0,Some(EffectAnimationBinding::EffAnimation49)=>49,Some(EffectAnimationBinding::EffAnimation6)=>6,Some(EffectAnimationBinding::EffAnimation47)=>47,Some(EffectAnimationBinding::EffAnimation48)=>48,Some(EffectAnimationBinding::EffAnimation26)=>26,Some(EffectAnimationBinding::EffAnimation21)=>21,Some(EffectAnimationBinding::EffAnimation22)=>22,Some(EffectAnimationBinding::EffAnimation44)=>44}]).unwrap();
    }
}
fn main(){
    let directory=std::env::args().nth(1).unwrap();let directory=Path::new(&directory);
    let input=fs::read(directory.join("mission.bin")).unwrap();let source=ConqMissionSource::parse(&input).unwrap();
    let tables=PhysicsTables::from_mapped_bytes(&fs::read(directory.join("angle.bin")).unwrap(),&fs::read(directory.join("bounce.bin")).unwrap()).unwrap();
    let trig=TrigTables::generate_pinned().unwrap();
    let mut output=BufWriter::new(fs::File::create(directory.join("actual.bin")).unwrap());
    for case in 0..16usize {
        let point=[20736+case as i16*7,13568-case as i16*9];
        let players=std::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,point,b"fixture\0").unwrap());
        let mut world=ActorWorld::from_mission(&source,&[[0xA5;200];20],[ActorHandleBindings::empty();20],players,
            BlueprintRng{seed:(case as u16).wrapping_mul(8191),ring:31,draw_count:u32::MAX-case as u32}).unwrap();
        world.admit_effects(&[[0xA5;81];200],[None;200]).unwrap();world.create_landing_core(0,0).unwrap();
        let context=world.mission_loading_physics();assert_eq!(context.water_height,1632);assert_eq!(context.gravity,1024);
        snapshot(&mut output,&world);
        for step in 0..8u16 {
            let event=world.step_landing_phase0(0,context,&tables,&trig).unwrap();
            assert!(!event.contact);assert_eq!(event.created_effect,Some(step+1));assert_eq!(event.draws.len(),4);assert_eq!(event.pending_events.len(),2);
            assert_eq!(event.draws.iter().map(|d|d.journal_code).collect::<Vec<_>>(),[0x7C8,0x7C8,0x7C8,0x21D]);
            snapshot(&mut output,&world);
        }
    }
    output.flush().unwrap();
}
