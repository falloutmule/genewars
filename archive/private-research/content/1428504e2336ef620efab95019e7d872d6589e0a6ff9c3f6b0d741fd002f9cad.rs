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
fn main(){let a:Vec<_>=std::env::args().collect();let p=Path::new(&a[1]);let out=Path::new(&a[2]);let mut g=runtime_scene::Game::open(&p.join("private/integration/game-runtime")).unwrap();g.landing_seed=Some(5428);g.pointer(Some((312,140)));g.left(true);g.left(false);g.finish_loading();g.pan(-897,-641);for _ in 0..189{g.tick()}g.select_actor(Some(0));g.build_order([320,234]);for _ in 0..100{g.tick()}g.start_selected_building_type(11).unwrap();for _ in 0..1000{g.tick()}capture(&g,out,"1289");g.select_actor(Some(0));g.build_order([400,370]);for step in 1290..=1489{g.tick();if step>=1479{capture(&g,out,&step.to_string());}}g.start_selected_building_type(3).unwrap();capture(&g,out,"1489-assigned");for step in 1490..=1510{g.tick();capture(&g,out,&step.to_string());}println!("captured1289,1479..1510,and1489assignment");}
