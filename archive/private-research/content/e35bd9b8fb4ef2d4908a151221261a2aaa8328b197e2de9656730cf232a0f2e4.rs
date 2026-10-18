use genewars_runtime::{blueprint,player_state};
#[path="../../rust/crates/genewars-runtime/src/mission_bootstrap.rs"]mod mission_bootstrap;
use genewars_assets::TextStringTable;
use genewars_core::cd_candidate::{interpolate_height,interpolate_entity_shade};
use player_state::{PlayerState,ActorOriginSample};
fn main(){
 let args:Vec<_>=std::env::args().collect();let mission=std::fs::read(&args[1]).unwrap();let text=std::fs::read(&args[2]).unwrap();let dir=std::path::Path::new(&args[3]);
 let strings=TextStringTable::parse(&text).unwrap();let names=blueprint::BlueprintNames::new(&strings.strings()[..128]).unwrap();
 let target=[20736,13568];let retained=PlayerState::fresh_campaign(0,target,b"fixture\0").unwrap();
 let restored=PlayerState::restored_for_mission(&mission[20..1825],0,&retained).unwrap();let mut loaded=*restored.bytes();loaded[0x4a3..0x4a7].copy_from_slice(&2000u32.to_le_bytes());let loaded=PlayerState::from_record(&loaded).unwrap();
 let map=&mission[0x355e..0x3b55e];let origin=ActorOriginSample{position:[0,0],height:interpolate_height(map,0,0).unwrap(),shade:interpolate_entity_shade(map,0,0).unwrap()};
 for case in 0..4{let fill=if case&1==0{0}else{0xa5};let ring=if case&2==0{0}else{31};let mut priors=[[fill;200];5];for a in &mut priors{a[6]=0;}
 let plan=mission_bootstrap::prepare_player(&loaded,&names,[fill;36],blueprint::BlueprintRng{seed:5428,ring,draw_count:16855},&priors,mission[0x1ebd],origin,target,0).unwrap();
 assert_eq!(plan.generated.draws.len(),64);assert_eq!(plan.generated.rng_after,blueprint::BlueprintRng{seed:10548,ring,draw_count:16919});assert_eq!(plan.prepared.selection.created_count,2);assert!(plan.category_backfill_is_noop&&plan.prepared.landing_factory_pending);
 assert_eq!(plan.prepared.actors[0][0x74],0);assert_eq!(plan.prepared.actors[1][0x74],10);
 assert!(mission_bootstrap::prepare_player(&loaded,&names,[fill;36],blueprint::BlueprintRng{seed:5428,ring,draw_count:16855},&priors,5,origin,target,1).is_err());
 let mut out=Vec::new();for r in plan.generated.records{out.extend(r);}out.extend(plan.prepared.player.bytes());for a in plan.prepared.actors{out.extend(a);}std::fs::write(dir.join(format!("case{case}.bin")),out).unwrap();
 }
}
