//!61C8A..62201 neutral same-Mule pre-factory birth prefix only.
//! Caller binds actual peer virtual68 and736F0 planar-distance results. No
//! factory success, post-factory reset, pool mutation or caller-tail is implied.
use crate::blueprint::BlueprintRng;
#[derive(Debug,Clone,PartialEq,Eq)]pub enum BirthError{Record,Rng,PairResetRequired,RngCycle}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct BirthRequest{pub owner:u8,pub species:u8,pub quality:u8,pub parent_id:u16,pub position:[i16;2],pub inherit:bool}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct BirthPlan{pub current_after:[u8;124],pub peer_after:[u8;124],pub rng_after:BlueprintRng,pub request:BirthRequest}
pub fn plan_birth(current:&[u8;124],peer:&[u8;124],mut rng:BlueprintRng,peer_unavailable:bool,planar_distance_squared:u32)->Result<BirthPlan,BirthError>{
 let word=|r:&[u8;124],at|i16::from_le_bytes([r[at],r[at+1]]);
 if word(current,108)!=0||word(current,15)==word(peer,15){return Err(BirthError::Record);}
 for(r,other)in[(current,peer),(peer,current)]{if r[6]!=1||r[24]!=4||r[122]!=4||r[94]!=10||r[87]!=1||word(r,85)!=word(other,15)||!(0..300).contains(&word(r,15)){return Err(BirthError::Record);}}
 if rng.ring>=32{return Err(BirthError::Rng);}if peer_unavailable||planar_distance_squared>16384{return Err(BirthError::PairResetRequired);}
 let adjust=|r:&[u8;124]|{let mut a=*r;let cost=word(r,98)>>1;let reserve=word(r,104);let next=if cost>reserve{0}else{reserve.wrapping_sub(cost)};a[104..106].copy_from_slice(&next.to_le_bytes());a[121]=a[121].wrapping_neg();a};
 fn draw(r:&mut BlueprintRng)->u16{r.seed=r.seed.wrapping_mul(0xe62d);r.ring=(r.ring+1)&31;r.draw_count=r.draw_count.wrapping_add(1);r.seed>>8}
 let eligible=[current[118]>90,peer[118]>90];
 //Mapped104A64 species4 gene pair[4,4],104A98[4*5+4]=4.
 //All selections therefore retain Mule identity, but branch-specific draws
 //and inheritance remain literal, including the rejection loop.
 let inherit=if draw(&mut rng)%100<50{draw(&mut rng)%100<50*(u16::from(eligible[0])+u16::from(eligible[1]))}else{
  let first=draw(&mut rng)%6;let forbidden=match first{0|1=>3,2=>4,3|4=>24,_=>32};let start_seed=rng.seed;
  loop{let other=draw(&mut rng)%6;if forbidden&(1<<other)==0{break;}if rng.seed==start_seed{return Err(BirthError::RngCycle);}}
  eligible[usize::from(first>=3)]
 };
 let difference=current[118].abs_diff(peer[118]);let quality=current[118].min(peer[118])+((draw(&mut rng)% (u16::from(difference)+1))as u8);
 Ok(BirthPlan{current_after:adjust(current),peer_after:adjust(peer),rng_after:rng,request:BirthRequest{owner:4,species:4,quality,parent_id:word(current,15)as u16,position:[word(current,7),word(current,9)],inherit}})
}
