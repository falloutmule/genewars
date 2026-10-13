//! Small, evidence-bounded CPU rendering helpers for GeneWars indexed assets.

pub mod terrain;
pub mod terrain_modes;
pub mod terrain_strip;
pub mod terrain_view;
pub mod entity_shadow;
pub mod plant;
pub mod plant_draw;
pub mod plant_scene;
pub use plant::{PlantColorInputs, PlantColorRoute, PlantFrame, PlantFrameComponentRef, PlantFrameDrawReceipt, PlantRenderError, PlantShadowStatus};
pub use plant_draw::{draw_normal_plant, PlantDrawError, PlantDrawReceipt};
pub use plant_scene::{draw_explicit_family1_scene, PlantSceneError, PlantSceneReceipt, PlantSceneRecord, PlantSceneRecordReceipt, PlantSceneRecordStatus};

use genewars_assets::{AnbSections, FontResource, IndexedSprite, Palette256, PaletteChannelMode, SpriteTable, Error as AssetError};
use std::fmt;

pub const GENE_PANEL_WIDTH: u32 = 128;
pub const GENE_PANEL_HEIGHT: u32 = 480;
pub const GENE_PANEL_INDEX_BYTES: usize = GENE_PANEL_WIDTH as usize * GENE_PANEL_HEIGHT as usize;
pub const CD_SCREEN_WIDTH: u32 = 640;
pub const CD_SCREEN_HEIGHT: u32 = 480;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CdArrow {
    Up,
    Down,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    WrongPanelIndexLength { expected: usize, actual: usize },
    PaletteChannelMode(PaletteChannelMode),
    PaletteComponentOutOfRange { entry: usize, channel: usize, value: u8 },
    InvalidDimensions { width: u32, height: u32 },
    PixelBufferLength { expected: usize, actual: usize },
    DimensionsTooLarge,
    FileTooLarge,
    AllocationFailed,
    SpritePlaneLength { expected: usize, indices: usize, opacity: usize },
    Asset(AssetError),
    UnsupportedIfCustomArrowBinding,
    UnsupportedCdTitleButtonBinding,
    TitleButtonPositionOverflow,
    TextCursorOverflow,
    ZeroWidthTextGlyph { byte: u8 },
}

/// Render the two statically verified CD IFCUSTOM arrow controls on a black
/// 640×480 canvas. This draws only the two arrow sprites; it does not model the
/// surrounding screen, input, list contents, or animation timing.
pub fn render_cd_ifcustom_arrows(
    anb: &[u8],
    palette: &Palette256,
    selected: bool,
) -> Result<RgbImage, Error> {
    let sections = AnbSections::parse(anb).map_err(Error::Asset)?;
    let metadata = sections.ui_metadata().map_err(Error::Asset)?;
    let sprites = SpriteTable::from_anb(anb).map_err(Error::Asset)?;
    let mut image = RgbImage::from_rgb24(CD_SCREEN_WIDTH, CD_SCREEN_HEIGHT, vec![0; (CD_SCREEN_WIDTH * CD_SCREEN_HEIGHT * 3) as usize])?;
    for (animation_index, control_x, control_y) in [(6usize, 455i32, 229i32), (7, 455, 261)] {
        let animation = metadata.animation(animation_index).map_err(Error::Asset)?;
        if animation.trailing_word != 2
            || animation.last_frame_offset.checked_sub(animation.first_frame_offset) != Some(14)
        {
            return Err(Error::UnsupportedIfCustomArrowBinding);
        }
        let frame_offset = if selected { animation.last_frame_offset } else { animation.first_frame_offset };
        let frame = metadata.frame_at(frame_offset).map_err(Error::Asset)?;
        if frame.component_count != 1 || frame.display_dimensions() != (19, 31) {
            return Err(Error::UnsupportedIfCustomArrowBinding);
        }
        let component = metadata.frame_component(frame, 0).map_err(Error::Asset)?;
        if component.flags != 0 {
            return Err(Error::UnsupportedIfCustomArrowBinding);
        }
        let (local_x, local_y) = component.local_position();
        let sprite_index = component.sprite_offset as usize / 6;
        let descriptor = sprites.records().get(sprite_index).ok_or(Error::UnsupportedIfCustomArrowBinding)?;
        if (descriptor.width, descriptor.height) != (19, 32) {
            return Err(Error::UnsupportedIfCustomArrowBinding);
        }
        let sprite = sprites.decode(sprite_index, anb).map_err(Error::Asset)?;
        image.blit_indexed_sprite(
            &sprite,
            palette,
            control_x + i32::from(local_x),
            control_y + i32::from(local_y),
        )?;
    }
    Ok(image)
}

/// Compose the two evidence-bound controls with at most one captured/selected.
pub fn render_cd_ifcustom_controls(
    anb: &[u8],
    palette: &Palette256,
    selected: Option<CdArrow>,
) -> Result<RgbImage, Error> {
    let sections = AnbSections::parse(anb).map_err(Error::Asset)?;
    let metadata = sections.ui_metadata().map_err(Error::Asset)?;
    let sprites = SpriteTable::from_anb(anb).map_err(Error::Asset)?;
    let mut image = RgbImage::from_rgb24(CD_SCREEN_WIDTH, CD_SCREEN_HEIGHT,
        vec![0; (CD_SCREEN_WIDTH * CD_SCREEN_HEIGHT * 3) as usize])?;
    for (arrow, animation_index, control_x, control_y) in [
        (CdArrow::Up, 6usize, 455i32, 229i32),
        (CdArrow::Down, 7usize, 455i32, 261i32),
    ] {
        let (sprite, x, y) = verified_arrow_sprite(anb, &metadata, &sprites, animation_index, selected == Some(arrow))?;
        image.blit_indexed_sprite(&sprite, palette, control_x + x, control_y + y)?;
    }
    Ok(image)
}

