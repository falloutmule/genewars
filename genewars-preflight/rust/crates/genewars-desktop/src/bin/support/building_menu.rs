//! Port-owned construction chooser. Source2AB70 supplies order/availability,
//! not the key binding, rectangles or typography used here.
use genewars_runtime::building_availability::{available,MENU_ORDER};
pub fn label(selector:u8)->String{match selector {
    11=>"Extractor".into(),3=>"First Stage Distributor".into(),6=>"Photon Processor".into(),
    _=>format!("Building {selector} - not yet supported"),
}}
pub fn choices(player:&[u8;1805])->Vec<u8>{MENU_ORDER.into_iter().filter(|s|available(player,*s)==Ok(true)).collect()}
pub fn lines(choices:&[u8],funds:i32,actor:Option<usize>,feedback:Option<&str>)->Vec<String>{
    let mut lines=vec!["CONSTRUCTION - PORT CONTROLS".into(),
        format!("FUNDS {funds} - ARCHITECT {}",actor.map(|v|v.to_string()).unwrap_or("NOT SELECTED".into())),
        "SELECT BUILDING THEN RIGHT CLICK GROUND".into()];
    lines.extend(choices.iter().map(|s|format!("{} - AVAILABLE",label(*s))));
    lines.push(feedback.unwrap_or("C / ESC: CLOSE - ONLY UNLOCKED BUILDINGS SHOWN").chars().take(51).collect());lines
}
pub fn hit(choices:&[u8],point:Option<(i32,i32)>)->Option<u8>{
    let (x,y)=point?;if !(10..630).contains(&x){return None;}
    let top=480-((choices.len()+4)*18+12)as i32;
    let row=y-(top+6+3*18);if row<0{return None;}
    choices.get((row/18)as usize).copied()
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn source_unlock_order_and_pointer_row_boundaries(){
  let mut player=[0;1805];assert_eq!(choices(&player),[11]);
  player[0x60b]=1;player[0x603]=1;let c=choices(&player);assert_eq!(c,[11,3,6]);
  let first=480-((c.len()+4)*18+12)as i32+6+54;
  assert_eq!(hit(&c,Some((20,first))),Some(11));assert_eq!(hit(&c,Some((20,first+18))),Some(3));
  assert_eq!(hit(&c,Some((20,first+54))),None);assert_eq!(hit(&c,Some((9,first))),None);
 }
}
