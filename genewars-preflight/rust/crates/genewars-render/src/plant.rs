//! Bounded indexed8 foreground color pass for the traced family-1 plant path.
//!
//! This caches one explicitly selected animation/frame from a caller-supplied
//! ANB. It does not apply visibility, sorting, timing, or the preceding shadow
//! pass. Its inputs name the proven source/runtime flag bytes; resource
//! identity remains the caller's responsibility.

use genewars_assets::{AnbSections, Error as AssetError, IndexedSprite, SpriteTable};
use std::fmt;

use crate::terrain::{TerrainFadeTable, TerrainPrimitiveError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlantRenderError {
    Asset(AssetError),
    Terrain(TerrainPrimitiveError),
    AllocationFailed,
    UnsupportedFrameRange { animation: usize, first: u32, last: u32 },
    FrameSelectorOutOfBounds { animation: usize, selector: u8, frame_count: u32 },
    UnsupportedComponentFlags { frame: u32, component: usize, flags: u8 },
    UnsupportedContextFlags { value: u8, mask: u8 },
    UnsupportedEntityFlags { value: u8, mask: u8 },
    UnsupportedNativeFlags { value: u8, mask: u8 },
    FadeTableRequired { shade: u8 },
    InvalidDestination { width: usize, height: usize, pitch: usize, actual: usize },
    DestinationExtentOverflow,
    InvalidCachedSprite { width: u8, height: u8, indices: usize, opacity: usize },
    PixelCountOverflow,
}

impl fmt::Display for PlantRenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for PlantRenderError {}

/// A fully decoded, explicitly selected indexed8 frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlantFrame {
    animation_selector: u8,
    frame_selector: u8,
    frame_offset: u32,
    display_dimensions: (i16, i16),
    components: Vec<PlantFrameComponent>,
}

/// Borrowed decoded component data for consumers that implement another
/// explicitly proved pass, such as the separately modeled shadow routine.
#[derive(Debug, Clone, Copy)]
pub struct PlantFrameComponentRef<'a> {
    pub sprite: &'a IndexedSprite,
    /// Component offsets after the ANB loader's signed arithmetic SAR1.
    pub local_x: i16,
    pub local_y: i16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlantFrameComponent {
    sprite: IndexedSprite,
    local_x: i16,
    local_y: i16,
}

/// Input bytes consumed by the closed foreground color dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlantColorInputs {
    /// Byte at the proved context address; bit 1 must be clear for this path.
    pub context_flags_at_16e0c4: u8,
    /// Entity source byte at +0x15. Bit 1 forces direct color; bit 4 selects
    /// flashing and is excluded by this bounded normal-pass renderer.
    pub entity_flags_at_15: u8,
    /// Native runtime byte at +0x2E. Bit 1 selects jitter and is excluded.
    pub native_flags_at_2e: u8,
    /// Unsigned source shade byte at +0x17.
    pub shade_at_17: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlantColorRoute {
    Direct,
    Fade { shade: u8 },
}

/// The shadow routine runs before this foreground pass but is not part of this
/// renderer. A caller must not treat the color receipt as a complete entity draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlantShadowStatus {
    RequiredButNotRendered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlantFrameDrawReceipt {
    pub route: PlantColorRoute,
    pub opaque_pixels_written: usize,
    pub opaque_pixels_clipped: usize,
    pub shadow: PlantShadowStatus,
}

