//! Draft host-side adapter for ordered winit mouse events.
//!
//! Kept under ignored private/ while the parent runs the workspace checkpoint.
//! This module has no backend or core dependency yet. A future tracked version
//! should convert physical cursor coordinates into the inspector's chosen
//! client-area coordinate system before creating a packet.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CursorPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ButtonSnapshot {
    pub physical_held: bool,
    pub press_latched: bool,
    pub first_press_position: Option<CursorPoint>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PressLatch {
    /// None means a down transition arrived before any cursor position.
    pub position: Option<CursorPoint>,
}

/// Retains the first press until one packet-production step consumes it.
#[derive(Clone, Copy, Debug, Default)]
pub struct LeftButtonLatch {
    cursor: Option<CursorPoint>,
    physical_held: bool,
    press_latch: Option<PressLatch>,
}

impl LeftButtonLatch {
    pub fn cursor_moved(&mut self, position: CursorPoint) {
        self.cursor = Some(position);
    }

    /// Apply one ordered left-button transition. Repeated down events while
    /// held do not replace the original press latch or its coordinates.
    pub fn button_transition(&mut self, pressed: bool) {
        if pressed {
            if !self.physical_held {
                if self.press_latch.is_none() {
                    self.press_latch = Some(PressLatch { position: self.cursor });
                }
                self.physical_held = true;
            }
        } else {
            self.physical_held = false;
        }
    }

    pub fn snapshot(&self) -> ButtonSnapshot {
        ButtonSnapshot {
            physical_held: self.physical_held,
            press_latched: self.press_latch.is_some(),
            first_press_position: self.press_latch.and_then(|press| press.position),
        }
    }

    /// Consume the pending press after producing one packet. Physical-held
    /// state is independent and survives this operation.
    pub fn consume_press_latch(&mut self) -> Option<PressLatch> {
        self.press_latch.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(x: f64, y: f64) -> CursorPoint {
        CursorPoint { x, y }
    }

    #[test]
    fn quick_press_and_release_before_packet_keeps_press_and_coords() {
        let mut input = LeftButtonLatch::default();
        input.cursor_moved(point(455.0, 229.0));
        input.button_transition(true);
        input.button_transition(false);

        assert_eq!(input.snapshot(), ButtonSnapshot {
            physical_held: false,
            press_latched: true,
            first_press_position: Some(point(455.0, 229.0)),
        });
        assert_eq!(input.consume_press_latch(), Some(PressLatch { position: Some(point(455.0, 229.0)) }));
        assert_eq!(input.snapshot(), ButtonSnapshot::default());
    }

    #[test]
    fn first_press_coordinates_survive_repress_until_consumed() {
        let mut input = LeftButtonLatch::default();
        input.cursor_moved(point(10.0, 20.0));
        input.button_transition(true);
        input.button_transition(false);
        input.cursor_moved(point(30.0, 40.0));
        input.button_transition(true);

        let state = input.snapshot();
        assert!(state.physical_held);
        assert_eq!(state.first_press_position, Some(point(10.0, 20.0)));
        assert_eq!(input.consume_press_latch(), Some(PressLatch { position: Some(point(10.0, 20.0)) }));
        assert!(input.snapshot().physical_held);
    }

    #[test]
    fn down_without_cursor_position_latches_no_coordinate() {
        let mut input = LeftButtonLatch::default();
        input.button_transition(true);

        let state = input.snapshot();
        assert!(state.physical_held);
        assert!(state.press_latched);
        assert_eq!(state.first_press_position, None);
        assert_eq!(input.consume_press_latch(), Some(PressLatch { position: None }));
    }

    #[test]
    fn duplicate_down_does_not_create_a_second_press_edge() {
        let mut input = LeftButtonLatch::default();
        input.cursor_moved(point(3.0, 4.0));
        input.button_transition(true);
        input.cursor_moved(point(5.0, 6.0));
        input.button_transition(true);

        assert_eq!(input.snapshot().first_press_position, Some(point(3.0, 4.0)));
        assert_eq!(input.consume_press_latch(), Some(PressLatch { position: Some(point(3.0, 4.0)) }));
        assert_eq!(input.consume_press_latch(), None);
    }
}
