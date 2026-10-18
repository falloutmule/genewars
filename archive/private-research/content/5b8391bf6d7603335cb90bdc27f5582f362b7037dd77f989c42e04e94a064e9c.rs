// Injected only into a frozen private copy of actor_world, never production.
use super::*;
use std::{fs,path::Path,io::Write};
fn snapshot(w:&ActorWorld)->Vec<u8>{let mut v=w.backing.clone();for r in w.actors.records(){v.extend(r)}for r in w.creatures.as_ref().unwrap().records(){v.extend(r)}for r in w.buildings.as_ref().unwrap().records(){v.extend(r)}for r in w.effects.as_ref().unwrap().records(){v.extend(r)}for r in w.family8.as_ref().unwrap().records(){v.extend(r)}v}
fn staged_refresh(w:&mut ActorWorld,pos:[i16;2],slope:bool)->Result<(),ActorWorldError>{let mut staged=w.clone();staged.refresh_terrain_vertex(pos,slope)?;*w=staged;Ok(())}
pub fn run(p:&Path,out:&Path){
 let raw=fs::read(p.join("private/integration/game-runtime/terrain/CONQ001.DAT")).unwrap();let src=ConqMissionSource::parse(&raw).unwrap();
 let mut ledger=fs::File::create(out.join("cases.tsv")).unwrap();
 for case in 0..24u8 {
  let pos=if case<12 {[19839i16,12927i16]}else{[0i16,0i16]};let slope=case%2!=0;
  let players=core::array::from_fn(|i|PlayerState::fresh_campaign(i as u8,[0,0],b"fixture\0").unwrap());
  let mut w=ActorWorld::from_mission(&src,&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed:5428,ring:0,draw_count:0}).unwrap();
  w.admit_creatures(&[[0;124];300]).unwrap();w.admit_buildings(&[[0;67];256],[None;256]).unwrap();w.admit_effects(&[[0;81];200],[None;200]).unwrap();w.admit_family8(&[[0;102];16],[None;16]).unwrap();
  for cell in 0..16384{w.backing[MAP_START+cell*14+6..MAP_START+cell*14+9].fill(0);}
  let mut cells=Vec::new();
  for (slot,(dx,dy)) in [(0,0),(-1,0),(-1,-1),(0,-1)].into_iter().enumerate(){
   let x=((i32::from(pos[0])>>8)+dx)&127;let y=((i32::from(pos[1])>>8)+dy)&127;let cell=(y*128+x)as usize;cells.push(cell);
   w.backing[MAP_START+cell*14+6..MAP_START+cell*14+9].copy_from_slice(&[slot as u8,0,1]);
   for (n,kind) in [1u8,2,3,4,5,8].into_iter().enumerate(){
    let width=match kind{1=>124,2=>31,3=>200,4=>67,5=>81,_=>102};let mut r=vec![0u8;width];r[6]=kind;r[15..17].copy_from_slice(&(slot as i16).to_le_bytes());
    if n<5{r[3..6].copy_from_slice(&[slot as u8,0,[1u8,2,3,4,5,8][n+1]]);}
    let fx=[0,17,127,255][slot];let fy=[255,128,1,0][slot];r[7..9].copy_from_slice(&((x*256+fx)as i16).to_le_bytes());r[9..11].copy_from_slice(&((y*256+fy)as i16).to_le_bytes());
    r[17..21].copy_from_slice(&(-12345i32).to_le_bytes());r[21]=if (case/2+slot as u8)%2==0{2}else{0};r[23]=199;
    if width>43 {let height=[-100000i32,0,100000,i32::MAX,i32::MIN][(usize::from(case/2)+slot)%5];r[39..43].copy_from_slice(&height.to_le_bytes());}
    if kind==5 {r[69]=if case%4<2{0x2e}else{0x21};}
    match kind{1=>w.creatures.as_mut().unwrap().fixture_set(slot,&r),3=>w.actors.fixture_set(slot,&r),4=>w.buildings.as_mut().unwrap().fixture_set(slot,&r),5=>w.effects.as_mut().unwrap().fixture_set(slot,&r),8=>w.family8.as_mut().unwrap().fixture_set(slot,&r),2=>{let start=w.layouts[1].file_offset+slot*w.layouts[1].packed_stride;w.backing[start..start+31].copy_from_slice(&r);},_=>unreachable!()}
   }
  }
  let dir=out.join(format!("case{case}"));fs::create_dir(&dir).unwrap();fs::write(dir.join("before.bin"),snapshot(&w)).unwrap();
  w.refresh_terrain_vertex(pos,slope).unwrap();for cell in cells{w.refresh_terrain_object_heights(cell as u16).unwrap();}
  fs::write(dir.join("after.bin"),snapshot(&w)).unwrap();writeln!(ledger,"case{case}\t{}\t{}\t{}",pos[0],pos[1],u8::from(slope)).unwrap();
  // Late broken next link: caller stages the helper and rejects without commit.
  for invalid in [[255,127,3],[0,0,2]]{let mut bad=w.clone();let start=bad.layouts[1].file_offset;bad.backing[start+3..start+6].copy_from_slice(&invalid);let before=bad.clone();assert!(staged_refresh(&mut bad,pos,slope).is_err());assert_eq!(bad,before);}
 }
}
