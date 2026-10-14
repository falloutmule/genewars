//! Hash-bound BTURR animation data used by the incomplete selector-1 callback.
//!
//! The loader admits only the CD resource reached through building selector 1
//! at descriptor 0x103a9c / destination 0x116828. It exposes the admitted
//! incomplete animation 0 and completed animation 1 frame ranges. Rendering
//! policy and callback state remain separate.

use genewars_assets::{AnbAnimationRecord, AnbComponentRecord, AnbFrameRecord, AnbSections, Error as AssetError, IndexedSprite, SpriteTable};
use std::fmt;

use crate::{
    actor_render::{draw_actor_components, ActorDecodedComponent, ActorDrawContext, ActorDrawReceipt,
        ActorRenderError, ActorRenderRecord},
    actor_initialization::ActorResourceTableKey,
    identity::sha256_hex,
};

pub const BTURR_SELECTOR: u8 = 1;
pub const BTURR_BYTES: usize = 68_030;
pub const BTURR_SHA256: &str = "c26df60af3a3166aec54112f197a4334966eb214eeaa48e0d9284e0510b8cd20";
pub const BTURR_ANIMATION0_FRAME_COUNT: usize = 3;
pub const BTURR_ANIMATION1_FRAME_COUNT: usize = 8;
const BTURR_SECTION_OFFSETS: [u32; 4] = [66_562, 67_000, 67_560, 67_980];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildingResourceError {
    WrongSelector(u8),
    WrongLength { expected: usize, actual: usize },
    WrongHash { expected: &'static str, actual: String },
    Anb(AssetError),
    SectionLayout,
    AnimationLayout,
    FrameLayout { frame: usize },
    ComponentLayout { frame: usize, component: usize },
    FrameOutOfBounds(usize),
    UnsupportedBuildingRecord,
    UnsupportedDrawFlags(u8),
    Render(ActorRenderError),
}

impl fmt::Display for BuildingResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for BuildingResourceError {}

impl From<ActorRenderError> for BuildingResourceError {
    fn from(value: ActorRenderError) -> Self { Self::Render(value) }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildingAnimation0Component {
    pub record: AnbComponentRecord,
    pub sprite_record: usize,
    pub sprite: IndexedSprite,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildingAnimation0Frame {
    pub index: u8,
    pub hold: u8,
    pub metadata: AnbFrameRecord,
    pub display_dimensions: (i16, i16),
    pub components: Vec<BuildingAnimation0Component>,
}

/// Exact selector-1 animation-0 holds, admitted from the source ANB bytes.
/// This is the callback input, not a generalized ANB timing interpretation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildingAnimation0Holds([u8; BTURR_ANIMATION0_FRAME_COUNT]);

impl BuildingAnimation0Holds {
    pub fn as_array(&self) -> &[u8; BTURR_ANIMATION0_FRAME_COUNT] { &self.0 }
    pub fn as_slice(&self) -> &[u8] { &self.0 }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildingAnimation0 {
    frames: [BuildingAnimation0Frame; BTURR_ANIMATION0_FRAME_COUNT],
    holds: BuildingAnimation0Holds,
    animation_record: AnbAnimationRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildingAnimation1 {
    frames: [BuildingAnimation0Frame; BTURR_ANIMATION1_FRAME_COUNT],
    animation_record: AnbAnimationRecord,
}

impl BuildingAnimation1 {
    pub fn frame_count(&self) -> usize { self.frames.len() }
    pub fn animation_record(&self) -> AnbAnimationRecord { self.animation_record }
    pub fn frame(&self, index: usize) -> Result<&BuildingAnimation0Frame, BuildingResourceError> {
        self.frames.get(index).ok_or(BuildingResourceError::FrameOutOfBounds(index))
    }
    pub fn frames(&self) -> &[BuildingAnimation0Frame; BTURR_ANIMATION1_FRAME_COUNT] { &self.frames }
}

/// The caller-supplied screen origin and the already-resolved low global draw
/// flags from the 43A40 caller. Only the normal unscaled 0/0x04 common path is
/// admitted here; selection, destruction transforms and blink are excluded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildingCommonDrawContext {
    pub origin: [i16; 2],
    pub global_flags: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildingCommonDrawReceipt {
    pub animation_index: u8,
    pub frame_index: u8,
    /// X after the wrapper's proved +0x30 parity adjustment.
    pub draw_origin: [i16; 2],
    pub component_count: usize,
    pub color: ActorDrawReceipt,
    /// 43A40 has additional post-image UI/owner overlays outside this helper.
    pub post_image_overlays_rendered: bool,
}

impl BuildingAnimation0 {
    pub fn frame_count(&self) -> usize { self.frames.len() }
    pub fn holds(&self) -> BuildingAnimation0Holds { self.holds }
    pub fn animation_record(&self) -> AnbAnimationRecord { self.animation_record }
    pub fn frame(&self, index: usize) -> Result<&BuildingAnimation0Frame, BuildingResourceError> {
        self.frames.get(index).ok_or(BuildingResourceError::FrameOutOfBounds(index))
    }
    pub fn frames(&self) -> &[BuildingAnimation0Frame; BTURR_ANIMATION0_FRAME_COUNT] { &self.frames }
}

/// Owned, exact-hash source resource for the selector-1 BTURR binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildingResourceDatabase {
    bytes: Vec<u8>,
    animation0: BuildingAnimation0,
    animation1: BuildingAnimation1,
}

impl BuildingResourceDatabase {
    pub fn load_selector1_bturr(decoded_anb: Vec<u8>) -> Result<Self, BuildingResourceError> {
        Self::load_for_selector(BTURR_SELECTOR, decoded_anb)
    }

    pub fn load_for_selector(selector: u8, decoded_anb: Vec<u8>) -> Result<Self, BuildingResourceError> {
        if selector != BTURR_SELECTOR { return Err(BuildingResourceError::WrongSelector(selector)); }
        if decoded_anb.len() != BTURR_BYTES {
            return Err(BuildingResourceError::WrongLength { expected: BTURR_BYTES, actual: decoded_anb.len() });
        }
        let actual = sha256_hex(&decoded_anb);
        if actual != BTURR_SHA256 {
            return Err(BuildingResourceError::WrongHash { expected: BTURR_SHA256, actual });
        }

        let sections = AnbSections::parse(&decoded_anb).map_err(BuildingResourceError::Anb)?;
        if sections.header_word() != 5 || sections.offsets() != BTURR_SECTION_OFFSETS {
            return Err(BuildingResourceError::SectionLayout);
        }
        let metadata = sections.ui_metadata().map_err(BuildingResourceError::Anb)?;
        if metadata.animation_count() != 5 || metadata.frame_count() != 30 || metadata.component_count() != 56 {
            return Err(BuildingResourceError::SectionLayout);
        }
        let animation = metadata.animation(0).map_err(BuildingResourceError::Anb)?;
        if animation.first_frame_offset != 0 || animation.last_frame_offset != 28 || animation.trailing_word != 3 {
            return Err(BuildingResourceError::AnimationLayout);
        }
        let table = SpriteTable::from_anb(&decoded_anb).map_err(BuildingResourceError::Anb)?;
        let mut frames = Vec::with_capacity(BTURR_ANIMATION0_FRAME_COUNT);
        for index in 0..BTURR_ANIMATION0_FRAME_COUNT {
            let offset = (index * 14) as u32;
            let frame = metadata.frame_at(offset).map_err(BuildingResourceError::Anb)?;
            let expected_components = if index == 2 { 2 } else { 1 };
            let expected_dimensions = [(66, 212), (74, 228), (204, 320)][index];
            if usize::from(frame.component_count) != expected_components
                || (frame.width_raw, frame.height_raw) != expected_dimensions
                || frame.byte_10 != 0
            {
                return Err(BuildingResourceError::FrameLayout { frame: index });
            }
            let mut components = Vec::with_capacity(expected_components);
            for component_index in 0..expected_components {
                let component = metadata.frame_component(frame, component_index).map_err(BuildingResourceError::Anb)?;
                if component.flags != 0 {
                    return Err(BuildingResourceError::ComponentLayout { frame: index, component: component_index });
                }
                let sprite_record = component.sprite_offset as usize / 6;
                let sprite = table.decode(sprite_record, &decoded_anb).map_err(BuildingResourceError::Anb)?;
                components.push(BuildingAnimation0Component { record: component, sprite_record, sprite });
            }
            frames.push(BuildingAnimation0Frame {
                index: index as u8,
                hold: frame.byte_10,
                metadata: frame,
                display_dimensions: frame.display_dimensions(),
                components,
            });
        }
        let frames: [BuildingAnimation0Frame; BTURR_ANIMATION0_FRAME_COUNT] = frames
            .try_into().map_err(|_| BuildingResourceError::AnimationLayout)?;
        let holds = [frames[0].hold, frames[1].hold, frames[2].hold];
        if holds != [0, 0, 0] { return Err(BuildingResourceError::AnimationLayout); }
        let animation1_record = metadata.animation(1).map_err(BuildingResourceError::Anb)?;
        if animation1_record.first_frame_offset != 42
            || animation1_record.last_frame_offset != 140
            || animation1_record.trailing_word != 8
        {
            return Err(BuildingResourceError::AnimationLayout);
        }
        let mut animation1_frames = Vec::with_capacity(BTURR_ANIMATION1_FRAME_COUNT);
        for index in 0..BTURR_ANIMATION1_FRAME_COUNT {
            let offset = animation1_record.first_frame_offset + (index as u32 * 14);
            let frame = metadata.frame_at(offset).map_err(BuildingResourceError::Anb)?;
            if frame.byte_10 != 0 {
                return Err(BuildingResourceError::FrameLayout { frame: index });
            }
            let mut components = Vec::with_capacity(usize::from(frame.component_count));
            for component_index in 0..usize::from(frame.component_count) {
                let component = metadata.frame_component(frame, component_index).map_err(BuildingResourceError::Anb)?;
                if component.flags != 0 {
                    return Err(BuildingResourceError::ComponentLayout { frame: index, component: component_index });
                }
                let sprite_record = component.sprite_offset as usize / 6;
                let sprite = table.decode(sprite_record, &decoded_anb).map_err(BuildingResourceError::Anb)?;
                components.push(BuildingAnimation0Component { record: component, sprite_record, sprite });
            }
            animation1_frames.push(BuildingAnimation0Frame {
                index: index as u8,
                hold: frame.byte_10,
                metadata: frame,
                display_dimensions: frame.display_dimensions(),
                components,
            });
        }
        let animation1_frames: [BuildingAnimation0Frame; BTURR_ANIMATION1_FRAME_COUNT] = animation1_frames
            .try_into().map_err(|_| BuildingResourceError::AnimationLayout)?;
        Ok(Self { bytes: decoded_anb, animation0: BuildingAnimation0 {
            frames, holds: BuildingAnimation0Holds(holds), animation_record: animation,
        }, animation1: BuildingAnimation1 { frames: animation1_frames, animation_record: animation1_record } })
    }

    pub fn selector(&self) -> u8 { BTURR_SELECTOR }
    pub fn bytes(&self) -> &[u8] { &self.bytes }
    pub fn animation0(&self) -> &BuildingAnimation0 { &self.animation0 }
    pub fn animation1(&self) -> &BuildingAnimation1 { &self.animation1 }

    /// Draw the ordinary main-image pass for the admitted incomplete selector-1
    /// building. The caller chooses screen origin and the source global flags;
    /// this does not perform projection, shadow, sorting or post-image overlays.
    pub fn draw_incomplete_selector1_common(
        &self,
        record: &[u8; 67],
        context: BuildingCommonDrawContext,
        fade_row: Option<&[u8]>,
        blend_table: Option<&[u8]>,
        destination: &mut [u8],
        width: usize,
        height: usize,
        pitch: usize,
    ) -> Result<BuildingCommonDrawReceipt, BuildingResourceError> {
        self.draw_selector1_common(record, 0, context, fade_row, blend_table, destination, width, height, pitch)
    }

    /// Draw the proved BTURR selector-1 animation-1 common main-image pass.
    /// The caller selects this by the record's animation byte; the completion
    /// bit controls later owner/status decorations and does not gate this pass.
    pub fn draw_completed_selector1_common(
        &self,
        record: &[u8; 67],
        context: BuildingCommonDrawContext,
        fade_row: Option<&[u8]>,
        blend_table: Option<&[u8]>,
        destination: &mut [u8],
        width: usize,
        height: usize,
        pitch: usize,
    ) -> Result<BuildingCommonDrawReceipt, BuildingResourceError> {
        self.draw_selector1_common(record, 1, context, fade_row, blend_table, destination, width, height, pitch)
    }

    fn draw_selector1_common(
        &self,
        record: &[u8; 67],
        expected_animation: u8,
        context: BuildingCommonDrawContext,
        fade_row: Option<&[u8]>,
        blend_table: Option<&[u8]>,
        destination: &mut [u8],
        width: usize,
        height: usize,
        pitch: usize,
    ) -> Result<BuildingCommonDrawReceipt, BuildingResourceError> {
        let source_flags = u16::from_le_bytes([record[0x31], record[0x32]]);
        let actor_id = i16::from_le_bytes([record[0x0f], record[0x10]]);
        let owner = record[0x18];
        let frame_index = record[0x1b];
        // 43A40's selected/destruction and special blink paths are outside this
        // bounded common pass. Completion and other overlay fields do not gate
        // the source main-image blitter.
        if record[0x06] != 4 || owner >= 4 || record[0x2d] != 1
            || record[0x19] != expected_animation
            || source_flags & 0x0800 != 0 || record[0x15] & 0x10 != 0
            || actor_id < 0 || usize::from(actor_id as u16) >= 256
        {
            return Err(BuildingResourceError::UnsupportedBuildingRecord);
        }
        if context.global_flags & !0x04 != 0 {
            return Err(BuildingResourceError::UnsupportedDrawFlags(context.global_flags));
        }
        let frame = if expected_animation == 0 {
            self.animation0.frame(usize::from(frame_index))?
        } else {
            self.animation1.frame(usize::from(frame_index))?
        };
        draw_common_frame(record, frame, context, fade_row, blend_table, destination, width, height, pitch)
    }
}

/// Shared already-guarded 43A40 main-image operation. Admission belongs to each exact resource.
pub(crate) fn draw_common_frame(record:&[u8;67],frame:&BuildingAnimation0Frame,
    context:BuildingCommonDrawContext,fade_row:Option<&[u8]>,blend_table:Option<&[u8]>,
    destination:&mut [u8],width:usize,height:usize,pitch:usize)->Result<BuildingCommonDrawReceipt,BuildingResourceError>{
    let actor_id=i16::from_le_bytes([record[0x0f],record[0x10]]);
    let owner=record[0x18];let expected_animation=record[0x19];let frame_index=record[0x1b];
        let phase = if record[0x30] > 1 { i16::from(record[0x30] & 1) } else { 0 };
        let draw_origin = [context.origin[0].wrapping_sub(phase), context.origin[1]];
        let components = frame.components.iter().map(|component| component.record).collect::<Vec<_>>();
        let decoded = frame.components.iter().map(|component| ActorDecodedComponent {
            component: component.record,
            sprite: component.sprite.clone(),
        }).collect::<Vec<_>>();
        let key = ActorResourceTableKey { style: 0, category: 0, flat_index: 0 };
        let native_position = [
            i16::from_le_bytes([record[0x07], record[0x08]]),
            i16::from_le_bytes([record[0x09], record[0x0a]]),
        ];
        let draw_record = ActorRenderRecord {
            actor_slot: actor_id as usize,
            actor_id: actor_id as u16,
            owner,
            blueprint: crate::actor_initialization::BlueprintRecordHandle { player_id: owner, blueprint_index: 0 },
            resource_key: key,
            world_position: native_position,
            virtual_height: 0,
            shade: record[0x17],
            actor_flags_15: record[0x15],
            animation_index: expected_animation,
            frame_index,
            draw_flags_55: 0,
            draw_flag_58: 0,
            draw_flag_59: 0,
            draw_flags_70: 0,
            draw_flag_7e: 0,
            draw_flag_82: 0,
            draw_mode_73: 0,
            frame_record: frame.metadata,
            components,
        };
        let color = draw_actor_components(
            &draw_record,
            &decoded,
            ActorDrawContext { origin: [i32::from(draw_origin[0]), i32::from(draw_origin[1]),], virtual_height: 0, global_flags: context.global_flags },
            fade_row,
            blend_table,
            destination,
            width,
            height,
            pitch,
        )?;
        Ok(BuildingCommonDrawReceipt {
            animation_index: expected_animation,
            frame_index,
            draw_origin,
            component_count: decoded.len(),
            color,
            post_image_overlays_rendered: false,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_wrong_selector_and_length_before_parsing() {
        assert_eq!(BuildingResourceDatabase::load_for_selector(0, Vec::new()), Err(BuildingResourceError::WrongSelector(0)));
        assert_eq!(BuildingResourceDatabase::load_selector1_bturr(Vec::new()), Err(BuildingResourceError::WrongLength { expected: BTURR_BYTES, actual: 0 }));
    }

    #[test]
    fn same_length_unpinned_resource_is_rejected_by_hash() {
        assert!(matches!(BuildingResourceDatabase::load_selector1_bturr(vec![0; BTURR_BYTES]), Err(BuildingResourceError::WrongHash { .. })));
    }

    #[test]
    fn cd_bturr_animation_zero_has_exact_three_zero_holds() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../private/reference-run/run-006/drive-c/GAME/DATA/BUILDING/BTURR.ANB");
        let bytes = std::fs::read(path).expect("pinned private BTURR input is staged");
        let database = BuildingResourceDatabase::load_selector1_bturr(bytes).unwrap();
        let animation = database.animation0();
        assert_eq!(animation.frame_count(), 3);
        assert_eq!(animation.animation_record(), AnbAnimationRecord {
            first_frame_offset: 0, last_frame_offset: 28, trailing_word: 3,
        });
        assert_eq!(animation.holds().as_array(), &[0, 0, 0]);
        assert_eq!([animation.frames()[0].hold, animation.frames()[1].hold, animation.frames()[2].hold], [0, 0, 0]);
        assert_eq!([animation.frames()[0].components.len(), animation.frames()[1].components.len(), animation.frames()[2].components.len()], [1, 1, 2]);
    }

    #[test]
    fn cd_bturr_completed_animation_one_has_eight_source_frames_and_draws_atomically() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../private/reference-run/run-006/drive-c/GAME/DATA/BUILDING/BTURR.ANB");
        let database = BuildingResourceDatabase::load_selector1_bturr(std::fs::read(path).unwrap()).unwrap();
        let animation = database.animation1();
        assert_eq!(animation.frame_count(), 8);
        assert_eq!(animation.animation_record(), AnbAnimationRecord {
            first_frame_offset: 42, last_frame_offset: 140, trailing_word: 8,
        });
        assert!(animation.frames().iter().all(|frame| frame.hold == 0));
        assert!(animation.frames().iter().all(|frame| !frame.components.is_empty()));

        for frame_index in 0..8u8 {
            let mut record = [0u8; 67];
            record[6] = 4;
            record[15..17].copy_from_slice(&17i16.to_le_bytes());
            record[18] = 0;
            record[0x19] = 1;
            record[0x1b] = frame_index;
            record[0x2d] = 1;
            record[0x31..0x33].copy_from_slice(&1u16.to_le_bytes());
            record[0x17] = 30;
            let width = 240usize;
            let height = 220usize;
            let pitch = 256usize;
            let mut pixels = vec![0x5a; (height - 1) * pitch + width];
            let receipt = database.draw_completed_selector1_common(
                &record,
                BuildingCommonDrawContext { origin: [120, 110], global_flags: 0 },
                None, None, &mut pixels, width, height, pitch,
            ).unwrap();
            assert_eq!(receipt.frame_index, frame_index);
            assert!(receipt.color.pixels_written > 0);
            assert!(!receipt.post_image_overlays_rendered);
            for row in 0..height - 1 {
                assert!(pixels[row * pitch + width..(row + 1) * pitch].iter().all(|v| *v == 0x5a));
            }
        }

        let mut record = [0u8; 67];
        record[6] = 4;
        record[15..17].copy_from_slice(&17i16.to_le_bytes());
        record[18] = 0;
        record[0x19] = 1;
        record[0x1b] = 8;
        record[0x2d] = 1;
        record[0x31..0x33].copy_from_slice(&1u16.to_le_bytes());
        let mut pixels = [0x77; 64];
        let before = pixels;
        assert_eq!(database.draw_completed_selector1_common(
            &record, BuildingCommonDrawContext { origin: [0, 0], global_flags: 0 },
            None, None, &mut pixels, 8, 8, 8,
        ), Err(BuildingResourceError::FrameOutOfBounds(8)));
        assert_eq!(pixels, before);
    }

    #[test]
    fn bturr_incomplete_common_path_draws_all_three_frames_and_clips_atomically() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../private/reference-run/run-006/drive-c/GAME/DATA/BUILDING/BTURR.ANB");
        let database = BuildingResourceDatabase::load_selector1_bturr(std::fs::read(path).unwrap()).unwrap();
        for frame in 0..3u8 {
            let mut record = [0u8; 67];
            record[6] = 4;
            record[15..17].copy_from_slice(&17i16.to_le_bytes());
            record[18] = 2;
            record[19] = 0;
            record[27] = frame;
            record[45] = 1;
            record[47] = 0;
            record[49..51].copy_from_slice(&0u16.to_le_bytes());
            record[0x17] = 30; // the wrapper's direct-color branch
            let width = 180usize;
            let height = 180usize;
            let pitch = 192usize;
            let len = (height - 1) * pitch + width;
            let mut pixels = vec![0x55; len];
            let receipt = database.draw_incomplete_selector1_common(
                &record,
                BuildingCommonDrawContext { origin: [2, 3], global_flags: 0 },
                None,
                None,
                &mut pixels,
                width,
                height,
                pitch,
            ).unwrap();
            assert_eq!(receipt.frame_index, frame);
            assert_eq!(receipt.component_count, [1, 1, 2][usize::from(frame)]);
            assert!(!receipt.post_image_overlays_rendered);
            assert!(receipt.color.pixels_written > 0);
            for row in 0..height - 1 {
                assert!(pixels[row * pitch + width..(row + 1) * pitch].iter().all(|v| *v == 0x55));
            }
        }

        let mut rejected = [0u8; 67];
        rejected[6] = 4;
        rejected[15..17].copy_from_slice(&17i16.to_le_bytes());
        rejected[18] = 2;
        rejected[19] = 0;
        rejected[27] = 0;
        rejected[45] = 1;
        rejected[49..51].copy_from_slice(&0u16.to_le_bytes());
        rejected[0x15] = 0x10;
        let mut destination = [0x5a; 16];
        let before = destination;
        assert_eq!(database.draw_incomplete_selector1_common(
            &rejected,
            BuildingCommonDrawContext { origin: [0, 0], global_flags: 0 },
            None, None, &mut destination, 4, 4, 4,
        ), Err(BuildingResourceError::UnsupportedBuildingRecord));
        assert_eq!(destination, before);
    }

    #[test]
    fn non_destruction_overlay_fields_do_not_gate_the_main_image_pass() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../private/reference-run/run-006/drive-c/GAME/DATA/BUILDING/BTURR.ANB");
        let database = BuildingResourceDatabase::load_selector1_bturr(std::fs::read(path).unwrap()).unwrap();
        let mut record = [0u8; 67];
        record[6] = 4;
        record[15..17].copy_from_slice(&17i16.to_le_bytes());
        record[18] = 2;
        record[0x19] = 0;
        record[0x1b] = 0;
        record[0x2d] = 1;
        record[0x2e] = 9;
        record[0x2f] = 3;
        record[0x17] = 30;
        // These word bits and paired display fields only control later UI
        // overlays. The common source image has already been rendered before
        // those branches; only bit 0x0800 selects the destruction transform.
        record[0x31..0x33].copy_from_slice(&0x0041u16.to_le_bytes());
        let len = (180 - 1) * 192 + 180;
        let mut pixels = vec![0x55; len];
        let receipt = database.draw_incomplete_selector1_common(
            &record,
            BuildingCommonDrawContext { origin: [2, 3], global_flags: 0 },
            None, None, &mut pixels, 180, 180, 192,
        ).unwrap();
        assert!(receipt.color.pixels_written > 0);
        assert!(!receipt.post_image_overlays_rendered);

        record[0x31..0x33].copy_from_slice(&0x0800u16.to_le_bytes());
        let before = pixels.clone();
        assert_eq!(database.draw_incomplete_selector1_common(
            &record,
            BuildingCommonDrawContext { origin: [2, 3], global_flags: 0 },
            None, None, &mut pixels, 180, 180, 192,
        ), Err(BuildingResourceError::UnsupportedBuildingRecord));
        assert_eq!(pixels, before);
    }

    #[test]
    fn wrapper_phase_adjusts_only_x_with_signed_word_wrap() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../private/reference-run/run-006/drive-c/GAME/DATA/BUILDING/BTURR.ANB");
        let database = BuildingResourceDatabase::load_selector1_bturr(std::fs::read(path).unwrap()).unwrap();
        let mut record = [0u8; 67];
        record[6] = 4;
        record[0x0f..0x11].copy_from_slice(&17i16.to_le_bytes());
        record[0x18] = 2;
        record[0x2d] = 1;
        record[0x30] = 3;
        record[0x31..0x33].copy_from_slice(&0u16.to_le_bytes());
        record[0x17] = 30;
        let mut pixels = vec![0u8; 32 * 32];
        let receipt = database.draw_incomplete_selector1_common(
            &record,
            BuildingCommonDrawContext { origin: [i16::MIN, 4], global_flags: 0 },
            None, None, &mut pixels, 32, 32, 32,
        ).unwrap();
        assert_eq!(receipt.draw_origin, [i16::MAX, 4]);

        record[0x30] = 2;
        let receipt = database.draw_incomplete_selector1_common(
            &record,
            BuildingCommonDrawContext { origin: [6, 4], global_flags: 0 },
            None, None, &mut pixels, 32, 32, 32,
        ).unwrap();
        assert_eq!(receipt.draw_origin, [6, 4]);
    }
}
