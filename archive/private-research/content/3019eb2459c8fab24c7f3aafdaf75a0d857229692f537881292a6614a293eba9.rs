//! Observed byte layouts with intentionally unnamed gameplay semantics.
use crate::cd_candidate::distance_squared_wrapping_15;

pub const RECORD_GROUP_BYTES: usize = 64 * 67;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundsError {
    pub required: usize,
    pub available: usize,
}

impl core::fmt::Display for BoundsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "requires {} bytes, received {}", self.required, self.available)
    }
}

impl std::error::Error for BoundsError {}

/// CD VA `0x432C0`: scan exactly 64 opaque 67-byte records for a radius match.
///
/// Validates a full group before scanning; extra bytes are ignored. This safety
/// check is additional to the original pointer-based routine. Qualification
/// offsets and arithmetic come from direct instructions; record meaning and
/// valid gameplay radius ranges are unknown. Assumes immutable source memory.
pub fn record_radius_match(records: &[u8], query: [i16; 2]) -> Result<bool, BoundsError> {
    let group = records.get(..RECORD_GROUP_BYTES).ok_or(BoundsError {
        required: RECORD_GROUP_BYTES, available: records.len(),
    })?;
    for record in group.chunks_exact(67) {
        if record[6] == 0 || record[0x31] & 1 == 0 || record[0x2d] != 3 {
            continue;
        }
        let coordinate = [i16::from_le_bytes([record[7], record[8]]),
                          i16::from_le_bytes([record[9], record[10]])];
        let radius = 1536 + 768 * u32::from(record[0x2e]);
        if distance_squared_wrapping_15(query, coordinate) <= radius.wrapping_mul(radius) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Select a complete group without assuming the original allocation's group count.
pub fn selected_group_radius_match(groups: &[u8], selector: u8, query: [i16; 2]) -> Result<bool, BoundsError> {
    let start = usize::from(selector) * RECORD_GROUP_BYTES;
    let end = start + RECORD_GROUP_BYTES;
    let selected = groups.get(start..end).ok_or(BoundsError { required: end, available: groups.len() })?;
    record_radius_match(selected, query)
}

fn offsets(coordinate: [i16; 2], columns: u8, rows: u8) -> impl Iterator<Item = usize> {
    let x = usize::from((coordinate[0] as u16 >> 8) & 127);
    (0..rows).flat_map(move |row| {
        let y = usize::from(((coordinate[1] as u16).wrapping_add(u16::from(row) << 8) >> 8) & 127);
        (0..columns).map(move |column| (y * 128 + x + usize::from(column)) * 14 + 11)
    })
}

fn change_grid_bit10(grid: &mut [u8], coordinate: [i16; 2], columns: u8, rows: u8, set: bool) -> Result<(), BoundsError> {
    // Validate the entire span before any write, including linear X spillover.
    if let Some(maximum) = offsets(coordinate, columns, rows).max() {
        if maximum >= grid.len() {
            return Err(BoundsError { required: maximum + 1, available: grid.len() });
        }
    }
    for offset in offsets(coordinate, columns, rows) {
        if set { grid[offset] |= 0x10; } else { grid[offset] &= !0x10; }
    }
    Ok(())
}

/// CD VA `0x8D580`: set bit 0x10 at byte 11 of addressed 14-byte records.
/// Y wraps at 128 rows; columns advance linearly. Bounds rejection is atomic.
/// Only touched bytes must exist; zero counts require no backing bytes.
pub fn set_grid_bit10(grid: &mut [u8], coordinate: [i16; 2], columns: u8, rows: u8) -> Result<(), BoundsError> {
    change_grid_bit10(grid, coordinate, columns, rows, true)
}

/// CD VA `0x8D5F0`: clear the same bit, including the shared preceding epilogue.
pub fn clear_grid_bit10(grid: &mut [u8], coordinate: [i16; 2], columns: u8, rows: u8) -> Result<(), BoundsError> {
    change_grid_bit10(grid, coordinate, columns, rows, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn qualifying_record(group: &mut [u8], index: usize, point: [i16; 2], radius: u8) {
        let record = &mut group[index * 67..(index + 1) * 67];
        record[6] = 255;
        record[7..9].copy_from_slice(&point[0].to_le_bytes());
        record[9..11].copy_from_slice(&point[1].to_le_bytes());
        record[0x2d] = 3;
        record[0x2e] = radius;
        record[0x31] = 1;
    }

    #[test]
    fn radius_preserves_wrap_unsigned_comparison_and_inclusive_boundary() {
        for (point, radius, expected) in [
            ([1536, 0], 0, true), ([1537, 0], 0, false),
            ([-1536, 0], 0, true), ([-32768, -32768], 0, true),
            ([16384, 16384], 83, true), ([16384, 16384], 84, false),
            ([0, 0], 254, true), ([1, 0], 254, false),
        ] {
            let mut group = [0u8; RECORD_GROUP_BYTES];
            qualifying_record(&mut group, 63, point, radius);
            assert_eq!(record_radius_match(&group, [0, 0]), Ok(expected));
        }
    }

    #[test]
    fn radius_requires_complete_group_and_exact_qualification_flags() {
        let mut group = [0u8; RECORD_GROUP_BYTES];
        assert_eq!(record_radius_match(&group, [0, 0]), Ok(false));
        qualifying_record(&mut group, 0, [0, 0], 0);
        assert!(record_radius_match(&group[..RECORD_GROUP_BYTES - 1], [0, 0]).is_err());
        for (offset, value) in [(6, 0), (0x2d, 2), (0x31, 0xfe)] {
            let old = group[offset];
            group[offset] = value;
            group[0x32] = 255;
            assert_eq!(record_radius_match(&group, [0, 0]), Ok(false));
            group[offset] = old;
        }
        assert_eq!(record_radius_match(&group, [0, 0]), Ok(true));
        assert!(selected_group_radius_match(&group, 1, [0, 0]).is_err());
        let mut groups = vec![0; RECORD_GROUP_BYTES * 2];
        groups[RECORD_GROUP_BYTES..].copy_from_slice(&group);
        assert_eq!(selected_group_radius_match(&groups, 0, [0, 0]), Ok(false));
        assert_eq!(selected_group_radius_match(&groups, 1, [0, 0]), Ok(true));
    }

    #[test]
    fn grid_preserves_linear_columns_wrapped_rows_and_unrelated_bytes() {
        let mut grid = vec![0xa5; 128 * 128 * 14];
        set_grid_bit10(&mut grid, [32512, 0], 2, 1).unwrap();
        assert_eq!(grid[1789], 0xb5);
        assert_eq!(grid[1803], 0xb5);
        assert_eq!(grid.iter().filter(|&&b| b != 0xa5).count(), 2);
        clear_grid_bit10(&mut grid, [32512, 0], 2, 1).unwrap();
        assert!(grid.iter().all(|&b| b == 0xa5));
        set_grid_bit10(&mut grid, [0, 32512], 1, 2).unwrap();
        assert_eq!(grid[227595], 0xb5);
        assert_eq!(grid[11], 0xb5);
        assert_eq!(grid.iter().filter(|&&b| b != 0xa5).count(), 2);
    }

    #[test]
    fn grid_bounds_failure_has_no_partial_writes_and_zero_counts_need_no_memory() {
        let mut grid = vec![0xa5; 128 * 128 * 14];
        let before = grid.clone();
        assert_eq!(set_grid_bit10(&mut grid, [-1, -1], 2, 1), Err(BoundsError { required: 229388, available: 229376 }));
        assert_eq!(grid, before);
        set_grid_bit10(&mut [], [-1, -1], 0, 255).unwrap();
        clear_grid_bit10(&mut [], [-1, -1], 255, 0).unwrap();
        grid.resize(232930, 0xa5);
        set_grid_bit10(&mut grid, [-1, -1], 255, 1).unwrap();
        assert_eq!(grid[232929], 0xb5);
        assert_eq!(grid.iter().filter(|&&b| b != 0xa5).count(), 255);
    }

    #[test]
    fn repeated_grid_rows_are_idempotent() {
        let mut once = vec![0; 128 * 128 * 14];
        let mut repeated = once.clone();
        set_grid_bit10(&mut once, [0, 0], 1, 128).unwrap();
        set_grid_bit10(&mut repeated, [0, 0], 1, 255).unwrap();
        assert_eq!(once, repeated);
        clear_grid_bit10(&mut repeated, [0, 0], 1, 255).unwrap();
        assert!(repeated.iter().all(|&b| b == 0));
    }
}