impl PlantFrame {
    /// Parse and cache one animation-relative frame from a caller-supplied ANB.
    ///
    /// The selector addresses an animation record; `frame_selector` is a
    /// 14-byte frame index relative to that animation's first-frame pointer.
    /// Component flags must be zero because this color path does not implement
    /// component transforms. The caller must supply one of the evidenced
    /// indexed8 resources; this routine does not hash-admit the file.
    pub fn parse_indexed8(
        anb: &[u8],
        animation_selector: u8,
        frame_selector: u8,
    ) -> Result<Self, PlantRenderError> {
        const FRAME_BYTES: u32 = 14;

        let sections = AnbSections::parse(anb).map_err(PlantRenderError::Asset)?;
        let metadata = sections.ui_metadata().map_err(PlantRenderError::Asset)?;
        let animation_index = usize::from(animation_selector);
        let animation = metadata.animation(animation_index).map_err(PlantRenderError::Asset)?;
        let span = animation.last_frame_offset.checked_sub(animation.first_frame_offset)
            .filter(|bytes| bytes % FRAME_BYTES == 0)
            .ok_or(PlantRenderError::UnsupportedFrameRange {
                animation: animation_index,
                first: animation.first_frame_offset,
                last: animation.last_frame_offset,
            })?;
        let frame_count = span / FRAME_BYTES + 1;
        if u32::from(frame_selector) >= frame_count {
            return Err(PlantRenderError::FrameSelectorOutOfBounds {
                animation: animation_index,
                selector: frame_selector,
                frame_count,
            });
        }
        let frame_offset = animation.first_frame_offset
            .checked_add(u32::from(frame_selector) * FRAME_BYTES)
            .ok_or(PlantRenderError::UnsupportedFrameRange {
                animation: animation_index,
                first: animation.first_frame_offset,
                last: animation.last_frame_offset,
            })?;
        let frame = metadata.frame_at(frame_offset).map_err(PlantRenderError::Asset)?;
        let sprite_table = SpriteTable::from_anb(anb).map_err(PlantRenderError::Asset)?;
        let mut components = Vec::new();
        components.try_reserve_exact(usize::from(frame.component_count))
            .map_err(|_| PlantRenderError::AllocationFailed)?;

        for local_index in 0..usize::from(frame.component_count) {
            let component = metadata.frame_component(frame, local_index).map_err(PlantRenderError::Asset)?;
            if component.flags != 0 {
                return Err(PlantRenderError::UnsupportedComponentFlags {
                    frame: frame_offset,
                    component: local_index,
                    flags: component.flags,
                });
            }
            let sprite_index = component.sprite_offset as usize / 6;
            let sprite = sprite_table.decode(sprite_index, anb).map_err(PlantRenderError::Asset)?;
            let (local_x, local_y) = component.local_position();
            components.push(PlantFrameComponent { sprite, local_x, local_y });
        }

        Ok(Self {
            animation_selector,
            frame_selector,
            frame_offset,
            display_dimensions: frame.display_dimensions(),
            components,
        })
    }

    pub fn animation_selector(&self) -> u8 { self.animation_selector }
    pub fn frame_selector(&self) -> u8 { self.frame_selector }
    pub fn frame_offset(&self) -> u32 { self.frame_offset }
    pub fn display_dimensions(&self) -> (i16, i16) { self.display_dimensions }
    pub fn component_count(&self) -> usize { self.components.len() }

