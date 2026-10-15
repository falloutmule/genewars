//! Bounded MULE composition for the proved 59470 wrapper path.
//!
//! This admits selector 4 and owner 4 in source-pinned states 0, 1, and 8
//! (animations 0..7), state 7 (animation 19), and the bounded mortality pairs
//! state 24/animation 17 and state 25/animation 18. Frame bounds are checked
//! against the ANB and ordinary shade rows 0..31. The parity-selected special
//! shade branch remains rejected. The caller supplies projected origin and
//! wrapper/global flags. Resource, health-depth, and frame selection come from
//! the checked record and ANB. Other creature states and shade paths remain
//! unsupported, except the independently admitted BONES selector-4 state
//! 26/27 pair implemented by `draw_supported_bones_record`.

use genewars_assets::IndexedSprite;
use crate::runtime_resources::{BonesAnimationDatabase, CreatureAnimationDatabase, CreatureFramePixels};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatureForegroundInputs {
    /// Wrapper scale passed to 94F30/93F80 (initial MULE path: 150).
    pub factor: u16,
    /// Effective 93F80 flags after component composition; only low two bits
    /// are admitted by this bounded foreground implementation.
    pub flags: u8,
    /// 94F30's two alternating palette bases (initial MULE: 192 and 208).
    pub palette_bases: [u8; 2],
    /// Initial value of 19CFAC bit 0 used to seed the pixel alternation.
    pub clip_flag_bit0: bool,
    pub depth: u8,
    pub threshold: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureForegroundReceipt {
    pub output_size: [usize; 2],
    pub pixels_written: usize,
    pub pixels_clipped: usize,
    /// 59470 invokes 95100 before this color pass when the global shadow gate
    /// is clear. Its pixel traversal is intentionally not claimed here.
    pub shadow_pending: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureFrameReceipt {
    pub animation: usize,
    pub frame_index: usize,
    pub effective_shade: u8,
    pub components: Vec<CreatureForegroundReceipt>,
    pub shadows: Vec<CreatureShadowReceipt>,
    pub shadow_pending: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureShadowReceipt {
    pub output_size: [usize; 2],
    pub pixels_written: usize,
    pub pixels_clipped: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureDrawError {
    InvalidSpriteStorage,
    InvalidFadeRow(usize),
    NegativeFadeRow(i32),
    InvalidViewport,
    InvalidScale,
    UnsupportedFlags(u8),
    ArithmeticOverflow,
    UnsupportedAnimationFrame { animation: usize, frame: usize },
    UnsupportedRecordState,
    InvalidGlaTable(usize),
}

/// Reproduce 59470's transient 19CFAC word setup. Its early animation/state
/// branch skips the yaw override for admitted low-animation states, including
/// the blocked-ground relocation state 0.
fn mule_wrapper_flags(record: &[u8; 124], inherited: u8) -> u8 {
    if record[0x19] < 0x10 && record[0x5e] < 0x1a { return inherited; }
    let yaw = i16::from_le_bytes([record[0x39], record[0x3a]]);
    let sector = yaw >> 7;
    if (6..=13).contains(&sector) { 1 } else { inherited }
}

/// Draw the 937A0 half-height shadow mask for one creature component. The
/// source RLE mask is taken from the first `height >> 1` scanlines because the
/// original routine passes a descriptor with that halved height. Literal
/// nibble values do not affect shadow opacity; MAIN.GLA remaps the explicit
/// shadow color against the current destination index.
pub fn draw_mule_shadow_component(
    sprite: &IndexedSprite,
    origin: [i32; 2],
    factor: u16,
    flags: u8,
    shadow_color: u8,
    main_gla: &[u8],
    clip: [usize; 2],
    pitch: usize,
    destination: &mut [u8],
) -> Result<CreatureShadowReceipt, CreatureDrawError> {
    if factor == 0 { return Err(CreatureDrawError::InvalidScale); }
    if flags & !3 != 0 { return Err(CreatureDrawError::UnsupportedFlags(flags)); }
    if main_gla.len() != 65536 { return Err(CreatureDrawError::InvalidGlaTable(main_gla.len())); }
    let sw=usize::from(sprite.width); let full_h=usize::from(sprite.height); let sh=full_h>>1;
    let pixels=sw.checked_mul(full_h).ok_or(CreatureDrawError::ArithmeticOverflow)?;
    if sprite.indices.len()!=pixels||sprite.opaque.len()!=pixels{return Err(CreatureDrawError::InvalidSpriteStorage);}
    if clip[0]==0||clip[1]==0||pitch<clip[0]{return Err(CreatureDrawError::InvalidViewport);}
    let required=pitch.checked_mul(clip[1]-1).and_then(|n|n.checked_add(clip[0])).ok_or(CreatureDrawError::ArithmeticOverflow)?;
    if destination.len()<required{return Err(CreatureDrawError::InvalidViewport);}
    let factor=usize::from(factor);
    let out_w=sw.checked_mul(factor).ok_or(CreatureDrawError::ArithmeticOverflow)?>>8;
    let out_h=sh.checked_mul(factor).ok_or(CreatureDrawError::ArithmeticOverflow)?>>8;
    if out_w==0||out_h==0{return Ok(CreatureShadowReceipt{output_size:[out_w,out_h],pixels_written:0,pixels_clipped:0});}
    let x_step=(sw<<16)/out_w; let y_step=(sh<<16)/out_h;
    let flip_x=flags&1!=0; let flip_y=flags&2!=0;
    let mut written=0usize; let mut clipped=0usize;
    for dy in 0..out_h {
        let sy=(dy.checked_mul(y_step).ok_or(CreatureDrawError::ArithmeticOverflow)?>>16).min(sh-1);
            let screen_y=i64::from(origin[1])+if flip_y{(out_h-1-dy) as i64}else{dy as i64};
        for dx in 0..out_w {
            let sx=(dx.checked_mul(x_step).ok_or(CreatureDrawError::ArithmeticOverflow)?>>16).min(sw-1);
            let source=sy*sw+sx;
            if !sprite.opaque[source]{continue;}
            let screen_x=i64::from(origin[0])+if flip_x{(out_w-1-dx) as i64}else{dx as i64};
            if !(0..clip[0] as i64).contains(&screen_x)||!(0..clip[1] as i64).contains(&screen_y){clipped+=1;continue;}
            let at=usize::try_from(screen_y).map_err(|_|CreatureDrawError::ArithmeticOverflow)?*pitch+
                usize::try_from(screen_x).map_err(|_|CreatureDrawError::ArithmeticOverflow)?;
            let old=usize::from(destination[at]);
            destination[at]=main_gla[usize::from(shadow_color)*256+old];written+=1;
        }
    }
    Ok(CreatureShadowReceipt{output_size:[out_w,out_h],pixels_written:written,pixels_clipped:clipped})
}

/// Draw one source-selected initial MULE record through the 59470 wrapper.
/// Its exact selector, owner, animation, frame, and scale parameters are
/// checked before decoding. `screen_origin` is caller-projected; the wrapper's
/// physical-height adjustment is applied only when its global shadow mode is
/// zero. Shade zero selects row zero; otherwise the virtual-height contribution
/// is added and capped at 31. The parity-selected flags15&0x10 special branch
/// is rejected because its frame-counter input is outside this adapter.
pub fn draw_initial_mule_record(
    database: &CreatureAnimationDatabase,
    record: &genewars_runtime::creature_world::CreatureRecord,
    screen_origin: [i32; 2],
    virtual_height: i32,
    global_shadow_mode: u8,
    incoming_wrapper_flags: u8,
    clip_flag_bit0: bool,
    shadow_color: u8,
    depth: u8,
    fade: &[u8],
    main_gla: &[u8],
    clip: [usize; 2],
    pitch: usize,
    destination: &mut [u8],
) -> Result<CreatureFrameReceipt, CreatureDrawError> {
    if record[0x5e] != 8 {
        return Err(CreatureDrawError::UnsupportedRecordState);
    }
    draw_supported_mule_record(database, record, screen_origin, virtual_height,
        global_shadow_mode, incoming_wrapper_flags, clip_flag_bit0, shadow_color,
        depth, fade, main_gla, clip, pitch, destination)
}

/// Draw a source-selected MULE record for the currently admitted startup
/// states. The extra pairs are pinned to the combined-world snapshots and
/// direction selector: states 0 and 1 use animations 0..7 with frame bounds from
/// the ANB, and state 7 uses animation 19/frames 0..3. State 8 retains the
/// earlier bounded 0..7 animation path. State 0 is admitted only for the
/// source-pinned low-animation case selected by `0x5E640`; this does not infer
/// the state machine or animation clock, and callers provide the current
/// record.
pub fn draw_supported_mule_record(
    database: &CreatureAnimationDatabase,
    record: &genewars_runtime::creature_world::CreatureRecord,
    screen_origin: [i32; 2],
    virtual_height: i32,
    global_shadow_mode: u8,
    incoming_wrapper_flags: u8,
    clip_flag_bit0: bool,
    shadow_color: u8,
    depth: u8,
    fade: &[u8],
    main_gla: &[u8],
    clip: [usize; 2],
    pitch: usize,
    destination: &mut [u8],
) -> Result<CreatureFrameReceipt, CreatureDrawError> {
    let animation = record[0x19];
    let frame_index = record[0x1b];
    let admitted_pair = match record[0x5e] {
        // The blocked-ground relocation path enters state 0, and `0x5E640`
        // dispatches case 0 through this bounded directional animation set.
        0 => animation <= 7,
        // State 1's source-proved directional selector chooses animation 0..7;
        // the ANB then bounds the current frame for each chosen direction.
        // Source651 pair initialization selects directional frames;59470
        // shares the animation<16/state<26 branch with ordinary approach.
        1 | 3 => animation <= 7,
        //65AEF corpse arrival selects state5;5E640 shares animation19 with state7.
        4 | 5 | 7 => animation == 19 && frame_index <= 3,
        8 => animation <= 7 || animation == 22,
        //5EA66 selects22;59470 uses the established high-animation wrapper.
        10 => animation <= 7 || animation == 22,
        //5E749 selects attack directions8..15;59470 uses its ordinary body.
        21 => (8..=15).contains(&animation),
        //5E614->5E817 selects16 for neutral hit reaction; owned20 excluded.
        20 => record[0x18] == 4 && animation == 16,
        // Literal mortality continuations pin state 24 to animation 17 and
        // state 25 to corpse animation 18. The ANB remains the authority for
        // their frame extents (7 and 3 frames respectively).
        24 => animation == 17,
        25 => animation == 18,
        _ => false,
    };
    let owned = record[0x18] < 4;
    //59DD8..59DEC: positive lifecycle stage draws an extra icon only
    //above half food reserve. Below that threshold the ordinary body is exact.
    let no_lifecycle_icon = record[0x79] as i8 <= 0
        || i16::from_le_bytes([record[0x68],record[0x69]]) <= (i16::from_le_bytes([record[0x62],record[0x63]]) >> 1);
    //59470 does not branch on task7 or these references in its body pass.
    //The world owns reciprocal live-target binding; this record-only renderer
    //admits the exact task7 pursuit or source-proved attack-entry shape.
    let target = i16::from_le_bytes([record[0x55],record[0x56]]);
    let owned_hostile = ((record[0x5e]==1 && matches!(record[0x60],0x48|0xc8))
        || (record[0x5e]==21 && matches!(record[0x60],0x08|0x88)))
        && record[0x78]==7 && record[0x61]==0
        && record[0x76]<=90 && record[0x4f]==0 && record[0x5d]==0
        && record[0x52]==1 && record[0x57]==1 && record[0x5a]==1
        && record[0x50..0x53]==record[0x55..0x58]
        && record[0x58..0x5b]==record[0x55..0x58]
        && (0..300).contains(&target) && target!=i16::from_le_bytes([record[15],record[16]]);
    let owned_ordinary = (matches!(record[0x5e],0|8) || owned_hostile) && no_lifecycle_icon
        && (record[0x7b]==0 || record[0x7b]>191) && !(record[0x5e]==0 && record[0x60]&0x20!=0);
    if record[6] != 1 || record[0x18] > 4 || record[0x7a] != 4 || !admitted_pair || (owned && !owned_ordinary) {
        return Err(CreatureDrawError::UnsupportedRecordState);
    }
    let animation = usize::from(record[0x19]);
    let frame_index = usize::from(record[0x1b]);
    let frame = database.decode_frame_packed4(animation, frame_index, 0)
        .map_err(|_| CreatureDrawError::UnsupportedRecordState)?;
    let factor = if record[0x77] == 255 && record[0x76] >= 100 {
        256
    } else {
        (u16::from(record[0x76]) * u16::from(record[0x77]) / 100).clamp(150, 256)
    };
    let wrapper_flags = mule_wrapper_flags(record, incoming_wrapper_flags);
    let origin = [
        screen_origin[0],
        if global_shadow_mode == 0 {
            screen_origin[1].checked_sub(virtual_height >> 6).ok_or(CreatureDrawError::ArithmeticOverflow)?
        } else { screen_origin[1] },
    ];
    // 59470 special-cases raw zero. Otherwise it adds the arithmetic-shifted
    // virtual height and applies only an upper cap of 31 (no lower clamp).
    let raw_shade = record[0x17];
    let shade = if raw_shade == 0 { 0 } else {
        let adjusted = i32::from(raw_shade) + (virtual_height >> 7);
        if adjusted < 0 { return Err(CreatureDrawError::NegativeFadeRow(adjusted)); }
        adjusted.min(31) as u8
    };
    // The other 59470 branch uses global frame parity to choose a special
    // owner-recolor path. Until that caller global is supplied, do not guess.
    if global_shadow_mode == 0 && shade != 0 && record[0x15] & 0x10 != 0 {
        return Err(CreatureDrawError::UnsupportedRecordState);
    }
    let fade_start = usize::from(shade) * 256;
    let fade_row = fade.get(fade_start..fade_start + 256)
        .ok_or(CreatureDrawError::InvalidFadeRow(usize::from(shade)))?;
    let mut scratch=destination.to_vec();
    let mut shadows=Vec::new();
    if global_shadow_mode==0 {
        if main_gla.len()!=65536{return Err(CreatureDrawError::InvalidGlaTable(main_gla.len()));}
        for component in &frame.components {
            let sw=usize::from(component.sprite.width);
            let out_w=sw*usize::from(factor)>>8;
            let parent_x=wrapper_flags&1!=0;
            let parent_y=wrapper_flags&2!=0;
            let scaled_x=(i32::from(component.local_position.0)*i32::from(factor))>>8;
            let scaled_y=(i32::from(component.local_position.1)*i32::from(factor))>>9;
            let x_offset=if parent_x{-scaled_x-out_w as i32}else{scaled_x};
            // 95100 flips component placement using the full sprite height;
            // 937A0 subsequently halves the descriptor height for its mask.
            let full_out_h=usize::from(component.sprite.height)*usize::from(factor)>>8;
            let y_offset=if parent_y{-scaled_y-full_out_h as i32}else{scaled_y};
            let shadow_base_y=screen_origin[1].checked_add(i32::from(factor>>5))
                .ok_or(CreatureDrawError::ArithmeticOverflow)?;
            let c_origin=[screen_origin[0].checked_add(x_offset).ok_or(CreatureDrawError::ArithmeticOverflow)?,
                shadow_base_y.checked_add(y_offset).ok_or(CreatureDrawError::ArithmeticOverflow)?];
            let flags=(component.flags|(wrapper_flags&!3))^(wrapper_flags&3);
            shadows.push(draw_mule_shadow_component(&component.sprite,c_origin,factor,flags,shadow_color,main_gla,
                clip,pitch,&mut scratch)?);
        }
    }
    //59470 owned branch uses94D70/94400: one owner palette, with
    //identity mapping when source supplies a NULL fade pointer.
    let identity: [u8;256]=std::array::from_fn(|i|i as u8);
    let owned_fade=if record[0x15]&2==0 && shade<30 {fade_row}else{&identity};
    let mut receipt = if owned {
        let base=record[0x18].wrapping_mul(16).wrapping_sub(64);
        draw_mule_color_frame_palette(&frame,origin,factor,wrapper_flags,clip_flag_bit0,depth,135,
            owned_fade,clip,pitch,&mut scratch,[base;2],true)?
    } else {
        draw_mule_color_frame(&frame, origin, factor,wrapper_flags,clip_flag_bit0,depth,135,fade_row,clip,pitch,&mut scratch)?
    };
    destination.copy_from_slice(&scratch);
    receipt.shadow_pending = false;
    receipt.shadows=shadows;
    receipt.effective_shade = shade;
    Ok(receipt)
}

/// Draw the narrowly source-pinned BONES selector-4 state-26/27 pair through
/// the ordinary 59470 shadow and 94F30 owner-palette color path. These states
/// use animation 4, whose two frames and all sprite records are admitted by
/// the exact BONES.ANB hash-bound database. Caller projection and global
/// wrapper inputs remain explicit.
pub fn draw_supported_bones_record(
    database: &BonesAnimationDatabase,
    record: &genewars_runtime::creature_world::CreatureRecord,
    screen_origin: [i32; 2],
    virtual_height: i32,
    global_shadow_mode: u8,
    incoming_wrapper_flags: u8,
    clip_flag_bit0: bool,
    shadow_color: u8,
    fade: &[u8],
    main_gla: &[u8],
    clip: [usize; 2],
    pitch: usize,
    destination: &mut [u8],
) -> Result<CreatureFrameReceipt, CreatureDrawError> {
    if record[6] != 1 || record[0x18] != 4 || record[0x7a] != 4
        || !matches!(record[0x5e], 26 | 27) || record[0x19] != 4 || record[0x1b] > 1
        || record[0x15] & 0x10 != 0 {
        return Err(CreatureDrawError::UnsupportedRecordState);
    }
    let frame = database.decode_frame_packed4(4, usize::from(record[0x1b]), 0)
        .map_err(|_| CreatureDrawError::UnsupportedRecordState)?;
    if frame.components.is_empty() || frame.components.iter().any(|component| component.flags != 0) {
        return Err(CreatureDrawError::UnsupportedRecordState);
    }
    let factor = if record[0x77] == 255 && record[0x76] >= 100 {
        256
    } else {
        (u16::from(record[0x76]) * u16::from(record[0x77]) / 100).clamp(150, 256)
    };
    let wrapper_flags = mule_wrapper_flags(record, incoming_wrapper_flags);
    let origin = [screen_origin[0], if global_shadow_mode == 0 {
        screen_origin[1].checked_sub(virtual_height >> 6).ok_or(CreatureDrawError::ArithmeticOverflow)?
    } else { screen_origin[1] }];
    let raw_shade = record[0x17];
    let shade = if raw_shade == 0 { 0 } else {
        let adjusted = i32::from(raw_shade) + (virtual_height >> 7);
        if adjusted < 0 { return Err(CreatureDrawError::NegativeFadeRow(adjusted)); }
        adjusted.min(31) as u8
    };
    let fade_start = usize::from(shade) * 256;
    let fade_row = fade.get(fade_start..fade_start + 256)
        .ok_or(CreatureDrawError::InvalidFadeRow(usize::from(shade)))?;
    // BONES 59470's state>=26 branch fixes health depth to zero. Its special
    // parity draw remains excluded by the record admission above.
    let mut scratch = destination.to_vec();
    let mut shadows = Vec::new();
    if global_shadow_mode == 0 {
        if main_gla.len() != 65536 { return Err(CreatureDrawError::InvalidGlaTable(main_gla.len())); }
        for component in &frame.components {
            let factor_usize = usize::from(factor);
            let width = usize::from(component.sprite.width);
            let height = usize::from(component.sprite.height);
            let out_w = width.checked_mul(factor_usize).ok_or(CreatureDrawError::ArithmeticOverflow)? >> 8;
            let scaled_x = (i32::from(component.local_position.0) * i32::from(factor)) >> 8;
            let scaled_y = (i32::from(component.local_position.1) * i32::from(factor)) >> 9;
            let x_offset = if wrapper_flags & 1 != 0 { -scaled_x - out_w as i32 } else { scaled_x };
            let out_h = height.checked_mul(factor_usize).ok_or(CreatureDrawError::ArithmeticOverflow)? >> 8;
            let y_offset = if wrapper_flags & 2 != 0 { -scaled_y - out_h as i32 } else { scaled_y };
            let shadow_y = screen_origin[1].checked_add(i32::from(factor >> 5))
                .ok_or(CreatureDrawError::ArithmeticOverflow)?;
            let shadow_origin = [screen_origin[0].checked_add(x_offset).ok_or(CreatureDrawError::ArithmeticOverflow)?,
                shadow_y.checked_add(y_offset).ok_or(CreatureDrawError::ArithmeticOverflow)?];
            shadows.push(draw_mule_shadow_component(&component.sprite, shadow_origin, factor,
                wrapper_flags, shadow_color, main_gla, clip, pitch, &mut scratch)?);
        }
    }
    let mut receipt = draw_mule_color_frame(&frame, origin, factor, wrapper_flags,
        clip_flag_bit0, 0, 135, fade_row, clip, pitch, &mut scratch)?;
    destination.copy_from_slice(&scratch);
    receipt.shadow_pending = false;
    receipt.shadows = shadows;
    receipt.effective_shade = shade;
    Ok(receipt)
}

/// Draw the foreground components of MULE's caller-selected frame through
/// 94F30/93F80. `origin` is the already projected and height-adjusted frame
/// origin supplied by the caller. The caller also supplies the incoming
/// 19CFAC wrapper bits and fade row; neither is guessed from asset bytes.
///
/// This does not issue 95100's shadow pass; the returned receipt marks it
/// pending. Animation and frame bounds are checked against the selected ANB
/// metadata; this helper does not infer animation timing or update state.
pub fn draw_mule_color_frame(
    frame: &CreatureFramePixels,
    origin: [i32; 2],
    factor: u16,
    incoming_wrapper_flags: u8,
    clip_flag_bit0: bool,
    depth: u8,
    threshold: u8,
    fade_row: &[u8],
    clip: [usize; 2],
    pitch: usize,
    destination: &mut [u8],
) -> Result<CreatureFrameReceipt, CreatureDrawError> {
    draw_mule_color_frame_palette(frame,origin,factor,incoming_wrapper_flags,clip_flag_bit0,depth,threshold,fade_row,clip,pitch,destination,[192,208],false)
}
fn draw_mule_color_frame_palette(
    frame: &CreatureFramePixels,
    origin: [i32; 2],
    factor: u16,
    incoming_wrapper_flags: u8,
    clip_flag_bit0: bool,
    depth: u8,
    threshold: u8,
    fade_row: &[u8],
    clip: [usize; 2],
    pitch: usize,
    destination: &mut [u8],
    palette_bases: [u8;2],
    truncate_offsets: bool,
) -> Result<CreatureFrameReceipt, CreatureDrawError> {
    if incoming_wrapper_flags & !3 != 0 { return Err(CreatureDrawError::UnsupportedFlags(incoming_wrapper_flags)); }
    if factor == 0 { return Err(CreatureDrawError::InvalidScale); }
    let mut scratch = destination.to_vec();
    let mut components = Vec::with_capacity(frame.components.len());
    for component in &frame.components {
        let out_w = usize::from(component.sprite.width).checked_mul(usize::from(factor))
            .ok_or(CreatureDrawError::ArithmeticOverflow)? >> 8;
        let out_h = usize::from(component.sprite.height).checked_mul(usize::from(factor))
            .ok_or(CreatureDrawError::ArithmeticOverflow)? >> 8;
        let sx=i32::from(component.local_position.0)*i32::from(factor);
        let scaled_x=if truncate_offsets{sx/256}else{sx>>8};
        let sy=i32::from(component.local_position.1)*i32::from(factor);
        let scaled_y=if truncate_offsets{sy/256}else{sy>>8};
        let flip_parent_x = incoming_wrapper_flags & 1 != 0;
        let flip_parent_y = incoming_wrapper_flags & 2 != 0;
        let x_offset = if flip_parent_x { -scaled_x - out_w as i32 } else { scaled_x };
        let y_offset = if flip_parent_y { -scaled_y - out_h as i32 } else { scaled_y };
        let effective_flags = (component.flags | (incoming_wrapper_flags & !3))
            ^ (incoming_wrapper_flags & 3);
        let component_origin = [
            origin[0].checked_add(x_offset).ok_or(CreatureDrawError::ArithmeticOverflow)?,
            origin[1].checked_add(y_offset).ok_or(CreatureDrawError::ArithmeticOverflow)?,
        ];
        components.push(draw_mule_foreground_component(
            &component.sprite,
            component_origin,
            CreatureForegroundInputs {
                factor,
                flags: effective_flags,
                palette_bases,
                clip_flag_bit0,
                depth,
                threshold,
            },
            fade_row,
            clip,
            pitch,
            &mut scratch,
        )?);
    }
    destination.copy_from_slice(&scratch);
    Ok(CreatureFrameReceipt {
        animation: frame.animation,
        frame_index: frame.frame_index,
        effective_shade: 0,
        shadow_pending: true,
        components,
        shadows: Vec::new(),
    })
}

/// Draw one already-decoded Packed4 component through the traced 93F80
/// foreground color path. The caller supplies the component's top-left
/// wrapper origin and 19CFAC flag state; screen clipping is expressed by the
/// supplied viewport and pitch.
///
/// For each destination sample, the source coordinate is the integer part
/// of the source-per-output 16.16 ratio produced by 8F8F0. Literal nibble zero
/// remains opaque according to the sprite's run-derived mask. The fade row
/// is applied after choosing the alternating 192/208 color base.
pub fn draw_mule_foreground_component(
    sprite: &IndexedSprite,
    origin: [i32; 2],
    inputs: CreatureForegroundInputs,
    fade_row: &[u8],
    clip: [usize; 2],
    pitch: usize,
    destination: &mut [u8],
) -> Result<CreatureForegroundReceipt, CreatureDrawError> {
    if inputs.factor == 0 { return Err(CreatureDrawError::InvalidScale); }
    if inputs.flags & !3 != 0 { return Err(CreatureDrawError::UnsupportedFlags(inputs.flags)); }
    let sw = usize::from(sprite.width);
    let sh = usize::from(sprite.height);
    let count = sw.checked_mul(sh).ok_or(CreatureDrawError::ArithmeticOverflow)?;
    if sprite.indices.len() != count || sprite.opaque.len() != count {
        return Err(CreatureDrawError::InvalidSpriteStorage);
    }
    if clip[0] == 0 || clip[1] == 0 || pitch < clip[0] {
        return Err(CreatureDrawError::InvalidViewport);
    }
    let required = pitch.checked_mul(clip[1] - 1).and_then(|n| n.checked_add(clip[0]))
        .ok_or(CreatureDrawError::ArithmeticOverflow)?;
    if destination.len() < required { return Err(CreatureDrawError::InvalidViewport); }
    if fade_row.len() != 256 { return Err(CreatureDrawError::InvalidFadeRow(fade_row.len())); }

    let factor = usize::from(inputs.factor);
    let out_w = sw.checked_mul(factor).ok_or(CreatureDrawError::ArithmeticOverflow)? >> 8;
    let out_h = sh.checked_mul(factor).ok_or(CreatureDrawError::ArithmeticOverflow)? >> 8;
    if out_w == 0 || out_h == 0 {
        return Ok(CreatureForegroundReceipt { output_size: [out_w, out_h], pixels_written: 0,
            pixels_clipped: 0, shadow_pending: true });
    }
    let x_step = sw.checked_shl(16).ok_or(CreatureDrawError::ArithmeticOverflow)? / out_w;
    let y_step = sh.checked_shl(16).ok_or(CreatureDrawError::ArithmeticOverflow)? / out_h;
    let flip_x = inputs.flags & 1 != 0;
    let flip_y = inputs.flags & 2 != 0;
    let mut pixels_written = 0usize;
    let mut pixels_clipped = 0usize;

    for dy in 0..out_h {
        let screen_y = i64::from(origin[1]) + if flip_y { (out_h - 1 - dy) as i64 } else { dy as i64 };
        let sy = (dy.checked_mul(y_step).ok_or(CreatureDrawError::ArithmeticOverflow)? >> 16).min(sh - 1);
        let row = sy.checked_mul(sw).ok_or(CreatureDrawError::ArithmeticOverflow)?;
        let visible_y = (0..clip[1] as i64).contains(&screen_y);
        for dx in 0..out_w {
            let screen_x = i64::from(origin[0]) + if flip_x { (out_w - 1 - dx) as i64 } else { dx as i64 };
            let sx = (dx.checked_mul(x_step).ok_or(CreatureDrawError::ArithmeticOverflow)? >> 16).min(sw - 1);
            let source = row + sx;
            let visible_x = (0..clip[0] as i64).contains(&screen_x);
            if !visible_x || !visible_y {
                if sprite.opaque[source] { pixels_clipped += 1; }
                continue;
            }
            if !sprite.opaque[source] { continue; }

            let x = usize::try_from(screen_x).map_err(|_| CreatureDrawError::ArithmeticOverflow)?;
            let y = usize::try_from(screen_y).map_err(|_| CreatureDrawError::ArithmeticOverflow)?;
            let at = y.checked_mul(pitch).and_then(|n| n.checked_add(x))
                .ok_or(CreatureDrawError::ArithmeticOverflow)?;
            let pointer_parity = (at as u8 ^ origin[0] as u8) & 1;
            let alternate = u8::from(inputs.clip_flag_bit0) ^ pointer_parity;
            let base = inputs.palette_bases[usize::from(alternate & 1)];
            let nibble = sprite.indices[source] & 0x0f;
            let mapped = if nibble < inputs.depth {
                inputs.threshold.wrapping_sub(nibble >> 2)
            } else {
                base.wrapping_add(nibble)
            };
            destination[at] = fade_row[usize::from(mapped)];
            pixels_written += 1;
        }
    }
    Ok(CreatureForegroundReceipt { output_size: [out_w, out_h], pixels_written,
        pixels_clipped, shadow_pending: true })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime_resources::Resources;

    fn sprite(w: u8, h: u8, values: &[u8], opaque: &[bool]) -> IndexedSprite {
        IndexedSprite { width: w, height: h, indices: values.to_vec(), opaque: opaque.to_vec(), consumed_bytes: 0 }
    }

    #[test]
    fn scaled_packed4_foreground_uses_opaque_zero_and_fade_row() {
        let s = sprite(4, 2, &[0,1,2,3,4,5,6,7], &[true;8]);
        let fade: Vec<u8> = (0..=255).map(|n| 255 - n as u8).collect();
        let mut dst = vec![0x55; 12 * 4];
        let r = draw_mule_foreground_component(&s, [2,1], CreatureForegroundInputs {
            factor: 256, flags: 0, palette_bases: [192,208], clip_flag_bit0: false,
            depth: 0, threshold: 135,
        }, &fade, [12,4], 12, &mut dst).unwrap();
        assert_eq!(r.output_size, [4,2]);
        assert_eq!(r.pixels_written, 8);
        assert_eq!(dst[12 + 2], fade[192]);
        assert_eq!(dst[12 + 3], fade[209]);
        assert_eq!(dst[24 + 5], fade[215]);
        assert!(dst.iter().enumerate().all(|(i,&v)| {
            let x=i%12; let y=i/12;
            if (2..6).contains(&x) && (1..3).contains(&y) { true } else { v==0x55 }
        }));
    }

    #[test]
    fn flips_clip_and_padding_are_bounded() {
        let s = sprite(3, 2, &[1,2,3,4,5,6], &[true;6]);
        let fade: Vec<u8> = (0..=255).map(|n| n as u8).collect();
        let mut dst = vec![0xcc; 8 * 3 + 2];
        let r = draw_mule_foreground_component(&s, [0,0], CreatureForegroundInputs {
            factor: 256, flags: 1, palette_bases: [192,208], clip_flag_bit0: false,
            depth: 0, threshold: 135,
        }, &fade, [8,3], 8, &mut dst).unwrap();
        assert_eq!(r.pixels_written, 6);
        assert_eq!(r.pixels_clipped, 0);
        assert_eq!(&dst[0..8], &[fade[195], fade[210], fade[193], fade[0xcc], fade[0xcc], fade[0xcc], fade[0xcc], fade[0xcc]]);
        assert!(dst[24..].iter().all(|&b| b==0xcc));
    }

    #[test]
    fn invalid_late_structure_leaves_frame_unchanged() {
        let s = sprite(1, 1, &[9], &[]);
        let fade = [0u8;256];
        let mut dst = [77u8;8];
        let before = dst;
        assert_eq!(draw_mule_foreground_component(&s, [0,0], CreatureForegroundInputs {
            factor: 150, flags: 0, palette_bases: [192,208], clip_flag_bit0: false,
            depth: 0, threshold: 135,
        }, &fade, [4,2], 4, &mut dst), Err(CreatureDrawError::InvalidSpriteStorage));
        assert_eq!(dst, before);
    }

    #[test]
    fn staged_mule_directional_frames_decode_and_compose_explicitly() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..").join("private/integration/game-runtime");
        let (resources, _, _) = Resources::load(&root).expect("hash-bound game resources");
        for (animation, last_frame) in [(4usize, 7usize), (5, 7), (6, 1)] {
            for frame_index in 0..=last_frame {
                let frame = resources.mule.decode_frame_packed4(animation, frame_index, 0)
                    .expect("proved MULE packed4 frame");
                assert!(!frame.components.is_empty());
                let mut pixels = vec![0u8; 96 * 80];
                let fade = [0u8; 256];
                let receipt = draw_mule_color_frame(&frame, [8, 8], 150, 0, false,
                    0, 135, &fade, [96, 80], 96, &mut pixels).unwrap();
                assert_eq!(receipt.animation, animation);
                assert_eq!(receipt.frame_index, frame_index);
                assert_eq!(receipt.components.len(), frame.components.len());
                assert!(receipt.components.iter().all(|c| c.output_size[0] <= 255 && c.output_size[1] <= 255));
            }
        }
    }

    #[test]
    fn bones_state26_source_frame_draws_shadow_then_owner_fade_atomically() {
        let repository = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let expected = std::fs::read(repository.join("private/review/mule-bones/expected.bin")).unwrap();
        assert_eq!(expected.len(), 112 * 124);
        let mut record = [0u8; 124];
        record.copy_from_slice(&expected[..124]);
        assert_eq!(genewars_runtime::identity::sha256_hex(&record),
            "26ca8cf70ac39dd9e324cdbd10a82341cb860367c9b960882120a181047bbac3");
        assert_eq!(record[6], 1);
        assert_eq!(record[0x18], 4);
        assert_eq!(record[0x19], 4);
        assert_eq!(record[0x1b], 0);
        assert_eq!(record[0x5e], 26);
        assert_eq!(record[0x7a], 4);
        let root = repository.join("private/integration/game-runtime");
        let (resources, _, _) = super::super::Resources::load(&root)
            .expect("exact 53-resource runtime bundle");
        let mut destination = vec![0x5au8; 640 * 479 + 512 + 16];
        let before = destination.clone();
        let receipt = draw_supported_bones_record(&resources.bones, &record, [32, 48], 0,
            0, 0, false, 0, resources.fade_bytes(), resources.main_gla_bytes(),
            [512, 480], 640, &mut destination).expect("pinned BONES state26 draw path");
        assert_eq!(receipt.animation, 4);
        assert_eq!(receipt.frame_index, 0);
        assert_eq!(receipt.effective_shade, 0);
        assert!(!receipt.shadow_pending);
        assert_eq!(receipt.components.len(), 1);
        assert_eq!(receipt.shadows.len(), 1);
        assert!(receipt.components[0].pixels_written > 0);
        assert!(receipt.shadows[0].pixels_written > 0);
        for y in 0..480 {
            let start = y * 640 + 512;
            let end = (start + 128).min(destination.len());
            assert_eq!(&destination[start..end], &before[start..end], "row padding {y}");
        }
        let mut invalid = record;
        invalid[0x5e] = 25;
        let before_reject = destination.clone();
        assert!(draw_supported_bones_record(&resources.bones, &invalid, [32, 48], 0,
            0, 0, false, 0, resources.fade_bytes(), resources.main_gla_bytes(),
            [512, 480], 640, &mut destination).is_err());
        assert_eq!(destination, before_reject);
    }

    #[test]
    fn export_source_bones_pixel_oracle_cases_when_requested() {
        let Ok(output_path) = std::env::var("GENEWARS_BONES_ORACLE_OUT") else { return; };
        use std::io::Write;
        let repository = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let root = repository.join("private/integration/game-runtime");
        let (resources, _, _) = super::super::Resources::load(&root)
            .expect("exact hash-bound runtime resources");
        let source = std::fs::read(repository.join("private/review/mule-bones/expected.bin"))
            .expect("source-traced BONES selector-4 fixture");
        assert_eq!(source.len(), 112 * 124);
        let cases = [
            (0usize, 26u8, 0u8, "26ca8cf70ac39dd9e324cdbd10a82341cb860367c9b960882120a181047bbac3"),
            (41usize, 27u8, 1u8, "91a81188484deaa31508163a1152a1b5d63aa27d676ebb46aaf6919b1206b5de"),
        ];
        let path = std::path::Path::new(&output_path);
        let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(path)
            .expect("oracle output path must be new");
        for (record_index, state, frame_index, expected_hash) in cases {
            let record_start = record_index * 124;
            let selected: [u8; 124] = source[record_start..record_start + 124]
                .try_into().expect("pinned BONES wrapper record");
            assert_eq!(genewars_runtime::identity::sha256_hex(&selected), expected_hash);
            assert_eq!(selected[0x5e], state);
            assert_eq!(selected[0x19], 4);
            assert_eq!(selected[0x1b], frame_index);
            let mut pixels = vec![0xa5; 68 * 48 + 4];
            let receipt = draw_supported_bones_record(&resources.bones, &selected, [24,18], 0,
                0, 0, false, 0, resources.fade_bytes(), resources.main_gla_bytes(),
                [64,48], 68, &mut pixels).expect("admitted BONES owner-4 wrapper branch");
            let hex = pixels.iter().map(|v| format!("{v:02x}")).collect::<String>();
            let fg_written = receipt.components.iter().map(|x|x.pixels_written).sum::<usize>();
            let fg_clipped = receipt.components.iter().map(|x|x.pixels_clipped).sum::<usize>();
            let sh_written = receipt.shadows.iter().map(|x|x.pixels_written).sum::<usize>();
            let sh_clipped = receipt.shadows.iter().map(|x|x.pixels_clipped).sum::<usize>();
            writeln!(file,"{{\"state\":{state},\"animation\":4,\"frame\":{frame_index},\"scale_a\":{},\"scale_b\":{},\"virtual_height\":0,\"raw_shade\":{},\"effective_shade\":{},\"shadow_mode\":0,\"inherited\":0,\"clip_bit\":0,\"origin\":[24,18],\"yaw\":{},\"shadow_color\":0,\"fg_written\":{fg_written},\"fg_clipped\":{fg_clipped},\"sh_written\":{sh_written},\"sh_clipped\":{sh_clipped},\"pixels\":\"{hex}\"}}",
                selected[0x76],selected[0x77],selected[0x17],receipt.effective_shade,
                i16::from_le_bytes([selected[0x39],selected[0x3a]])).unwrap();
        }
    }

    #[test]
    fn shadow_halves_source_height_and_uses_explicit_gla_color() {
        let s = sprite(3, 4, &[0; 12], &[true; 12]);
        let mut gla = vec![0u8; 65536];
        gla[7 * 256 + 3] = 0x91;
        let mut dst = vec![3u8; 8 * 4 + 2];
        let receipt = draw_mule_shadow_component(&s, [2, 1], 256, 0, 7,
            &gla, [8, 4], 8, &mut dst).unwrap();
        assert_eq!(receipt.output_size, [3, 2]);
        assert_eq!(receipt.pixels_written, 6);
        assert_eq!(&dst[8..16], &[3, 3, 0x91, 0x91, 0x91, 3, 3, 3]);
        assert_eq!(&dst[16..24], &[3, 3, 0x91, 0x91, 0x91, 3, 3, 3]);
        assert!(dst[24..].iter().all(|&v| v == 3));
    }

    #[test]
    fn wrapper_yaw_override_obeys_early_animation_and_state_branch() {
        let mut record = [0u8; 124];
        record[0x19] = 4;
        record[0x5e] = 8;
        record[0x39..0x3b].copy_from_slice(&(6i16 * 128).to_le_bytes());
        assert_eq!(mule_wrapper_flags(&record, 2), 2);
        record[0x19] = 16;
        assert_eq!(mule_wrapper_flags(&record, 2), 1);
        record[0x39..0x3b].copy_from_slice(&(13i16 * 128).to_le_bytes());
        assert_eq!(mule_wrapper_flags(&record, 0), 1);
        record[0x39..0x3b].copy_from_slice(&(14i16 * 128).to_le_bytes());
        assert_eq!(mule_wrapper_flags(&record, 2), 2);
        record[0x39..0x3b].copy_from_slice(&(-6i16 * 128).to_le_bytes());
        assert_eq!(mule_wrapper_flags(&record, 3), 3);
    }

    #[test]
    fn source_selected_state8_frames_draw_shadow_then_color() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..").join("private/integration/game-runtime");
        let (resources, _, _) = Resources::load(&root).expect("hash-bound game resources");
        for animation in 0u8..8 {
            for frame in 0u8..8 {
            if animation == 4 && frame == 0 { continue; }
            let mut record = [0u8; 124];
            record[6] = 1;
            record[0x18] = 4;
            record[0x19] = animation;
            record[0x1b] = frame;
            record[0x5e] = 8;
            record[0x17] = 0;
            record[0x76] = 30;
            record[0x77] = 127;
            record[0x7a] = 4;
            let mut pixels = vec![0u8; 640 * 480];
            let receipt = draw_initial_mule_record(&resources.mule, &record,
                [260, 200], 0, 0, 0, false, 7, 0,
                &resources.fade, &resources.gla, [640, 480], 640, &mut pixels).unwrap();
            assert_eq!(receipt.animation, usize::from(animation));
            assert_eq!(receipt.frame_index, usize::from(frame));
            assert!(!receipt.components.is_empty());
            assert_eq!(receipt.shadows.len(), receipt.components.len());
            assert!(!receipt.shadow_pending);
            assert!(receipt.components.iter().map(|c| c.pixels_written).sum::<usize>() > 0);
            assert!(receipt.shadows.iter().map(|c| c.pixels_written).sum::<usize>() > 0);
            }
        }
    }

    #[test]
    fn combined_world_approach_and_feed_snapshots_draw_their_source_frames() {
        let repository = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let (resources, _, _) = Resources::load(&repository.join("private/integration/game-runtime"))
            .expect("hash-bound game resources");
        for (relative, expected_hash, expected_state, expected_animation, expected_frame) in [
            ("combined-before-mule704/source-after704.bin",
                "992a276fab0cf19e9c8290a17c6a3d2b23b7f7cc845469fd40a6c55b609318aa", 0, 6, 1),
            ("combined-seed0-before-mule640/source-after640.bin",
                "febfd1a200dfcec4ebf91b5b7d2cc20e11a72eef92c2cf0645abccb0454d0ddc", 0, 6, 1),
            ("combined-seed65535-before-mule608/source-after608.bin",
                "33f91be4c6b427f132f498b3d62d19a631437c446c5d3c625bc6f1df824b309b", 0, 2, 1),
            ("combined-before-mule366/source-after366.bin",
                "68c3ab88608abbbd82ff0f9bfd4a740ead1ca5a1e8daf797b2ec6f3b352ab041", 1, 0, 0),
            ("combined-before-mule385/source-after385.bin",
                "fac6842c180146b2cee77dcd9da407bed9cbafd57d414b554d00076056d539bd", 7, 19, 1),
        ] {
            let bytes = std::fs::read(repository.join("private/integration").join(relative)).unwrap();
            assert_eq!(genewars_runtime::identity::sha256_hex(&bytes), expected_hash);
            let record: [u8; 124] = bytes.try_into().unwrap();
            assert_eq!(record[0x5e], expected_state);
            assert_eq!(record[0x19], expected_animation);
            assert_eq!(record[0x1b], expected_frame);
            let virtual_height = i32::from_le_bytes(record[0x27..0x2b].try_into().unwrap())
                - i32::from_le_bytes(record[0x11..0x15].try_into().unwrap());
            let mut pixels = vec![0x55; 64 * 48];
            let receipt = draw_supported_mule_record(&resources.mule, &record, [24,18],
                virtual_height, 0, 0, false, 0, 0, &resources.fade, &resources.gla,
                [64,48], 64, &mut pixels).expect("source-proved state/frame draw");
            assert_eq!(receipt.animation, usize::from(expected_animation));
            assert_eq!(receipt.frame_index, usize::from(expected_frame));
            assert!(receipt.components.iter().map(|component| component.pixels_written).sum::<usize>() > 0);
            assert!(receipt.shadows.iter().map(|component| component.pixels_written).sum::<usize>() > 0);
        }
    }

    #[test]
    fn paired_mule_source_frames_match_directional_rendering_and_reject_other_animations() {
        let repository=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let(resources,_,_)=Resources::load(&repository.join("private/integration/game-runtime")).unwrap();
        let bytes=std::fs::read(repository.join("private/review/krell-mule-callback/17804c3e759c449d8e854bb84dc8680c/creatures.bin")).unwrap();
        assert_eq!(genewars_runtime::identity::sha256_hex(&bytes),"222df291bc6afd9fd1f9ab349aabc04d8bd3d71e9ad5943186253b5f22a98fe0");
        for slot in [0,2]{
            let mut record:[u8;124]=bytes[slot*124..(slot+1)*124].try_into().unwrap();assert_eq!(record[0x5e],3);assert!(record[0x19]<=7);
            let height=i32::from_le_bytes(record[39..43].try_into().unwrap())-i32::from_le_bytes(record[17..21].try_into().unwrap());
            let draw=|r:&[u8;124],pixels:&mut Vec<u8>|draw_supported_mule_record(&resources.mule,r,[48,40],height,0,0,false,0,0,&resources.fade,&resources.gla,[96,80],96,pixels);
            let mut pair_pixels=vec![0x55;96*80];let receipt=draw(&record,&mut pair_pixels).unwrap();assert!(receipt.components.iter().map(|c|c.pixels_written).sum::<usize>()>0);
            record[0x5e]=1;let mut approach_pixels=vec![0x55;96*80];draw(&record,&mut approach_pixels).unwrap();assert_eq!(pair_pixels,approach_pixels);
            record[0x5e]=3;record[0x19]=8;let saved=pair_pixels.clone();assert_eq!(draw(&record,&mut pair_pixels),Err(CreatureDrawError::UnsupportedRecordState));assert_eq!(pair_pixels,saved);
            record[0x19]=0;record[0x1b]=255;assert_eq!(draw(&record,&mut pair_pixels),Err(CreatureDrawError::UnsupportedRecordState));assert_eq!(pair_pixels,saved);
            record[0x5e]=2;record[0x1b]=0;assert_eq!(draw(&record,&mut pair_pixels),Err(CreatureDrawError::UnsupportedRecordState));assert_eq!(pair_pixels,saved);
        }
    }

    #[test]
    fn supported_mule_states_reject_unproven_state_animation_pairs_atomically() {
        let repository = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let (resources, _, _) = Resources::load(&repository.join("private/integration/game-runtime"))
            .expect("hash-bound game resources");
        let mut record: [u8; 124] = std::fs::read(repository.join("private/integration/combined-before-mule385/source-after385.bin"))
            .unwrap().try_into().unwrap();
        let mut pixels = vec![0x55; 64 * 48];
        let before = pixels.clone();
        record[0x19] = 7;
        assert_eq!(draw_supported_mule_record(&resources.mule, &record, [24,18], 0, 0,
            0, false, 0, 0, &resources.fade, &resources.gla, [64,48], 64, &mut pixels),
            Err(CreatureDrawError::UnsupportedRecordState));
        assert_eq!(pixels, before);
        record[0x5e] = 24;
        record[0x19] = 17;
        record[0x1b] = 7;
        assert_eq!(draw_supported_mule_record(&resources.mule, &record, [24,18], 0, 0,
            0, false, 0, 0, &resources.fade, &resources.gla, [64,48], 64, &mut pixels),
            Err(CreatureDrawError::UnsupportedRecordState));
        assert_eq!(pixels, before);
        record[0x5e] = 25;
        record[0x19] = 18;
        record[0x1b] = 3;
        assert_eq!(draw_supported_mule_record(&resources.mule, &record, [24,18], 0, 0,
            0, false, 0, 0, &resources.fade, &resources.gla, [64,48], 64, &mut pixels),
            Err(CreatureDrawError::UnsupportedRecordState));
        assert_eq!(pixels, before);
        record[0x5e] = 24;
        record[0x19] = 18;
        record[0x1b] = 0;
        assert_eq!(draw_supported_mule_record(&resources.mule, &record, [24,18], 0, 0,
            0, false, 0, 0, &resources.fade, &resources.gla, [64,48], 64, &mut pixels),
            Err(CreatureDrawError::UnsupportedRecordState));
        assert_eq!(pixels, before);
        record[0x5e] = 25;
        record[0x19] = 17;
        assert_eq!(draw_supported_mule_record(&resources.mule, &record, [24,18], 0, 0,
            0, false, 0, 0, &resources.fade, &resources.gla, [64,48], 64, &mut pixels),
            Err(CreatureDrawError::UnsupportedRecordState));
        assert_eq!(pixels, before);
        record[0x19] = 19;
        record[0x1b] = 4;
        assert_eq!(draw_supported_mule_record(&resources.mule, &record, [24,18], 0, 0,
            0, false, 0, 0, &resources.fade, &resources.gla, [64,48], 64, &mut pixels),
            Err(CreatureDrawError::UnsupportedRecordState));
        assert_eq!(pixels, before);
        record[0x5e] = 0;
        record[0x19] = 8;
        record[0x1b] = 0;
        assert_eq!(draw_supported_mule_record(&resources.mule, &record, [24,18], 0, 0,
            0, false, 0, 0, &resources.fade, &resources.gla, [64,48], 64, &mut pixels),
            Err(CreatureDrawError::UnsupportedRecordState));
        assert_eq!(pixels, before);
    }

    #[test]
    fn ordinary_shade_uses_height_adjusted_row_and_caps_only_above() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..").join("private/integration/game-runtime");
        let (resources, _, _) = Resources::load(&root).expect("hash-bound game resources");
        let mut record = [0u8; 124];
        record[6] = 1; record[0x18] = 4; record[0x19] = 5; record[0x1b] = 2;
        record[0x5e] = 8; record[0x17] = 4; record[0x76] = 100;
        record[0x77] = 255; record[0x7a] = 4;
        let mut pixels = vec![0u8; 64 * 48];
        let r = draw_initial_mule_record(&resources.mule, &record, [24,18], 0, 1,
            0, false, 0, 0, &resources.fade, &resources.gla, [64,48],64,&mut pixels).unwrap();
        assert_eq!(r.effective_shade, 4);
        let mut capped = vec![0u8; 64 * 48];
        let r = draw_initial_mule_record(&resources.mule, &record, [24,18], 16_384, 1,
            0, false, 0, 0, &resources.fade, &resources.gla, [64,48],64,&mut capped).unwrap();
        assert_eq!(r.effective_shade, 31);
        assert_ne!(pixels, capped);
    }

    #[test]
    fn negative_shade_and_unresolved_parity_special_branch_are_atomic_errors() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..").join("private/integration/game-runtime");
        let (resources, _, _) = Resources::load(&root).expect("hash-bound game resources");
        let mut record = [0u8; 124];
        record[6] = 1; record[0x18] = 4; record[0x19] = 5; record[0x1b] = 2;
        record[0x5e] = 8; record[0x17] = 1; record[0x76] = 100;
        record[0x77] = 255; record[0x7a] = 4;
        let mut pixels = vec![0x5a; 64 * 48];
        let before = pixels.clone();
        assert!(matches!(draw_initial_mule_record(&resources.mule,&record,[24,18],-512,1,
            0,false,0,0,&resources.fade,&resources.gla,[64,48],64,&mut pixels),
            Err(CreatureDrawError::NegativeFadeRow(_))));
        assert_eq!(pixels,before);
        record[0x17] = 4;
        record[0x15] = 0x10;
        assert_eq!(draw_initial_mule_record(&resources.mule,&record,[24,18],0,0,
            0,false,0,0,&resources.fade,&resources.gla,[64,48],64,&mut pixels),
            Err(CreatureDrawError::UnsupportedRecordState));
        assert_eq!(pixels,before);
    }

    #[test]
    fn export_source_mule_pixel_oracle_cases_when_requested() {
        let Ok(output_path) = std::env::var("GENEWARS_MULE_ORACLE_OUT") else { return; };
        use std::io::Write;
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..").join("private/integration/game-runtime");
        let (resources, _, _) = Resources::load(&root).expect("hash-bound game resources");
        let mut cases: Vec<(u8,u8,u8,u8,i32,u8,u8,bool,[i32;2],i16,u8)> = vec![
            (4,0,30,127,0,0,0,false,[24,18],0,7),
            (4,7,70,255,64,0,0,true,[2,3],6*128,11),
            (5,0,100,255,128,0,0,false,[24,18],13*128,23),
            (5,3,30,127,64,0,2,true,[-2,1],5*128,31),
            (5,7,100,255,0,1,0,false,[60,44],0,41),
            (6,0,70,255,128,1,0,true,[1,1],7*128,59),
            (6,1,100,255,512,0,1,false,[16,12],14*128,83),
            (4,3,99,255,64,0,3,true,[30,20],-6*128,97),
            (5,5,100,255,0,0,0,false,[32,-2],8*128,127),
            (6,0,30,127,128,1,2,true,[62,46],12*128,151),
            (4,1,50,255,64,0,1,false,[-10,30],4*128,191),
            (5,2,100,255,0,0,2,true,[28,16],13*128,223),
            (0,0,30,127,0,0,0,false,[24,18],0,7),
            (1,0,70,255,64,0,0,true,[2,3],6*128,11),
            (2,0,100,255,128,0,0,false,[24,18],13*128,23),
            (3,0,30,127,64,0,2,true,[-2,1],5*128,31),
            (7,0,100,255,0,0,0,false,[32,12],8*128,127),
        ];
        for animation in 0u8..8 {
            for frame_index in 0u8..8 {
                if animation != 4 || frame_index != 0 {
                    cases.push((animation,frame_index,30,127,0,0,0,false,[24,18],0,7));
                }
            }
        }
        cases.push((5,2,100,255,0,1,0,false,[24,18],0,7));
        cases.push((5,2,100,255,16_384,1,0,false,[24,18],0,7));
        cases.push((5,2,100,255,-64,1,0,false,[24,18],0,7));
        let path = std::path::Path::new(&output_path);
        let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(path)
            .expect("oracle output path must be fresh");
        for (case_index,(animation,frame_index,scale_a,scale_b,virtual_height,shadow_mode,inherited,clip_bit,
             origin,yaw,shadow_color)) in cases.into_iter().enumerate() {
            let raw_shade = match case_index { 80 => 4, 81 => 250, 82 => 1, _ => 0 };
            let mut record = [0u8; 124];
            record[6] = 1; record[0x18] = 4; record[0x19] = animation;
            record[0x1b] = frame_index; record[0x5e] = 8; record[0x76] = scale_a;
            record[0x77] = scale_b; record[0x7a] = 4;
            record[0x17] = raw_shade;
            record[0x39..0x3b].copy_from_slice(&yaw.to_le_bytes());
            let mut pixels = vec![0xa5; 68 * 48 + 4];
            let receipt = draw_initial_mule_record(&resources.mule,&record,origin,virtual_height,
                shadow_mode,inherited,clip_bit,shadow_color,0,&resources.fade,&resources.gla,
                [64,48],68,&mut pixels).expect("source-selected wrapper draw");
            let hex = pixels.iter().map(|v| format!("{v:02x}")).collect::<String>();
            let fg_written = receipt.components.iter().map(|x|x.pixels_written).sum::<usize>();
            let fg_clipped = receipt.components.iter().map(|x|x.pixels_clipped).sum::<usize>();
            let sh_written = receipt.shadows.iter().map(|x|x.pixels_written).sum::<usize>();
            let sh_clipped = receipt.shadows.iter().map(|x|x.pixels_clipped).sum::<usize>();
            writeln!(file,"{{\"animation\":{animation},\"frame\":{frame_index},\"scale_a\":{scale_a},\"scale_b\":{scale_b},\"virtual_height\":{virtual_height},\"raw_shade\":{raw_shade},\"effective_shade\":{},\"shadow_mode\":{shadow_mode},\"inherited\":{inherited},\"clip_bit\":{},\"origin\":[{},{}],\"yaw\":{yaw},\"shadow_color\":{shadow_color},\"fg_written\":{fg_written},\"fg_clipped\":{fg_clipped},\"sh_written\":{sh_written},\"sh_clipped\":{sh_clipped},\"pixels\":\"{hex}\"}}",
                receipt.effective_shade,
                u8::from(clip_bit),origin[0],origin[1]).unwrap();
        }
        // State 1 uses the proved directional selector range 0..7. Exercise
        // every ANB-bounded frame for each selected animation. A yaw sector
        // that would set the wrapper flip for later states is deliberately
        // supplied; this state/animation branch preserves incoming flags.
        for animation in 0u8..8 {
            for frame_index in 0u8..8 {
                let mut record = [0u8; 124];
                record[6] = 1; record[0x18] = 4; record[0x19] = animation;
                record[0x1b] = frame_index; record[0x5e] = 1; record[0x76] = 30;
                record[0x77] = 127; record[0x7a] = 4;
                record[0x39..0x3b].copy_from_slice(&(7 * 128i16).to_le_bytes());
                let mut pixels = vec![0xa5; 68 * 48 + 4];
                let receipt = draw_supported_mule_record(&resources.mule,&record,[24,18],0,
                    0,0,false,7,0,&resources.fade,&resources.gla,[64,48],68,&mut pixels)
                    .expect("source-selected state-1 direction/frame");
                let hex = pixels.iter().map(|v|format!("{v:02x}")).collect::<String>();
                let fg_written = receipt.components.iter().map(|x|x.pixels_written).sum::<usize>();
                let fg_clipped = receipt.components.iter().map(|x|x.pixels_clipped).sum::<usize>();
                let sh_written = receipt.shadows.iter().map(|x|x.pixels_written).sum::<usize>();
                let sh_clipped = receipt.shadows.iter().map(|x|x.pixels_clipped).sum::<usize>();
                writeln!(file,"{{\"source_domain\":\"state1-directional\",\"state\":1,\"animation\":{animation},\"frame\":{frame_index},\"scale_a\":30,\"scale_b\":127,\"virtual_height\":0,\"raw_shade\":0,\"effective_shade\":{},\"shadow_mode\":0,\"inherited\":0,\"clip_bit\":0,\"origin\":[24,18],\"yaw\":896,\"shadow_color\":7,\"fg_written\":{fg_written},\"fg_clipped\":{fg_clipped},\"sh_written\":{sh_written},\"sh_clipped\":{sh_clipped},\"pixels\":\"{hex}\"}}",receipt.effective_shade).unwrap();
            }
        }
        for (snapshot_name, relative_path, expected_hash) in [
            ("approach-366", "combined-before-mule366/source-after366.bin",
                "68c3ab88608abbbd82ff0f9bfd4a740ead1ca5a1e8daf797b2ec6f3b352ab041"),
            ("approach-375", "combined-before-mule375/source-after375.bin",
                "dbd2674496026b73ea92d4932be0508b846281f47565b2de0591c6fa81fd7475"),
            ("feed-385", "combined-before-mule385/source-after385.bin",
                "fac6842c180146b2cee77dcd9da407bed9cbafd57d414b554d00076056d539bd"),
            ("feed-415", "combined-before-mule415/source-after415.bin",
                "99180a26ab1b842d90b4188421d30fa644357efaba1fd033b924bef730f8a935"),
        ] {
            let repository = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
            let bytes = std::fs::read(repository.join("private/integration").join(relative_path))
                .expect("pinned combined-world creature record");
            assert_eq!(genewars_runtime::identity::sha256_hex(&bytes), expected_hash,
                "source fixture hash changed: {snapshot_name}");
            let record: [u8; 124] = bytes.try_into().expect("native creature record size");
            assert_eq!(record[6], 1);
            let state = record[0x5e];
            let animation = record[0x19];
            let frame_index = record[0x1b];
            let virtual_height = i32::from_le_bytes(record[0x27..0x2b].try_into().unwrap())
                - i32::from_le_bytes(record[0x11..0x15].try_into().unwrap());
            let raw_shade = record[0x17];
            let effective_shade = if raw_shade == 0 { 0 } else {
                (i32::from(raw_shade) + (virtual_height >> 7)).min(31)
            };
            assert!(effective_shade >= 0);
            let yaw = i16::from_le_bytes([record[0x39], record[0x3a]]);
            let mut pixels = vec![0xa5; 68 * 48 + 4];
            let receipt = draw_supported_mule_record(&resources.mule, &record, [24,18],
                virtual_height, 0, 0, false, 7, 0, &resources.fade, &resources.gla,
                [64,48], 68, &mut pixels).expect("admitted source snapshot draw");
            let hex = pixels.iter().map(|v| format!("{v:02x}")).collect::<String>();
            let fg_written = receipt.components.iter().map(|x|x.pixels_written).sum::<usize>();
            let fg_clipped = receipt.components.iter().map(|x|x.pixels_clipped).sum::<usize>();
            let sh_written = receipt.shadows.iter().map(|x|x.pixels_written).sum::<usize>();
            let sh_clipped = receipt.shadows.iter().map(|x|x.pixels_clipped).sum::<usize>();
            writeln!(file,"{{\"source_snapshot\":\"{snapshot_name}\",\"state\":{state},\"animation\":{animation},\"frame\":{frame_index},\"scale_a\":{},\"scale_b\":{},\"virtual_height\":{virtual_height},\"raw_shade\":{raw_shade},\"effective_shade\":{},\"shadow_mode\":0,\"inherited\":0,\"clip_bit\":0,\"origin\":[24,18],\"yaw\":{yaw},\"shadow_color\":7,\"fg_written\":{fg_written},\"fg_clipped\":{fg_clipped},\"sh_written\":{sh_written},\"sh_clipped\":{sh_clipped},\"pixels\":\"{hex}\"}}",
                record[0x76], record[0x77], receipt.effective_shade).unwrap();
        }
        // Frames 0 and 2 complete the four-frame metadata extent of the
        // source-selected state-7 animation 19. They vary only the frame byte
        // on the pinned native pass-385 record; they are codec/render-domain
        // checks, not claims about callback timing.
        let repository = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let feed_bytes = std::fs::read(repository.join("private/integration/combined-before-mule385/source-after385.bin"))
            .expect("pinned state-7 feed record");
        assert_eq!(genewars_runtime::identity::sha256_hex(&feed_bytes),
            "fac6842c180146b2cee77dcd9da407bed9cbafd57d414b554d00076056d539bd");
        let feed: [u8;124] = feed_bytes.try_into().unwrap();
        for frame_index in [0u8,2] {
            let mut record=feed;
            record[0x1b]=frame_index;
            let virtual_height=i32::from_le_bytes(record[0x27..0x2b].try_into().unwrap())-
                i32::from_le_bytes(record[0x11..0x15].try_into().unwrap());
            let mut pixels=vec![0xa5;68*48+4];
            let receipt=draw_supported_mule_record(&resources.mule,&record,[24,18],virtual_height,
                0,0,false,7,0,&resources.fade,&resources.gla,[64,48],68,&mut pixels)
                .expect("state-7 animation-19 metadata frame");
            let hex=pixels.iter().map(|v|format!("{v:02x}")).collect::<String>();
            let fg_written=receipt.components.iter().map(|x|x.pixels_written).sum::<usize>();
            let fg_clipped=receipt.components.iter().map(|x|x.pixels_clipped).sum::<usize>();
            let sh_written=receipt.shadows.iter().map(|x|x.pixels_written).sum::<usize>();
            let sh_clipped=receipt.shadows.iter().map(|x|x.pixels_clipped).sum::<usize>();
            let effective=if record[0x17]==0 {0} else {(i32::from(record[0x17])+(virtual_height>>7)).min(31)};
            let yaw=i16::from_le_bytes([record[0x39],record[0x3a]]);
            writeln!(file,"{{\"source_domain\":\"state7-frame-metadata\",\"state\":7,\"animation\":19,\"frame\":{frame_index},\"scale_a\":{},\"scale_b\":{},\"virtual_height\":{virtual_height},\"raw_shade\":{},\"effective_shade\":{},\"shadow_mode\":0,\"inherited\":0,\"clip_bit\":0,\"origin\":[24,18],\"yaw\":{yaw},\"shadow_color\":7,\"fg_written\":{fg_written},\"fg_clipped\":{fg_clipped},\"sh_written\":{sh_written},\"sh_clipped\":{sh_clipped},\"pixels\":\"{hex}\"}}",record[0x76],record[0x77],record[0x17],effective).unwrap();
        }
        //Same-frame neutral hit reaction after actual5876 damage. Other
        //families were frozen in this source follow-up; no native claim.
        {
            let repository=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
            let bytes=std::fs::read(repository.join("private/review/neutral-hit-reaction-193/0e9f48028d9c4d95b20da0eb8c68991e/frame-5876/creatures.bin")).unwrap();
            let record:[u8;124]=bytes[5*124..6*124].try_into().unwrap();
            assert_eq!(genewars_runtime::identity::sha256_hex(&record),"7459f0687ed74d3ea84241dfb03ad762dc5ccf7e29c6b67b5fe2b8bb3832c599");
            assert_eq!((record[24],record[94],record[25],record[27]),(4,20,16,1));
            let virtual_height=i32::from_le_bytes(record[39..43].try_into().unwrap())-i32::from_le_bytes(record[17..21].try_into().unwrap());
            let mut pixels=vec![0xa5;68*48+4];
            let receipt=draw_supported_mule_record(&resources.mule,&record,[24,18],virtual_height,0,0,false,0,0,&resources.fade,&resources.gla,[64,48],68,&mut pixels).unwrap();
            assert!(receipt.components.iter().map(|x|x.pixels_written).sum::<usize>()>0);
            let hex=pixels.iter().map(|v|format!("{v:02x}")).collect::<String>();
            let fg_written=receipt.components.iter().map(|x|x.pixels_written).sum::<usize>();
            let fg_clipped=receipt.components.iter().map(|x|x.pixels_clipped).sum::<usize>();
            let sh_written=receipt.shadows.iter().map(|x|x.pixels_written).sum::<usize>();
            let sh_clipped=receipt.shadows.iter().map(|x|x.pixels_clipped).sum::<usize>();
            writeln!(file,"{{\"source_domain\":\"neutral-state20-hit-reaction\",\"state\":20,\"animation\":16,\"frame\":1,\"scale_a\":{},\"scale_b\":{},\"virtual_height\":{virtual_height},\"raw_shade\":{},\"effective_shade\":{},\"shadow_mode\":0,\"inherited\":0,\"clip_bit\":0,\"origin\":[24,18],\"yaw\":{},\"shadow_color\":0,\"fg_written\":{fg_written},\"fg_clipped\":{fg_clipped},\"sh_written\":{sh_written},\"sh_clipped\":{sh_clipped},\"pixels\":\"{hex}\"}}",record[118],record[119],record[23],receipt.effective_shade,i16::from_le_bytes([record[57],record[58]])).unwrap();
            for(offset,value)in[(25,23),(25,15),(27,255),(24,0)]{
                let mut bad=record;bad[offset]=value;let saved=pixels.clone();
                assert_eq!(draw_supported_mule_record(&resources.mule,&bad,[24,18],virtual_height,0,0,false,0,0,&resources.fade,&resources.gla,[64,48],68,&mut pixels),Err(CreatureDrawError::UnsupportedRecordState));assert_eq!(pixels,saved);
            }
        }
        // Mortality inputs are literal, source-traced 124-byte records. The
        // frame substitutions below check every frame admitted by each ANB
        // animation; only the unmodified fixture frames claim lifecycle
        // reachability (state 24/frame 1 and state 25/frame 0).
        for (snapshot_name, relative_path, expected_hash, state, animation, frame_count) in [
            ("state24-frame1", "private/review/mule-mortality/frame9053/source-after9053.bin",
                "05a69c5622613cc9da069a22597b18dbc7d0918e540e60931a21ca5df346bc0d", 24u8, 17u8, 7u8),
            ("state25-frame0", "private/review/mule-mortality/frame9059/source-after9059.bin",
                "af935ddafbd0905e5a9af69ad4070ad97af1f459d66ddede60e2cbece09e455a", 25u8, 18u8, 3u8),
        ] {
            let repository = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
            let bytes = std::fs::read(repository.join(relative_path)).expect("pinned mortality source record");
            assert_eq!(genewars_runtime::identity::sha256_hex(&bytes), expected_hash);
            let base: [u8; 124] = bytes.try_into().expect("native creature record size");
            assert_eq!(base[0x5e], state);
            assert_eq!(base[0x19], animation);
            for frame_index in 0..frame_count {
                let mut record = base;
                record[0x1b] = frame_index;
                let virtual_height = i32::from_le_bytes(record[0x27..0x2b].try_into().unwrap())
                    - i32::from_le_bytes(record[0x11..0x15].try_into().unwrap());
                let mut pixels = vec![0xa5; 68 * 48 + 4];
                let receipt = draw_supported_mule_record(&resources.mule, &record, [24,18],
                    virtual_height, 0, 0, false, 0, 0, &resources.fade, &resources.gla,
                    [64,48], 68, &mut pixels).expect("source-bound mortality animation frame");
                let hex = pixels.iter().map(|v| format!("{v:02x}")).collect::<String>();
                let fg_written = receipt.components.iter().map(|x|x.pixels_written).sum::<usize>();
                let fg_clipped = receipt.components.iter().map(|x|x.pixels_clipped).sum::<usize>();
                let sh_written = receipt.shadows.iter().map(|x|x.pixels_written).sum::<usize>();
                let sh_clipped = receipt.shadows.iter().map(|x|x.pixels_clipped).sum::<usize>();
                writeln!(file,"{{\"source_domain\":\"mortality-frame-domain\",\"source_snapshot\":\"{snapshot_name}\",\"state\":{state},\"animation\":{animation},\"frame\":{frame_index},\"scale_a\":{},\"scale_b\":{},\"virtual_height\":{virtual_height},\"raw_shade\":{},\"effective_shade\":{},\"shadow_mode\":0,\"inherited\":0,\"clip_bit\":0,\"origin\":[24,18],\"yaw\":{},\"shadow_color\":0,\"fg_written\":{fg_written},\"fg_clipped\":{fg_clipped},\"sh_written\":{sh_written},\"sh_clipped\":{sh_clipped},\"pixels\":\"{hex}\"}}",
                    record[0x76], record[0x77], record[0x17], receipt.effective_shade,
                    i16::from_le_bytes([record[0x39], record[0x3a]])).unwrap();
            }
        }
    }
}

