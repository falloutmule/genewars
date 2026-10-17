//! Bounded owned state8 retained-hostile attack resume (624B8 dispatch ->
//! 5B580/5CC60 disposition), reached from the shared5EB30 choice when the
//! state8 attack-ready timer0 record still carries the live hostile pair.
//! Evidence: interpreter run on private/review/attempt13-fx-5940 (frame 5940,
//! slot 1), output private/review/owned-mule-captured-width/606d12841ae14e6b8
//! 7a2bd999f44e63a: status source_prepared, 11594 instructions, 5 draws,
//! participating 1, calls include 5eb30/6b150/5b580/5cc60/7ac00. Slot1 record
//! deltas only: 0x19 07->0f, 0x1b..0x1e 0->1, 0x4b 1482->1481, 0x5e 8->21,
//! 0x66 8657->8655, 0x68 14->13, 0x6c 0->32766; all seven other buffers and
//! the enemy record byte-identical.
//! Division of labour with the shared body (plan_mule_admitted_body_*):
//! 0x4b is the proven peer heading angle(enemy-current*2) as written by the
//! state21 attack continuation (62A16 region width fix immediately before the
//! 5B580 call); 0x66-2 comes from the 5E0B0 energy decrement plus the
//! 5F4xx lifecycle refresh; 0x68-1 from the 60 bit80 reserve branch; 0x6c-1
//! from the shared tail, so this planner writes 32767. The state21 attack
//! the state21 attack animation 0x19=0x0f and the 0x1b..0x1e fields are
//! left to the shared5E749 tail selection and its frame advance. Every other
//! domain rejects. This is
//! not a damage, flee, operator or alternate-query implementation; the enemy
//! record is not mutated and no audio or reveal request is produced.
use crate::{blueprint::BlueprintRng,creature_world::CreatureRecord,physics_tables::PhysicsTables};
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ReattackResumeError { Identity, Current, Enemy, InvalidRng }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct ReattackResumeDraw { pub index:usize, pub seed_before:u16, pub seed_after:u16 }
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ReattackResumePlan {
 pub after:CreatureRecord,pub rng_before:BlueprintRng,pub rng_after:BlueprintRng,
 /// Five measured dispatch draws; the per-draw source decomposition inside
 /// 624B8/5B580 is not yet attributed, so only the ordered chain is admitted.
 pub draws:[ReattackResumeDraw;5],
}
fn word(r:&CreatureRecord,at:usize)->i16{i16::from_le_bytes([r[at],r[at+1]])}
fn put(r:&mut CreatureRecord,at:usize,v:i16){r[at..at+2].copy_from_slice(&v.to_le_bytes());}
/// Current is the measured state8/0x6c==0 retained hostile pair shape; the
/// caller proves sole-hostile/no-friend world admission and the live enemy
/// binding. Input records are never mutated.
pub fn plan_resume(current:&CreatureRecord,enemy:&CreatureRecord,rng_before:BlueprintRng,physics:&PhysicsTables)
 ->Result<ReattackResumePlan,ReattackResumeError>{
 use ReattackResumeError as Error;
 let id=word(current,15);let target=word(enemy,15);
 if !(0..300).contains(&id)||!(0..300).contains(&target)||id==target{return Err(Error::Identity);}
 if current[6]!=1||current[24]!=0||current[122]!=4||current[94]!=8
  ||current[0x60]!=0x88||current[0x61]!=0||current[0x78]!=7
  ||current[0x76]>90||current[0x77]!=255||current[0x7b]!=255
  ||current[0x15]!=0x80||!matches!(current[0x16],2|0x82)
  ||current[0x4f]!=0||current[0x5d]!=0||current[0x52]!=1||current[0x57]!=1||current[0x5a]!=1
  ||word(current,0x6c)!=0||word(current,0x6a)<=0||word(current,0x66)<=2
  ||word(current,0x49)as u16&(0x100|0x400)!=0
  ||[0x50,0x55,0x58].iter().any(|&at|word(current,at)!=target){return Err(Error::Current);}
 if enemy[6]!=1||enemy[24]!=4||enemy[122]!=4||enemy[94]!=21
  ||enemy[0x60]!=0x88||enemy[0x61]!=0x40||enemy[0x78]!=0
  ||enemy[0x4f]!=0||enemy[0x5d]!=0||enemy[0x52]!=1||enemy[0x57]!=1
  ||word(enemy,0x50)!=id||word(enemy,0x55)!=id
  ||word(enemy,0x49)as u16&(0x100|0x400)!=0||word(enemy,0x6a)<=0{return Err(Error::Enemy);}
 if rng_before.ring>=32{return Err(Error::InvalidRng);}
 let mut rng=rng_before;
 let draws=std::array::from_fn(|index|{
  let before=rng.seed;rng.seed=rng.seed.wrapping_mul(0xe62d);rng.ring=rng.ring.wrapping_add(1)&31;rng.draw_count=rng.draw_count.wrapping_add(1);
  ReattackResumeDraw{index,seed_before:before,seed_after:rng.seed}});
 let mut after=*current;
 //Animation selection stays with the shared5E749 tail: it sees state21
 //with the unselected 0x19=7, resets 0x1b/0x1c/0x1d/0x1e and picks the
 //directional attack animation 15 from heading 0x39 (measured 0x07->0x0f,
 //frame advance 0x1b->1), so this planner writes none of those fields.
 //62A16-region peer heading: same formula as the proven state21 continuation.
 let dx=word(enemy,7).wrapping_mul(2).wrapping_sub(word(current,7).wrapping_mul(2));
 let dy=word(enemy,9).wrapping_mul(2).wrapping_sub(word(current,9).wrapping_mul(2));
 put(&mut after,0x4b,crate::actor_motion::angle(i32::from(dx),i32::from(dy),physics.angles())as i16);
 after[94]=21;
 //5B5CC-adjacent entry timer precedes the shared tail decrement (32766 measured).
 put(&mut after,0x6c,32767);
 Ok(ReattackResumePlan{after,rng_before,rng_after:rng,draws})
}
