//! Bounded CD7B550/7AE20 terrain target conversion.
//!
//! Explicit32/16 pixel steps, height factor2 and a640x480 screen. This is
//! independent of874B0's minimap conversion. The caller supplies the current
//! camera globals and must perform the separate UI/controller eligibility.
//! Failed intersections return an error instead of transforming stale caller
//! output words as the original routine can do. No world state is mutated.

const MAP_BYTES: usize = 128 * 128 * 14;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainTargetError {
    MapLength(usize),
    CameraOutsideAdmittedRange([u16; 2]),
    ScreenOutsideViewport([i32; 2]),
    NoIntersection,
    DivisionOverflow,
}
impl core::fmt::Display for TerrainTargetError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f,"{self:?}") }
}
impl std::error::Error for TerrainTargetError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainTargetTriangle { First, Second }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerrainTarget {
    pub point: [i16; 2],
    pub cell_index: u16,
    /// Zero-based pair in the source's bounded40-pair walk.
    pub strip_pair: u8,
    pub triangle: TerrainTargetTriangle,
    pub clamped_screen_y: i32,
}

fn height_screen(map: &[u8], x: i32, y: i32) -> i32 {
    let at = (((y & 127) * 128 + (x & 127)) * 14 + 2) as usize;
    2 * (i32::from(i16::from_le_bytes([map[at],map[at+1]])) >> 6)
}

/// Literal7AE20 signed32 inverse, returning the two low-word weights only
/// when the original writes both output words. No upper weight clamp exists.
fn triangle_inverse(point: [i32; 2], vertices: [[i32; 2]; 3])
    -> Result<Option<[u16; 2]>, TerrainTargetError>
{
    let [[a,b],[c,d],[e,f]] = vertices;
    let [px,py] = point;
    let dx = e.wrapping_sub(a);
    let dy = f.wrapping_sub(b);
    let first_y = d.wrapping_sub(b);
    let first_x = c.wrapping_sub(a);
    let numerator = py.wrapping_sub(b).wrapping_mul(dx)
        .wrapping_add(a.wrapping_sub(px).wrapping_mul(dy));
    let determinant = first_y.wrapping_mul(dx).wrapping_sub(first_x.wrapping_mul(dy));
    if determinant == 0 { return Ok(None); }
    let alpha = numerator.wrapping_shl(8).checked_div(determinant)
        .ok_or(TerrainTargetError::DivisionOverflow)?;
    if alpha < 0 { return Ok(None); }
    let (second_numerator, second_divisor) = if dx == 0 {
        (py.wrapping_shl(8).wrapping_sub(b.wrapping_shl(8))
            .wrapping_sub(first_y.wrapping_mul(alpha)),dy)
    } else {
        (px.wrapping_shl(8).wrapping_sub(a.wrapping_shl(8))
            .wrapping_sub(first_x.wrapping_mul(alpha)),dx)
    };
    let beta = second_numerator.checked_div(second_divisor)
        .ok_or(TerrainTargetError::DivisionOverflow)?;
    if beta < 0 { return Ok(None); }
    Ok(Some([alpha as u16,beta as u16]))
}

fn finish(point: [i16; 2], pair: u8, triangle: TerrainTargetTriangle, y: i32) -> TerrainTarget {
    let cell_index = ((point[1] as u16 >> 8) & 127) * 128 + ((point[0] as u16 >> 8) & 127);
    TerrainTarget { point, cell_index, strip_pair: pair, triangle, clamped_screen_y: y }
}

