//! Composition of the traced normal plant shadow and indexed8 foreground
//! passes. This does not decide visibility, ordering against other entities,
//! animation advancement, or any wider scene state.

use core::fmt;

use crate::entity_shadow::{draw_normal_component_shadow, ShadowDrawReceipt, ShadowError};
use crate::plant::{PlantColorInputs, PlantColorRoute, PlantFrame, PlantRenderError};
use crate::terrain::TerrainFadeTable;
use crate::terrain_modes::MainGlaTable;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlantDrawError {
    InvalidClip { width: usize, height: usize },
    InvalidPitch { width: usize, pitch: usize },
    DestinationExtentOverflow,
    ShortDestination { required: usize, actual: usize },
    AllocationFailed,
    Shadow(ShadowError),
    Foreground(PlantRenderError),
    PixelCountOverflow,
}

impl fmt::Display for PlantDrawError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for PlantDrawError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlantDrawReceipt {
    pub foreground_route: PlantColorRoute,
    pub foreground_pixels_written: usize,
    pub foreground_pixels_clipped: usize,
    /// Per-component shadow samples in table order. All entries precede the
    /// foreground pass, matching the traced 6C230 call order.
    pub component_shadows: Vec<ShadowDrawReceipt>,
    pub shadow_pixels_written: usize,
    pub shadow_pixels_clipped: usize,
}

