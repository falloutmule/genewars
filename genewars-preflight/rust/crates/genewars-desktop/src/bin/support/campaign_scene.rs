//! Bound original campaign background and next-mission header.
//! Planet widgets and system movies remain presentation fidelity work.
use std::path::Path;
use genewars_assets::{Palette256,PaletteChannelMode};
use genewars_render::RgbImage;
use genewars_core::cd_candidate::CampaignResourcePlan;
use crate::runtime_resources::read_bound;
pub struct CampaignDisplay {pub image:RgbImage,pub name:String,pub resources:CampaignResourcePlan}
impl CampaignDisplay {
    pub fn load(root:&Path,resources:CampaignResourcePlan)->Result<Self,String>{
        let basename=|p:&str|p.rsplit('/').next().unwrap().to_ascii_uppercase();
        let pixels=read_bound(&root.join("campaign").join(basename(&resources.campaign_background)))?;
        if pixels.len()!=640*480{return Err("campaign background must be640x480".into());}
        let pal=Palette256::parse(&read_bound(&root.join("campaign").join(basename(&resources.campaign_palette)))?,PaletteChannelMode::Rgb).map_err(|e|e.to_string())?;
        if pal.entries().iter().flatten().any(|&v|v>63){return Err("unsupported campaign palette channels".into());}
        let header=read_bound(&root.join("campaign").join(basename(&resources.header)))?;
        let tail=header.get(32..).ok_or("mission header lacks name")?;
        let end=tail.iter().position(|&b|b==0).ok_or("unterminated mission name")?;
        let name=String::from_utf8(tail[..end].to_vec()).map_err(|e|e.to_string())?;
        let mut rgb=Vec::with_capacity(640*480*3);
        for i in pixels{rgb.extend(pal.entries()[i as usize].map(|v|((u16::from(v)*255)/63)as u8));}
        Ok(Self{image:RgbImage::from_rgb24(640,480,rgb).map_err(|e|e.to_string())?,name,resources})
    }
}
