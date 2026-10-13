#[path="../../rust/crates/genewars-core/src/player_creation.rs"] mod creation;
use creation::*;
use std::{env,fs,io::{BufWriter,Write},path::Path};
fn main() {
 let args:Vec<_>=env::args().collect();let dir=Path::new(&args[1]);
 let mut out=BufWriter::new(fs::File::create(dir.join("results.bin")).unwrap());
 for b in fs::read(dir.join("gates.bin")).unwrap().chunks_exact(4) {
  out.write_all(&[actor_category_is_enabled(b[0],b[1],b[2]as i8,b[3]!=0)as u8]).unwrap();
 }
 for flag in 0..=255 {out.write_all(&[family2_ordinary_update_is_suppressed(flag)as u8]).unwrap();}
 for b in fs::read(dir.join("plans.bin")).unwrap().chunks_exact(74) {
  let records=core::array::from_fn(|i|ActorBlueprintCandidate{active_name_byte:b[10+2*i],category:b[11+2*i]});let before=records;let slots:[u8;5]=b[5..10].try_into().unwrap();
  let mut row=[0u8;64];
  match plan_player_actor_creation(&records,b[0],b[1]as i8,b[2]!=0,b[3],slots,b[4]as i8) {
   Err(e)=>{row[0]=match e{PlayerCreationError::InvalidPlayer(_)=>1,PlayerCreationError::InvalidStoredCount(_)=>2,PlayerCreationError::StoredListFull=>3};},
   Ok(r)=>{row[1]=r.attempt_count;row[2]=r.created_count;row[3]=r.player_count_after;row[4..9].copy_from_slice(&r.slot_types_after);row[9]=r.target_reset_player;
    for(i,attempt)in r.attempts.iter().enumerate(){let at=10+13*i;if let Some(a)=attempt{row[at]=1;row[at+1]=a.blueprint_index;row[at+2]=a.category;if let Some(q)=a.allocation{row[at+3..at+13].copy_from_slice(&[1,q.local_slot,q.global_slot_id,q.owner,q.active_type,q.player_id_list_offset,q.blueprint_flags20_or,q.blueprint_index_at74,q.flags15_word as u8,(q.flags15_word>>8)as u8]);assert_eq!(q.flags70_word,0);}}}
   }
  }
  assert_eq!(records,before);out.write_all(&row).unwrap();
 }
 out.flush().unwrap();
}