/// Return the first control, in original list order, whose currently displayed
/// frame has an opaque sprite pixel at the supplied 640x480 screen coordinate.
pub fn hit_test_cd_ifcustom_arrows(
    anb: &[u8],
    selected: Option<CdArrow>,
    x: i32,
    y: i32,
) -> Result<Option<CdArrow>, Error> {
    let sections = AnbSections::parse(anb).map_err(Error::Asset)?;
    let metadata = sections.ui_metadata().map_err(Error::Asset)?;
    let sprites = SpriteTable::from_anb(anb).map_err(Error::Asset)?;
    for (arrow, animation_index, control_x, control_y) in [
        (CdArrow::Up, 6usize, 455i32, 229i32),
        (CdArrow::Down, 7usize, 455i32, 261i32),
    ] {
        let (sprite, local_x, local_y) = verified_arrow_sprite(anb, &metadata, &sprites, animation_index, selected == Some(arrow))?;
        let source_x = i64::from(x) - i64::from(control_x) - i64::from(local_x);
        let source_y = i64::from(y) - i64::from(control_y) - i64::from(local_y);
        if source_x >= 0 && source_y >= 0
            && source_x < i64::from(sprite.width) && source_y < i64::from(sprite.height)
            && sprite.opaque[source_y as usize * usize::from(sprite.width) + source_x as usize]
        {
            return Ok(Some(arrow));
        }
    }
    Ok(None)
}

fn verified_arrow_sprite(
    anb: &[u8],
    metadata: &genewars_assets::AnbUiMetadata<'_>,
    sprites: &SpriteTable,
    animation_index: usize,
    selected: bool,
) -> Result<(IndexedSprite, i32, i32), Error> {
    let animation = metadata.animation(animation_index).map_err(Error::Asset)?;
    if animation.trailing_word != 2
        || animation.last_frame_offset.checked_sub(animation.first_frame_offset) != Some(14)
    {
        return Err(Error::UnsupportedIfCustomArrowBinding);
    }
    let frame_offset = if selected { animation.last_frame_offset } else { animation.first_frame_offset };
    let frame = metadata.frame_at(frame_offset).map_err(Error::Asset)?;
    if frame.component_count != 1 || frame.display_dimensions() != (19, 31) {
        return Err(Error::UnsupportedIfCustomArrowBinding);
    }
    let component = metadata.frame_component(frame, 0).map_err(Error::Asset)?;
    if component.flags != 0 {
        return Err(Error::UnsupportedIfCustomArrowBinding);
    }
    let (local_x, local_y) = component.local_position();
    let sprite_index = component.sprite_offset as usize / 6;
    let descriptor = sprites.records().get(sprite_index).ok_or(Error::UnsupportedIfCustomArrowBinding)?;
    if (descriptor.width, descriptor.height) != (19, 32) {
        return Err(Error::UnsupportedIfCustomArrowBinding);
    }
    let sprite = sprites.decode(sprite_index, anb).map_err(Error::Asset)?;
    Ok((sprite, i32::from(local_x), i32::from(local_y)))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TitleButtonComponent {
    sprite: IndexedSprite,
    x: i16,
    y: i16,
}

/// Cached normal/selected artwork for the one traced CD title-button ANB.
///
/// Parsing admits only its single animation (frame byte offsets 0 and 28,
/// trailing word 3), 346×75 frames, three components per frame, and zero
/// component flags. It intentionally does not implement component transforms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CdTitleButton {
    normal: Vec<TitleButtonComponent>,
    selected: Vec<TitleButtonComponent>,
}

impl CdTitleButton {
    pub fn parse(anb: &[u8]) -> Result<Self, Error> {
        let sections = AnbSections::parse(anb).map_err(Error::Asset)?;
        let metadata = sections.ui_metadata().map_err(Error::Asset)?;
        if metadata.animation_count() != 1 {
            return Err(Error::UnsupportedCdTitleButtonBinding);
        }
        let animation = metadata.animation(0).map_err(Error::Asset)?;
        if animation.first_frame_offset != 0
            || animation.last_frame_offset != 28
            || animation.trailing_word != 3
        {
            return Err(Error::UnsupportedCdTitleButtonBinding);
        }
        let sprites = SpriteTable::from_anb(anb).map_err(Error::Asset)?;
        let normal = parse_title_button_frame(anb, &metadata, &sprites, 0)?;
        let selected = parse_title_button_frame(anb, &metadata, &sprites, 28)?;
        Ok(Self { normal, selected })
    }

    /// Test local signed coordinates against the union of normal-frame opaque
    /// literal pixels. Selected artwork never changes the hit footprint.
    pub fn hit_test(&self, x: i32, y: i32) -> bool {
        self.normal.iter().any(|component| {
            let source_x = i64::from(x) - i64::from(component.x);
            let source_y = i64::from(y) - i64::from(component.y);
            source_x >= 0
                && source_y >= 0
                && source_x < i64::from(component.sprite.width)
                && source_y < i64::from(component.sprite.height)
                && component.sprite.opaque[
                    source_y as usize * usize::from(component.sprite.width) + source_x as usize
                ]
        })
    }

