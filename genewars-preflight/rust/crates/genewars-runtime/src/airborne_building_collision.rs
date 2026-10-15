//! Literal 6DD50 numeric impulse after the caller proves 74270 collision.
//! Building footprint/frame-height lookup and scheduler admission stay external.
use crate::{actor_activation::TrigTables, physics_tables::PhysicsTables};

pub type MuleRecord = [u8; 124];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionError {
    UnsupportedMule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionPlan {
    pub before: MuleRecord,
    pub after: MuleRecord,
    pub impulse: [i16; 3],
}

fn word(record: &[u8], offset: usize) -> i16 {
    i16::from_le_bytes(record[offset..offset + 2].try_into().unwrap())
}

fn put(record: &mut [u8], offset: usize, value: i16) {
    record[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

/// 6DD50/73A50. Obstacle XY and ground height belong to the actual source
/// query result; passing geometry alone does not establish collision admission.
pub fn plan_collision(
    before: &MuleRecord,
    obstacle_xy: [i16; 2],
    obstacle_ground: i32,
    physics: &PhysicsTables,
    trig: &TrigTables,
) -> Result<CollisionPlan, CollisionError> {
    if before[6] != 1 || before[24] > 4 || before[0x7a] != 4 || word(before, 0x3d) < 0 {
        return Err(CollisionError::UnsupportedMule);
    }
    let doubled = [
        word(before, 7).wrapping_mul(2).wrapping_sub(obstacle_xy[0].wrapping_mul(2)),
        word(before, 9).wrapping_mul(2).wrapping_sub(obstacle_xy[1].wrapping_mul(2)),
    ];
    let yaw = crate::actor_motion::angle(
        i32::from(doubled[0]), i32::from(doubled[1]), physics.angles(),
    );
    let delta = doubled.map(|value| i32::from(value >> 1));
    let distance = genewars_core::cd_candidate::integer_sqrt(
        delta[0].wrapping_mul(delta[0]).wrapping_add(delta[1].wrapping_mul(delta[1])) as u32,
    ) as i32;
    let height = i32::from_le_bytes(before[39..43].try_into().unwrap());
    let pitch = crate::actor_motion::angle(
        distance, obstacle_ground.wrapping_sub(height) >> 3, physics.angles(),
    );
    let yaw_index = (i32::from(yaw) - 512).rem_euclid(2048) as usize;
    let pitch_index = (i32::from(pitch) - 512).rem_euclid(2048) as usize;
    let half = i64::from(word(before, 0x3d) >> 1);
    let horizontal = (i64::from(trig.cosine()[pitch_index]) * half) >> 16;
    let impulse = [
        ((i64::from(trig.cosine()[yaw_index]) * horizontal) >> 16) as i16,
        ((i64::from(trig.sine()[yaw_index]) * horizontal) >> 16) as i16,
        ((-i64::from(trig.sine()[pitch_index]) * half) >> 16) as i16,
    ];
    let mut after = *before;
    for (axis, addition) in impulse.iter().enumerate() {
        put(&mut after, 0x2d + axis * 2, word(before, 0x2d + axis * 2).wrapping_add(*addition));
    }

    // 6DEBA..6DF1A recomputes orientation from the new wrapped velocity.
    let (x, y, z) = (
        i32::from(word(&after, 0x2d)),
        i32::from(word(&after, 0x2f)),
        i32::from(word(&after, 0x31)),
    );
    let xy = genewars_core::cd_candidate::integer_sqrt(
        x.wrapping_mul(x).wrapping_add(y.wrapping_mul(y)) as u32,
    ) as i32;
    if xy != 0 {
        put(&mut after, 0x39, crate::actor_motion::angle(x, y, physics.angles()) as i16);
    }
    let speed = genewars_core::cd_candidate::integer_sqrt(
        xy.wrapping_mul(xy).wrapping_add(z.wrapping_mul(z)) as u32,
    ) as i16;
    put(&mut after, 0x3d, speed);
    put(&mut after, 0x3b, if speed == 0 {
        512
    } else {
        crate::actor_motion::angle(xy, -z, physics.angles()) as i16
    });
    Ok(CollisionPlan { before: *before, after, impulse })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tables() -> (PhysicsTables, TrigTables) {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../private/integration/game-runtime/physics");
        let physics = PhysicsTables::from_mapped_bytes(
            &std::fs::read(path.join("angle.bin")).unwrap(),
            &std::fs::read(path.join("bounce.bin")).unwrap(),
        ).unwrap();
        (physics, TrigTables::generate_pinned().unwrap())
    }

    fn mule() -> MuleRecord {
        let mut record = [0x55; 124];
        record[6] = 1;
        record[24] = 0;
        record[0x7a] = 4;
        put(&mut record, 0x3d, 4800);
        record
    }

    #[test]
    fn collision_changes_only_velocity_orientation_and_keeps_command_speed() {
        let (physics, trig) = tables();
        let before = mule();
        let plan = plan_collision(&before, [32760, -32760], -2000, &physics, &trig).unwrap();
        for index in 0..124 {
            if !(0x2d..0x33).contains(&index) && !(0x39..0x3f).contains(&index) {
                assert_eq!(plan.after[index], before[index], "byte{index:x}");
            }
        }
        assert_eq!(&plan.after[0x3f..0x43], &before[0x3f..0x43]);
    }

    #[test]
    fn zero_impulse_preserves_velocity_and_recomputes_stationary_pitch() {
        let (physics, trig) = tables();
        let mut before = mule();
        before[0x2d..0x33].fill(0);
        put(&mut before, 0x3d, 0);
        let plan = plan_collision(&before, [0, 0], 0, &physics, &trig).unwrap();
        assert_eq!(plan.impulse, [0; 3]);
        assert_eq!(word(&plan.after, 0x39), word(&before, 0x39));
        assert_eq!(word(&plan.after, 0x3b), 512);
        assert_eq!(word(&plan.after, 0x3d), 0);
        before[6] = 3;
        let saved = before;
        assert_eq!(plan_collision(&before, [0, 0], 0, &physics, &trig), Err(CollisionError::UnsupportedMule));
        assert_eq!(before, saved);
    }
}
