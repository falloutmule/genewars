//! 5A140 host boundary for the hash-bound MULE database, with literal 1F260
//! height and 1E840 local frame clamp. World/control/visibility order is caller-owned.
use crate::runtime_resources::CreatureAnimationDatabase;
use genewars_assets::AnbSections;
use genewars_runtime::{creature_pick::{self, MulePickInput, PickComponent},
    creature_world::{CreatureAnimationBinding, CreatureRecord}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MulePickError {
    RecordBinding, RecordIdentity, CoordinateRange, Resource(String),
    InvalidFrameRange, InvalidComponents, Numeric(creature_pick::PickError),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MulePickReceipt {
    pub hit: bool,
    pub animation: usize,
    pub frame_index: usize,
    pub frame_count: usize,
    pub relative_height: i32,
    pub factor: u16,
    pub component_count: usize,
}
fn validate_record(record: &CreatureRecord, binding: Option<CreatureAnimationBinding>) -> Result<(), MulePickError> {
    if binding != Some(CreatureAnimationBinding::MuleSelector4)
        || record[6] != 1 || record[0x7a] != 4 || record[0x5e] >= 26 {
        return Err(MulePickError::RecordBinding);
    }
    if record[24] > 4 || u16::from_le_bytes(record[15..17].try_into().unwrap()) >= 300 {
        return Err(MulePickError::RecordIdentity);
    }
    Ok(())
}
/// The source getter uses 32-bit subtraction before the wrapper's arithmetic shift.
pub fn relative_height(record: &CreatureRecord) -> i32 {
    i32::from_le_bytes(record[0x27..0x2b].try_into().unwrap())
        .wrapping_sub(i32::from_le_bytes(record[0x11..0x15].try_into().unwrap()))
}
fn clamped_frame(first: u32, last: u32, trailing: u16, current: u8) -> Result<(usize, usize), MulePickError> {
    let count = usize::from(trailing as u8);
    if count == 0 || last < first || (last-first)%14 != 0
        || ((last-first)/14+1) as usize != count { return Err(MulePickError::InvalidFrameRange); }
    Ok((usize::from(current).min(count-1), count))
}
/// `ground` and `point` are absolute screen coordinates, including the panel
/// margin. No opacity, owner selection permission, render-state gate or queue
/// priority is inferred here. The immutable creature record retains its frame.
pub fn hit_mule_record(database: &CreatureAnimationDatabase, record: &CreatureRecord,
    binding: Option<CreatureAnimationBinding>, ground: [i32; 2], point: [i32; 2])
    -> Result<MulePickReceipt, MulePickError>
{
    validate_record(record, binding)?;
    let words = |v: [i32; 2]| -> Result<[i16; 2], MulePickError> {
        Ok([i16::try_from(v[0]).map_err(|_| MulePickError::CoordinateRange)?,
            i16::try_from(v[1]).map_err(|_| MulePickError::CoordinateRange)?])
    };
    let ground = words(ground)?; let point = words(point)?;
    let sections = AnbSections::parse(database.bytes()).map_err(|e| MulePickError::Resource(e.to_string()))?;
    let metadata = sections.ui_metadata().map_err(|e| MulePickError::Resource(e.to_string()))?;
    let animation = usize::from(record[0x19]);
    let selected = metadata.animation(animation).map_err(|e| MulePickError::Resource(e.to_string()))?;
    let (frame_index, frame_count) = clamped_frame(selected.first_frame_offset,
        selected.last_frame_offset, selected.trailing_word, record[0x1b])?;
    let frame = database.decode_frame_packed4(animation, frame_index, 0)
        .map_err(MulePickError::Resource)?;
    if frame.components.is_empty() || frame.components.len() != usize::from(frame.frame.component_count) {
        return Err(MulePickError::InvalidComponents);
    }
    let components: Vec<_> = frame.components.iter().map(|c| PickComponent {
        offset: [c.local_position.0, c.local_position.1], flags: c.flags,
        sprite_dimensions: Some([c.sprite.width, c.sprite.height]),
    }).collect();
    let relative_height = relative_height(record);
    let factor = creature_pick::mule_factor(record[0x76], record[0x77]);
    let hit = creature_pick::hit_mule(&components, MulePickInput {
        scale76: record[0x76], scale77: record[0x77], ground, point, relative_height,
    }).map_err(MulePickError::Numeric)?;
    Ok(MulePickReceipt { hit, animation, frame_index, frame_count,
        relative_height, factor, component_count: components.len() })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_frame_clamp_does_not_accept_empty_or_inconsistent_ranges() {
        assert_eq!(clamped_frame(28, 126, 8, 255), Ok((7,8)));
        assert_eq!(clamped_frame(28, 126, 8, 0), Ok((0,8)));
        assert_eq!(clamped_frame(28, 126, 0, 0), Err(MulePickError::InvalidFrameRange));
        assert_eq!(clamped_frame(28, 126, 7, 0), Err(MulePickError::InvalidFrameRange));
        let mut record = [0;124];record[0x27..0x2b].copy_from_slice(&i32::MIN.to_le_bytes());
        record[0x11..0x15].copy_from_slice(&1i32.to_le_bytes());
        assert_eq!(relative_height(&record), i32::MAX);
        assert_eq!(validate_record(&record,None), Err(MulePickError::RecordBinding));
        record[6]=1;record[0x7a]=4;record[0x5e]=26;
        assert_eq!(validate_record(&record,Some(CreatureAnimationBinding::MuleSelector4)), Err(MulePickError::RecordBinding));
    }
    #[test]
    #[ignore = "Requires explicit source-generated private cases via GENEWARS_MULE_PICK_CASES"]
    fn captured_mule_pick_resource_cases() {
        let path = std::env::var("GENEWARS_MULE_PICK_CASES").expect("explicit source fixture required");
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../private/integration/game-runtime");
        let (resources,_,_) = crate::runtime_resources::Resources::load(&root).unwrap();
        let data = std::fs::read_to_string(path).unwrap();let mut count=0;
        for line in data.lines() {
            let v:Vec<_>=line.split_whitespace().collect();assert_eq!(v.len(),12);
            let mut record=[0;124];for(i,b)in record.iter_mut().enumerate(){*b=u8::from_str_radix(&v[0][i*2..i*2+2],16).unwrap();}
            let original=record;let n=|i:usize|v[i].parse::<i32>().unwrap();
            let got=hit_mule_record(&resources.mule,&record,Some(CreatureAnimationBinding::MuleSelector4),[n(1),n(2)],[n(3),n(4)]).unwrap();
            assert_eq!((got.hit,got.animation,got.frame_index,got.frame_count,got.relative_height,got.factor,got.component_count),
                (n(5)!=0,n(6)as usize,n(7)as usize,n(8)as usize,n(9),n(10)as u16,n(11)as usize),"case {count}");
            assert_eq!(record,original);count+=1;
        }
        assert!(count>=288,"source fixture unexpectedly short");
        eprintln!("Mule resource picking source cases: {count} passed");
    }
}
