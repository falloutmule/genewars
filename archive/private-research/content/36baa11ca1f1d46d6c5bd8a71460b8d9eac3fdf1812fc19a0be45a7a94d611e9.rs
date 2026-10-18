//! Explicit-input terrain vertex, triangle-row, and B32 UV primitives from
//! the static receipts.
//!
//! These helpers do not choose a camera, choose lighting operands, or assemble
//! a complete scene. The shade arithmetic accepts raw
//! operands; no factor is inferred. The vertex primitive accepts explicit
//! unwrapped map and camera coordinates plus caller placement. UV pairs are
//! the four entries of the verified table in `[00, 10, 11, 01]` order; this
//! module does not interpolate between them.

use core::fmt;

pub const TERRAIN_HORIZONTAL_STEP: i32 = 32;
pub const TERRAIN_DIAGONAL_VERTICAL_STEP: i32 = 16;
pub const B32_UV_MAX_16_16: u32 = 0x001f_ffff;
pub const B32_UV_ORIENTATION_COUNT: usize = 8;
pub const CELL_TRIANGLES: [[usize; 3]; 2] = [[0, 2, 3], [0, 1, 2]];
pub const TERRAIN_FADE_SHADE_COUNT: usize = 64;
pub const TERRAIN_FADE_ROW_BYTES: usize = 256;
pub const TERRAIN_FADE_TABLE_BYTES: usize = TERRAIN_FADE_SHADE_COUNT * TERRAIN_FADE_ROW_BYTES;
pub const MODE5_TEXTURE_ROW_STRIDE: usize = 256;

const B32_UV_CORNERS: [[(u32, u32); 4]; B32_UV_ORIENTATION_COUNT] = [
    [(0, 0), (B32_UV_MAX_16_16, 0), (B32_UV_MAX_16_16, B32_UV_MAX_16_16), (0, B32_UV_MAX_16_16)],
    [(B32_UV_MAX_16_16, 0), (0, 0), (0, B32_UV_MAX_16_16), (B32_UV_MAX_16_16, B32_UV_MAX_16_16)],
    [(0, B32_UV_MAX_16_16), (B32_UV_MAX_16_16, B32_UV_MAX_16_16), (B32_UV_MAX_16_16, 0), (0, 0)],
    [(B32_UV_MAX_16_16, B32_UV_MAX_16_16), (0, B32_UV_MAX_16_16), (0, 0), (B32_UV_MAX_16_16, 0)],
    [(0, 0), (0, B32_UV_MAX_16_16), (B32_UV_MAX_16_16, B32_UV_MAX_16_16), (B32_UV_MAX_16_16, 0)],
    [(0, B32_UV_MAX_16_16), (0, 0), (B32_UV_MAX_16_16, 0), (B32_UV_MAX_16_16, B32_UV_MAX_16_16)],
    [(B32_UV_MAX_16_16, 0), (B32_UV_MAX_16_16, B32_UV_MAX_16_16), (0, B32_UV_MAX_16_16), (0, 0)],
    [(B32_UV_MAX_16_16, B32_UV_MAX_16_16), (B32_UV_MAX_16_16, 0), (0, 0), (0, B32_UV_MAX_16_16)],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainPrimitiveError {
    UvOrientationOutOfRange { orientation: u8 },
    CoordinateOverflow,
    FadeTableLength { expected: usize, actual: usize },
    FadeShadeIndexOutOfRange { shade_index: u8 },
    TextureSampleOutOfBounds { offset: usize, available: usize },
    TextureAddressOverflow,
    NegativeClipWidth { clip_width: i32 },
    SpanOutputTooSmall { required: usize, available: usize },
    TriangleClipHeightOutOfRange { clip_height: i32 },
    TriangleClipWidthOutOfRange { clip_width: i32 },
    TriangleCoordinateOutOfRange { x: i32, y: i32 },
    TriangleFramebufferPitchTooSmall { pitch: usize, clip_width: usize },
    TriangleFramebufferTooSmall { required: usize, available: usize },
    TriangleFramebufferExtentOverflow,
    TriangleSpanInvariant,
    TriangleZeroGeneralWidth,
    Mode6ClipWidthOutOfRange { clip_width: i32 },
    Mode6InvertedSpan { left_16_16: i32, right_16_16: i32 },
    Mode6EdgeOutOfRange { edge: i32 },
    Mode6OutputTooSmall { required: usize, available: usize },
    MainGlaTableLength { expected: usize, actual: usize },
}

impl fmt::Display for TerrainPrimitiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for TerrainPrimitiveError {}

/// Return four 16.16 UV pairs for one explicitly selected table row.
pub fn b32_uv_corners(orientation: u8) -> Result<[(u32, u32); 4], TerrainPrimitiveError> {
    B32_UV_CORNERS.get(usize::from(orientation)).copied()
        .ok_or(TerrainPrimitiveError::UvOrientationOutOfRange { orientation })
}

/// Select the UV row using bits 4–6 of a map-cell flag byte.
pub fn b32_uv_corners_for_cell_flags(flags: u8) -> [(u32, u32); 4] {
    B32_UV_CORNERS[usize::from((flags & 0x70) >> 4)]
}

/// Convert a signed map-cell height to the traced vertex height displacement.
/// This is `2 * (height >> 6)`; shifting first is observably different from
/// `(2 * height) >> 6` for nonmultiples of 64 and negative values.
pub fn terrain_vertex_height_displacement(height: i16) -> i32 {
    2 * (i32::from(height) >> 6)
}

/// Compute the traced signed 16.16 shade from explicit raw operands.
/// The source factor's producer and gameplay meaning remain unresolved.
pub fn terrain_vertex_shade_16_16(light_operand: i16, factor_operand: u8) -> i32 {
    4 * i32::from(light_operand) * i32::from(factor_operand) + 0x8000
}

/// Borrowed view of the 64×256 FADE.DAT lookup table.
#[derive(Debug, Clone, Copy)]
pub struct TerrainFadeTable<'a> {
    bytes: &'a [u8],
}

