//! Explicit-input family-1 queue and packet helpers for the pinned CD candidate.
//!
//! These helpers do not scan mission source tables, construct runtime objects,
//! run the dirty prepass, or infer first-scene state. Candidate order, projected
//! coordinates, flags, and context bytes are explicit caller inputs.

use core::fmt;

pub const DISPLAY_PITCH: i32 = 640;
pub const DISPLAY_HEIGHT: i32 = 480;
pub const FAMILY1_QUEUE_CAPACITY: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectedFamily1 {
    /// Caller-owned identity token; it is not interpreted as a source index.
    pub id: u32,
    pub x: i32,
    pub y: i32,
    /// Raw runtime byte +0x17, supplied explicitly for the optional packet.
    pub shade: u8,
    /// Raw runtime byte +0x15, supplied explicitly for packet helpers.
    pub flags: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueuedFamily1 {
    pub id: u32,
    pub x: i16,
    pub draw_y: i16,
    pub projected_y: i32,
    pub bucket: u16,
    pub residual: i16,
    pub shade: u8,
    pub flags: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Family1Queue {
    /// Entries in the exact traversal order used by the traced bucket/chain walk.
    pub draw_order: Vec<QueuedFamily1>,
    /// Includes an explicit pre-existing count, which can represent other
    /// families already using the shared 256-entry capacity.
    pub final_count: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueError {
    InvalidInitialCount(i16),
    InvalidMargin(u16),
    AllocationFailed,
}

impl fmt::Display for QueueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{self:?}") }
}

/// Insert an explicit sequence of already-projected family-1 candidates.
///
/// `active_module` stands for the caller-provided comparison against the
/// module pointer at `0x116974`. `initial_count` is the already occupied
/// shared queue count; safe callers must provide 0..=256. The original signed
/// check only rejects values >=256, but negative counts are excluded here.
/// The candidate sequence is never sorted by ID or source-table position.
pub fn queue_family1(
    active_module: bool,
    initial_count: i16,
    margin: u16,
    candidates: &[ProjectedFamily1],
) -> Result<Family1Queue, QueueError> {
    if !(0..=FAMILY1_QUEUE_CAPACITY as i16).contains(&initial_count) {
        return Err(QueueError::InvalidInitialCount(initial_count));
    }
    if margin != 0 && margin != 128 {
        return Err(QueueError::InvalidMargin(margin));
    }

    let remaining = FAMILY1_QUEUE_CAPACITY - initial_count as usize;
    let mut buckets: Vec<Vec<QueuedFamily1>> = Vec::new();
    buckets.try_reserve_exact(DISPLAY_HEIGHT as usize).map_err(|_| QueueError::AllocationFailed)?;
    for _ in 0..DISPLAY_HEIGHT {
        buckets.push(Vec::new());
    }
    let mut accepted_count = 0usize;

    if active_module {
        for candidate in candidates {
            if accepted_count >= remaining { break; }
            if candidate.x < -40 || candidate.x >= DISPLAY_PITCH - i32::from(margin) + 40
                || candidate.y < -10 || candidate.y >= DISPLAY_HEIGHT + 40
            {
                continue;
            }
            // The admission rectangle proves x and the reconstructed draw y
            // fit signed16, but keep the conversion checked at this boundary.
            let (bucket, residual) = if candidate.y <= 0 {
                (0usize, candidate.y)
            } else if candidate.y < DISPLAY_HEIGHT {
                (candidate.y as usize, 0)
            } else {
                ((DISPLAY_HEIGHT - 1) as usize, candidate.y - DISPLAY_HEIGHT)
            };
            let queued = QueuedFamily1 {
                id: candidate.id,
                x: i16::try_from(candidate.x).expect("admission bounds fit signed16 x"),
                draw_y: i16::try_from(bucket as i32 + residual).expect("admission bounds fit signed16 y"),
                projected_y: candidate.y,
                bucket: bucket as u16,
                residual: i16::try_from(residual).expect("admission bounds fit signed16 residual"),
                shade: candidate.shade,
                flags: candidate.flags,
            };
            let entries = &mut buckets[bucket];
            if candidate.y > 0 && candidate.y < DISPLAY_HEIGHT {
                // 0x83140 inserts at the head for ordinary on-screen buckets:
                // equal-row admission order is reversed.
                entries.try_reserve(1).map_err(|_| QueueError::AllocationFailed)?;
                entries.insert(0, queued);
            } else {
                // Top and bottom special buckets are signed-residual ascending,
                // stable for equal residuals (new equal entries append).
                let position = entries.partition_point(|entry| entry.residual <= queued.residual);
                entries.try_reserve(1).map_err(|_| QueueError::AllocationFailed)?;
                entries.insert(position, queued);
            }
            accepted_count += 1;
        }
    }

    let mut draw_order = Vec::new();
    draw_order.try_reserve_exact(accepted_count).map_err(|_| QueueError::AllocationFailed)?;
    for bucket in buckets {
        draw_order.extend(bucket);
    }
    Ok(Family1Queue {
        draw_order,
        final_count: (initial_count as usize + accepted_count) as u16,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrawPacket {
    pub draws: bool,
    pub dirty_class: Option<u8>,
    pub flags_after: u8,
}

/// Apply the isolated `0x83930` family-1 draw decision for selected=None.
/// The family-1 virtual predicate is fixed true; selection/flash handling is
/// outside this helper. A skipped candidate retains its flags unchanged.
pub fn family1_draw_packet(context_gate: bool, mode: u8, flags: u8) -> DrawPacket {
    if context_gate && (mode != 0 || flags & 0x08 != 0) {
        DrawPacket {
            draws: true,
            dirty_class: Some(if flags & 0x04 != 0 { 3 } else { 1 }),
            flags_after: flags & 0xF3,
        }
    } else {
        DrawPacket { draws: false, dirty_class: None, flags_after: flags }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmissionPacketInput {
    pub native_type_at_6: u8,
    pub context_flags_at_1c: u8,
    pub context_gate_before: bool,
    pub shade_at_17: u8,
    pub countdown_at_1d6: i8,
    pub flags_at_15: u8,
    pub old_frame_present: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmissionPacket {
    pub context_gate_after: bool,
    pub flags_after: u8,
    /// Whether the packet issues the virtual +0x70 dirty-mark request.
    pub dirty_mark_requested: bool,
    /// Whether the old cached frame pointer is non-null, so the dirty routine
    /// can proceed past its own null-frame guard.
    pub dirty_body_runs: bool,
}

/// Apply only the optional post-insertion `0x83140` packet for native type 2
/// when the selection-interaction bit is clear. `None` means this helper's
/// traced preconditions are not met; it does not claim behavior for that path.
pub fn family1_admission_packet(input: AdmissionPacketInput) -> Option<AdmissionPacket> {
    if input.native_type_at_6 != 2 || input.context_flags_at_1c & 0x01 != 0 {
        return None;
    }
    if input.shade_at_17 == 0 {
        return Some(AdmissionPacket {
            context_gate_after: input.context_gate_before,
            flags_after: input.flags_at_15,
            dirty_mark_requested: false,
            dirty_body_runs: false,
        });
    }
    let mut flags = input.flags_at_15;
    if input.countdown_at_1d6 > 0 {
        flags |= 0x04;
    }
    let dirty_mark_requested = flags & 0x04 != 0;
    if dirty_mark_requested {
        flags |= 0x08;
    }
    Some(AdmissionPacket {
        context_gate_after: true,
        flags_after: flags,
        dirty_mark_requested,
        dirty_body_runs: dirty_mark_requested && input.old_frame_present,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(id: u32, x: i32, y: i32) -> ProjectedFamily1 {
        ProjectedFamily1 { id, x, y, shade: 1, flags: 0 }
    }

    #[test]
    fn matches_special_bucket_stability_and_middle_bucket_lifo() {
        let input = [
            candidate(0, 0, -1), candidate(1, 0, 0), candidate(2, 0, 1),
            candidate(3, 0, 1), candidate(4, 0, -1), candidate(5, 0, 480),
            candidate(6, 0, 479), candidate(7, 0, 480), candidate(8, 0, 519),
            candidate(9, 0, 479), candidate(10, 0, 0),
        ];
        let queue = queue_family1(true, 0, 0, &input).unwrap();
        assert_eq!(queue.draw_order.iter().map(|entry| entry.id).collect::<Vec<_>>(),
            [0, 4, 1, 10, 3, 2, 9, 6, 5, 7, 8]);
        assert_eq!(queue.draw_order.iter().map(|entry| entry.draw_y).collect::<Vec<_>>(),
            [-1, -1, 0, 0, 1, 1, 479, 479, 479, 479, 518]);
        assert_eq!(queue.final_count, input.len() as u16);
    }

    #[test]
    fn module_margin_geometry_and_capacity_are_explicit() {
        let points = [candidate(0, -40, -10), candidate(1, 599, 519), candidate(2, 600, 519),
            candidate(3, -41, 0), candidate(4, 600, 0), candidate(5, 0, 520)];
        let zero_margin = queue_family1(true, 0, 0, &points).unwrap();
        assert_eq!(zero_margin.draw_order.iter().map(|e| e.id).collect::<Vec<_>>(), [0, 4, 1, 2]);
        let margin = queue_family1(true, 0, 128, &points).unwrap();
        assert_eq!(margin.draw_order.iter().map(|e| e.id).collect::<Vec<_>>(), [0]);
        assert!(queue_family1(false, 0, 0, &points).unwrap().draw_order.is_empty());
        assert!(queue_family1(true, 1, 0, &points).unwrap().final_count <= 256);
        let many: Vec<_> = (0..257).map(|id| candidate(id, 10, 1)).collect();
        let full = queue_family1(true, 0, 0, &many).unwrap();
        assert_eq!(full.final_count, 256);
        assert_eq!(full.draw_order.len(), 256);
        assert_eq!(queue_family1(true, 256, 0, &points).unwrap().draw_order.len(), 0);
        assert_eq!(queue_family1(true, -1, 0, &points), Err(QueueError::InvalidInitialCount(-1)));
        assert_eq!(queue_family1(true, 257, 0, &points), Err(QueueError::InvalidInitialCount(257)));
        assert_eq!(queue_family1(true, 0, 1, &points), Err(QueueError::InvalidMargin(1)));
    }

    #[test]
    fn draw_packet_matches_gate_mode_and_flag_masks() {
        assert_eq!(family1_draw_packet(false, 1, 0x0C), DrawPacket { draws: false, dirty_class: None, flags_after: 0x0C });
        assert_eq!(family1_draw_packet(true, 0, 0x00), DrawPacket { draws: false, dirty_class: None, flags_after: 0 });
        assert_eq!(family1_draw_packet(true, 0, 0x08), DrawPacket { draws: true, dirty_class: Some(1), flags_after: 0 });
        assert_eq!(family1_draw_packet(true, 1, 0x04), DrawPacket { draws: true, dirty_class: Some(3), flags_after: 0 });
        assert_eq!(family1_draw_packet(true, 1, 0xFF), DrawPacket { draws: true, dirty_class: Some(3), flags_after: 0xF3 });
        assert_eq!(family1_draw_packet(true, 0, 0x10), DrawPacket { draws: false, dirty_class: None, flags_after: 0x10 });
    }

    #[test]
    fn admission_packet_preserves_zero_shade_and_handles_countdown_and_null_frame() {
        let base = AdmissionPacketInput { native_type_at_6: 2, context_flags_at_1c: 0,
            context_gate_before: false, shade_at_17: 0, countdown_at_1d6: 1,
            flags_at_15: 4, old_frame_present: true };
        assert_eq!(family1_admission_packet(base), Some(AdmissionPacket {
            context_gate_after: false, flags_after: 4, dirty_mark_requested: false, dirty_body_runs: false,
        }));
        assert_eq!(family1_admission_packet(AdmissionPacketInput { shade_at_17: 1, ..base }), Some(AdmissionPacket {
            context_gate_after: true, flags_after: 12, dirty_mark_requested: true, dirty_body_runs: true,
        }));
        assert_eq!(family1_admission_packet(AdmissionPacketInput { shade_at_17: 1, countdown_at_1d6: -1, flags_at_15: 0, ..base }), Some(AdmissionPacket {
            context_gate_after: true, flags_after: 0, dirty_mark_requested: false, dirty_body_runs: false,
        }));
        assert_eq!(family1_admission_packet(AdmissionPacketInput { shade_at_17: 255, countdown_at_1d6: -1, flags_at_15: 4, old_frame_present: false, ..base }), Some(AdmissionPacket {
            context_gate_after: true, flags_after: 12, dirty_mark_requested: true, dirty_body_runs: false,
        }));
        assert_eq!(family1_admission_packet(AdmissionPacketInput { native_type_at_6: 3, ..base }), None);
        assert_eq!(family1_admission_packet(AdmissionPacketInput { context_flags_at_1c: 1, ..base }), None);
    }
}
