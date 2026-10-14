//! Additional exact-build building assets. No callback or drawing policy is inferred.
use genewars_assets::{AnbSections,SpriteTable};
use crate::{identity::sha256_hex,building_resources::{BuildingResourceError,BuildingAnimation0Frame,BuildingAnimation0Component}};

pub const BSOLA_SHA256:&str="26ac960d7e68d97bddf7b0c13740976af4cf955ecfe98a4bea51ee95c292958b";
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum SourceBuildingBinding { CdBsolaSelector6, CdBmineSelector11, CdBpoweSelector3 }
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct SourceBuildingResourceDatabase { binding:SourceBuildingBinding, bytes:Vec<u8>, animations:Vec<Vec<BuildingAnimation0Frame>> }
impl SourceBuildingResourceDatabase {
    pub fn load(binding:SourceBuildingBinding,bytes:Vec<u8>)->Result<Self,BuildingResourceError>{
        let (length,hash,offsets,components,frame_count,completed)=match binding {
            SourceBuildingBinding::CdBsolaSelector6=>(35327,BSOLA_SHA256,[33753,34035,34745,35277],71,38,10),
            SourceBuildingBinding::CdBmineSelector11=>(34955,"218b117a4b0f84c6e5a362e5afdc46d29fb1525930c6131e2ffff3bcfcda2142",[32421,32691,33981,34905],129,66,20),
            SourceBuildingBinding::CdBpoweSelector3=>(77457,"4145c098f3d35dd9c39003ae8289c3d32fdb8d822327b2a23dfd52c60ca2a8cc",[75947,76127,76917,77407],79,35,9),
        };
        if bytes.len()!=length{return Err(BuildingResourceError::WrongLength{expected:length,actual:bytes.len()});}
        let actual=sha256_hex(&bytes);if actual!=hash{return Err(BuildingResourceError::WrongHash{expected:hash,actual});}
        let sections=AnbSections::parse(&bytes).map_err(BuildingResourceError::Anb)?;
        if sections.header_word()!=5||sections.offsets()!=offsets{return Err(BuildingResourceError::SectionLayout);}
        let metadata=sections.ui_metadata().map_err(BuildingResourceError::Anb)?;
        if metadata.animation_count()!=5||metadata.frame_count()!=frame_count||metadata.component_count()!=components{return Err(BuildingResourceError::SectionLayout);}
        let table=SpriteTable::from_anb(&bytes).map_err(BuildingResourceError::Anb)?;
        let mut animations=Vec::new();
        for (a,(first,last,count)) in [(0,28,3),(42,42+(completed-1)*14,completed as u16)].into_iter().enumerate(){
            let record=metadata.animation(a).map_err(BuildingResourceError::Anb)?;
            if (record.first_frame_offset,record.last_frame_offset,record.trailing_word)!=(first,last,count){return Err(BuildingResourceError::AnimationLayout);}
            let mut frames=Vec::new();
            for index in 0..count as usize {
                let frame=metadata.frame_at(first+index as u32*14).map_err(BuildingResourceError::Anb)?;
                let mut components=Vec::new();
                for ci in 0..usize::from(frame.component_count){
                    let component=metadata.frame_component(frame,ci).map_err(BuildingResourceError::Anb)?;
                    let sprite_record=component.sprite_offset as usize/6;
                    let sprite=table.decode(sprite_record,&bytes).map_err(BuildingResourceError::Anb)?;
                    components.push(BuildingAnimation0Component{record:component,sprite_record,sprite});
                }
                frames.push(BuildingAnimation0Frame{index:index as u8,hold:frame.byte_10,metadata:frame,
                    display_dimensions:frame.display_dimensions(),components});
            }
            animations.push(frames);
        }
        Ok(Self{binding,bytes,animations})
    }
    /// Bounded source common main image; no projection, sorting, shadow or status overlay.
    pub fn draw_common(&self,record:&[u8;67],context:crate::building_resources::BuildingCommonDrawContext,
        fade_row:Option<&[u8]>,blend_table:Option<&[u8]>,destination:&mut [u8],
        width:usize,height:usize,pitch:usize)->Result<crate::building_resources::BuildingCommonDrawReceipt,BuildingResourceError>{
        let flags=u16::from_le_bytes([record[0x31],record[0x32]]);
        let id=i16::from_le_bytes([record[0x0f],record[0x10]]);
        if record[6]!=4||record[0x18]>=4||record[0x2d]!=self.selector()||record[0x19]>1
            ||flags&0x0800!=0||record[0x15]&0x10!=0||!(0..256).contains(&id){
            return Err(BuildingResourceError::UnsupportedBuildingRecord);
        }
        if context.global_flags&!4!=0{return Err(BuildingResourceError::UnsupportedDrawFlags(context.global_flags));}
        let frame=self.animation(record[0x19] as usize)?.get(record[0x1b] as usize)
            .ok_or(BuildingResourceError::FrameOutOfBounds(record[0x1b] as usize))?;
        crate::building_resources::draw_common_frame(record,frame,context,fade_row,blend_table,destination,width,height,pitch)
    }
    pub fn selector(&self)->u8{match self.binding{SourceBuildingBinding::CdBsolaSelector6=>6,SourceBuildingBinding::CdBmineSelector11=>11,SourceBuildingBinding::CdBpoweSelector3=>3}}
    pub fn bytes(&self)->&[u8]{&self.bytes}
    pub fn animation(&self,index:usize)->Result<&[BuildingAnimation0Frame],BuildingResourceError>{
        self.animations.get(index).map(Vec::as_slice).ok_or(BuildingResourceError::AnimationLayout)
    }
}

