//! A caller-placed BTURR main-image adapter with explicit layer omissions.
//!
//! This module consumes an already hash-validated selector-1 BTURR database.
//! It does not stage resources, run a building callback, project/sort records,
//! or infer any live-world state. Only the tested common main-image pass is
//! drawn; callers receive omission flags so the result cannot be mistaken for
//! a complete building scene.

use genewars_runtime::building_resources::{
    BuildingCommonDrawContext, BuildingCommonDrawReceipt, BuildingResourceDatabase,
    BuildingResourceError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildingSceneOmissions {
    /// No building shadow pass is composed by this main-image adapter.
    pub shadow_pass_included: bool,
    /// The source's later owner/selection overlay code is not composed.
    pub post_image_overlays_included: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildingSceneReceipt {
    pub main_image: BuildingCommonDrawReceipt,
    pub omissions: BuildingSceneOmissions,
}

/// Draw the bounded BTURR selector-1 incomplete common image at an explicit
/// origin. `record` and `resource` are validated by the runtime resource
/// adapter. Optional tables are passed through unchanged to the proven
/// indexed component compositor. Destination width, height, and pitch are
/// explicit and row padding remains untouched.
pub fn draw_bturr_incomplete_common(
    resource: &BuildingResourceDatabase,
    record: &[u8; 67],
    origin: [i16; 2],
    global_flags: u8,
    fade_row: Option<&[u8]>,
    blend_table: Option<&[u8]>,
    destination: &mut [u8],
    width: usize,
    height: usize,
    pitch: usize,
) -> Result<BuildingSceneReceipt, BuildingResourceError> {
    let main_image = resource.draw_incomplete_selector1_common(
        record,
        BuildingCommonDrawContext { origin, global_flags },
        fade_row,
        blend_table,
        destination,
        width,
        height,
        pitch,
    )?;
    Ok(BuildingSceneReceipt {
        main_image,
        omissions: BuildingSceneOmissions {
            shadow_pass_included: false,
            post_image_overlays_included: false,
        },
    })
}

/// Draw the proved completed BTURR selector-1 main image. The completed-state
/// status/owner decoration branch remains omitted, as does the shadow pass.
pub fn draw_bturr_completed_common(
    resource: &BuildingResourceDatabase,
    record: &[u8; 67],
    origin: [i16; 2],
    global_flags: u8,
    fade_row: Option<&[u8]>,
    blend_table: Option<&[u8]>,
    destination: &mut [u8],
    width: usize,
    height: usize,
    pitch: usize,
) -> Result<BuildingSceneReceipt, BuildingResourceError> {
    let main_image = resource.draw_completed_selector1_common(
        record,
        BuildingCommonDrawContext { origin, global_flags },
        fade_row,
        blend_table,
        destination,
        width,
        height,
        pitch,
    )?;
    Ok(BuildingSceneReceipt {
        main_image,
        omissions: BuildingSceneOmissions {
            shadow_pass_included: false,
            post_image_overlays_included: false,
        },
    })
}

/// Type-level marker for callers that persist a renderer policy alongside a
/// frame. The only supported policy is a common main-image diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingScenePolicy {
    BturrSelector1CommonMainImageOnly,
}

pub const BUILDING_SCENE_POLICY: BuildingScenePolicy = BuildingScenePolicy::BturrSelector1CommonMainImageOnly;

#[cfg(test)]
mod tests {
    use super::*;
    use genewars_runtime::building_resources::BTURR_BYTES;
    use std::{fs, path::Path};

    fn resource() -> BuildingResourceDatabase {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../private/reference-run/run-006/drive-c/GAME/DATA/BUILDING/BTURR.ANB");
        let bytes = fs::read(path).expect("private BTURR resource is staged");
        assert_eq!(bytes.len(), BTURR_BYTES);
        BuildingResourceDatabase::load_selector1_bturr(bytes).unwrap()
    }

    fn record(frame: u8) -> [u8; 67] {
        let mut record = [0u8; 67];
        record[6] = 4;
        record[0x0f..0x11].copy_from_slice(&17i16.to_le_bytes());
        record[0x18] = 2;
        record[0x1b] = frame;
        record[0x2d] = 1;
        record[0x17] = 30;
        record
    }

    #[test]
    fn wrapper_reports_main_image_and_explicitly_omits_later_layers() {
        let resource = resource();
        assert_eq!(BUILDING_SCENE_POLICY, BuildingScenePolicy::BturrSelector1CommonMainImageOnly);
        for frame_index in 0..3 {
            let mut pixels = vec![0x55u8; (180 - 1) * 192 + 180];
            let receipt = draw_bturr_incomplete_common(
                &resource, &record(frame_index), [2, 3], 0, None, None,
                &mut pixels, 180, 180, 192,
            ).unwrap();
            assert_eq!(receipt.main_image.frame_index, frame_index);
            assert!(receipt.main_image.color.pixels_written > 0);
            assert!(!receipt.omissions.shadow_pass_included);
            assert!(!receipt.omissions.post_image_overlays_included);
            for row in 0..179 {
                assert!(pixels[row * 192 + 180..(row + 1) * 192].iter().all(|pixel| *pixel == 0x55));
            }
        }
    }

    #[test]
    fn completed_wrapper_draws_animation_one_and_keeps_omissions_explicit() {
        let resource = resource();
        for frame in 0..8u8 {
            let mut completed = record(frame);
            completed[0x19] = 1;
            completed[0x31..0x33].copy_from_slice(&1u16.to_le_bytes());
            completed[0x17] = 30;
            let mut pixels = vec![0x5a; (480 - 1) * 648 + 640];
            let receipt = draw_bturr_completed_common(
                &resource, &completed, [320, 240], 0, None, None,
                &mut pixels, 640, 480, 648,
            ).unwrap();
            assert_eq!(receipt.main_image.animation_index, 1);
            assert_eq!(receipt.main_image.frame_index, frame);
            assert!(receipt.main_image.color.pixels_written > 0);
            assert!(!receipt.omissions.shadow_pass_included);
            assert!(!receipt.omissions.post_image_overlays_included);
            for row in 0..479 {
                assert!(pixels[row * 648 + 640..(row + 1) * 648].iter().all(|pixel| *pixel == 0x5a));
            }
        }
    }

    #[test]
    fn rejected_overlay_or_frame_inputs_do_not_partially_draw() {
        let resource = resource();
        let mut pixels = vec![0x5a; 64 * 64];
        let before = pixels.clone();
        let mut wrong_binding = record(0);
        wrong_binding[0x2d] = 0;
        assert!(draw_bturr_incomplete_common(
            &resource, &wrong_binding, [0, 0], 0, None, None,
            &mut pixels, 64, 64, 64,
        ).is_err());
        assert_eq!(pixels, before);

        let mut destruction = record(0);
        destruction[0x31..0x33].copy_from_slice(&0x0800u16.to_le_bytes());
        assert!(draw_bturr_incomplete_common(
            &resource, &destruction, [0, 0], 0, None, None,
            &mut pixels, 64, 64, 64,
        ).is_err());
        assert_eq!(pixels, before);
    }
}
