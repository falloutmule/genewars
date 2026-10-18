#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/mission_player_funds.rs"]mod funds;

use std::io::{Read,Write};
fn main(){let mut input=Vec::new();std::io::stdin().read_to_end(&mut input).unwrap();
let mut output=Vec::new();for row in input.chunks_exact(1808){
let plan=funds::plan_mission_player_funds(row[..1805].try_into().unwrap(),row[1805],row[1806],row[1807]).unwrap();
output.push(plan.applied as u8);output.extend_from_slice(&plan.after);}
std::io::stdout().write_all(&output).unwrap();}
