//! Bounded family-8 ordinary rendering for the pinned ETHEREAL.ANB.
//!
//! This accepts an explicit family-8 record, projected ground origin, fade
//! table, and framebuffer. It implements selected, source-checked states on
//! the ordinary 0x10EA0 path; selector decorations and unadmitted states are
//! rejected. Caller ordering, visibility, and update timing remain external.

use genewars_assets::{AnbSections, IndexedSprite, SpriteTable};
use genewars_render::{
    entity_shadow::draw_normal_component_shadow, PlantColorRoute,
    terrain::TerrainFadeTable, terrain_modes::MainGlaTable,
};
use genewars_runtime::identity::sha256_hex;

pub const ETHEREAL_DECODED_BYTES: usize = 91_123;
pub const ETHEREAL_SHA256: &str = "318dcb26efa6968c34357e67c05b6514efe7d7a8b4052a4e811a851a2ba01bc4";

const STATE4_FRAMES: [(i16, i16, u8, u8, i16, i16); 11] = [
    (116, 102, 116, 102, -60, -72), (117, 102, 117, 102, -60, -72),
    (117, 103, 117, 103, -60, -73), (118, 102, 118, 102, -59, -72),
    (118, 103, 118, 103, -58, -72), (117, 103, 117, 103, -56, -71),
    (118, 104, 118, 104, -56, -71), (118, 103, 118, 103, -56, -70),
    (118, 104, 118, 104, -56, -70), (118, 104, 118, 104, -55, -69),
    (117, 103, 117, 103, -54, -68),
];

#[derive(Debug, Clone)]
struct Family8Frame {
    display: (i16, i16),
    sprite: IndexedSprite,
    local: [i16; 2],
    component_flags: u8,
}

/// Hash-admitted ETHEREAL bytes and the eleven decoded frames for selector
/// zero. Its animation base is the eighth ten-byte ANB descriptor because the
/// family-8 loader binds `0x116820 + 0x50` for selectors 0 and 2.
pub struct EtherealAnimationDatabase {
    frames: Vec<Family8Frame>,
    selector1_directions: Vec<Family8Frame>,
}

impl EtherealAnimationDatabase {
    pub fn from_bytes(anb: &[u8]) -> Result<Self, Family8SceneError> {
        validate_resource(anb)?;
        if sha256_hex(anb) != ETHEREAL_SHA256 { return Err(Family8SceneError::ResourceIdentityMismatch); }
        let sections = AnbSections::parse(anb).map_err(|e| Family8SceneError::Anb(e.to_string()))?;
        let metadata = sections.ui_metadata().map_err(|e| Family8SceneError::Anb(e.to_string()))?;
        let idle = metadata.animation(0).map_err(|e| Family8SceneError::Anb(e.to_string()))?;
        let idle_frame_record = metadata.frame_at(idle.first_frame_offset)
            .map_err(|e| Family8SceneError::Anb(e.to_string()))?;
        let idle_component = metadata.frame_component(idle_frame_record, 0)
            .map_err(|e| Family8SceneError::Anb(e.to_string()))?;
        if (idle.first_frame_offset, idle.last_frame_offset, idle.trailing_word) != (0, 0, 1)
            || idle_frame_record.component_count != 1
            || idle_frame_record.display_dimensions() != (27, 26)
            || idle_component.flags != 0
            || idle_component.local_position() != (-13, -18)
        {
            return Err(Family8SceneError::UnsupportedFrameShape);
        }
        let mut selector1_directions = Vec::with_capacity(8);
        for direction in 0..8u8 {
            let cached = parse_family8_frame(anb, direction, 0)?;
            if cached.component_flags & !1 != 0 || cached.sprite.width != 27 || cached.sprite.height != if direction == 0 { 26 } else { 27 }
                || cached.local != [ -13, if direction == 0 { -18 } else { -19 } ]
            {
                return Err(Family8SceneError::UnsupportedFrameShape);
            }
            selector1_directions.push(cached);
        }
        let mut frames = Vec::with_capacity(STATE4_FRAMES.len());
        for (index, expected) in STATE4_FRAMES.iter().enumerate() {
            let frame = parse_family8_frame(anb, 8, index as u8)?;
            if frame.display != (expected.0, expected.1) || frame.component_flags != 0
                || (frame.sprite.width, frame.sprite.height, frame.local[0], frame.local[1])
                    != (expected.2, expected.3, expected.4, expected.5)
            {
                return Err(Family8SceneError::UnsupportedFrameShape);
            }
            frames.push(frame);
        }
        Ok(Self { frames, selector1_directions })
    }

