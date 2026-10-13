//! Explicit mode-specific terrain samplers layered on prepared triangle rows.
//!
//! Mode 6 has independent signed 32-bit U/V/S accumulators whose observable
//! fields are 16 fractional and 8 integer bits. It is not the packed mode-5
//! carry machine. Source index zero is transparent and does not consult FADE;
//! nonzero indices are mapped through the caller-supplied 64×256 table.

use core::fmt;

use crate::terrain::{
    classify_triangle, Mode5TriangleDrawReceipt, Mode5TriangleVertex,
    PreparedMode5Triangle, TerrainFadeTable, TerrainPrimitiveError,
    TriangleKind, MODE5_TEXTURE_ROW_STRIDE,
};

pub const MAIN_GLA_TABLE_BYTES: usize = 65_536;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mode6SpanInput {
    pub clip_width: i32,
    pub left_16_16: i32,
    pub right_16_16: i32,
    pub u_16_16: i32,
    pub v_16_16: i32,
    pub shade_16_16: i32,
    pub du_16_16: i32,
    pub dv_16_16: i32,
    pub dshade_16_16: i32,
    pub texture_base: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mode6SpanReceipt {
    pub start_x: i32,
    pub pixel_count: usize,
    pub opaque_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerrainModeError {
    Primitive(TerrainPrimitiveError),
    ClipWidthOutOfRange { clip_width: i32 },
    InvertedSpan { left_16_16: i32, right_16_16: i32 },
    EdgeOutOfRange { edge: i32 },
    OutputTooSmall { required: usize, available: usize },
    MainGlaLength { expected: usize, actual: usize },
}

impl fmt::Display for TerrainModeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for TerrainModeError {}

impl From<TerrainPrimitiveError> for TerrainModeError {
    fn from(value: TerrainPrimitiveError) -> Self { Self::Primitive(value) }
}

/// Geometry-only rows used by flat-color mode 0 and MAIN.GLA mode 15.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreparedGeometryRow {
    pub y: i32,
    pub left_16_16: i32,
    pub right_16_16: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedMode0_15Triangle {
    pub rows: Vec<PreparedGeometryRow>,
    pub clip_height: usize,
}

/// Borrowed exact-size `MAIN.GLA` lookup table used by mode 15.
#[derive(Debug, Clone, Copy)]
pub struct MainGlaTable<'a> {
    bytes: &'a [u8],
}

impl<'a> MainGlaTable<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, TerrainModeError> {
        if bytes.len() != MAIN_GLA_TABLE_BYTES {
            return Err(TerrainModeError::MainGlaLength { expected: MAIN_GLA_TABLE_BYTES, actual: bytes.len() });
        }
        Ok(Self { bytes })
    }

    pub fn map_destination(&self, destination: u8, color: u8) -> u8 {
        self.bytes[usize::from(destination) * 256 + usize::from(color)]
    }
}

