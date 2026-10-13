//! Original portable Rust reconstructions of explicitly identified binary behavior.
//! This crate does not yet implement a game simulation.

mod opaque_records;
mod custom_list;
mod mouse_packet;
mod construction;

/// Helpers whose evidence is restricted to the supplied CD analysis candidate.
pub mod cd_candidate {
    pub use crate::construction::{construction_stage3_debit, ConstructionSelector,
        InvalidConstructionSelector, ConstructionPools, ConstructionDebit, ConstructionDebitStatus};
    pub use crate::custom_list::{custom_list_offset_step, custom_list_selection, custom_list_name,
        CustomArrow, CustomArrowCapture, CustomListNameError, CUSTOM_LIST_STORAGE_BYTES};
    pub use crate::mouse_packet::{mode2_button_packet, ButtonPacket};
    pub use crate::opaque_records::{
        BoundsError, record_radius_match, selected_group_radius_match,
        set_grid_bit10, clear_grid_bit10, RECORD_GROUP_BYTES,
    };
    /// SHA-256 of the exact input binary; not an assertion of canonical retail identity.
    pub const BINARY_SHA256: &str =
        "4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba";

    /// Squared distance reconstructed from CD function VA `0x73690`.
    ///
    /// Each coordinate difference is reduced to [-16384, 16383], then squared
    /// and summed. Static instruction evidence uses 16-bit doubling/subtraction,
    /// signed 16-bit arithmetic shift, sign extension and 32-bit multiplication.
    /// Inputs are two signed coordinate components; their gameplay units are not
    /// established. This must not be replaced with ordinary Euclidean subtraction.
    ///
    /// Evidence: object 1 + `0x63690`, 95 bytes, no calls or global accesses.
    /// No original game execution was used to establish these semantics.
    pub fn distance_squared_wrapping_15(source: [i16; 2], target: [i16; 2]) -> u32 {
        fn delta(from: i16, to: i16) -> i32 {
            (i32::from(to) - i32::from(from) + 16384).rem_euclid(32768) - 16384
        }
        let dx = delta(source[0], target[0]);
        let dy = delta(source[1], target[1]);
        (dx * dx + dy * dy) as u32
    }

    /// Floor square root for every `u32`, matching the result semantics of CD
    /// function VA `0x97910`. Uses an independently chosen Newton upper bound;
    /// no proprietary lookup-table bytes are embedded. Iteration timing is not
    /// claimed equivalent to the original helper.
    pub fn integer_sqrt(value: u32) -> u32 {
        if value == 0 {
            return 0;
        }
        let bits = u32::BITS - value.leading_zeros();
        let mut estimate = 1u32 << ((bits + 1) / 2);
        loop {
            let quotient = value / estimate;
            if quotient >= estimate {
                return estimate;
            }
            estimate = (estimate + quotient) / 2;
        }
    }

    /// Integer distance helper VA `0x737F0`: wrapped squared distance followed
    /// by the unsigned floor-square-root routine. Gameplay units remain unknown.
    pub fn distance_wrapping_15(source: [i16; 2], target: [i16; 2]) -> u32 {
        integer_sqrt(distance_squared_wrapping_15(source, target))
    }
}

#[cfg(test)]
mod tests {
    use super::cd_candidate::distance_squared_wrapping_15 as distance;

    #[test]
    fn static_instruction_vectors() {
        for (source, target, expected) in [
            ([0, 0], [0, 0], 0),
            ([0, 0], [3, 4], 25),
            ([0, 0], [32767, 0], 1),
            ([0, 0], [16384, 0], 268435456),
            ([0, 0], [32767, 32767], 2),
            ([0, 0], [16384, 16384], 536870912),
            ([0, 0], [-32768, -32768], 0),
            ([-32768, 32767], [32767, -32768], 2),
        ] {
            assert_eq!(distance(source, target), expected);
        }
    }

    #[test]
    fn symmetry_and_bound_hold_at_signed_and_wrap_boundaries() {
        let edges = [-32768, -32767, -16385, -16384, -1, 0, 1, 16383, 16384, 32766, 32767];
        for a in edges {
            for b in edges {
                for c in edges {
                    let source = [a, b];
                    let target = [c, a];
                    assert_eq!(distance(source, target), distance(target, source));
                    assert!(distance(source, target) <= 536870912);
                }
            }
        }
    }

    #[test]
    fn coordinates_congruent_modulo_32768_have_identical_distance() {
        for x in [-32768i16, -20000, -1, 0, 16384, 32767] {
            for y in [-32768i16, -1, 0, 16384, 32767] {
                let shifted = x.wrapping_add(i16::MIN);
                assert_eq!(distance([x, y], [123, -456]), distance([shifted, y], [123, -456]));
            }
        }
    }

    #[test]
    fn square_root_edges_and_perfect_square_neighbors() {
        use super::cd_candidate::integer_sqrt;
        assert_eq!(integer_sqrt(0), 0);
        assert_eq!(integer_sqrt(u32::MAX), 65535);
        for n in 1u32..=65535 {
            let square = n * n;
            assert_eq!(integer_sqrt(square), n);
            assert_eq!(integer_sqrt(square - 1), n - 1);
            assert_eq!(integer_sqrt(square + 1), n);
        }
    }

    #[test]
    fn combined_distance_applies_floor_after_wrapping() {
        use super::cd_candidate::distance_wrapping_15;
        assert_eq!(distance_wrapping_15([0, 0], [3, 4]), 5);
        assert_eq!(distance_wrapping_15([0, 0], [32767, 32767]), 1);
        assert_eq!(distance_wrapping_15([0, 0], [16384, 16384]), 23170);
    }
}
