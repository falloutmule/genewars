//! Typed actor draw inputs extracted from the owned native actor pool.
//!
//! The bounded compositor below consumes explicit screen coordinates and
//! wrapper state. It does not invent camera projection, draw ordering,
//! visibility gates, animation timing, or the caller's 56F10 flag setup.

use core::fmt;
use genewars_assets::{AnbComponentRecord, AnbFrameRecord, IndexedSprite};

use crate::{
    actor_initialization::BlueprintRecordHandle,
    actor_pool::{ActorPool, ActorPoolError},
    actor_resources::{ActorAnimationDatabase, ActorResourceCatalog, ActorResourceError},
    actor_initialization::ActorResourceTableKey,
    player_state::PlayerState,
};

const ACTOR_TYPE: usize = 0x06;
const POSITION_X: usize = 0x07;
const POSITION_Y: usize = 0x09;
const ACTOR_ID: usize = 0x0f;
const FLAGS_15: usize = 0x15;
const SHADE: usize = 0x17;
const OWNER: usize = 0x18;
const ANIMATION: usize = 0x19;
const FRAME: usize = 0x1b;
const DRAW_FLAGS: usize = 0x55;
const DRAW_MODE: usize = 0x73;
const NATIVE_ACTOR_BYTES: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActorSourceFields {
    actor_id: u16,
    owner: u8,
    resource_key: ActorResourceTableKey,
    world_position: [i16; 2],
    virtual_height: i32,
    shade: u8,
    actor_flags_15: u8,
    animation_index: u8,
    frame_index: u8,
    draw_flags_55: u8,
    draw_flag_58: u8,
    draw_flag_59: i8,
    draw_flags_70: u16,
    draw_flag_7e: u8,
    draw_flag_82: u8,
    draw_mode_73: u8,
}

