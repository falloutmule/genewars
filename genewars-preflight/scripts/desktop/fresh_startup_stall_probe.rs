use std::{fs,path::Path};
#[path="support/menu_art.rs"]mod menu_art;
#[path="support/runtime_resources.rs"]mod runtime_resources;
#[path="support/actor_scene.rs"]mod actor_scene;
#[path="support/building_scene.rs"]mod building_scene;
#[path="support/family8_scene.rs"]mod family8_scene;
#[path="support/effect_scene.rs"]mod effect_scene;
#[path="support/shared_scene.rs"]mod shared_scene;
#[path="support/runtime_scene.rs"]mod runtime_scene;
#[path="support/status_text.rs"]mod status_text;
fn read(p:&Path)->Result<Vec<u8>,String>{runtime_resources::read_bound(p)}
fn capture(g:&runtime_scene::Game,out:&Path,label:&str){let dir=out.join(label);fs::create_dir(&dir).unwrap();fs::write(dir.join("report.json"),g.report()).unwrap();let m=g.mission.as_ref().unwrap();let runtime_scene::MissionState::Landing(s)=&m.world else{panic!()};let w=&s.world;
 fs::write(dir.join("backing.bin"),w.backing_bytes()).unwrap();fs::write(dir.join("map.bin"),w.map()).unwrap();
 for (name,data) in [("actors",w.actors().records().iter().flatten().copied().collect::<Vec<_>>()),("creatures",w.creatures().unwrap().records().iter().flatten().copied().collect()),("buildings",w.buildings().unwrap().records().iter().flatten().copied().collect()),("effects",w.effects().unwrap().records().iter().flatten().copied().collect()),("family8",w.family8().unwrap().records().iter().flatten().copied().collect())]{fs::write(dir.join(format!("{name}.bin")),data).unwrap();}
 fs::write(dir.join("actor0.bin"),w.actors().record(0).unwrap()).unwrap();let mut players=Vec::new();for p in 0..4{players.extend(w.player(p).unwrap().bytes())}fs::write(dir.join("players.bin"),players).unwrap();let rng=w.rng();let mut r=rng.seed.to_le_bytes().to_vec();r.push(rng.ring);r.extend(rng.draw_count.to_le_bytes());fs::write(dir.join("rng.bin"),r).unwrap();fs::write(dir.join("jobs.bin"),s.fixture_jobs()).unwrap();
}
fn main(){let a:Vec<_>=std::env::args().collect();let p=Path::new(&a[1]);let out=Path::new(&a[2]);let mut g=runtime_scene::Game::open(&p.join("private/integration/game-runtime")).unwrap();g.reference_startup=true;g.pointer(Some((312,140)));g.left(true);g.left(false);g.finish_loading();for _ in 0..95{g.tick()}capture(&g,out,"95");
 let m=g.mission.as_mut().unwrap();let runtime_scene::MissionState::Landing(s)=&mut m.world else{panic!()};
 let physics=genewars_runtime::physics_tables::PhysicsTables::from_mapped_bytes(&fs::read(p.join("private/integration/game-runtime/physics/angle.bin")).unwrap(),&fs::read(p.join("private/integration/game-runtime/physics/bounce.bin")).unwrap()).unwrap();let trig=genewars_runtime::actor_activation::TrigTables::generate_pinned().unwrap();let animations=genewars_runtime::creature_world::MuleAnimations::from_anb(&fs::read(p.join("private/reference-run/run-006/drive-c/GAME/DATA/CREATURE/MULE.ANB")).unwrap()).unwrap();
 s.world.step_mule_followup_with_water(genewars_runtime::creature_world::MuleStepContext{frame_counter:95,gravity:1024,water_height:1632,climate:106},&physics,&trig,&animations).unwrap();capture(&g,out,"95-callback");println!("captured freshstartup95 callback");}