#[cfg(test)]
mod pair_stop_render_tests {
 use super::*;
 use crate::runtime_resources::Resources;
 #[test]
 fn actual_pair_stop_renders_peer_direction_and_current_animation22_with_source_flip() {
  let repository=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
  let(resources,_,_)=Resources::load(&repository.join("private/integration/game-runtime")).unwrap();
  let bytes=std::fs::read(repository.join("private/review/krell-mule-callback/5c2e6dc1b906478fb0681aea922061d0/creatures.bin")).unwrap();
  assert_eq!(genewars_runtime::identity::sha256_hex(&bytes),"9ec31b51523675e6baa9e560703fc200ae08290dc98a6b6879667b6055bffaed");
  for(slot,animation,frame)in[(0,6,2),(2,22,1)]{
   let mut r:[u8;124]=bytes[slot*124..(slot+1)*124].try_into().unwrap();assert_eq!((r[94],r[25],r[27]),(10,animation,frame));
   let height=i32::from_le_bytes(r[39..43].try_into().unwrap())-i32::from_le_bytes(r[17..21].try_into().unwrap());
   let draw=|record:&[u8;124],inherited,pixels:&mut Vec<u8>|draw_supported_mule_record(&resources.mule,record,[48,40],height,0,inherited,false,0,0,&resources.fade,&resources.gla,[96,80],96,pixels);
   let mut pixels=vec![0x55;96*80];let receipt=draw(&r,0,&mut pixels).unwrap();
   assert!(receipt.components.iter().map(|c|c.pixels_written).sum::<usize>()>0);
   assert!(receipt.shadows.iter().map(|c|c.pixels_written).sum::<usize>()>0);
   if slot==0{
    //Peer's earlier callback cannot advance its animation after the later pair write.
    let mut directional=r;directional[94]=3;let mut expected=vec![0x55;96*80];draw(&directional,0,&mut expected).unwrap();assert_eq!(pixels,expected);
   }else{
    //59470: highanimation yawsector6 overrides wrapper toXflip; sector1 preservesit.
    let mut explicit_flip=vec![0x55;96*80];draw(&r,1,&mut explicit_flip).unwrap();
    r[57..59].copy_from_slice(&768i16.to_le_bytes());let mut source_flip=vec![0x55;96*80];draw(&r,0,&mut source_flip).unwrap();assert_eq!(source_flip,explicit_flip);
    assert_ne!(source_flip,pixels,"actualanimation22 fixture must discriminate wrapperflip");
   }
   let saved=pixels.clone();r[25]=23;assert_eq!(draw(&r,0,&mut pixels),Err(CreatureDrawError::UnsupportedRecordState));assert_eq!(pixels,saved);
   r[25]=22;r[27]=255;assert_eq!(draw(&r,0,&mut pixels),Err(CreatureDrawError::UnsupportedRecordState));assert_eq!(pixels,saved);
  }
 }
 #[test]
 fn actual_birth_peer_state8_retains_animation22_wrapper_until_own_callback() {
  let repository=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
  let(resources,_,_)=Resources::load(&repository.join("private/integration/game-runtime")).unwrap();
  let bytes=std::fs::read(repository.join("private/review/krell-mule-callback/f101bdcfa78248ee816e95af1df4e9ca/creatures.bin")).unwrap();
  assert_eq!(genewars_runtime::identity::sha256_hex(&bytes),"d503a8c655604e65eff38ec2704c85c13120b35b261b76ee5d312dfe3a298312");
  let peer:[u8;124]=bytes[..124].try_into().unwrap();assert_eq!((peer[94],peer[25],peer[27]),(8,22,0));
  let height=i32::from_le_bytes(peer[39..43].try_into().unwrap())-i32::from_le_bytes(peer[17..21].try_into().unwrap());
  let draw=|record:&[u8;124],pixels:&mut Vec<u8>|draw_supported_mule_record(&resources.mule,record,[48,40],height,0,0,false,0,0,&resources.fade,&resources.gla,[96,80],96,pixels);
  let mut actual=vec![0x55;96*80];let receipt=draw(&peer,&mut actual).unwrap();
  assert!(receipt.components.iter().map(|c|c.pixels_written).sum::<usize>()>0);assert!(receipt.shadows.iter().map(|c|c.pixels_written).sum::<usize>()>0);
  let mut stopped=peer;stopped[94]=10;let mut expected=vec![0x55;96*80];draw(&stopped,&mut expected).unwrap();assert_eq!(actual,expected);
  let mut invalid=peer;invalid[25]=23;let saved=actual.clone();assert_eq!(draw(&invalid,&mut actual),Err(CreatureDrawError::UnsupportedRecordState));assert_eq!(actual,saved);
 }
}

