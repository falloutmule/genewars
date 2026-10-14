//! Source-bounded campaign victory screen (7DBC0 / 7D800).
//! Returns a static screen; BIOWIN playback and native input remain explicit obligations.
use std::path::Path;
use genewars_assets::{FontResource, Palette256, PaletteChannelMode, TextStringTable};
use genewars_render::{RgbImage, measure_text_line};

pub struct VictoryResources { background: Vec<u8>, palette: Palette256, font: Vec<u8>, tab: Vec<u8>, text: Vec<u8> }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VictoryScreenReceipt {
    pub feedback_text_index: usize,
    pub statistic_values: [i32;6],
    pub feedback_lines: usize,
    pub palette_service_indices: [u8;8],
    pub animation_pending: bool,
    pub native_input_acceptance: bool,
}
impl VictoryResources {
    pub fn load(root: &Path) -> Result<Self,String> {
        Self::from_bytes(
            crate::runtime_resources::read_bound(&root.join("results/WINSCR.DAT"))?,
            &crate::runtime_resources::read_bound(&root.join("results/WINSCR.PAL"))?,
            crate::runtime_resources::read_bound(&root.join("results/FONT7.DAT"))?,
            crate::runtime_resources::read_bound(&root.join("results/FONT7.TAB"))?,
            crate::runtime_resources::read_bound(&root.join("menu/text.dat"))?)
    }
    pub fn from_bytes(background:Vec<u8>, palette:&[u8], font:Vec<u8>, tab:Vec<u8>, text:Vec<u8>) -> Result<Self,String> {
        if background.len()!=640*480 { return Err("victory background must be 640x480".into()); }
        let palette=Palette256::parse(palette,PaletteChannelMode::Rgb).map_err(|e|e.to_string())?;
        if palette.entries().iter().flatten().any(|&v|v>63) { return Err("victory palette is not RGB6".into()); }
        FontResource::parse(&font,&tab).map_err(|e|e.to_string())?;
        TextStringTable::parse(&text).map_err(|e|e.to_string())?;
        Ok(Self{background,palette,font,tab,text})
    }
    /// `player` is the post-entry selected player, before72E50 clears statistics.
    /// Admission is normal campaign progress1..24; alternate modes use other text rules.
    pub fn render(&self, player:&[u8;1805]) -> Result<(RgbImage,VictoryScreenReceipt),String> {
        let progress=player[0x6e7] as i8;
        if !(1..=24).contains(&progress) { return Err("unsupported victory feedback progress".into()); }
        let font=FontResource::parse(&self.font,&self.tab).map_err(|e|e.to_string())?;
        let text=TextStringTable::parse(&self.text).map_err(|e|e.to_string())?;
        let index=477+progress as usize;
        let feedback=text.strings()[index];
        let mut rgb=Vec::with_capacity(640*480*3);
        for &i in &self.background { rgb.extend(self.palette.entries()[i as usize].map(|v|((v as u16*255)/63)as u8)); }
        let lines=wrap_ascii(&font,feedback,559)?;
        let line_height=font.record(0).map_err(|e|e.to_string())?.height as i32;
        for (row,line) in lines.iter().enumerate() { draw(&mut rgb,&font,&self.palette,line,43,31+row as i32*line_height,[43,31,560,114])?; }
        let offsets=[0x6f9,0x6fd,0x701,0x705,0x709,0x00d];
        let mut values=[0;6];
        for n in 0..6 {
            let y=196+n as i32*45;
            draw(&mut rgb,&font,&self.palette,text.strings()[546+n],218,y,[0,0,640,480])?;
            values[n]=i32::from_le_bytes(player[offsets[n]..offsets[n]+4].try_into().unwrap());
            let number=values[n].to_string();
            let width=measure_text_line(&font,number.as_bytes()).map_err(|e|e.to_string())?;
            // 1F750 flag10 centers within the58-wide clip, ignoring submitted x.
            draw(&mut rgb,&font,&self.palette,number.as_bytes(),444+29-(width>>1),y,[444,0,58,480])?;
        }
        let targets=[[0,0,0],[63,63,63],[63,0,0],[0,63,0],[0,0,63],[63,63,0],[0,63,63],[63,0,63]];
        let receipt=VictoryScreenReceipt{feedback_text_index:index,statistic_values:values,feedback_lines:lines.len(),palette_service_indices:targets.map(|t|palette_index(&self.palette,t)),animation_pending:true,native_input_acceptance:false};
        Ok((RgbImage::from_rgb24(640,480,rgb).map_err(|e|e.to_string())?,receipt))
    }
}

// A3F54: exact match first, then squared distance, Manhattan distance,
// weighted absolute color energy, and first index. All arithmetic fits i32.
pub fn palette_index(palette:&Palette256,target:[u8;3])->u8 {
    palette.entries().iter().enumerate().min_by_key(|(i,c)| {
        let d=[c[0] as i32-target[0] as i32,c[1] as i32-target[1] as i32,c[2] as i32-target[2] as i32];
        (d.iter().map(|v|v*v).sum::<i32>(),d.iter().map(|v|v.abs()).sum::<i32>(),2*(c[0] as i32).pow(2)+2*(c[1] as i32).pow(2)+(c[2] as i32).pow(2),*i)
    }).unwrap().0 as u8
}
fn wrap_ascii<'a>(font:&FontResource<'_>,text:&'a[u8],limit:i32)->Result<Vec<&'a[u8]>,String> {
    if text.iter().any(|&b|b<32||b>126) { return Err("result text controls/extended bytes need source layout admission".into()); }
    let mut lines=Vec::new(); let mut start=0;
    while start<text.len() {
        while start<text.len()&&text[start]==b' ' { start+=1; }
        if start==text.len(){break}
        let mut end=start;let mut accepted=start;
        while end<text.len() {
            while end<text.len()&&text[end]!=b' ' {end+=1;}
            let width=measure_text_line(font,&text[start..end]).map_err(|e|e.to_string())?;
            if width>limit && accepted>start {break}
            accepted=end;
            if end==text.len(){break}
            end+=1;
        }
        if accepted==start{return Err("empty result layout line".into())}
        lines.push(&text[start..accepted]);start=accepted;
    }
    Ok(lines)
}
fn draw(rgb:&mut[u8],font:&FontResource<'_>,palette:&Palette256,text:&[u8],mut x:i32,y:i32,clip:[i32;4])->Result<(),String> {
    for &b in text {
        let record=font.record_for_byte(b).map_err(|e|e.to_string())?;
        let g=font.decode_glyph(record.record).map_err(|e|e.to_string())?;
        if b!=b' ' { for gy in 0..g.height as usize { for gx in 0..g.width as usize {
            let at=gy*g.width as usize+gx;if !g.opaque[at]{continue}
            let px=x+gx as i32;let py=y+gy as i32;
            if px<clip[0]||px>=clip[0]+clip[2]||py<clip[1]||py>=clip[1]+clip[3]||px<0||px>=640||py<0||py>=480{continue}
            let literal=g.indices[at];
            let index=if literal==1 {2} else if [2,4,12,16,20,24,32,36,40,48,52,56,60].contains(&literal){80}else{literal};
            let color=palette.entries()[index as usize].map(|v|((v as u16*255)/63)as u8);
            let dest=(py as usize*640+px as usize)*3;rgb[dest..dest+3].copy_from_slice(&color);
        }}}
        x+=record.width as i32-1;
    }
    Ok(())
}
