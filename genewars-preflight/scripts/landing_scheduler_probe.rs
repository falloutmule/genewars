//! Actual-map family4 pass regression with explicit startup fixture inputs.
use genewars_assets::ConqMissionSource;
use genewars_runtime::{actor_world::{ActorWorld,LandingEffectCallback},actor_pool::ActorHandleBindings,
    blueprint::BlueprintRng,player_state::PlayerState,actor_activation::TrigTables,
    physics_tables::PhysicsTables,landing_effect_lifecycle::RetirementUiState};
use std::{fs,path::Path};
fn word(bytes:&[u8],offset:usize)->i16 {i16::from_le_bytes(bytes[offset..offset+2].try_into().unwrap())}
fn main(){
    let directory=std::env::args().nth(1).unwrap();let directory=Path::new(&directory);
    let input=fs::read(directory.join("mission.bin")).unwrap();let source=ConqMissionSource::parse(&input).unwrap();
    let tables=PhysicsTables::from_mapped_bytes(&fs::read(directory.join("angle.bin")).unwrap(),&fs::read(directory.join("bounce.bin")).unwrap()).unwrap();
    let trig=TrigTables::generate_pinned().unwrap();let ui=RetirementUiState::native_empty();
    println!("seed,passes,created,retired,max_active,contact_pass,rng_draws,quiet_timer,rollback");
    for seed in [0,1,5428,32768,65535] {
        let players=std::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,[20736,13568],b"fixture\0").unwrap());
        let mut world=ActorWorld::from_mission(&source,&[[0xA5;200];20],[ActorHandleBindings::empty();20],players,
            BlueprintRng{seed,ring:31,draw_count:0}).unwrap();
        world.admit_effects(&[[0xA5;81];200],[None;200]).unwrap();world.create_landing_core(0,0).unwrap();
        let context=world.mission_loading_physics();assert_eq!(context.water_height,1632);assert_eq!(context.gravity,1024);
        let(mut created,mut retired,mut max_active,mut contact)=(0,0,0,0);
        for pass in 1..=289 {
            let result=world.step_landing_effect_callbacks(context,&tables,&trig,&ui).unwrap();
            for callback in result.primary.iter().chain(&result.newborn) {
                match callback {
                    LandingEffectCallback::Landing(e)=>{created+=usize::from(e.created_effect.is_some());if e.contact{assert_eq!(contact,0);contact=pass;}},
                    LandingEffectCallback::Smoke(e)=>retired+=usize::from(e.retired),
                }
            }
            assert!(world.effects().unwrap().records().iter().all(|r|r[6]==0||r[21]&1==0));
            max_active=max_active.max(result.queue_candidates.len());
        }
        assert_eq!(created,188);assert_eq!(retired,188);assert_eq!(contact,189);assert!(max_active<60);
        assert_eq!(world.rng().draw_count,752);assert_eq!(word(world.backing_bytes(),0x3554),1);
        assert_eq!(world.player(0).unwrap().activation_count(),1);
        let before=world.clone();assert!(world.step_landing_effect_callbacks(context,&tables,&trig,&ui).is_err());assert_eq!(world,before);
        println!("{seed},289,{created},{retired},{max_active},{contact},{},{},true",world.rng().draw_count,word(world.effects().unwrap().record(0).unwrap(),74));
    }
}
