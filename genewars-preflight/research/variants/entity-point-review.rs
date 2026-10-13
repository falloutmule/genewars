use genewars_core::cd_candidate::entity_view_position;
use std::{env,fs,path::Path};
fn main(){let a:Vec<String>=env::args().collect();let p=Path::new(&a[1]);let maps:Vec<Vec<u8>>=(0..5).map(|i|fs::read(p.join(format!("map-{i}.bin"))).unwrap()).collect();let cases=fs::read(p.join("cases.bin")).unwrap();let mut out=Vec::with_capacity(cases.len()/12*8);
 for b in cases.chunks_exact(12){let u=|i|u16::from_le_bytes(b[i..i+2].try_into().unwrap());let map=&maps[u(0)as usize];let value=entity_view_position(map,[u(2)as i16,u(4)as i16],[u(6),u(8)],u(10)).unwrap();for n in value{out.extend(n.to_le_bytes());}}
 fs::write(p.join("results.bin"),out).unwrap();for length in [0,1,229375,229377]{assert!(entity_view_position(&vec![0;length],[0,0],[0,0],128).is_err());}}
