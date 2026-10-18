//! Bounded neutral-Mule hostile acquisition, not corpse food or combat resolution.
//! Source626EA performs the neighborhood query before62949 examines enemycount.
//! Caller must prove the query's selected hostile is this one live owned Mule,
//! with no extra candidate-dependent RNG, and supply the postphysics record.
//! First two draws are the query traversal;629C0..62A45 draws journalCD6.
//! Only roll>90 is admitted: it jumps62E11 directly to5B580(BL0).
//! Mule parameter1046B0+24h is90 in mapped object3 SHA256
//! aaab09272771ee7f3fbe71e9a3dc75add592fcca42fdd8d982e069629ac1ce87.
//! The alternate flee/disposition branch is deliberately unsupported.
use crate::{blueprint::BlueprintRng, creature_world::{CreatureRecord, mule_requested_speed}};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostileAcquisitionError { Current, Enemy, Identity, InvalidRng, UnsupportedDisposition }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostileDrawPurpose { NeighborhoodX, NeighborhoodY, Disposition }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostileDraw { pub purpose: HostileDrawPurpose, pub seed_before: u16, pub seed_after: u16, pub value: u8 }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostileAcquisitionPlan {
    pub after: CreatureRecord,
    pub enemy_slot: u16,
    pub rng_before: BlueprintRng,
    pub rng_after: BlueprintRng,
    pub draws: [HostileDraw; 3],
}
fn word(r: &CreatureRecord, at: usize) -> i16 { i16::from_le_bytes([r[at], r[at+1]]) }
fn put(r: &mut CreatureRecord, at: usize, value: i16) { r[at..at+2].copy_from_slice(&value.to_le_bytes()); }
/// Caller proves the source query radius/traversal, sole eligible hostile,
/// exact MuleSelector4 bindings, and virtual10(enemy)==false. Neither input
/// is mutated. No goal copy, target damage, reservation, animation or physics.
/// Returned timer32767 precedes the shared callback tail decrement.
pub fn plan_acquisition(current: &CreatureRecord, enemy: &CreatureRecord, rng_before: BlueprintRng)
    -> Result<HostileAcquisitionPlan, HostileAcquisitionError>
{
    use HostileAcquisitionError as Error;
    let id=word(current,15);let target=word(enemy,15);
    if !(0..300).contains(&id)||!(0..300).contains(&target)||id==target{return Err(Error::Identity);}
    if current[6]!=1||current[24]!=4||current[122]!=4||current[94]!=8
        ||current[0x60]!=8||!(current[0x61]==0x40||current[0x61]==0)||current[0x78]!=0
        ||current[0x76]>90||current[0x77]!=255||current[0x7b]!=255
        ||current[0x15]!=0x80||!matches!(current[0x16],2|0x82)
        ||current[0x4f]!=0||current[0x52]!=0||current[0x57]!=0||current[0x5d]!=0
        ||word(current,0x6c)!=0||word(current,0x6a)<=0||word(current,0x66)<=2
        ||word(current,0x68)<0||word(current,0x62)<=0
        ||word(current,0x68)>(word(current,0x62)>>1)
        ||word(current,0x49)as u16&0x100!=0{return Err(Error::Current);}
    //6119 measured enemy shape: a live WALKING owner0 MuleSelector4 in state1
    //(0x60=0xc8, 0x52/0x57 retained 1 from its own pursuit). 5B580 reads the
    //enemy's id/kind/virtual10 and species only (0005b5bc..0005b6f4: ESI+0xf,
    //ESI+6, ESI+0x2d, the 1ee20 virtual10 call); 5CC60 reads the CURRENT record
    //(EDX=current), not the enemy - so the prior enemy 0x52/0x57==0 and
    //state==8 gates were narrowings to the first measured shape, not source
    //predicates. The hostile classification itself is 6B9B0 (state!=0xF).
    if enemy[6]!=1||enemy[24]!=0||enemy[122]!=4
        ||!(enemy[94]==8||enemy[94]==1&&enemy[0x60]==0xc8)
        ||word(enemy,0x49)as u16&0x400!=0||word(enemy,0x6a)<=0
        ||enemy[0x4f]!=0||enemy[0x5d]!=0
        ||enemy[0x52]>1||enemy[0x57]>1
    {return Err(Error::Enemy);}
    if rng_before.ring>=32{return Err(Error::InvalidRng);}
    let mut rng=rng_before;
    //629C0..62ACE (instructions-0005ed20.txt): the disposition is TWO draws of
    //the shared 0x11878a LCG after the two query-traversal draws. Draw3 gates:
    //(v>>8&0x7fff)/100 CMP [EBX+0x24](=90), JG 62e11 REFUSES when >90 (the old
    //3-draw roll>90-only admission was a narrowing to the first measured
    //shape). Draw4: (v>>8&0x7fff)/256 CMP confidence, JL 62e11 refuses when
    //>= confidence; confidence = word(0x6a)*256/word(0x64) -
    //((weight/512)<<5) + 10 with weight = 1024 - sqrt(sample distance^2)
    //(6B7BE subtracts the sample distance; 62935..62987 builds ECX), the same
    //pinned formula the F14/F24 owned planner uses. The measured 6119 record
    //admits: gate 55 <= 90, confidence 266 > roll 167.
    let dx=cell_delta(word(current,7),word(enemy,7));let dy=cell_delta(word(current,9),word(enemy,9));
    let distance=(dx*dx+dy*dy)*256*256;
    let weight=1024-genewars_core::cd_candidate::integer_sqrt(distance.max(0)as u32)as i32;
    let confidence=i32::from(word(current,0x6a))*256/i32::from(word(current,0x64))
        -((weight/512)<<5)+10;
    let draws=[HostileDrawPurpose::NeighborhoodX,HostileDrawPurpose::NeighborhoodY,HostileDrawPurpose::DispositionGate,HostileDrawPurpose::ConfidenceGate].map(|purpose|{
        let before=rng.seed;rng.seed=rng.seed.wrapping_mul(0xe62d);
        rng.rng_ring_update();
        HostileDraw{purpose,seed_before:before,seed_after:rng.seed,value:(rng.seed>>8)as u8}
    });
    if draws[2].value%100>90||i32::from(draws[3].value)>=confidence{return Err(Error::UnsupportedDisposition);}
    let mut after=*current;
    //5B58A→5CC60 state8 local reset; null reference indices remain opaque.
    after[0x15]&=0xdf;after[0x33..0x37].fill(0);
    put(&mut after,0x60,(word(current,0x60)as u16&0xd71a)as i16);
    after[0x5d]=0;after[0x57]=0;put(&mut after,0x6c,0);
    //5B5BC stores target55; kind1 table5B56C selects5B6C3.
    put(&mut after,0x55,target);after[0x57]=1;
    after[0x16]&=0x3f;put(&mut after,0x49,((word(current,0x49)as u16|0x14)&0xeffc)as i16);
    after[0x54]=20;after[0x4f]=0;put(&mut after,0x50,target);after[0x52]=1;
    //5B773..5B7CE: virtual80, timer, low80 and pursuit state1/low40.
    let speed=mule_requested_speed(&after);put(&mut after,0x3f,speed);
    put(&mut after,0x6c,32767);after[0x60]|=0xc0;after[0x5e]=1;
    //65B4D subsequently calls5D360; owner4/skill<=90 returns without action
    //at5D399..5D3E2 even when its phase gate is active. Caller owns common tail.
    Ok(HostileAcquisitionPlan{after,enemy_slot:target as u16,rng_before,rng_after:rng,draws})
}
#[cfg(test)]
mod tests {
 use super::*;
 fn fixture()->(CreatureRecord,CreatureRecord,BlueprintRng){
  let mut a=[0xa5;124];let mut b=[0x5a;124];
  for(r,id,owner)in[(&mut a,5,4),(&mut b,1,0)]{r[6]=1;r[24]=owner;r[122]=4;r[94]=8;put(r,15,id);put(r,0x6a,67);put(r,0x49,0x200);for at in[0x4f,0x52,0x57,0x5d]{r[at]=0;}}
  a[0x60]=8;a[0x61]=0x40;a[0x78]=0;a[0x76]=50;a[0x77]=255;a[0x7b]=255;a[0x15]=0x80;a[0x16]=0x82;
  put(&mut a,0x6c,0);put(&mut a,0x66,7500);put(&mut a,0x62,143);put(&mut a,0x68,3);
  (a,b,BlueprintRng{seed:45441,ring:18,draw_count:48155})
 }
 #[test]fn hostile_acquisition_keeps_goal_and_enemy_opaque_bytes(){
  let(a,b,rng)=fixture();let saved=(a,b);let p=plan_acquisition(&a,&b,rng).unwrap();
  assert_eq!(p.draws.map(|d|d.value),[25,241,197]);assert_eq!(p.rng_after.seed,50549);
  assert_eq!(p.after[94],1);assert_eq!(&p.after[0x50..0x53],&[1,0,1]);assert_eq!(&p.after[0x55..0x58],&[1,0,1]);
  assert_eq!(&p.after[11..15],&a[11..15]);assert_eq!(&p.after[..6],&a[..6]);assert_eq!(&p.after[31..39],&a[31..39]);assert_eq!(p.after[0x61],0x40);assert_eq!(word(&p.after,0x6c),32767);assert_eq!((a,b),saved);
 }
 #[test]fn disposition_and_target_failures_preserve_inputs_and_rng_wrap(){
  let(a,b,mut rng)=fixture();let saved=(a,b);rng.ring=31;rng.draw_count=u32::MAX-1;let p=plan_acquisition(&a,&b,rng).unwrap();assert_eq!(p.rng_after.ring,2);assert_eq!(p.rng_after.draw_count,1);
  rng.seed=0;assert_eq!(plan_acquisition(&a,&b,rng),Err(HostileAcquisitionError::UnsupportedDisposition));
  for(offset,value)in[(24,4),(94,24),(0x4f,1)]{let mut bad=b;bad[offset]=value;assert!(plan_acquisition(&a,&bad,rng).is_err());}
  let mut commanded=a;commanded[0x60]|=0x10;assert_eq!(plan_acquisition(&commanded,&b,rng),Err(HostileAcquisitionError::Current));assert_eq!((a,b),saved);
 }
}
