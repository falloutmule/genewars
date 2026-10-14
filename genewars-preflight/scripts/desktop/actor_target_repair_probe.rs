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
#[path="support/result_scene.rs"]mod result_scene;
fn read(p:&Path)->Result<Vec<u8>,String>{runtime_resources::read_bound(p)}
fn capture(g:&runtime_scene::Game,out:&Path,label:&str){let dir=out.join(label);fs::create_dir(&dir).unwrap();fs::write(dir.join("report.json"),g.report()).unwrap();let m=g.mission.as_ref().unwrap();let runtime_scene::MissionState::Landing(s)=&m.world else{panic!()};let w=&s.world;
 fs::write(dir.join("backing.bin"),w.backing_bytes()).unwrap();fs::write(dir.join("map.bin"),w.map()).unwrap();
 for (name,data) in [("actors",w.actors().records().iter().flatten().copied().collect::<Vec<_>>()),("creatures",w.creatures().unwrap().records().iter().flatten().copied().collect()),("buildings",w.buildings().unwrap().records().iter().flatten().copied().collect()),("effects",w.effects().unwrap().records().iter().flatten().copied().collect()),("family8",w.family8().unwrap().records().iter().flatten().copied().collect())]{fs::write(dir.join(format!("{name}.bin")),data).unwrap();}
 fs::write(dir.join("actor0.bin"),w.actors().record(0).unwrap()).unwrap();let mut players=Vec::new();for p in 0..4{players.extend(w.player(p).unwrap().bytes())}fs::write(dir.join("players.bin"),players).unwrap();let rng=w.rng();let mut r=rng.seed.to_le_bytes().to_vec();r.push(rng.ring);r.extend(rng.draw_count.to_le_bytes());fs::write(dir.join("rng.bin"),r).unwrap();fs::write(dir.join("jobs.bin"),s.fixture_jobs()).unwrap();
}
fn main(){let a:Vec<_>=std::env::args().collect();let p=Path::new(&a[1]);let out=Path::new(&a[2]);let mut g=runtime_scene::Game::open(&p.join("private/integration/game-runtime")).unwrap();g.reference_startup=true;g.pointer(Some((312,140)));g.left(true);g.left(false);g.finish_loading();g.pan(-897,-641);for _ in 0..200{g.tick()}g.select_actor(Some(0));g.place_selected_building_type(11,[320,234]).unwrap();for _ in 0..1100{g.tick()}g.place_selected_building_type(3,[400,370]).unwrap();for _ in 0..1200{g.tick()}g.place_selected_building_type(6,[430,350]).unwrap();for _ in 0..30{g.tick()}capture(&g,out,"2530");g.place_selected_building_type(6,[468,402]).unwrap();for step in 2531..=2640{
 if step==2637{
  let m=g.mission.as_mut().unwrap();let runtime_scene::MissionState::Landing(shared)=&mut m.world else{panic!()};
  let saved_world=shared.world.clone();let physics=genewars_runtime::physics_tables::PhysicsTables::from_mapped_bytes(&fs::read(p.join("private/integration/game-runtime/physics/angle.bin")).unwrap(),&fs::read(p.join("private/integration/game-runtime/physics/bounce.bin")).unwrap()).unwrap();let trig=genewars_runtime::actor_activation::TrigTables::generate_pinned().unwrap();
  let(candidate,target)=shared.world.actor_motion_route_candidate(0,2636,&physics,&trig,false).unwrap();fs::write(out.join("route-actor.bin"),candidate).unwrap();fs::write(out.join("route-target.bin"),target.iter().flat_map(|v|v.to_le_bytes()).collect::<Vec<_>>()).unwrap();
  let key=shared.world.actors().bindings(0).unwrap().animation_database.unwrap();let resource=genewars_runtime::actor_resources::ActorAnimationDatabase::load(key,fs::read(p.join(format!("private/integration/game-runtime/actors/key-{:02}.anb",key.flat_index))).unwrap()).unwrap();
  let bytes=shared.fixture_jobs();let mut jobs=genewars_runtime::terrain_job_pool::TerrainJobPool::admit(core::array::from_fn(|i|bytes[1+i*176..1+(i+1)*176].try_into().unwrap()),bytes[0]).unwrap();
  let patterns=fs::read(p.join("private/integration/game-runtime/construction/terrain-patterns.bin")).unwrap();let border=genewars_runtime::terrain_border_table::build(&patterns).unwrap();
  let result=shared.world.step_architect_construction_callback(0,2636,genewars_runtime::actor_world::LandingPhysicsContext{gravity:1024,water_height:1632},&physics,&trig,&mut jobs,&border,&resource,true,true);
  fs::write(out.join("isolated-result.txt"),format!("{result:?}")).unwrap();capture(&g,out,"2636-callback");let runtime_scene::MissionState::Landing(shared)=&mut g.mission.as_mut().unwrap().world else{panic!()};shared.world=saved_world;
 }
 g.tick();if step>=2634{capture(&g,out,&step.to_string());}if g.mission.as_ref().unwrap().blocked.is_some(){break;}}println!("captured2636 target repair boundary");}
