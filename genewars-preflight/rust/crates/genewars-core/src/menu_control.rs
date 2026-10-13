//! Restricted callback-0 transitions for the CD main-menu `0x1048` controls.
//!
//! This module does not perform hit testing. The caller supplies the first
//! enabled opaque normal-frame hit while uncaptured, and the opaque hit result
//! for the captured control while captured. The instruction-derived callback
//! vectors are summarized in `research/ghidra/menu-input-bridge.md` and
//! `research/ghidra/menu-input-evidence.json`.

/// The four controls handled by the traced main-menu callback-0 branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuControl {
    Id0,
    Id1,
    Id3,
    Id2,
}

impl MenuControl {
    pub const fn event_id(self) -> i16 {
        match self {
            Self::Id0 => 0,
            Self::Id1 => 1,
            Self::Id3 => 3,
            Self::Id2 => 2,
        }
    }

    /// Manager traversal order established by the static control list.
    pub const fn hit_order() -> [Self; 4] {
        [Self::Id0, Self::Id1, Self::Id3, Self::Id2]
    }
}

/// The packet bytes consumed by the restricted callback-0 branch.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MenuPacket {
    pub byte8: u8,
    pub byte9: u8,
}

/// Output of one callback-0 update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuCallbackResult {
    /// Signed event ID returned by the manager, or -1 when no transition occurs.
    pub event_id: i16,
    /// A write to the shared input-mask global. `None` means this branch leaves
    /// its previous value untouched.
    pub input_mask_write: Option<u8>,
    pub captured: Option<MenuControl>,
    pub selected: Option<MenuControl>,
    pub countdown: u16,
    /// Capture-mask byte for the active control record, if any.
    pub capture_mask: u8,
}

/// State for the proven callback-0 capture path with initial countdown one.
/// It does not model the outer menu handler or other control flag combinations.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MenuCallback0State {
    captured: Option<MenuControl>,
    selected: Option<MenuControl>,
    countdown: u16,
}

impl MenuCallback0State {
    pub fn captured(&self) -> Option<MenuControl> {
        self.captured
    }

    pub fn selected(&self) -> Option<MenuControl> {
        self.selected
    }

    pub fn countdown(&self) -> u16 {
        self.countdown
    }

    /// Update callback-0 state using a caller-supplied opaque normal-frame hit.
    ///
    /// When uncaptured, `first_enabled_hit` must already be the first matching
    /// control in manager order `[0, 1, 3, 2]`. When captured, only
    /// `captured_control_hit` is consulted; it must test that control's normal
    /// frame footprint, including its transparent pixels.
    pub fn update(
        &mut self,
        packet: MenuPacket,
        first_enabled_hit: Option<MenuControl>,
        captured_control_hit: bool,
    ) -> MenuCallbackResult {
        if let Some(control) = self.captured {
            return self.update_captured(packet, control, captured_control_hit);
        }

        // The uncaptured branch captures on press, even when a release event is
        // also present in this packet. The returned ID is not an activation:
        // this branch writes input mask zero.
        if packet.byte8 & 0x01 != 0 {
            if let Some(control) = first_enabled_hit {
                self.captured = Some(control);
                self.selected = Some(control);
                self.countdown = 1;
                return self.result(control.event_id(), Some(0));
            }
        }

        self.result(-1, None)
    }

    fn update_captured(
        &mut self,
        packet: MenuPacket,
        control: MenuControl,
        hit: bool,
    ) -> MenuCallbackResult {
        // This port admits only the traced initial countdown state, which can
        // subsequently be zero or one while capture remains pending.
        debug_assert!(self.countdown <= 1);

        if packet.byte8 & 0x02 != 0 {
            let event_id = if hit { control.event_id() } else { -1 };
            self.captured = None;
            self.selected = None;
            // The original clears capture/draw state but does not reset the
            // countdown here; retain its last value after capture ends.
            return self.result(event_id, Some(8));
        }

        let was_selected = self.selected == Some(control);
        let mut event_id = -1;
        let mut input_mask_write = Some(
            (if packet.byte8 & 0x04 != 0 { 2 } else { 0 })
                | (if packet.byte9 & 0x01 != 0 { 4 } else { 0 }),
        );
        if hit != was_selected {
            event_id = control.event_id();
            if was_selected {
                // Leaving the normal-frame footprint overwrites held-button
                // mask with one; this is a distinct observed transition.
                input_mask_write = Some(1);
            } else {
                // Entering retains the packet's current held-button mask.
            }
        }

        self.selected = if hit { Some(control) } else { None };
        self.countdown = if hit { 1 } else { 0 };
        self.result(event_id, input_mask_write)
    }

