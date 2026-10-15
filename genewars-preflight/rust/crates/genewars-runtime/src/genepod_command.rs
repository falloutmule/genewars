//! Verified QA BETA V5 opcode12/814A0 numeric handler, bounded Mule species4.
//! Eligibility belongs to upstream command producers; this handler has no
//! research, capacity or sufficient-funds rejection. No creature is produced.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum GenepodCommandError { Building, Caller, Species, Slot }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct GenepodRequest {pub species:i16,pub building_slot:i16,pub start:bool}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct GenepodCommandContext {
 /// Type byte+2 of the global player indexed by caller byte4A1.
 pub funds_identity_player_type:u8,
 /// Conjunction of original module, selected-building and panel-root checks.
 pub selected_panel_refresh_eligible:bool,
}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct GenepodCommandPlan {
 pub building_after:[u8;67],pub caller_after:[u8;1805],
 pub panel_refresh_requested:bool,
}
fn d(a:&[u8],at:usize)->i32{i32::from_le_bytes(a[at..at+4].try_into().unwrap())}
fn put(a:&mut[u8],at:usize,v:i32){a[at..at+4].copy_from_slice(&v.to_le_bytes());}
pub fn plan_mule_command(building:&[u8;67],caller:&[u8;1805],request:GenepodRequest,context:GenepodCommandContext)->Result<GenepodCommandPlan,GenepodCommandError>{
 if building[6]!=4||building[24]>=4||building[45]!=0||building[46]>2||building[49]&1==0{return Err(GenepodCommandError::Building);}
 if caller[1]>=4||caller[0x4a1]>=4{return Err(GenepodCommandError::Caller);}
 if request.species!=4{return Err(GenepodCommandError::Species);}
 if !(0..256).contains(&request.building_slot)||i16::from_le_bytes(building[15..17].try_into().unwrap())!=request.building_slot{return Err(GenepodCommandError::Slot);}
 let mut a=*building;let mut p=*caller;let scale=1i32<<building[46];let remainder=75&(scale-1);
 if request.start {
  let work=75i32>>building[46];put(&mut a,54,work);a[52..54].copy_from_slice(&(work as i16).to_le_bytes());a[51]=4;
  let excess=remainder.wrapping_sub(d(&p,0x4a3));
  if excess<=0{put(&mut p,0x4a3,excess.wrapping_neg());}
  else{let secondary=d(&p,0x4a7).wrapping_sub(excess);put(&mut p,0x4a3,0);put(&mut p,0x4a7,secondary);}
  if context.funds_identity_player_type!=0&&d(&p,0x4a3).wrapping_add(d(&p,0x4a7))<=1000{put(&mut p,0x4a7,1000);}
 }else{
  let total=i32::from(i16::from_le_bytes(a[52..54].try_into().unwrap()));
  let refund=total.wrapping_sub(d(&a,54)).wrapping_mul(scale).wrapping_add(remainder);
  let old_secondary=d(&p,0x4a7);let sum=old_secondary.wrapping_add(refund);let cap=d(&p,0x5e6);
  if (sum as u32)<(cap as u32){put(&mut p,0x4a7,sum);}
  else{put(&mut p,0x4a7,cap);let primary=d(&p,0x4a3).wrapping_add(refund.wrapping_sub(cap.wrapping_sub(old_secondary)));put(&mut p,0x4a3,primary);}
  put(&mut a,54,0);
 }
 if caller[2]==0{a[58]=caller[1];}
 Ok(GenepodCommandPlan{building_after:a,caller_after:p,panel_refresh_requested:request.start&&context.selected_panel_refresh_eligible})
}