fn parse_actor_source_fields(
    actor_slot: usize,
    record: &[u8],
    resource_key: ActorResourceTableKey,
) -> Result<ActorSourceFields, ActorRenderError> {
    if record.len() != NATIVE_ACTOR_BYTES {
        return Err(ActorRenderError::RecordLength { expected: NATIVE_ACTOR_BYTES, actual: record.len() });
    }
    if record[ACTOR_TYPE] != 3 {
        return Err(ActorRenderError::WrongActorType { slot: actor_slot, actor_type: record[ACTOR_TYPE] });
    }
    let actor_id = u16::from_le_bytes([record[ACTOR_ID], record[ACTOR_ID + 1]]);
    if usize::from(actor_id) != actor_slot {
        return Err(ActorRenderError::ActorIdMismatch { slot: actor_slot, record_id: actor_id });
    }
    let owner = record[OWNER];
    if owner >= 4 {
        return Err(ActorRenderError::OwnerOutOfRange { slot: actor_slot, owner });
    }
    Ok(ActorSourceFields {
        actor_id,
        owner,
        resource_key,
        world_position: [
            i16::from_le_bytes([record[POSITION_X], record[POSITION_X + 1]]),
            i16::from_le_bytes([record[POSITION_Y], record[POSITION_Y + 1]]),
        ],
        virtual_height: i32::from_le_bytes(record[0x27..0x2b].try_into().expect("checked native actor record"))
            .wrapping_sub(i32::from_le_bytes(record[0x11..0x15].try_into().expect("checked native actor record"))),
        shade: record[SHADE],
        actor_flags_15: record[FLAGS_15],
        animation_index: record[ANIMATION],
        frame_index: record[FRAME],
        draw_flags_55: record[DRAW_FLAGS],
        draw_flag_58: record[0x58],
        draw_flag_59: record[0x59] as i8,
        draw_flags_70: u16::from_le_bytes([record[0x70], record[0x71]]),
        draw_flag_7e: record[0x7e],
        draw_flag_82: record[0x82],
        draw_mode_73: record[DRAW_MODE],
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActorRenderError {
    Pool(ActorPoolError),
    Resource(ActorResourceError),
    WrongActorType { slot: usize, actor_type: u8 },
    ActorIdMismatch { slot: usize, record_id: u16 },
    OwnerOutOfRange { slot: usize, owner: u8 },
    MissingResourceBinding(usize),
    RecordLength { expected: usize, actual: usize },
    ResourceBindingMismatch { slot: usize, bound: ActorResourceTableKey, loaded: ActorResourceTableKey },
    InvalidResourceKey(ActorResourceTableKey),
    BlueprintBindingMismatch { slot: usize, expected: BlueprintRecordHandle, actual: Option<BlueprintRecordHandle> },
    BlueprintPlayerMismatch { expected: u8, actual: u8 },
    BlueprintIndexOutOfRange(u8),
    AllocationFailed,
    FrameRecordMismatch,
    InvalidDrawFlags(u8),
    FadeRowRequired,
    FadeRowLength { actual: usize },
    BlendTableRequired,
    BlendTableLength { actual: usize },
    InvalidFramebufferExtent,
    SpriteStorageMismatch,
    AllocationFailedForDraw,
    SelectedPlayerOutOfRange(u8),
    FadeShadeOutOfRange(i32),
}

impl fmt::Display for ActorRenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for ActorRenderError {}

impl From<ActorPoolError> for ActorRenderError {
    fn from(value: ActorPoolError) -> Self { Self::Pool(value) }
}
impl From<ActorResourceError> for ActorRenderError {
    fn from(value: ActorResourceError) -> Self { Self::Resource(value) }
}

/// One active family-2 type-3 actor with its resource and frame references
/// checked against the exact loaded resource database. `world_position` is
/// copied from the signed words at native offsets +7 and +9; it is not a
/// screen position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorRenderRecord {
    pub actor_slot: usize,
    pub actor_id: u16,
    pub owner: u8,
    pub blueprint: BlueprintRecordHandle,
    pub resource_key: ActorResourceTableKey,
    pub world_position: [i16; 2],
    /// Result of the proved family2 virtual +4 at 0x1F260: signed dword
    /// +0x27 minus signed dword +0x11, with source 32-bit wrap semantics.
    pub virtual_height: i32,
    pub shade: u8,
    pub actor_flags_15: u8,
    pub animation_index: u8,
    pub frame_index: u8,
    pub draw_flags_55: u8,
    pub draw_flag_58: u8,
    pub draw_flag_59: i8,
    pub draw_flags_70: u16,
    pub draw_flag_7e: u8,
    pub draw_flag_82: u8,
    pub draw_mode_73: u8,
    pub frame_record: AnbFrameRecord,
    pub components: Vec<AnbComponentRecord>,
}

/// One component sprite decoded through the actor wrapper's indexed8 RLE
/// path. The sprite's mask remains separate from its indices, so literal
/// palette index zero stays opaque when the caller later composites it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorDecodedComponent {
    pub component: AnbComponentRecord,
    pub sprite: IndexedSprite,
}

/// Explicit inputs normally established by the wrapper/caller around 57070.
/// `origin` is a signed screen coordinate, not a projected world point.
/// `global_flags` is the already-resolved value from 56F10; this compositor
/// does not infer the associated blueprint or selected-player state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorDrawContext {
    pub origin: [i32; 2],
    pub virtual_height: i32,
    pub global_flags: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorWrapperKind {
    /// Ordinary 57070 path; caller must perform the separately proved shadow pass.
    World,
    /// 57050 direct wrapper; skips shadow and vertical-origin subtraction.
    PortraitDirect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorDrawPolicy {
    pub visible: bool,
    pub context: ActorDrawContext,
    pub effective_shade: i32,
    /// Signed row index; source selection compares raw shade +17, while the
    /// effective row also includes the virtual-height shift. The caller must
    /// bound this index against its supplied table.
    pub fade_row: Option<i32>,
    pub shadow_required: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ActorDrawReceipt {
    pub pixels_written: usize,
    pub pixels_clipped: usize,
    pub opaque_pixels: usize,
}

/// Resolve the proved 56F10 wrapper state from explicit associated-blueprint
/// bytes and the actor fields copied into `ActorRenderRecord`. The two
/// blueprint bytes are at offsets +15 (class) and +16 (mode) of the bound
/// 36-byte player blueprint. The caller also supplies the global selected
/// player/frame counter and screen origin; none are guessed here.
fn plan_actor_draw_fields(
    record: &ActorRenderRecord,
    blueprint_class: u8,
    blueprint_mode: u8,
    selected_player: u8,
    frame_counter: u8,
    screen_origin: [i16; 2],
    virtual_height: i32,
    wrapper: ActorWrapperKind,
) -> Result<ActorDrawPolicy, ActorRenderError> {
    if selected_player >= 4 { return Err(ActorRenderError::SelectedPlayerOutOfRange(selected_player)); }

    let parity = frame_counter.wrapping_add(record.actor_id as u8) & 1;
    let visible = blueprint_class != 3
        || record.draw_mode_73 < 224
        || (record.owner == selected_player && parity == 0);

    let mut flags = 0u8;
    if blueprint_class == 3 {
        if record.draw_mode_73 >= 171 { flags = 4; }
        else if record.draw_mode_73 >= 85 { flags = 12; }
    }

    let draw_mode = record.draw_flags_55;
    if draw_mode == 1 {
        if record.draw_flags_70 & 0x04 != 0 { flags |= 1; }
    } else if draw_mode == 7 {
        if record.draw_flags_70 & 0x0104 == 0x0104 { flags |= 1; }
    } else {
        match blueprint_mode {
            0 => {
                if record.draw_flag_58 & 1 != 0
                    && (record.draw_flag_7e <= 1 || record.draw_flag_7e == 4)
                    && record.draw_flags_70 & 4 != 0
                { flags |= 1; }
            }
            1 => {
                if record.draw_flags_70 & 0x84 == 0x84 { flags |= 1; }
            }
            2 => {
                if record.draw_flag_82 != 0
                    && record.draw_flags_70 & 0x84 == 0x84
                    && matches!(record.draw_flag_59, 13 | 14)
                { flags |= 1; }
            }
            _ => {}
        }
    }

    if record.actor_flags_15 & 0x10 != 0 && parity != 0 { flags |= 4; }
    let effective_shade = i32::from(record.shade) + (virtual_height >> 7);
    let fade_row = if record.actor_flags_15 & 0x02 == 0 && record.shade < 30 {
        if effective_shade < 0 {
            return Err(ActorRenderError::FadeShadeOutOfRange(effective_shade));
        }
        Some(effective_shade)
    } else { None };
    let direct = wrapper == ActorWrapperKind::PortraitDirect;
    let origin_y = if direct { i32::from(screen_origin[1]) }
        else { i32::from(screen_origin[1]) - (virtual_height >> 6) };
    Ok(ActorDrawPolicy {
        visible,
        context: ActorDrawContext { origin: [i32::from(screen_origin[0]), origin_y], virtual_height, global_flags: flags },
        effective_shade,
        fade_row,
        shadow_required: !direct,
    })
}

/// Resolve wrapper state from the owned player record bound to this actor.
/// This keeps the class/mode bytes at blueprint +15/+16 attached to the
/// actor's typed C0 binding rather than accepting unrelated caller bytes.
pub fn plan_actor_draw(
    record: &ActorRenderRecord,
    player: &PlayerState,
    selected_player: u8,
    frame_counter: u8,
    screen_origin: [i16; 2],
    virtual_height: i32,
    wrapper: ActorWrapperKind,
) -> Result<ActorDrawPolicy, ActorRenderError> {
    if player.player_id() != record.blueprint.player_id {
        return Err(ActorRenderError::BlueprintPlayerMismatch {
            expected: record.blueprint.player_id, actual: player.player_id(),
        });
    }
    let blueprint = player.blueprint(usize::from(record.blueprint.blueprint_index))
        .map_err(|_| ActorRenderError::BlueprintIndexOutOfRange(record.blueprint.blueprint_index))?;
    plan_actor_draw_fields(record, blueprint[0x15], blueprint[0x16], selected_player,
        frame_counter, screen_origin, virtual_height, wrapper)
}

/// Apply the bounded indexed8 actor color pass for one already-selected frame.
///
/// This implements the non-shadow component pass at 57070/96870/96990. The
/// caller supplies the 56F10 global flags and screen origin. Fade rows and the
/// 65,536-byte blend table are required only for component modes that access
/// them. Unsupported high component flag bits are rejected. All checks and
/// pixel operations are completed before the destination is changed.
pub fn draw_actor_components(
    record: &ActorRenderRecord,
    decoded: &[ActorDecodedComponent],
    context: ActorDrawContext,
    fade_rows: Option<&[u8]>,
    blend_table: Option<&[u8]>,
    destination: &mut [u8],
    width: usize,
    height: usize,
    pitch: usize,
) -> Result<ActorDrawReceipt, ActorRenderError> {
    if decoded.len() != record.components.len()
        || decoded.iter().zip(&record.components).any(|(a, b)| a.component != *b)
    {
        return Err(ActorRenderError::FrameRecordMismatch);
    }
    let required = if width == 0 || height == 0 { 0 } else {
        pitch.checked_mul(height - 1).and_then(|n| n.checked_add(width)).ok_or(ActorRenderError::InvalidFramebufferExtent)?
    };
    if pitch < width || required > destination.len() || width > 640 || height > 480 {
        return Err(ActorRenderError::InvalidFramebufferExtent);
    }
    if width == 0 || height == 0 { return Ok(ActorDrawReceipt::default()); }

    let mut staged = Vec::new();
    staged.try_reserve_exact(required).map_err(|_| ActorRenderError::AllocationFailedForDraw)?;
    staged.extend_from_slice(&destination[..required]);

    let effective_shade = i32::from(record.shade) + (context.virtual_height >> 7);
    let use_fade = record.actor_flags_15 & 0x02 == 0 && record.shade < 30;
    if use_fade {
        let _row = usize::try_from(effective_shade).map_err(|_| ActorRenderError::FadeRowRequired)?;
        let table = fade_rows.ok_or(ActorRenderError::FadeRowRequired)?;
        if table.len() != 256 { return Err(ActorRenderError::FadeRowLength { actual: table.len() }); }
    }

    let bias = record.owner.wrapping_mul(16).wrapping_sub(64);
    let mut receipt = ActorDrawReceipt::default();
    for item in decoded {
        let sprite = &item.sprite;
        let pixel_count = usize::from(sprite.width).checked_mul(usize::from(sprite.height)).ok_or(ActorRenderError::SpriteStorageMismatch)?;
        if sprite.indices.len() != pixel_count || sprite.opaque.len() != pixel_count {
            return Err(ActorRenderError::SpriteStorageMismatch);
        }
        if sprite.width == 0 || sprite.height == 0 { continue; }
        let mut flags = if context.global_flags & 3 == 0 {
            context.global_flags | item.component.flags
        } else {
            (context.global_flags & 0xfc) | item.component.flags
        };
        if flags & 0xf0 != 0 { return Err(ActorRenderError::InvalidDrawFlags(flags)); }
        let (local_x, local_y) = item.component.local_position();
        let mut ox = i32::from(local_x);
        let mut oy = i32::from(local_y);
        if context.global_flags & 1 != 0 {
            ox = -i32::from(sprite.width) - ox;
            flags ^= 1;
        }
        if context.global_flags & 2 != 0 {
            oy = -i32::from(sprite.height) - oy;
            flags ^= 2;
        }
        // The native origin is supplied by a host and can be any i32. Widen
        // before adding component offsets and raster coordinates so extreme
        // but valid off-screen placements clip cleanly in debug and release.
        let left = i64::from(context.origin[0]) + i64::from(ox);
        let top = i64::from(context.origin[1]) + i64::from(oy);
        let hflip = flags & 1 != 0;
        let vflip = flags & 2 != 0;
        let blend_mode = flags & 0x0c;
        if blend_mode != 0 {
            let table = blend_table.ok_or(ActorRenderError::BlendTableRequired)?;
            if table.len() != 65_536 { return Err(ActorRenderError::BlendTableLength { actual: table.len() }); }
        }
        for dy in 0..usize::from(sprite.height) {
            let py = top + dy as i64;
            for dx in 0..usize::from(sprite.width) {
                let src_x = if hflip { usize::from(sprite.width) - 1 - dx } else { dx };
                let src_y = if vflip { usize::from(sprite.height) - 1 - dy } else { dy };
                let source_index = src_y * usize::from(sprite.width) + src_x;
                if !sprite.opaque[source_index] { continue; }
                receipt.opaque_pixels += 1;
                let px = left + dx as i64;
                if px < 0 || py < 0 || px >= width as i64 || py >= height as i64 {
                    receipt.pixels_clipped += 1;
                    continue;
                }
                let mut mapped = sprite.indices[source_index];
                if (0xc0..=0xcf).contains(&mapped) { mapped = bias.wrapping_add(mapped & 0x0f); }
                if use_fade { mapped = fade_rows.expect("validated fade row")[mapped as usize]; }
                if blend_mode != 0 {
                    let offset = py as usize * pitch + px as usize;
                    let old = staged[offset];
                    let table = blend_table.expect("validated blend table");
                    mapped = match blend_mode {
                        4 => table[usize::from(old) * 256 + usize::from(mapped)],
                        8 | 12 => table[usize::from(mapped) * 256 + usize::from(old)],
                        _ => return Err(ActorRenderError::InvalidDrawFlags(flags)),
                    };
                }
                let offset = py as usize * pitch + px as usize;
                staged[offset] = mapped;
                receipt.pixels_written += 1;
            }
        }
    }
    destination[..required].copy_from_slice(&staged);
    Ok(receipt)
}

/// Decode only the components referenced by this validated actor frame.
/// Drawing transforms, palette selection, fade, clipping, and projection stay
/// explicit inputs to a later compositor.
pub fn decode_actor_components_indexed8<B: AsRef<[u8]>>(
    record: &ActorRenderRecord,
    database: &ActorAnimationDatabase<B>,
) -> Result<Vec<ActorDecodedComponent>, ActorRenderError> {
    if database.key() != record.resource_key {
        return Err(ActorRenderError::ResourceBindingMismatch {
            slot: record.actor_slot,
            bound: record.resource_key,
            loaded: database.key(),
        });
    }
    let frame = database.frame(usize::from(record.animation_index), usize::from(record.frame_index))?;
    if frame != record.frame_record || usize::from(frame.component_count) != record.components.len() {
        return Err(ActorRenderError::FrameRecordMismatch);
    }
    let mut decoded = Vec::new();
    decoded.try_reserve_exact(record.components.len()).map_err(|_| ActorRenderError::AllocationFailed)?;
    for (component_index, component) in record.components.iter().copied().enumerate() {
        let actual = database.component(usize::from(record.animation_index), usize::from(record.frame_index), component_index)?;
        if actual != component { return Err(ActorRenderError::FrameRecordMismatch); }
        let sprite = database.decode_component_indexed8(
            usize::from(record.animation_index),
            usize::from(record.frame_index),
            component_index,
        )?;
        decoded.push(ActorDecodedComponent { component, sprite });
    }
    Ok(decoded)
}

/// Read one actor's proved common fields. The caller supplies the database
/// resolved by the actor's typed pool binding; mismatched or absent bindings
/// are errors. No native pointer bytes are interpreted.
pub fn extract_actor_render_record<B: AsRef<[u8]>>(
    pool: &ActorPool,
    actor_slot: usize,
    catalog: &ActorResourceCatalog,
    database: &ActorAnimationDatabase<B>,
) -> Result<ActorRenderRecord, ActorRenderError> {
    let record = pool.record(actor_slot)?;
    if record[ACTOR_TYPE] != 3 {
        return Err(ActorRenderError::WrongActorType { slot: actor_slot, actor_type: record[ACTOR_TYPE] });
    }
    let actor_id = u16::from_le_bytes([record[ACTOR_ID], record[ACTOR_ID + 1]]);
    if usize::from(actor_id) != actor_slot {
        return Err(ActorRenderError::ActorIdMismatch { slot: actor_slot, record_id: actor_id });
    }
    let owner = record[OWNER];
    if owner >= 4 {
        return Err(ActorRenderError::OwnerOutOfRange { slot: actor_slot, owner });
    }

    let bindings = pool.bindings(actor_slot)?;
    let blueprint_index = record[0x74];
    if blueprint_index >= 32 { return Err(ActorRenderError::BlueprintIndexOutOfRange(blueprint_index)); }
    let expected_blueprint = BlueprintRecordHandle { player_id: owner, blueprint_index };
    if bindings.blueprint != Some(expected_blueprint) {
        return Err(ActorRenderError::BlueprintBindingMismatch {
            slot: actor_slot, expected: expected_blueprint, actual: bindings.blueprint,
        });
    }
    let key = bindings.animation_database.ok_or(ActorRenderError::MissingResourceBinding(actor_slot))?;
    if bindings.resource_database != Some(key) || database.key() != key {
        return Err(ActorRenderError::ResourceBindingMismatch { slot: actor_slot, bound: key, loaded: database.key() });
    }
    catalog.spec(key).map_err(|_| ActorRenderError::InvalidResourceKey(key))?;
    let fields = parse_actor_source_fields(actor_slot, record, key)?;

    let frame_record = database.frame(usize::from(fields.animation_index), usize::from(fields.frame_index))?;
    let mut components = Vec::new();
    components.try_reserve_exact(usize::from(frame_record.component_count)).map_err(|_| ActorRenderError::AllocationFailed)?;
    for component in 0..usize::from(frame_record.component_count) {
        components.push(database.component(usize::from(fields.animation_index), usize::from(fields.frame_index), component)?);
    }

    Ok(ActorRenderRecord {
        actor_slot,
        actor_id: fields.actor_id,
        owner: fields.owner,
        blueprint: expected_blueprint,
        resource_key: fields.resource_key,
        world_position: fields.world_position,
        virtual_height: fields.virtual_height,
        shade: fields.shade,
        actor_flags_15: fields.actor_flags_15,
        animation_index: fields.animation_index,
        frame_index: fields.frame_index,
        draw_flags_55: fields.draw_flags_55,
        draw_flag_58: fields.draw_flag_58,
        draw_flag_59: fields.draw_flag_59,
        draw_flags_70: fields.draw_flags_70,
        draw_flag_7e: fields.draw_flag_7e,
        draw_flag_82: fields.draw_flag_82,
        draw_mode_73: fields.draw_mode_73,
        frame_record,
        components,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_fields_keep_signed_extremes_and_reject_bad_records() {
        let key = ActorResourceTableKey { style: 0, category: 0, flat_index: 0 };
        let mut bytes = [0u8; NATIVE_ACTOR_BYTES];
        bytes[ACTOR_TYPE] = 3;
        bytes[ACTOR_ID..ACTOR_ID + 2].copy_from_slice(&7u16.to_le_bytes());
        bytes[OWNER] = 2;
        bytes[POSITION_X..POSITION_X + 2].copy_from_slice(&i16::MIN.to_le_bytes());
        bytes[POSITION_Y..POSITION_Y + 2].copy_from_slice(&i16::MAX.to_le_bytes());
        bytes[0x27..0x2b].copy_from_slice(&i32::MIN.to_le_bytes());
        bytes[0x11..0x15].copy_from_slice(&i32::MAX.to_le_bytes());
        bytes[SHADE] = 31;
        bytes[FLAGS_15] = 0x12;
        bytes[ANIMATION] = 4;
        bytes[FRAME] = 9;
        bytes[DRAW_FLAGS] = 0xa5;
        bytes[DRAW_MODE] = 0xfe;
        let fields = parse_actor_source_fields(7, &bytes, key).unwrap();
        assert_eq!(fields.world_position, [i16::MIN, i16::MAX]);
        assert_eq!(fields.virtual_height, 1); // source SUB dword wraps at 32 bits
        assert_eq!((fields.actor_id, fields.owner, fields.resource_key), (7, 2, key));
        assert_eq!((fields.shade, fields.actor_flags_15, fields.animation_index, fields.frame_index), (31, 0x12, 4, 9));
        assert_eq!((fields.draw_flags_55, fields.draw_mode_73), (0xa5, 0xfe));
        assert!(matches!(parse_actor_source_fields(7, &bytes[..199], key), Err(ActorRenderError::RecordLength { .. })));
        bytes[ACTOR_TYPE] = 2;
        assert!(matches!(parse_actor_source_fields(7, &bytes, key), Err(ActorRenderError::WrongActorType { .. })));
        bytes[ACTOR_TYPE] = 3;
        assert!(matches!(parse_actor_source_fields(8, &bytes, key), Err(ActorRenderError::ActorIdMismatch { .. })));
        bytes[OWNER] = 4;
        assert!(matches!(parse_actor_source_fields(7, &bytes, key), Err(ActorRenderError::OwnerOutOfRange { .. })));
    }

    #[test]
    fn draw_policy_resolves_special_visibility_flags_shade_and_wrapper_height() {
        let component = AnbComponentRecord { sprite_offset: 0, x_raw: 0, y_raw: 0, flags: 0, extra: 0 };
        let mut record = draw_record(1, 0, 0x10, vec![component]);
        record.actor_id = 3;
        record.draw_mode_73 = 224;
        record.draw_flags_55 = 1;
        record.draw_flags_70 = 4;
        let hidden = plan_actor_draw_fields(&record, 3, 0, 0, 0, [20, 10], 128, ActorWrapperKind::World).unwrap();
        assert!(!hidden.visible); // owner mismatch, despite even parity
        assert_eq!(hidden.context.global_flags, 5); // class flags 4, low flag 1, parity flag 4
        assert_eq!(hidden.context.origin, [20, 8]);
        assert_eq!(hidden.effective_shade, 1);
        assert_eq!(hidden.fade_row, Some(1));
        assert!(hidden.shadow_required);

        let direct = plan_actor_draw_fields(&record, 3, 0, 1, 1, [20, 10], 128, ActorWrapperKind::PortraitDirect).unwrap();
        assert!(direct.visible);
        assert_eq!(direct.context.origin, [20, 10]);
        assert!(!direct.shadow_required);

        record.draw_flags_55 = 0;
        record.draw_mode_73 = 0;
        record.actor_flags_15 = 0;
        record.draw_flag_58 = 1;
        record.draw_flag_7e = 4;
        record.draw_flags_70 = 4;
        let category = plan_actor_draw_fields(&record, 0, 0, 1, 0, [0, 0], 0, ActorWrapperKind::PortraitDirect).unwrap();
        assert_eq!(category.context.global_flags, 1);

        record.shade = 0;
        assert!(matches!(plan_actor_draw_fields(&record, 0, 0, 1, 0, [0, 0], -128, ActorWrapperKind::PortraitDirect), Err(ActorRenderError::FadeShadeOutOfRange(-1))));

        record.shade = 29;
        let raw29 = plan_actor_draw_fields(&record, 0, 0, 1, 0, [0, 0], 128, ActorWrapperKind::PortraitDirect).unwrap();
        assert_eq!(raw29.effective_shade, 30);
        assert_eq!(raw29.fade_row, Some(30)); // raw shade chooses fade, effective shade selects row
        record.shade = 30;
        let raw30 = plan_actor_draw_fields(&record, 0, 0, 1, 0, [0, 0], -128, ActorWrapperKind::PortraitDirect).unwrap();
        assert_eq!(raw30.effective_shade, 29);
        assert_eq!(raw30.fade_row, None); // raw shade chooses owner-only even when effective row is below 30
    }

    #[test]
    fn public_draw_policy_reads_the_blueprint_bound_to_the_actor() {
        let component = AnbComponentRecord { sprite_offset: 0, x_raw: 0, y_raw: 0, flags: 0, extra: 0 };
        let mut record = draw_record(0, 30, 0, vec![component]);
        record.draw_mode_73 = 225;
        let mut bytes = vec![0u8; crate::player_state::PLAYER_RECORD_BYTES];
        bytes[1] = 0;
        let start = 0x16;
        bytes[start + 0x15] = 3;
        bytes[start + 0x16] = 1;
        let player = PlayerState::from_record(&bytes).unwrap();
        let policy = plan_actor_draw(&record, &player, 0, 0, [11, 12], 0, ActorWrapperKind::PortraitDirect).unwrap();
        assert!(policy.visible);
        assert_eq!(policy.fade_row, None);
        assert_eq!(policy.effective_shade, 30);
        assert_eq!(policy.context.origin, [11, 12]);
        assert_eq!(policy.context.global_flags, 4);

        let other = PlayerState::from_record(&{ let mut b = bytes.clone(); b[1] = 1; b }).unwrap();
        assert!(matches!(plan_actor_draw(&record, &other, 0, 0, [0, 0], 0, ActorWrapperKind::PortraitDirect), Err(ActorRenderError::BlueprintPlayerMismatch { .. })));
    }

    #[test]
    fn resource_key_table_is_bounded_to_the_sixteen_proved_entries() {
        let catalog = ActorResourceCatalog;
        for spec in catalog.specs() {
            assert_eq!(catalog.spec(spec.key).unwrap(), spec);
        }
        assert!(catalog.spec(ActorResourceTableKey { style: 4, category: 0, flat_index: 16 }).is_err());
    }

    fn draw_record(owner: u8, shade: u8, actor_flags_15: u8, components: Vec<AnbComponentRecord>) -> ActorRenderRecord {
        ActorRenderRecord {
            actor_slot: 0, actor_id: 0, owner,
            blueprint: BlueprintRecordHandle { player_id: 0, blueprint_index: 0 },
            resource_key: ActorResourceTableKey { style: 0, category: 0, flat_index: 0 },
            world_position: [0, 0], virtual_height: 0, shade, actor_flags_15, animation_index: 0, frame_index: 0,
            draw_flags_55: 0, draw_flag_58: 0, draw_flag_59: 0, draw_flags_70: 0,
            draw_flag_7e: 0, draw_flag_82: 0, draw_mode_73: 0,
            frame_record: AnbFrameRecord { frame_offset: 0, component_offset: 0, component_count: components.len() as u8, width_raw: 0, height_raw: 0, byte_10: 0, trailing_bytes: [0; 2] },
            components,
        }
    }

    fn decoded(component: AnbComponentRecord, width: u8, height: u8, indices: &[u8], opaque: &[bool]) -> ActorDecodedComponent {
        ActorDecodedComponent { component, sprite: IndexedSprite { width, height, indices: indices.to_vec(), opaque: opaque.to_vec(), consumed_bytes: 0 } }
    }

    #[test]
    fn explicit_component_draw_recolors_opaque_zero_clips_and_preserves_pitch_padding() {
        let component = AnbComponentRecord { sprite_offset: 0, x_raw: 0, y_raw: 0, flags: 0, extra: 0 };
        let record = draw_record(1, 40, 0, vec![component]);
        let sprite = decoded(component, 2, 1, &[0xc0, 0xc0], &[true, true]);
        let mut framebuffer = [33u8; 7]; // Minimal addressed extent for 3 pixels, pitch 4, two rows.
        let receipt = draw_actor_components(&record, &[sprite], ActorDrawContext { origin: [-1, 1], virtual_height: 0, global_flags: 0 }, None, None, &mut framebuffer, 3, 2, 4).unwrap();
        assert_eq!(framebuffer, [33, 33, 33, 33, 208, 33, 33]);
        assert_eq!(receipt.pixels_written, 1);
        assert_eq!(receipt.pixels_clipped, 1);
        assert_eq!(receipt.opaque_pixels, 2);
    }

    #[test]
    fn extreme_host_origins_clip_without_overflow_or_touching_destination() {
        let component = AnbComponentRecord { sprite_offset: 0, x_raw: 0, y_raw: 0, flags: 0, extra: 0 };
        let record = draw_record(0, 40, 0x02, vec![component]);
        let sprite = decoded(component, 2, 2, &[1, 2, 3, 4], &[true; 4]);
        for origin in [[i32::MIN, i32::MIN], [i32::MAX, i32::MAX], [i32::MIN, i32::MAX], [i32::MAX, i32::MIN]] {
            let mut destination = [0xA5; 7];
            let receipt = draw_actor_components(
                &record, &[sprite.clone()], ActorDrawContext { origin, virtual_height: 0, global_flags: 0 },
                None, None, &mut destination, 3, 2, 4,
            ).unwrap();
            assert_eq!(destination, [0xA5; 7]);
            assert_eq!(receipt.pixels_written, 0);
            assert_eq!(receipt.pixels_clipped, 4);
            assert_eq!(receipt.opaque_pixels, 4);
        }
    }

    #[test]
    fn overlapping_blended_components_read_the_staged_prior_component() {
        let a = AnbComponentRecord { sprite_offset: 0, x_raw: 0, y_raw: 0, flags: 4, extra: 0 };
        let b = AnbComponentRecord { sprite_offset: 6, x_raw: 0, y_raw: 0, flags: 4, extra: 0 };
        let record = draw_record(0, 31, 0x02, vec![a, b]);
        let components = [decoded(a, 1, 1, &[10], &[true]), decoded(b, 1, 1, &[20], &[true])];
        let mut blend = vec![0u8; 65_536];
        for dest in 0..256 { for source in 0..256 { blend[dest * 256 + source] = (dest as u8).wrapping_add(source as u8); } }
        let mut framebuffer = [1u8; 1];
        let receipt = draw_actor_components(&record, &components, ActorDrawContext { origin: [0, 0], virtual_height: 0, global_flags: 0 }, None, Some(&blend), &mut framebuffer, 1, 1, 1).unwrap();
        assert_eq!(framebuffer, [31]);
        assert_eq!(receipt.pixels_written, 2);
    }

    #[test]
    fn component_flip_flags_change_source_orientation_without_changing_extent() {
        let horizontal = AnbComponentRecord { sprite_offset: 0, x_raw: 0, y_raw: 0, flags: 1, extra: 0 };
        let vertical = AnbComponentRecord { sprite_offset: 6, x_raw: 0, y_raw: 0, flags: 2, extra: 0 };
        let horizontal_record = draw_record(0, 31, 0x02, vec![horizontal]);
        let vertical_record = draw_record(0, 31, 0x02, vec![vertical]);
        let h = decoded(horizontal, 2, 1, &[10, 20], &[true, true]);
        let v = decoded(vertical, 1, 2, &[10, 20], &[true, true]);
        let mut hframe = [0u8; 2];
        let mut vframe = [0u8; 2];
        draw_actor_components(&horizontal_record, &[h], ActorDrawContext { origin: [0, 0], virtual_height: 0, global_flags: 0 }, None, None, &mut hframe, 2, 1, 2).unwrap();
        draw_actor_components(&vertical_record, &[v], ActorDrawContext { origin: [0, 0], virtual_height: 0, global_flags: 0 }, None, None, &mut vframe, 1, 2, 1).unwrap();
        assert_eq!(hframe, [20, 10]);
        assert_eq!(vframe, [20, 10]);
    }

    #[test]
    fn late_unsupported_component_is_atomic_and_bad_fade_extent_rejects() {
        let good = AnbComponentRecord { sprite_offset: 0, x_raw: 0, y_raw: 0, flags: 0, extra: 0 };
        let bad = AnbComponentRecord { sprite_offset: 6, x_raw: 0, y_raw: 0, flags: 0x80, extra: 0 };
        let record = draw_record(0, 1, 0x02, vec![good, bad]);
        let components = [decoded(good, 1, 1, &[9], &[true]), decoded(bad, 1, 1, &[10], &[true])];
        let mut framebuffer = [77u8; 4];
        assert!(matches!(draw_actor_components(&record, &components, ActorDrawContext { origin: [0, 0], virtual_height: 0, global_flags: 0 }, None, None, &mut framebuffer, 1, 1, 4), Err(ActorRenderError::InvalidDrawFlags(0x80))));
        assert_eq!(framebuffer, [77; 4]);

        let fade_record = draw_record(0, 1, 0, vec![good]);
        assert!(matches!(draw_actor_components(&fade_record, &[decoded(good, 1, 1, &[9], &[true])], ActorDrawContext { origin: [0, 0], virtual_height: 0, global_flags: 0 }, Some(&[0; 255]), None, &mut framebuffer, 1, 1, 4), Err(ActorRenderError::FadeRowLength { actual: 255 })));
        assert_eq!(framebuffer, [77; 4]);
    }
}