    /// Draw the cached normal frame or selected last frame at signed screen
    /// coordinates. Sprite opacity is the explicit positive-literal mask;
    /// palette index zero is drawn when that mask marks it opaque.
    pub fn draw(
        &self,
        image: &mut RgbImage,
        palette: &Palette256,
        x: i32,
        y: i32,
        selected: bool,
    ) -> Result<(), Error> {
        let components = if selected { &self.selected } else { &self.normal };
        for component in components {
            let component_x = x.checked_add(i32::from(component.x))
                .ok_or(Error::TitleButtonPositionOverflow)?;
            let component_y = y.checked_add(i32::from(component.y))
                .ok_or(Error::TitleButtonPositionOverflow)?;
            image.blit_indexed_sprite(&component.sprite, palette, component_x, component_y)?;
        }
        Ok(())
    }
}

fn parse_title_button_frame(
    anb: &[u8],
    metadata: &genewars_assets::AnbUiMetadata<'_>,
    sprites: &SpriteTable,
    frame_offset: u32,
) -> Result<Vec<TitleButtonComponent>, Error> {
    let frame = metadata.frame_at(frame_offset).map_err(Error::Asset)?;
    if frame.component_count != 3 || frame.display_dimensions() != (346, 75) {
        return Err(Error::UnsupportedCdTitleButtonBinding);
    }
    let mut components = Vec::new();
    components.try_reserve_exact(3).map_err(|_| Error::AllocationFailed)?;
    for local_index in 0..3 {
        let component = metadata.frame_component(frame, local_index).map_err(Error::Asset)?;
        if component.flags != 0 {
            return Err(Error::UnsupportedCdTitleButtonBinding);
        }
        let sprite_index = component.sprite_offset as usize / 6;
        let sprite = sprites.decode(sprite_index, anb).map_err(Error::Asset)?;
        let (x, y) = component.local_position();
        components.push(TitleButtonComponent { sprite, x, y });
    }
    Ok(components)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

/// Packed row-major RGB bytes with a top-left origin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbImage {
    width: u32,
    height: u32,
    rgb: Vec<u8>,
}