/// Render one prepared mode-6 span as optional palette indices. `None`
/// represents a transparent source zero and leaves the destination untouched.
/// Texture addresses are validated for every texel; fade rows are validated
/// only for nonzero texels, matching the source's transparent early branch.
pub fn render_mode6_span(
    texture: &[u8],
    fade: &TerrainFadeTable<'_>,
    input: Mode6SpanInput,
    output: &mut [Option<u8>],
) -> Result<Mode6SpanReceipt, TerrainModeError> {
    if !(1..=640).contains(&input.clip_width) {
        return Err(TerrainModeError::ClipWidthOutOfRange { clip_width: input.clip_width });
    }
    let left_pixel = input.left_16_16 >> 16;
    let right_pixel = input.right_16_16 >> 16;
    if !(-4096..=4096).contains(&left_pixel) {
        return Err(TerrainModeError::EdgeOutOfRange { edge: left_pixel });
    }
    if !(-4096..=4096).contains(&right_pixel) {
        return Err(TerrainModeError::EdgeOutOfRange { edge: right_pixel });
    }
    if right_pixel < left_pixel {
        return Err(TerrainModeError::InvertedSpan {
            left_16_16: input.left_16_16, right_16_16: input.right_16_16,
        });
    }
    let start_x = left_pixel.max(0);
    let end_x = right_pixel.min(input.clip_width);
    let pixel_count = if end_x > start_x { (end_x - start_x) as usize } else { 0 };
    if pixel_count == 0 {
        return Ok(Mode6SpanReceipt { start_x, pixel_count, opaque_count: 0 });
    }
    if output.len() < pixel_count {
        return Err(TerrainModeError::OutputTooSmall { required: pixel_count, available: output.len() });
    }

    let mut u = input.u_16_16;
    let mut v = input.v_16_16;
    let mut shade = input.shade_16_16;
    if left_pixel < 0 {
        let skipped = -left_pixel;
        u = u.wrapping_add(skipped.wrapping_mul(input.du_16_16));
        v = v.wrapping_add(skipped.wrapping_mul(input.dv_16_16));
        shade = shade.wrapping_add(skipped.wrapping_mul(input.dshade_16_16));
    }
    // The mode-6 register chain keeps exactly 16 fractional and 8 integer
    // bits for each attribute; high bits wrap away after clip advancement.
    u &= 0x00ff_ffff;
    v &= 0x00ff_ffff;
    shade &= 0x00ff_ffff;

    let mut staged = Vec::with_capacity(pixel_count);
    for _ in 0..pixel_count {
        let u_byte = (u as u32 >> 16) as u8;
        let v_byte = (v as u32 >> 16) as u8;
        let shade_byte = (shade as u32 >> 16) as u8;
        let local_offset = usize::from(v_byte) * MODE5_TEXTURE_ROW_STRIDE + usize::from(u_byte);
        let offset = input.texture_base.checked_add(local_offset)
            .ok_or(TerrainModeError::Primitive(TerrainPrimitiveError::TextureAddressOverflow))?;
        let texel = *texture.get(offset).ok_or(TerrainModeError::Primitive(
            TerrainPrimitiveError::TextureSampleOutOfBounds { offset, available: texture.len() },
        ))?;
        let mapped = if texel == 0 {
            None
        } else {
            Some(fade.map_palette_index(shade_byte, texel)?)
        };
        staged.push(mapped);
        u = u.wrapping_add(input.du_16_16) & 0x00ff_ffff;
        v = v.wrapping_add(input.dv_16_16) & 0x00ff_ffff;
        shade = shade.wrapping_add(input.dshade_16_16) & 0x00ff_ffff;
    }
    let opaque_count = staged.iter().filter(|pixel| pixel.is_some()).count();
    output[..pixel_count].copy_from_slice(&staged);
    Ok(Mode6SpanReceipt { start_x, pixel_count, opaque_count })
}

