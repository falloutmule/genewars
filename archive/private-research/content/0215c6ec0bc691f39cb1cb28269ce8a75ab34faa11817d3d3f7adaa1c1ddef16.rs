mod player_state{pub const PLAYER_RECORD_BYTES:usize=1805;}
mod mission_status;
use std::{fs,io::Write,path::Path};
fn main(){let root=std::env::args().nth(1).unwrap();let root=Path::new(&root);let data=fs::read(root.join("input.bin")).unwrap();let mut out=fs::File::create(root.join("actual.bin")).unwrap();
for case in data.chunks_exact(1825){let player=case[..1805].try_into().unwrap();let mut visits=Vec::new();
let exhausted=mission_status::construction_capability_exhausted(player,|id|{visits.push(id as u8);Some(case[1805+id as usize])}).unwrap();
out.write_all(&[exhausted as u8,visits.len()as u8]).unwrap();visits.resize(5,255);out.write_all(&visits).unwrap();}}
