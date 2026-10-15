//!454C7..45D8E nonterminal Genepod Mule work/payment, after shared power prefix.
//! Caller supplies live flags/frame/players; common animation/power tail is separate.
//! TerminalBoundary returns no proposed afterimage: full terminal RNG/factory must
//! be one transaction, never a partial payment committed by this planner.
#[derive(Debug,Clone,PartialEq,Eq)]pub enum WorkError{Building,Player,TerminalBoundary}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum WorkStatus{FrameWait,FundsWait,Paid{payer:u8,cost:i32}}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct WorkPlan{pub building_after:[u8;67],pub players_after:[[u8;1805];4],pub status:WorkStatus}
pub fn plan_work(building:&[u8;67],players:&[[u8;1805];4],frame:u32)->Result<WorkPlan,WorkError>{
 fn d(a:&[u8],i:usize)->i32{i32::from_le_bytes(a[i..i+4].try_into().unwrap())}
 fn put(a:&mut[u8],i:usize,v:i32){a[i..i+4].copy_from_slice(&v.to_le_bytes());}
 let flags=u16::from_le_bytes([building[49],building[50]]);let work=d(building,54);
 if building[6]!=4||building[24]>=4||building[45]!=0||building[46]>2||building[51]!=4||flags&0x141!=1||work<=0{return Err(WorkError::Building);}
 let mut a=*building;a[63]=3;let mut p=*players;
 let result=|a,p,status|Ok(WorkPlan{building_after:a,players_after:p,status});
 if flags&4==0&&frame%6!=0{return result(a,p,WorkStatus::FrameWait);}
 let owner=usize::from(building[24]);let cost=1i32<<building[46];
 if d(&p[owner],0x4a3).wrapping_add(d(&p[owner],0x4a7))<cost{return result(a,p,WorkStatus::FundsWait);}
 let payer=if p[owner][2]==0{building[58]}else{building[24]};
 if payer>=4||p[usize::from(payer)][0x4a1]>=4{return Err(WorkError::Player);}
 if work==1{return Err(WorkError::TerminalBoundary);}
 let i=usize::from(payer);let excess=cost.wrapping_sub(d(&p[i],0x4a3));
 if excess<=0{put(&mut p[i],0x4a3,excess.wrapping_neg());}else{let secondary=d(&p[i],0x4a7).wrapping_sub(excess);put(&mut p[i],0x4a3,0);put(&mut p[i],0x4a7,secondary);}
 let identity=usize::from(p[i][0x4a1]);if p[identity][2]!=0&&d(&p[i],0x4a3).wrapping_add(d(&p[i],0x4a7))<=1000{put(&mut p[i],0x4a7,1000);}
 put(&mut a,54,work-1);result(a,p,WorkStatus::Paid{payer,cost})
}