    pub fn components(&self) -> impl Iterator<Item = PlantFrameComponentRef<'_>> {
        self.components.iter().map(|component| PlantFrameComponentRef {
            sprite: &component.sprite,
            local_x: component.local_x,
            local_y: component.local_y,
        })
    }

    /// Draw only the proved foreground color pass into an indexed framebuffer.
    ///
    /// The supplied i32 origin is first narrowed to signed 16 bits, as at the
    /// traced call boundary. Each already-SAR1 component offset is then added
    /// and the sum is narrowed to signed 16 bits. The indexed destination uses
    /// explicit width, height and pitch. It is left unchanged on validation
    /// errors; clipped positive literals are consumed by the cached decoder and
    /// do not modify the destination. Palette index zero is opaque when its
    /// RLE mask marks it literal.
    pub fn draw_color_pass(
        &self,
        destination: &mut [u8],
        width: usize,
        height: usize,
        pitch: usize,
        origin: [i32; 2],
        inputs: PlantColorInputs,
        fade: Option<&TerrainFadeTable<'_>>,
    ) -> Result<PlantFrameDrawReceipt, PlantRenderError> {
        validate_flags(inputs)?;
        let route = if inputs.entity_flags_at_15 & 0x02 != 0 || inputs.shade_at_17 >= 30 {
            PlantColorRoute::Direct
        } else {
            if fade.is_none() {
                return Err(PlantRenderError::FadeTableRequired { shade: inputs.shade_at_17 });
            }
            PlantColorRoute::Fade { shade: inputs.shade_at_17 }
        };
        // A view may start inside a wider framebuffer. Only the addressed
        // portion of the last row is required, not its trailing padding.
        let required = if width == 0 || height == 0 { 0 } else {
            (height - 1).checked_mul(pitch).and_then(|v| v.checked_add(width))
                .ok_or(PlantRenderError::DestinationExtentOverflow)?
        };
        if width > i64::MAX as usize || height > i64::MAX as usize
            || pitch < width || destination.len() < required
        {
            return Err(PlantRenderError::InvalidDestination { width, height, pitch, actual: destination.len() });
        }

        let origin_x = origin[0] as i16;
        let origin_y = origin[1] as i16;
        let mut total_opaque = 0usize;
        let mut visible_opaque = 0usize;
        for component in &self.components {
            let sprite_pixels = usize::from(component.sprite.width)
                .checked_mul(usize::from(component.sprite.height))
                .ok_or(PlantRenderError::PixelCountOverflow)?;
            if component.sprite.indices.len() != sprite_pixels || component.sprite.opaque.len() != sprite_pixels {
                return Err(PlantRenderError::InvalidCachedSprite {
                    width: component.sprite.width,
                    height: component.sprite.height,
                    indices: component.sprite.indices.len(),
                    opacity: component.sprite.opaque.len(),
                });
            }
            let x = origin_x.wrapping_add(component.local_x);
            let y = origin_y.wrapping_add(component.local_y);
            let (visible, opaque) = count_component_pixels(&component.sprite, x, y, width, height)?;
            visible_opaque = visible_opaque.checked_add(visible).ok_or(PlantRenderError::PixelCountOverflow)?;
            total_opaque = total_opaque.checked_add(opaque).ok_or(PlantRenderError::PixelCountOverflow)?;
        }

        let fade_map = match route {
            PlantColorRoute::Direct => None,
            PlantColorRoute::Fade { shade } => {
                let fade = fade.expect("shaded route was validated");
                let mut map = [0u8; 256];
                for (source, mapped) in map.iter_mut().enumerate() {
                    *mapped = fade.map_palette_index(shade, source as u8)
                        .map_err(PlantRenderError::Terrain)?;
                }
                Some(map)
            }
        };

        // All possible errors have been checked before the first destination write.
        for component in &self.components {
            let x = origin_x.wrapping_add(component.local_x);
            let y = origin_y.wrapping_add(component.local_y);
            draw_component(component, destination, pitch, width, height, x, y, fade_map.as_ref());
        }

        Ok(PlantFrameDrawReceipt {
            route,
            opaque_pixels_written: visible_opaque,
            opaque_pixels_clipped: total_opaque - visible_opaque,
            shadow: PlantShadowStatus::RequiredButNotRendered,
        })
    }
}

fn validate_flags(inputs: PlantColorInputs) -> Result<(), PlantRenderError> {
    for (value, mask, name) in [
        (inputs.context_flags_at_16e0c4, 0x02, "context_flags_at_16e0c4"),
        (inputs.entity_flags_at_15, 0x10, "entity_flags_at_15"),
        (inputs.native_flags_at_2e, 0x02, "native_flags_at_2e"),
    ] {
        if value & mask != 0 {
            let error = match name {
                "context_flags_at_16e0c4" => PlantRenderError::UnsupportedContextFlags { value, mask },
                "entity_flags_at_15" => PlantRenderError::UnsupportedEntityFlags { value, mask },
                _ => PlantRenderError::UnsupportedNativeFlags { value, mask },
            };
            return Err(error);
        }
    }
    Ok(())
}

fn count_component_pixels(
    sprite: &IndexedSprite,
    x: i16,
    y: i16,
    width: usize,
    height: usize,
) -> Result<(usize, usize), PlantRenderError> {
    let pixels = usize::from(sprite.width).checked_mul(usize::from(sprite.height))
        .ok_or(PlantRenderError::PixelCountOverflow)?;
    if sprite.indices.len() != pixels || sprite.opaque.len() != pixels {
        return Err(PlantRenderError::InvalidCachedSprite {
            width: sprite.width,
            height: sprite.height,
            indices: sprite.indices.len(),
            opacity: sprite.opaque.len(),
        });
    }
    let mut total = 0usize;
    let mut visible = 0usize;
    let left = i64::from(x);
    let top = i64::from(y);
    for sy in 0..usize::from(sprite.height) {
        for sx in 0..usize::from(sprite.width) {
            if !sprite.opaque[sy * usize::from(sprite.width) + sx] { continue; }
            total = total.checked_add(1).ok_or(PlantRenderError::PixelCountOverflow)?;
            let dx = left + sx as i64;
            let dy = top + sy as i64;
            if dx >= 0 && dy >= 0 && dx < width as i64 && dy < height as i64 {
                visible = visible.checked_add(1).ok_or(PlantRenderError::PixelCountOverflow)?;
            }
        }
    }
    Ok((visible, total))
}

