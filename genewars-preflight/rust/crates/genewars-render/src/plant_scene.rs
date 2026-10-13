//! Transactional composition of already-selected family1 plant frames.
//!
//! Callers provide the explicit draw order, projected origins, color-pass
//! inputs, and per-record draw decision. This module does not infer any game
//! admission, visibility, queueing, shade, or initialization behavior.

use core::fmt;

use crate::plant::{PlantColorInputs, PlantFrame};
use crate::plant_draw::{draw_normal_plant, PlantDrawError, PlantDrawReceipt};
use crate::terrain::TerrainFadeTable;
use crate::terrain_modes::MainGlaTable;

#[derive(Debug, Clone, Copy)]
pub struct PlantSceneRecord<'a> {
    pub id: u32,
    pub frame: &'a PlantFrame,
    pub origin: [i32; 2],
    pub inputs: PlantColorInputs,
    /// Explicit caller-provided decision; no draw gate is modeled here.
    pub draw: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlantSceneRecordStatus {
    SkippedByCaller,
    Drawn(PlantDrawReceipt),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlantSceneRecordReceipt {
    pub id: u32,
    pub status: PlantSceneRecordStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlantSceneReceipt {
    /// Receipts remain in caller-supplied draw order, including skipped items.
    pub records: Vec<PlantSceneRecordReceipt>,
    pub foreground_pixels_written: usize,
    pub shadow_pixels_written: usize,
    pub foreground_pixels_clipped: usize,
    pub shadow_pixels_clipped: usize,
    /// Summed pixel-write operations across draws; overlapping writes count
    /// once per record and this is not a count of distinct final pixels.
    pub pixels_written: usize,
    pub pixels_clipped: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlantSceneError {
    InvalidClip { width: usize, height: usize },
    InvalidPitch { width: usize, pitch: usize },
    ExtentOverflow,
    ShortDestination { required: usize, actual: usize },
    AllocationFailed,
    AggregateOverflow,
    RecordDraw { id: u32, source: PlantDrawError },
}

impl fmt::Display for PlantSceneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for PlantSceneError {}

/// Draw an explicit ordered set of already-initialized family1 frames.
///
/// All records draw into a temporary indexed view. Only after every requested
/// draw and receipt aggregation succeeds are addressed bytes copied to the
/// caller. The minimum addressed extent is `(height - 1) * pitch + width`;
/// bytes between rows and any tail remain unchanged. Even an empty or fully
/// skipped scene validates its clip, pitch, and destination extent.
pub fn draw_explicit_family1_scene(
    records: &[PlantSceneRecord<'_>],
    destination: &mut [u8],
    clip: [usize; 2],
    pitch: usize,
    fade: Option<&TerrainFadeTable<'_>>,
    gla: &MainGlaTable<'_>,
) -> Result<PlantSceneReceipt, PlantSceneError> {
    let [width, height] = clip;
    if !(1..=640).contains(&width) || !(1..=480).contains(&height) {
        return Err(PlantSceneError::InvalidClip { width, height });
    }
    if pitch < width {
        return Err(PlantSceneError::InvalidPitch { width, pitch });
    }
    let required = (height - 1)
        .checked_mul(pitch)
        .and_then(|value| value.checked_add(width))
        .ok_or(PlantSceneError::ExtentOverflow)?;
    if destination.len() < required {
        return Err(PlantSceneError::ShortDestination {
            required,
            actual: destination.len(),
        });
    }

    let mut staged = Vec::new();
    staged
        .try_reserve_exact(required)
        .map_err(|_| PlantSceneError::AllocationFailed)?;
    staged.extend_from_slice(&destination[..required]);

    let mut receipts = Vec::new();
    receipts
        .try_reserve_exact(records.len())
        .map_err(|_| PlantSceneError::AllocationFailed)?;
    let mut foreground_pixels_written = 0usize;
    let mut shadow_pixels_written = 0usize;
    let mut foreground_pixels_clipped = 0usize;
    let mut shadow_pixels_clipped = 0usize;

    for record in records {
        let status = if !record.draw {
            PlantSceneRecordStatus::SkippedByCaller
        } else {
            let receipt = draw_normal_plant(
                record.frame,
                &mut staged,
                clip,
                pitch,
                record.origin,
                record.inputs,
                fade,
                gla,
            )
            .map_err(|source| PlantSceneError::RecordDraw {
                id: record.id,
                source,
            })?;
            foreground_pixels_written = foreground_pixels_written
                .checked_add(receipt.foreground_pixels_written)
                .ok_or(PlantSceneError::AggregateOverflow)?;
            shadow_pixels_written = shadow_pixels_written
                .checked_add(receipt.shadow_pixels_written)
                .ok_or(PlantSceneError::AggregateOverflow)?;
            foreground_pixels_clipped = foreground_pixels_clipped
                .checked_add(receipt.foreground_pixels_clipped)
                .ok_or(PlantSceneError::AggregateOverflow)?;
            shadow_pixels_clipped = shadow_pixels_clipped
                .checked_add(receipt.shadow_pixels_clipped)
                .ok_or(PlantSceneError::AggregateOverflow)?;
            PlantSceneRecordStatus::Drawn(receipt)
        };
        receipts.push(PlantSceneRecordReceipt {
            id: record.id,
            status,
        });
    }

    let pixels_written = foreground_pixels_written
        .checked_add(shadow_pixels_written)
        .ok_or(PlantSceneError::AggregateOverflow)?;
    let pixels_clipped = foreground_pixels_clipped
        .checked_add(shadow_pixels_clipped)
        .ok_or(PlantSceneError::AggregateOverflow)?;
    destination[..required].copy_from_slice(&staged);
    Ok(PlantSceneReceipt {
        records: receipts,
        foreground_pixels_written,
        shadow_pixels_written,
        foreground_pixels_clipped,
        shadow_pixels_clipped,
        pixels_written,
        pixels_clipped,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plant::{PlantColorRoute, PlantRenderError};
    use crate::terrain_modes::MAIN_GLA_TABLE_BYTES;

    fn one_pixel_anb(pixel: u8) -> Vec<u8> {
        const DATA: usize = 18;
        const TABLE: usize = DATA + 3;
        const COMPONENTS: usize = TABLE + 6;
        const FRAMES: usize = COMPONENTS + 10;
        const ANIMATIONS: usize = FRAMES + 14;
        let mut bytes = vec![0u8; ANIMATIONS + 10];
        bytes[..2].copy_from_slice(&1u16.to_le_bytes());
        for (index, offset) in [TABLE, COMPONENTS, FRAMES, ANIMATIONS]
            .into_iter()
            .enumerate()
        {
            bytes[2 + index * 4..6 + index * 4]
                .copy_from_slice(&(offset as u32).to_le_bytes());
        }
        // One indexed8 literal followed by row end.
        bytes[DATA..DATA + 3].copy_from_slice(&[1, pixel, 0]);
        bytes[TABLE..TABLE + 6].copy_from_slice(&[0, 0, 0, 0, 1, 1]);
        bytes[COMPONENTS..COMPONENTS + 4].copy_from_slice(&0u32.to_le_bytes());
        bytes[FRAMES..FRAMES + 4].copy_from_slice(&0u32.to_le_bytes());
        bytes[FRAMES + 4..FRAMES + 6].copy_from_slice(&1i16.to_le_bytes());
        bytes[FRAMES + 6..FRAMES + 8].copy_from_slice(&1i16.to_le_bytes());
        bytes[FRAMES + 11] = 1;
        bytes[ANIMATIONS..ANIMATIONS + 4].copy_from_slice(&0u32.to_le_bytes());
        bytes[ANIMATIONS + 4..ANIMATIONS + 8].copy_from_slice(&0u32.to_le_bytes());
        bytes[ANIMATIONS + 8..ANIMATIONS + 10].copy_from_slice(&1u16.to_le_bytes());
        bytes
    }

    fn inputs() -> PlantColorInputs {
        PlantColorInputs {
            context_flags_at_16e0c4: 0,
            entity_flags_at_15: 0x02,
            native_flags_at_2e: 0,
            shade_at_17: 35,
        }
    }

    fn gla() -> MainGlaTable<'static> {
        MainGlaTable::parse(Box::leak(vec![0; MAIN_GLA_TABLE_BYTES].into_boxed_slice())).unwrap()
    }

    #[test]
    fn explicit_order_controls_overlap_and_receipts_preserve_skips_and_padding() {
        let anb_a = one_pixel_anb(3);
        let anb_b = one_pixel_anb(9);
        let frame_a = PlantFrame::parse_indexed8(&anb_a, 0, 0).unwrap();
        let frame_b = PlantFrame::parse_indexed8(&anb_b, 0, 0).unwrap();
        let gla = gla();
        let mut frame = [41u8; 8]; // 2x2 image, pitch4; row padding and tail
        let records = [
            PlantSceneRecord { id: 71, frame: &frame_a, origin: [0, 0], inputs: inputs(), draw: true },
            PlantSceneRecord { id: 72, frame: &frame_b, origin: [0, 0], inputs: inputs(), draw: true },
            PlantSceneRecord { id: 73, frame: &frame_a, origin: [1, 0], inputs: inputs(), draw: false },
        ];
        let receipt = draw_explicit_family1_scene(&records, &mut frame, [2, 2], 4, None, &gla).unwrap();
        assert_eq!(frame[0], 9); // later caller record covers earlier literal
        assert_eq!(frame[1], 41);
        assert_eq!(&frame[2..4], &[41, 41]);
        assert_eq!(&frame[4..], &[41; 4]);
        assert_eq!(receipt.records.iter().map(|row| row.id).collect::<Vec<_>>(), [71, 72, 73]);
        assert!(matches!(receipt.records[0].status, PlantSceneRecordStatus::Drawn(_)));
        assert_eq!(receipt.records[2].status, PlantSceneRecordStatus::SkippedByCaller);
        assert_eq!((receipt.foreground_pixels_written, receipt.shadow_pixels_written, receipt.pixels_written), (2, 0, 2));
        assert_eq!((receipt.foreground_pixels_clipped, receipt.shadow_pixels_clipped, receipt.pixels_clipped), (0, 0, 0));
        match &receipt.records[0].status {
            PlantSceneRecordStatus::Drawn(draw) => assert_eq!(draw.foreground_route, PlantColorRoute::Direct),
            PlantSceneRecordStatus::SkippedByCaller => panic!(),
        }

        let mut reversed = [41u8; 8];
        let reverse_records = [
            PlantSceneRecord { id: 72, frame: &frame_b, origin: [0, 0], inputs: inputs(), draw: true },
            PlantSceneRecord { id: 71, frame: &frame_a, origin: [0, 0], inputs: inputs(), draw: true },
        ];
        draw_explicit_family1_scene(&reverse_records, &mut reversed, [2, 2], 4, None, &gla).unwrap();
        assert_eq!(reversed[0], 3); // reversing explicit order reverses the overlap winner
    }

    #[test]
    fn later_invalid_record_leaves_entire_destination_and_padding_unchanged() {
        let anb_a = one_pixel_anb(3);
        let anb_b = one_pixel_anb(9);
        let frame_a = PlantFrame::parse_indexed8(&anb_a, 0, 0).unwrap();
        let frame_b = PlantFrame::parse_indexed8(&anb_b, 0, 0).unwrap();
        let gla = gla();
        let mut destination = [77u8; 8];
        let before = destination;
        let records = [
            PlantSceneRecord { id: 1, frame: &frame_a, origin: [0, 0], inputs: inputs(), draw: true },
            PlantSceneRecord { id: 2, frame: &frame_b, origin: [1, 0], inputs: PlantColorInputs { context_flags_at_16e0c4: 2, ..inputs() }, draw: true },
        ];
        let error = draw_explicit_family1_scene(&records, &mut destination, [2, 2], 4, None, &gla).unwrap_err();
        assert!(matches!(error, PlantSceneError::RecordDraw { id: 2, source: PlantDrawError::Foreground(PlantRenderError::UnsupportedContextFlags { .. }) }));
        assert_eq!(destination, before);
    }

    #[test]
    fn empty_or_skipped_scene_still_validates_geometry_and_minimum_extent() {
        let anb = one_pixel_anb(4);
        let frame = PlantFrame::parse_indexed8(&anb, 0, 0).unwrap();
        let gla = gla();
        let mut destination = [5u8; 6];
        let skipped = [PlantSceneRecord { id: 5, frame: &frame, origin: [0, 0], inputs: inputs(), draw: false }];
        let result = draw_explicit_family1_scene(&skipped, &mut destination, [2, 2], 4, None, &gla).unwrap();
        assert_eq!(result.records[0].status, PlantSceneRecordStatus::SkippedByCaller);
        assert_eq!(result.pixels_written, 0);
        assert_eq!(draw_explicit_family1_scene(&[], &mut destination, [2, 2], 4, None, &gla), Ok(PlantSceneReceipt { records: vec![], foreground_pixels_written: 0, shadow_pixels_written: 0, foreground_pixels_clipped: 0, shadow_pixels_clipped: 0, pixels_written: 0, pixels_clipped: 0 }));
        assert!(matches!(draw_explicit_family1_scene(&[], &mut destination[..5], [2, 2], 4, None, &gla), Err(PlantSceneError::ShortDestination { required: 6, actual: 5 })));
        assert!(matches!(draw_explicit_family1_scene(&[], &mut destination, [0, 2], 4, None, &gla), Err(PlantSceneError::InvalidClip { .. })));
    }
}