#[cfg(test)]mod tests {
    use super::*;
    #[test]fn wrong_native_bindings_reject_before_framebuffer_changes(){
        let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../private/reference-run/run-006/drive-c/GAME/DATA/BUILDING");
        for (name,binding) in [("BSOLA",SourceBuildingBinding::CdBsolaSelector6),("BMINE",SourceBuildingBinding::CdBmineSelector11),("BPOWE",SourceBuildingBinding::CdBpoweSelector3)]{
            let db=SourceBuildingResourceDatabase::load(binding,std::fs::read(root.join(format!("{name}.ANB"))).unwrap()).unwrap();
            let mut base=[0;67];base[6]=4;base[0x2d]=db.selector();base[0x17]=30;
            for (field,value) in [(6,3),(0x2d,1),(0x18,4),(0x19,2),(0x0f,255),(0x1b,255)]{
                let mut record=base;record[field]=value;if field==0x0f{record[0x10]=255;}
                let mut pixels=[0x5a;64];
                assert!(db.draw_common(&record,crate::building_resources::BuildingCommonDrawContext{origin:[0,0],global_flags:0},None,None,&mut pixels,8,8,8).is_err());
                assert_eq!(pixels,[0x5a;64]);
            }
        }
    }
    #[test]fn exact_bsola_layout_and_rejection(){
        let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../private/reference-run/run-006/drive-c/GAME/DATA/BUILDING/BSOLA.ANB");
        let bytes=std::fs::read(path).unwrap();
        let db=SourceBuildingResourceDatabase::load(SourceBuildingBinding::CdBsolaSelector6,bytes.clone()).unwrap();
        assert_eq!(db.selector(),6);assert_eq!(db.animation(0).unwrap().len(),3);assert_eq!(db.animation(1).unwrap().len(),10);
        assert_eq!(db.animation(1).unwrap().iter().map(|f|f.hold).collect::<Vec<_>>(),[0,1,0,0,0,0,1,0,0,0]);
        assert!(db.animation(2).is_err());
        let mut changed=bytes.clone();changed[18]^=1;
        assert!(matches!(SourceBuildingResourceDatabase::load(SourceBuildingBinding::CdBsolaSelector6,changed),Err(BuildingResourceError::WrongHash{..})));
        assert!(matches!(SourceBuildingResourceDatabase::load(SourceBuildingBinding::CdBsolaSelector6,bytes[..100].to_vec()),Err(BuildingResourceError::WrongLength{..})));
    }
}