/// Draw a prepared triangle through the mode-6 indexed sampler. `texture`
/// begins at the selected overlay tile's source base and retains the full
/// 256-byte source pitch. The operation stages the full frame, so a late
/// invalid texture/shade sample cannot partially change caller storage.
pub fn draw_mode6_triangle(
    texture: &[u8],
    fade: &TerrainFadeTable<'_>,
    triangle: &PreparedMode5Triangle,
    clip_width: i32,
    framebuffer: &mut [u8],
    pitch: usize,
) -> Result<Mode5TriangleDrawReceipt, TerrainPrimitiveError> {
    if !(1..=640).contains(&clip_width) {
        return Err(TerrainPrimitiveError::Mode6ClipWidthOutOfRange { clip_width });
    }
    if !(1..=480).contains(&triangle.clip_height) {
        return Err(TerrainPrimitiveError::TriangleClipHeightOutOfRange {
            clip_height: i32::try_from(triangle.clip_height).unwrap_or(i32::MAX),
        });
    }
    let width = clip_width as usize;
    if pitch < width {
        return Err(TerrainPrimitiveError::TriangleFramebufferPitchTooSmall { pitch, clip_width: width });
    }
    let required = pitch.checked_mul(triangle.clip_height)
        .ok_or(TerrainPrimitiveError::TriangleFramebufferExtentOverflow)?;
    if framebuffer.len() < required {
        return Err(TerrainPrimitiveError::TriangleFramebufferTooSmall { required, available: framebuffer.len() });
    }

    let mut staged_frame = framebuffer[..required].to_vec();
    let mut pixels_written = 0usize;
    let mut span_output = vec![None; width];
    for row in &triangle.rows {
        if row.y < 0 || row.y as usize >= triangle.clip_height {
            return Err(TerrainPrimitiveError::TriangleSpanInvariant);
        }
        let span = render_mode6_span(texture, fade, Mode6SpanInput {
            clip_width,
            left_16_16: row.left_16_16,
            right_16_16: row.right_16_16,
            u_16_16: row.u_16_16,
            v_16_16: row.v_16_16,
            shade_16_16: row.shade_16_16,
            du_16_16: triangle.gradients_du_dv_dshade[0],
            dv_16_16: triangle.gradients_du_dv_dshade[1],
            dshade_16_16: triangle.gradients_du_dv_dshade[2],
            texture_base: 0,
        }, &mut span_output).map_err(|error| match error {
            TerrainModeError::Primitive(primitive) => primitive,
            TerrainModeError::ClipWidthOutOfRange { clip_width } => TerrainPrimitiveError::Mode6ClipWidthOutOfRange { clip_width },
            TerrainModeError::InvertedSpan { left_16_16, right_16_16 } => TerrainPrimitiveError::Mode6InvertedSpan { left_16_16, right_16_16 },
            TerrainModeError::EdgeOutOfRange { edge } => TerrainPrimitiveError::Mode6EdgeOutOfRange { edge },
            TerrainModeError::OutputTooSmall { required, available } => TerrainPrimitiveError::Mode6OutputTooSmall { required, available },
            TerrainModeError::MainGlaLength { expected, actual } => TerrainPrimitiveError::MainGlaTableLength { expected, actual },
        })?;
        if span.pixel_count == 0 { continue; }
        let row_start = row.y as usize * pitch + span.start_x as usize;
        for (x, pixel) in span_output[..span.pixel_count].iter().enumerate() {
            if let Some(index) = pixel {
                staged_frame[row_start + x] = *index;
                pixels_written += 1;
            }
        }
    }
    framebuffer[..required].copy_from_slice(&staged_frame);
    Ok(Mode5TriangleDrawReceipt { rows: triangle.rows.len(), pixels_written })
}

/// A geometry-only input vertex. No UV, shade or map-cell interpretation is
/// attached to mode 0/15 row preparation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerrainGeometryVertex {
    pub x: i32,
    pub y: i32,
}

