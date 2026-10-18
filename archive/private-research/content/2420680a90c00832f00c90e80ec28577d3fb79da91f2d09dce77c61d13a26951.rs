//! Source-bounded selected-actor name and category text.
//!
//! The panel formatter at 0x265F0 receives the generated name through the
//! typed blueprint metadata binding at actor +0xC0 and a category string from
//! TEXT indices 128..131. It formats `%s\n%s` in font context 0x187054. That
//! context is initialized from descriptor slot zero; the newline handler
//! advances by the height byte of its first glyph record (space).

use genewars_assets::{FontResource, Palette256, TextStringTable};
use genewars_render::{measure_text_line, RgbImage, CD_SCREEN_HEIGHT, CD_SCREEN_WIDTH};
use genewars_runtime::{actor_world::ActorWorld, actor_initialization::BlueprintRecordHandle};

const NAME_BYTES: usize = 20;
const BLUEPRINT_CATEGORY_OFFSET: usize = 0x16;
const CATEGORY_TEXT_BASE: usize = 128;
const TEXT_CLIP_ORIGIN: [usize; 2] = [84, 382];
const TEXT_CLIP_SIZE: [usize; 2] = [78, 21];
const PANEL_BARS: [(usize, usize, usize, u8); 3] = [
    (0x17, 116, 415, 0xE8),
    (0x19, 116, 435, 0xC4),
    (0x1A, 116, 435, 0xC8),
];
// In the selected-panel font context, 0x1FF30 treats these literal glyph
// bytes as the second context color marker (context+0x18). Byte 1 selects
// context+0x14. Other literal bytes are copied as palette indices.
const FONT_BLACK_MARKERS: &[u8] = &[2, 4, 12, 16, 20, 24, 32, 36, 40, 48, 52, 56, 60];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectedActorTextReceipt {
    pub actor_slot: usize,
    pub blueprint: BlueprintRecordHandle,
    pub category_index: usize,
    pub name_origin: [usize; 2],
    pub category_origin: [usize; 2],
    pub line_advance: u8,
    pub name_width: i32,
    pub category_width: i32,
    pub bars: [Option<PanelBarReceipt>; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelBarReceipt {
    pub blueprint_field_offset: usize,
    pub raw_signed_value: i8,
    pub origin: [usize; 2],
    pub size: [usize; 2],
    pub palette_index: u8,
}

/// Resolve the actor's typed blueprint metadata and draw the two strings sent
/// by the traced `%s\n%s` formatter. FONT0/TEXT/palette are caller-supplied
/// hash-admitted resources; this helper does not load or reinterpret them.
pub fn draw_selected_actor_details(
    image: &mut RgbImage,
    font: &FontResource<'_>,
    text: &TextStringTable<'_>,
    palette: &Palette256,
    world: &ActorWorld,
    actor_slot: usize,
) -> Result<SelectedActorTextReceipt, String> {
    if image.width() != CD_SCREEN_WIDTH || image.height() != CD_SCREEN_HEIGHT {
        return Err(format!("selected actor details require a {CD_SCREEN_WIDTH}x{CD_SCREEN_HEIGHT} framebuffer"));
    }
    let actor = world.actors().record(actor_slot).map_err(|e| e.to_string())?;
    if actor[6] != 3 {
        return Err(format!("selected slot {actor_slot} is not an admitted family-2 actor"));
    }
    let blueprint = world.actors().bindings(actor_slot).map_err(|e| e.to_string())?.blueprint
        .ok_or_else(|| format!("selected actor {actor_slot} has no typed blueprint binding"))?;
    if blueprint.player_id != actor[0x18] {
        return Err(format!("selected actor {actor_slot} blueprint owner does not match its actor record"));
    }
    let record = world.player(blueprint.player_id).map_err(|e| e.to_string())?
        .blueprint(usize::from(blueprint.blueprint_index)).map_err(|e| e.to_string())?;
    draw_blueprint_details(image, font, text, palette, actor_slot, blueprint, record)
}

/// Draw from a blueprint already obtained through a validated typed handle.
/// Exposed separately for independent byte-oracle tests and callers that have
/// already resolved the handle.
pub fn draw_blueprint_details(
    image: &mut RgbImage,
    font: &FontResource<'_>,
    text: &TextStringTable<'_>,
    palette: &Palette256,
    actor_slot: usize,
    blueprint: BlueprintRecordHandle,
    record: &[u8; 36],
) -> Result<SelectedActorTextReceipt, String> {
    if image.width() != CD_SCREEN_WIDTH || image.height() != CD_SCREEN_HEIGHT {
        return Err(format!("selected actor details require a {CD_SCREEN_WIDTH}x{CD_SCREEN_HEIGHT} framebuffer"));
    }
    let name_end = record[..NAME_BYTES].iter().position(|&byte| byte == 0)
        .ok_or_else(|| "blueprint name is not NUL-terminated within its 20-byte field".to_owned())?;
    let name = &record[..name_end];
    if name.is_empty() { return Err("selected actor blueprint name is empty".into()); }
    let category = record[BLUEPRINT_CATEGORY_OFFSET];
    if category > 3 { return Err(format!("blueprint category {category} is outside the admitted 0..4 range")); }
    let category_index = CATEGORY_TEXT_BASE + usize::from(category);
    let category_text = *text.strings().get(category_index)
        .ok_or_else(|| format!("TEXT table has no category entry {category_index}"))?;

    // Measure and decode the complete pair before modifying the framebuffer.
    // This prevents a malformed later glyph from leaving the name half-drawn.
    let name_width = measure_text_line(font, name).map_err(|e| e.to_string())?;
    let category_width = measure_text_line(font, category_text).map_err(|e| e.to_string())?;
    for &byte in name.iter().chain(category_text) {
        if byte != b' ' {
            font.decode_glyph(font.record_for_byte(byte).map_err(|e| e.to_string())?.record)
                .map_err(|e| e.to_string())?;
        }
    }
    let line_advance = font.record(0).map_err(|e| e.to_string())?.height;
    if line_advance == 0 { return Err("FONT0 space glyph has zero height; newline advance is undefined".into()); }
    // The selected-panel caller sets a 78x21 text viewport at (84,382) with
    // 1F490, then calls 1F750 with local cursor (0,0). Render into a copy of
    // that viewport so writes cannot escape its source-established clip.
    let mut region = Vec::with_capacity(TEXT_CLIP_SIZE[0] * TEXT_CLIP_SIZE[1] * 3);
    for row in 0..TEXT_CLIP_SIZE[1] {
        let start = ((TEXT_CLIP_ORIGIN[1] + row) * image.width() as usize + TEXT_CLIP_ORIGIN[0]) * 3;
        region.extend_from_slice(&image.rgb_bytes()[start..start + TEXT_CLIP_SIZE[0] * 3]);
    }
    let mut viewport = RgbImage::from_rgb24(TEXT_CLIP_SIZE[0] as u32, TEXT_CLIP_SIZE[1] as u32, region)
        .map_err(|e| e.to_string())?;
    let second_y = i32::from(line_advance);
    let white_index = nearest_palette_index(palette, [63, 63, 63]);
    let black_index = nearest_palette_index(palette, [0, 0, 0]);
    blit_selected_panel_line(&mut viewport, font, name, palette, white_index, black_index, 0, 0)?;
    blit_selected_panel_line(&mut viewport, font, category_text, palette, white_index, black_index, 0, second_y)?;
    let mut final_bytes = image.rgb_bytes().to_vec();
    for row in 0..TEXT_CLIP_SIZE[1] {
        let dest = ((TEXT_CLIP_ORIGIN[1] + row) * image.width() as usize + TEXT_CLIP_ORIGIN[0]) * 3;
        let src = row * TEXT_CLIP_SIZE[0] * 3;
        final_bytes[dest..dest + TEXT_CLIP_SIZE[0] * 3]
            .copy_from_slice(&viewport.rgb_bytes()[src..src + TEXT_CLIP_SIZE[0] * 3]);
    }
    let bars = draw_raw_blueprint_bars(&mut final_bytes, image.width() as usize, image.height() as usize, palette, record);
    let next = RgbImage::from_rgb24(image.width(), image.height(), final_bytes).map_err(|e| e.to_string())?;
    *image = next;
    Ok(SelectedActorTextReceipt {
        actor_slot,
        blueprint,
        category_index,
        name_origin: TEXT_CLIP_ORIGIN,
        category_origin: [TEXT_CLIP_ORIGIN[0], TEXT_CLIP_ORIGIN[1] + usize::from(line_advance)],
        line_advance,
        name_width,
        category_width,
        bars,
    })
}

/// The game computes these two indices by nearest-color search against its
/// active palette. `MAIN.PAL` is the palette bound by this caller.
fn nearest_palette_index(palette: &Palette256, target: [u8; 3]) -> u8 {
    palette.entries().iter().enumerate().min_by_key(|(index, entry)| {
        let distance = entry.iter().zip(target).map(|(&value, wanted)| {
            let delta = i32::from(value) - i32::from(wanted);
            delta * delta
        }).sum::<i32>();
        (distance, *index)
    }).map(|(index, _)| index as u8).unwrap_or(0)
}

fn blit_selected_panel_line(
    image: &mut RgbImage,
    font: &FontResource<'_>,
    text: &[u8],
    palette: &Palette256,
    white_index: u8,
    black_index: u8,
    x: i32,
    y: i32,
) -> Result<(), String> {
    let width = image.width() as usize;
    let height = image.height() as usize;
    let mut bytes = image.rgb_bytes().to_vec();
    let mut cursor_x = x;
    for &byte in text {
        let record = font.record_for_byte(byte).map_err(|e| e.to_string())?.record;
        let glyph = font.decode_glyph(record).map_err(|e| e.to_string())?;
        if byte != b' ' {
            for gy in 0..usize::from(glyph.height) {
                for gx in 0..usize::from(glyph.width) {
                    let source = gy * usize::from(glyph.width) + gx;
                    if !glyph.opaque[source] { continue; }
                    let px = i64::from(cursor_x) + gx as i64;
                    let py = i64::from(y) + gy as i64;
                    if px < 0 || py < 0 || px >= width as i64 || py >= height as i64 { continue; }
                    let literal = glyph.indices[source];
                    let index = if literal == 1 { white_index }
                        else if FONT_BLACK_MARKERS.contains(&literal) { black_index }
                        else { literal };
                    let color = palette.entries()[usize::from(index)].map(|v| ((u16::from(v) * 255) / 63) as u8);
                    let at = (py as usize * width + px as usize) * 3;
                    bytes[at..at + 3].copy_from_slice(&color);
                }
            }
        }
        cursor_x = cursor_x.checked_add(i32::from(glyph.width) - 1)
            .ok_or_else(|| "selected-panel text cursor overflow".to_owned())?;
    }
    *image = RgbImage::from_rgb24(image.width(), image.height(), bytes).map_err(|e| e.to_string())?;
    Ok(())
}

/// Draw the three source-submitted rectangles. Their associated fields remain
/// unnamed: this function records only their signed-byte and geometry facts.
fn draw_raw_blueprint_bars(
    frame: &mut [u8], width: usize, height: usize, palette: &Palette256, record: &[u8; 36],
) -> [Option<PanelBarReceipt>; 3] {
    let mut receipts = [None; 3];
    for (slot, &(field, x, y, color)) in PANEL_BARS.iter().enumerate() {
        let value = record[field] as i8;
        if value <= 0 { continue; }
        let bar_width = ((45_i32 * i32::from(value)) / 100).max(1) as usize;
        let color_rgb = palette.entries()[usize::from(color)].map(|component|
            ((u16::from(component) * 255) / 63) as u8);
        for py in y..(y + 6).min(height) {
            for px in x..(x + bar_width).min(width) {
                let at = (py * width + px) * 3;
                frame[at..at + 3].copy_from_slice(&color_rgb);
            }
        }
        receipts[slot] = Some(PanelBarReceipt {
            blueprint_field_offset: field,
            raw_signed_value: value,
            origin: [x, y],
            size: [bar_width, 6],
            palette_index: color,
        });
    }
    receipts
}

#[cfg(test)]
mod tests {
    use super::*;
    use genewars_assets::PaletteChannelMode;

    fn font_fixture() -> (Vec<u8>, Vec<u8>) {
        let data = vec![0, 1, 1, 0, 1, 2, 0];
        let mut table = vec![0; 6];
        for record in 0..96 {
            let width = if record == 0 { 2 } else { 1 };
            let height = if record == 0 { 2 } else { 1 };
            let offset = if record == 0 { 0 } else if record == 33 { 1 } else if record == 34 { 4 } else { 0 };
            table.extend_from_slice(&(offset as u32).to_le_bytes());
            table.extend_from_slice(&[width, height]);
        }
        // Record 33 ('A') is the white marker; record 34 ('B') is a black marker.
        (data, table)
    }

    fn text_fixture() -> Vec<u8> {
        let mut bytes = Vec::new();
        for index in 0..604 {
            let value: &[u8] = match index { 130 => b"B", _ => b"x" };
            bytes.extend_from_slice(value);
            bytes.push(0);
        }
        bytes
    }

    #[test]
    fn two_line_name_and_category_use_blueprint_bytes_and_space_height() {
        let (dat, tab) = font_fixture();
        let font = FontResource::parse(&dat, &tab).unwrap();
        let text_bytes = text_fixture();
        let text = TextStringTable::parse(&text_bytes).unwrap();
        let mut pal = vec![0; 768];
        pal[145 * 3..145 * 3 + 3].copy_from_slice(&[63, 63, 63]);
        let palette = Palette256::parse(&pal, PaletteChannelMode::Rgb).unwrap();
        let mut record = [0; 36];
        record[0] = b'A';
        record[1] = 0;
        record[0x16] = 2;
        let mut image = RgbImage::from_rgb24(640, 480, vec![0; 640 * 480 * 3]).unwrap();
        let handle = BlueprintRecordHandle { player_id: 1, blueprint_index: 7 };
        let receipt = draw_blueprint_details(&mut image, &font, &text, &palette, 3, handle, &record).unwrap();
        assert_eq!(receipt.category_index, 130);
        assert_eq!(receipt.name_origin, [84, 382]);
        assert_eq!(receipt.category_origin, [84, 384]);
        assert_eq!(&image.rgb_bytes()[((382 * 640 + 84) * 3)..((382 * 640 + 84) * 3 + 3)], &[255, 255, 255]);
        assert_eq!(&image.rgb_bytes()[((384 * 640 + 84) * 3)..((384 * 640 + 84) * 3 + 3)], &[0, 0, 0]);
    }

    #[test]
    fn malformed_second_line_is_rejected_before_any_pixels_change() {
        let (dat, tab) = font_fixture();
        let _font = FontResource::parse(&dat, &tab).unwrap();
        let text_bytes = text_fixture();
        let text = TextStringTable::parse(&text_bytes).unwrap();
        let palette = Palette256::parse(&vec![0; 768], PaletteChannelMode::Rgb).unwrap();
        let mut record = [0; 36]; record[0] = b'A'; record[0x16] = 2;
        let mut image = RgbImage::from_rgb24(640, 480, vec![17; 640 * 480 * 3]).unwrap();
        // Corrupt B's stream offset so preflight fails after the first line's glyph.
        let mut bad_table = tab.clone();
        let offset = 6 + 34 * 6;
        bad_table[offset..offset + 4].copy_from_slice(&u32::MAX.to_le_bytes());
        let bad_font = FontResource::parse(&dat, &bad_table).unwrap();
        let before = image.rgb_bytes().to_vec();
        assert!(draw_blueprint_details(&mut image, &bad_font, &text, &palette, 0,
            BlueprintRecordHandle { player_id: 0, blueprint_index: 0 }, &record).is_err());
        assert_eq!(image.rgb_bytes(), before);
    }

    #[test]
    fn raw_bar_fields_are_signed_nonpositive_skips_and_third_call_overlays_second() {
        let mut record = [0; 36];
        record[0x17] = 1;
        record[0x19] = 100;
        record[0x1A] = 50;
        let mut bytes = vec![0; 640 * 480 * 3];
        let mut palette_bytes = vec![0; 768];
        palette_bytes[0xE8 * 3..0xE8 * 3 + 3].copy_from_slice(&[63, 0, 0]);
        palette_bytes[0xC4 * 3..0xC4 * 3 + 3].copy_from_slice(&[0, 63, 0]);
        palette_bytes[0xC8 * 3..0xC8 * 3 + 3].copy_from_slice(&[0, 0, 63]);
        let palette = Palette256::parse(&palette_bytes, PaletteChannelMode::Rgb).unwrap();
        let bars = draw_raw_blueprint_bars(&mut bytes, 640, 480, &palette, &record);
        assert_eq!(bars[0].unwrap().size, [1, 6]);
        assert_eq!(bars[1].unwrap().size, [45, 6]);
        assert_eq!(bars[2].unwrap().size, [22, 6]);
        // Both rows are drawn in call order; palette C8 is last at overlap.
        let overlap = (435 * 640 + 116) * 3;
        assert_eq!(&bytes[overlap..overlap + 3], &[0, 0, 255]);
        let second_only = (435 * 640 + 140) * 3;
        assert_eq!(&bytes[second_only..second_only + 3], &[0, 255, 0]);
        record[0x19] = 0xff;
        assert!((record[0x19] as i8) < 0);
        assert!(draw_raw_blueprint_bars(&mut bytes, 640, 480, &palette, &record)[1].is_none());
    }
}