impl RgbImage {
    /// Construct an RGB image after validating dimensions and exact buffer size.
    pub fn from_rgb24(width: u32, height: u32, rgb: Vec<u8>) -> Result<Self, Error> {
        let expected = rgb_len(width, height)?;
        if rgb.len() != expected {
            return Err(Error::PixelBufferLength { expected, actual: rgb.len() });
        }
        Ok(Self { width, height, rgb })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn rgb_bytes(&self) -> &[u8] {
        &self.rgb
    }

    /// Composite opaque sprite pixels at a caller-selected signed position.
    /// Negative or beyond-edge destination pixels are clipped. Source index zero
    /// is drawn whenever its explicit opacity-mask entry is true.
    pub fn blit_indexed_sprite(
        &mut self,
        sprite: &IndexedSprite,
        palette: &Palette256,
        x: i32,
        y: i32,
    ) -> Result<(), Error> {
        let expected = usize::from(sprite.width) * usize::from(sprite.height);
        if sprite.indices.len() != expected || sprite.opaque.len() != expected {
            return Err(Error::SpritePlaneLength {
                expected,
                indices: sprite.indices.len(),
                opacity: sprite.opaque.len(),
            });
        }
        validate_rgb6_palette(palette)?;

        let dest_width = self.width as usize;
        for source_y in 0..usize::from(sprite.height) {
            let dest_y = i64::from(y) + source_y as i64;
            if dest_y < 0 || dest_y >= i64::from(self.height) {
                continue;
            }
            for source_x in 0..usize::from(sprite.width) {
                let source_index = source_y * usize::from(sprite.width) + source_x;
                if !sprite.opaque[source_index] {
                    continue;
                }
                let dest_x = i64::from(x) + source_x as i64;
                if dest_x < 0 || dest_x >= i64::from(self.width) {
                    continue;
                }
                let [red, green, blue] = palette.entries()[sprite.indices[source_index] as usize];
                let dest_index = (dest_y as usize * dest_width + dest_x as usize) * 3;
                self.rgb[dest_index..dest_index + 3].copy_from_slice(&[
                    to_u8_display(red),
                    to_u8_display(green),
                    to_u8_display(blue),
                ]);
            }
        }
        Ok(())
    }

    /// Serialize as deterministic Windows-compatible 24-bit uncompressed BMP.
    /// The output is bottom-up BGR with four-byte row alignment.
    pub fn to_bmp_bytes(&self) -> Result<Vec<u8>, Error> {
        let row_bytes = usize::try_from(self.width).map_err(|_| Error::DimensionsTooLarge)?
            .checked_mul(3).ok_or(Error::FileTooLarge)?;
        let stride = row_bytes.checked_add(3).ok_or(Error::FileTooLarge)? & !3;
        let pixel_bytes = stride.checked_mul(self.height as usize).ok_or(Error::FileTooLarge)?;
        let file_bytes = 54usize.checked_add(pixel_bytes).ok_or(Error::FileTooLarge)?;
        let file_size = u32::try_from(file_bytes).map_err(|_| Error::FileTooLarge)?;
        let image_size = u32::try_from(pixel_bytes).map_err(|_| Error::FileTooLarge)?;
        let width = i32::try_from(self.width).map_err(|_| Error::DimensionsTooLarge)?;
        let height = i32::try_from(self.height).map_err(|_| Error::DimensionsTooLarge)?;

        let mut bmp = Vec::new();
        bmp.try_reserve_exact(file_bytes).map_err(|_| Error::AllocationFailed)?;
        bmp.extend_from_slice(b"BM");
        bmp.extend_from_slice(&file_size.to_le_bytes());
        bmp.extend_from_slice(&[0; 4]);
        bmp.extend_from_slice(&54u32.to_le_bytes());
        bmp.extend_from_slice(&40u32.to_le_bytes());
        bmp.extend_from_slice(&width.to_le_bytes());
        bmp.extend_from_slice(&height.to_le_bytes());
        bmp.extend_from_slice(&1u16.to_le_bytes());
        bmp.extend_from_slice(&24u16.to_le_bytes());
        bmp.extend_from_slice(&0u32.to_le_bytes());
        bmp.extend_from_slice(&image_size.to_le_bytes());
        bmp.extend_from_slice(&0i32.to_le_bytes());
        bmp.extend_from_slice(&0i32.to_le_bytes());
        bmp.extend_from_slice(&0u32.to_le_bytes());
        bmp.extend_from_slice(&0u32.to_le_bytes());

        let padding = stride - row_bytes;
        let padding_bytes = [0u8; 3];
        for y in (0..self.height as usize).rev() {
            let start = y.checked_mul(row_bytes).ok_or(Error::FileTooLarge)?;
            let row = &self.rgb[start..start + row_bytes];
            for pixel in row.chunks_exact(3) {
                bmp.extend_from_slice(&[pixel[2], pixel[1], pixel[0]]);
            }
            bmp.extend_from_slice(&padding_bytes[..padding]);
        }
        Ok(bmp)
    }
}

/// Expand a verified 128×480, row-major 8-bit index plane with a raw RGB
/// six-bit DAC palette. Components are scaled for display as floor(v*255/63);
/// this presentation conversion does not claim analog VGA output fidelity.
pub fn render_gene_panel(indices: &[u8], palette: &Palette256) -> Result<RgbImage, Error> {
    if indices.len() != GENE_PANEL_INDEX_BYTES {
        return Err(Error::WrongPanelIndexLength {
            expected: GENE_PANEL_INDEX_BYTES,
            actual: indices.len(),
        });
    }
    validate_rgb6_palette(palette)?;
    let output_len = rgb_len(GENE_PANEL_WIDTH, GENE_PANEL_HEIGHT)?;
    let mut rgb = Vec::new();
    rgb.try_reserve_exact(output_len).map_err(|_| Error::AllocationFailed)?;
    for &index in indices {
        let [red, green, blue] = palette.entries()[index as usize];
        rgb.extend_from_slice(&[to_u8_display(red), to_u8_display(green), to_u8_display(blue)]);
    }
    RgbImage::from_rgb24(GENE_PANEL_WIDTH, GENE_PANEL_HEIGHT, rgb)
}

/// Measure the traced unkerned single-line branch. It accepts bytes >= 32,
/// advances by glyph width minus one, and rejects controls rather than
/// guessing newline, wrapping, or formatting behavior.
pub fn measure_text_line(font: &FontResource<'_>, text: &[u8]) -> Result<i32, Error> {
    let mut width = 0i32;
    for &byte in text {
        let glyph = font.record_for_byte(byte).map_err(Error::Asset)?;
        if glyph.width == 0 {
            return Err(Error::ZeroWidthTextGlyph { byte });
        }
        width = width.checked_add(i32::from(glyph.width) - 1).ok_or(Error::TextCursorOverflow)?;
    }
    Ok(width)
}

/// Draw one unkerned line using the caller-selected font and palette. Space
/// advances but is not painted. Literal glyph bytes, including palette index
/// zero, follow the font RLE opacity mask. Returns the final x cursor.
pub fn blit_text_line(
    image: &mut RgbImage,
    font: &FontResource<'_>,
    text: &[u8],
    palette: &Palette256,
    x: i32,
    y: i32,
) -> Result<i32, Error> {
    let _ = measure_text_line(font, text)?;
    validate_rgb6_palette(palette)?;
    let mut cursor = x;
    for &byte in text {
        let glyph_record = font.record_for_byte(byte).map_err(Error::Asset)?;
        if byte != b' ' {
            let glyph = font.decode_glyph(glyph_record.record).map_err(Error::Asset)?;
            image.blit_indexed_sprite(&glyph, palette, cursor, y)?;
        }
        cursor = cursor.checked_add(i32::from(glyph_record.width) - 1).ok_or(Error::TextCursorOverflow)?;
    }
    Ok(cursor)
}

fn validate_rgb6_palette(palette: &Palette256) -> Result<(), Error> {
    if palette.channel_mode() != PaletteChannelMode::Rgb {
        return Err(Error::PaletteChannelMode(palette.channel_mode()));
    }
    for (entry_index, entry) in palette.entries().iter().enumerate() {
        for (channel, &value) in entry.iter().enumerate() {
            if value > 63 {
                return Err(Error::PaletteComponentOutOfRange { entry: entry_index, channel, value });
            }
        }
    }
    Ok(())
}

fn to_u8_display(component: u8) -> u8 {
    ((u16::from(component) * 255) / 63) as u8
}

fn rgb_len(width: u32, height: u32) -> Result<usize, Error> {
    if width == 0 || height == 0 {
        return Err(Error::InvalidDimensions { width, height });
    }
    if width > i32::MAX as u32 || height > i32::MAX as u32 {
        return Err(Error::DimensionsTooLarge);
    }
    usize::try_from(width).map_err(|_| Error::DimensionsTooLarge)?
        .checked_mul(usize::try_from(height).map_err(|_| Error::DimensionsTooLarge)?)
        .and_then(|pixels| pixels.checked_mul(3))
        .ok_or(Error::FileTooLarge)
}

#[cfg(test)]
mod font_line_tests {
    use super::*;

