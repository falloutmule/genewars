//! Explicit source-backed family-4 effect drawing for the admitted EFF.ANB
//! paths. Source pin: `research/ghidra/effect-draw-contract.json`. This module
//! does not decide cross-family ordering or update timing.

use std::collections::HashSet;

use genewars_assets::{AnbFrameRecord, AnbSections, IndexedSprite, SpriteTable};
use genewars_render::{
    entity_shadow::draw_normal_component_shadow,
    terrain::TerrainFadeTable,
    terrain_modes::MainGlaTable,
};
use genewars_runtime::{
    effect_pool::{EffectAnimationBinding, EffectPool},
    landing::LandingRecord,
};

use super::runtime_resources::Resources;

const EFFECT_ACTIVE: u8 = 5;
const CODE_31: u8 = 0x31;
const CODE_0X30: u8 = 0x30;
const CODE_6: u8 = 6;
const CODE_47: u8 = 0x2f;
const CODE_26: u8 = 0x1a;
const CODE_15: u8 = 0x15;
const CODE_16: u8 = 0x16;
const CODE_2C: u8 = 0x2c;
const SPECIAL_ANIMATION: usize = 66;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectSceneError {
    InvalidClip([usize; 2]),
    InvalidPitch { width: usize, pitch: usize },
    DestinationTooShort { required: usize, actual: usize },
    ExtentOverflow,
    AllocationFailed,
    DuplicateSlot(usize),
    SlotOutOfRange(usize),
    InactiveSlot(usize),
    InvalidBinding { slot: usize, code: u8, binding: Option<EffectAnimationBinding> },
    UnsupportedInitialGlobalFlags(u8),
    UnsupportedAnimationField { slot: usize, value: u8 },
    UnsupportedComponentFlags { slot: usize, frame: usize, component: usize, flags: u8 },
    MissingFadeRow { slot: usize, row: i32 },
    Shadow(String),
    Resource(String),
    PixelCountOverflow,
}