impl<'a> TerrainFadeTable<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, TerrainPrimitiveError> {
        if bytes.len() != TERRAIN_FADE_TABLE_BYTES {
            return Err(TerrainPrimitiveError::FadeTableLength {
                expected: TERRAIN_FADE_TABLE_BYTES,
                actual: bytes.len(),
            });
        }
        Ok(Self { bytes })
    }

    /// Look up one caller-selected shade row and palette index. This does not
    /// infer a row from the 16.16 shade calculation or choose lighting inputs.
    pub fn map_palette_index(
        &self,
        shade_index: u8,
        source_palette_index: u8,
    ) -> Result<u8, TerrainPrimitiveError> {
        if usize::from(shade_index) >= TERRAIN_FADE_SHADE_COUNT {
            return Err(TerrainPrimitiveError::FadeShadeIndexOutOfRange { shade_index });
        }
        let offset = usize::from(shade_index) * TERRAIN_FADE_ROW_BYTES + usize::from(source_palette_index);
        Ok(self.bytes[offset])
    }
}

/// Perform one explicit mode-5 texture/fade sample. `u` and `v` are the
/// integer-byte coordinates supplied to the sampler; shade row selection is
/// also explicit. Every source index, including zero, is passed through FADE.
pub fn mode5_sample_index(
    texture: &[u8],
    fade: &TerrainFadeTable<'_>,
    u_integer_byte: u8,
    v_integer_byte: u8,
    shade_index: u8,
) -> Result<u8, TerrainPrimitiveError> {
    let offset = usize::from(v_integer_byte) * MODE5_TEXTURE_ROW_STRIDE
        + usize::from(u_integer_byte);
    let source_index = *texture.get(offset).ok_or(TerrainPrimitiveError::TextureSampleOutOfBounds {
        offset,
        available: texture.len(),
    })?;
    fade.map_palette_index(shade_index, source_index)
}

/// Fully prepared horizontal span inputs in signed 16.16 form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mode5SpanInput {
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
pub struct Mode5SpanReceipt {
    pub start_x: i32,
    pub pixel_count: usize,
}

/// One explicitly supplied triangle vertex. `x` and `y` are integer
/// coordinates; U, V and shade are signed 16.16 bitpatterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mode5TriangleVertex {
    pub x: i32,
    pub y: i32,
    pub u: i32,
    pub v: i32,
    pub shade: i32,
}

/// A prepared row, with attributes anchored at its left edge as in the
/// original row producer. The separate span sampler floors both x edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreparedMode5Row {
    pub y: i32,
    pub left_16_16: i32,
    pub right_16_16: i32,
    pub u_16_16: i32,
    pub v_16_16: i32,
    pub shade_16_16: i32,
}

/// Explicit triangle rows and the three horizontal attribute gradients.
/// `clip_height` records the bounds used to prepare the rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedMode5Triangle {
    pub rows: Vec<PreparedMode5Row>,
    pub gradients_du_dv_dshade: [i32; 3],
    pub clip_height: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mode5TriangleDrawReceipt {
    pub rows: usize,
    pub pixels_written: usize,
}

#[derive(Clone, Copy)]
struct PackedSpanState {
    c: u32,
    d: u32,
    v_byte: u8,
    gradient_c: u32,
    gradient_d: u32,
    gradient_v_byte: u8,
}

/// Render one already-prepared mode-5 horizontal span into caller storage.
///
/// This ports the coupled C/D/v-byte carry machine literally. It does not
/// derive triangle edges or gradients. Bounds are validated for the complete
/// span before output bytes are changed. A zero clip width returns an empty
/// span as a safe-port deviation from the source's edge case; negative clip
/// widths are rejected.
pub fn render_mode5_span(
    texture: &[u8],
    fade: &TerrainFadeTable<'_>,
    input: Mode5SpanInput,
    output: &mut [u8],
) -> Result<Mode5SpanReceipt, TerrainPrimitiveError> {
    if input.clip_width < 0 {
        return Err(TerrainPrimitiveError::NegativeClipWidth { clip_width: input.clip_width });
    }
    let left_pixel = input.left_16_16 >> 16;
    let right_pixel = input.right_16_16 >> 16;
    let start_x = left_pixel.max(0);
    let end_x = right_pixel.min(input.clip_width);
    let pixel_count = if end_x > start_x { (end_x - start_x) as usize } else { 0 };
    if pixel_count == 0 { return Ok(Mode5SpanReceipt { start_x, pixel_count }); }
    if output.len() < pixel_count {
        return Err(TerrainPrimitiveError::SpanOutputTooSmall { required: pixel_count, available: output.len() });
    }

    let state = packed_span_state(input, left_pixel);
    let mut c = state.c;
    let mut d = state.d;
    let mut v_byte = state.v_byte;

    // Validate every source and fade access before mutating the output slice.
    for _ in 0..pixel_count {
        let u = (d & 0xff) as u8;
        let shade = ((c >> 8) & 0xff) as u8;
        validate_span_sample(texture, fade, input.texture_base, u, v_byte, shade)?;
        (c, d, v_byte) = advance_span_state(c, d, v_byte, state);
    }

    c = state.c;
    d = state.d;
    v_byte = state.v_byte;
    for destination in output.iter_mut().take(pixel_count) {
        let u = (d & 0xff) as u8;
        let shade = ((c >> 8) & 0xff) as u8;
        let texture_offset = input.texture_base + usize::from(v_byte) * MODE5_TEXTURE_ROW_STRIDE + usize::from(u);
        let source_index = texture[texture_offset];
        let fade_offset = usize::from(shade) * TERRAIN_FADE_ROW_BYTES + usize::from(source_index);
        *destination = fade.bytes[fade_offset];
        (c, d, v_byte) = advance_span_state(c, d, v_byte, state);
    }

    Ok(Mode5SpanReceipt { start_x, pixel_count })
}

