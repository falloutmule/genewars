//! Normal and component-horizontal-flip shadows from CD0x96AC0 -> 0x92F80.
//!
//! Admission: drawing enabled, context/component flags zero, extra argument
//! zero, positive clip within640x480, and a previously decoded indexed8 sprite.
//! Visibility, frame selection and foreground drawing are separate contracts.
use genewars_assets::IndexedSprite;
use crate::terrain_modes::MainGlaTable;
use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShadowError {
    InvalidClip { width: usize, height: usize },
    InvalidPitch { width: usize, pitch: usize },
    ExtentOverflow,
    ShortDestination { required: usize, actual: usize },
    InvalidSpritePlanes { required: usize, indices: usize, opacity: usize },
}

impl fmt::Display for ShadowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for ShadowError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShadowDrawReceipt {
    /// Full-width component position; unlike foreground it is not narrowed.
    pub component_position: [i32; 2],
    pub shadow_height: usize,
    pub opaque_samples: usize,
    pub pixels_written: usize,
    pub clipped_opaque_samples: usize,
}

/// Apply the normal component's shadow to an indexed destination.
///
/// Offsets are already loader-halved signed values. The shadow halves Y a
/// second time with truncation toward zero, adds using full signed32 values,
/// and samples rows1,3,5,... of the source opacity mask. The final odd source
/// row is discarded when height is odd. Literal palette zero remains opaque;
/// source color values do not influence the shadow lookup.
///
/// All malformed extents are rejected before any destination write. Row
/// padding and clipped samples preserve their previous bytes.
pub fn draw_normal_component_shadow(
    sprite: &IndexedSprite, table: &MainGlaTable<'_>, origin: [i16; 2],
    component_offset: [i16; 2], color: u8, clip: [usize; 2], pitch: usize,
    destination: &mut [u8],
) -> Result<ShadowDrawReceipt, ShadowError> {
    draw_component_shadow(sprite, table, origin, component_offset, color, clip, pitch, destination, false)
}

/// Global wrapper flags0, component flag1, extra argument0.96AC0 leaves
/// placement unchanged;92F80 reverses the horizontal opacity traversal.
/// This is not a globally flipped frame (which would also reflect offsets).
pub fn draw_hflipped_component_shadow(
    sprite: &IndexedSprite, table: &MainGlaTable<'_>, origin: [i16; 2],
    component_offset: [i16; 2], color: u8, clip: [usize; 2], pitch: usize,
    destination: &mut [u8],
) -> Result<ShadowDrawReceipt, ShadowError> {
    draw_component_shadow(sprite, table, origin, component_offset, color, clip, pitch, destination, true)
}

