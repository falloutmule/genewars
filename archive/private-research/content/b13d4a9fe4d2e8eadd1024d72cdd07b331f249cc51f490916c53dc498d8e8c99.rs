//! Byte after-images for the proved family-2 actor creation/initialization path.
//!
//! This models the actor writes in `0x4FD30`, followed by the `0x80830`
//! no-associated-unit pass and `0x56E20` resource binding. Position used by
//! `0x4FD30` and its already-sampled height/shade are explicit inputs; the
//! later `0x80830` target is a separate explicit pair. Runtime addresses are
//! represented by typed handle requests and are never forged as byte values.
//! See `research/ghidra/blueprint-init-evidence.json` and
//! `research/ghidra/player-creation-evidence.json`.

use core::fmt;

pub const ACTOR_INITIALIZATION_BYTES: usize = 0xC8;
pub const ACTOR_PARTITION_SLOTS: usize = 5;
pub const PLAYER_SLOTS: usize = 4;
pub const BLUEPRINTS_PER_PLAYER: usize = 32;
pub const BLUEPRINT_RECORD_BYTES: usize = 36;
pub const ACTOR_PLAYER_RECORD_BYTES: usize = 0x70D;
pub const ACTOR_TABLE_BASE: usize = 0x16360E;
pub const ACTOR_TABLE_STRIDE: usize = 0xC8;
/// Half-open byte ranges represented by `handle_bindings`, not native bytes.
pub const ACTOR_HANDLE_BYTE_RANGES: [(usize, usize); 4] = [
    (0x1F, 0x23), (0xB0, 0xB4), (0xC0, 0xC4), (0xC4, 0xC8),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActorInitializationError {
    ActorRecordLength { expected: usize, actual: usize },
    PlayerRecordLength { expected: usize, actual: usize },
    InvalidPlayer(u8),
    PlayerRecordMismatch { requested: u8, record: u8 },
    InvalidLocalSlot(usize),
    OccupiedSlot { active_type: u8 },
    InvalidBlueprintIndex(u8),
    InvalidBlueprintStyle(i8),
    InvalidBlueprintCategory(i8),
    AllocationFailed,
}

impl fmt::Display for ActorInitializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{self:?}") }
}

impl std::error::Error for ActorInitializationError {}

/// Stable logical address of one player metadata record, replacing native C0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlueprintRecordHandle {
    pub player_id: u8,
    pub blueprint_index: u8,
}

/// The resource-table key used by the two distinct 56E20 pointer slots.
/// Flat order is `4 * style + category`, where both signed source bytes have
/// already been admitted to 0..=3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorResourceTableKey {
    pub style: u8,
    pub category: u8,
    pub flat_index: u8,
}

/// A native pointer destination expressed as a logical binding request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorHandleBinding {
    BlueprintMetadata { actor_offset: u8, handle: BlueprintRecordHandle },
    ResourceDatabase { actor_offset: u8, key: ActorResourceTableKey },
    AnimationDatabase { actor_offset: u8, key: ActorResourceTableKey },
    InternalActorField { actor_offset: u8, target_actor_offset: u8 },
}

/// Exact 200-byte nonpointer after-image. Native pointer slots remain zero
/// placeholders and are identified by `handle_bindings`; a host must resolve
/// those handles in its own runtime arena before dispatching pointer consumers.
/// Unmentioned fields, including the actor vtable bytes+0x23..+0x27, are copied
/// from the caller's prior slot and remain opaque.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorInitializationPlan {
    pub player_id: u8,
    pub local_slot: u8,
    pub global_actor_id: u16,
    pub blueprint_index: u8,
    /// The temporary input position consumed by 4FD30's height/shade samples.
    pub temporary_origin: [i16; 2],
    /// The separate position written later by 80830.
    pub final_target: [i16; 2],
    pub sampled_origin_height: i32,
    pub sampled_origin_shade: u8,
    pub resource_key: ActorResourceTableKey,
    pub actor_before: Vec<u8>,
    pub actor_after_nonpointer: Vec<u8>,
    pub handle_bindings: [ActorHandleBinding; 4],
}

