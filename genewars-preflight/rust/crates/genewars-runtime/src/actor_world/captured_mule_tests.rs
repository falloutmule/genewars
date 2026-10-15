//! Opt-in exact captured callback replay. This is not campaign acceptance.
use super::*;
use std::path::Path;
use crate::creature_world::{CreatureAnimationBinding, MuleAnimations, MuleStepContext};

fn records<const N: usize, const W: usize>(path: &Path) -> Box<[[u8; W]; N]> {
    let bytes=std::fs::read(path).unwrap();assert_eq!(bytes.len(),N*W,"capture extent: {}",path.display());
    let rows:Vec<[u8;W]>=bytes.chunks_exact(W).map(|r|r.try_into().unwrap()).collect();
    rows.into_boxed_slice().try_into().unwrap_or_else(|_|unreachable!())
}
fn buffers(world:&ActorWorld)->Vec<(&'static str,Vec<u8>)> {
    let mut rng=world.rng.seed.to_le_bytes().to_vec();rng.push(world.rng.ring);rng.extend(world.rng.draw_count.to_le_bytes());
    vec![("backing",world.backing.clone()),("actors",world.actors.records().concat()),
        ("creatures",world.creatures.as_ref().unwrap().records().concat()),
        ("effects",world.effects.as_ref().unwrap().records().concat()),
        ("family8",world.family8.as_ref().unwrap().records().concat()),
        ("buildings",world.buildings.as_ref().unwrap().records().concat()),
        ("players",world.players.iter().flat_map(|p|p.bytes().iter().copied()).collect()),("rng",rng)]
}
fn effect_binding(code:u8)->EffectAnimationBinding {
    match code {6=>EffectAnimationBinding::EffAnimation6,0x15=>EffectAnimationBinding::EffAnimation21,
        0x16=>EffectAnimationBinding::EffAnimation22,0x1a=>EffectAnimationBinding::EffAnimation26,
        0x20=>EffectAnimationBinding::EffAnimation32,0x21=>EffectAnimationBinding::EffAnimation33,
        0x2c=>EffectAnimationBinding::EffAnimation44,0x2f=>EffectAnimationBinding::EffAnimation47,
        0x30=>EffectAnimationBinding::EffAnimation48,0x31=>EffectAnimationBinding::EffAnimation49,
        _=>panic!("unproved captured effect binding {code:02x}")}
}
#[test]
#[ignore = "requires explicit immutable private capture and fresh output directory"]
fn captured_mule_callback() {
    let input=std::path::PathBuf::from(std::env::var_os("GENEWARS_CAPTURED_MULE_INPUT").expect("capture input required"));
    let output=std::path::PathBuf::from(std::env::var_os("GENEWARS_CAPTURED_MULE_OUTPUT").expect("fresh capture output required"));
    let frame:u32=std::env::var("GENEWARS_CAPTURED_MULE_FRAME").expect("source frame required").parse().unwrap();
    let slot:u16=std::env::var("GENEWARS_CAPTURED_MULE_SLOT").expect("source slot required").parse().unwrap();
    assert!(input.is_dir());assert!(!output.exists(),"never overwrite callback evidence");std::fs::create_dir(&output).unwrap();
    let resources=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../private/integration/game-runtime");
    let backing=std::fs::read(input.join("backing.bin")).unwrap();
    let actors=records::<20,200>(&input.join("actors.bin"));let creatures=records::<300,124>(&input.join("creatures.bin"));
    let effects=records::<200,81>(&input.join("effects.bin"));let family8=records::<16,102>(&input.join("family8.bin"));
    let buildings=records::<256,67>(&input.join("buildings.bin"));let player_records=records::<4,1805>(&input.join("players.bin"));
    let players=std::array::from_fn(|i|PlayerState::from_record(&player_records[i]).unwrap());
    let raw_rng=std::fs::read(input.join("rng.bin")).unwrap();assert_eq!(raw_rng.len(),7);
    let rng=BlueprintRng{seed:u16::from_le_bytes(raw_rng[..2].try_into().unwrap()),ring:raw_rng[2],draw_count:u32::from_le_bytes(raw_rng[3..].try_into().unwrap())};assert!(rng.ring<32);
    // Bootstrap only layout metadata from the pinned original mission. All
    // captured overlays and backing replace it before callback admission.
    let source_bytes=std::fs::read(resources.join("terrain/CONQ002.DAT")).unwrap();
    let source=ConqMissionSource::parse(&source_bytes).unwrap();assert_eq!(backing.len(),source_bytes.len());
    let mut world=ActorWorld::from_mission(&source,&actors,[ActorHandleBindings::empty();20],players,rng).unwrap();
    world.backing=backing;
    for (id,r) in actors.iter().enumerate().filter(|(_,r)|r[6]!=0) {
        assert_eq!(r[6],3);assert_eq!(word(r,15)as usize,id);assert!(r[24]<4);assert_eq!(id/5,usize::from(r[24]));
    }
    world.actors=ActorPool::from_captured_records(*actors,[ActorHandleBindings::empty();20]);
    let creature_bindings=std::array::from_fn(|i|{
        let r=&creatures[i];if r[6]==0{return None;}assert_eq!(r[6],1);assert_eq!(word(r,15)as usize,i);assert!(r[24]<=4);assert_eq!(r[0x7a],4);
        Some(if r[0x5e]>=26{CreatureAnimationBinding::BonesSelector4}else{CreatureAnimationBinding::MuleSelector4})
    });
    world.creatures=Some(Box::new(CreaturePool::from_captured_records(*creatures,creature_bindings)));
    let effect_bindings=std::array::from_fn(|i|{
        let r=&effects[i];if r[6]==0{return None;}assert_eq!(r[6],5);assert_eq!(word(r,15)as usize,i);assert!(r[24]<=4);Some(effect_binding(r[69]))
    });
    world.effects=Some(Box::new(EffectPool::from_captured_records(*effects,effect_bindings)));
    let family_bindings=std::array::from_fn(|i|{
        let r=&family8[i];if r[6]==0{return None;}assert_eq!(r[6],8);assert_eq!(word(r,15)as usize,i);assert!(r[24]<=4);
        Some(match r[0x55]{1=>Family8AnimationBinding::Base,0|2=>Family8AnimationBinding::Offset80,_=>panic!("unproved family8 binding")})
    });
    world.family8=Some(Box::new(Family8Pool::from_captured_records(*family8,family_bindings)));
    let building_bindings=std::array::from_fn(|i|(buildings[i][6]!=0).then_some(crate::building::BuildingAnimationBinding{selector:buildings[i][45]}));
    world.buildings=Some(Box::new(BuildingPool::admit(*buildings,building_bindings).unwrap()));
    world.admit_plants(&std::fs::read(resources.join("plant-parameters.bin")).unwrap()).unwrap();
    world.validate_links().unwrap();world.validate_effect_count().unwrap();world.validate_family8_count().unwrap();world.validate_building_counts().unwrap();
    assert_eq!(word(&world.backing,0x3550)as usize,world.creatures.as_ref().unwrap().records().iter().filter(|r|r[6]!=0).count());
    for (name,bytes) in buffers(&world){assert_eq!(bytes,std::fs::read(input.join(format!("{name}.bin"))).unwrap(),"capture roundtrip {name}");}
    let physics=crate::physics_tables::PhysicsTables::from_mapped_bytes(&std::fs::read(resources.join("physics/angle.bin")).unwrap(),&std::fs::read(resources.join("physics/bounce.bin")).unwrap()).unwrap()
        .with_corpse_friction(&std::fs::read(resources.join("physics/corpse-friction.bin")).unwrap()).unwrap();
    let trig=TrigTables::generate_pinned().unwrap();let animations=MuleAnimations::from_anb(&std::fs::read(resources.join("creatures/MULE.ANB")).unwrap()).unwrap();
    use crate::source_building_resources::{SourceBuildingResourceDatabase,SourceBuildingBinding};
    let building_resources=[(SourceBuildingBinding::CdBgeneSelector0,"BGENE"),(SourceBuildingBinding::CdBpoweSelector3,"BPOWE"),(SourceBuildingBinding::CdBmineSelector11,"BMINE")].into_iter()
        .map(|(binding,name)|SourceBuildingResourceDatabase::load(binding,std::fs::read(resources.join(format!("buildings/{name}.ANB"))).unwrap()).unwrap()).collect::<Vec<_>>();
    if std::env::var_os("GENEWARS_CAPTURED_MULE_NO_BUILDING_RESOURCES").is_none(){world.bind_building_collision_resources(&building_resources).unwrap();}
    let physics_context=world.mission_loading_physics();let climate=word(&world.backing,0x1c61);
    if let Ok(command)=std::env::var("GENEWARS_CAPTURED_MULE_COMMAND") {
        let point=command.split(',').map(str::parse::<i16>).collect::<Result<Vec<_>,_>>().unwrap();assert_eq!(point.len(),2);
        let export=|name:&str,w:&ActorWorld|{let path=output.join(name);std::fs::create_dir(&path).unwrap();for(name,bytes)in buffers(w){std::fs::write(path.join(format!("{name}.bin")),bytes).unwrap();}};
        export("command-before",&world);
        let receipt=world.dispatch_owned_mule_move(slot,[point[0],point[1]],crate::mule_owned_command::CommandContext{caller_owner:0,virtual_dead:world.creatures.as_ref().unwrap().record(usize::from(slot)).unwrap()[94]>=24,caller_is_selected:false,selected_panel_refresh:false}).unwrap();
        assert_eq!(receipt.outcome,crate::mule_owned_command::CommandOutcome::Installed);export("command-after",&world);
    }
    let steps=std::env::var("GENEWARS_CAPTURED_MULE_STEPS").ok().map(|v|v.parse::<u32>().unwrap()).unwrap_or(1);assert!((1..=512).contains(&steps));
    let before=world.clone();let mut final_frame=frame;let mut grounded=false;let mut pending_audio=Vec::new();
    let mut result:Result<Vec<u8>,ActorWorldError>=Err(ActorWorldError::Creatures(CreatureError::UnsupportedMotionState));
    for offset in 0..steps {
        final_frame=frame+offset;let prior=world.clone();
        let context=MuleStepContext{frame_counter:final_frame,gravity:physics_context.gravity,water_height:physics_context.water_height,climate};
        result=if world.creatures.as_ref().unwrap().record(usize::from(slot)).unwrap()[24]==4 {
            world.step_neutral_mule_slot_with_water(slot,context,&physics,&trig,&animations).map(|r|r.pending_audio)
        }else{world.step_owned_mule_slot(slot,0,0x3e,context,&physics,&trig,&animations).map(|r|r.pending_audio)};
        if result.is_err(){assert_eq!(world,prior,"failed callback must rollback every world field");break;}
        pending_audio.extend_from_slice(result.as_ref().unwrap());
        let record=world.creatures.as_ref().unwrap().record(usize::from(slot)).unwrap();
        grounded=i32::from_le_bytes(record[39..43].try_into().unwrap())<=i32::from_le_bytes(record[17..21].try_into().unwrap());
        if grounded&&std::env::var_os("GENEWARS_CAPTURED_MULE_STOP_GROUNDED").is_some(){break;}
    }
    let (status,error)=match &result {Ok(_)=>("passed",String::new()),Err(e)=>("blocked",format!("{e:?}"))};
    println!("captured Mule sequence: final_frame={final_frame}, grounded={grounded}, callbacks={}",final_frame-frame+1);
    if let Ok(expected_audio)=std::env::var("GENEWARS_CAPTURED_MULE_EXPECT_AUDIO") {
        let expected=expected_audio.split(',').map(str::parse::<u8>).collect::<Result<Vec<_>,_>>().unwrap();assert_eq!(pending_audio,expected);
    }
    println!("captured pending audio: {pending_audio:?}");
    world.validate_links().unwrap();
    for(name,bytes)in buffers(&world){std::fs::write(output.join(format!("{name}.bin")),bytes).unwrap();}
    let report=format!("{{\"status\":\"{status}\",\"frame\":{final_frame},\"slot\":{slot},\"input_roundtrip\":true,\"error\":{error:?},\"rng_draws\":{},\"scope\":\"captured Mule callbacks with other families frozen; not campaign acceptance\"}}\n",world.rng.draw_count.wrapping_sub(before.rng.draw_count));
    std::fs::write(output.join("result.json"),&report).unwrap();println!("{report}");
    let expected=std::env::var("GENEWARS_CAPTURED_MULE_EXPECT").unwrap_or_else(|_|"passed".into());
    match expected.as_str(){"passed"=>assert!(result.is_ok(),"captured callback blocked: {error}"),"blocked"=>assert!(result.is_err(),"expected unresolved boundary passed; update acceptance expectation"),_=>panic!("expected result must be passed or blocked")};
}
