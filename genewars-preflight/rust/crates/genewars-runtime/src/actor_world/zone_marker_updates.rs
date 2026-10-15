//! Atomic kind5/code3C callback using the proved EFF60 motion/retirement packet.
//! Source virtual70 is false: no building query, children or RNG draws here.
use super::*;
use crate::landing_effect_lifecycle::{CleanupRecord, RetirementUiState};
use crate::physics_tables::PhysicsTables;
use crate::zone_score_marker_motion as marker;

impl ActorWorld {
    pub fn step_zone_score_marker(
        &mut self,
        slot: u16,
        context: LandingPhysicsContext,
        physics: &PhysicsTables,
        trig: &TrigTables,
        ui: &RetirementUiState,
    ) -> Result<EffectStepReceipt, ActorWorldError> {
        if !self.validate_links()?.contains(&(5, slot as i16)) {
            return Err(ActorWorldError::MissingEffectLink(slot));
        }
        self.validate_effect_count()?;
        let pool = self.effects.as_ref().ok_or(ActorWorldError::EffectsNotAdmitted)?;
        if pool.binding(usize::from(slot))? != Some(EffectAnimationBinding::EffAnimation60) {
            return Err(ActorWorldError::EffectBinding(slot));
        }
        let before = *pool.record(usize::from(slot))?;
        if before[6] != 5 || before[69] != 0x3c || before[24] > 4
            || word(&before, 15) != slot as i16
        {
            return Err(ActorWorldError::UnsupportedEffectEvent);
        }
        let at = marker::physics_candidate(&before)?;
        let (ground, shade) = self.effect_ground_sample(at)?;
        let height = i32::from_le_bytes(before[39..43].try_into().unwrap());
        let old_ground = i32::from_le_bytes(before[17..21].try_into().unwrap());
        let next_height = height.wrapping_add(
            i32::from(word(&before, 49).wrapping_add(word(&before, 55))) >> 3,
        );
        let flight = if height <= old_ground && next_height <= ground {
            let candidate = marker::grounded_candidate(
                &before, ground, physics.angles(),
                trig.sine().try_into().unwrap(), trig.cosine().try_into().unwrap(),
            )?;
            let point = [word(&candidate, 7), word(&candidate, 9)];
            let (second_ground, second_shade) = self.effect_ground_sample(point)?;
            marker::plan_grounded_after_second_sample(
                &before, &candidate, second_ground, second_shade, context.water_height,
            )?
        } else {
            // Same exact wrapped four samples as step_particle's private
            // effect_height_corners; no visibility change to that sibling.
            let x = (i32::from(at[0]) >> 8) & 127;
            let y = (i32::from(at[1]) >> 8) & 127;
            let corners = [(0, 0), (1, 0), (0, 1), (1, 1)].map(|(dx, dy)| {
                word(self.map(), ((((y + dy) & 127) * 128 + ((x + dx) & 127)) * 14 + 2) as usize)
            });
            marker::plan_flight(
                &before, ground, shade, context.water_height, corners,
                physics.bounce(self.terrain_class_at(at)?)?, physics.angles(),
            )?
        };
        let changed_cell = floor_cell([word(&before, 7), word(&before, 9)])
            != floor_cell([word(&flight.after, 7), word(&flight.after, 9)]);
        let mut staged = self.clone();
        if changed_cell { staged.unlink_effect(slot)?; }
        staged.effects.as_mut().unwrap().replace_record_checked(
            usize::from(slot), &before, &flight.after,
        )?;
        // Type5 virtual88 is1F030 RET; registry insertion has no other callback.
        if changed_cell { staged.insert_effect_head(slot)?; }
        let moved = *staged.effects.as_ref().unwrap().record(usize::from(slot))?;
        let animation = marker::plan_animation_after_physics(&moved)?;
        staged.effects.as_mut().unwrap().replace_record_checked(
            usize::from(slot), &moved, &animation.after,
        )?;
        let mut receipt = EffectStepReceipt {
            slot, created_effect: None, changed_cell, retired: false,
            cleaned_reference_bytes: 0, ui_after: ui.clone(), pending_ui_events: Vec::new(),
        };
        if animation.retirement_required {
            let current = *staged.effects.as_ref().unwrap().record(usize::from(slot))?;
            let records = staged.effect_cleanup_records();
            let views = records.iter().map(|(kind, slot, bytes)| CleanupRecord {
                kind: *kind, slot: *slot, bytes,
            }).collect::<Vec<_>>();
            let plan = marker::plan_retirement(
                &current, word(&staged.backing, 0x3554), &views, ui,
            )?;
            staged.unlink_effect(slot)?;
            for patch in &plan.cleanup {
                if patch.kind == 5 && patch.slot == slot { continue; }
                staged.apply_cleanup_byte(*patch)?;
            }
            staged.effects.as_mut().unwrap().retire_checked(
                usize::from(slot), &current, &plan.after,
            )?;
            staged.backing[0x3554..0x3556].copy_from_slice(&plan.pool_count_after.to_le_bytes());
            receipt.retired = true;
            receipt.cleaned_reference_bytes = plan.cleanup.len();
            receipt.ui_after = plan.ui_after;
            receipt.pending_ui_events = plan.ui_events;
        }
        staged.validate_links()?;
        staged.validate_effect_count()?;
        // No random values or child factories exist in the descriptor3C tail.
        *self = staged;
        Ok(receipt)
    }
}
