//! Numeric 101D0/1EE80 creation plan for source family8 selectors0..2.
//! Pool allocation, map commits, callbacks, and animation resources are external.

pub const RECORD_BYTES: usize = 102;
pub type Family8Record = [u8; RECORD_BYTES];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellHeadIdentity { pub index: u16, pub kind: u8 }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreationError { InvalidSlot, InvalidOwner, InvalidSelector, ActiveSlot, InvalidHead }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreationPlan {
    pub after: Family8Record,
    /// Byte offset into the source116820 animation table; no machine pointer.
    pub animation_table_offset: u8,
    pub cell: u16,
    pub cell_head_after: CellHeadIdentity,
    pub old_head_previous: Option<(CellHeadIdentity, CellHeadIdentity)>,
}

#[derive(Debug, Clone, Copy)]
pub struct CreationInput {
    pub slot: u16,
    pub owner: u8,
    pub selector: u8,
    pub position: [i16; 2],
    pub ground_height: i32,
    pub shade: u8,
    pub initial_height: i32,
    pub old_head: Option<CellHeadIdentity>,
}

fn word(record: &mut Family8Record, offset: usize, value: u16) {
    record[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}
fn dword(record: &mut Family8Record, offset: usize, value: i32) {
    record[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

/// The factory's pinned112D14 vector is zero, so its initial velocity is zero.
/// Pointer slots1F..26 remain opaque; the animation binding is returned typed.
pub fn plan_creation(before: &Family8Record, input: CreationInput) -> Result<CreationPlan, CreationError> {
    if input.slot >= 16 { return Err(CreationError::InvalidSlot); }
    if input.owner >= 4 { return Err(CreationError::InvalidOwner); }
    if input.selector > 2 { return Err(CreationError::InvalidSelector); }
    if before[6] != 0 { return Err(CreationError::ActiveSlot); }
    if input.old_head.is_some_and(|h| h.kind == 0 || h.kind > 8) { return Err(CreationError::InvalidHead); }
    let mut after = *before;
    after[2] = 0; after[5] = 0; after[6] = 8;
    for (offset, value) in [(7, input.position[0] as u16), (9, input.position[1] as u16), (15, input.slot)] {
        word(&mut after, offset, value);
    }
    after[0x18] = input.owner;
    dword(&mut after, 0x11, input.ground_height);
    after[0x17] = input.shade;
    word(&mut after, 0x15, 0);
    dword(&mut after, 0xb, 0);
    for offset in [0x19, 0x1a, 0x1b, 0x1d, 0x53, 0x54] { after[offset] = 0; }
    after[0x1c] = 1;
    after[0x2d..0x3f].fill(0);
    dword(&mut after, 0x27, input.initial_height);
    after[0x4a] &= 0x3b;
    after[0x15] |= 0x20;
    after[0x55] = input.selector;
    let cell = ((input.position[1] as u16 >> 8) & 127) * 128
        + ((input.position[0] as u16 >> 8) & 127);
    word(&mut after, 0, cell);
    let identity = CellHeadIdentity { index: input.slot, kind: 8 };
    if let Some(head) = input.old_head {
        word(&mut after, 3, head.index); after[5] = head.kind;
    }
    word(&mut after, 0x2b, if input.selector == 1 { 0x20 } else { 8 });
    after[0x56] = [4, 0, 10][usize::from(input.selector)];
    dword(&mut after, 0xb, input.ground_height.wrapping_add([16000, 4800, 22400][usize::from(input.selector)]));
    after[0x16] |= 0x40;
    after[0x49] |= 0x10;
    after[0x4f] = 0; after[0x52] = 0;
    word(&mut after, 0x3f, if input.selector == 1 { 0xf00 } else { 0x780 });
    word(&mut after, 0x41, if input.selector == 1 { 0 } else { 0x780 });
    word(&mut after, 0x3b, 0x400);
    if input.selector == 1 {
        word(&mut after, 0x39, 0x300);
        word(&mut after, 0x4b, 0x300);
        after[0x5b] = 0;
    } else {
        after[0x4a] &= 0xdf;
        if input.selector == 2 { after[0x18] = 4; after[0x58] = input.owner; }
    }
    Ok(CreationPlan {
        after, animation_table_offset: if input.selector == 1 { 0 } else { 0x50 },
        cell, cell_head_after: identity,
        old_head_previous: input.old_head.map(|head| (head, identity)),
    })
}