fn draw_component_shadow(
    sprite: &IndexedSprite, table: &MainGlaTable<'_>, origin: [i16; 2],
    component_offset: [i16; 2], color: u8, clip: [usize; 2], pitch: usize,
    destination: &mut [u8], horizontal_flip: bool,
) -> Result<ShadowDrawReceipt, ShadowError> {
    let [width, height] = clip;
    if !(1..=640).contains(&width) || !(1..=480).contains(&height) {
        return Err(ShadowError::InvalidClip { width, height });
    }
    if pitch < width { return Err(ShadowError::InvalidPitch { width, pitch }); }
    let required = (height - 1).checked_mul(pitch).and_then(|v| v.checked_add(width))
        .ok_or(ShadowError::ExtentOverflow)?;
    if destination.len() < required {
        return Err(ShadowError::ShortDestination { required, actual: destination.len() });
    }
    let (source_width, source_height) = (usize::from(sprite.width), usize::from(sprite.height));
    let source_pixels = source_width * source_height;
    if sprite.indices.len() != source_pixels || sprite.opaque.len() != source_pixels {
        return Err(ShadowError::InvalidSpritePlanes {
            required: source_pixels, indices: sprite.indices.len(), opacity: sprite.opaque.len(),
        });
    }
    let position = [i32::from(origin[0]) + i32::from(component_offset[0]),
        i32::from(origin[1]) + i32::from(component_offset[1]) / 2];
    let mut receipt = ShadowDrawReceipt { component_position: position,
        shadow_height: source_height / 2, opaque_samples: 0, pixels_written: 0, clipped_opaque_samples: 0 };
    for row in 0..receipt.shadow_height {
        let source_y = row * 2 + 1;
        let y = position[1] + row as i32;
        for x in 0..source_width {
            if !sprite.opaque[source_y * source_width + x] { continue; }
            receipt.opaque_samples += 1;
            let screen_x = position[0] + if horizontal_flip { (source_width - 1 - x) as i32 } else { x as i32 };
            if screen_x < 0 || y < 0 || screen_x >= width as i32 || y >= height as i32 {
                receipt.clipped_opaque_samples += 1;
                continue;
            }
            let index = y as usize * pitch + screen_x as usize;
            // The same MAIN.GLA is indexed in the opposite order to terrain
            // mode15: shadow uses color*256 + previous_destination.
            destination[index] = table.map_destination(color, destination[index]);
            receipt.pixels_written += 1;
        }
    }
    Ok(receipt)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sprite() -> IndexedSprite {
        IndexedSprite { width: 3, height: 5, indices: vec![0; 15],
            opaque: vec![true, true, true, true, false, true, true, true, true,
                         false, true, false, true, true, true], consumed_bytes: 0 }
    }

    #[test]
    fn shadow_selects_odd_rows_and_uses_color_major_lookup_with_opaque_zero() {
        let data: Vec<u8> = (0..65536).map(|v| ((v / 256) as u8).wrapping_mul(3).wrapping_add(v as u8)).collect();
        let table = MainGlaTable::parse(&data).unwrap();
        let mut frame = vec![10; 24];
        let result = draw_normal_component_shadow(&sprite(), &table, [1, 1], [0, -1], 7, [4, 4], 6, &mut frame).unwrap();
        // -1/2 truncates to0, and row4 is discarded with odd height5.
        assert_eq!(result.component_position, [1, 1]);
        assert_eq!(result.shadow_height, 2);
        assert_eq!((result.opaque_samples, result.pixels_written, result.clipped_opaque_samples), (3, 3, 0));
        for (index, &value) in frame.iter().enumerate() {
            assert_eq!(value, if [7, 9, 14].contains(&index) { 31 } else { 10 });
        }
    }

    #[test]
    fn clipping_and_full_width_component_addition_preserve_padding() {
        let data = vec![99; 65536];
        let table = MainGlaTable::parse(&data).unwrap();
        let mut frame = vec![11; 18];
        let result = draw_normal_component_shadow(&sprite(), &table, [-1, -1], [0, 0], 0, [4, 3], 6, &mut frame).unwrap();
        assert_eq!((result.pixels_written, result.clipped_opaque_samples), (1, 2));
        assert_eq!(frame[0], 99);
        assert!(frame[1..].iter().all(|&v| v == 11));
        let before = frame.clone();
        let result = draw_normal_component_shadow(&sprite(), &table, [32767, 0], [32767, 0], 0, [4, 3], 6, &mut frame).unwrap();
        assert_eq!(result.component_position, [65534, 0]);
        assert_eq!(result.pixels_written, 0);
        assert_eq!(frame, before);
    }

    #[test]
    fn malformed_inputs_fail_atomically() {
        let data = vec![1; 65536];
        let table = MainGlaTable::parse(&data).unwrap();
        let mut frame = vec![88; 12];
        let before = frame.clone();
        let mut invalid = sprite(); invalid.opaque.pop();
        assert!(draw_normal_component_shadow(&invalid, &table, [0, 0], [0, 0], 0, [4, 3], 4, &mut frame).is_err());
        for (clip, pitch) in [([0, 3], 4), ([641, 1], 641), ([4, 3], 3), ([4, 3], usize::MAX), ([4, 4], 4)] {
            assert!(draw_normal_component_shadow(&sprite(), &table, [0, 0], [0, 0], 0, clip, pitch, &mut frame).is_err());
        }
        assert_eq!(frame, before);
    }
}