    fn table(widths: &[(u8, u8)]) -> Vec<u8> {
        let mut table = vec![0; 6];
        for &(width, height) in widths {
            table.extend_from_slice(&0u32.to_le_bytes());
            table.extend_from_slice(&[width, height]);
        }
        table
    }

    fn palette() -> Palette256 {
        let mut bytes = vec![0; 768];
        bytes[0..3].copy_from_slice(&[63, 0, 0]);
        bytes[3..6].copy_from_slice(&[0, 63, 0]);
        Palette256::parse(&bytes, PaletteChannelMode::Rgb).unwrap()
    }

    #[test]
    fn text_line_measures_width_minus_one_and_paints_opaque_zero_indices() {
        let data = [2, 0, 1, 0];
        let table = table(&[(3, 1), (2, 1)]);
        let font = FontResource::parse(&data, &table).unwrap();
        assert_eq!(measure_text_line(&font, b"! !").unwrap(), 4);
        let mut image = RgbImage::from_rgb24(4, 1, vec![0; 12]).unwrap();
        assert_eq!(blit_text_line(&mut image, &font, b"! !", &palette(), 0, 0).unwrap(), 4);
        assert_eq!(&image.rgb_bytes()[0..3], &[255, 0, 0]);
        assert_eq!(&image.rgb_bytes()[3..6], &[0, 255, 0]);
        assert_eq!(&image.rgb_bytes()[9..12], &[255, 0, 0]);
    }