fn packed_span_state(input: Mode5SpanInput, left_pixel: i32) -> PackedSpanState {
    let mut u = input.u_16_16;
    let mut v = input.v_16_16;
    let mut shade = input.shade_16_16;
    if left_pixel < 0 {
        let skipped = -left_pixel;
        u = u.wrapping_add(skipped.wrapping_mul(input.du_16_16));
        v = v.wrapping_add(skipped.wrapping_mul(input.dv_16_16));
        shade = shade.wrapping_add(skipped.wrapping_mul(input.dshade_16_16));
    }
    PackedSpanState {
        c: pack_c(u, shade),
        d: pack_d(v, u),
        v_byte: ((v as u32 >> 16) & 0xff) as u8,
        gradient_c: pack_c(input.du_16_16, input.dshade_16_16),
        gradient_d: pack_gradient_d(input.dv_16_16, input.du_16_16),
        gradient_v_byte: ((input.dv_16_16 as u32 >> 16) & 0xff) as u8,
    }
}

#[inline]
fn pack_c(u: i32, shade: i32) -> u32 {
    ((u as u32).rotate_left(16) & 0xffff_0000) | ((shade as u32 >> 8) & 0xffff)
}

#[inline]
fn pack_d(v: i32, u: i32) -> u32 {
    ((v as u32).rotate_left(16) & 0xffff_0000) | ((u as u32 >> 16) & 0xff)
}

#[inline]
fn pack_gradient_d(dv: i32, du: i32) -> u32 {
    ((dv as u32).rotate_left(16) & 0xffff_ff00) | ((du as u32 >> 16) & 0xff)
}

#[inline]
fn advance_span_state(c: u32, d: u32, v_byte: u8, state: PackedSpanState) -> (u32, u32, u8) {
    let (next_c, carry_c) = c.overflowing_add(state.gradient_c);
    let (d_sum, carry_d0) = d.overflowing_add(state.gradient_d);
    let (next_d, carry_d1) = d_sum.overflowing_add(u32::from(carry_c));
    let carry_d = carry_d0 || carry_d1;
    let (v_sum, _) = v_byte.overflowing_add(state.gradient_v_byte);
    let (next_v, _) = v_sum.overflowing_add(u8::from(carry_d));
    (next_c, next_d, next_v)
}