#[cfg(test)] mod owned_render_tests {
 use super::*;
 #[test] fn captured_owned_mule_uses_owner_palette_and_source_fade_gate() {
  let p=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
  let(resources,_,_)=crate::runtime_resources::Resources::load(&p.join("private/integration/game-runtime")).unwrap();
  let raw=std::fs::read(p.join("private/review/krell-production-100/frame-2724-slot-7-owned-after/creatures.bin")).unwrap();
  let captured:[u8;124]=raw[7*124..8*124].try_into().unwrap();assert_eq!((captured[24],captured[94],captured[25]),(0,8,3));
  let hostile_pool=std::fs::read(p.join("private/review/owned-mule-captured-width/42967be6fee844c78bec6cd127c7158f/creatures.bin")).unwrap();
  let hostile:[u8;124]=hostile_pool[124..248].try_into().unwrap();
  assert_eq!((hostile[94],hostile[0x78],hostile[0x60],hostile[0x61]),(1,7,0xc8,0));
  let attack_pool=std::fs::read(p.join("private/review/neutral-mule-captured-width/2589f8c58e974fcc93f578007a2cc239/creatures.bin")).unwrap();
  let mut attack:[u8;124]=attack_pool[5*124..6*124].try_into().unwrap();
  assert_eq!((attack[94],attack[25],attack[27],attack[0x60],attack[0x61]),(21,14,1,0x88,0x40));
  //Source-equal neutral attack body with explicit owned palette/task ports.
  //Compositor evidence only; native owned combat is not claimed here.
  attack[24]=0;attack[0x78]=7;attack[0x61]=0;
  let target:[u8;3]=attack[0x55..0x58].try_into().unwrap();attack[0x58..0x5b].copy_from_slice(&target);
  let mut lines=Vec::new();let mut owner_images=Vec::new();
  for owner in 0..4 {for variant in 0..7 {
   let mut r=if variant==6{attack}else if variant==5{hostile}else{captured};r[24]=owner;let mut height=if variant>=5{0}else{4312};let mut inherited=0;
   match variant {1=>{height=0;r[23]=5;},2=>{height=0;r[23]=5;r[21]|=2;inherited=1;},3=>{height=0;r[118]=100;r[119]=255;inherited=2;},4=>{r[121]=3;r[0x62..0x64].copy_from_slice(&127i16.to_le_bytes());r[0x68..0x6a].copy_from_slice(&16i16.to_le_bytes());},_=>{}}
   let mut pixels=vec![0xa5;68*128+4];let receipt=draw_supported_mule_record(&resources.mule,&r,[24,90],height,0,inherited,false,23,3,&resources.fade,&resources.gla,[64,128],68,&mut pixels).unwrap();
   assert!(receipt.components.iter().map(|p|p.pixels_written).sum::<usize>()>0);if variant==0{owner_images.push(pixels.clone());}
   let hex=|b:&[u8]|b.iter().map(|x|format!("{x:02x}")).collect::<String>();
   lines.push(format!("{{\"owner\":{owner},\"variant\":{variant},\"record\":\"{}\",\"height\":{height},\"inherited\":{inherited},\"pixels\":\"{}\"}}",hex(&r),hex(&pixels)));
  }}
  for pair in owner_images.windows(2){assert_ne!(pair[0],pair[1]);}
  if let Some(output)=std::env::var_os("GENEWARS_OWNED_RENDER_OUTPUT"){assert!(!std::path::Path::new(&output).exists());std::fs::write(output,lines.join("\n")+"\n").unwrap();}
  //Unsupported task/reference/flag shapes must reject before destination writes.
  for(offset,value)in[(0x78,0),(0x52,0),(0x57,0),(0x5a,0),(0x61,0x40),(0x60,8)]{
   let mut bad=hostile;bad[offset]=value;let mut out=vec![0x55;68*128];let saved=out.clone();
   assert_eq!(draw_supported_mule_record(&resources.mule,&bad,[24,90],0,0,0,false,23,3,&resources.fade,&resources.gla,[64,128],68,&mut out),Err(CreatureDrawError::UnsupportedRecordState));assert_eq!(out,saved);
  }
  for(offset,value)in[(0x19,7),(0x19,16),(0x5a,0),(0x61,0x40),(0x60,0xc8)]{
   let mut bad=attack;bad[offset]=value;let mut out=vec![0x55;68*128];let saved=out.clone();
   assert_eq!(draw_supported_mule_record(&resources.mule,&bad,[24,90],0,0,0,false,23,3,&resources.fade,&resources.gla,[64,128],68,&mut out),Err(CreatureDrawError::UnsupportedRecordState));assert_eq!(out,saved);
  }
  let mut icon_required=captured;icon_required[121]=3;
  icon_required[0x62..0x64].copy_from_slice(&127i16.to_le_bytes());icon_required[0x68..0x6a].copy_from_slice(&64i16.to_le_bytes());
  let mut out=vec![0x55;68*128];let prior=out.clone();
  assert_eq!(draw_supported_mule_record(&resources.mule,&icon_required,[24,90],4312,0,0,false,23,3,&resources.fade,&resources.gla,[64,128],68,&mut out),Err(CreatureDrawError::UnsupportedRecordState));assert_eq!(out,prior);
  let mut unsupported=captured;unsupported[123]=128;let mut pixels=vec![0x55;68*128];let saved=pixels.clone();
  assert_eq!(draw_supported_mule_record(&resources.mule,&unsupported,[24,90],4312,0,0,false,23,3,&resources.fade,&resources.gla,[64,128],68,&mut pixels),Err(CreatureDrawError::UnsupportedRecordState));assert_eq!(pixels,saved);
 }
}

