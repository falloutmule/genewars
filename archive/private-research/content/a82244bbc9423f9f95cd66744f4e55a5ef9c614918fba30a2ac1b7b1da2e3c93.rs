//! Private source7F3B0 request/caller protocol. Factory and reveal are ports.
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Request {pub owner:u8,pub position:[i16;2],pub object_code:u8,pub height:i32,pub motion:[u16;3],pub options:[u32;4]}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Completion {pub player:[u8;1805],pub created:bool,pub reveal:Option<([i16;2],u16)>}
fn word(a:&[u8],o:usize)->i16{i16::from_le_bytes(a[o..o+2].try_into().unwrap())}
pub fn request<E>(player:&[u8;1805],counter_1ec2:u32,special_target_1c9a:[i16;2],
 mut height:impl FnMut([i16;2])->Result<i32,E>)->Result<Request,E> {
 let special=player[2]==0&&counter_1ec2!=0;
 let position=if special{special_target_1c9a}else{[word(player,9),word(player,11)]};
 let ground=height(position)?;
 Ok(Request{owner:player[1],position,object_code:0x31,height:ground.wrapping_add(0x6400),motion:[0,0x400,0x440],options:[u32::from(special),u32::from(player[1]),0,0]})
}
pub fn complete(player:&[u8;1805],selected:u8,factory_result:Option<&[u8;81]>)->Completion {
 let mut after=*player;
 if let Some(effect)=factory_result {
  after[0x496..0x498].copy_from_slice(&effect[15..17]);after[0x498]=effect[6];
  Completion{player:after,created:true,reveal:(player[1]==selected).then_some(([word(effect,7),word(effect,9)],4096))}
 }else{Completion{player:after,created:false,reveal:None}}
}