    fn result(&self, event_id: i16, input_mask_write: Option<u8>) -> MenuCallbackResult {
        MenuCallbackResult {
            event_id,
            input_mask_write,
            captured: self.captured,
            selected: self.selected,
            countdown: self.countdown,
            capture_mask: if self.captured.is_some() { 8 } else { 0 },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press() -> MenuPacket {
        MenuPacket { byte8: 0x01, byte9: 0 }
    }

    fn release() -> MenuPacket {
        MenuPacket { byte8: 0x02, byte9: 0 }
    }

    #[test]
    fn menu_control_order_is_the_observed_manager_traversal() {
        assert_eq!(
            MenuControl::hit_order().map(MenuControl::event_id),
            [0, 1, 3, 2]
        );
    }

    #[test]
    fn hover_and_uncaptured_no_hit_do_not_change_state_or_input_mask() {
        let mut state = MenuCallback0State::default();
        let hover = state.update(MenuPacket::default(), Some(MenuControl::Id1), false);
        assert_eq!(hover.event_id, -1);
        assert_eq!(hover.input_mask_write, None);
        assert_eq!(hover.captured, None);
        assert_eq!(hover.selected, None);

        let miss = state.update(press(), None, false);
        assert_eq!(miss.event_id, -1);
        assert_eq!(miss.input_mask_write, None);
        assert_eq!(miss.countdown, 0);
    }

    #[test]
    fn uncaptured_press_returns_id_but_writes_zero_mask_and_selects_control() {
        for control in MenuControl::hit_order() {
            let mut state = MenuCallback0State::default();
            let result = state.update(press(), Some(control), false);
            assert_eq!(result.event_id, control.event_id());
            assert_eq!(result.input_mask_write, Some(0));
            assert_eq!(result.captured, Some(control));
            assert_eq!(result.selected, Some(control));
            assert_eq!(result.countdown, 1);
            assert_eq!(result.capture_mask, 8);
        }
    }

    #[test]
    fn simultaneous_press_and_release_captures_without_activation() {
        let mut state = MenuCallback0State::default();
        let result = state.update(
            MenuPacket { byte8: 0x03, byte9: 0 },
            Some(MenuControl::Id3),
            false,
        );
        assert_eq!(result.event_id, 3);
        assert_eq!(result.input_mask_write, Some(0));
        assert_eq!(result.captured, Some(MenuControl::Id3));
        assert_eq!(result.selected, Some(MenuControl::Id3));
    }

    #[test]
    fn captured_release_activates_only_inside_normal_opaque_footprint() {
        for (hit, expected_id) in [(true, 2), (false, -1)] {
            let mut state = MenuCallback0State::default();
            state.update(press(), Some(MenuControl::Id2), false);
            let result = state.update(release(), None, hit);
            assert_eq!(result.event_id, expected_id);
            assert_eq!(result.input_mask_write, Some(8));
            assert_eq!(result.captured, None);
            assert_eq!(result.selected, None);
            assert_eq!(result.countdown, 1);
            assert_eq!(result.capture_mask, 0);
        }
    }

    #[test]
    fn release_after_leave_accepts_zero_countdown_and_preserves_it_after_clear() {
        let mut state = MenuCallback0State::default();
        state.update(press(), Some(MenuControl::Id3), false);
        let left = state.update(MenuPacket::default(), None, false);
        assert_eq!(left.countdown, 0);
        assert_eq!(left.captured, Some(MenuControl::Id3));
        assert_eq!(left.selected, None);

        let released_outside = state.update(release(), None, false);
        assert_eq!(released_outside.event_id, -1);
        assert_eq!(released_outside.input_mask_write, Some(8));
        assert_eq!(released_outside.captured, None);
        assert_eq!(released_outside.selected, None);
        assert_eq!(released_outside.countdown, 0);
    }

    #[test]
    fn captured_hover_transitions_write_exact_masks_and_preserve_capture() {
        let mut state = MenuCallback0State::default();
        state.update(press(), Some(MenuControl::Id0), false);

        let inside_unchanged = state.update(
            MenuPacket { byte8: 0x04, byte9: 0x01 },
            Some(MenuControl::Id1),
            true,
        );
        assert_eq!(inside_unchanged.event_id, -1);
        assert_eq!(inside_unchanged.input_mask_write, Some(6));
        assert_eq!(inside_unchanged.selected, Some(MenuControl::Id0));
        assert_eq!(inside_unchanged.countdown, 1);

        let leaves = state.update(MenuPacket { byte8: 0x04, byte9: 0x01 }, None, false);
        assert_eq!(leaves.event_id, 0);
        assert_eq!(leaves.input_mask_write, Some(1));
        assert_eq!(leaves.selected, None);
        assert_eq!(leaves.countdown, 0);
        assert_eq!(leaves.captured, Some(MenuControl::Id0));

        let enters = state.update(MenuPacket { byte8: 0x04, byte9: 0x01 }, None, true);
        assert_eq!(enters.event_id, 0);
        assert_eq!(enters.input_mask_write, Some(6));
        assert_eq!(enters.selected, Some(MenuControl::Id0));
        assert_eq!(enters.countdown, 1);
        assert_eq!(enters.captured, Some(MenuControl::Id0));
    }

    #[test]
    fn captured_press_is_ignored_unless_a_release_event_is_also_present() {
        let mut state = MenuCallback0State::default();
        state.update(press(), Some(MenuControl::Id1), false);
        let pressed_again = state.update(press(), Some(MenuControl::Id3), true);
        assert_eq!(pressed_again.event_id, -1);
        assert_eq!(pressed_again.input_mask_write, Some(0));
        assert_eq!(pressed_again.captured, Some(MenuControl::Id1));

        let release_and_press = state.update(
            MenuPacket { byte8: 0x03, byte9: 0 },
            Some(MenuControl::Id3),
            true,
        );
        assert_eq!(release_and_press.event_id, 1);
        assert_eq!(release_and_press.input_mask_write, Some(8));
        assert_eq!(release_and_press.captured, None);
    }
}
