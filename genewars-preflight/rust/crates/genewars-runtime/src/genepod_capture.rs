//! Isolated enabled original0x9128/action8 control capture. Caller supplies the
//! original A86D0 opaque hit after manager-origin traversal. No rectangle or
//! whole-panel traversal is inferred here. Action IDs are explicitly low16.
#[derive(Debug,Clone,Copy,PartialEq,Eq,Default)]pub struct CaptureState{pub captured:bool,pub inside:bool,pub draw:u8,pub event:u16}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub struct CapturePlan{pub after:CaptureState,pub action:u16}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum CaptureError{Packet,State}
/// Only ordinary left idle/down/up/hold packets are admitted; other mouse
/// buttons/double-edge packets remain unsupported rather than reinterpreted.
pub fn plan_capture(before:CaptureState,packet8:u8,opaque_hit:bool)->Result<CapturePlan,CaptureError>{
 if !matches!(packet8,0|1|2|4){return Err(CaptureError::Packet);}
 if before.draw>1||(!before.captured&&before.draw!=0){return Err(CaptureError::State);}
 let mut a=before;let mut action=0xffff;
 if a.captured{
  if packet8&2!=0{a.event=8|if opaque_hit{0}else{0x8000};action=8;a.captured=false;a.draw=0;}
  else{a.event=if packet8&4!=0{2}else{0};if opaque_hit!=a.inside{action=8;if a.inside{a.event=1;}a.inside=opaque_hit;}a.draw=u8::from(opaque_hit);}
 }else if packet8&1!=0&&opaque_hit{a.captured=true;a.inside=true;a.draw=1;a.event=0;action=8;}
 Ok(CapturePlan{after:a,action})
}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub struct ReadyPlan{pub panel_state_after:u8,pub service_40810_55_requested:bool}
///21F0C..21FA5; ownership/disabled-control admission remains upstream.
/// The service request has EAX16F284, EBX55, DLselectedplayer; caller handles it.
pub fn plan_ready(panel_state:u8,building_flags:u16,animation_frame:u8,work:i32)->ReadyPlan{
 let eligible=building_flags&0x141==1;
 if !eligible{return ReadyPlan{panel_state_after:0,service_40810_55_requested:true};}
 if panel_state<=1&&animation_frame==0&&work<=0{return ReadyPlan{panel_state_after:3,service_40810_55_requested:true};}
 ReadyPlan{panel_state_after:panel_state,service_40810_55_requested:false}
}