/// Convert an admitted absolute screen point using the current source camera
/// cell globals. Camera words0..255 include82E20's zero-extended byte writes.
/// ScreenY0..4 uses5, exactly as the source. Panel hit rejection belongs to the
/// caller; no panel margin is subtracted here.
///
/// Products and intermediate shifts use source wrapping32 arithmetic; final
/// successful coordinates retain signed low-word behavior. On a zero/negative
/// intersection or40-pair miss the original can reuse previous output bytes.
/// This safe API reports `NoIntersection`, with no fabricated prior point.
pub fn project_terrain_target(map: &[u8], camera: [u16; 2], screen: [i32; 2])
    -> Result<TerrainTarget, TerrainTargetError>
{
    if map.len() != MAP_BYTES { return Err(TerrainTargetError::MapLength(map.len())); }
    if camera.iter().any(|&v| v > 255) { return Err(TerrainTargetError::CameraOutsideAdmittedRange(camera)); }
    if !(0..640).contains(&screen[0]) || !(0..480).contains(&screen[1]) {
        return Err(TerrainTargetError::ScreenOutsideViewport(screen));
    }
    let column = screen[0] / 32;
    let fraction = ((screen[0] % 32) << 8) / 32;
    let target_y = screen[1].max(5);
    let half = column >> 1;
    let mut map_x = i32::from(camera[0]) + half;
    let mut map_y = i32::from(camera[1]) - half;
    let mut baseline = if column & 1 == 0 { 0 } else { map_y -= 1; -16 };
    let mut left_y = baseline - height_screen(map,map_x,map_y);
    let mut prior_y = left_y;
    map_x += 1;
    baseline += 16;
    let mut right_y = baseline - height_screen(map,map_x,map_y);
    // Source stack+20 starts as the first right vertex, then retains each
    // newly sampled left vertex across the following pair iteration.
    let mut cached_y = right_y;
    let mut left_map_x = map_x - 1;
    for pair in 0..40u8 {
        let first_edge = left_y.wrapping_add(right_y.wrapping_sub(left_y).wrapping_mul(fraction) >> 8);
        let saved_prior = prior_y;
        prior_y = cached_y;
        if target_y <= first_edge {
            let Some([alpha,beta]) = triangle_inverse([fraction,target_y.wrapping_sub(saved_prior)],
                [[0,cached_y.wrapping_sub(saved_prior)],[256,0],[256,right_y.wrapping_sub(saved_prior)]])?
                else { return Err(TerrainTargetError::NoIntersection); };
            let point = [(left_map_x.wrapping_shl(8) as u16).wrapping_add(beta) as i16,
                (map_y.wrapping_shl(8) as u16).wrapping_sub(alpha) as i16];
            return Ok(finish(point,pair,TerrainTargetTriangle::First,target_y));
        }
        baseline += 16;
        map_y += 1;
        let next_left_y = baseline - height_screen(map,map_x,map_y);
        cached_y = next_left_y;
        let second_edge = next_left_y.wrapping_add(right_y.wrapping_sub(next_left_y).wrapping_mul(fraction) >> 8);
        let saved_right = prior_y;
        prior_y = right_y;
        if target_y <= second_edge {
            let Some([alpha,beta]) = triangle_inverse([fraction,target_y.wrapping_sub(saved_right)],
                [[256,right_y.wrapping_sub(saved_right)],[0,0],[0,next_left_y.wrapping_sub(saved_right)]])?
                else { return Err(TerrainTargetError::NoIntersection); };
            let point = [(left_map_x.wrapping_shl(8) as u16).wrapping_add(255u16.wrapping_sub(alpha)) as i16,
                ((map_y-1).wrapping_shl(8) as u16).wrapping_add(beta) as i16];
            return Ok(finish(point,pair,TerrainTargetTriangle::Second,target_y));
        }
        baseline += 16;
        map_x += 1;
        left_map_x += 1;
        left_y = next_left_y;
        right_y = baseline - height_screen(map,map_x,map_y);
    }
    Err(TerrainTargetError::NoIntersection)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inverse_preserves_source_division_and_rejects_unwritten_outputs() {
        assert_eq!(triangle_inverse([64,64],[[0,0],[256,0],[0,256]]).unwrap(),Some([64,64]));
        assert_eq!(triangle_inverse([-1,64],[[0,0],[256,0],[0,256]]).unwrap(),None);
        assert_eq!(triangle_inverse([0,0],[[0,0],[1,1],[2,2]]).unwrap(),None);
        assert_eq!(triangle_inverse([512,256],[[0,0],[256,0],[0,256]]).unwrap(),Some([512,256]));
    }
    #[test]
    fn bounded_inputs_reject_without_map_mutation() {
        let map=vec![0;MAP_BYTES];let before=map.clone();
        assert_eq!(project_terrain_target(&map[..MAP_BYTES-1],[0,0],[0,0]),Err(TerrainTargetError::MapLength(MAP_BYTES-1)));
        assert!(matches!(project_terrain_target(&map,[256,0],[0,0]),Err(TerrainTargetError::CameraOutsideAdmittedRange(_))));
        for screen in [[-1,0],[640,0],[0,-1],[0,480],[i32::MIN,i32::MAX]] {
            assert!(matches!(project_terrain_target(&map,[0,0],screen),Err(TerrainTargetError::ScreenOutsideViewport(_))));
        }
        assert_eq!(map,before);
    }
    #[test]
    fn flat_surface_keeps_cached_vertex_order_and_low_word_wrapping() {
        let map=vec![0;MAP_BYTES];
        for (camera,screen,point,pair,triangle) in [
            ([0,0],[0,239],[1911,1912],7,TerrainTargetTriangle::Second),
            ([66,48],[320,240],[20095,12928],7,TerrainTargetTriangle::Second),
            ([66,48],[319,240],[20092,12932],8,TerrainTargetTriangle::First),
            ([255,255],[639,479],[6132,1020],15,TerrainTargetTriangle::First),
        ] {
            let target=project_terrain_target(&map,camera,screen).unwrap();
            assert_eq!((target.point,target.strip_pair,target.triangle),(point,pair,triangle));
        }
        assert_eq!(project_terrain_target(&map,[0,0],[16,5]),Err(TerrainTargetError::NoIntersection));
    }
}
