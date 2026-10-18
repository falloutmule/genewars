mod player_state {pub const PLAYER_RECORD_BYTES:usize=1805;}
mod bookkeeping;
use std::{fs,io::Write,path::Path};
fn main(){
 let root=std::env::args().nth(1).unwrap();let root=Path::new(&root);
 let input=fs::read(root.join("input.bin")).unwrap();let mut out=fs::File::create(root.join("actual.bin")).unwrap();
 for case in input.chunks_exact(24375){
  let period=u16::from_le_bytes([case[1],case[2]]);let mut buildings=Vec::new();
  for owner in 0..4{
   let player=case[3+owner*1805..3+(owner+1)*1805].try_into().unwrap();
   let start=7223+owner*64*67;
   let records:Vec<[u8;67]>=case[start..start+64*67].chunks_exact(67).map(|r|r.try_into().unwrap()).collect();
   let plan=bookkeeping::plan_admitted_building_player(player,&records,case[0],period).unwrap();
   out.write_all(&plan.player.after).unwrap();for r in plan.records_after{buildings.extend(r);}
  }
  out.write_all(&buildings).unwrap();
 }
 // Admission remains narrower than all branches in47F80/437A0.
 let mut player=[0;1805];player[0x4a2]=1;let mut records=[[0;67];64];records[0][6]=4;records[0][45]=11;
 assert!(bookkeeping::plan_selector1_player(&player,&records,0,6000).is_err());
 for (offset,value) in [(0x15,1),(0x2e,1),(0x2d,4),(0x2d,13),(0x31,0x40),(0x32,8)]{
  let mut changed=records;changed[0][offset]=value;
  assert!(bookkeeping::plan_admitted_building_player(&player,&changed,0,6000).is_err());
 }
}
