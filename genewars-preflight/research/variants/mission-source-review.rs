use genewars_assets::mission_source::*;
use std::{env,fs,io::{BufWriter,Write}};
fn put(w:&mut impl Write,n:usize){w.write_all(&(n as u64).to_le_bytes()).unwrap();}
fn inspect(bytes:&[u8],w:&mut impl Write){
 let before=bytes.to_vec();let source=ConqMissionSource::parse(bytes).unwrap();assert_eq!(source.bytes(),bytes);assert_eq!(source.bytes().as_ptr(),bytes.as_ptr());
 put(w,source.copied_prefix().len());w.write_all(source.copied_prefix()).unwrap();
 for index in 0..4{for word in source.target_words(index).unwrap(){w.write_all(&word.to_le_bytes()).unwrap();}}
 for index in [4,usize::MAX]{assert!(matches!(source.target_words(index),Err(MissionSourceError::TargetIndexOutOfBounds{..})));}
 for (family,group) in source.groups().iter().enumerate(){
  let layout=group.layout();let direct=source.group(family).unwrap();assert_eq!(direct.bytes(),group.bytes());
  assert_eq!(group.bytes().as_ptr()as usize-bytes.as_ptr()as usize,layout.file_offset);
  for n in [usize::from(layout.family),layout.file_offset,layout.count,layout.packed_stride,group.bytes().len()]{put(w,n)}
  for index in 0..layout.count{let raw=group.record(index).unwrap();assert_eq!(raw.as_ptr()as usize-group.bytes().as_ptr()as usize,index*layout.packed_stride);put(w,raw.len());w.write_all(&[group.raw_byte_6(index).unwrap()]).unwrap();w.write_all(raw).unwrap();
   if family==1{let f=source.family1_sprite_fields(index).unwrap();put(w,f.record_index);for p in f.signed_position_words{w.write_all(&p.to_le_bytes()).unwrap();}w.write_all(&[f.animation_selector,f.frame_selector,f.resource_slot]).unwrap();}}
  for index in [layout.count,usize::MAX]{assert!(matches!(group.record(index),Err(MissionSourceError::RecordIndexOutOfBounds{..})));assert!(matches!(group.raw_byte_6(index),Err(MissionSourceError::RecordIndexOutOfBounds{..})));}
 }
 for family in [8,usize::MAX]{assert!(matches!(source.group(family),Err(MissionSourceError::GroupIndexOutOfBounds{..})));}
 for index in [512,usize::MAX]{assert!(matches!(source.family1_sprite_fields(index),Err(MissionSourceError::RecordIndexOutOfBounds{family:1,..})));}
 w.write_all(source.tail_globals()).unwrap();w.write_all(&[source.flags_byte(),source.active_count_byte()]).unwrap();w.write_all(source.viewport_snapshot()).unwrap();w.write_all(&source.extension_tag().to_le_bytes()).unwrap();put(w,source.extension_payload().len());w.write_all(source.extension_payload()).unwrap();assert_eq!(bytes,before);
}
fn main(){
 let a:Vec<String>=env::args().collect();let mut out=BufWriter::new(fs::File::create(&a[1]).unwrap());let first=fs::read(&a[2]).unwrap();
 for file in &a[2..]{let bytes=fs::read(file).unwrap();inspect(&bytes,&mut out);}
 for n in 0..first.len(){assert!(matches!(ConqMissionSource::parse(&first[..n]),Err(MissionSourceError::UnsupportedSourceVariantLength{expected:CONQ_MISSION_BYTES,actual})if actual==n));}
 for n in [CONQ_MISSION_BYTES+1,375830]{let bytes=vec![0;n];assert!(matches!(ConqMissionSource::parse(&bytes),Err(MissionSourceError::UnsupportedSourceVariantLength{actual,..})if actual==n));}
 for bit in 0..32{
  let mut bytes=first.clone();let original=u32::from_le_bytes(bytes[..4].try_into().unwrap());let changed=original^(1<<bit);bytes[..4].copy_from_slice(&changed.to_le_bytes());assert!(matches!(ConqMissionSource::parse(&bytes),Err(MissionSourceError::UnsupportedFastTag{actual})if actual==changed));
  let mut bytes=first.clone();let p=CONQ_EXTENSION_TAG_OFFSET;let original=u32::from_le_bytes(bytes[p..p+4].try_into().unwrap());let changed=original^(1<<bit);bytes[p..p+4].copy_from_slice(&changed.to_le_bytes());assert!(matches!(ConqMissionSource::parse(&bytes),Err(MissionSourceError::UnsupportedExtensionTag{actual})if actual==changed));
 }
 for pattern in 0..3{let mut bytes:Vec<u8>=(0..CONQ_MISSION_BYTES).map(|i|match pattern{0=>0,1=>255,_=>((i*151+97)&255)as u8}).collect();bytes[..4].copy_from_slice(&CONQ_MISSION_FAST_TAG.to_le_bytes());bytes[CONQ_EXTENSION_TAG_OFFSET..CONQ_EXTENSION_TAG_OFFSET+4].copy_from_slice(&CONQ_EXTENSION_TAG.to_le_bytes());inspect(&bytes,&mut out);}
 out.flush().unwrap();
 println!("all_truncations={} tags=64 opaque_patterns=3",first.len());
}
