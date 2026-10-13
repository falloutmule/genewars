use genewars_assets::ConqMissionSource;
use genewars_runtime::{MissionWorld,blueprint::BlueprintRng,plant_spread::SpreadDirections,world::TickEvent,
    actor_world::ActorWorld,actor_pool::ActorHandleBindings,player_state::PlayerState};
use std::{fs,path::Path,io::Write};
fn rng_bytes(r:BlueprintRng)->Vec<u8>{let mut b=r.seed.to_le_bytes().to_vec();b.push(r.ring);b.extend(r.draw_count.to_le_bytes());b}
fn main(){
    let args:Vec<_>=std::env::args().collect();let root=Path::new(&args[1]);let out=Path::new(&args[2]);
    let data=fs::read(root.join("private/integration/game-runtime/terrain/CONQ001.DAT")).unwrap();
    let parameters=fs::read(root.join("private/integration/game-runtime/plant-parameters.bin")).unwrap();
    let mapped=fs::read(root.join("private/ghidra/validated/memory-.object3.bin")).unwrap();
    let directions=SpreadDirections::from_mapped_bytes(&mapped[0x4564..0x4588]).unwrap();
    let source=ConqMissionSource::parse(&data).unwrap();let mut ledger=fs::File::create(out.join("cases.tsv")).unwrap();
    for seed in [5428,0,65535] {
        let mut world=MissionWorld::from_source(&source,&parameters).unwrap();let mut rng=BlueprintRng{seed,ring:31,draw_count:u32::MAX};let mut attempts=0;
        let players=core::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,[20736,13568],b"fixture\0").unwrap());
        let mut shared=ActorWorld::from_mission(&source,&[[0;200];20],[ActorHandleBindings::empty();20],players,rng).unwrap();
        shared.admit_creatures(&[[0;124];300]).unwrap();shared.admit_plants(&parameters).unwrap();
        for tick in 1..=4000 {
            let before=world.source_bytes().to_vec();let before_rng=rng;
            let report=world.tick_plants_with_reproduction(&mut rng,&directions,816).unwrap();
            let shared_report=shared.tick_plants_with_reproduction(&directions).unwrap();
            assert_eq!(report,shared_report);assert_eq!(rng,shared.rng());
            if rng!=before_rng||tick==1100||tick==1102 {
                let mut shared_packed=shared.backing_bytes().to_vec();
                for (slot,record)in shared.creatures().unwrap().records().iter().enumerate(){
                    let at=243038+slot*116;shared_packed[at..at+31].copy_from_slice(&record[..31]);shared_packed[at+31..at+116].copy_from_slice(&record[39..]);
                }
                assert_eq!(shared_packed,world.source_bytes(),"shared/packed storage seed{seed} tick{tick}");
                let name=format!("seed{seed}-tick{tick}");let dir=out.join(&name);fs::create_dir(&dir).unwrap();
                fs::write(dir.join("before.bin"),before).unwrap();fs::write(dir.join("after.bin"),world.source_bytes()).unwrap();
                fs::write(dir.join("rng-before.bin"),rng_bytes(before_rng)).unwrap();fs::write(dir.join("rng-after.bin"),rng_bytes(rng)).unwrap();
                let mut queue=Vec::new();for i in &report.queued_indices{queue.extend((*i as u16).to_le_bytes());}fs::write(dir.join("queue.bin"),queue).unwrap();
                let created=report.events.iter().filter(|e|matches!(e,TickEvent::Created{..})).count();
                writeln!(ledger,"{name}\t{tick}\t{created}\t{}",report.active_plant_count_after).unwrap();
                assert!(!report.suspended.family1_creation_pass);if rng!=before_rng{attempts+=1;}if attempts==12{break;}
            }
        }
        assert_eq!(attempts,12);
    }
}
