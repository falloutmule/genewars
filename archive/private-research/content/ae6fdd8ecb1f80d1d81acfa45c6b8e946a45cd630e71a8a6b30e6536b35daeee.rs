use genewars_runtime::{actor_world::ActorWorld,actor_pool::ActorHandleBindings,player_state::PlayerState,blueprint::BlueprintRng,creature_world::{MuleStepContext,MuleAnimations},actor_activation::TrigTables,physics_tables::PhysicsTables};
use std::{fs,path::PathBuf};
fn main(){
 let root=PathBuf::from(std::env::args().nth(1).unwrap());let out=PathBuf::from(std::env::args().nth(2).unwrap());
 let mission=fs::read(root.join("private/ghidra/campaign-resources/152.dat")).unwrap();let source=genewars_assets::ConqMissionSource::parse(&mission).unwrap();
 let object1=fs::read(root.join("private/ghidra/validated/memory-.object1.bin")).unwrap();let object3=fs::read(root.join("private/ghidra/validated/memory-.object3.bin")).unwrap();
 let physics=PhysicsTables::from_mapped_bytes(&object1[0x9e134..0x9e134+514],&object3[0x4530..0x4540]).unwrap();let trig=TrigTables::generate_pinned().unwrap();let animations=MuleAnimations::from_anb(&fs::read(root.join("private/reference-run/run-006/drive-c/GAME/DATA/CREATURE/MULE.ANB")).unwrap()).unwrap();
 for case in 0..3 {
 let mut w=ActorWorld::from_mission(&source,&[[0;200];20],[ActorHandleBindings::empty();20],core::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,[20736,13568],b"Player\0").unwrap()),BlueprintRng{seed:10548,ring:0,draw_count:16919}).unwrap();
 w.admit_creatures(&[[0;124];300]).unwrap();w.admit_buildings(&[[0;67];256],[None;256]).unwrap();w.admit_effects(&[[0;81];200],[None;200]).unwrap();w.admit_family8(&[[0;102];16],[None;16]).unwrap();
 if case==1 {w.initialize_mission_visibility(0).unwrap();}
 for frame in (if case==2 {1}else{0})..4 {let case=case*4+frame;
 let save_rng=|label:&str,w:&ActorWorld|{let rng=w.rng();let mut bytes=Vec::new();bytes.extend(rng.seed.to_le_bytes());bytes.push(rng.ring);bytes.extend(rng.draw_count.to_le_bytes());fs::write(out.join(format!("{case}-{label}-rng.bin")),bytes).unwrap();};
 save_rng("before",&w);
 fs::write(out.join(format!("{case}-before-backing.bin")),w.backing_bytes()).unwrap();fs::write(out.join(format!("{case}-before-native.bin")),w.creatures().unwrap().record(0).unwrap()).unwrap();
 w.step_neutral_creature_callbacks(1,MuleStepContext{frame_counter:frame,gravity:1024,water_height:1632,climate:106},&physics,&trig,&animations).unwrap();
 fs::write(out.join(format!("{case}-after-map.bin")),w.map()).unwrap();fs::write(out.join(format!("{case}-after-native.bin")),w.creatures().unwrap().record(0).unwrap()).unwrap();save_rng("after",&w);
 }
 }
}
