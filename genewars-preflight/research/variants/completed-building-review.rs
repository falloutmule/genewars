use genewars_runtime::{blueprint,completed_building as completed};
use std::{fs,path::Path};
fn main(){let a:Vec<_>=std::env::args().collect();let p=Path::new(&a[1]);let b=fs::read(p.join("input.bin")).unwrap();let mut out=Vec::new();
 for (case,row)in b.chunks_exact(1879).enumerate(){let before=row[..67].try_into().unwrap();let player=row[67..1872].try_into().unwrap();let rng=blueprint::BlueprintRng{seed:u16::from_le_bytes(row[1872..1874].try_into().unwrap()),ring:row[1874],draw_count:u32::from_le_bytes(row[1875..1879].try_into().unwrap())};
  let plan=completed::plan_quiet_selector1(before,player,rng,|_|Ok(None)).unwrap_or_else(|e|panic!("{case}:{e:?}"));out.extend(plan.after);out.extend(plan.player_after);out.extend(plan.rng_after.seed.to_le_bytes());out.push(plan.rng_after.ring);out.extend(plan.rng_after.draw_count.to_le_bytes());out.extend((plan.search_cells as u32).to_le_bytes());
  if plan.search_cells!=0{assert_eq!(plan.search_cells,113);assert!(completed::plan_quiet_selector1(before,player,rng,|_|Ok(Some(completed::TurretCellCandidate{position:[before[7]as i16+1000,before[9]as i16+1000],state:8,timer:0}))).is_err());}
 }
 fs::write(p.join("actual.bin"),out).unwrap();println!("128 complete quiet callbacks");}
