//! Reproducible combined numeric trajectory. Requires private hash-bound source resources.
use genewars_assets::{ConqMissionSource,TextStringTable};
use genewars_runtime::{actor_world::ActorWorld,actor_pool::ActorHandleBindings,actor_activation::TrigTables,actor_resources::{ActorAnimationDatabase,actor_resource_spec},blueprint::{BlueprintRng,BlueprintNames},player_state::{PlayerState,ActorOriginSample},physics_tables::PhysicsTables,landing_effect_lifecycle::RetirementUiState,creature_world::{MuleStepContext,MuleAnimations},actor_motion::ActorIdleContext};
use std::{fs,path::Path};
fn word(b:&[u8],p:usize)->i16{i16::from_le_bytes(b[p..p+2].try_into().unwrap())}
fn capture_world(world:&ActorWorld,out:&Path){
    fs::create_dir_all(out).unwrap();
    fs::write(out.join("backing.bin"),world.backing_bytes()).unwrap();
    fs::write(out.join("creatures.bin"),world.creatures().unwrap().records().concat()).unwrap();
    fs::write(out.join("family8.bin"),world.family8().unwrap().records().concat()).unwrap();
    fs::write(out.join("actors.bin"),world.actors().records().concat()).unwrap();
    fs::write(out.join("effects.bin"),world.effects().unwrap().records().concat()).unwrap();
    fs::write(out.join("players.bin"),(0..4).flat_map(|p|world.player(p).unwrap().bytes().iter().copied()).collect::<Vec<_>>()).unwrap();
    let r=world.rng();fs::write(out.join("rng.bin"),[r.seed.to_le_bytes().as_slice(),&[r.ring],&r.draw_count.to_le_bytes()].concat()).unwrap();
}
fn main(){let base=std::env::args().nth(1).unwrap();let base=Path::new(&base);let with_mule=std::env::args().nth(2).as_deref()==Some("mule");let seed=std::env::args().nth(3).map(|s|s.parse::<u16>().unwrap()).unwrap_or(5428);let runtime=base.join("private/integration/game-runtime");let output=std::env::args().nth(5).expect("private output directory required");let output=Path::new(&output);
let bytes=fs::read(runtime.join("terrain/CONQ001.DAT")).unwrap();let src=ConqMissionSource::parse(&bytes).unwrap();let physics=PhysicsTables::from_mapped_bytes(&fs::read(runtime.join("physics/angle.bin")).unwrap(),&fs::read(runtime.join("physics/bounce.bin")).unwrap()).unwrap();let trig=TrigTables::generate_pinned().unwrap();
let text=fs::read(runtime.join("menu/text.dat")).unwrap();let strings=TextStringTable::parse(&text).unwrap();let names=BlueprintNames::new(&strings.strings()[..128]).unwrap();let players=std::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,[20736,13568],b"fixture\0").unwrap());
let mut world=ActorWorld::from_mission(&src,&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed,ring:0,draw_count:0}).unwrap();world.admit_creatures(&[[0;124];300]).unwrap();world.admit_effects(&[[0;81];200],[None;200]).unwrap();world.admit_family8(&[[0;102];16],[None;16]).unwrap();world.admit_plants(&fs::read(runtime.join("plant-parameters.bin")).unwrap()).unwrap();
world.regenerate_player_blueprints(0,[0;36],&names,true).unwrap();let origin=ActorOriginSample{position:[0,0],height:genewars_core::cd_candidate::interpolate_height(world.map(),0,0).unwrap(),shade:genewars_core::cd_candidate::interpolate_entity_shade(world.map(),0,0).unwrap()};world.prepare_fresh_player(0,5,origin,[20736,13568]).unwrap();world.create_landing_core(0,0).unwrap();world.apply_reveal_4096([20736,13568]).unwrap();
let context=world.mission_loading_physics();let mut ui=RetirementUiState::native_empty();let mule=MuleAnimations::from_anb(&fs::read(runtime.join("creatures/MULE.ANB")).unwrap()).unwrap();
let resources=(0..2).map(|id|{let key=world.actors().bindings(id).unwrap().animation_database.unwrap();let spec=actor_resource_spec(key).unwrap();ActorAnimationDatabase::load(key,fs::read(base.join(format!("private/integration/sprites/{}.anb",spec.decoded_sha256))).unwrap()).unwrap()}).collect::<Vec<_>>();
let plant_spread=genewars_runtime::plant_spread::SpreadDirections::from_mapped_bytes(&fs::read(runtime.join("physics/plant-spread.bin")).unwrap()).unwrap();
let family8_directions=genewars_runtime::family8_motion::Family8DirectionalAnimations::from_ethereal_anb(&fs::read(runtime.join("countdown/ETHEREAL.ANB")).unwrap()).unwrap();
let family8_animation=genewars_runtime::family8_motion::Family8Animation::from_ethereal_anb(&fs::read(runtime.join("countdown/ETHEREAL.ANB")).unwrap()).unwrap();
let max_frame=std::env::args().nth(4).map(|s|s.parse::<u32>().unwrap()).unwrap_or(450);
let capture_stride=std::env::args().nth(6).map(|s|s.parse::<u32>().unwrap()).unwrap_or(1);assert!(capture_stride>0);
for frame in 1..=max_frame {
    let result=(||->Result<(),String>{
        for player in 0..4u8 {let mut index=0usize;loop {let p=world.player(player).unwrap();if index>=p.actor_count()as usize{break;}let id=p.bytes()[4+index]as u16;index+=1;
            if world.actors().record(id as usize).unwrap()[0x70]&1==0{continue;}
            let idle=ActorIdleContext{runtime_mode:0,override_180d80:0,runtime_count_11c2bc:word(world.backing_bytes(),0x355c)as u16,player_mode:p.creation_mode()};
            world.step_actor_callback_with_idle(id,frame,context,&physics,&trig,&resources[id as usize],idle).map_err(|e|format!("actor{id} {e:?}"))?;
        }}
        world.step_empty_building_bookkeeping(0).map_err(|e|format!("building {e:?}"))?;
        if with_mule {
            world.step_neutral_creature_callbacks(4,MuleStepContext{frame_counter:frame,gravity:context.gravity,water_height:context.water_height,climate:word(world.backing_bytes(),0x1c61)},&physics,&trig,&mule).map_err(|e|format!("mule {e:?}"))?;
        }
        world.tick_plants_with_reproduction(&plant_spread).map_err(|e|format!("plants {e:?}"))?;
        let capture=output.join(format!("frame{frame}"));
        let sampled=frame>=6001&&(frame==6001||frame==max_frame||frame%capture_stride==0);
        let fallback=if frame>=6001&&!sampled{Some(world.clone())}else{None};
        if sampled {capture_world(&world,&capture.join("before"));}
        let result=world.step_family8_callbacks(0,4,0,frame,context.water_height,&physics,&trig,&family8_animation,&family8_directions);
        if sampled||result.is_err() {if let Some(ref before)=fallback{capture_world(before,&capture.join("before"));}capture_world(&world,&capture.join("after"));}
        result.map_err(|e|format!("family8 {e:?}"))?;
        world.step_empty_families_7_and_6().map_err(|e|format!("empty67 {e:?}"))?;
        ui=world.step_landing_effect_callbacks(context,&physics,&trig,&ui).map_err(|e|format!("effects {e:?}"))?.ui_after;
        Ok(())
    })();
    if let Err(e)=result{capture_world(&world,&output.join(format!("frame{frame}/boundary")));println!("STOP frame{frame}: {e}; rng={:?}",world.rng());return;}
    if frame%1000==0||frame==189{println!("frame{frame} rng={:?} count={} actors={:?} mule={:?}",world.rng(),word(world.backing_bytes(),0x355c),(0..2).map(|i|{let a=world.actors().record(i).unwrap();(a[0x55],word(a,7),word(a,9))}).collect::<Vec<_>>(),{let a=world.creatures().unwrap().record(0).unwrap();(word(a,7),word(a,9),a[0x19],a[0x1b])});}
}
println!("PASS{max_frame} mule={with_mule} seed={seed}");
}
