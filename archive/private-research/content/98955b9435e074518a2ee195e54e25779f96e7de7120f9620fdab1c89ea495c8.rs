//! Restricted custom-screen transitions from the exact CD candidate.

/// The two flags-0x48 controls bound to CD IFCUSTOM animations 6 and 7.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomArrow {
    Up,
    Down,
}

impl CustomArrow {
    pub const fn event_id(self) -> i16 {
        match self { Self::Up => 27, Self::Down => 28 }
    }
}

/// The capture state of those controls with the screen's initial delay of one.
///
/// This is a packet-level contract from CD manager VA 0xA87D0. It does not
/// identify physical mouse buttons, emulate other control flags, or accept an
/// externally modified countdown. The caller supplies the first opaque hit
/// in original list order, using the actual animation's component pixels.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CustomArrowCapture {
    captured: Option<CustomArrow>,
}

impl CustomArrowCapture {
    pub fn selected(&self, arrow: CustomArrow) -> bool {
        self.captured == Some(arrow)
    }

    /// Return the original signed-word event: -1 no event, -2 newly captured,
    /// or 27/28 activated. A pending capture without matching packet bit1
    /// returns -1, while preserving its selected appearance. Release need
    /// not hit the control again because these controls lack flag 0x1000.
    pub fn update(&mut self, packet_byte8: u8, first_opaque_hit: Option<CustomArrow>) -> i16 {
        if let Some(arrow) = self.captured {
            if packet_byte8 & 0x02 != 0 {
                self.captured = None;
                return arrow.event_id();
            }
            return -1;
        }
        if packet_byte8 & 0x01 != 0 {
            if let Some(arrow) = first_opaque_hit {
                self.captured = Some(arrow);
                return -2;
            }
        }
        -1
    }
}

pub const CUSTOM_LIST_STORAGE_BYTES: usize = 10 * 225;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomListNameError {
    CountExceedsCapacity,
    SelectionOutsideCount,
    StorageTruncated,
    MissingTerminator,
}

/// Read a selected name from the ten-record storage at CD screen state+0x507.
///
/// Initializer 0x7E0B0 initializes/clears ten 225-byte records; maintainer
/// 0x7EF30 bounds the count at ten. Names start at record+0xE. Requiring a NUL
/// before the following field at +0x30 is a conservative safety rule for this
/// port, not a guarantee made by the original unchecked copy. Returned bytes
/// carry no guessed text encoding. Byte-wrapped selection must be validated
/// here before indexing, even when its arithmetic matches the original.
pub fn custom_list_name(storage: &[u8], count: u8, selected: u8) -> Result<&[u8], CustomListNameError> {
    if count > 10 {
        return Err(CustomListNameError::CountExceedsCapacity);
    }
    if selected >= count {
        return Err(CustomListNameError::SelectionOutsideCount);
    }
    if storage.len() < CUSTOM_LIST_STORAGE_BYTES {
        return Err(CustomListNameError::StorageTruncated);
    }
    let record = usize::from(selected) * 225;
    let window = &storage[record + 0xE..record + 0x30];
    let end = window.iter().position(|&byte| byte == 0).ok_or(CustomListNameError::MissingTerminator)?;
    Ok(&window[..end])
}

/// The two offset branches in CD handler VA `0x7E400`.
///
/// Control 27 (branch `0x7E8DC`) decrements state byte +0x16 unless it is zero.
/// Control 28 (`0x7E8EF`) increments modulo 256. There is no item-count clamp in
/// these branches. Unknown controls return None, leaving other handler behavior
/// outside this API. Initialization separately sets the offset byte to zero.
pub fn custom_list_offset_step(offset: u8, control_id: i16) -> Option<u8> {
    match control_id {
        27 => Some(offset.saturating_sub(1)),
        28 => Some(offset.wrapping_add(1)),
        _ => None,
    }
}