    #[test]
    fn text_line_rejects_controls_and_zero_width_glyphs_without_partial_draw() {
        let data = [1, 1, 0];
        let table = table(&[(1, 1), (0, 1)]);
        let font = FontResource::parse(&data, &table).unwrap();
        assert_eq!(measure_text_line(&font, b"\n"), Err(Error::Asset(AssetError::FontUnsupportedTextByte { byte: 10 })));
        assert_eq!(measure_text_line(&font, b" "), Ok(0));
        assert_eq!(measure_text_line(&font, b"!"), Err(Error::ZeroWidthTextGlyph { byte: b'!' }));
        let mut image = RgbImage::from_rgb24(2, 1, vec![0; 6]).unwrap();
        assert!(blit_text_line(&mut image, &font, b" \n", &palette(), 0, 0).is_err());
        assert_eq!(image.rgb_bytes(), &[0; 6]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_rgb_palette() -> Palette256 {
        let mut bytes = vec![0; 256 * 3];
        bytes[3..6].copy_from_slice(&[63, 0, 0]);
        bytes[6..9].copy_from_slice(&[0, 63, 0]);
        bytes[9..12].copy_from_slice(&[0, 0, 63]);
        Palette256::parse(&bytes, PaletteChannelMode::Rgb).unwrap()
    }

    fn synthetic_ifcustom_arrows() -> Vec<u8> {
        const RASTER_BYTES: usize = 21 * 32;
        const SPRITE_TABLE: usize = 18 + 4 * RASTER_BYTES;
        const COMPONENT_TABLE: usize = SPRITE_TABLE + 24;
        const FRAME_TABLE: usize = COMPONENT_TABLE + 40;
        const ANIMATION_TABLE: usize = FRAME_TABLE + 56;
        let mut anb = vec![0; ANIMATION_TABLE + 80];
        anb[..2].copy_from_slice(&8u16.to_le_bytes());
        for (index, offset) in [SPRITE_TABLE as u32, COMPONENT_TABLE as u32, FRAME_TABLE as u32, ANIMATION_TABLE as u32].into_iter().enumerate() {
            anb[2 + index * 4..6 + index * 4].copy_from_slice(&offset.to_le_bytes());
        }
        for sprite in 0..4usize {
            let start = 18 + sprite * RASTER_BYTES;
            for row in 0..32usize {
                let row_start = start + row * 21;
                anb[row_start] = 19;
                anb[row_start + 1..row_start + 20].fill(1);
                anb[row_start + 20] = 0;
            }
            let table = SPRITE_TABLE + sprite * 6;
            anb[table..table + 4].copy_from_slice(&((sprite * RASTER_BYTES) as u32).to_le_bytes());
            anb[table + 4..table + 6].copy_from_slice(&[19, 32]);
            let component = COMPONENT_TABLE + sprite * 10;
            anb[component..component + 4].copy_from_slice(&((sprite * 6) as u32).to_le_bytes());
            anb[component + 4..component + 6].copy_from_slice(&0i16.to_le_bytes());
            anb[component + 6..component + 8].copy_from_slice(&if sprite % 2 == 0 { 0i16 } else { 2i16 }.to_le_bytes());
            let frame = FRAME_TABLE + sprite * 14;
            anb[frame..frame + 4].copy_from_slice(&((sprite * 10) as u32).to_le_bytes());
            anb[frame + 4..frame + 6].copy_from_slice(&38i16.to_le_bytes());
            anb[frame + 6..frame + 8].copy_from_slice(&62i16.to_le_bytes());
            anb[frame + 11] = 1;
        }
        for animation in 0..8usize {
            let first_last = match animation {
                6 => (0u32, 14u32),
                7 => (28u32, 42u32),
                _ => (0u32, 0u32),
            };
            let start = ANIMATION_TABLE + animation * 10;
            anb[start..start + 4].copy_from_slice(&first_last.0.to_le_bytes());
            anb[start + 4..start + 8].copy_from_slice(&first_last.1.to_le_bytes());
            anb[start + 8..start + 10].copy_from_slice(&2u16.to_le_bytes());
        }
        anb
    }

    fn red_index_palette() -> Palette256 {
        let mut bytes = vec![0; 256 * 3];
        bytes[3..6].copy_from_slice(&[63, 0, 0]);
        Palette256::parse(&bytes, PaletteChannelMode::Rgb).unwrap()
    }

    fn pixel(image: &RgbImage, x: usize, y: usize) -> [u8; 3] {
        let offset = (y * CD_SCREEN_WIDTH as usize + x) * 3;
        image.rgb_bytes()[offset..offset + 3].try_into().unwrap()
    }

    #[test]
    fn composes_verified_custom_arrows_at_component_adjusted_positions() {
        let anb = synthetic_ifcustom_arrows();
        let palette = red_index_palette();
        let normal = render_cd_ifcustom_arrows(&anb, &palette, false).unwrap();
        let selected = render_cd_ifcustom_arrows(&anb, &palette, true).unwrap();
        assert_eq!((normal.width(), normal.height()), (640, 480));
        assert_eq!(pixel(&normal, 455, 229), [255, 0, 0]);
        assert_eq!(pixel(&normal, 455, 261), [255, 0, 0]);
        assert_eq!(pixel(&selected, 455, 230), [255, 0, 0]);
        assert_eq!(pixel(&selected, 455, 262), [255, 0, 0]);
        assert_eq!(pixel(&selected, 455, 229), [0, 0, 0]);
        assert_eq!(pixel(&selected, 455, 261), [255, 0, 0]);
    }

    #[test]
    fn custom_arrow_hit_test_uses_selected_frame_offsets_and_opaque_pixels() {
        let anb = synthetic_ifcustom_arrows();
        assert_eq!(hit_test_cd_ifcustom_arrows(&anb, None, 455, 229).unwrap(), Some(CdArrow::Up));
        assert_eq!(hit_test_cd_ifcustom_arrows(&anb, None, 455, 261).unwrap(), Some(CdArrow::Down));
        assert_eq!(hit_test_cd_ifcustom_arrows(&anb, None, 474, 229).unwrap(), None);
        assert_eq!(hit_test_cd_ifcustom_arrows(&anb, None, i32::MIN, i32::MIN).unwrap(), None);
        assert_eq!(hit_test_cd_ifcustom_arrows(&anb, None, i32::MAX, i32::MAX).unwrap(), None);
        assert_eq!(hit_test_cd_ifcustom_arrows(&anb, Some(CdArrow::Up), 455, 229).unwrap(), None);
        assert_eq!(hit_test_cd_ifcustom_arrows(&anb, Some(CdArrow::Up), 455, 230).unwrap(), Some(CdArrow::Up));
        // The up control's selected raster reaches y=261; original list order
        // checks it before the down control at the shared boundary.
        assert_eq!(hit_test_cd_ifcustom_arrows(&anb, Some(CdArrow::Up), 455, 261).unwrap(), Some(CdArrow::Up));
    }

    #[test]
    fn composes_only_the_captured_arrow_in_selected_frame() {
        let anb = synthetic_ifcustom_arrows();
        let image = render_cd_ifcustom_controls(&anb, &red_index_palette(), Some(CdArrow::Up)).unwrap();
        assert_eq!(pixel(&image, 455, 230), [255, 0, 0]);
        assert_eq!(pixel(&image, 455, 229), [0, 0, 0]);
        assert_eq!(pixel(&image, 455, 261), [255, 0, 0]);
    }

    #[test]
    fn rejects_unproven_ifcustom_animation_shapes() {
        let mut anb = synthetic_ifcustom_arrows();
        let animation_table = u32::from_le_bytes(anb[14..18].try_into().unwrap()) as usize;
        anb[animation_table + 6 * 10 + 8..animation_table + 6 * 10 + 10].copy_from_slice(&3u16.to_le_bytes());
        assert_eq!(
            render_cd_ifcustom_arrows(&anb, &red_index_palette(), false),
            Err(Error::UnsupportedIfCustomArrowBinding)
        );
    }

    #[test]
    fn renders_fixed_panel_rgb_order_scale_and_orientation() {
        let mut indices = vec![0; GENE_PANEL_INDEX_BYTES];
        indices[0] = 1;
        indices[1] = 2;
        indices[GENE_PANEL_INDEX_BYTES - GENE_PANEL_WIDTH as usize] = 3;
        let image = render_gene_panel(&indices, &synthetic_rgb_palette()).unwrap();
        assert_eq!((image.width(), image.height()), (128, 480));
        assert_eq!(&image.rgb_bytes()[..9], &[255, 0, 0, 0, 255, 0, 0, 0, 0]);
        let bottom_left = (GENE_PANEL_INDEX_BYTES - GENE_PANEL_WIDTH as usize) * 3;
        assert_eq!(&image.rgb_bytes()[bottom_left..bottom_left + 3], &[0, 0, 255]);
    }

    #[test]
    fn scales_midrange_dac_components_with_floor() {
        let mut bytes = vec![0; 256 * 3];
        bytes[3..6].copy_from_slice(&[32, 47, 0]);
        let palette = Palette256::parse(&bytes, PaletteChannelMode::Rgb).unwrap();
        let mut indices = vec![0; GENE_PANEL_INDEX_BYTES];
        indices[0] = 1;
        let image = render_gene_panel(&indices, &palette).unwrap();
        assert_eq!(&image.rgb_bytes()[..3], &[129, 190, 0]);
    }

    #[test]
    fn composites_explicit_opacity_with_destination_clipping_and_midtones() {
        let mut palette_bytes = vec![0; 256 * 3];
        palette_bytes[..3].copy_from_slice(&[32, 47, 0]);
        palette_bytes[6..9].copy_from_slice(&[63, 0, 0]);
        palette_bytes[15..18].copy_from_slice(&[0, 0, 63]);
        let palette = Palette256::parse(&palette_bytes, PaletteChannelMode::Rgb).unwrap();
        let sprite = IndexedSprite {
            width: 3,
            height: 2,
            indices: vec![0, 1, 2, 3, 0, 5],
            opaque: vec![true, false, true, true, true, true],
            consumed_bytes: 0,
        };
        let mut background = RgbImage::from_rgb24(2, 2, [12, 13, 14].repeat(4)).unwrap();
        background.blit_indexed_sprite(&sprite, &palette, -1, 0).unwrap();
        assert_eq!(background.rgb_bytes(), &[
            12, 13, 14, 255, 0, 0,
            129, 190, 0, 0, 0, 255,
        ]);
    }

    #[test]
    fn rejects_sprite_planes_that_do_not_match_declared_dimensions() {
        let palette = synthetic_rgb_palette();
        let sprite = IndexedSprite {
            width: 2,
            height: 1,
            indices: vec![1],
            opaque: vec![true],
            consumed_bytes: 0,
        };
        let mut background = RgbImage::from_rgb24(1, 1, vec![0; 3]).unwrap();
        assert_eq!(
            background.blit_indexed_sprite(&sprite, &palette, 0, 0),
            Err(Error::SpritePlaneLength { expected: 2, indices: 1, opacity: 1 })
        );
    }

    #[test]
    fn rejects_bad_index_length_palette_mode_and_components() {
        assert_eq!(
            render_gene_panel(&[], &synthetic_rgb_palette()).unwrap_err(),
            Error::WrongPanelIndexLength { expected: GENE_PANEL_INDEX_BYTES, actual: 0 }
        );

        let bgr = Palette256::parse(&[0; 256 * 3], PaletteChannelMode::Bgr).unwrap();
        assert_eq!(
            render_gene_panel(&vec![0; GENE_PANEL_INDEX_BYTES], &bgr).unwrap_err(),
            Error::PaletteChannelMode(PaletteChannelMode::Bgr)
        );

        let mut invalid = vec![0; 256 * 3];
        invalid[3] = 64;
        let palette = Palette256::parse(&invalid, PaletteChannelMode::Rgb).unwrap();
        assert_eq!(
            render_gene_panel(&vec![0; GENE_PANEL_INDEX_BYTES], &palette).unwrap_err(),
            Error::PaletteComponentOutOfRange { entry: 1, channel: 0, value: 64 }
        );
    }

    #[test]
    fn bmp_has_expected_windows_header_and_rgb_to_bgr_conversion() {
        let image = RgbImage::from_rgb24(1, 1, vec![1, 2, 3]).unwrap();
        let bmp = image.to_bmp_bytes().unwrap();
        assert_eq!(&bmp[..2], b"BM");
        assert_eq!(u32::from_le_bytes(bmp[2..6].try_into().unwrap()), 58);
        assert_eq!(u32::from_le_bytes(bmp[10..14].try_into().unwrap()), 54);
        assert_eq!(u32::from_le_bytes(bmp[34..38].try_into().unwrap()), 4);
        assert_eq!(&bmp[54..58], &[3, 2, 1, 0]);
    }

    #[test]
    fn rejects_invalid_rgb_image_buffer_dimensions() {
        assert_eq!(
            RgbImage::from_rgb24(1, 1, vec![0; 2]),
            Err(Error::PixelBufferLength { expected: 3, actual: 2 })
        );
        assert_eq!(
            RgbImage::from_rgb24(0, 1, vec![]),
            Err(Error::InvalidDimensions { width: 0, height: 1 })
        );
    }
}

#[cfg(test)]
mod cd_title_button_tests {
    use super::*;

    const COMPONENT_BYTES: usize = 10;

    fn synthetic_title_anb() -> (Vec<u8>, usize, usize, usize) {
        let mut anb = vec![0u8; 18];
        let sprite_rle = [1, 0, 0xFE, 0, 0xFF, 1, 1, 0xFF, 0];
        for _ in 0..3 {
            anb.extend_from_slice(&sprite_rle);
        }
        let sprite_table = anb.len();
        for sprite in 0..3u32 {
            anb.extend_from_slice(&(sprite * sprite_rle.len() as u32).to_le_bytes());
            anb.extend_from_slice(&[3, 2]);
        }
        let component_table = anb.len();
        let positions = [
            [(0i16, 0i16), (4, 0), (0, 3)],
            [(0i16, 0i16), (4, 0), (0, 3)],
            [(0i16, 5i16), (4, 5), (0, 8)],
        ];
        for frame_components in positions {
            for (index, (x, y)) in frame_components.into_iter().enumerate() {
                let sprite_index = index as u32;
                anb.extend_from_slice(&(sprite_index * 6).to_le_bytes());
                anb.extend_from_slice(&(x * 2).to_le_bytes());
                anb.extend_from_slice(&(y * 2).to_le_bytes());
                anb.extend_from_slice(&[0, 0]);
            }
        }
        let frame_table = anb.len();
        for (component_offset, width, height, count) in [
            (0u32, 692i16, 150i16, 3u8),
            (0u32, 692i16, 150i16, 3u8),
            (6 * COMPONENT_BYTES as u32, 692i16, 150i16, 3u8),
        ] {
            anb.extend_from_slice(&component_offset.to_le_bytes());
            anb.extend_from_slice(&width.to_le_bytes());
            anb.extend_from_slice(&height.to_le_bytes());
            anb.extend_from_slice(&[0, 0, 0, count, 0, 0]);
        }
        let animation_table = anb.len();
        anb.extend_from_slice(&0u32.to_le_bytes());
        anb.extend_from_slice(&28u32.to_le_bytes());
        anb.extend_from_slice(&3u16.to_le_bytes());

        anb[..2].copy_from_slice(&1u16.to_le_bytes());
        for (index, offset) in [
            sprite_table as u32,
            component_table as u32,
            frame_table as u32,
            animation_table as u32,
        ]
        .into_iter()
        .enumerate()
        {
            let start = 2 + index * 4;
            anb[start..start + 4].copy_from_slice(&offset.to_le_bytes());
        }
        (anb, component_table, frame_table, animation_table)
    }

    fn palette() -> Palette256 {
        let mut bytes = vec![0u8; 256 * 3];
        for (index, color) in [
            [63, 0, 0],
            [0, 63, 0],
            [0, 0, 63],
        ]
        .into_iter()
        .enumerate()
        {
            bytes[index * 3..index * 3 + 3].copy_from_slice(&color);
        }
        Palette256::parse(&bytes, PaletteChannelMode::Rgb).unwrap()
    }

    fn pixel(image: &RgbImage, x: usize, y: usize) -> [u8; 3] {
        let start = (y * image.width() as usize + x) * 3;
        image.rgb_bytes()[start..start + 3].try_into().unwrap()
    }

    #[test]
    fn title_button_hit_uses_union_of_normal_positive_literal_masks() {
        let (anb, _, _, _) = synthetic_title_anb();
        let button = CdTitleButton::parse(&anb).unwrap();
        assert_eq!(button.selected[0].y, 5);
        assert_eq!(button.selected[1].y, 5);
        assert_eq!(button.selected[2].y, 8);
        assert!(button.hit_test(0, 0)); // literal palette index zero is opaque
        assert!(!button.hit_test(1, 0)); // transparent hole in each sprite box
        assert!(button.hit_test(1, 1));
        assert!(button.hit_test(4, 0)); // second component
        assert!(button.hit_test(0, 3)); // third component
        assert!(!button.hit_test(-1, 0));
        assert!(!button.hit_test(i32::MIN, i32::MAX));
        assert!(!button.hit_test(i32::MAX, i32::MIN));
        // These coordinates hit the selected composition, but not the normal
        // footprint used for callback hit testing.
        assert!(!button.hit_test(0, 5));
    }

    #[test]
    fn title_button_draws_cached_normal_or_selected_components_at_signed_origin() {
        let (anb, _, _, _) = synthetic_title_anb();
        let button = CdTitleButton::parse(&anb).unwrap();
        assert_eq!(button.selected[0].y, 5);
        assert_eq!(button.selected[1].y, 5);
        assert_eq!(button.selected[2].y, 8);
        let mut normal = RgbImage::from_rgb24(12, 12, vec![0; 12 * 12 * 3]).unwrap();
        button.draw(&mut normal, &palette(), 1, 1, false).unwrap();
        assert_eq!(pixel(&normal, 1, 1), [255, 0, 0]); // opaque source index 0
        assert_eq!(pixel(&normal, 2, 1), [0, 0, 0]); // transparent source hole
        assert_eq!(pixel(&normal, 2, 2), [0, 255, 0]);
        assert_eq!(pixel(&normal, 5, 1), [255, 0, 0]);

        let mut selected = RgbImage::from_rgb24(12, 12, vec![0; 12 * 12 * 3]).unwrap();
        button.draw(&mut selected, &palette(), 1, 1, true).unwrap();
        assert_eq!(pixel(&selected, 1, 1), [0, 0, 0]);
        assert_eq!(pixel(&selected, 1, 6), [255, 0, 0]);
        assert_eq!(pixel(&selected, 2, 7), [0, 255, 0]);

        assert_eq!(
            button.draw(&mut selected, &palette(), i32::MAX, 0, true),
            Err(Error::TitleButtonPositionOverflow)
        );
    }

    #[test]
    fn title_button_rejects_unproven_animation_frames_or_component_flags() {
        let (mut wrong_animation, _, _, animation_table) = synthetic_title_anb();
        wrong_animation[animation_table + 8..animation_table + 10]
            .copy_from_slice(&4u16.to_le_bytes());
        assert_eq!(
            CdTitleButton::parse(&wrong_animation),
            Err(Error::UnsupportedCdTitleButtonBinding)
        );

        let (mut wrong_frame, _, frame_table, _) = synthetic_title_anb();
        wrong_frame[frame_table + 4..frame_table + 6]
            .copy_from_slice(&690i16.to_le_bytes());
        assert_eq!(
            CdTitleButton::parse(&wrong_frame),
            Err(Error::UnsupportedCdTitleButtonBinding)
        );

        let (mut transformed_component, component_table, _, _) = synthetic_title_anb();
        transformed_component[component_table + 8] = 1;
        assert_eq!(
            CdTitleButton::parse(&transformed_component),
            Err(Error::UnsupportedCdTitleButtonBinding)
        );
    }
}
