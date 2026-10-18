extern crate genewars_runtime;pub use genewars_runtime::physics_tables;mod actor_motion{pub(crate) fn angle(x:i32,y:i32,t:&[u16;257])->u16 {
    if x==0 && y==0 { return 0; }
    let ax=x.unsigned_abs(); let ay=y.unsigned_abs();
    let a=if ax<ay { t[(u64::from(ax)*256/u64::from(ay)) as usize] }
          else { t[(u64::from(ay)*256/u64::from(ax)) as usize] };
    match (x>=0,y>=0,ax<ay) {
        (true,false,true)=>a,(true,false,false)=>512-a,
        (true,true,true)=>1024-a,(true,true,false)=>512+a,
        (false,false,true)=>2048-a,(false,false,false)=>1536+a,
        (false,true,true)=>1024+a,(false,true,false)=>1536-a,
    }
}}#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/mule_waypoint_wait.rs"]mod wait;use std::io::{Read,Write};fn main(){let args:Vec<_>=std::env::args().collect();let physics=physics_tables::PhysicsTables::from_mapped_bytes(&std::fs::read(&args[1]).unwrap(),&std::fs::read(&args[2]).unwrap()).unwrap();let mut input=Vec::new();std::io::stdin().read_to_end(&mut input).unwrap();let mut output=Vec::new();for c in input.chunks_exact(129){let mut calls=0;let q=wait::plan_wait(c[..124].try_into().unwrap(),u32::from_le_bytes(c[124..128].try_into().unwrap()),&physics,|_|{calls+=1;Ok::<_,()>(c[128]!=0)});if args.len()>3{assert!(q.is_err());assert_eq!(calls,0);output.push(1);continue;}let q=q.unwrap();output.extend(q.record_after);output.extend(q.saved_target_speed.to_le_bytes());output.push(calls);}std::io::stdout().write_all(&output).unwrap();}