/// Selection arithmetic at CD VA `0x7E8F5..0x7E98A`.
///
/// Requires nonzero byte flag (global 0x19CFB6), signed-word control result -1, and signed
/// coordinates within the half-open rectangle [180,454) x [224,296). Returns
/// the resulting state byte +0x19, or None if this branch leaves it unchanged.
/// The original also copies a record's text; that unchecked memory operation is
/// deliberately outside this numeric transition, whose output may exceed the
/// actual number of list records. Callers must validate before indexing data.
pub fn custom_list_selection(offset: u8, control_result: i16, coordinate: [i32; 2], packet_flag: u8) -> Option<u8> {
    if packet_flag == 0 || control_result != -1
        || !(180..454).contains(&coordinate[0]) || !(224..296).contains(&coordinate[1]) {
        return None;
    }
    let row = ((coordinate[1] - 224) / 12) as u8;
    Some(offset.wrapping_add(row))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_names_reject_wrapped_indices_and_terminators_in_following_fields() {
        let mut storage = [0x7f; CUSTOM_LIST_STORAGE_BYTES];
        for slot in 0..10usize {
            let start = slot * 225 + 0xE;
            storage[start] = b'A' + slot as u8;
            storage[start + 1] = 0;
            assert_eq!(custom_list_name(&storage, 10, slot as u8).unwrap(), &[b'A' + slot as u8]);
        }
        assert_eq!(custom_list_name(&storage, 11, 0), Err(CustomListNameError::CountExceedsCapacity));
        for (count, selected) in [(0, 0), (1, 1), (10, 10), (10, 255)] {
            assert_eq!(custom_list_name(&storage, count, selected), Err(CustomListNameError::SelectionOutsideCount));
        }
        assert_eq!(custom_list_name(&storage[..CUSTOM_LIST_STORAGE_BYTES - 1], 10, 9), Err(CustomListNameError::StorageTruncated));
        storage[0xE..0x30].fill(b'X');
        storage[0x30] = 0;
        assert_eq!(custom_list_name(&storage, 1, 0), Err(CustomListNameError::MissingTerminator));
        storage[0x2F] = 0;
        assert_eq!(custom_list_name(&storage, 1, 0).unwrap().len(), 33);
        storage[0xE] = 0;
        assert_eq!(custom_list_name(&storage, 1, 0).unwrap(), b"");
    }

    #[test]
    fn capture_pending_and_outside_release_preserve_exact_sentinels() {
        for arrow in [CustomArrow::Up, CustomArrow::Down] {
            let mut state = CustomArrowCapture::default();
            assert_eq!(state.update(1, Some(arrow)), -2);
            assert!(state.selected(arrow));
            assert_eq!(state.update(0, None), -1);
            assert!(state.selected(arrow));
            assert_eq!(state.update(2, None), arrow.event_id());
            assert!(!state.selected(arrow));
            assert_eq!(state.update(2, None), -1);
        }
    }

    #[test]
    fn all_packet_bits_obey_capture_state_and_mask_eight() {
        for bits in 0..=u8::MAX {
            let mut state = CustomArrowCapture::default();
            assert_eq!(state.update(bits, None), -1);
            assert_eq!(state.update(bits, Some(CustomArrow::Down)), if bits & 1 != 0 { -2 } else { -1 });
            assert_eq!(state.selected(CustomArrow::Down), bits & 1 != 0);

            let mut captured = CustomArrowCapture::default();
            assert_eq!(captured.update(1, Some(CustomArrow::Up)), -2);
            // A different control under the cursor cannot replace capture.
            assert_eq!(captured.update(bits, Some(CustomArrow::Down)), if bits & 2 != 0 { 27 } else { -1 });
            assert_eq!(captured.selected(CustomArrow::Up), bits & 2 == 0);
            assert!(!captured.selected(CustomArrow::Down));
        }
    }

    #[test]
    fn all_offsets_match_byte_width_and_zero_guard() {
        for offset in 0..=u8::MAX {
            assert_eq!(custom_list_offset_step(offset, 27), Some(if offset == 0 { 0 } else { offset - 1 }));
            assert_eq!(custom_list_offset_step(offset, 28), Some(((u16::from(offset) + 1) % 256) as u8));
            assert_eq!(custom_list_offset_step(offset, -1), None);
            assert_eq!(custom_list_offset_step(offset, 29), None);
        }
    }

    #[test]
    fn selection_has_half_open_edges_and_requires_the_observed_packet_conditions() {
        assert_eq!(custom_list_selection(0, -1, [180, 224], 1), Some(0));
        assert_eq!(custom_list_selection(0, -1, [453, 295], u8::MAX), Some(5));
        assert_eq!(custom_list_selection(255, -1, [200, 295], 1), Some(4));
        for point in [[179, 224], [454, 224], [180, 223], [180, 296], [i32::MIN, i32::MIN], [i32::MAX, i32::MAX]] {
            assert_eq!(custom_list_selection(0, -1, point, 1), None);
        }
        assert_eq!(custom_list_selection(0, -1, [180, 224], 0), None);
        assert_eq!(custom_list_selection(0, 27, [180, 224], 1), None);
        assert_eq!(custom_list_selection(0, -2, [180, 224], 1), None);
    }

    #[test]
    fn each_of_six_rows_uses_twelve_scanlines_and_wraps_after_255() {
        for offset in [0, 250, 255] {
            for row in 0..6 {
                for scanline in 0..12 {
                    assert_eq!(custom_list_selection(offset, -1, [200, 224 + row * 12 + scanline], 1),
                               Some(((u16::from(offset) + row as u16) % 256) as u8));
                }
            }
        }
    }
}
