//! Small, evidence-bounded CPU rendering helpers for GeneWars indexed assets.

use genewars_assets::{AnbSections, IndexedSprite, Palette256, PaletteChannelMode, SpriteTable, Error as AssetError};
use std::fmt;

pub const GENE_PANEL_WIDTH: u32 = 128;
pub const GENE_PANEL_HEIGHT: u32 = 480;
pub const GENE_PANEL_INDEX_BYTES: usize = GENE_PANEL_WIDTH as usize * GENE_PANEL_HEIGHT as usize;
pub const CD_SCREEN_WIDTH: u32 = 640;
pub const CD_SCREEN_HEIGHT: u32 = 480;

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
