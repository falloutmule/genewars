use std::{fs,path::Path};
fn records<const N:usize,const W:usize>(p:&Path)->[[u8;W];N]{let b=fs::read(p).unwrap();assert_eq!(b.len(),N*W);std::array::from_fn(|i|b[i*W..(i+1)*W].try_into().unwrap())}
pub fn review_relocation(p:&Path,d:&Path,w:&Path){
 let b=fs::read(d.join("backing.bin")).unwrap();let source=ConqMissionSource::parse(&b).unwrap();
 let players_raw=records::<4,1805>(&d.join("players.bin"));let players=std::array::from_fn(|i|PlayerState::from_record(&players_raw[i]).unwrap());
 let actor_records=records::<20,200>(&d.join("actors.bin"));
 let bindings=std::array::from_fn(|i|{let a=actor_records[i];if a[6]!=3{return ActorHandleBindings::empty()};let index=a[0x74];let bp=&players_raw[a[24]as usize][0x16+index as usize*36..];let key=crate::actor_initialization::ActorResourceTableKey{style:bp[0x15],category:bp[0x16],flat_index:4*bp[0x15]+bp[0x16]};ActorHandleBindings{blueprint:Some(crate::actor_initialization::BlueprintRecordHandle{player_id:a[24],blueprint_index:index}),resource_database:Some(key),animation_database:Some(key),internal_actor_offset:Some(0xc4)}});
 let r=fs::read(d.join("rng.bin")).unwrap();let rng=BlueprintRng{seed:u16::from_le_bytes(r[..2].try_into().unwrap()),ring:r[2],draw_count:u32::from_le_bytes(r[3..7].try_into().unwrap())};
 let mut world=ActorWorld{backing:b.clone(),layouts:source.groups().map(|g|g.layout()),actors:ActorPool{records:actor_records,bindings},players,rng,plants:None,effects:Some(Box::new(EffectPool{records:records::<200,81>(&d.join("effects.bin")),bindings:[None;200]})),creatures:Some(Box::new(CreaturePool{records:records::<300,124>(&d.join("creatures.bin")),bindings:[None;300]})),family8:Some(Box::new(Family8Pool{records:records::<16,102>(&d.join("family8.bin")),bindings:[None;16]})),buildings:None};
 world.validate_links().unwrap();
 let t=p.join("private/review/family8-state4");let physics=crate::physics_tables::PhysicsTables::from_mapped_bytes(&fs::read(t.join("angles.bin")).unwrap(),&fs::read(t.join("bounce.bin")).unwrap()).unwrap();let trig=TrigTables::generate_pinned().unwrap();
 let key=bindings[1].animation_database.unwrap();let hash=crate::actor_resources::actor_resource_spec(key).unwrap().decoded_sha256;
 let resource=crate::actor_resources::ActorAnimationDatabase::load(key,fs::read(p.join(format!("private/integration/sprites/{hash}.anb"))).unwrap()).unwrap();
 let context=LandingPhysicsContext{water_height:0,gravity:1024};let idle=crate::actor_motion::ActorIdleContext{runtime_mode:0,override_180d80:0,runtime_count_11c2bc:word(&b,0x355c)as u16,player_mode:players_raw[0][2]};
 let before=world.clone();
 let report=world.step_actor_callback_with_idle(1,14111,context,&physics,&trig,&resource,idle).unwrap();
 fs::write(w.join("rust-callback.bin"),world.actors.records[1]).unwrap();
 let after_actor=world.actors.records[1];world.actors.records[1]=before.actors.records[1];assert_eq!(world,before,"all other world state must remain unchanged");world.actors.records[1]=after_actor;
 assert_eq!(report.rng_before,report.rng_after);assert!(report.acknowledgement.is_some());
 let mut bad=before.clone();bad.actors.records[1][0x16]|=0x20;let unchanged=bad.clone();assert!(bad.step_actor_callback_with_idle(1,14111,context,&physics,&trig,&resource,idle).is_err());assert_eq!(bad,unchanged);
 println!("passed saved-world callback, whole-world preservation, RNG, acknowledgement and rollback");
}