fn validate_span_sample(
    texture: &[u8],
    fade: &TerrainFadeTable<'_>,
    texture_base: usize,
    u: u8,
    v: u8,
    shade: u8,
) -> Result<(), TerrainPrimitiveError> {
    if usize::from(shade) >= TERRAIN_FADE_SHADE_COUNT {
        return Err(TerrainPrimitiveError::FadeShadeIndexOutOfRange { shade_index: shade });
    }
    let local_offset = usize::from(v) * MODE5_TEXTURE_ROW_STRIDE + usize::from(u);
    let offset = texture_base.checked_add(local_offset).ok_or(TerrainPrimitiveError::TextureAddressOverflow)?;
    if offset >= texture.len() {
        return Err(TerrainPrimitiveError::TextureSampleOutOfBounds { offset, available: texture.len() });
    }
    let source_index = texture[offset];
    let fade_offset = usize::from(shade) * TERRAIN_FADE_ROW_BYTES + usize::from(source_index);
    debug_assert!(fade_offset < fade.bytes.len());
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TriangleKind {
    FlatTop,
    FlatBottom,
    RightMiddle,
    LeftMiddle,
}

fn wrap_sub(a: i32, b: i32) -> i32 { a.wrapping_sub(b) }
fn wrap_add(a: i32, b: i32) -> i32 { a.wrapping_add(b) }
fn edge_slope(a: Mode5TriangleVertex, b: Mode5TriangleVertex, height: i32) -> i32 {
    wrap_sub(b.x, a.x).wrapping_shl(16) / height
}

fn triangle_attributes(vertex: Mode5TriangleVertex) -> [i32; 3] {
    [vertex.u, vertex.v, vertex.shade]
}

fn div_attribute_delta(a: i32, b: i32, denominator: i32) -> i32 {
    wrap_sub(b, a) / denominator
}

fn low_product_shift(a: i32, b: i32) -> i32 {
    a.wrapping_mul(b) >> 16
}

fn signed_product_shift(a: i32, b: i32) -> i32 {
    ((i64::from(a) * i64::from(b)) >> 16) as i32
}

/// Prepare mode-five scan rows from three explicit vertices.
///
/// The source preserves cyclic order and winding. Reversed, degenerate, or
/// fully clipped triangles return `Ok(None)`. The safe port admits coordinates
/// in -4096..=4096 and clip heights 1..=480; these bounds are implementation
/// limits, not original-runtime checks. General triangles with zero rounded
/// gradient width are rejected because the source leaves those gradients
/// uninitialized.
pub fn prepare_mode5_triangle(
    vertices: [Mode5TriangleVertex; 3],
    clip_height: i32,
) -> Result<Option<PreparedMode5Triangle>, TerrainPrimitiveError> {
    if !(1..=480).contains(&clip_height) {
        return Err(TerrainPrimitiveError::TriangleClipHeightOutOfRange { clip_height });
    }
    for vertex in vertices {
        if !(-4096..=4096).contains(&vertex.x) || !(-4096..=4096).contains(&vertex.y) {
            return Err(TerrainPrimitiveError::TriangleCoordinateOutOfRange { x: vertex.x, y: vertex.y });
        }
    }

    let Some((kind, top, middle, bottom)) = classify_triangle(vertices) else {
        return Ok(None);
    };
    let top_y = top.y;
    let bottom_y = bottom.y;
    if bottom_y <= 0 || top_y >= clip_height { return Ok(None); }
    let height = bottom_y - top_y;
    let top_attrs = triangle_attributes(top);
    let middle_attrs = triangle_attributes(middle);
    let bottom_attrs = triangle_attributes(bottom);
    let (left0, right0, left_slope, right_slope, middle_height,
        lower_left_slope, lower_right_slope, upper_attr, lower_attr, gradients) = match kind {
        TriangleKind::FlatTop => {
            let left_slope = edge_slope(top, bottom, height);
            let right_slope = edge_slope(middle, bottom, height);
            let gradients: [i32; 3] = core::array::from_fn(|i|
                div_attribute_delta(top_attrs[i], middle_attrs[i], middle.x - top.x));
            let vertical: [i32; 3] = core::array::from_fn(|i|
                div_attribute_delta(top_attrs[i], bottom_attrs[i], height));
            (top.x.wrapping_shl(16), middle.x.wrapping_shl(16), left_slope,
                right_slope, height, left_slope, right_slope, vertical, vertical, gradients)
        }
        TriangleKind::FlatBottom => {
            // Canonical order is top, left-bottom, right-bottom.
            let left_slope = edge_slope(top, middle, height);
            let right_slope = edge_slope(top, bottom, height);
            let gradients: [i32; 3] = core::array::from_fn(|i|
                div_attribute_delta(middle_attrs[i], bottom_attrs[i], bottom.x - middle.x));
            let vertical: [i32; 3] = core::array::from_fn(|i|
                div_attribute_delta(top_attrs[i], middle_attrs[i], height));
            (top.x.wrapping_shl(16), top.x.wrapping_shl(16), left_slope,
                right_slope, height, left_slope, right_slope, vertical, vertical, gradients)
        }
        TriangleKind::RightMiddle | TriangleKind::LeftMiddle => {
            let middle_height = middle.y - top_y;
            let long_slope = edge_slope(top, bottom, height);
            let upper_slope = edge_slope(top, middle, middle_height);
            let lower_slope = edge_slope(middle, bottom, height - middle_height);
            let ratio = middle_height.wrapping_shl(16) / height;
            let (width, numerators, left_slope, right_slope,
                lower_left_slope, lower_right_slope, upper_attr, lower_attr) =
                if kind == TriangleKind::RightMiddle {
                    if upper_slope <= long_slope { return Ok(None); }
                    let width = wrap_add(
                        wrap_sub(middle.x, top.x),
                        low_product_shift(wrap_sub(top.x, bottom.x), ratio),
                    );
                    let numerators: [i32; 3] = core::array::from_fn(|i| {
                        let product = signed_product_shift(wrap_sub(top_attrs[i], bottom_attrs[i]), ratio);
                        wrap_sub(wrap_add(product, middle_attrs[i]), top_attrs[i])
                    });
                    let vertical: [i32; 3] = core::array::from_fn(|i|
                        div_attribute_delta(top_attrs[i], bottom_attrs[i], height));
                    (width, numerators, long_slope, upper_slope, long_slope, lower_slope,
                        vertical, vertical)
                } else {
                    if long_slope <= upper_slope { return Ok(None); }
                    let width = wrap_add(
                        wrap_sub(top.x, middle.x),
                        low_product_shift(wrap_sub(bottom.x, top.x), ratio),
                    );
                    let numerators: [i32; 3] = core::array::from_fn(|i| {
                        let product = signed_product_shift(wrap_sub(bottom_attrs[i], top_attrs[i]), ratio);
                        wrap_sub(wrap_add(product, top_attrs[i]), middle_attrs[i])
                    });
                    let upper: [i32; 3] = core::array::from_fn(|i|
                        div_attribute_delta(top_attrs[i], middle_attrs[i], middle_height));
                    let lower: [i32; 3] = core::array::from_fn(|i|
                        div_attribute_delta(middle_attrs[i], bottom_attrs[i], height - middle_height));
                    (width, numerators, upper_slope, long_slope, lower_slope, long_slope,
                        upper, lower)
                };
            if width < 0 { return Ok(None); }
            if width == 0 { return Err(TerrainPrimitiveError::TriangleZeroGeneralWidth); }
            let gradients: [i32; 3] = core::array::from_fn(|i| numerators[i] / width.wrapping_add(1));
            (top.x.wrapping_shl(16), top.x.wrapping_shl(16), left_slope,
                right_slope, middle_height, lower_left_slope, lower_right_slope,
                upper_attr, lower_attr, gradients)
        }
    };

    let mut rows = Vec::with_capacity((bottom_y.min(clip_height) - top_y.max(0)).max(0) as usize);
    for y in top_y.max(0)..bottom_y.min(clip_height) {
        let k = y - top_y;
        let upper_rows = k.min(middle_height);
        let lower_rows = (k - middle_height).max(0);
        let mut left = left0
            .wrapping_add(left_slope.wrapping_mul(upper_rows))
            .wrapping_add(lower_left_slope.wrapping_mul(lower_rows));
        let mut right = right0
            .wrapping_add(right_slope.wrapping_mul(upper_rows))
            .wrapping_add(lower_right_slope.wrapping_mul(lower_rows));
        if lower_rows > 0 && kind == TriangleKind::RightMiddle {
            right = middle.x.wrapping_shl(16).wrapping_add(lower_right_slope.wrapping_mul(lower_rows));
        }
        if lower_rows > 0 && kind == TriangleKind::LeftMiddle {
            left = middle.x.wrapping_shl(16).wrapping_add(lower_left_slope.wrapping_mul(lower_rows));
        }
        if k == middle_height && kind == TriangleKind::RightMiddle { right = middle.x.wrapping_shl(16); }
        if k == middle_height && kind == TriangleKind::LeftMiddle { left = middle.x.wrapping_shl(16); }

        let attrs = core::array::from_fn::<_, 3, _>(|i| {
            top_attrs[i]
                .wrapping_add(upper_attr[i].wrapping_mul(upper_rows))
                .wrapping_add(lower_attr[i].wrapping_mul(lower_rows))
        });
        rows.push(PreparedMode5Row {
            y,
            left_16_16: left,
            right_16_16: right,
            u_16_16: attrs[0],
            v_16_16: attrs[1],
            shade_16_16: attrs[2],
        });
    }
    Ok(Some(PreparedMode5Triangle {
        rows,
        gradients_du_dv_dshade: gradients,
        clip_height: clip_height as usize,
    }))
}

pub(crate) fn classify_triangle(
    vertices: [Mode5TriangleVertex; 3],
) -> Option<(TriangleKind, Mode5TriangleVertex, Mode5TriangleVertex, Mode5TriangleVertex)> {
    for rotation in 0..3 {
        let t = vertices[rotation];
        let p = vertices[(rotation + 1) % 3];
        let q = vertices[(rotation + 2) % 3];
        if t.y == p.y && p.y < q.y {
            return (p.x > t.x).then_some((TriangleKind::FlatTop, t, p, q));
        }
        if t.y < p.y && p.y == q.y {
            return (p.x > q.x).then_some((TriangleKind::FlatBottom, t, q, p));
        }
        if t.y < p.y && p.y < q.y {
            return Some((TriangleKind::RightMiddle, t, p, q));
        }
        if t.y < q.y && q.y < p.y {
            return Some((TriangleKind::LeftMiddle, t, q, p));
        }
    }
    None
}

/// Draw prepared rows through the verified mode-five span sampler into an
/// indexed framebuffer. The entire requested frame extent is staged and all
/// spans are validated before the caller's bytes are changed.
pub fn draw_mode5_triangle(
    texture: &[u8],
    fade: &TerrainFadeTable<'_>,
    triangle: &PreparedMode5Triangle,
    clip_width: i32,
    framebuffer: &mut [u8],
    pitch: usize,
) -> Result<Mode5TriangleDrawReceipt, TerrainPrimitiveError> {
    if !(1..=640).contains(&clip_width) {
        return Err(TerrainPrimitiveError::TriangleClipWidthOutOfRange { clip_width });
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
    let mut staged = framebuffer[..required].to_vec();
    let mut pixels_written = 0usize;
    let mut span_output = vec![0u8; width];
    for row in &triangle.rows {
        if row.y < 0 || row.y as usize >= triangle.clip_height {
            return Err(TerrainPrimitiveError::TriangleSpanInvariant);
        }
        let span = render_mode5_span(texture, fade, Mode5SpanInput {
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
        }, &mut span_output)?;
        if span.pixel_count == 0 { continue; }
        let row_start = row.y as usize * pitch + span.start_x as usize;
        staged[row_start..row_start + span.pixel_count]
            .copy_from_slice(&span_output[..span.pixel_count]);
        pixels_written += span.pixel_count;
    }
    framebuffer[..required].copy_from_slice(&staged);
    Ok(Mode5TriangleDrawReceipt { rows: triangle.rows.len(), pixels_written })
}

/// Convert an explicitly supplied unwrapped map cell into buffer coordinates.
///
/// `map_cell` and `camera_cell` are integer cell coordinates; they are not
/// masked to 0..127. Callers select the neighboring unwrapped cell while map
/// data lookup separately wraps its indices. The returned position is
/// `screen_anchor + ((dx-dy)*32, (dx+dy)*16 - 2*(height>>6) - vertical_top)`.
/// `screen_anchor` and `vertical_top` are explicit placement inputs, not a
/// claim about the original startup camera. Unlike the source's 32-bit wrap,
/// overflow is rejected for this safe helper.
pub fn terrain_vertex_buffer_position(
    map_cell: [i32; 2],
    camera_cell: [i32; 2],
    height: i16,
    vertical_top: i32,
    screen_anchor: [i32; 2],
) -> Result<[i32; 2], TerrainPrimitiveError> {
    let dx = map_cell[0].checked_sub(camera_cell[0]).ok_or(TerrainPrimitiveError::CoordinateOverflow)?;
    let dy = map_cell[1].checked_sub(camera_cell[1]).ok_or(TerrainPrimitiveError::CoordinateOverflow)?;
    let x = dx.checked_sub(dy)
        .and_then(|v| v.checked_mul(TERRAIN_HORIZONTAL_STEP))
        .and_then(|v| v.checked_add(screen_anchor[0]))
        .ok_or(TerrainPrimitiveError::CoordinateOverflow)?;
    let y = dx.checked_add(dy)
        .and_then(|v| v.checked_mul(TERRAIN_DIAGONAL_VERTICAL_STEP))
        .and_then(|v| v.checked_sub(terrain_vertex_height_displacement(height)))
        .and_then(|v| v.checked_sub(vertical_top))
        .and_then(|v| v.checked_add(screen_anchor[1]))
        .ok_or(TerrainPrimitiveError::CoordinateOverflow)?;
    Ok([x, y])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uv_table_matches_all_eight_corner_permutations() {
        let m = B32_UV_MAX_16_16;
        let expected = [
            [(0, 0), (m, 0), (m, m), (0, m)],
            [(m, 0), (0, 0), (0, m), (m, m)],
            [(0, m), (m, m), (m, 0), (0, 0)],
            [(m, m), (0, m), (0, 0), (m, 0)],
            [(0, 0), (0, m), (m, m), (m, 0)],
            [(0, m), (0, 0), (m, 0), (m, m)],
            [(m, 0), (m, m), (0, m), (0, 0)],
            [(m, m), (m, 0), (0, 0), (0, m)],
        ];
        for orientation in 0..8 {
            assert_eq!(b32_uv_corners(orientation).unwrap(), expected[usize::from(orientation)]);
            assert_eq!(b32_uv_corners_for_cell_flags(orientation << 4), expected[usize::from(orientation)]);
            assert_eq!(b32_uv_corners_for_cell_flags((orientation << 4) | 0x8f), expected[usize::from(orientation)]);
        }
        assert_eq!(b32_uv_corners(8), Err(TerrainPrimitiveError::UvOrientationOutOfRange { orientation: 8 }));
    }

    #[test]
    fn vertex_height_shifts_signed_value_before_doubling() {
        assert_eq!(terrain_vertex_height_displacement(0), 0);
        assert_eq!(terrain_vertex_height_displacement(63), 0);
        assert_eq!(terrain_vertex_height_displacement(64), 2);
        assert_eq!(terrain_vertex_height_displacement(-1), -2);
        assert_eq!(terrain_vertex_height_displacement(-63), -2);
        assert_eq!(terrain_vertex_height_displacement(-64), -2);
        assert_eq!(terrain_vertex_height_displacement(-65), -4);
        assert_eq!(terrain_vertex_height_displacement(i16::MIN), -1024);
        assert_eq!(terrain_vertex_height_displacement(i16::MAX), 1022);
    }

    #[test]
    fn shade_calculation_matches_signed_16_16_vectors_and_extrema() {
        for (light, factor, expected) in [
            (0, 0, 32768),
            (8192, 0, 32768),
            (8192, 64, 2129920),
            (3840, 64, 1015808),
            (8192, 255, 8388608),
            (-1, 64, 32512),
            (i16::MIN, 255, -33390592),
            (i16::MAX, 255, 33455108),
        ] {
            assert_eq!(terrain_vertex_shade_16_16(light, factor), expected);
        }
    }

    #[test]
    fn fade_lookup_uses_explicit_bounded_row_and_palette_index() {
        let mut bytes = vec![0u8; TERRAIN_FADE_TABLE_BYTES];
        for shade in 0..TERRAIN_FADE_SHADE_COUNT {
            for palette in 0..TERRAIN_FADE_ROW_BYTES {
                bytes[shade * TERRAIN_FADE_ROW_BYTES + palette] = (shade as u8) ^ (palette as u8);
            }
        }
        let table = TerrainFadeTable::parse(&bytes).unwrap();
        assert_eq!(table.map_palette_index(0, 0), Ok(0));
        assert_eq!(table.map_palette_index(32, 1), Ok(33));
        assert_eq!(table.map_palette_index(63, 255), Ok(192));
        assert_eq!(table.map_palette_index(64, 0), Err(TerrainPrimitiveError::FadeShadeIndexOutOfRange { shade_index: 64 }));
        assert!(matches!(TerrainFadeTable::parse(&bytes[..bytes.len() - 1]), Err(TerrainPrimitiveError::FadeTableLength { .. })));
        let mut extra = bytes;
        extra.push(0);
        assert!(matches!(TerrainFadeTable::parse(&extra), Err(TerrainPrimitiveError::FadeTableLength { .. })));
    }

    #[test]
    fn mode5_sample_uses_vu_address_then_fade_even_for_zero_source() {
        let mut texture = vec![0xA5; 65_536];
        let offset = 7 * MODE5_TEXTURE_ROW_STRIDE + 19;
        texture[offset] = 0;
        let mut fade_bytes = vec![0; TERRAIN_FADE_TABLE_BYTES];
        fade_bytes[11 * TERRAIN_FADE_ROW_BYTES] = 0xD3;
        let fade = TerrainFadeTable::parse(&fade_bytes).unwrap();
        assert_eq!(mode5_sample_index(&texture, &fade, 19, 7, 11), Ok(0xD3));
        assert_eq!(mode5_sample_index(&texture[..offset], &fade, 19, 7, 11), Err(TerrainPrimitiveError::TextureSampleOutOfBounds { offset, available: offset }));
        assert_eq!(mode5_sample_index(&texture, &fade, 19, 7, 64), Err(TerrainPrimitiveError::FadeShadeIndexOutOfRange { shade_index: 64 }));
    }

    fn synthetic_mode5_resources() -> (Vec<u8>, Vec<u8>) {
        let mut texture = vec![0; 65_536];
        for v in 0..256usize {
            for u in 0..256usize {
                texture[v * 256 + u] = ((3 * u + 5 * v) & 255) as u8;
            }
        }
        let mut fade = vec![0; TERRAIN_FADE_TABLE_BYTES];
        for shade in 0..TERRAIN_FADE_SHADE_COUNT {
            for source in 0..TERRAIN_FADE_ROW_BYTES {
                fade[shade * 256 + source] = ((source + 7 * shade) & 255) as u8;
            }
        }
        (texture, fade)
    }

    #[test]
    fn prepared_span_matches_all_receipt_synthetic_vectors() {
        let (texture, fade_bytes) = synthetic_mode5_resources();
        let fade = TerrainFadeTable::parse(&fade_bytes).unwrap();
        let vectors = [
            (Mode5SpanInput { clip_width: 16, left_16_16: 0, right_16_16: 524_288, u_16_16: 163_840, v_16_16: 278_528, shade_16_16: 688_128, du_16_16: 73_728, dv_16_16: 32_768, dshade_16_16: 0, texture_base: 0 }, 0, &[96, 99, 107, 110, 121, 124, 132, 135][..]),
            (Mode5SpanInput { clip_width: 6, left_16_16: -163_840, right_16_16: 376_832, u_16_16: 32_768, v_16_16: 65_536, shade_16_16: 294_912, du_16_16: 32_768, dv_16_16: 65_536, dshade_16_16: 16_384, texture_base: 0 }, 0, &[61, 66, 74, 86, 94][..]),
            (Mode5SpanInput { clip_width: 8, left_16_16: 163_840, right_16_16: 851_967, u_16_16: 2_064_384, v_16_16: 131_072, shade_16_16: 1_310_720, du_16_16: -65_536, dv_16_16: 16_384, dshade_16_16: 0, texture_base: 0 }, 2, &[243, 240, 237, 234, 236, 233][..]),
            (Mode5SpanInput { clip_width: 8, left_16_16: 0, right_16_16: 327_680, u_16_16: 196_607, v_16_16: 278_528, shade_16_16: 688_128, du_16_16: 0, dv_16_16: 0, dshade_16_16: -32_768, texture_base: 0 }, 0, &[96, 99, 92, 92, 85][..]),
            (Mode5SpanInput { clip_width: 8, left_16_16: 0, right_16_16: 327_680, u_16_16: 16_760_832, v_16_16: 16_744_448, shade_16_16: 655_360, du_16_16: 32_768, dv_16_16: 49_152, dshade_16_16: 0, texture_base: 0 }, 0, &[62, 70, 75, 78, 83][..]),
            (Mode5SpanInput { clip_width: 8, left_16_16: 16_384, right_16_16: 49_152, u_16_16: 0, v_16_16: 0, shade_16_16: 4_194_304, du_16_16: 0, dv_16_16: 0, dshade_16_16: 0, texture_base: 0 }, 0, &[][..]),
            (Mode5SpanInput { clip_width: 32, left_16_16: 0, right_16_16: 1_245_184, u_16_16: 65_536, v_16_16: 131_072, shade_16_16: 196_608, du_16_16: 32_768, dv_16_16: 16_384, dshade_16_16: 0, texture_base: 0 }, 0, &[34, 34, 37, 37, 45, 45, 48, 48, 56, 56, 59, 59, 67, 67, 70, 70, 78, 78, 81][..]),
            (Mode5SpanInput { clip_width: 8, left_16_16: 0, right_16_16: 393_216, u_16_16: 131_072, v_16_16: 294_912, shade_16_16: 786_432, du_16_16: 0, dv_16_16: -16_384, dshade_16_16: 0, texture_base: 0 }, 0, &[110, 110, 110, 105, 105, 105][..]),
        ];
        for (input, expected_start, expected_pixels) in vectors {
            let mut output = vec![0xCC; expected_pixels.len()];
            let receipt = render_mode5_span(&texture, &fade, input, &mut output).unwrap();
            assert_eq!(receipt, Mode5SpanReceipt { start_x: expected_start, pixel_count: expected_pixels.len() });
            assert_eq!(output, expected_pixels);
        }
    }

    #[test]
    fn prepared_span_validates_full_output_before_writing() {
        let (texture, fade_bytes) = synthetic_mode5_resources();
        let fade = TerrainFadeTable::parse(&fade_bytes).unwrap();
        let input = Mode5SpanInput { clip_width: 4, left_16_16: 0, right_16_16: 4 << 16, u_16_16: 0, v_16_16: 0, shade_16_16: 63 << 16, du_16_16: 0, dv_16_16: 0, dshade_16_16: 1 << 16, texture_base: 0 };
        let mut output = [0xA5; 4];
        assert!(matches!(render_mode5_span(&texture, &fade, input, &mut output), Err(TerrainPrimitiveError::FadeShadeIndexOutOfRange { shade_index: 64 })));
        assert_eq!(output, [0xA5; 4]);

        let mut output = [0xA5; 4];
        assert!(matches!(render_mode5_span(&texture, &fade, input, &mut output[..2]), Err(TerrainPrimitiveError::SpanOutputTooSmall { required: 4, available: 2 })));
        assert_eq!(output, [0xA5; 4]);

        let valid = Mode5SpanInput { shade_16_16: 0, dshade_16_16: 0, texture_base: 1, ..input };
        let mut output = [0xA5; 4];
        assert!(matches!(render_mode5_span(&texture[..1], &fade, valid, &mut output), Err(TerrainPrimitiveError::TextureSampleOutOfBounds { .. })));
        assert_eq!(output, [0xA5; 4]);

        let overflowing = Mode5SpanInput { texture_base: usize::MAX, u_16_16: 1 << 16, ..valid };
        let mut output = [0xA5; 4];
        assert_eq!(render_mode5_span(&texture, &fade, overflowing, &mut output), Err(TerrainPrimitiveError::TextureAddressOverflow));
        assert_eq!(output, [0xA5; 4]);

        let zero_width = Mode5SpanInput { clip_width: 0, left_16_16: -65_536, right_16_16: 65_536, ..valid };
        assert_eq!(render_mode5_span(&texture, &fade, zero_width, &mut []), Ok(Mode5SpanReceipt { start_x: 0, pixel_count: 0 }));
        let negative_width = Mode5SpanInput { clip_width: -1, ..valid };
        assert_eq!(render_mode5_span(&texture, &fade, negative_width, &mut output), Err(TerrainPrimitiveError::NegativeClipWidth { clip_width: -1 }));
    }

    #[test]
    fn vertex_position_uses_unwrapped_deltas_and_explicit_placement() {
        // dx=3,dy=2; height65 displaces 2 pixels and top contributes 7.
        assert_eq!(terrain_vertex_buffer_position([4, 3], [1, 1], 65, 7, [100, 200]), Ok([132, 271]));
        // A seam neighbor is represented as cell128, not wrapped to zero.
        assert_eq!(terrain_vertex_buffer_position([128, 0], [127, 0], 0, 0, [0, 0]), Ok([32, 16]));
        assert_eq!(terrain_vertex_buffer_position([0, 0], [1, 0], 0, 0, [640, 480]), Ok([608, 464]));
    }

    #[test]
    fn vertex_position_rejects_i32_overflow() {
        assert_eq!(terrain_vertex_buffer_position([i32::MAX, 0], [i32::MIN, 0], 0, 0, [0, 0]), Err(TerrainPrimitiveError::CoordinateOverflow));
        assert_eq!(terrain_vertex_buffer_position([i32::MAX, 0], [0, 0], 0, 0, [i32::MAX, 0]), Err(TerrainPrimitiveError::CoordinateOverflow));
    }

    fn vertex(x: i32, y: i32, u: i32, v: i32, shade: i32) -> Mode5TriangleVertex {
        Mode5TriangleVertex { x, y, u, v, shade }
    }

    #[test]
    fn triangle_rows_match_static_flat_and_middle_vectors() {
        let flat_top = prepare_mode5_triangle([
            vertex(2, 1, 0, 0, 0x208000),
            vertex(10, 1, 0x1f_ffff, 0, 0x208000),
            vertex(6, 7, 0x10_0000, 0x1f_ffff, 0x208000),
        ], 10).unwrap().unwrap();
        assert_eq!(flat_top.gradients_du_dv_dshade, [262_143, 0, 0]);
        assert_eq!(flat_top.rows[0], PreparedMode5Row {
            y: 1, left_16_16: 131_072, right_16_16: 655_360,
            u_16_16: 0, v_16_16: 0, shade_16_16: 2_129_920,
        });
        assert_eq!(flat_top.rows[3], PreparedMode5Row {
            y: 4, left_16_16: 262_142, right_16_16: 524_290,
            u_16_16: 524_286, v_16_16: 1_048_575, shade_16_16: 2_129_920,
        });
        assert_eq!(flat_top.rows.len(), 6);

        let right_middle = prepare_mode5_triangle([
            vertex(3, 1, 0, 0, 0x208000),
            vertex(12, 4, 0x1f_ffff, 0x10_0000, 0x218000),
            vertex(5, 9, 0, 0x1f_ffff, 0x208000),
        ], 10).unwrap().unwrap();
        assert_eq!(right_middle.gradients_du_dv_dshade, [233_016, 29_127, 7_281]);
        assert_eq!(right_middle.rows[0], PreparedMode5Row {
            y: 1, left_16_16: 196_608, right_16_16: 196_608,
            u_16_16: 0, v_16_16: 0, shade_16_16: 2_129_920,
        });
        assert_eq!(right_middle.rows[3].right_16_16, 786_432);
        assert_eq!(right_middle.rows[4].right_16_16, 694_682);

        let left_middle = prepare_mode5_triangle([
            vertex(7, 1, 0x10_0000, 0, 0x208000),
            vertex(11, 9, 0x1f_ffff, 0x1f_ffff, 0x208000),
            vertex(1, 4, 0, 0x10_0000, 0x218000),
        ], 10).unwrap().unwrap();
        assert_eq!(left_middle.gradients_du_dv_dshade, [180_223, -32_768, -8_192]);
        assert_eq!(left_middle.rows[3], PreparedMode5Row {
            y: 4, left_16_16: 65_536, right_16_16: 557_056,
            u_16_16: 1, v_16_16: 1_048_575, shade_16_16: 2_195_455,
        });
        // At the middle edge the short x edge resets, but attributes retain
        // accumulated division loss instead of resetting to the middle vertex.
        assert_eq!(left_middle.rows[3].u_16_16, 1);
        assert_ne!(left_middle.rows[3].u_16_16, 0);
    }

    #[test]
    fn triangle_rows_advance_accumulators_through_vertical_clipping() {
        let vertices = [
            vertex(7, -7, 0x10_0000, 0, 0x208000),
            vertex(11, 9, 0x1f_ffff, 0x1f_ffff, 0x208000),
            vertex(1, -3, 0, 0x10_0000, 0x218000),
        ];
        let clipped = prepare_mode5_triangle(vertices, 6).unwrap().unwrap();
        assert_eq!(clipped.rows.len(), 6);
        assert_eq!(clipped.rows[0], PreparedMode5Row {
            y: 0, left_16_16: 229_375, right_16_16: 573_440,
            u_16_16: 524_286, v_16_16: 1_310_719, shade_16_16: 2_179_073,
        });
        assert_eq!(clipped.rows[5].y, 5);
        assert_eq!(clipped.rows[5].u_16_16, 1_398_096);
        assert_eq!(prepare_mode5_triangle(vertices, 0), Err(TerrainPrimitiveError::TriangleClipHeightOutOfRange { clip_height: 0 }));
    }

    #[test]
    fn triangle_draw_is_bounded_and_atomic_across_all_rows() {
        let prepared = prepare_mode5_triangle([
            vertex(1, 1, 0, 0, 0),
            vertex(5, 1, 0x1f_ffff, 0, 0),
            vertex(3, 4, 0x10_0000, 0x10_0000, 0),
        ], 5).unwrap().unwrap();
        let mut fade_bytes = vec![0; TERRAIN_FADE_TABLE_BYTES];
        for shade in 0..64usize {
            for source in 0..256usize {
                fade_bytes[shade * 256 + source] = (source ^ shade) as u8;
            }
        }
        let fade = TerrainFadeTable::parse(&fade_bytes).unwrap();
        let texture = vec![7; 65_536];
        let mut framebuffer = vec![0xA5; 6 * 5 + 3];
        let receipt = draw_mode5_triangle(&texture, &fade, &prepared, 6, &mut framebuffer, 6).unwrap();
        assert_eq!(receipt.rows, 3);
        assert!(receipt.pixels_written > 0);
        assert_eq!(&framebuffer[30..], &[0xA5; 3]);

        let mut unchanged = vec![0xCC; 30];
        let original = unchanged.clone();
        assert!(matches!(draw_mode5_triangle(&texture[..8], &fade, &prepared, 6, &mut unchanged, 6), Err(TerrainPrimitiveError::TextureSampleOutOfBounds { .. })));
        assert_eq!(unchanged, original);

        // Force a late-row failure after earlier rows have been staged. The
        // caller's frame remains byte-identical on this transactional error.
        let mut late_invalid = prepared.clone();
        late_invalid.rows.last_mut().unwrap().shade_16_16 = 64 << 16;
        assert!(matches!(draw_mode5_triangle(&texture, &fade, &late_invalid, 6, &mut unchanged, 6), Err(TerrainPrimitiveError::FadeShadeIndexOutOfRange { shade_index: 64 })));
        assert_eq!(unchanged, original);

        assert!(matches!(draw_mode5_triangle(&texture, &fade, &prepared, 7, &mut unchanged, 6), Err(TerrainPrimitiveError::TriangleFramebufferPitchTooSmall { .. })));
        assert_eq!(unchanged, original);
        assert!(matches!(draw_mode5_triangle(&texture, &fade, &prepared, 6, &mut unchanged[..5], 6), Err(TerrainPrimitiveError::TriangleFramebufferTooSmall { .. })));
        assert_eq!(unchanged, original);
        assert_eq!(draw_mode5_triangle(&texture, &fade, &prepared, 6, &mut unchanged, usize::MAX), Err(TerrainPrimitiveError::TriangleFramebufferExtentOverflow));
        assert_eq!(unchanged, original);
    }

    #[test]
    fn triangle_draw_accepts_a_fully_right_clipped_triangle_without_indexing() {
        let prepared = prepare_mode5_triangle([
            vertex(1000, 0, 0, 0, 0),
            vertex(1010, 0, 0x1f_ffff, 0, 0),
            vertex(1005, 3, 0x10_0000, 0x10_0000, 0),
        ], 3).unwrap().unwrap();
        let (texture, fade_bytes) = synthetic_mode5_resources();
        let fade = TerrainFadeTable::parse(&fade_bytes).unwrap();
        let mut framebuffer = vec![0xA5; 640 * 3];
        assert_eq!(
            draw_mode5_triangle(&texture, &fade, &prepared, 640, &mut framebuffer, 640),
            Ok(Mode5TriangleDrawReceipt { rows: 3, pixels_written: 0 }),
        );
        assert!(framebuffer.iter().all(|byte| *byte == 0xA5));
    }
}
