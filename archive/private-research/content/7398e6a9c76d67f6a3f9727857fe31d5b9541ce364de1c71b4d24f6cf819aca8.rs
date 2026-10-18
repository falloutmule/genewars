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
    println!("seed,passes,created,smoke_retired,departure_retired,max_active,contact_pass,landing_retired_pass,rng_draws,messages,active_after");
    for seed in [0,1,5428,32768,65535] {
        let players=std::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,[20736,13568],b"fixture\0").unwrap());
        let mut world=ActorWorld::from_mission(&source,&[[0xA5;200];20],[ActorHandleBindings::empty();20],players,
            BlueprintRng{seed,ring:31,draw_count:0}).unwrap();
        world.admit_effects(&[[0xA5;81];200],[None;200]).unwrap();world.create_landing_core(0,0).unwrap();
        world.admit_creatures(&[[0xA5;124];300]).unwrap();
        let context=world.mission_loading_physics();assert_eq!(context.water_height,1632);assert_eq!(context.gravity,1024);
        let(mut created,mut retired,mut departure_retired,mut max_active,mut contact,mut landing_retired,mut messages)=(0,0,0,0,0,0,0);
        for pass in 1..=450 {
            let result=world.step_landing_effect_callbacks(context,&tables,&trig,&ui).unwrap_or_else(|error| {
                eprintln!("seed={seed} pass={pass} error={error:?}");
                let mut probe=world.clone();
                for slot in 0..200u16 {
                    let r=*probe.effects().unwrap().record(usize::from(slot)).unwrap();if r[6]==0||r[21]&1!=0 { continue; }
                    let e=match r[69] {
                        6=>probe.step_effect6(slot,context,&tables,&trig,&ui).map(|_|()),
                        0x2F=>probe.step_effect2f(slot,context,&tables,&trig,&ui).map(|_|()),
                        0x31 if r[70]==0=>probe.step_landing_phase0(slot,context,&tables,&trig).map(|_|()),
                        0x31 if r[70]==1=>probe.step_landing_phase1(slot,context,&trig).map(|_|()),
                        _=>probe.step_landing_departure(slot,context,&tables,&trig,&ui).map(|_|()),
                    };
                    if let Err(e)=e {eprintln!("rejected primary slot={slot} {e:?}");fs::write(directory.join("failed-slot.bin"),r).unwrap();fs::write(directory.join("failed-map.bin"),probe.map()).unwrap();break;}
                }
                for(slot,r)in world.effects().unwrap().records().iter().enumerate().filter(|(_,r)|r[6]!=0) {
                    eprintln!("slot={slot} code={} phase={} pos={:?} height={} ground={} vel={:?} angles={:?} animation={:?}",r[69],r[70],[word(r,7),word(r,9)],i32::from_le_bytes(r[39..43].try_into().unwrap()),i32::from_le_bytes(r[17..21].try_into().unwrap()),[word(r,45),word(r,47),word(r,49)],[word(r,57),word(r,59),word(r,61)],&r[26..31]);
                }
                panic!("family4 composition rejected");
            });
            for callback in result.primary.iter().chain(&result.newborn) {
                match callback {
                    LandingEffectCallback::Landing(e)=>{
                        created+=usize::from(e.created_effect.is_some());if e.contact{assert_eq!(contact,0);contact=pass;}
                        if e.retirement.is_some(){assert_eq!(landing_retired,0);landing_retired=pass;}
                        messages+=e.pending_events.iter().filter(|e|matches!(e,genewars_runtime::landing::LandingEvent::LandingMessage{owner:0,message_index:79..=86})).count();
                    },
                    LandingEffectCallback::Smoke(e)=>retired+=usize::from(e.retired),
                    LandingEffectCallback::EtherealParticle(e)|LandingEffectCallback::DepartureParticle(e)=>departure_retired+=usize::from(e.retired),
                    LandingEffectCallback::Construction(_)|LandingEffectCallback::WaterSplash(_)=>panic!("water splash is outside this landing-only fixture"),
                }
            }
            assert!(world.effects().unwrap().records().iter().all(|r|r[6]==0||r[21]&1==0));
            max_active=max_active.max(result.queue_candidates.len());
        }
        assert_eq!(created,285);assert_eq!(retired,266);assert_eq!(departure_retired,19);assert_eq!(contact,189);assert_eq!(landing_retired,389);assert_eq!(messages,1);assert!(max_active<60);
        assert_eq!(world.rng().draw_count,1122);assert_eq!(word(world.backing_bytes(),0x3554),0);
        assert_eq!(world.player(0).unwrap().activation_count(),1);
        assert_eq!(world.player(0).unwrap().bytes()[0x498],0);assert_eq!(&world.player(0).unwrap().bytes()[0x499..0x49D],&700u32.to_le_bytes());
        println!("{seed},450,{created},{retired},{departure_retired},{max_active},{contact},{landing_retired},{},{messages},0",world.rng().draw_count);
    }
}
