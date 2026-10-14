use genewars_runtime::{blueprint::BlueprintRng,projectile_impact::plan_neutral_mule_impact};
fn word(a:&[u8],i:usize)->u16{u16::from_le_bytes([a[i],a[i+1]])}
fn draw(r:&mut BlueprintRng){r.seed=r.seed.wrapping_mul(0xe62d);r.ring=(r.ring+1)&31;r.draw_count=r.draw_count.wrapping_add(1)}
fn main(){
 let p=std::env::args().nth(1).unwrap();let input=std::fs::read(format!("{p}/damage-input.bin")).unwrap();let mut out=Vec::new();
 for row in input.chunks_exact(134){
  let mut before:[u8;124]=row[..124].try_into().unwrap();let mut rng=BlueprintRng{seed:word(row,124),ring:row[126],draw_count:0};let kills=u32::from_le_bytes(row[127..131].try_into().unwrap());let slot=word(row,131);let owner=row[133];
  // Literal factory/query effects are outside the impact planner.
  before[..2].copy_from_slice(&slot.to_le_bytes());before[2]=5;draw(&mut rng);draw(&mut rng);
  let mut effect=[0u8;81];effect[6]=5;effect[69]=20;effect[73]=4;effect[24]=owner;effect[15..17].copy_from_slice(&slot.to_le_bytes());
  let plan=plan_neutral_mule_impact(&before,&effect,rng).unwrap();out.extend(plan.after);out.extend(plan.rng_after.seed.to_le_bytes());out.push(plan.rng_after.ring);out.extend(plan.rng_after.draw_count.to_le_bytes());
  let mut audio:Vec<u8>=plan.audio.iter().map(|a|{assert_eq!((a.mode,a.parameter,a.stack_byte,a.stack_dword),(2,0,127,100));a.channel}).collect();audio.push(57);out.push(audio.len()as u8);audio.resize(4,0);out.extend(audio);
  let mut players=[0u8;7220];for p in 0..4{players[p*1805+0x6fd..p*1805+0x701].copy_from_slice(&kills.wrapping_add(u32::from(plan.kill_credit==Some(p as u8))).to_le_bytes())}out.extend(players);
  let saved=before;before[94]=7;assert!(plan_neutral_mule_impact(&before,&effect,rng).is_err());before=saved;before[96]|=2;assert!(plan_neutral_mule_impact(&before,&effect,rng).is_err());
 }
 std::fs::write(format!("{p}/damage-actual.bin"),out).unwrap();
}
