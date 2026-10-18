extern crate self as genewars_core;
#[path = "C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-core/src/player_creation.rs"] mod selection;
pub mod cd_candidate { pub use crate::selection::*; }
#[path = "C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/player_state.rs"] mod state;
#[path = "C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/blueprint.rs"] mod blueprint;
#[path = "C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/actor_initialization.rs"] mod actor_initialization;
use state::PlayerState;
fn main() {
 let dir=std::path::PathBuf::from(std::env::args().nth(1).unwrap());
 let input=std::fs::read(dir.join("inputs.bin")).unwrap();
 let label=std::fs::read(dir.join("label.bin")).unwrap();
 let mut output=Vec::new();
 for chunk in input.chunks_exact(5+2*0x70D) {
  let player=chunk[0];
  let target=[i16::from_le_bytes([chunk[1],chunk[2]]),i16::from_le_bytes([chunk[3],chunk[4]])];
  let saved=PlayerState::from_record(&chunk[5..5+0x70D]).unwrap();
  let fresh=PlayerState::fresh_campaign(player,target,&label).unwrap();
  let restored=PlayerState::restored_for_mission(&chunk[5+0x70D..],player,&saved).unwrap();
  output.extend_from_slice(fresh.bytes()); output.extend_from_slice(restored.bytes());
 }
 std::fs::write(dir.join("actual.bin"),output).unwrap();
}