/// Prepare geometry-only rows for modes 0 and 15. Unlike mode 5/6 this path
/// does not calculate rounded attribute-gradient width, so a general triangle
/// with that rounded width equal to zero remains admissible.
pub fn prepare_mode0_15_triangle(
    vertices: [TerrainGeometryVertex; 3],
    clip_height: i32,
) -> Result<Option<PreparedMode0_15Triangle>, TerrainPrimitiveError> {
    if !(1..=480).contains(&clip_height) {
        return Err(TerrainPrimitiveError::TriangleClipHeightOutOfRange { clip_height });
    }
    for vertex in vertices {
        if !(-4096..=4096).contains(&vertex.x) || !(-4096..=4096).contains(&vertex.y) {
            return Err(TerrainPrimitiveError::TriangleCoordinateOutOfRange { x: vertex.x, y: vertex.y });
        }
    }
    let as_triangle_vertex = |vertex: TerrainGeometryVertex| Mode5TriangleVertex {
        x: vertex.x, y: vertex.y, u: 0, v: 0, shade: 0,
    };
    let input = vertices.map(as_triangle_vertex);
    let Some((kind, top, middle, bottom)) = classify_triangle(input) else {
        return Ok(None);
    };
    if bottom.y <= 0 || top.y >= clip_height { return Ok(None); }
    let height = bottom.y - top.y;
    let (left0, right0, left_slope, right_slope, middle_height, lower_left, lower_right) = match kind {
        TriangleKind::FlatTop => {
            let left_slope = geometry_edge_slope(top.x, bottom.x, height);
            let right_slope = geometry_edge_slope(middle.x, bottom.x, height);
            (top.x.wrapping_shl(16), middle.x.wrapping_shl(16), left_slope,
                right_slope, height, left_slope, right_slope)
        }
        TriangleKind::FlatBottom => {
            let left_slope = geometry_edge_slope(top.x, middle.x, height);
            let right_slope = geometry_edge_slope(top.x, bottom.x, height);
            (top.x.wrapping_shl(16), top.x.wrapping_shl(16), left_slope,
                right_slope, height, left_slope, right_slope)
        }
        TriangleKind::RightMiddle | TriangleKind::LeftMiddle => {
            let middle_height = middle.y - top.y;
            let long_slope = geometry_edge_slope(top.x, bottom.x, height);
            let upper_slope = geometry_edge_slope(top.x, middle.x, middle_height);
            let lower_slope = geometry_edge_slope(middle.x, bottom.x, height - middle_height);
            if kind == TriangleKind::RightMiddle {
                if upper_slope <= long_slope { return Ok(None); }
                (top.x.wrapping_shl(16), top.x.wrapping_shl(16), long_slope,
                    upper_slope, middle_height, long_slope, lower_slope)
            } else {
                if long_slope <= upper_slope { return Ok(None); }
                (top.x.wrapping_shl(16), top.x.wrapping_shl(16), upper_slope,
                    long_slope, middle_height, lower_slope, long_slope)
            }
        }
    };

    let mut rows = Vec::with_capacity((bottom.y.min(clip_height) - top.y.max(0)).max(0) as usize);
    for y in top.y.max(0)..bottom.y.min(clip_height) {
        let k = y - top.y;
        let upper_rows = k.min(middle_height);
        let lower_rows = (k - middle_height).max(0);
        let mut left = left0
            .wrapping_add(left_slope.wrapping_mul(upper_rows))
            .wrapping_add(lower_left.wrapping_mul(lower_rows));
        let mut right = right0
            .wrapping_add(right_slope.wrapping_mul(upper_rows))
            .wrapping_add(lower_right.wrapping_mul(lower_rows));
        if k >= middle_height && kind == TriangleKind::RightMiddle {
            right = middle.x.wrapping_shl(16).wrapping_add(lower_right.wrapping_mul(lower_rows));
        }
        if k >= middle_height && kind == TriangleKind::LeftMiddle {
            left = middle.x.wrapping_shl(16).wrapping_add(lower_left.wrapping_mul(lower_rows));
        }
        rows.push(PreparedGeometryRow { y, left_16_16: left, right_16_16: right });
    }
    Ok(Some(PreparedMode0_15Triangle { rows, clip_height: clip_height as usize }))
}

fn geometry_edge_slope(a: i32, b: i32, height: i32) -> i32 {
    b.wrapping_sub(a).wrapping_shl(16) / height
}

/// Fill mode-0 covered spans with one explicit palette index.
pub fn draw_mode0_triangle(
    triangle: &PreparedMode0_15Triangle,
    clip_width: i32,
    framebuffer: &mut [u8],
    pitch: usize,
    color: u8,
) -> Result<Mode5TriangleDrawReceipt, TerrainModeError> {
    draw_geometry_triangle(triangle, clip_width, framebuffer, pitch, |destination| {
        let _ = destination;
        color
    })
}

