mod creature_world{type CreatureRecord=[u8;124];fn mule_skill_percent(record:&CreatureRecord,minimum:i32)->i32 {
    minimum+i32::from(record[0x76].min(100))*(100-minimum)/100
}pub(crate) fn mule_requested_speed(record:&CreatureRecord)->i16 {
    ((704*mule_skill_percent(record,75)/100)*(i32::from(record[0x7b])+1)>>8) as i16
}}
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/mule_idle_pair_init.rs"]mod pair;use std::io::{Read,Write};fn main(){let mut data=Vec::new();std::io::stdin().read_to_end(&mut data).unwrap();let mut out=Vec::new();for c in data.chunks_exact(248){let p=pair::plan_pair_init(c[..124].try_into().unwrap(),c[124..].try_into().unwrap());if std::env::args().any(|a|a=="reject"){assert!(p.is_err());out.push(1);continue;}let p=p.unwrap();out.extend(p.current_after);out.extend(p.peer_after);}std::io::stdout().write_all(&out).unwrap();}