#[cfg(test)]mod corpse_render_tests{
 use super::*;
 #[test]fn source_equal_corpse_arrival_renders_same_animation_as_plant_feeding(){
  let p=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
  let(res,_,_)=crate::runtime_resources::Resources::load(&p.join("private/integration/game-runtime")).unwrap();
  let bytes=std::fs::read(p.join("private/review/isolated-corpse-arrival-after-165/creatures.bin")).unwrap();
  assert_eq!(genewars_runtime::identity::sha256_hex(&bytes),"8ec16ef0e98809378c50f622026d6cb8e592c02a1c943794dee2b3624b3da557");
  let mut r:[u8;124]=bytes[8*124..9*124].try_into().unwrap();assert_eq!((r[94],r[25],r[27]),(5,19,1));
  let height=i32::from_le_bytes(r[39..43].try_into().unwrap())-i32::from_le_bytes(r[17..21].try_into().unwrap());
  let draw=|r:&[u8;124],out:&mut Vec<u8>|draw_supported_mule_record(&res.mule,r,[48,40],height,0,0,false,0,0,&res.fade,&res.gla,[96,80],96,out);
  for state in [4,5]{r[94]=state;for frame in 0..4{r[27]=frame;let mut actual=vec![0x55;96*80];let receipt=draw(&r,&mut actual).unwrap();assert!(receipt.components.iter().map(|x|x.pixels_written).sum::<usize>()>0);
   let mut plant=r;plant[94]=7;let mut expected=vec![0x55;96*80];draw(&plant,&mut expected).unwrap();assert_eq!(actual,expected);
  }}
  for(offset,value)in[(25,18),(27,4),(24,0)]{let mut invalid=r;invalid[offset]=value;let mut out=vec![0x55;96*80];let saved=out.clone();assert_eq!(draw(&invalid,&mut out),Err(CreatureDrawError::UnsupportedRecordState));assert_eq!(out,saved);}
 }
}
