use genewars_assets::ConqMissionSource;
use genewars_runtime::{actor_world::ActorWorld,actor_pool::ActorHandleBindings,blueprint::BlueprintRng,
 player_state::PlayerState,building_resources::BuildingResourceDatabase,bookkeeping};
use std::{fs,path::Path,io::Write};
fn snapshot(w:&ActorWorld)->Vec<u8>{
 let mut b=w.backing_bytes().to_vec();for r in w.actors().records(){b.extend(r)}
 for r in w.creatures().unwrap().records(){b.extend(r)}
 for r in w.buildings().unwrap().records(){b.extend(r)}
 for p in 0..4 {b.extend(w.player(p).unwrap().bytes())}
 let r=w.rng();b.extend(r.seed.to_le_bytes());b.push(r.ring);b.extend(r.draw_count.to_le_bytes());
 for slot in 0..256{b.push(w.buildings().unwrap().binding(slot).unwrap().map(|r|r.selector+1).unwrap_or(0));}b
}
fn main(){let a:Vec<_>=std::env::args().collect();let p=Path::new(&a[1]);let out=Path::new(&a[2]);
 let mission=fs::read(p.join("private/integration/game-runtime/terrain/CONQ001.DAT")).unwrap();let source=ConqMissionSource::parse(&mission).unwrap();
 let resource=BuildingResourceDatabase::load_selector1_bturr(fs::read(p.join("private/reference-run/run-006/drive-c/GAME/DATA/BUILDING/BTURR.ANB")).unwrap()).unwrap();
 let mut ledger=fs::File::create(out.join("cases.tsv")).unwrap();
 for case in 0..64u8 {
  let selected=case%4;let player_count=[0,1,2,4][usize::from((case%32)/8)];
  let players=core::array::from_fn(|owner|{let mut r=*PlayerState::fresh_campaign(owner as u8,[20736,13568],b"fixture\0").unwrap().bytes();
   r[0x5e5]=[0,49,255][(usize::from(case)+owner)%3];
   r[0x4a7..0x4ab].copy_from_slice(&[-1i32,0,1,999][(usize::from(case)+owner)%4].to_le_bytes());
   for n in 0..8{r[0x5ea+n*2..0x5ec+n*2].copy_from_slice(&[0u16,1,999,1000,65535][(usize::from(case)+owner+n)%5].to_le_bytes());}
   r[0x499..0x49d].copy_from_slice(&[0u32,1,700,u32::MAX][(usize::from(case)+owner)%4].to_le_bytes());PlayerState::from_record(&r).unwrap()});
  let mut w=ActorWorld::from_mission(&source,&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed:5428,ring:31,draw_count:u32::MAX}).unwrap();
  w.admit_creatures(&[[0;124];300]).unwrap();w.admit_buildings(&[[0xA5;67];256],[None;256]).unwrap();
  for owner in 0..4u8 {for n in 0..[0,1,3,8][usize::from((case+owner)%4)]{
   let slot=w.create_building(owner,1,[19839+n*256,12927]).unwrap().unwrap().slot;
   let increment=[0,1,62,119][usize::from((case+n as u8)%4)];if increment!=0{w.progress_selector1_building(slot,increment,true).unwrap();}
  }}
  if case>=32{w.review_sparse_buildings();}
  let dir=out.join(format!("case{case}"));fs::create_dir(&dir).unwrap();fs::write(dir.join("before.bin"),snapshot(&w)).unwrap();
  let plan=w.step_incomplete_building_callbacks(player_count,selected,&resource).unwrap();
  assert!(plan.callbacks.iter().all(|(_,r)|r.stop_audio_23));
  fs::write(dir.join("after.bin"),snapshot(&w)).unwrap();writeln!(ledger,"case{case}\t{player_count}\t{selected}\t{}",plan.callbacks.len()).unwrap();
  let unchanged=w.clone();assert!(w.step_incomplete_building_callbacks(5,selected,&resource).is_err());assert_eq!(w,unchanged);
 }
 // Source inactive slots skip48002 DEC; sparse bookkeeping is admitted.
 let mut p=*PlayerState::fresh_campaign(0,[0,0],b"fixture\0").unwrap().bytes();p[0x4a2]=1;
 let mut rows=[[0;67];64];rows[1][6]=4;rows[1][0x2d]=1;rows[1][15]=1;
 assert!(bookkeeping::plan_incomplete_selector1_player(&p,&rows,0,6000).is_ok());
 rows.swap(0,1);rows[0][15]=0;assert!(bookkeeping::plan_incomplete_selector1_player(&p,&rows,0,6000).is_ok());
 for (offset,value)in [(0x15,1),(0x2d,2),(0x2e,1),(6,3)] {let mut bad=rows;bad[0][offset]=value;assert!(bookkeeping::plan_incomplete_selector1_player(&p,&bad,0,6000).is_err());}
 println!("64 owned passes, sparse admissions and explicit invalid-record checks");
}