    pub fn frame_count(&self) -> usize { self.frames.len() }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Family8SceneError {
    ResourceLength(usize),
    Anb(String),
    ResourceIdentityMismatch,
    WrongRecordKind(u8),
    UnsupportedSelector(u8),
    UnsupportedState(u8),
    UnsupportedEntityFlags(u16),
    UnsupportedAnimation { animation: u8, frame: u8 },
    UnsupportedFrameShape,
    NegativeFadeRow(i32),
    Draw(String),
}

impl core::fmt::Display for Family8SceneError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for Family8SceneError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Family8DrawReceipt {
    pub slot: usize,
    pub animation: u8,
    pub global_animation: u8,
    pub frame: u8,
    pub ground_origin: [i16; 2],
    pub virtual_height: i32,
    pub color_origin: [i16; 2],
    pub effective_shade: i32,
    pub route: PlantColorRoute,
    pub shadow_pixels: usize,
    pub shadow_clipped: usize,
    pub color_pixels: usize,
    pub color_clipped: usize,
}

/// Draw exactly one active family-8 selector-0, state-4 factory record.
///
/// The record must retain the factory's initial flag word `0x4020` and local
/// animation byte zero. The ordinary wrapper calls the shadow pass
/// at the ground origin, computes virtual height as `+0x27 - +0x11`, then
/// draws the indexed8 foreground at signed16 `(origin_y - (height >> 6))`.
/// Shade zero selects row zero; nonzero shade adds the arithmetic-shifted
/// height (`height >> 7`). A direct route needs no fade row; a faded route is
/// admitted only for a nonnegative row below 30. Errors leave the frame intact.
pub fn draw_family8_selector2_state10(
    resource:&EtherealAnimationDatabase,record:&[u8;102],origin:[i16;2],
    fade:&TerrainFadeTable<'_>,gla:&MainGlaTable<'_>,destination:&mut[u8],
    clip:[usize;2],pitch:usize,
)->Result<Family8DrawReceipt,Family8SceneError>{
    if record[6]!=8{return Err(Family8SceneError::WrongRecordKind(record[6]));}
    if record[0x55]!=2{return Err(Family8SceneError::UnsupportedSelector(record[0x55]));}
    if record[0x56]!=10{return Err(Family8SceneError::UnsupportedState(record[0x56]));}
    let flags=u16::from_le_bytes([record[0x15],record[0x16]]);
    if flags!=0x4020{return Err(Family8SceneError::UnsupportedEntityFlags(flags));}
    if record[0x19]!=0||usize::from(record[0x1b])>=resource.frames.len(){return Err(Family8SceneError::UnsupportedAnimation{animation:record[0x19],frame:record[0x1b]});}
    draw_family8_common(&resource.frames,record,8,origin,fade,gla,destination,clip,pitch)
}

pub fn draw_family8_selector0_state4(
    resource: &EtherealAnimationDatabase,
    record: &[u8; 102],
    origin: [i16; 2],
    fade: &TerrainFadeTable<'_>,
    gla: &MainGlaTable<'_>,
    destination: &mut [u8],
    clip: [usize; 2],
    pitch: usize,
) -> Result<Family8DrawReceipt, Family8SceneError> {
    if record[6] != 8 { return Err(Family8SceneError::WrongRecordKind(record[6])); }
    if record[0x55] != 0 { return Err(Family8SceneError::UnsupportedSelector(record[0x55])); }
    if record[0x56] != 4 { return Err(Family8SceneError::UnsupportedState(record[0x56])); }
    let flags = u16::from_le_bytes([record[0x15], record[0x16]]);
    if flags != 0x4020 { return Err(Family8SceneError::UnsupportedEntityFlags(flags)); }
    draw_family8_common(&resource.frames, record, 8, origin, fade, gla, destination, clip, pitch)
}

/// Draw the ordinary selector-0/state-5 arrival record. Its arrival callback
/// clears the factory word's high-byte 0x40 bit, leaving 0x0020. No selector-0
/// state-specific decoration branch is taken for state 5 in 0x10EA0. Animation
/// remains local byte zero; the byte frame index is admitted only within the
/// global animation-8 frame list.
pub fn draw_family8_selector0_state5(
    resource: &EtherealAnimationDatabase,
    record: &[u8; 102],
    origin: [i16; 2],
    fade: &TerrainFadeTable<'_>,
    gla: &MainGlaTable<'_>,
    destination: &mut [u8],
    clip: [usize; 2],
    pitch: usize,
) -> Result<Family8DrawReceipt, Family8SceneError> {
    if record[6] != 8 { return Err(Family8SceneError::WrongRecordKind(record[6])); }
    if record[0x55] != 0 { return Err(Family8SceneError::UnsupportedSelector(record[0x55])); }
    if record[0x56] != 5 { return Err(Family8SceneError::UnsupportedState(record[0x56])); }
    let flags = u16::from_le_bytes([record[0x15], record[0x16]]);
    if flags != 0x0020 { return Err(Family8SceneError::UnsupportedEntityFlags(flags)); }
    if record[0x19] != 0 || usize::from(record[0x1b]) >= resource.frames.len() {
        return Err(Family8SceneError::UnsupportedAnimation { animation: record[0x19], frame: record[0x1b] });
    }
    draw_family8_common(&resource.frames, record, 8, origin, fade, gla, destination, clip, pitch)
}

/// Draw a source-captured selector-0/state-8 parent through the ordinary
/// 0x10EA0 path. For substate zero, that routine's state dispatch sends state
/// 8 directly to its common color pass; the selected frame remains global
/// animation 8, indexed by the record's frame byte. The admission here is
/// intentionally limited to selector 0, substate 0, flag word 0x0020, and a
/// frame present in the pinned animation table.
pub fn draw_family8_selector0_state8(
    resource: &EtherealAnimationDatabase,
    record: &[u8; 102],
    origin: [i16; 2],
    fade: &TerrainFadeTable<'_>,
    gla: &MainGlaTable<'_>,
    destination: &mut [u8],
    clip: [usize; 2],
    pitch: usize,
) -> Result<Family8DrawReceipt, Family8SceneError> {
    draw_family8_selector0_parent(resource, record, 8, origin, fade, gla, destination, clip, pitch)
}

/// Draw a source-captured selector-0/state-9 parent through the same ordinary
/// path. State 9's zero-substate branch also bypasses the state-specific
/// decoration handlers in 0x10EA0. No later ascent/retirement callback is
/// implied by this renderer admission.
pub fn draw_family8_selector0_state9(
    resource: &EtherealAnimationDatabase,
    record: &[u8; 102],
    origin: [i16; 2],
    fade: &TerrainFadeTable<'_>,
    gla: &MainGlaTable<'_>,
    destination: &mut [u8],
    clip: [usize; 2],
    pitch: usize,
) -> Result<Family8DrawReceipt, Family8SceneError> {
    draw_family8_selector0_parent(resource, record, 9, origin, fade, gla, destination, clip, pitch)
}

fn draw_family8_selector0_parent(
    resource: &EtherealAnimationDatabase,
    record: &[u8; 102],
    expected_state: u8,
    origin: [i16; 2],
    fade: &TerrainFadeTable<'_>,
    gla: &MainGlaTable<'_>,
    destination: &mut [u8],
    clip: [usize; 2],
    pitch: usize,
) -> Result<Family8DrawReceipt, Family8SceneError> {
    if record[6] != 8 { return Err(Family8SceneError::WrongRecordKind(record[6])); }
    if record[0x55] != 0 { return Err(Family8SceneError::UnsupportedSelector(record[0x55])); }
    if record[0x56] != expected_state { return Err(Family8SceneError::UnsupportedState(record[0x56])); }
    let flags = u16::from_le_bytes([record[0x15], record[0x16]]);
    if flags != 0x0020 { return Err(Family8SceneError::UnsupportedEntityFlags(flags)); }
    if record[0x19] != 0 || usize::from(record[0x1b]) >= resource.frames.len() {
        return Err(Family8SceneError::UnsupportedAnimation { animation: record[0x19], frame: record[0x1b] });
    }
    draw_family8_common(&resource.frames, record, 8, origin, fade, gla, destination, clip, pitch)
}


/// Draw the factory-initialized selector-1/state-0 record through the common
/// 0x10EA0 color path. Selector 1 binds ETHEREAL animation table base without
/// the +0x50 selector-0 adjustment; local animation/frame are restricted to
/// the proven initial pair 0/0.
pub fn draw_family8_selector1_state0(
    resource: &EtherealAnimationDatabase,
    record: &[u8; 102],
    origin: [i16; 2],
    fade: &TerrainFadeTable<'_>,
    gla: &MainGlaTable<'_>,
    destination: &mut [u8],
    clip: [usize; 2],
    pitch: usize,
) -> Result<Family8DrawReceipt, Family8SceneError> {
    if record[6] != 8 { return Err(Family8SceneError::WrongRecordKind(record[6])); }
    if record[0x55] != 1 { return Err(Family8SceneError::UnsupportedSelector(record[0x55])); }
    if record[0x56] != 0 { return Err(Family8SceneError::UnsupportedState(record[0x56])); }
    let flags = u16::from_le_bytes([record[0x15], record[0x16]]);
    if flags != 0x4020 { return Err(Family8SceneError::UnsupportedEntityFlags(flags)); }
    if record[0x1b] != 0 || usize::from(record[0x19]) >= resource.selector1_directions.len() {
        return Err(Family8SceneError::UnsupportedAnimation { animation: record[0x19], frame: record[0x1b] });
    }
    draw_family8_common(std::slice::from_ref(&resource.selector1_directions[usize::from(record[0x19])]),
        record, record[0x19], origin, fade, gla, destination, clip, pitch)
}

/// Draw a selector-1 state-1 record after the pinned 0x114B0 transition.
/// The transition leaves the family-8 flag word at 0x0020; the common
/// 0x10EA0 path selects the same direction-indexed one-frame animations as
/// state 0 and takes no state-specific decoration branch for selector 1.
pub fn draw_family8_selector1_state1(
    resource: &EtherealAnimationDatabase,
    record: &[u8; 102],
    origin: [i16; 2],
    fade: &TerrainFadeTable<'_>,
    gla: &MainGlaTable<'_>,
    destination: &mut [u8],
    clip: [usize; 2],
    pitch: usize,
) -> Result<Family8DrawReceipt, Family8SceneError> {
    if record[6] != 8 { return Err(Family8SceneError::WrongRecordKind(record[6])); }
    if record[0x55] != 1 { return Err(Family8SceneError::UnsupportedSelector(record[0x55])); }
    if record[0x56] != 1 { return Err(Family8SceneError::UnsupportedState(record[0x56])); }
    let flags = u16::from_le_bytes([record[0x15], record[0x16]]);
    if flags != 0x0020 { return Err(Family8SceneError::UnsupportedEntityFlags(flags)); }
    if record[0x1b] != 0 || usize::from(record[0x19]) >= resource.selector1_directions.len() {
        return Err(Family8SceneError::UnsupportedAnimation { animation: record[0x19], frame: record[0x1b] });
    }
    draw_family8_common(std::slice::from_ref(&resource.selector1_directions[usize::from(record[0x19])]),
        record, record[0x19], origin, fade, gla, destination, clip, pitch)
}

/// Draw a selector-1/state-3 record through the same ordinary 0x10EA0 route.
/// The common wrapper selects the direction-indexed frame and performs its
/// shadow/height/color passes before the later state-specific decoration
/// dispatch. The return path is admitted with either the pre-return 0x0020
/// word or the observed 0x4020 substate-2 word; its common color wrapper does
/// not branch on bit 0x4000. The direction-animation frame domain remains
/// validated by the resource binding.
pub fn draw_family8_selector1_state3(
    resource: &EtherealAnimationDatabase,
    record: &[u8; 102],
    origin: [i16; 2],
    fade: &TerrainFadeTable<'_>,
    gla: &MainGlaTable<'_>,
    destination: &mut [u8],
    clip: [usize; 2],
    pitch: usize,
) -> Result<Family8DrawReceipt, Family8SceneError> {
    if record[6] != 8 { return Err(Family8SceneError::WrongRecordKind(record[6])); }
    if record[0x55] != 1 { return Err(Family8SceneError::UnsupportedSelector(record[0x55])); }
    if record[0x56] != 3 { return Err(Family8SceneError::UnsupportedState(record[0x56])); }
    let flags = u16::from_le_bytes([record[0x15], record[0x16]]);
    if flags != 0x0020 && flags != 0x4020 { return Err(Family8SceneError::UnsupportedEntityFlags(flags)); }
    if record[0x1b] != 0 || usize::from(record[0x19]) >= resource.selector1_directions.len() {
        return Err(Family8SceneError::UnsupportedAnimation { animation: record[0x19], frame: record[0x1b] });
    }
    draw_family8_common(std::slice::from_ref(&resource.selector1_directions[usize::from(record[0x19])]),
        record, record[0x19], origin, fade, gla, destination, clip, pitch)
}

fn draw_family8_common(
    frames: &[Family8Frame],
    record: &[u8; 102],
    global_animation: u8,
    origin: [i16; 2],
    fade: &TerrainFadeTable<'_>,
    gla: &MainGlaTable<'_>,
    destination: &mut [u8],
    clip: [usize; 2],
    pitch: usize,
) -> Result<Family8DrawReceipt, Family8SceneError> {
    let entity_flags = u16::from_le_bytes([record[0x15], record[0x16]]);
    if record[0x15] & 0x10 != 0 { return Err(Family8SceneError::UnsupportedEntityFlags(entity_flags)); }
    let animation = record[0x19];
    let frame = record[0x1b];
    if usize::from(frame) >= frames.len() {
        return Err(Family8SceneError::UnsupportedAnimation { animation, frame });
    }
    let cached = frames.get(usize::from(frame))
        .ok_or(Family8SceneError::UnsupportedAnimation { animation, frame })?;

    let virtual_height = i32::from_le_bytes(record[0x27..0x2b].try_into().expect("fixed family8 record"))
        .wrapping_sub(i32::from_le_bytes(record[0x11..0x15].try_into().expect("fixed family8 record")));
    let height_delta = virtual_height >> 6;
    let color_y = (i32::from(origin[1]).wrapping_sub(height_delta)) as i16;
    let raw_shade = record[0x17];
    let effective = if raw_shade == 0 { 0 } else {
        i32::from(raw_shade).wrapping_add(virtual_height >> 7)
    };
    if effective < 0 { return Err(Family8SceneError::NegativeFadeRow(effective)); }
    // 0x10EA0 uses the ordinary fade path below 30 and switches to the direct
    // blitter at 30. Flags 0x20 do not force direct rendering.
    let direct = record[0x15] & 2 != 0 || effective >= 30;
    let shade = u8::try_from(effective).unwrap_or(u8::MAX);
    // The wrapper's shadow call uses the ground origin, while its foreground
    // call applies the virtual-height displacement. Stage both passes so an
    // invalid foreground input cannot leave a shadow behind.
    let mut staged = destination.to_vec();
    let mut shadow_pixels = 0usize;
    let mut shadow_clipped = 0usize;
    for component in std::iter::once(cached) {
        let shadow_sprite = flip_sprite_horizontally_if_needed(&component.sprite, component.component_flags);
        let shadow = draw_normal_component_shadow(
            &shadow_sprite, gla, origin, component.local,
            0, clip, pitch, &mut staged,
        ).map_err(|e| Family8SceneError::Draw(e.to_string()))?;
        shadow_pixels = shadow_pixels.checked_add(shadow.pixels_written)
            .ok_or_else(|| Family8SceneError::Draw("shadow count overflow".into()))?;
        shadow_clipped = shadow_clipped.checked_add(shadow.clipped_opaque_samples)
            .ok_or_else(|| Family8SceneError::Draw("shadow count overflow".into()))?;
    }
    let color = draw_family8_color(cached, origin, color_y, direct, shade, fade, &mut staged, clip, pitch)?;
    destination.copy_from_slice(&staged);
    Ok(receipt(record, animation, frame, global_animation, origin, virtual_height, color_y,
        effective, if direct { PlantColorRoute::Direct } else { PlantColorRoute::Fade { shade } }, shadow_pixels, shadow_clipped,
        color.0, color.1))
}

fn parse_family8_frame(anb: &[u8], animation_index: u8, frame_index: u8) -> Result<Family8Frame, Family8SceneError> {
    let sections = AnbSections::parse(anb).map_err(|e| Family8SceneError::Anb(e.to_string()))?;
    let metadata = sections.ui_metadata().map_err(|e| Family8SceneError::Anb(e.to_string()))?;
    let animation = metadata.animation(usize::from(animation_index)).map_err(|e| Family8SceneError::Anb(e.to_string()))?;
    let span = animation.last_frame_offset.checked_sub(animation.first_frame_offset)
        .filter(|bytes| bytes % 14 == 0).ok_or(Family8SceneError::UnsupportedFrameShape)?;
    let count = usize::try_from(span / 14 + 1).map_err(|_| Family8SceneError::UnsupportedFrameShape)?;
    if usize::from(frame_index) >= count { return Err(Family8SceneError::UnsupportedFrameShape); }
    let offset = animation.first_frame_offset.checked_add(u32::from(frame_index) * 14)
        .ok_or(Family8SceneError::UnsupportedFrameShape)?;
    let frame = metadata.frame_at(offset).map_err(|e| Family8SceneError::Anb(e.to_string()))?;
    if frame.component_count != 1 { return Err(Family8SceneError::UnsupportedFrameShape); }
    let component = metadata.frame_component(frame, 0).map_err(|e| Family8SceneError::Anb(e.to_string()))?;
    if component.flags & !1 != 0 { return Err(Family8SceneError::UnsupportedFrameShape); }
    let table = SpriteTable::from_anb(anb).map_err(|e| Family8SceneError::Anb(e.to_string()))?;
    let sprite = table.decode(component.sprite_offset as usize / 6, anb)
        .map_err(|e| Family8SceneError::Anb(e.to_string()))?;
    let (local_x, local_y) = component.local_position();
    Ok(Family8Frame {
        display: frame.display_dimensions(), sprite, local: [local_x, local_y], component_flags: component.flags,
    })
}

fn flip_sprite_horizontally_if_needed(sprite: &IndexedSprite, flags: u8) -> IndexedSprite {
    if flags & 1 == 0 { return sprite.clone(); }
    let width = usize::from(sprite.width);
    let height = usize::from(sprite.height);
    let mut flipped = sprite.clone();
    for y in 0..height {
        for x in 0..width {
            let src = y * width + (width - 1 - x);
            let dst = y * width + x;
            flipped.indices[dst] = sprite.indices[src];
            flipped.opaque[dst] = sprite.opaque[src];
        }
    }
    flipped
}

fn draw_family8_color(
    frame: &Family8Frame,
    origin: [i16; 2],
    color_y: i16,
    direct: bool,
    shade: u8,
    fade: &TerrainFadeTable<'_>,
    destination: &mut [u8],
    clip: [usize; 2],
    pitch: usize,
) -> Result<(usize, usize), Family8SceneError> {
    let [width, height] = clip;
    if !(1..=640).contains(&width) || !(1..=480).contains(&height) || pitch < width {
        return Err(Family8SceneError::Draw("invalid bounded framebuffer extent".into()));
    }
    let required = (height - 1).checked_mul(pitch).and_then(|n| n.checked_add(width))
        .ok_or_else(|| Family8SceneError::Draw("framebuffer extent overflow".into()))?;
    if destination.len() < required { return Err(Family8SceneError::Draw("short framebuffer".into())); }
    let sprite = &frame.sprite;
    let count = usize::from(sprite.width).checked_mul(usize::from(sprite.height))
        .ok_or_else(|| Family8SceneError::Draw("sprite extent overflow".into()))?;
    if sprite.indices.len() != count || sprite.opaque.len() != count {
        return Err(Family8SceneError::Draw("invalid cached sprite".into()));
    }
    if !direct && shade >= 64 {
        return Err(Family8SceneError::Draw("fade row outside table".into()));
    }
    let left = origin[0].wrapping_add(frame.local[0]) as i32;
    let top = color_y.wrapping_add(frame.local[1]) as i32;
    let w = usize::from(sprite.width);
    let mut written = 0usize;
    let mut clipped = 0usize;
    for y in 0..usize::from(sprite.height) {
        for x in 0..w {
            let source_x = if frame.component_flags & 1 != 0 { w - 1 - x } else { x };
            let at = y * w + source_x;
            if !sprite.opaque[at] { continue; }
            let px = left + x as i32;
            let py = top + y as i32;
            if px < 0 || py < 0 || px >= width as i32 || py >= height as i32 {
                clipped = clipped.checked_add(1).ok_or_else(|| Family8SceneError::Draw("pixel count overflow".into()))?;
                continue;
            }
            let value = sprite.indices[at];
            destination[py as usize * pitch + px as usize] = if direct {
                value
            } else {
                fade.map_palette_index(shade, value)
                    .map_err(|e| Family8SceneError::Draw(e.to_string()))?
            };
            written = written.checked_add(1).ok_or_else(|| Family8SceneError::Draw("pixel count overflow".into()))?;
        }
    }
    Ok((written, clipped))
}

/// Hash admission is performed by the caller's fixed runtime catalog. These
/// structure checks make the helper independently fail closed when called
/// with an arbitrary byte slice.
fn validate_resource(anb: &[u8]) -> Result<(), Family8SceneError> {
    if anb.len() != ETHEREAL_DECODED_BYTES { return Err(Family8SceneError::ResourceLength(anb.len())); }
    let sections = AnbSections::parse(anb).map_err(|e| Family8SceneError::Anb(e.to_string()))?;
    if sections.header_word() != 9 || sections.offsets() != [90_475, 90_577, 90_767, 91_033] {
        return Err(Family8SceneError::ResourceIdentityMismatch);
    }
    let metadata = sections.ui_metadata().map_err(|e| Family8SceneError::Anb(e.to_string()))?;
    let animation = metadata.animation(8).map_err(|e| Family8SceneError::Anb(e.to_string()))?;
    if animation.first_frame_offset != 112 || animation.last_frame_offset != 252 || animation.trailing_word != 267 {
        return Err(Family8SceneError::ResourceIdentityMismatch);
    }
    Ok(())
}

fn receipt(
    record: &[u8; 102], animation: u8, frame: u8, global_animation: u8, ground_origin: [i16; 2],
    virtual_height: i32, color_y: i16, effective_shade: i32, route: PlantColorRoute,
    shadow_pixels: usize, shadow_clipped: usize, color_pixels: usize, color_clipped: usize,
) -> Family8DrawReceipt {
    Family8DrawReceipt {
        slot: usize::from(u16::from_le_bytes([record[0x0f], record[0x10]])),
        animation, global_animation, frame, ground_origin, virtual_height,
        color_origin: [ground_origin[0], color_y],
        effective_shade,
        route,
        shadow_pixels,
        shadow_clipped,
        color_pixels,
        color_clipped,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use genewars_render::terrain_modes::MAIN_GLA_TABLE_BYTES;
    use std::{fs, path::PathBuf};

    fn record() -> [u8; 102] {
        let mut r = [0u8; 102];
        r[6] = 8; r[0x0f..0x11].copy_from_slice(&3u16.to_le_bytes());
        r[0x11..0x15].copy_from_slice(&1000i32.to_le_bytes());
        r[0x15..0x17].copy_from_slice(&0x4020u16.to_le_bytes()); r[0x19] = 0; r[0x1b] = 0; r[0x55] = 0; r[0x56] = 4;
        r[0x27..0x2b].copy_from_slice(&1000i32.to_le_bytes());
        r
    }

    fn tables() -> (Vec<u8>, Vec<u8>) {
        let mut fade = vec![0u8; 64 * 256];
        for row in 0..64 { for index in 0..256 { fade[row*256+index] = (index as u8).wrapping_add(row as u8); } }
        let mut gla = vec![0u8; MAIN_GLA_TABLE_BYTES];
        for c in 0..256 { for d in 0..256 { gla[c*256+d] = (c as u8).wrapping_add(d as u8); } }
        (fade, gla)
    }

    #[test]
    fn strict_state4_admission_rejects_other_states_without_touching_frame() {
        let mut r = record(); let mut frame = [0x5au8; 16];
        let before = frame;
        let (fade_bytes, gla_bytes) = tables();
        let fade = TerrainFadeTable::parse(&fade_bytes).unwrap();
        let gla = MainGlaTable::parse(&gla_bytes).unwrap();
        let resource = real_resource();
        r[0x56] = 7;
        assert!(matches!(draw_family8_selector0_state4(&resource, &r, [0,0], &fade, &gla, &mut frame, [4,4], 4), Err(Family8SceneError::UnsupportedState(7))));
        r[0x56] = 4; r[0x55] = 1;
        assert!(matches!(draw_family8_selector0_state4(&resource, &r, [0,0], &fade, &gla, &mut frame, [4,4], 4), Err(Family8SceneError::UnsupportedSelector(1))));
        r[0x55] = 0; r[0x15..0x17].copy_from_slice(&0x4030u16.to_le_bytes());
        assert!(matches!(draw_family8_selector0_state4(&resource, &r, [0,0], &fade, &gla, &mut frame, [4,4], 4), Err(Family8SceneError::UnsupportedEntityFlags(0x4030))));
        assert_eq!(frame, before);
    }

    fn real_resource() -> EtherealAnimationDatabase {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let bytes = fs::read(root.join("private/integration/rnc-decoded/7660297d85773f47cacc07dcfdf96e8daa118dc52c8a38afc054c1ef9dfa20bc.bin")).unwrap();
        EtherealAnimationDatabase::from_bytes(&bytes).unwrap()
    }

    fn real_tables() -> (Vec<u8>, Vec<u8>) {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        (fs::read(root.join("private/integration/game-runtime/terrain/FADE.DAT")).unwrap(),
            fs::read(root.join("private/integration/game-runtime/terrain/MAIN.GLA")).unwrap())
    }

    #[test]
    fn ethereal_selector_zero_binds_animation_eight_and_all_eleven_frames() {
        let resource = real_resource();
        assert_eq!(resource.frame_count(), 11);
        let (fade_bytes, gla_bytes) = real_tables();
        let fade = TerrainFadeTable::parse(&fade_bytes).unwrap();
        let gla = MainGlaTable::parse(&gla_bytes).unwrap();
        for frame in 0..11 {
            let mut r = record(); r[0x1b] = frame;
            let mut image = vec![0x33; 640 * 480];
            let result = draw_family8_selector0_state4(
                &resource, &r, [320, 380], &fade, &gla, &mut image, [640, 480], 640,
            ).unwrap();
            assert_eq!(result.animation, 0);
            assert_eq!(result.frame, frame);
            assert_eq!(result.virtual_height, 0);
            assert_eq!(result.effective_shade, 0);
            assert_eq!(result.route, PlantColorRoute::Fade { shade: 0 });
            assert!(result.color_pixels > 0);
            assert!(result.shadow_pixels > 0);
        }
    }

    #[test]
    fn selector_zero_parent_states_eight_and_nine_share_only_the_proven_common_draw_path() {
        let resource = real_resource();
        let (fade_bytes, gla_bytes) = real_tables();
        let fade = TerrainFadeTable::parse(&fade_bytes).unwrap();
        let gla = MainGlaTable::parse(&gla_bytes).unwrap();
        let mut state8 = record();
        state8[0x15..0x17].copy_from_slice(&0x0020u16.to_le_bytes());
        state8[0x56] = 8;
        state8[0x1b] = 2;
        state8[0x17] = 49;
        state8[0x11..0x15].copy_from_slice(&6272i32.to_le_bytes());
        state8[0x27..0x2b].copy_from_slice(&22302i32.to_le_bytes());
        let mut state9 = state8;
        state9[0x56] = 9;
        let mut pixels8 = vec![0x35u8; 479 * 648 + 640];
        let mut pixels9 = pixels8.clone();
        let r8 = draw_family8_selector0_state8(&resource, &state8, [280, 170], &fade, &gla,
            &mut pixels8, [640, 480], 648).unwrap();
        let r9 = draw_family8_selector0_state9(&resource, &state9, [280, 170], &fade, &gla,
            &mut pixels9, [640, 480], 648).unwrap();
        assert_eq!(pixels8, pixels9);
        assert_eq!((r8.global_animation, r8.animation, r8.frame), (8, 0, 2));
        assert_eq!((r9.global_animation, r9.animation, r9.frame), (8, 0, 2));
        assert_eq!((r8.virtual_height, r8.effective_shade, r8.color_origin[1]), (16030, 174, -80));
        assert_eq!((r9.virtual_height, r9.effective_shade, r9.color_origin[1]), (16030, 174, -80));
        assert_eq!(r8.color_pixels, 0);
        assert_eq!(r9.color_pixels, 0);
        assert!(r8.shadow_pixels > 0 && r9.shadow_pixels > 0);

        for (state, flags, animation, frame) in [
            (8u8, 0x4020u16, 0u8, 2u8),
            (9u8, 0x0020u16, 1u8, 2u8),
            (8u8, 0x0020u16, 0u8, 11u8),
        ] {
            let mut invalid = if state == 8 { state8 } else { state9 };
            invalid[0x15..0x17].copy_from_slice(&flags.to_le_bytes());
            invalid[0x19] = animation;
            invalid[0x1b] = frame;
            let before = pixels8.clone();
            let result = if state == 8 {
                draw_family8_selector0_state8(&resource, &invalid, [280, 170], &fade, &gla,
                    &mut pixels8, [640, 480], 648)
            } else {
                draw_family8_selector0_state9(&resource, &invalid, [280, 170], &fade, &gla,
                    &mut pixels8, [640, 480], 648)
            };
            assert!(result.is_err());
            assert_eq!(pixels8, before);
        }
    }

    #[test]
    fn ethereal_selector_one_state_zero_binds_global_animation_zero() {
        let resource = real_resource();
        let (fade_bytes, gla_bytes) = real_tables();
        let fade = TerrainFadeTable::parse(&fade_bytes).unwrap();
        let gla = MainGlaTable::parse(&gla_bytes).unwrap();
        let mut r = record();
        r[0x55] = 1;
        r[0x56] = 0;
        let mut image = vec![0x33; (79 * 104) + 100];
        let result = draw_family8_selector1_state0(
            &resource, &r, [35, 35], &fade, &gla, &mut image, [100, 80], 104,
        ).unwrap();
        assert_eq!((result.animation, result.global_animation, result.frame), (0, 0, 0));
        assert_eq!(result.route, PlantColorRoute::Fade { shade: 0 });
        assert!(result.color_pixels > 0);
        assert!(result.shadow_pixels > 0);

        r[0x1b] = 1;
        let before = image.clone();
        assert!(matches!(draw_family8_selector1_state0(
            &resource, &r, [35, 35], &fade, &gla, &mut image, [100, 80], 104,
        ), Err(Family8SceneError::UnsupportedAnimation { animation: 0, frame: 1 })));
        assert_eq!(image, before);
    }

    #[test]
    fn ethereal_selector_one_direction_four_uses_proved_horizontal_component_flip() {
        let resource = real_resource();
        let (fade_bytes, gla_bytes) = real_tables();
        let fade = TerrainFadeTable::parse(&fade_bytes).unwrap();
        let gla = MainGlaTable::parse(&gla_bytes).unwrap();
        let mut r = record();
        r[0x55] = 1;
        r[0x56] = 0;
        r[0x19] = 4;
        let mut image = vec![0x33; 640 * 480];
        let result = draw_family8_selector1_state0(
            &resource, &r, [180, 160], &fade, &gla, &mut image, [640, 480], 640,
        ).unwrap();
        assert_eq!((result.animation, result.global_animation, result.frame), (4, 4, 0));
        assert_eq!(result.route, PlantColorRoute::Fade { shade: 0 });
        assert!(result.color_pixels > 0 && result.shadow_pixels > 0);

        let before = image.clone();
        r[0x19] = 8;
        assert!(matches!(draw_family8_selector1_state0(
            &resource, &r, [180, 160], &fade, &gla, &mut image, [640, 480], 640,
        ), Err(Family8SceneError::UnsupportedAnimation { animation: 8, frame: 0 })));
        assert_eq!(image, before);
    }

    #[test]
    fn ethereal_selector_one_state_one_uses_common_direction_frame_path() {
        let resource = real_resource();
        let (fade_bytes, gla_bytes) = real_tables();
        let fade = TerrainFadeTable::parse(&fade_bytes).unwrap();
        let gla = MainGlaTable::parse(&gla_bytes).unwrap();
        let mut state0 = record();
        state0[0x55] = 1;
        state0[0x56] = 0;
        state0[0x19] = 4;
        let mut state1 = state0;
        state1[0x56] = 1;
        state1[0x15..0x17].copy_from_slice(&0x0020u16.to_le_bytes());
        let mut state0_pixels = vec![0x33; 640 * 480];
        let mut state1_pixels = vec![0x33; 640 * 480];
        let first = draw_family8_selector1_state0(&resource, &state0, [180, 160], &fade, &gla,
            &mut state0_pixels, [640, 480], 640).unwrap();
        let second = draw_family8_selector1_state1(&resource, &state1, [180, 160], &fade, &gla,
            &mut state1_pixels, [640, 480], 640).unwrap();
        assert_eq!((second.animation, second.global_animation, second.frame), (4, 4, 0));
        assert_eq!(second.route, first.route);
        assert_eq!(state1_pixels, state0_pixels);

        state1[0x15..0x17].copy_from_slice(&0x4020u16.to_le_bytes());
        let before = state1_pixels.clone();
        assert!(matches!(draw_family8_selector1_state1(&resource, &state1, [180, 160], &fade, &gla,
            &mut state1_pixels, [640, 480], 640), Err(Family8SceneError::UnsupportedEntityFlags(0x4020))));
        assert_eq!(state1_pixels, before);
    }

    #[test]
    fn ethereal_selector_one_state_three_uses_state_independent_common_draw_path() {
        let resource = real_resource();
        let (fade_bytes, gla_bytes) = real_tables();
        let fade = TerrainFadeTable::parse(&fade_bytes).unwrap();
        let gla = MainGlaTable::parse(&gla_bytes).unwrap();
        let mut state1 = record();
        state1[0x55] = 1;
        state1[0x56] = 1;
        state1[0x15..0x17].copy_from_slice(&0x0020u16.to_le_bytes());
        state1[0x19] = 7;
        state1[0x11..0x15].copy_from_slice(&4414i32.to_le_bytes());
        state1[0x27..0x2b].copy_from_slice(&8308i32.to_le_bytes());
        let mut state3 = state1;
        state3[0x56] = 3;
        let mut state1_pixels = vec![0x35; (479 * 648) + 640];
        let mut state3_pixels = state1_pixels.clone();
        let first = draw_family8_selector1_state1(&resource, &state1, [280, 170], &fade, &gla,
            &mut state1_pixels, [640, 480], 648).unwrap();
        let second = draw_family8_selector1_state3(&resource, &state3, [280, 170], &fade, &gla,
            &mut state3_pixels, [640, 480], 648).unwrap();
        assert_eq!(second, first);
        assert_eq!(state3_pixels, state1_pixels);

        state3[0x1b] = 1;
        let before = state3_pixels.clone();
        assert!(matches!(draw_family8_selector1_state3(&resource, &state3, [280, 170], &fade, &gla,
            &mut state3_pixels, [640, 480], 648),
            Err(Family8SceneError::UnsupportedAnimation { animation: 7, frame: 1 })));
        assert_eq!(state3_pixels, before);
    }

    #[test]
    fn selector_one_state_three_accepts_only_the_two_proved_flag_words_across_directions() {
        let resource = real_resource();
        let (fade_bytes, gla_bytes) = real_tables();
        let fade = TerrainFadeTable::parse(&fade_bytes).unwrap();
        let gla = MainGlaTable::parse(&gla_bytes).unwrap();
        for direction in 0..8u8 {
            let mut low_flags = record();
            low_flags[0x55] = 1;
            low_flags[0x56] = 3;
            low_flags[0x19] = direction;
            low_flags[0x15..0x17].copy_from_slice(&0x0020u16.to_le_bytes());
            let mut high_flags = low_flags;
            high_flags[0x15..0x17].copy_from_slice(&0x4020u16.to_le_bytes());
            let mut expected = vec![0x47; 640 * 480];
            let mut actual = expected.clone();
            draw_family8_selector1_state3(&resource, &low_flags, [312, 204], &fade, &gla,
                &mut expected, [640, 480], 640).unwrap();
            draw_family8_selector1_state3(&resource, &high_flags, [312, 204], &fade, &gla,
                &mut actual, [640, 480], 640).unwrap();
            assert_eq!(actual, expected, "direction {direction}");
            high_flags[0x15..0x17].copy_from_slice(&0x6020u16.to_le_bytes());
            let before = actual.clone();
            assert!(matches!(draw_family8_selector1_state3(&resource, &high_flags, [312, 204], &fade, &gla,
                &mut actual, [640, 480], 640), Err(Family8SceneError::UnsupportedEntityFlags(0x6020))));
            assert_eq!(actual, before);
        }
    }

    #[test]
    fn ethereal_selector_zero_state_five_accepts_arrival_flags_and_frame_domain() {
        let resource = real_resource();
        let (fade_bytes, gla_bytes) = real_tables();
        let fade = TerrainFadeTable::parse(&fade_bytes).unwrap();
        let gla = MainGlaTable::parse(&gla_bytes).unwrap();
        let mut r = record();
        r[0x15..0x17].copy_from_slice(&0x0020u16.to_le_bytes());
        r[0x56] = 5;
        r[0x1b] = 6;
        let mut image = vec![0x33; 640 * 480];
        let result = draw_family8_selector0_state5(
            &resource, &r, [320, 380], &fade, &gla, &mut image, [640, 480], 640,
        ).unwrap();
        assert_eq!((result.animation, result.global_animation, result.frame), (0, 8, 6));
        assert_eq!(result.route, PlantColorRoute::Fade { shade: 0 });
        assert!(result.color_pixels > 0 && result.shadow_pixels > 0);

        let before = image.clone();
        r[0x15..0x17].copy_from_slice(&0x4020u16.to_le_bytes());
        assert!(matches!(draw_family8_selector0_state5(
            &resource, &r, [320, 380], &fade, &gla, &mut image, [640, 480], 640,
        ), Err(Family8SceneError::UnsupportedEntityFlags(0x4020))));
        assert_eq!(image, before);
    }

    #[test]
    fn state4_uses_factory_height_offset_for_y_and_direct_fade_dispatch() {
        let resource = real_resource();
        let (fade_bytes, gla_bytes) = real_tables();
        let fade = TerrainFadeTable::parse(&fade_bytes).unwrap();
        let gla = MainGlaTable::parse(&gla_bytes).unwrap();
        let mut r = record();
        r[0x11..0x15].copy_from_slice(&2000i32.to_le_bytes());
        r[0x27..0x2b].copy_from_slice(&18000i32.to_le_bytes());
        let mut image = vec![0x33; 640 * 480];
        let result = draw_family8_selector0_state4(
            &resource, &r, [320, 380], &fade, &gla, &mut image, [640, 480], 640,
        ).unwrap();
        assert_eq!(result.virtual_height, 16000);
        assert_eq!(result.color_origin, [320, 130]);
        assert_eq!(result.route, PlantColorRoute::Fade { shade: 0 });

        r[0x17] = 200;
        image.fill(0x33);
        let result = draw_family8_selector0_state4(
            &resource, &r, [320, 380], &fade, &gla, &mut image, [640, 480], 640,
        ).unwrap();
        assert_eq!(result.effective_shade, 325);
        assert_eq!(result.route, PlantColorRoute::Direct);
    }

    #[test]
    fn out_of_range_frame_and_changed_resource_reject_before_writing() {
        let resource = real_resource();
        let (fade_bytes, gla_bytes) = real_tables();
        let fade = TerrainFadeTable::parse(&fade_bytes).unwrap();
        let gla = MainGlaTable::parse(&gla_bytes).unwrap();
        let mut r = record(); r[0x1b] = 11;
        let mut image = vec![0x77; 640 * 480]; let before = image.clone();
        assert!(matches!(draw_family8_selector0_state4(
            &resource, &r, [320,380], &fade, &gla, &mut image, [640,480], 640,
        ), Err(Family8SceneError::UnsupportedAnimation { frame: 11, .. })));
        assert_eq!(image, before);
        let mut changed = fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../private/integration/rnc-decoded/7660297d85773f47cacc07dcfdf96e8daa118dc52c8a38afc054c1ef9dfa20bc.bin")).unwrap();
        changed[0] ^= 1;
        assert!(matches!(EtherealAnimationDatabase::from_bytes(&changed), Err(Family8SceneError::ResourceIdentityMismatch)));
    }
}
