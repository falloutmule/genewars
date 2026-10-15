//! CD1E840/1F3C0/98C40 opaque main-image hit. Queue priority belongs to caller.
use genewars_runtime::building_resources::BuildingAnimation0Frame;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildingPickError { EmptyAnimation, InvalidComponents, InvalidOpacity, CoordinateRange }

/// `origin` is the absolute screen ground origin, including the panel margin.
/// The caller resolves animation19 and applies source visibility/family gates.
pub fn hit_animation(
    frames: &[BuildingAnimation0Frame], current_frame: u8,
    origin: [i32; 2], mouse: [i32; 2],
) -> Result<Option<usize>, BuildingPickError> {
    if frames.is_empty() { return Err(BuildingPickError::EmptyAnimation); }
    if origin.into_iter().chain(mouse).any(|v| i16::try_from(v).is_err()) {
        return Err(BuildingPickError::CoordinateRange);
    }
    let frame = &frames[usize::from(current_frame).min(frames.len() - 1)];
    if frame.components.is_empty() || frame.components.len() != usize::from(frame.metadata.component_count) {
        return Err(BuildingPickError::InvalidComponents);
    }
    for (index, component) in frame.components.iter().enumerate().rev() {
        let (ox, oy) = component.record.local_position();
        let sprite = &component.sprite;
        let width = usize::from(sprite.width);
        let height = usize::from(sprite.height);
        if sprite.opaque.len() != width * height { return Err(BuildingPickError::InvalidOpacity); }
        // Source subtracts in32 bits then passes signed low words to98B60.
        let mut x = mouse[0] - origin[0] - i32::from(ox);
        let mut y = mouse[1] - origin[1] - i32::from(oy);
        if component.record.flags & 1 != 0 { x = width as i32 - 1 - x; }
        if component.record.flags & 2 != 0 { y = height as i32 - 1 - y; }
        let x = i32::from(x as i16);
        let y = i32::from(y as i16);
        if x >= 0 && y >= 0 && (x as usize) < width && (y as usize) < height
            && sprite.opaque[y as usize * width + x as usize] {
            return Ok(Some(index));
        }
    }
    Ok(None)
}