#[allow(clippy::too_many_arguments)]
fn draw_component(
    component: &PlantFrameComponent,
    destination: &mut [u8],
    pitch: usize,
    width: usize,
    height: usize,
    x: i16,
    y: i16,
    fade_map: Option<&[u8; 256]>,
) {
    let sprite = &component.sprite;
    let left = i64::from(x);
    let top = i64::from(y);
    for sy in 0..usize::from(sprite.height) {
        let dy = top + sy as i64;
        if dy < 0 || dy >= height as i64 { continue; }
        for sx in 0..usize::from(sprite.width) {
            let source = sy * usize::from(sprite.width) + sx;
            if !sprite.opaque[source] { continue; }
            let dx = left + sx as i64;
            if dx < 0 || dx >= width as i64 { continue; }
            let color = fade_map.map_or(sprite.indices[source], |map| map[usize::from(sprite.indices[source])]);
            destination[dy as usize * pitch + dx as usize] = color;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::{TERRAIN_FADE_ROW_BYTES, TERRAIN_FADE_TABLE_BYTES};

    fn synthetic_anb() -> Vec<u8> {
        // Two one-pixel RLE sprites, one one-component frame and one animation.
        const RASTER_BASE: usize = 18;
        const SPRITE_TABLE: usize = RASTER_BASE + 6;
        const COMPONENTS: usize = SPRITE_TABLE + 12;
        const FRAMES: usize = COMPONENTS + 10;
        const ANIMATIONS: usize = FRAMES + 14;
        let mut anb = vec![0u8; ANIMATIONS + 10];
        anb[..2].copy_from_slice(&1u16.to_le_bytes());
        for (i, offset) in [SPRITE_TABLE, COMPONENTS, FRAMES, ANIMATIONS].into_iter().enumerate() {
            anb[2 + i * 4..6 + i * 4].copy_from_slice(&(offset as u32).to_le_bytes());
        }
        anb[RASTER_BASE..RASTER_BASE + 6].copy_from_slice(&[1, 0, 0, 1, 7, 0]);
        for index in 0..2usize {
            let table = SPRITE_TABLE + index * 6;
            let data_offset = if index == 0 { 0 } else { 3 };
            anb[table..table + 4].copy_from_slice(&(data_offset as u32).to_le_bytes());
            anb[table + 4..table + 6].copy_from_slice(&[1, 1]);
        }
        anb[COMPONENTS..COMPONENTS + 4].copy_from_slice(&0u32.to_le_bytes());
        // The source position is already represented as signed half-coordinate words.
        anb[COMPONENTS + 4..COMPONENTS + 6].copy_from_slice(&(-2i16).to_le_bytes());
        anb[COMPONENTS + 6..COMPONENTS + 8].copy_from_slice(&2i16.to_le_bytes());
        anb[FRAMES..FRAMES + 4].copy_from_slice(&0u32.to_le_bytes());
        anb[FRAMES + 4..FRAMES + 6].copy_from_slice(&2i16.to_le_bytes());
        anb[FRAMES + 6..FRAMES + 8].copy_from_slice(&2i16.to_le_bytes());
        anb[FRAMES + 11] = 1;
        anb[ANIMATIONS..ANIMATIONS + 4].copy_from_slice(&0u32.to_le_bytes());
        anb[ANIMATIONS + 4..ANIMATIONS + 8].copy_from_slice(&0u32.to_le_bytes());
        anb
    }

    fn inputs(flags: u8, shade: u8) -> PlantColorInputs {
        PlantColorInputs {
            context_flags_at_16e0c4: 0,
            entity_flags_at_15: flags,
            native_flags_at_2e: 0,
            shade_at_17: shade,
        }
    }

    fn fade() -> TerrainFadeTable<'static> {
        let mut bytes = vec![0u8; TERRAIN_FADE_TABLE_BYTES];
        for source in 0..TERRAIN_FADE_ROW_BYTES {
            bytes[3 * TERRAIN_FADE_ROW_BYTES + source] = 200u8.wrapping_add(source as u8);
        }
        TerrainFadeTable::parse(Box::leak(bytes.into_boxed_slice())).unwrap()
    }

    #[test]
    fn parses_explicit_animation_relative_frame_and_component_offsets() {
        let anb = synthetic_anb();
        let frame = PlantFrame::parse_indexed8(&anb, 0, 0).unwrap();
        assert_eq!((frame.animation_selector(), frame.frame_selector(), frame.frame_offset()), (0, 0, 0));
        assert_eq!(frame.display_dimensions(), (1, 1));
        assert_eq!(frame.component_count(), 1);
        assert!(matches!(PlantFrame::parse_indexed8(&anb, 0, 1), Err(PlantRenderError::FrameSelectorOutOfBounds { .. })));
        assert!(matches!(PlantFrame::parse_indexed8(&anb, 1, 0), Err(PlantRenderError::Asset(_))));
    }

    #[test]
    fn fade_and_direct_color_paths_preserve_index_zero_opacity_and_clip() {
        let anb = synthetic_anb();
        let frame = PlantFrame::parse_indexed8(&anb, 0, 0).unwrap();
        let mut destination = [9u8; 4];
        let result = frame.draw_color_pass(&mut destination, 2, 2, 2, [1, 0], inputs(0, 3), Some(&fade())).unwrap();
        assert_eq!(result.route, PlantColorRoute::Fade { shade: 3 });
        assert_eq!(result.opaque_pixels_written, 1);
        assert_eq!(result.opaque_pixels_clipped, 0);
        assert_eq!(result.shadow, PlantShadowStatus::RequiredButNotRendered);
        assert_eq!(destination, [9, 9, 200, 9]);

        let direct = frame.draw_color_pass(&mut destination, 2, 2, 2, [1, 0], inputs(0x02, 3), None).unwrap();
        assert_eq!(direct.route, PlantColorRoute::Direct);
        assert_eq!(destination[2], 0); // opaque source literal zero is written
    }

    #[test]
    fn rejects_unsupported_flags_and_bad_destination_before_any_write() {
        let anb = synthetic_anb();
        let frame = PlantFrame::parse_indexed8(&anb, 0, 0).unwrap();
        for invalid in [
            PlantColorInputs { context_flags_at_16e0c4: 2, ..inputs(0, 35) },
            PlantColorInputs { entity_flags_at_15: 0x10, ..inputs(0, 35) },
            PlantColorInputs { native_flags_at_2e: 2, ..inputs(0, 35) },
        ] {
            let mut destination = [42u8; 4];
            assert!(frame.draw_color_pass(&mut destination, 2, 2, 2, [0, 0], invalid, None).is_err());
            assert_eq!(destination, [42; 4]);
        }
        let mut destination = [42u8; 3];
        assert!(matches!(frame.draw_color_pass(&mut destination, 2, 2, 2, [0, 0], inputs(0, 35), None), Err(PlantRenderError::InvalidDestination { .. })));
        assert_eq!(destination, [42; 3]);
    }

    #[test]
    fn accepts_offset_view_without_trailing_padding_and_preserves_other_bytes() {
        let frame = PlantFrame::parse_indexed8(&synthetic_anb(), 0, 0).unwrap();
        let mut destination = [42u8; 6];
        let result = frame.draw_color_pass(&mut destination, 2, 2, 4, [1, 0], inputs(2, 0), None).unwrap();
        assert_eq!(result.opaque_pixels_written, 1);
        assert_eq!(destination, [42, 42, 42, 42, 0, 42]);
        let before = destination;
        assert!(frame.draw_color_pass(&mut destination[..5], 2, 2, 4, [1, 0], inputs(2, 0), None).is_err());
        assert_eq!(destination, before);
    }
}
