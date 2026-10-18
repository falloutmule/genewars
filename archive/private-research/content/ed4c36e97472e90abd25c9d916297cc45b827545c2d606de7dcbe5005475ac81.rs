//! Shared, source-bound CD menu artwork. No scene or simulation state.
use genewars_assets::{FontResource, Palette256, TextStringTable};
use genewars_render::{blit_text_line, measure_text_line, CdTitleButton, RgbImage};
use std::path::Path;
const WIDTH: usize = 640;
const HEIGHT: usize = 480;
use super::read;

pub fn compose(dir: &Path, palette: &Palette256) -> Result<(Vec<RgbImage>, CdTitleButton, usize, usize), String> {
    let background = read(&dir.join("titlescr.dat"))?;
    if background.len() != WIDTH * HEIGHT {
        return Err("TITLESCR must contain exactly 640x480 decoded indices".into());
    }
    if palette.entries().iter().flatten().any(|&v| v > 63) {
        return Err("menu palette must contain RGB6 channels".into());
    }
    let rgb = background.iter().flat_map(|&index| {
        palette.entries()[usize::from(index)].map(|v| (u16::from(v) * 255 / 63) as u8)
    }).collect();
    let background = RgbImage::from_rgb24(WIDTH as u32, HEIGHT as u32, rgb).map_err(|e| e.to_string())?;
    let anb = read(&dir.join("iftitle.anb"))?;
    let button = CdTitleButton::parse(&anb).map_err(|e| e.to_string())?;
    let dat = read(&dir.join("font2.dat"))?;
    let tab = read(&dir.join("font2.tab"))?;
    let bytes = read(&dir.join("text.dat"))?;
    let font = FontResource::parse(&dat, &tab).map_err(|e| e.to_string())?;
    let text = TextStringTable::parse(&bytes).map_err(|e| e.to_string())?;
    // The initial non-network branch disables record one and repositions the
    // remaining controls. Text labels are loaded from the user's own resource.
    let mut images = Vec::with_capacity(4);
    for selected in 0..4 {
        let mut image = background.clone();
        for (row, y) in [115, 186, 257].into_iter().enumerate() {
            button.draw(&mut image, palette, 140, y, selected == row + 1).map_err(|e| e.to_string())?;
        }
        for (index, y) in [(255, 140), (258, 211), (259, 282)] {
            let label = text.strings()[index];
            let width = measure_text_line(&font, label).map_err(|e| e.to_string())?;
            blit_text_line(&mut image, &font, label, palette, 312 - (width >> 1), y)
                .map_err(|e| e.to_string())?;
        }
        images.push(image);
    }
    Ok((images, button, text.consumed_len(), text.trailing().len()))
}