/// Apply the closed normal-flag shadow layer followed by the plant color pass.
///
/// `origin` remains an explicit signed coordinate input: shadow receives its
/// signed16 value and adds loader-halved component offsets in signed32; the
/// foreground port independently performs its proved signed16 component-sum
/// narrowing. `destination` may be a view beginning inside a wider row-pitched
/// framebuffer; only the minimum addressed extent is required or committed.
/// The operation is staged so rejected flags, extents, or cache/table inputs
/// cannot leave partial shadows in the caller's frame.
pub fn draw_normal_plant(
    frame: &PlantFrame,
    destination: &mut [u8],
    clip: [usize; 2],
    pitch: usize,
    origin: [i32; 2],
    inputs: PlantColorInputs,
    fade: Option<&TerrainFadeTable<'_>>,
    gla: &MainGlaTable<'_>,
) -> Result<PlantDrawReceipt, PlantDrawError> {
    let [width, height] = clip;
    if !(1..=640).contains(&width) || !(1..=480).contains(&height) {
        return Err(PlantDrawError::InvalidClip { width, height });
    }
    if pitch < width {
        return Err(PlantDrawError::InvalidPitch { width, pitch });
    }
    let required = (height - 1).checked_mul(pitch).and_then(|value| value.checked_add(width))
        .ok_or(PlantDrawError::DestinationExtentOverflow)?;
    if destination.len() < required {
        return Err(PlantDrawError::ShortDestination { required, actual: destination.len() });
    }

    let mut staged = Vec::new();
    staged.try_reserve_exact(required).map_err(|_| PlantDrawError::AllocationFailed)?;
    staged.extend_from_slice(&destination[..required]);

    let mut shadow_receipts = Vec::new();
    shadow_receipts.try_reserve_exact(frame.component_count())
        .map_err(|_| PlantDrawError::AllocationFailed)?;
    let shadow_origin = [origin[0] as i16, origin[1] as i16];
    for component in frame.components() {
        let receipt = draw_normal_component_shadow(
            component.sprite,
            gla,
            shadow_origin,
            [component.local_x, component.local_y],
            0,
            clip,
            pitch,
            &mut staged,
        ).map_err(PlantDrawError::Shadow)?;
        shadow_receipts.push(receipt);
    }

    // The frame cache and pass inputs are checked on the staged destination;
    // any failure discards shadow changes without mutating the caller's bytes.
    let foreground = frame.draw_color_pass(
        &mut staged,
        width,
        height,
        pitch,
        origin,
        inputs,
        fade,
    ).map_err(PlantDrawError::Foreground)?;

    let mut shadow_pixels_written = 0usize;
    let mut shadow_pixels_clipped = 0usize;
    for receipt in &shadow_receipts {
        shadow_pixels_written = shadow_pixels_written.checked_add(receipt.pixels_written)
            .ok_or(PlantDrawError::PixelCountOverflow)?;
        shadow_pixels_clipped = shadow_pixels_clipped.checked_add(receipt.clipped_opaque_samples)
            .ok_or(PlantDrawError::PixelCountOverflow)?;
    }

    destination[..required].copy_from_slice(&staged);
    Ok(PlantDrawReceipt {
        foreground_route: foreground.route,
        foreground_pixels_written: foreground.opaque_pixels_written,
        foreground_pixels_clipped: foreground.opaque_pixels_clipped,
        component_shadows: shadow_receipts,
        shadow_pixels_written,
        shadow_pixels_clipped,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plant::PlantColorInputs;
    use crate::terrain_modes::MAIN_GLA_TABLE_BYTES;

    fn synthetic_anb() -> Vec<u8> {
        // Two 1x2 indexed8 sprites; the second has a transparent top row and
        // an opaque bottom row, so its odd-row shadow overlaps the first color.
        const DATA: usize = 18;
        const TABLE: usize = DATA + 11;
        const COMPONENTS: usize = TABLE + 12;
        const FRAMES: usize = COMPONENTS + 20;
        const ANIMATIONS: usize = FRAMES + 14;
        let mut bytes = vec![0u8; ANIMATIONS + 10];
        bytes[..2].copy_from_slice(&1u16.to_le_bytes());
        for (index, offset) in [TABLE, COMPONENTS, FRAMES, ANIMATIONS].into_iter().enumerate() {
            bytes[2 + index * 4..6 + index * 4].copy_from_slice(&(offset as u32).to_le_bytes());
        }
        bytes[DATA..DATA + 6].copy_from_slice(&[1, 3, 0, 1, 4, 0]);
        bytes[DATA + 6..DATA + 11].copy_from_slice(&[0xff, 0, 1, 5, 0]);
        for (index, data_offset) in [0u32, 6].into_iter().enumerate() {
            let entry = TABLE + index * 6;
            bytes[entry..entry + 4].copy_from_slice(&data_offset.to_le_bytes());
            bytes[entry + 4..entry + 6].copy_from_slice(&[1, 2]);
            let component = COMPONENTS + index * 10;
            bytes[component..component + 4].copy_from_slice(&((index * 6) as u32).to_le_bytes());
            bytes[component + 4..component + 6].copy_from_slice(&0i16.to_le_bytes());
            bytes[component + 6..component + 8].copy_from_slice(&0i16.to_le_bytes());
        }
        bytes[FRAMES..FRAMES + 4].copy_from_slice(&0u32.to_le_bytes());
        bytes[FRAMES + 4..FRAMES + 6].copy_from_slice(&2i16.to_le_bytes());
        bytes[FRAMES + 6..FRAMES + 8].copy_from_slice(&4i16.to_le_bytes());
        bytes[FRAMES + 11] = 2;
        bytes[ANIMATIONS..ANIMATIONS + 4].copy_from_slice(&0u32.to_le_bytes());
        bytes[ANIMATIONS + 4..ANIMATIONS + 8].copy_from_slice(&0u32.to_le_bytes());
        bytes
    }

    fn inputs(context: u8, entity: u8, native: u8, shade: u8) -> PlantColorInputs {
        PlantColorInputs {
            context_flags_at_16e0c4: context,
            entity_flags_at_15: entity,
            native_flags_at_2e: native,
            shade_at_17: shade,
        }
    }

    fn main_gla() -> MainGlaTable<'static> {
        let mut bytes = vec![0u8; MAIN_GLA_TABLE_BYTES];
        for destination in 0..256usize {
            bytes[destination] = (destination as u8).wrapping_add(50);
        }
        MainGlaTable::parse(Box::leak(bytes.into_boxed_slice())).unwrap()
    }

    #[test]
    fn applies_all_component_shadows_before_all_foreground_and_preserves_padding() {
        let anb = synthetic_anb();
        let frame = PlantFrame::parse_indexed8(&anb, 0, 0).unwrap();
        assert_eq!(frame.component_count(), 2);
        let gla = main_gla();
        let mut destination = [10u8; 6]; // 2x2 clip, pitch 4, exact minimum extent
        let receipt = draw_normal_plant(
            &frame,
            &mut destination,
            [2, 2],
            4,
            [0, 0],
            inputs(0, 0x02, 0, 35),
            None,
            &gla,
        ).unwrap();
        assert_eq!(receipt.foreground_route, PlantColorRoute::Direct);
        assert_eq!(receipt.component_shadows.len(), 2);
        assert_eq!((receipt.shadow_pixels_written, receipt.shadow_pixels_clipped), (2, 0));
        // Both shadow writes occur first; the first component's opaque top
        // foreground pixel is then restored to index 3.
        assert_eq!(destination[0], 3);
        assert_eq!(destination[1], 10);
        assert_eq!(destination[4], 5);
        assert_eq!(destination[2..4], [10, 10]);
        assert_eq!(destination[5], 10);
    }

    #[test]
    fn rejected_foreground_inputs_leave_shadow_and_destination_unmodified() {
        let anb = synthetic_anb();
        let frame = PlantFrame::parse_indexed8(&anb, 0, 0).unwrap();
        let gla = main_gla();
        let mut destination = [17u8; 6];
        let before = destination;
        let error = draw_normal_plant(
            &frame,
            &mut destination,
            [2, 2],
            4,
            [0, 0],
            inputs(0x02, 0, 0, 0),
            None,
            &gla,
        ).unwrap_err();
        assert!(matches!(error, PlantDrawError::Foreground(PlantRenderError::UnsupportedContextFlags { .. })));
        assert_eq!(destination, before);
    }

    #[test]
    fn shaded_dispatch_requires_fade_before_any_shadow_write() {
        let anb = synthetic_anb();
        let frame = PlantFrame::parse_indexed8(&anb, 0, 0).unwrap();
        let gla = main_gla();
        let mut destination = [17u8; 6];
        let before = destination;
        assert!(matches!(
            draw_normal_plant(&frame, &mut destination, [2, 2], 4, [0, 0], inputs(0, 0, 0, 7), None, &gla),
            Err(PlantDrawError::Foreground(PlantRenderError::FadeTableRequired { shade: 7 }))
        ));
        assert_eq!(destination, before);
    }

    #[test]
    fn minimal_right_view_extent_and_bad_bounds_are_atomic() {
        let anb = synthetic_anb();
        let frame = PlantFrame::parse_indexed8(&anb, 0, 0).unwrap();
        let gla = main_gla();
        let mut view = [8u8; 7]; // width2, height2, pitch5 requires7 bytes
        let receipt = draw_normal_plant(&frame, &mut view, [2, 2], 5, [0, 0], inputs(0, 0x02, 0, 35), None, &gla).unwrap();
        assert_eq!(receipt.foreground_pixels_written, 3);
        assert_eq!(view[2..5], [8, 8, 8]); // inter-row padding untouched
        let before = view;
        assert!(matches!(draw_normal_plant(&frame, &mut view[..6], [2, 2], 5, [0, 0], inputs(0, 0x02, 0, 35), None, &gla), Err(PlantDrawError::ShortDestination { .. })));
        assert_eq!(view, before);
    }
}