/// Draw mode 15 through the exact 256×256 MAIN.GLA table. Every covered pixel
/// is replaced by `table[existing_destination * 256 + color]`.
pub fn draw_mode15_triangle(
    triangle: &PreparedMode0_15Triangle,
    table: &MainGlaTable<'_>,
    clip_width: i32,
    framebuffer: &mut [u8],
    pitch: usize,
    color: u8,
) -> Result<Mode5TriangleDrawReceipt, TerrainModeError> {
    draw_geometry_triangle(triangle, clip_width, framebuffer, pitch, |destination| {
        table.map_destination(destination, color)
    })
}

fn draw_geometry_triangle(
    triangle: &PreparedMode0_15Triangle,
    clip_width: i32,
    framebuffer: &mut [u8],
    pitch: usize,
    mut map_destination: impl FnMut(u8) -> u8,
) -> Result<Mode5TriangleDrawReceipt, TerrainModeError> {
    if !(1..=640).contains(&clip_width) {
        return Err(TerrainModeError::ClipWidthOutOfRange { clip_width });
    }
    if !(1..=480).contains(&triangle.clip_height) {
        return Err(TerrainModeError::Primitive(TerrainPrimitiveError::TriangleClipHeightOutOfRange {
            clip_height: i32::try_from(triangle.clip_height).unwrap_or(i32::MAX),
        }));
    }
    let width = clip_width as usize;
    if pitch < width {
        return Err(TerrainModeError::Primitive(TerrainPrimitiveError::TriangleFramebufferPitchTooSmall { pitch, clip_width: width }));
    }
    let required = pitch.checked_mul(triangle.clip_height)
        .ok_or(TerrainModeError::Primitive(TerrainPrimitiveError::TriangleFramebufferExtentOverflow))?;
    if framebuffer.len() < required {
        return Err(TerrainModeError::Primitive(TerrainPrimitiveError::TriangleFramebufferTooSmall { required, available: framebuffer.len() }));
    }
    let mut staged = framebuffer[..required].to_vec();
    let mut pixels_written = 0usize;
    for row in &triangle.rows {
        if row.y < 0 || row.y as usize >= triangle.clip_height {
            return Err(TerrainModeError::Primitive(TerrainPrimitiveError::TriangleSpanInvariant));
        }
        let left = row.left_16_16 >> 16;
        let right = row.right_16_16 >> 16;
        if !(-4096..=4096).contains(&left) {
            return Err(TerrainModeError::EdgeOutOfRange { edge: left });
        }
        if !(-4096..=4096).contains(&right) {
            return Err(TerrainModeError::EdgeOutOfRange { edge: right });
        }
        if right < left {
            return Err(TerrainModeError::InvertedSpan { left_16_16: row.left_16_16, right_16_16: row.right_16_16 });
        }
        let start_x = left.max(0);
        let end_x = right.min(clip_width);
        if end_x <= start_x { continue; }
        let row_start = row.y as usize * pitch + start_x as usize;
        let row_end = row.y as usize * pitch + end_x as usize;
        for destination in &mut staged[row_start..row_end] {
            *destination = map_destination(*destination);
            pixels_written += 1;
        }
    }
    framebuffer[..required].copy_from_slice(&staged);
    Ok(Mode5TriangleDrawReceipt { rows: triangle.rows.len(), pixels_written })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::{prepare_mode5_triangle, Mode5TriangleVertex, TERRAIN_FADE_TABLE_BYTES};

    fn resources() -> (Vec<u8>, Vec<u8>) {
        let texture = vec![0; 65_536];
        let mut fade = vec![0; TERRAIN_FADE_TABLE_BYTES];
        for shade in 0..64usize {
            for texel in 0..256usize {
                fade[shade * 256 + texel] = (shade ^ texel) as u8;
            }
        }
        (texture, fade)
    }

    fn patterned_resources() -> (Vec<u8>, Vec<u8>) {
        let mut texture = vec![0; 65_536];
        for (offset, value) in texture.iter_mut().enumerate() {
            *value = ((3 * (offset & 255) + 5 * (offset >> 8)) & 255) as u8;
        }
        let mut fade = vec![0; TERRAIN_FADE_TABLE_BYTES];
        for shade in 0..64usize {
            for texel in 0..256usize {
                fade[shade * 256 + texel] = ((texel + 7 * shade) & 255) as u8;
            }
        }
        (texture, fade)
    }

    fn span(left: i32, right: i32, shade: i32) -> Mode6SpanInput {
        Mode6SpanInput {
            clip_width: 4,
            left_16_16: left,
            right_16_16: right,
            u_16_16: 0,
            v_16_16: 0,
            shade_16_16: shade,
            du_16_16: 0,
            dv_16_16: 0,
            dshade_16_16: 0,
            texture_base: 0,
        }
    }

    #[test]
    fn mode6_zero_is_transparent_and_does_not_validate_fade_shade() {
        let (texture, fade_bytes) = resources();
        let fade = TerrainFadeTable::parse(&fade_bytes).unwrap();
        let mut output = [Some(0xA5); 4];
        let receipt = render_mode6_span(&texture, &fade, span(0, 4 << 16, 64 << 16), &mut output).unwrap();
        assert_eq!(receipt, Mode6SpanReceipt { start_x: 0, pixel_count: 4, opaque_count: 0 });
        assert_eq!(output, [None; 4]);

        // Address validation happens even for a zero texel.
        assert!(matches!(render_mode6_span(&texture[..1], &fade, Mode6SpanInput {
            du_16_16: 1 << 16, ..span(0, 2 << 16, 0)
        }, &mut output),
            Err(TerrainModeError::Primitive(TerrainPrimitiveError::TextureSampleOutOfBounds { .. }))));
    }

    #[test]
    fn mode6_nonzero_texels_use_24bit_fields_and_fade_rows() {
        let (mut texture, fade_bytes) = resources();
        let fade = TerrainFadeTable::parse(&fade_bytes).unwrap();
        texture[0] = 7;
        texture[1] = 0;
        texture[2] = 9;
        let input = Mode6SpanInput {
            clip_width: 4, left_16_16: 0, right_16_16: 3 << 16,
            u_16_16: 0, v_16_16: 0, shade_16_16: 3 << 16,
            du_16_16: 1 << 16, dv_16_16: 0, dshade_16_16: 1 << 16,
            texture_base: 0,
        };
        let mut output = [None; 4];
        let receipt = render_mode6_span(&texture, &fade, input, &mut output).unwrap();
        assert_eq!(receipt, Mode6SpanReceipt { start_x: 0, pixel_count: 3, opaque_count: 2 });
        assert_eq!(output[..3], [Some(4), None, Some(12)]);
        assert_eq!(output[3], None);
        assert!(matches!(render_mode6_span(&texture, &fade, Mode6SpanInput { shade_16_16: 64 << 16, ..input }, &mut output),
            Err(TerrainModeError::Primitive(TerrainPrimitiveError::FadeShadeIndexOutOfRange { shade_index: 64 }))));
    }

    #[test]
    fn mode6_attributes_wrap_at_24_bits_after_each_sample() {
        let (mut texture, fade_bytes) = resources();
        let fade = TerrainFadeTable::parse(&fade_bytes).unwrap();
        texture[255] = 0;
        texture[0] = 10;
        let input = Mode6SpanInput {
            clip_width: 4, left_16_16: 0, right_16_16: 2 << 16,
            u_16_16: 0x00ff_ffff, v_16_16: 0, shade_16_16: 0x00ff_ffff,
            du_16_16: 1, dv_16_16: 0, dshade_16_16: 1,
            texture_base: 0,
        };
        let mut output = [None; 2];
        let receipt = render_mode6_span(&texture, &fade, input, &mut output).unwrap();
        assert_eq!(receipt.opaque_count, 1);
        assert_eq!(output, [None, Some(10)]);
    }

    #[test]
    fn mode6_matches_all_four_static_span_vectors() {
        let (texture, fade_bytes) = patterned_resources();
        let fade = TerrainFadeTable::parse(&fade_bytes).unwrap();
        let vectors = [
            (Mode6SpanInput {
                clip_width: 8, left_16_16: 0, right_16_16: 8 << 16,
                u_16_16: 0, v_16_16: 0, shade_16_16: 32 << 16,
                du_16_16: 1 << 16, dv_16_16: 0, dshade_16_16: 0, texture_base: 0,
            }, vec![None, Some(227), Some(230), Some(233), Some(236), Some(239), Some(242), Some(245)]),
            (Mode6SpanInput {
                clip_width: 8, left_16_16: -3 << 16, right_16_16: 5 << 16,
                u_16_16: 0x18000, v_16_16: 0x24000, shade_16_16: 0x208000,
                du_16_16: 0x8000, dv_16_16: 0x4000, dshade_16_16: -0x1000, texture_base: 0,
            }, vec![Some(248), Some(248), Some(251), Some(251), Some(3)]),
            (Mode6SpanInput {
                clip_width: 8, left_16_16: 0, right_16_16: 8 << 16,
                u_16_16: 0x68000, v_16_16: 0x74000, shade_16_16: 0x208000,
                du_16_16: -0x8000, dv_16_16: -0x4000, dshade_16_16: -0x1000, texture_base: 0,
            }, vec![Some(21), Some(21), Some(13), Some(13), Some(10), Some(10), Some(2), Some(2)]),
            (Mode6SpanInput {
                clip_width: 8, left_16_16: 0, right_16_16: 8 << 16,
                u_16_16: 0x18000, v_16_16: 0x24000, shade_16_16: 0x20fffe,
                du_16_16: 0, dv_16_16: 0, dshade_16_16: 1, texture_base: 0,
            }, vec![Some(237), Some(237), Some(244), Some(244), Some(244), Some(244), Some(244), Some(244)]),
        ];
        for (input, expected) in vectors {
            let mut output = vec![None; expected.len()];
            let receipt = render_mode6_span(&texture, &fade, input, &mut output).unwrap();
            assert_eq!(receipt.start_x, 0);
            assert_eq!(receipt.pixel_count, expected.len());
            assert_eq!(output, expected);
        }
    }

    #[test]
    fn mode6_span_checks_clipping_and_edges() {
        let (texture, fade_bytes) = resources();
        let fade = TerrainFadeTable::parse(&fade_bytes).unwrap();
        let mut output = [None; 4];
        let clipped = Mode6SpanInput { left_16_16: -2 << 16, right_16_16: 2 << 16, ..span(0, 1 << 16, 0) };
        assert_eq!(render_mode6_span(&texture, &fade, clipped, &mut output).unwrap().start_x, 0);
        assert_eq!(render_mode6_span(&texture, &fade, Mode6SpanInput { clip_width: 641, ..clipped }, &mut output),
            Err(TerrainModeError::ClipWidthOutOfRange { clip_width: 641 }));
        assert!(matches!(render_mode6_span(&texture, &fade, Mode6SpanInput { left_16_16: 2 << 16, right_16_16: 1 << 16, ..clipped }, &mut output),
            Err(TerrainModeError::InvertedSpan { .. })));
        assert!(matches!(render_mode6_span(&texture, &fade, Mode6SpanInput { left_16_16: 4097 << 16, ..clipped }, &mut output),
            Err(TerrainModeError::EdgeOutOfRange { edge: 4097 })));
    }

    #[test]
    fn mode6_triangle_composites_transparency_and_is_atomic() {
        let vertices = [
            Mode5TriangleVertex { x: 0, y: 0, u: 0, v: 0, shade: 0 },
            Mode5TriangleVertex { x: 3, y: 0, u: 3 << 16, v: 0, shade: 0 },
            Mode5TriangleVertex { x: 0, y: 3, u: 0, v: 3 << 16, shade: 0 },
        ];
        let triangle = prepare_mode5_triangle(vertices, 4).unwrap().unwrap();
        let (mut texture, fade_bytes) = resources();
        texture.fill(0);
        texture[0] = 5;
        let fade = TerrainFadeTable::parse(&fade_bytes).unwrap();
        let mut frame = vec![0xA5; 4 * 4];
        let receipt = draw_mode6_triangle(&texture, &fade, &triangle, 4, &mut frame, 4).unwrap();
        assert_eq!(receipt.pixels_written, 1);
        assert_eq!(frame[0], 5);
        assert!(frame[1..].iter().all(|byte| *byte == 0xA5));

        let before = frame.clone();
        let mut invalid = triangle.clone();
        invalid.rows.last_mut().unwrap().shade_16_16 = 64 << 16;
        let invalid_texture = vec![1; 65_536];
        assert!(matches!(draw_mode6_triangle(&invalid_texture, &fade, &invalid, 4, &mut frame, 4),
            Err(TerrainPrimitiveError::FadeShadeIndexOutOfRange { shade_index: 64 })));
        assert_eq!(frame, before);
    }

    #[test]
    fn mode0_15_geometry_admits_zero_gradient_width_and_matches_receipt() {
        let triangle = prepare_mode0_15_triangle([
            TerrainGeometryVertex { x: 0, y: 0 },
            TerrainGeometryVertex { x: 1, y: 1 },
            TerrainGeometryVertex { x: 1, y: 3 },
        ], 4).unwrap().unwrap();
        assert_eq!(triangle.rows, [
            PreparedGeometryRow { y: 0, left_16_16: 0, right_16_16: 0 },
            PreparedGeometryRow { y: 1, left_16_16: 21_845, right_16_16: 65_536 },
            PreparedGeometryRow { y: 2, left_16_16: 43_690, right_16_16: 65_536 },
        ]);
    }

    #[test]
    fn mode0_fills_and_mode15_uses_destination_then_color_lookup_atomically() {
        let triangle = prepare_mode0_15_triangle([
            TerrainGeometryVertex { x: 1, y: 0 },
            TerrainGeometryVertex { x: 5, y: 0 },
            TerrainGeometryVertex { x: 3, y: 3 },
        ], 4).unwrap().unwrap();
        let mut mode0 = vec![0xA5; 4 * 6];
        let receipt = draw_mode0_triangle(&triangle, 6, &mut mode0, 6, 0).unwrap();
        assert!(receipt.pixels_written > 0);
        assert!(mode0.iter().any(|byte| *byte == 0));
        assert_eq!(mode0[5], 0xA5);

        let mut bytes = vec![0; MAIN_GLA_TABLE_BYTES];
        for destination in 0..256usize {
            for color in 0..256usize {
                bytes[destination * 256 + color] = (destination * 3 + color) as u8;
            }
        }
        let table = MainGlaTable::parse(&bytes).unwrap();
        assert_eq!(table.map_destination(17, 3), 54);
        assert!(matches!(MainGlaTable::parse(&bytes[..bytes.len() - 1]), Err(TerrainModeError::MainGlaLength { .. })));
        let mut frame = vec![3; 4 * 6];
        let before = frame.clone();
        let receipt = draw_mode15_triangle(&triangle, &table, 6, &mut frame, 6, 0).unwrap();
        assert!(receipt.pixels_written > 0);
        assert!(frame.iter().any(|byte| *byte == 9));
        assert!(frame.iter().any(|byte| *byte == 3));
        assert_ne!(frame, before);

        let before_error = frame.clone();
        let mut malformed = triangle.clone();
        malformed.rows.last_mut().unwrap().y = 4;
        assert!(matches!(draw_mode15_triangle(&malformed, &table, 6, &mut frame, 6, 0),
            Err(TerrainModeError::Primitive(TerrainPrimitiveError::TriangleSpanInvariant))));
        assert_eq!(frame, before_error);
    }
}
