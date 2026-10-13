//! Restricted mode-two button transition from CD producer VA 0x70E40.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonPacket {
    pub pressed: bool,
    pub released: bool,
    pub held: bool,
}

/// Produce one button's next packet from the previous packet-held bit, the
/// asynchronous press latch, and current physical-held state. The caller
/// consumes the press latch after this update. This function does not choose
/// a polling cadence or replace the original mouse-coordinate integration.
///
/// A quick press/release between polls still yields a press packet, followed
/// by a release on the next poll. A new latched press while packet-held was
/// already set yields both event bits. This follows the original priority,
/// rather than deriving events only from differences in held-state samples.
pub fn mode2_button_packet(previous_packet_held: bool, press_latched: bool, physical_held: bool) -> ButtonPacket {
    if press_latched {
        ButtonPacket { pressed: true, released: previous_packet_held, held: true }
    } else if physical_held {
        ButtonPacket { pressed: false, released: false, held: true }
    } else {
        ButtonPacket { pressed: false, released: previous_packet_held, held: false }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instruction_truth_table_covers_every_input_combination() {
        let cases = [
            ((false, false, false), (false, false, false)),
            ((false, false, true), (false, false, true)),
            ((false, true, false), (true, false, true)),
            ((false, true, true), (true, false, true)),
            ((true, false, false), (false, true, false)),
            ((true, false, true), (false, false, true)),
            ((true, true, false), (true, true, true)),
            ((true, true, true), (true, true, true)),
        ];
        for ((old, latch, current), (pressed, released, held)) in cases {
            assert_eq!(mode2_button_packet(old, latch, current), ButtonPacket { pressed, released, held });
        }
    }

    #[test]
    fn quick_click_survives_two_polls_and_repress_emits_both_events() {
        let press = mode2_button_packet(false, true, false);
        assert_eq!(press, ButtonPacket { pressed: true, released: false, held: true });
        let release = mode2_button_packet(press.held, false, false);
        assert_eq!(release, ButtonPacket { pressed: false, released: true, held: false });
        assert_eq!(mode2_button_packet(release.held, false, false), ButtonPacket { pressed: false, released: false, held: false });
        assert_eq!(mode2_button_packet(true, true, true), ButtonPacket { pressed: true, released: true, held: true });
    }
}
