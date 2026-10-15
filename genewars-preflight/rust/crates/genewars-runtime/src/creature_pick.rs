//! Literal 95300/952A0 scaled component rectangle picking; pixels are not read.
//! Frame resolution (1E840), visibility, ordering and world control are callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PickComponent {
    pub offset: [i16; 2],
    pub sprite_dimensions: Option<[u8; 2]>,
    pub flags: u8,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickError { EmptyFrame, TooManyComponents, Factor }
/// 952A0 rectangle membership, including the source flip transforms.
/// Bounded scale 0..=256 keeps sprite products in the proved signed domain.
pub fn hit_component(component: PickComponent, relative: [i32; 2], factor: u16)
    -> Result<bool, PickError>
{
    if factor > 256 { return Err(PickError::Factor); }
    let Some(dimensions) = component.sprite_dimensions else { return Ok(false) };
    let extent = dimensions.map(|v| (i32::from(v) * i32::from(factor)) >> 8);
    let mut at = relative;
    for axis in 0..2 {
        if component.flags & (1 << axis) != 0 {
            at[axis] = extent[axis].wrapping_sub(at[axis]).wrapping_sub(1);
        }
        if at[axis] < 0 || at[axis] >= extent[axis] { return Ok(false); }
    }
    Ok(true)
}
/// 95300 returns the last matching component, or None (source -1).
/// Unlike extents, signed component offsets use division truncated toward zero.
pub fn hit_frame(components: &[PickComponent], origin: [i32; 2], point: [i32; 2], factor: u16)
    -> Result<Option<u8>, PickError>
{
    if components.is_empty() { return Err(PickError::EmptyFrame); }
    if components.len() > 255 { return Err(PickError::TooManyComponents); }
    if factor > 256 { return Err(PickError::Factor); }
    for (index, &component) in components.iter().enumerate().rev() {
        let scaled = component.offset.map(|v| i32::from(v) * i32::from(factor) / 256);
        let relative = [point[0].wrapping_sub(origin[0].wrapping_add(scaled[0])),
            point[1].wrapping_sub(origin[1].wrapping_add(scaled[1]))];
        if hit_component(component, relative, factor)? { return Ok(Some(index as u8)); }
    }
    Ok(None)
}
/// 5A140 inputs are signed screen words; relative_height is virtual +4's result.
/// The supplied components must be the resource-bound, source-clamped frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MulePickInput {
    pub scale76: u8,
    pub scale77: u8,
    pub ground: [i16; 2],
    pub point: [i16; 2],
    pub relative_height: i32,
}
pub fn mule_factor(scale76: u8, scale77: u8) -> u16 {
    ((u32::from(scale76) * u32::from(scale77) / 100).clamp(150, 256)) as u16
}
pub fn hit_mule(components: &[PickComponent], input: MulePickInput) -> Result<bool, PickError> {
    let origin = [i32::from(input.ground[0]),
        i32::from(input.ground[1]).wrapping_sub(input.relative_height >> 6)];
    Ok(hit_frame(components, origin, input.point.map(i32::from),
        mule_factor(input.scale76, input.scale77))?.is_some())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn last_rectangle_wins_and_transparency_is_not_an_input() {
        let c = PickComponent { offset: [-1, -1], sprite_dimensions: Some([4, 4]), flags: 3 };
        assert_eq!(hit_frame(&[c, c], [0, 0], [0, 0], 150), Ok(Some(1)));
        assert_eq!(hit_frame(&[c], [0, 0], [-1, 0], 150), Ok(None));
        assert_eq!(hit_frame(&[c], [0, 0], [2, 0], 150), Ok(None));
    }
    #[test]
    fn wrapper_preserves_scale255_and_signed_height_shift() {
        assert_eq!(mule_factor(100, 255), 255);
        assert_eq!(mule_factor(0, 255), 150);
        assert_eq!(mule_factor(255, 255), 256);
        let c = PickComponent { offset: [0, 0], sprite_dimensions: Some([2, 2]), flags: 0 };
        let i = MulePickInput { scale76: 100, scale77: 255, ground: [10, 10], point: [10, 11], relative_height: -1 };
        assert_eq!(hit_mule(&[c], i), Ok(true));
        assert_eq!(hit_frame(&[], [0, 0], [0, 0], 256), Err(PickError::EmptyFrame));
        assert_eq!(hit_frame(&[c], [0, 0], [0, 0], 257), Err(PickError::Factor));
    }
}