/// Plan the allocator's first-free family-2 slot through its explicit no-unit
/// initialization pass. This does not allocate a slot, append the player ID,
/// mutate the blueprint's active bit, calculate terrain samples, execute
/// resource loading, or make the actor ready for the separate 80AE0 path.
///
/// The full player record is checked so both the owner and selected metadata
/// record are bound to the supplied IDs. Only the actor and blueprint bytes
/// proven by the bridge are read; all native addresses remain handles.
pub fn plan_no_unit_actor_initialization(
    actor_before: &[u8],
    player_id: u8,
    local_slot: usize,
    player_record: &[u8],
    blueprint_index: u8,
    temporary_origin: [i16; 2],
    sampled_origin_height: i32,
    sampled_origin_shade: u8,
    final_target: [i16; 2],
) -> Result<ActorInitializationPlan, ActorInitializationError> {
    if actor_before.len() != ACTOR_INITIALIZATION_BYTES {
        return Err(ActorInitializationError::ActorRecordLength {
            expected: ACTOR_INITIALIZATION_BYTES, actual: actor_before.len(),
        });
    }
    if player_record.len() != ACTOR_PLAYER_RECORD_BYTES {
        return Err(ActorInitializationError::PlayerRecordLength {
            expected: ACTOR_PLAYER_RECORD_BYTES, actual: player_record.len(),
        });
    }
    if player_id >= PLAYER_SLOTS as u8 { return Err(ActorInitializationError::InvalidPlayer(player_id)); }
    if player_record[1] != player_id {
        return Err(ActorInitializationError::PlayerRecordMismatch { requested: player_id, record: player_record[1] });
    }
    if local_slot >= ACTOR_PARTITION_SLOTS { return Err(ActorInitializationError::InvalidLocalSlot(local_slot)); }
    if actor_before[6] != 0 { return Err(ActorInitializationError::OccupiedSlot { active_type: actor_before[6] }); }
    if usize::from(blueprint_index) >= BLUEPRINTS_PER_PLAYER {
        return Err(ActorInitializationError::InvalidBlueprintIndex(blueprint_index));
    }

    let blueprint_start = 0x16 + usize::from(blueprint_index) * BLUEPRINT_RECORD_BYTES;
    let blueprint = &player_record[blueprint_start..blueprint_start + BLUEPRINT_RECORD_BYTES];
    let style = blueprint[0x15] as i8;
    let category = blueprint[0x16] as i8;
    if !(0..=3).contains(&style) { return Err(ActorInitializationError::InvalidBlueprintStyle(style)); }
    if !(0..=3).contains(&category) { return Err(ActorInitializationError::InvalidBlueprintCategory(category)); }
    let style = style as u8;
    let category = category as u8;
    let resource_key = ActorResourceTableKey { style, category, flat_index: style * 4 + category };
    let global_actor_id = u16::from(player_id) * ACTOR_PARTITION_SLOTS as u16 + local_slot as u16;

    let mut after = Vec::new();
    after.try_reserve_exact(ACTOR_INITIALIZATION_BYTES).map_err(|_| ActorInitializationError::AllocationFailed)?;
    after.extend_from_slice(actor_before);
    // 4FD30 starts with two null-reference helper calls. They clear only the
    // kind bytes; the unused reference index words remain from the slot.
    after[2] = 0;
    after[5] = 0;
    after[6] = 3;
    after[0x0F..0x11].copy_from_slice(&global_actor_id.to_le_bytes());
    after[0x07..0x09].copy_from_slice(&temporary_origin[0].to_le_bytes());
    after[0x09..0x0B].copy_from_slice(&temporary_origin[1].to_le_bytes());
    after[0x11..0x15].copy_from_slice(&sampled_origin_height.to_le_bytes());
    after[0x17] = sampled_origin_shade;
    after[0x0B..0x0F].fill(0);
    after[0x15..0x17].copy_from_slice(&0x0280u16.to_le_bytes());
    after[0x18] = player_id;
    after[0x19] = 8;
    after[0x1A] = 0;
    after[0x1B] = 0;
    after[0x1C] = 1;
    after[0x1D] = 0;
    after[0x27..0x2B].fill(0);
    after[0x2B..0x2D].copy_from_slice(&50u16.to_le_bytes());
    after[0x2D..0x39].fill(0);
    after[0x39..0x3F].copy_from_slice(&[0, 0, 0, 2, 0, 0]); // [0, 512, 0] angle words
    after[0x4A] &= 0x3B;
    after[0x53] = 1;
    after[0x54] = 0;
    after[0x55] = 1;
    after[0x68..0x6C].fill(0);
    after[0x70..0x72].copy_from_slice(&0x0400u16.to_le_bytes());
    after[0x74] = blueprint_index;
    after[0x07..0x09].copy_from_slice(&final_target[0].to_le_bytes());
    after[0x09..0x0B].copy_from_slice(&final_target[1].to_le_bytes());
    after[0x15..0x17].copy_from_slice(&0x0A80u16.to_le_bytes());

    // 56E20 clears native94..BF before binding its internal pointer at B0.
    // Zero placeholders keep this byte view deterministic without inventing
    // process addresses for B0/C0/C4 or the animation handle at+1F.
    after[0x94..0xC0].fill(0);
    after[0x1F..0x23].fill(0);
    after[0xC0..0xC8].fill(0);
    let handle_bindings = [
        ActorHandleBinding::BlueprintMetadata {
            actor_offset: 0xC0,
            handle: BlueprintRecordHandle { player_id, blueprint_index },
        },
        ActorHandleBinding::ResourceDatabase { actor_offset: 0xC4, key: resource_key },
        ActorHandleBinding::AnimationDatabase { actor_offset: 0x1F, key: resource_key },
        ActorHandleBinding::InternalActorField { actor_offset: 0xB0, target_actor_offset: 0xC4 },
    ];

    let mut before = Vec::new();
    before.try_reserve_exact(ACTOR_INITIALIZATION_BYTES).map_err(|_| ActorInitializationError::AllocationFailed)?;
    before.extend_from_slice(actor_before);
    Ok(ActorInitializationPlan {
        player_id, local_slot: local_slot as u8, global_actor_id, blueprint_index,
        temporary_origin, final_target, sampled_origin_height, sampled_origin_shade,
        resource_key, actor_before: before, actor_after_nonpointer: after,
        handle_bindings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player(player_id: u8) -> Vec<u8> {
        let mut bytes = vec![0x6D; ACTOR_PLAYER_RECORD_BYTES];
        bytes[1] = player_id;
        bytes
    }

    fn set_blueprint(bytes: &mut [u8], index: u8, style: i8, category: i8) {
        let start = 0x16 + usize::from(index) * BLUEPRINT_RECORD_BYTES;
        bytes[start] = b'P';
        bytes[start + 0x15] = style as u8;
        bytes[start + 0x16] = category as u8;
    }

    #[test]
    fn composed_actor_init_preserves_unknowns_and_emits_typed_handles() {
        let player = {
            let mut bytes = player(2);
            set_blueprint(&mut bytes, 10, 2, 3);
            bytes
        };
        let mut actor = vec![0xA5; ACTOR_INITIALIZATION_BYTES];
        actor[6] = 0; // first-free allocator result
        let before = actor.clone();
        let plan = plan_no_unit_actor_initialization(
            &actor, 2, 1, &player, 10, [100, -200], 123_456, 37, [777, -888],
        ).unwrap();

        assert_eq!(plan.global_actor_id, 11);
        assert_eq!(plan.temporary_origin, [100, -200]);
        assert_eq!(plan.final_target, [777, -888]);
        assert_eq!(plan.sampled_origin_height, 123_456);
        assert_eq!(plan.sampled_origin_shade, 37);
        assert_eq!(plan.actor_before, before);
        let after = &plan.actor_after_nonpointer;
        assert_eq!(after.len(), ACTOR_INITIALIZATION_BYTES);
        assert_eq!(after[2], 0);
        assert_eq!(after[5], 0);
        assert_eq!(&after[0..2], &before[0..2]);
        assert_eq!(&after[3..5], &before[3..5]);
        assert_eq!(after[6], 3);
        assert_eq!(&after[7..11], &[777i16.to_le_bytes(), (-888i16).to_le_bytes()].concat());
        assert_eq!(&after[0x11..0x15], &123_456i32.to_le_bytes());
        assert_eq!(after[0x17], 37);
        assert_eq!(&after[0x0F..0x11], &11u16.to_le_bytes());
        assert_eq!(after[0x18], 2);
        assert_eq!(&after[0x15..0x17], &0x0A80u16.to_le_bytes());
        assert_eq!(after[0x19], 8);
        assert_eq!(after[0x1A], 0);
        assert_eq!(after[0x1B], 0);
        assert_eq!(after[0x1C], 1);
        assert_eq!(after[0x1D], 0);
        assert_eq!(&after[0x2B..0x2D], &50u16.to_le_bytes());
        assert_eq!(&after[0x39..0x3F], &[0, 0, 0, 2, 0, 0]);
        assert_eq!(after[0x53], 1);
        assert_eq!(after[0x54], 0);
        assert_eq!(after[0x55], 1);
        assert_eq!(&after[0x68..0x6C], &[0; 4]);
        assert_eq!(&after[0x70..0x72], &0x0400u16.to_le_bytes()); // ready bit0 remains clear
        assert_eq!(after[0x74], 10);
        assert_eq!(after[0x23], 0xA5); // unknown byte outside pointer fields is preserved
        for (start, end) in ACTOR_HANDLE_BYTE_RANGES { assert!(after[start..end].iter().all(|&byte| byte == 0)); }
        assert_eq!(plan.resource_key, ActorResourceTableKey { style: 2, category: 3, flat_index: 11 });
        assert_eq!(plan.handle_bindings, [
            ActorHandleBinding::BlueprintMetadata {
                actor_offset: 0xC0, handle: BlueprintRecordHandle { player_id: 2, blueprint_index: 10 },
            },
            ActorHandleBinding::ResourceDatabase { actor_offset: 0xC4, key: plan.resource_key },
            ActorHandleBinding::AnimationDatabase { actor_offset: 0x1F, key: plan.resource_key },
            ActorHandleBinding::InternalActorField { actor_offset: 0xB0, target_actor_offset: 0xC4 },
        ]);
        assert_eq!(actor, before); // planning is pure
    }

    #[test]
    fn no_unit_initialization_rejects_bad_identity_slot_and_signed_resource_keys() {
        let actor = vec![0; ACTOR_INITIALIZATION_BYTES];
        let mut player = player(0);
        set_blueprint(&mut player, 0, 0, 0);
        let valid = plan_no_unit_actor_initialization(&actor, 0, 0, &player, 0, [0; 2], 0, 0, [0; 2]);
        assert!(valid.is_ok());
        assert_eq!(plan_no_unit_actor_initialization(&actor, 4, 0, &player, 0, [0; 2], 0, 0, [0; 2]).unwrap_err(), ActorInitializationError::InvalidPlayer(4));
        assert_eq!(plan_no_unit_actor_initialization(&actor, 1, 0, &player, 0, [0; 2], 0, 0, [0; 2]).unwrap_err(), ActorInitializationError::PlayerRecordMismatch { requested: 1, record: 0 });
        assert_eq!(plan_no_unit_actor_initialization(&actor, 0, 5, &player, 0, [0; 2], 0, 0, [0; 2]).unwrap_err(), ActorInitializationError::InvalidLocalSlot(5));
        assert_eq!(plan_no_unit_actor_initialization(&actor, 0, 0, &player, 32, [0; 2], 0, 0, [0; 2]).unwrap_err(), ActorInitializationError::InvalidBlueprintIndex(32));

        player[0x16 + 0x15] = 0xFF;
        assert_eq!(plan_no_unit_actor_initialization(&actor, 0, 0, &player, 0, [0; 2], 0, 0, [0; 2]).unwrap_err(), ActorInitializationError::InvalidBlueprintStyle(-1));
        player[0x16 + 0x15] = 0;
        player[0x16 + 0x16] = 4;
        assert_eq!(plan_no_unit_actor_initialization(&actor, 0, 0, &player, 0, [0; 2], 0, 0, [0; 2]).unwrap_err(), ActorInitializationError::InvalidBlueprintCategory(4));

        let mut occupied = actor.clone();
        occupied[6] = 3;
        assert_eq!(plan_no_unit_actor_initialization(&occupied, 0, 0, &player, 0, [0; 2], 0, 0, [0; 2]).unwrap_err(), ActorInitializationError::OccupiedSlot { active_type: 3 });
    }
}
