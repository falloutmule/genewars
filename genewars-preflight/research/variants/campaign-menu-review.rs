use genewars_core::cd_candidate::{CampaignResourcePlan,MenuCallback0State,MenuControl,MenuPacket};
use genewars_assets::{CampaignHeader,ConqFastPrefix,Palette256,PaletteChannelMode};
use genewars_render::{CdTitleButton,RgbImage};
use std::{env,fs,io::Write,path::Path};

fn control(id:i8)->Option<MenuControl>{match id{0=>Some(MenuControl::Id0),1=>Some(MenuControl::Id1),3=>Some(MenuControl::Id3),2=>Some(MenuControl::Id2),_=>None}}
fn id(value:Option<MenuControl>)->i16{value.map(|x|x.event_id()).unwrap_or(-1)}

fn main(){
 let args:Vec<String>=env::args().collect();let out=Path::new(&args[1]);
 let mut plans=fs::File::create(out.join("plans.tsv")).unwrap();let mut admitted=0;
 for progress in i8::MIN..=i8::MAX{for prior in 0..=u8::MAX{
  let result=CampaignResourcePlan::new(progress,prior);
  assert_eq!(result.is_ok(),(0..24).contains(&progress)&&prior<4);
  if let Ok(p)=result{admitted+=1;writeln!(plans,"{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",progress,prior,p.local_planet,p.effective_system,p.header,p.mission,p.texture_atlas,p.campaign_background,p.campaign_palette,p.system_movie.as_deref().unwrap_or("-"),p.transition_background,p.transition_palette).unwrap();}
 }}assert_eq!(admitted,96);
 let cases=fs::read(out.join("packet-cases.bin")).unwrap();let mut results=Vec::new();
 for c in cases.chunks_exact(7){
  let mut state=MenuCallback0State::default();let captured=control(c[0] as i8);
  if let Some(target)=captured{
   state.update(MenuPacket{byte8:1,byte9:0},Some(target),false);
   if c[1]==0{state.update(MenuPacket::default(),None,false);}
  }else if c[2]!=0{
   state.update(MenuPacket{byte8:1,byte9:0},Some(MenuControl::Id0),false);
   state.update(MenuPacket{byte8:2,byte9:0},None,true);
  }
  let r=state.update(MenuPacket{byte8:c[3],byte9:c[4]},control(c[5] as i8),c[6]!=0);
  results.extend(r.event_id.to_le_bytes());results.push(r.input_mask_write.is_some() as u8);results.push(r.input_mask_write.unwrap_or(0));
  results.extend(id(r.captured).to_le_bytes());results.extend(id(r.selected).to_le_bytes());results.extend(r.countdown.to_le_bytes());results.push(r.capture_mask);
 }fs::write(out.join("packet-results.bin"),results).unwrap();
 let anb=fs::read(&args[2]).unwrap();let button=CdTitleButton::parse(&anb).unwrap();
 let mask:Vec<u8>=(0..75).flat_map(|y|(0..346).map(move|x|(x,y))).map(|(x,y)|button.hit_test(x,y)as u8).collect();
 fs::write(out.join("title-mask.bin"),mask).unwrap();
 for &(x,y) in &[(i32::MIN,0),(i32::MAX,0),(0,i32::MIN),(0,i32::MAX),(-1,0),(346,0),(0,75)]{assert!(!button.hit_test(x,y));}
 let pal=Palette256::parse(&fs::read(&args[3]).unwrap(),PaletteChannelMode::Rgb).unwrap();
 for selected in [false,true]{let mut image=RgbImage::from_rgb24(640,480,vec![17;640*480*3]).unwrap();button.draw(&mut image,&pal,140,115,selected).unwrap();fs::write(out.join(if selected{"title-selected.rgb"}else{"title-normal.rgb"}),image.rgb_bytes()).unwrap();}
 let header=fs::read(&args[4]).unwrap();let h=CampaignHeader::parse(&header).unwrap();fs::write(out.join("title.bytes"),h.title_bytes()).unwrap();assert_eq!(h.bytes(),header);
 let mut synthetic=vec![65;699];assert!(CampaignHeader::parse(&synthetic).is_err());synthetic[698]=0;assert_eq!(CampaignHeader::parse(&synthetic).unwrap().title_bytes().len(),666);synthetic[32]=0;assert!(CampaignHeader::parse(&synthetic).unwrap().title_bytes().is_empty());
 assert!(CampaignHeader::parse(&synthetic[..698]).is_err());synthetic.push(0);assert!(CampaignHeader::parse(&synthetic).is_err());
 let map=fs::read(&args[5]).unwrap();let prefix=ConqFastPrefix::parse(&map).unwrap();fs::write(out.join("map-prefix.bin"),prefix.copied_prefix()).unwrap();fs::write(out.join("map-tail.bin"),prefix.trailing_bytes()).unwrap();
 for i in 0..4{fs::write(out.join(format!("player-{i}.bin")),prefix.initial_player_record(i).unwrap()).unwrap();}
 fs::write(out.join("map-region.bin"),prefix.opaque_14_byte_region()).unwrap();
 assert!(prefix.initial_player_record(4).is_err());assert!(prefix.initial_player_record(usize::MAX).is_err());
 for length in [0,3,0x3b55d]{assert!(ConqFastPrefix::parse(&map[..length]).is_err());}
 let mut bad=map.clone();bad[0]^=1;assert!(ConqFastPrefix::parse(&bad).is_err());
}
