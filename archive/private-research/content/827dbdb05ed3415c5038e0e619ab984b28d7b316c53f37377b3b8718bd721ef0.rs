//! Bounded source 6A540, 14EC0 and 4F580 family scheduling.
//!
//! The sole neutral Mule callback is admitted. Owned creatures, newborns,
//! and active kind6/7 entities require their own callback implementations.
use super::*;
use crate::creature_world::{CreatureAnimationBinding, MuleAnimations, MuleStepContext};
use crate::physics_tables::PhysicsTables;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatureUiAnimationRequest {
    /// Address of the source resource pointer, not a host pointer.
    pub resource_global: u32,
    pub descriptor_offset: u16,
    pub state_global: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreaturePassReceipt {
    pub primary_slots: Vec<u16>,
    pub pending_ui_animations: [CreatureUiAnimationRequest; 8],
    pub pending_audio: Vec<u8>,
    pub pending_plant_retirements: Vec<u16>,
    pub pending_portrait_refreshes: Vec<u16>,
}

fn ui_animation_requests() -> [CreatureUiAnimationRequest; 8] {
    core::array::from_fn(|i| CreatureUiAnimationRequest {
        resource_global: if i == 1 { 0x116818 } else { 0x11680c },
        descriptor_offset: [0, 0x15e, 0x8c, 0x172, 0xd2, 0xdc, 0xe6, 0xf0][i],
        state_global: 0x184cbc + i as u32 * 8,
    })
}

impl ActorWorld {
    /// Validate and perform the admitted neutral subset of source 6A540.
    /// Numeric player counters, callback state, links and RNG commit together.
    /// UI animation requests retain source order and remain unconsumed.
    pub fn step_neutral_creature_callbacks(
        &mut self, player_count: u8, context: MuleStepContext,
        physics: &PhysicsTables, trig: &TrigTables, animations: &MuleAnimations,
    ) -> Result<CreaturePassReceipt, ActorWorldError> {
        self.neutral_creature_pass_with(player_count, |world, slot| {
            if slot != 0 { return Err(ActorWorldError::UnsupportedCreatureSchedule); }
            if context.frame_counter == 1 {
                world.step_initial_mule_choice(context.frame_counter, trig)?;
                Ok((Vec::new(),Vec::new(),false))
            } else {
                let receipt=world.step_mule_followup_with_water(context, physics, trig, animations)?;
                Ok((receipt.pending_audio,receipt.pending_plant_retirements,receipt.pending_portrait_refresh))
            }
        })
    }

    fn neutral_creature_pass_with<F>(&mut self, player_count: u8, mut callback: F)
        -> Result<CreaturePassReceipt, ActorWorldError>
    where F: FnMut(&mut Self, u16) -> Result<(Vec<u8>,Vec<u16>,bool), ActorWorldError> {
        if player_count > 4 { return Err(ActorWorldError::InvalidPlayer(player_count)); }
        let pool = self.creatures.as_ref().ok_or(ActorWorldError::UnsupportedCreatureSchedule)?;
        let declared = word(&self.backing, 0x3550);
        let active = pool.records().iter().filter(|r| r[6] != 0).count();
        if declared < 0 || declared as usize != active {
            return Err(ActorWorldError::CreatureCountMismatch { declared, active });
        }
        if active > 1 { return Err(ActorWorldError::UnsupportedCreatureSchedule); }
        for (slot, record) in pool.records().iter().enumerate().filter(|(_, r)| r[6] != 0) {
            if slot != 0 || record[6] != 1 || record[0x18] != 4 || record[0x15] & 1 != 0
                || pool.binding(slot)? != Some(CreatureAnimationBinding::MuleSelector4) {
                return Err(ActorWorldError::UnsupportedCreatureSchedule);
            }
        }
        let mut staged = self.clone();
        // 6A554..6A56B resets +616 for the current source player count.
        for player in 0..usize::from(player_count) {
            let before = *staged.players[player].bytes();
            let mut after = before;
            after[0x616..0x61a].fill(0);
            staged.players[player].replace_bookkeeping_checked(&before, &after)?;
        }
        let mut slots = Vec::new();
        let mut pending_audio=Vec::new();
        let mut pending_plant_retirements=Vec::new();
        let mut pending_portrait_refreshes=Vec::new();
        if active == 1 {
            // Virtual68 at 1EC50 is the pure state>=24 predicate. Owner4
            // skips the player-counter increment for either result.
            let (audio,retirements,portrait)=callback(&mut staged, 0)?;
            pending_audio.extend(audio);pending_plant_retirements.extend(retirements);
            if portrait {pending_portrait_refreshes.push(0);}
            slots.push(0);
        }
        staged.validate_links()?;
        *self = staged;
        Ok(CreaturePassReceipt { primary_slots: slots, pending_ui_animations: ui_animation_requests(), pending_audio, pending_plant_retirements, pending_portrait_refreshes })
    }

    /// Source14EC0 scans all 32 kind7 records. Source4F580 with count zero
    /// invokes no callbacks. Require coherent empty records in both pools.
    pub fn step_empty_families_7_and_6(&self) -> Result<(), ActorWorldError> {
        for kind in [7u8, 6] {
            let group = self.layouts[usize::from(kind - 1)];
            if (0..group.count).any(|i| self.backing[group.file_offset + i * group.packed_stride + 6] != 0) {
                return Err(ActorWorldError::UnsupportedBookkeepingFamily(kind));
            }
        }
        if word(&self.backing, 0x3556) != 0 {
            return Err(ActorWorldError::UnsupportedBookkeepingFamily(6));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> ActorWorld {
        let mut bytes = vec![0; 343_958];
        bytes[..4].copy_from_slice(&0x55336_u32.to_le_bytes());
        bytes[337_054..337_058].copy_from_slice(&0x6be_u32.to_le_bytes());
        let players = core::array::from_fn(|id| {
            let mut record = *PlayerState::fresh_campaign(id as u8, [4096,4096], b"fixture\0").unwrap().bytes();
            record[0x616..0x61a].copy_from_slice(&(0x12345678 + id as u32).to_le_bytes());
            PlayerState::from_record(&record).unwrap()
        });
        let mut world = ActorWorld::from_parts(&ConqMissionSource::parse(&bytes).unwrap(), &[[0;200];20],
            [ActorHandleBindings::empty();20], players, BlueprintRng { seed: 5428, ring: 0, draw_count: 0 }).unwrap();
        world.admit_creatures(&[[0;124];300]).unwrap();
        world
    }
    #[test]
    fn empty_creature_pass_resets_only_current_player_counters_and_orders_ui() {
        for count in 0..=4 {
            let mut world=fixture(); let before=world.clone();
            let receipt=world.neutral_creature_pass_with(count, |_,_| panic!("empty pool callback")).unwrap();
            assert!(receipt.primary_slots.is_empty());
            for p in 0..4 { let mut expected=*before.players[p].bytes();
                if p<usize::from(count) { expected[0x616..0x61a].fill(0); }
                assert_eq!(world.players[p].bytes(),&expected);
            }
            assert_eq!(world.backing,before.backing);assert_eq!(world.rng,before.rng);
            assert_eq!(receipt.pending_ui_animations[1].descriptor_offset,0x15e);
            assert_eq!(receipt.pending_ui_animations[7].state_global,0x184cf4);
        }
    }
    #[test]
    fn inconsistent_counts_and_hidden_empty_family_members_reject_without_changes() {
        let mut world=fixture();world.backing[0x3550..0x3552].copy_from_slice(&1i16.to_le_bytes());let before=world.clone();
        assert!(matches!(world.neutral_creature_pass_with(4,|_,_|Ok((Vec::new(),Vec::new(),false))),Err(ActorWorldError::CreatureCountMismatch{..})));
        assert_eq!(world,before);
        for kind in [6u8,7] {
            let mut world=fixture();assert!(world.step_empty_families_7_and_6().is_ok());
            let group=world.layouts[usize::from(kind-1)];world.backing[group.file_offset+(group.count-1)*group.packed_stride+6]=kind;
            let before=world.clone();assert!(world.step_empty_families_7_and_6().is_err());assert_eq!(world,before);
        }
        let mut world=fixture();world.backing[0x3556]=1;assert!(world.step_empty_families_7_and_6().is_err());
    }
    #[test]
    fn creature_callback_failure_rolls_back_counters_rng_and_backing() {
        let mut world=fixture();
        let mut packed=vec![0;300*116];packed[6]=1;packed[24]=4;packed[0x7a-8]=4;packed[0x5e-8]=8;
        world.creatures=Some(Box::new(CreaturePool::from_packed(&packed,&[[0;124];300]).unwrap()));
        world.backing[0x3550..0x3552].copy_from_slice(&1i16.to_le_bytes());
        world.backing[MAP_START+6..MAP_START+9].copy_from_slice(&[0,0,1]);
        let before=world.clone();
        let result=world.neutral_creature_pass_with(4,|staged,slot| {
            assert_eq!(slot,0);assert_eq!(&staged.players[3].bytes()[0x616..0x61a],&[0;4]);
            staged.rng.draw_count=99;staged.backing[MAP_START+12]=55;
            Err(CreatureError::UnsupportedMotionState.into())
        });
        assert!(result.is_err());assert_eq!(world,before);
    }
}