impl core::fmt::Display for EffectSceneError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for EffectSceneError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectDrawReceipt {
    pub slot: usize,
    pub code: u8,
    pub binding: EffectAnimationBinding,
    pub animation_frame: u8,
    pub ground_origin: [i16; 2],
    pub color_origin: [i32; 2],
    pub virtual_height: i32,
    pub effective_shade: i32,
    pub special_pixels_written: usize,
    pub special_pixels_clipped: usize,
    pub shadow_pixels_written: usize,
    pub shadow_pixels_clipped: usize,
    pub color_pixels_written: usize,
    pub color_pixels_clipped: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectSceneReceipt {
    pub items: Vec<EffectDrawReceipt>,
    pub special_pixels_written: usize,
    pub special_pixels_clipped: usize,
    pub shadow_pixels_written: usize,
    pub color_pixels_written: usize,
    pub pixels_clipped: usize,
}

#[derive(Debug, Clone)]
struct DecodedComponent {
    sprite: IndexedSprite,
    local: [i16; 2],
    flags: u8,
}

#[derive(Debug, Clone)]
struct DecodedFrame {
    frame_offset: u32,
    components: Vec<DecodedComponent>,
}

/// Draw active family-4 records in the supplied order. Coordinates are
/// caller-projected ground origins; this helper applies the proved per-effect
/// vertical displacement and does not infer family-4 admission or global draw
/// order. A failure leaves `destination` unchanged.
pub fn draw_effect_slots(
    resources: &Resources,
    pool: &EffectPool,
    slots_in_draw_order: &[usize],
    ground_origins: &[[i16; 2]],
    frame_counter: u8,
    initial_global_flags: u8,
    destination: &mut [u8],
    clip: [usize; 2],
    pitch: usize,
) -> Result<EffectSceneReceipt, EffectSceneError> {
    let mut bindings = Vec::with_capacity(pool.records().len());
    for slot in 0..pool.records().len() {
        bindings.push(pool.binding(slot).map_err(|_| EffectSceneError::SlotOutOfRange(slot))?);
    }
    draw_effect_records_inner(
        resources, pool.records(), &bindings, slots_in_draw_order, ground_origins,
        frame_counter, initial_global_flags, destination, clip, pitch,
    )
}

fn draw_effect_records_inner(
    resources: &Resources,
    records: &[LandingRecord],
    bindings: &[Option<EffectAnimationBinding>],
    slots_in_draw_order: &[usize],
    ground_origins: &[[i16; 2]],
    frame_counter: u8,
    initial_global_flags: u8,
    destination: &mut [u8],
    clip: [usize; 2],
    pitch: usize,
) -> Result<EffectSceneReceipt, EffectSceneError> {
    let [width, height] = clip;
    if !(1..=640).contains(&width) || !(1..=480).contains(&height) {
        return Err(EffectSceneError::InvalidClip(clip));
    }
    if pitch < width {
        return Err(EffectSceneError::InvalidPitch { width, pitch });
    }
    let required = (height - 1).checked_mul(pitch).and_then(|v| v.checked_add(width))
        .ok_or(EffectSceneError::ExtentOverflow)?;
    if destination.len() < required {
        return Err(EffectSceneError::DestinationTooShort { required, actual: destination.len() });
    }
    // The traced wrappers' local path starts with no inherited transform or
    // blend bits. Keep other 19CFAC contexts out of this bounded compositor.
    if initial_global_flags != 0 {
        return Err(EffectSceneError::UnsupportedInitialGlobalFlags(initial_global_flags));
    }

    let mut seen = HashSet::with_capacity(slots_in_draw_order.len());
    let mut staged = Vec::new();
    staged.try_reserve_exact(required).map_err(|_| EffectSceneError::AllocationFailed)?;
    staged.extend_from_slice(&destination[..required]);
    let mut receipt = EffectSceneReceipt {
        items: Vec::with_capacity(slots_in_draw_order.len()),
        special_pixels_written: 0,
        special_pixels_clipped: 0,
        shadow_pixels_written: 0,
        color_pixels_written: 0,
        pixels_clipped: 0,
    };
    let fade = TerrainFadeTable::parse(resources.fade_bytes()).map_err(|e| EffectSceneError::Resource(e.to_string()))?;
    let gla = MainGlaTable::parse(resources.main_gla_bytes()).map_err(|e| EffectSceneError::Resource(e.to_string()))?;

    for &slot in slots_in_draw_order {
        if !seen.insert(slot) { return Err(EffectSceneError::DuplicateSlot(slot)); }
        let record = records.get(slot).ok_or(EffectSceneError::SlotOutOfRange(slot))?;
        let binding = *bindings.get(slot).ok_or(EffectSceneError::SlotOutOfRange(slot))?;
        let ground_origin = *ground_origins.get(slot).ok_or(EffectSceneError::SlotOutOfRange(slot))?;
        if record[6] != EFFECT_ACTIVE { return Err(EffectSceneError::InactiveSlot(slot)); }
        let code = record[69];
        let expected_binding = match code {
            CODE_31 => Some(EffectAnimationBinding::EffAnimation49),
            CODE_6 => Some(EffectAnimationBinding::EffAnimation6),
            CODE_0X30 => Some(EffectAnimationBinding::EffAnimation48),
            CODE_47 => Some(EffectAnimationBinding::EffAnimation47),
            CODE_26 => Some(EffectAnimationBinding::EffAnimation26),
            CODE_15 => Some(EffectAnimationBinding::EffAnimation21),
            CODE_16 => Some(EffectAnimationBinding::EffAnimation22),
            CODE_2C => Some(EffectAnimationBinding::EffAnimation44),
            _ => None,
        };
        if expected_binding.is_none() || binding != expected_binding {
            return Err(EffectSceneError::InvalidBinding { slot, code, binding });
        }
        // The typed binding selects the code-specific animation base. The
        // object-local animation byte is still expected to be its proven base
        // selector (zero) for these bound EFF entries.
        if record[0x19] != 0 {
            return Err(EffectSceneError::UnsupportedAnimationField { slot, value: record[0x19] });
        }
        let binding = binding.expect("validated effect binding");
        let animation_frame = record[0x1b];
        let main = decode_bound_frame(resources.effects.bytes(), binding, usize::from(animation_frame))
            .map_err(EffectSceneError::Resource)?;
        if code == CODE_47 {
            for (component, value) in main.components.iter().enumerate() {
                if value.flags != 0 {
                    return Err(EffectSceneError::UnsupportedComponentFlags {
                        slot, frame: main.frame_offset as usize, component, flags: value.flags,
                    });
                }
            }
        } else {
            validate_frame_flags(slot, &main, false)?;
        }

        let virtual_height = read_i32(record, 0x27).wrapping_sub(read_i32(record, 0x11));
        let dy = virtual_height >> 6;
        let color_origin = if code == CODE_47 {
            // The code2F wrapper subtracts height from its signed-16 input in
            // a 32-bit register and does not narrow the resulting Y coordinate.
            [i32::from(ground_origin[0]), i32::from(ground_origin[1]).wrapping_sub(dy)]
        } else {
            [i32::from(ground_origin[0]), i32::from((i32::from(ground_origin[1]).wrapping_sub(dy)) as i16)]
        };
        let raw_shade = record[0x17];
        let effective_shade = if code == CODE_47 {
            i32::from(raw_shade).wrapping_add(virtual_height >> 7)
        } else if raw_shade == 0 { 0 } else {
            i32::from(raw_shade).wrapping_add(virtual_height >> 7)
        };
        let direct = if code == CODE_47 { effective_shade >= 30 }
            else { record[0x15] & 0x02 != 0 || effective_shade >= 30 };
        let fade_row = if direct { None } else {
            if !(0..64).contains(&effective_shade) {
                return Err(EffectSceneError::MissingFadeRow { slot, row: effective_shade });
            }
            Some(effective_shade as u8)
        };
        let parity_blend = code != CODE_47 && record[0x15] & 0x10 != 0
            && frame_counter.wrapping_add(record[0x0f]) & 1 != 0;

        let mut item = EffectDrawReceipt {
            slot, code, binding, animation_frame, ground_origin, color_origin,
            virtual_height, effective_shade,
            special_pixels_written: 0, shadow_pixels_written: 0,
            special_pixels_clipped: 0,
            shadow_pixels_clipped: 0, color_pixels_written: 0, color_pixels_clipped: 0,
        };

        if code == CODE_31 {
            let special = decode_frame(resources.effects.bytes(), SPECIAL_ANIMATION, 0)
                .map_err(EffectSceneError::Resource)?;
            validate_frame_flags(slot, &special, true)?;
            let (written, clipped) = draw_components(
                &special, ground_origin, Some(0x0c), None, Some(&gla),
                &mut staged, width, height, pitch,
            )?;
            item.special_pixels_written = written;
            item.special_pixels_clipped = clipped;
        } else if code == CODE_6 || code == CODE_0X30 || code == CODE_26
            || code == CODE_15 || code == CODE_16 || code == CODE_2C {
            // These ordinary wrappers request the mode-0 component-shadow
            // pass before indexed8 color. Their EFF animation ranges differ.
            for component in &main.components {
                let shadow = draw_normal_component_shadow(
                    &component.sprite, &gla, ground_origin, component.local, 0,
                    clip, pitch, &mut staged,
                ).map_err(|e| EffectSceneError::Shadow(e.to_string()))?;
                item.shadow_pixels_written = item.shadow_pixels_written.checked_add(shadow.pixels_written)
                    .ok_or(EffectSceneError::PixelCountOverflow)?;
                item.shadow_pixels_clipped = item.shadow_pixels_clipped.checked_add(shadow.clipped_opaque_samples)
                    .ok_or(EffectSceneError::PixelCountOverflow)?;
            }
        }

        let base_global = if parity_blend { 0x04 } else { 0 };
        let owner_recolor_bias = (code == CODE_47)
            .then(|| record[0x18].wrapping_mul(16).wrapping_sub(64));
        let (written, clipped) = draw_components_color(
            &main, color_origin, base_global, fade_row, owner_recolor_bias, &fade, &gla,
            &mut staged, width, height, pitch,
        )?;
        item.color_pixels_written = written;
        item.color_pixels_clipped = clipped;
        receipt.special_pixels_written = receipt.special_pixels_written.checked_add(item.special_pixels_written)
            .ok_or(EffectSceneError::PixelCountOverflow)?;
        receipt.special_pixels_clipped = receipt.special_pixels_clipped.checked_add(item.special_pixels_clipped)
            .ok_or(EffectSceneError::PixelCountOverflow)?;
        receipt.shadow_pixels_written = receipt.shadow_pixels_written.checked_add(item.shadow_pixels_written)
            .ok_or(EffectSceneError::PixelCountOverflow)?;
        receipt.color_pixels_written = receipt.color_pixels_written.checked_add(item.color_pixels_written)
            .ok_or(EffectSceneError::PixelCountOverflow)?;
        receipt.pixels_clipped = receipt.pixels_clipped.checked_add(item.shadow_pixels_clipped)
            .and_then(|v| v.checked_add(item.color_pixels_clipped))
            .and_then(|v| v.checked_add(item.special_pixels_clipped))
            .ok_or(EffectSceneError::PixelCountOverflow)?;
        receipt.items.push(item);
    }

    destination[..required].copy_from_slice(&staged);
    Ok(receipt)
}

fn decode_bound_frame(
    anb: &[u8], binding: EffectAnimationBinding, relative_frame: usize,
) -> Result<DecodedFrame, String> {
    let animation = match binding {
        EffectAnimationBinding::EffAnimation49 => 49,
        EffectAnimationBinding::EffAnimation6 => 6,
        EffectAnimationBinding::EffAnimation48 => 48,
        EffectAnimationBinding::EffAnimation47 => 47,
        EffectAnimationBinding::EffAnimation26 => 26,
        EffectAnimationBinding::EffAnimation21 => 21,
        EffectAnimationBinding::EffAnimation22 => 22,
        EffectAnimationBinding::EffAnimation44 => 44,
    };
    decode_frame(anb, animation, relative_frame)
}

fn decode_frame(anb: &[u8], animation_index: usize, relative_frame: usize) -> Result<DecodedFrame, String> {
    let sections = AnbSections::parse(anb).map_err(|e| e.to_string())?;
    let metadata = sections.ui_metadata().map_err(|e| e.to_string())?;
    let animation = metadata.animation(animation_index).map_err(|e| e.to_string())?;
    let span = animation.last_frame_offset.checked_sub(animation.first_frame_offset)
        .filter(|span| span % 14 == 0).ok_or("invalid EFF animation frame range")?;
    let count = usize::try_from(span / 14 + 1).map_err(|e| e.to_string())?;
    if relative_frame >= count { return Err(format!("EFF animation {animation_index} frame {relative_frame} exceeds {count}")); }
    let offset = animation.first_frame_offset.checked_add(u32::try_from(relative_frame).map_err(|e| e.to_string())?.checked_mul(14).ok_or("frame offset overflow")?)
        .ok_or("frame offset overflow")?;
    let frame = metadata.frame_at(offset).map_err(|e| e.to_string())?;
    decode_frame_record(anb, &metadata, frame)
}

fn decode_frame_record(
    anb: &[u8], metadata: &genewars_assets::AnbUiMetadata<'_>, frame: AnbFrameRecord,
) -> Result<DecodedFrame, String> {
    let table = SpriteTable::from_anb(anb).map_err(|e| e.to_string())?;
    let mut components = Vec::with_capacity(usize::from(frame.component_count));
    for index in 0..usize::from(frame.component_count) {
        let component = metadata.frame_component(frame, index).map_err(|e| e.to_string())?;
        let sprite_index = usize::try_from(component.sprite_offset / 6).map_err(|e| e.to_string())?;
        let sprite = table.decode(sprite_index, anb).map_err(|e| e.to_string())?;
        let (x, y) = component.local_position();
        components.push(DecodedComponent { sprite, local: [x, y], flags: component.flags });
    }
    Ok(DecodedFrame { frame_offset: frame.frame_offset, components })
}

fn validate_frame_flags(slot: usize, frame: &DecodedFrame, special: bool) -> Result<(), EffectSceneError> {
    for (index, component) in frame.components.iter().enumerate() {
        let valid = if special { component.flags == 0x04 }
            else { matches!(component.flags, 0 | 0x04 | 0x84) };
        if !valid {
            return Err(EffectSceneError::UnsupportedComponentFlags {
                slot, frame: frame.frame_offset as usize, component: index, flags: component.flags,
            });
        }
    }
    Ok(())
}

fn draw_components(
    frame: &DecodedFrame, origin: [i16; 2], wrapper_flags: Option<u8>, fade_row: Option<u8>,
    gla: Option<&MainGlaTable<'_>>, destination: &mut [u8], width: usize, height: usize, pitch: usize,
) -> Result<(usize, usize), EffectSceneError> {
    let mut written = 0usize;
    let mut clipped = 0usize;
    for component in &frame.components {
        let global = wrapper_flags.unwrap_or(0);
        let (count, clipped_part) = draw_component(
            component, origin, global, fade_row, None, gla, destination, width, height, pitch,
        )?;
        written = written.checked_add(count).ok_or(EffectSceneError::PixelCountOverflow)?;
        clipped = clipped.checked_add(clipped_part).ok_or(EffectSceneError::PixelCountOverflow)?;
    }
    Ok((written, clipped))
}

fn draw_components_color(
    frame: &DecodedFrame, origin: [i32; 2], wrapper_flags: u8, fade_row: Option<u8>, owner_recolor_bias: Option<u8>,
    fade: &TerrainFadeTable<'_>, gla: &MainGlaTable<'_>, destination: &mut [u8],
    width: usize, height: usize, pitch: usize,
) -> Result<(usize, usize), EffectSceneError> {
    let mut written = 0usize;
    let mut clipped = 0usize;
    for component in &frame.components {
        let (drawn, clipped_part) = draw_component_with_recolor(
            component, origin, wrapper_flags, fade_row, Some(fade), Some(gla),
            owner_recolor_bias, destination, width, height, pitch,
        )?;
        written = written.checked_add(drawn).ok_or(EffectSceneError::PixelCountOverflow)?;
        clipped = clipped.checked_add(clipped_part).ok_or(EffectSceneError::PixelCountOverflow)?;
    }
    Ok((written, clipped))
}

fn draw_component(
    component: &DecodedComponent, origin: [i16; 2], wrapper_flags: u8, fade_row: Option<u8>,
    fade: Option<&TerrainFadeTable<'_>>, gla: Option<&MainGlaTable<'_>>,
    destination: &mut [u8], width: usize, height: usize, pitch: usize,
) -> Result<(usize, usize), EffectSceneError> {
    draw_component_with_recolor(component, [i32::from(origin[0]), i32::from(origin[1])], wrapper_flags,
        fade_row, fade, gla, None, destination, width, height, pitch)
}

fn draw_component_with_recolor(
    component: &DecodedComponent, origin: [i32; 2], wrapper_flags: u8, fade_row: Option<u8>,
    fade: Option<&TerrainFadeTable<'_>>, gla: Option<&MainGlaTable<'_>>, owner_recolor_bias: Option<u8>,
    destination: &mut [u8], width: usize, height: usize, pitch: usize,
) -> Result<(usize, usize), EffectSceneError> {
    let sprite = &component.sprite;
    let pixels = usize::from(sprite.width).checked_mul(usize::from(sprite.height))
        .ok_or(EffectSceneError::ExtentOverflow)?;
    if sprite.indices.len() != pixels || sprite.opaque.len() != pixels {
        return Err(EffectSceneError::Resource("decoded EFF sprite planes have inconsistent lengths".into()));
    }
    let flags = wrapper_flags | component.flags;
    let hflip = flags & 1 != 0;
    let vflip = flags & 2 != 0;
    let blend_order = flags & 0x0c;
    let offset_x = if hflip { -i64::from(sprite.width) - i64::from(component.local[0]) }
        else { i64::from(component.local[0]) };
    let offset_y = if vflip { -i64::from(sprite.height) - i64::from(component.local[1]) }
        else { i64::from(component.local[1]) };
    let left = i64::from(origin[0]) + offset_x;
    let top = i64::from(origin[1]) + offset_y;
    let mut written = 0usize;
    let mut clipped = 0usize;
    for dy in 0..usize::from(sprite.height) {
        let py = top + dy as i64;
        for dx in 0..usize::from(sprite.width) {
            let sx = if hflip { usize::from(sprite.width) - 1 - dx } else { dx };
            let sy = if vflip { usize::from(sprite.height) - 1 - dy } else { dy };
            let source = sy * usize::from(sprite.width) + sx;
            if !sprite.opaque[source] { continue; }
            let px = left + dx as i64;
            if px < 0 || py < 0 || px >= width as i64 || py >= height as i64 {
                clipped = clipped.checked_add(1).ok_or(EffectSceneError::PixelCountOverflow)?;
                continue;
            }
            let offset = py as usize * pitch + px as usize;
            let mut color = sprite.indices[source];
            if let Some(bias) = owner_recolor_bias {
                if color & 0xf0 == 0xc0 { color = bias.wrapping_add(color & 0x0f); }
            }
            if let Some(row) = fade_row {
                let fade = fade.ok_or_else(|| EffectSceneError::Resource("fade row selected without FADE.DAT".into()))?;
                color = fade.map_palette_index(row, color).map_err(|e| EffectSceneError::Resource(e.to_string()))?;
            }
            if blend_order != 0 {
                let gla = gla.ok_or_else(|| EffectSceneError::Resource("blend mode selected without MAIN.GLA".into()))?;
                let old = destination[offset];
                color = match blend_order {
                    4 => gla.map_destination(old, color),
                    8 | 12 => gla.map_destination(color, old),
                    _ => return Err(EffectSceneError::Resource(format!("unsupported EFF blend flags {blend_order:#x}"))),
                };
            }
            destination[offset] = color;
            written = written.checked_add(1).ok_or(EffectSceneError::PixelCountOverflow)?;
        }
    }
    Ok((written, clipped))
}

fn read_i32(bytes: &LandingRecord, offset: usize) -> i32 {
    i32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("fixed record range"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use genewars_assets::IndexedSprite;
    use genewars_render::terrain_modes::MAIN_GLA_TABLE_BYTES;

    fn sprite(index: u8, opaque: bool) -> DecodedComponent {
        DecodedComponent { sprite: IndexedSprite { width: 1, height: 1, indices: vec![index], opaque: vec![opaque], consumed_bytes: 0 }, local: [0,0], flags: 0 }
    }

    fn gla() -> MainGlaTable<'static> {
        let mut bytes = vec![0; MAIN_GLA_TABLE_BYTES];
        for destination in 0..256usize {
            for source in 0..256usize {
                bytes[destination * 256 + source] = (destination as u8).wrapping_mul(3).wrapping_add(source as u8);
            }
        }
        MainGlaTable::parse(Box::leak(bytes.into_boxed_slice())).unwrap()
    }

    #[test]
    fn special_code31_mode12_uses_source_high_lookup_and_preserves_transparency() {
        let table = gla();
        let frame = DecodedFrame { frame_offset: 4634, components: vec![{
            let mut c = sprite(5, true); c.flags = 4; c
        }, sprite(7, false)] };
        let mut dst = [11u8, 13, 17, 19];
        let (written, clipped) = draw_components(&frame, [0,0], Some(0x0c), None, Some(&table), &mut dst, 2, 1, 4).unwrap();
        assert_eq!((written,clipped),(1,0));
        assert_eq!(dst, [5u8.wrapping_mul(3).wrapping_add(11),13,17,19]);
    }

    #[test]
    fn direct_fade_and_parity_blend_order_are_explicit() {
        let table = gla();
        let component = sprite(9, true);
        let mut dst = [10u8];
        let (w, c) = draw_component(&component, [0,0], 0, None, None, None, &mut dst, 1, 1, 1).unwrap();
        assert_eq!((w,c,dst[0]),(1,0,9));
        let mut mode4 = [10u8];
        draw_component(&component, [0,0], 4, None, None, Some(&table), &mut mode4, 1, 1, 1).unwrap();
        assert_eq!(mode4[0], 10u8.wrapping_mul(3).wrapping_add(9));
        let mut mode8 = [10u8];
        draw_component(&component, [0,0], 8, None, None, Some(&table), &mut mode8, 1, 1, 1).unwrap();
        assert_eq!(mode8[0], 9u8.wrapping_mul(3).wrapping_add(10));
    }

    #[test]
    fn empty_transparent_component_is_noop_and_extreme_origins_clip_safely() {
        let component = sprite(0, false);
        let mut dst = [23u8; 6];
        let before = dst;
        let (written, clipped) = draw_component(&component, [i16::MIN, i16::MAX], 0, None, None, None, &mut dst, 2, 2, 3).unwrap();
        assert_eq!((written,clipped),(0,0));
        assert_eq!(dst,before);
    }

    #[test]
    fn pinned_eff_frames_match_the_supported_effect_component_contract() {
        use std::{fs, path::PathBuf};
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .join("private/integration/game-runtime/effects/EFF.ANB");
        let bytes = fs::read(path).unwrap();
        assert_eq!(genewars_runtime::identity::sha256_hex(&bytes),
            "f45f1201e50ce67199f1e1e8448d250f221cad71f1328ee44b7bacf9ee62c253");
        let frame49 = decode_frame(&bytes, 49, 0).unwrap();
        assert_eq!(frame49.frame_offset, 3486);
        assert_eq!(frame49.components.len(), 2);
        assert!(frame49.components.iter().all(|c| c.flags == 0));
        let frame6 = decode_frame(&bytes, 6, 0).unwrap();
        assert_eq!(frame6.frame_offset, 854);
        assert_eq!(frame6.components.len(), 1);
        assert_eq!(frame6.components[0].flags, 0);
        let late6 = decode_frame(&bytes, 6, 7).unwrap();
        assert_eq!(late6.components[0].flags, 0x84);
        let late6b = decode_frame(&bytes, 6, 10).unwrap();
        assert_eq!(late6b.components[0].flags, 0x04);
        let water = decode_frame(&bytes, 48, 0).unwrap();
        assert_eq!(water.frame_offset, 3318);
        assert_eq!(water.components.len(), 1);
        assert_eq!(water.components[0].flags, 0);
        for frame_index in 0..12 {
            let frame = decode_frame(&bytes, 48, frame_index).unwrap();
            assert_eq!(frame.frame_offset, 3318 + (frame_index * 14) as u32);
            assert!(frame.components.iter().all(|c| c.flags == 0));
        }
        let special = decode_frame(&bytes, SPECIAL_ANIMATION, 0).unwrap();
        assert_eq!(special.frame_offset, 4634);
        assert_eq!(special.components.len(), 1);
        assert_eq!(special.components[0].flags, 0x04);
        for frame_index in 0..6 {
            let frame47 = decode_frame(&bytes, 47, frame_index).unwrap();
            assert_eq!(frame47.frame_offset, 3234 + (frame_index * 14) as u32);
            assert!(frame47.components.iter().all(|c| c.flags == 0));
        }
        let animation26 = AnbSections::parse(&bytes).unwrap().ui_metadata().unwrap().animation(26).unwrap();
        assert_eq!((animation26.first_frame_offset, animation26.last_frame_offset, animation26.trailing_word), (2030, 2086, 5));
        for frame_index in 0..5 {
            let frame26 = decode_frame(&bytes, 26, frame_index).unwrap();
            assert_eq!(frame26.frame_offset, 2030 + (frame_index * 14) as u32);
            assert_eq!(AnbSections::parse(&bytes).unwrap().ui_metadata().unwrap()
                .frame_at(frame26.frame_offset).unwrap().byte_10, 1);
            assert_eq!(frame26.components[0].flags, [0, 0, 0, 0x84, 0x04][frame_index]);
        }
        for (index, first, last, trailing, count) in [
            (21usize, 1890u32, 1904u32, 2u16, 2usize),
            (22, 1918, 1946, 3, 3),
            (44, 3024, 3136, 9, 9),
        ] {
            let metadata = AnbSections::parse(&bytes).unwrap().ui_metadata().unwrap();
            let animation = metadata.animation(index).unwrap();
            assert_eq!((animation.first_frame_offset,animation.last_frame_offset,animation.trailing_word),(first,last,trailing));
            for frame_index in 0..count {
                let frame = decode_frame(&bytes,index,frame_index).unwrap();
                assert_eq!(frame.frame_offset,first+(frame_index*14) as u32);
                let record=metadata.frame_at(frame.frame_offset).unwrap();
                assert_eq!(record.byte_10,0);
                assert!(!frame.components.is_empty());
                assert!(frame.components.iter().all(|component|component.flags==0));
            }
        }
        validate_frame_flags(0, &late6, false).unwrap();
        validate_frame_flags(0, &special, true).unwrap();
    }

    fn staged_resources() -> Resources {
        use std::path::PathBuf;
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .join("private/integration/game-runtime");
        Resources::load(&root).unwrap().0
    }

    fn active_effect(code: u8, frame: u8, shade: u8) -> LandingRecord {
        let mut record = [0u8; 81];
        record[6] = EFFECT_ACTIVE;
        record[0x19] = 0;
        record[0x1b] = frame;
        record[0x17] = shade;
        record[0x45] = code;
        record
    }

    #[test]
    fn actual_code6_and31_frames_draw_with_their_separate_layers() {
        let resources = staged_resources();
        let records = [active_effect(CODE_31,0,0),active_effect(CODE_6,0,0)];
        let bindings = [Some(EffectAnimationBinding::EffAnimation49),Some(EffectAnimationBinding::EffAnimation6)];
        let origins = [[180,160],[260,200]];
        let mut destination = vec![23u8;512*480];
        let receipt = draw_effect_records_inner(
            &resources,&records,&bindings,&[0,1],&origins,0,0,&mut destination,[512,480],512,
        ).unwrap();
        assert_eq!(receipt.items.len(),2);
        assert_eq!(receipt.items[0].special_pixels_written, receipt.special_pixels_written);
        assert_eq!(receipt.items[1].shadow_pixels_written, receipt.shadow_pixels_written);
        assert!(receipt.special_pixels_written > 0);
        assert!(receipt.shadow_pixels_written > 0);
        assert!(receipt.color_pixels_written > 0);
        assert!(destination.iter().any(|&pixel| pixel != 23));
    }

    #[test]
    fn actual_code47_recolors_then_fades_and_keeps_i32_vertical_origin() {
        let resources = staged_resources();
        let anb = resources.effects.bytes();
        let frame_index = (0..6).find(|&index| {
            decode_frame(anb, 47, index).unwrap().components.iter().any(|component|
                component.sprite.indices.iter().zip(&component.sprite.opaque)
                    .any(|(&pixel, &opaque)| opaque && pixel & 0xf0 == 0xc0))
        }).expect("pinned EFF47 frames should exercise the owner-recolor branch");
        let main = decode_frame(anb, 47, frame_index).unwrap();
        let mut record = active_effect(CODE_47, frame_index as u8, 3);
        record[0x18] = 2;
        record[0x15] = 0x02; // Code2F ignores the ordinary direct-color bit.
        let records = [record];
        let bindings = [Some(EffectAnimationBinding::EffAnimation47)];
        let origins = [[210i16, 170i16]];
        let mut actual = vec![23u8; 512 * 480];
        let receipt = draw_effect_records_inner(
            &resources, &records, &bindings, &[0], &origins, 1, 0,
            &mut actual, [512, 480], 512,
        ).unwrap();

        let fade = TerrainFadeTable::parse(resources.fade_bytes()).unwrap();
        let mut expected = vec![23u8; 512 * 480];
        let bias = 2u8.wrapping_mul(16).wrapping_sub(64);
        let mut recolored = 0usize;
        let mut opaque = 0usize;
        for component in &main.components {
            for y in 0..usize::from(component.sprite.height) {
                for x in 0..usize::from(component.sprite.width) {
                    let source = y * usize::from(component.sprite.width) + x;
                    if !component.sprite.opaque[source] { continue; }
                    opaque += 1;
                    let px = 210i32 + i32::from(component.local[0]) + x as i32;
                    let py = 170i32 + i32::from(component.local[1]) + y as i32;
                    if px < 0 || py < 0 || px >= 512 || py >= 480 { continue; }
                    let mut color = component.sprite.indices[source];
                    if color & 0xf0 == 0xc0 {
                        color = bias.wrapping_add(color & 0x0f);
                        recolored += 1;
                    }
                    expected[py as usize * 512 + px as usize] = fade.map_palette_index(3, color).unwrap();
                }
            }
        }
        assert!(recolored > 0 && opaque > 0);
        assert_eq!(actual, expected, "EFF47 owner recolor precedes the selected FADE row");
        assert_eq!(receipt.items[0].binding, EffectAnimationBinding::EffAnimation47);
        assert_eq!(receipt.items[0].effective_shade, 3);
        assert_eq!(receipt.items[0].color_pixels_written, opaque);

        // The code2F wrapper subtracts in 32 bits. A signed-word narrowing
        // would wrap this vertical origin back onscreen.
        let mut tall = active_effect(CODE_47, frame_index as u8, 30);
        tall[0x27..0x2b].copy_from_slice(&(1i32 << 22).to_le_bytes());
        let tall_records = [tall];
        let mut clipped = vec![23u8; 512 * 480];
        let before = clipped.clone();
        let tall_receipt = draw_effect_records_inner(
            &resources, &tall_records, &bindings, &[0], &[[0i16, 0i16]], 0, 0,
            &mut clipped, [512, 480], 512,
        ).unwrap();
        assert_eq!(tall_receipt.items[0].color_origin[1], -65_536);
        assert!(tall_receipt.items[0].color_pixels_clipped > 0);
        assert_eq!(clipped, before);
    }

    #[test]
    fn code0x30_water_frames_use_the_verified_shadow_and_ordinary_fade_path() {
        let resources = staged_resources();
        let fade = TerrainFadeTable::parse(resources.fade_bytes()).unwrap();
        let gla = MainGlaTable::parse(resources.main_gla_bytes()).unwrap();

        for frame_index in [0usize, 5, 11] {
            let frame = decode_frame(resources.effects.bytes(), 48, frame_index).unwrap();
            let mut record = active_effect(CODE_0X30, frame_index as u8, 3);
            record[0x11..0x15].copy_from_slice(&0i32.to_le_bytes());
            record[0x27..0x2b].copy_from_slice(&(512i32).to_le_bytes());
            let records = [record];
            let bindings = [Some(EffectAnimationBinding::EffAnimation48)];
            let ground = [180i16, 160i16];
            let mut actual = vec![23u8; 512 * 480];
            let receipt = draw_effect_records_inner(
                &resources, &records, &bindings, &[0], &[ground], 0, 0,
                &mut actual, [512, 480], 512,
            ).unwrap();
            let item = &receipt.items[0];
            assert_eq!(item.binding, EffectAnimationBinding::EffAnimation48);
            assert_eq!(item.effective_shade, 7);
            assert_eq!(item.shadow_pixels_written > 0, true);

            // Independent foreground walk: positive indexed8 literals remain
            // opaque (including index zero), then the selected FADE row maps
            // each source value. Code30 has no owner recolor or special layer.
            let mut expected = vec![23u8; 512 * 480];
            for component in &frame.components {
                let shadow = draw_normal_component_shadow(
                    &component.sprite, &gla, ground, component.local, 0,
                    [512, 480], 512, &mut expected,
                ).unwrap();
                assert!(shadow.pixels_written > 0);
            }
            for component in &frame.components {
                for y in 0..usize::from(component.sprite.height) {
                    for x in 0..usize::from(component.sprite.width) {
                        let source = y * usize::from(component.sprite.width) + x;
                        if !component.sprite.opaque[source] { continue; }
                        let px = i32::from(ground[0]) + i32::from(component.local[0]) + x as i32;
                        let py = item.color_origin[1] + i32::from(component.local[1]) + y as i32;
                        if px < 0 || py < 0 || px >= 512 || py >= 480 { continue; }
                        expected[py as usize * 512 + px as usize] =
                            fade.map_palette_index(7, component.sprite.indices[source]).unwrap();
                    }
                }
            }
            assert_eq!(actual, expected, "code0x30 frame {frame_index}");
            assert_eq!(item.special_pixels_written, 0);
            assert!(item.color_pixels_written > 0);
        }
    }

    #[test]
    fn codes15_16_and2c_use_the_common_shadow_then_fade_framebuffer_path() {
        let resources=staged_resources();
        let fade=TerrainFadeTable::parse(resources.fade_bytes()).unwrap();
        let gla=MainGlaTable::parse(resources.main_gla_bytes()).unwrap();
        for (code,binding,index) in [
            (CODE_15,EffectAnimationBinding::EffAnimation21,21usize),
            (CODE_16,EffectAnimationBinding::EffAnimation22,22),
            (CODE_2C,EffectAnimationBinding::EffAnimation44,44),
        ] {
            let animation=AnbSections::parse(resources.effects.bytes()).unwrap().ui_metadata().unwrap().animation(index).unwrap();
            let count=((animation.last_frame_offset-animation.first_frame_offset)/14+1) as usize;
            for frame_index in 0..count {
                let frame=decode_frame(resources.effects.bytes(),index,frame_index).unwrap();
                validate_frame_flags(0,&frame,false).unwrap();
                let mut record=active_effect(code,frame_index as u8,5);
                record[0x11..0x15].copy_from_slice(&0i32.to_le_bytes());
                record[0x27..0x2b].copy_from_slice(&512i32.to_le_bytes());
                let records=[record];
                let bindings=[Some(binding)];
                let ground=[240i16,200i16];
                let mut actual=vec![23u8;512*480];
                let receipt=draw_effect_records_inner(&resources,&records,&bindings,&[0],&[ground],0,0,
                    &mut actual,[512,480],512).unwrap();
                let item=&receipt.items[0];
                assert_eq!(item.binding,binding);
                assert_eq!(item.effective_shade,9);
                assert_eq!(item.special_pixels_written,0);
                assert!(item.shadow_pixels_written>0);
                let mut expected=vec![23u8;512*480];
                for component in &frame.components {
                    draw_normal_component_shadow(&component.sprite,&gla,ground,component.local,0,
                        [512,480],512,&mut expected).unwrap();
                }
                for component in &frame.components {
                    for y in 0..usize::from(component.sprite.height) {
                        for x in 0..usize::from(component.sprite.width) {
                            let at=y*usize::from(component.sprite.width)+x;
                            if !component.sprite.opaque[at]{continue;}
                            let px=i32::from(ground[0])+i32::from(component.local[0])+x as i32;
                            let py=item.color_origin[1]+i32::from(component.local[1])+y as i32;
                            if px<0||py<0||px>=512||py>=480{continue;}
                            expected[py as usize*512+px as usize]=fade.map_palette_index(9,component.sprite.indices[at]).unwrap();
                        }
                    }
                }
                assert_eq!(actual,expected,"code {code:#x} frame {frame_index} whole clipped framebuffer");
            }
            let records=[active_effect(code,count as u8,0)];
            let bindings=[Some(binding)];
            let before=vec![0x5au8;512*480];
            let mut destination=before.clone();
            assert!(draw_effect_records_inner(&resources,&records,&bindings,&[0],&[[0,0]],0,0,
                &mut destination,[512,480],512).is_err());
            assert_eq!(destination,before,"failed frame selection is atomic for code {code:#x}");
        }
    }

    #[test]
    fn code0x1a_uses_its_five_pinned_frames_on_the_generic_shadow_and_main_path() {
        let resources = staged_resources();
        let fade = TerrainFadeTable::parse(resources.fade_bytes()).unwrap();
        let gla = MainGlaTable::parse(resources.main_gla_bytes()).unwrap();
        assert_eq!(resources.effects.frame_count(EffectAnimationBinding::EffAnimation26).unwrap(), 5);

        for frame_index in [0usize, 2, 3, 4] {
            let frame = decode_frame(resources.effects.bytes(), 26, frame_index).unwrap();
            assert_eq!(frame.frame_offset, 2030 + (frame_index * 14) as u32);
            assert_eq!(frame.components[0].flags, [0, 0, 0, 0x84, 0x04][frame_index]);
            let mut record = active_effect(CODE_26, frame_index as u8, 3);
            record[0x27..0x2b].copy_from_slice(&512i32.to_le_bytes());
            let records = [record];
            let bindings = [Some(EffectAnimationBinding::EffAnimation26)];
            let ground = [180i16, 160i16];
            let mut actual = vec![23u8; 512 * 480];
            let receipt = draw_effect_records_inner(
                &resources, &records, &bindings, &[0], &[ground], 0, 0,
                &mut actual, [512, 480], 512,
            ).unwrap();
            let item = &receipt.items[0];
            assert_eq!(item.binding, EffectAnimationBinding::EffAnimation26);
            assert_eq!(item.code, CODE_26);
            assert_eq!(item.animation_frame, frame_index as u8);
            assert_eq!(item.effective_shade, 7);
            assert!(item.shadow_pixels_written > 0 && item.color_pixels_written > 0);

            // Independent complete framebuffer walk: shadow first at ground,
            // then the positive indexed8 literals through FADE row 7.
            let mut expected = vec![23u8; 512 * 480];
            for component in &frame.components {
                draw_normal_component_shadow(
                    &component.sprite, &gla, ground, component.local, 0,
                    [512, 480], 512, &mut expected,
                ).unwrap();
            }
            for component in &frame.components {
                for y in 0..usize::from(component.sprite.height) {
                    for x in 0..usize::from(component.sprite.width) {
                        let source = y * usize::from(component.sprite.width) + x;
                        if !component.sprite.opaque[source] { continue; }
                        let px = i32::from(ground[0]) + i32::from(component.local[0]) + x as i32;
                        let py = item.color_origin[1] + i32::from(component.local[1]) + y as i32;
                        if px < 0 || py < 0 || px >= 512 || py >= 480 { continue; }
                        let at = py as usize * 512 + px as usize;
                        let color = fade.map_palette_index(7, component.sprite.indices[source]).unwrap();
                        expected[at] = if component.flags & 0x0c == 4 {
                            gla.map_destination(expected[at], color)
                        } else { color };
                    }
                }
            }
            assert_eq!(actual, expected, "EFF26 frame {frame_index}");
            assert_eq!(item.special_pixels_written, 0);
            println!("EFF26_ORACLE frame={frame_index} sha256={} shadow={} shadow_clipped={} color={} color_clipped={} effective_shade={} color_origin_y={}",
                genewars_runtime::identity::sha256_hex(&actual), item.shadow_pixels_written,
                item.shadow_pixels_clipped, item.color_pixels_written, item.color_pixels_clipped,
                item.effective_shade, item.color_origin[1]);
        }
    }

    #[test]
    fn later_invalid_effect_cannot_commit_earlier_rendered_pixels() {
        let resources = staged_resources();
        let records = [active_effect(CODE_31,0,0),active_effect(0x77,0,0)];
        let bindings = [Some(EffectAnimationBinding::EffAnimation49),None];
        let origins = [[180,160],[260,200]];
        let mut destination = vec![23u8;512*480];
        let before = destination.clone();
        assert!(matches!(draw_effect_records_inner(
            &resources,&records,&bindings,&[0,1],&origins,0,0,&mut destination,[512,480],512,
        ),Err(EffectSceneError::InvalidBinding { slot: 1, .. })));
        assert_eq!(destination,before);
    }
}
