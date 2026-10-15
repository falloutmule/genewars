//! Bounded source 6A540, 14EC0 and 4F580 family scheduling.
//!
//! Mule callbacks use ascending live slots and shared world/RNG. Marked newborns
//! and active kind6/7 entities remain outside the admitted scheduler.
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
    pub retirements: Vec<MuleRetirementReceipt>,
    pub newborn_slots: Vec<u16>,
    pub pending_reveal_vertices: Vec<[i16; 2]>,
}

fn ui_animation_requests() -> [CreatureUiAnimationRequest; 8] {
    core::array::from_fn(|i| CreatureUiAnimationRequest {
        resource_global: if i == 1 { 0x116818 } else { 0x11680c },
        descriptor_offset: [0, 0x15e, 0x8c, 0x172, 0xd2, 0xdc, 0xe6, 0xf0][i],
        state_global: 0x184cbc + i as u32 * 8,
    })
}

impl ActorWorld {
    /// Validate and perform the admitted no-birth neutral subset of source 6A540.
    /// Numeric player counters, callback state, links and RNG commit together.
    /// UI animation requests retain source order and remain unconsumed.
    pub fn step_neutral_creature_callbacks(
        &mut self, player_count: u8, context: MuleStepContext,
        physics: &PhysicsTables, trig: &TrigTables, animations: &MuleAnimations,
    ) -> Result<CreaturePassReceipt, ActorWorldError> {
        self.neutral_creature_pass_with(player_count, |world, slot| {
            // The callback must consume the current world even at frame1:
            // authentic startup has already executed frame0 and refreshed shade.
            let receipt=world.step_neutral_mule_slot_with_water(slot,context, physics, trig, animations)?;
            Ok((receipt.pending_audio,receipt.pending_plant_retirements,receipt.pending_portrait_refresh,receipt.retirement,receipt.newborn_slots))
        })
    }

    /// Source 6A540 with independently admitted neutral and owned Mule callbacks.
    /// The selected player and runtime flags are source globals, not actor selection.
    pub fn step_creature_callbacks(
        &mut self, player_count: u8, selected_player: u8, runtime_flags: u16,
        context: MuleStepContext, physics: &PhysicsTables, trig: &TrigTables,
        animations: &MuleAnimations,
    ) -> Result<CreaturePassReceipt, ActorWorldError> {
        self.creature_pass_with(player_count, true, |world, slot| {
            let owner=world.creatures.as_ref().unwrap().record(usize::from(slot))?[0x18];
            if owner==4 {
                let r=world.step_neutral_mule_slot_with_water(slot,context,physics,trig,animations)?;
                Ok((r.pending_audio,r.pending_plant_retirements,r.pending_portrait_refresh,r.retirement,r.newborn_slots,Vec::new()))
            } else {
                let r=world.step_owned_mule_slot(slot,selected_player,runtime_flags,context,physics,trig,animations)?;
                Ok((Vec::new(),Vec::new(),false,None,Vec::new(),r.pending_reveal_vertices))
            }
        })
    }

    fn neutral_creature_pass_with<F>(&mut self, player_count: u8, mut callback: F)
        -> Result<CreaturePassReceipt, ActorWorldError>
    where F: FnMut(&mut Self, u16) -> Result<(Vec<u8>,Vec<u16>,bool,Option<MuleRetirementReceipt>,Vec<u16>), ActorWorldError> {
        self.creature_pass_with(player_count, false, |world, slot| {
            let (audio,plants,portrait,retirement,newborns)=callback(world,slot)?;
            Ok((audio,plants,portrait,retirement,newborns,Vec::new()))
        })
    }

    fn creature_pass_with<F>(&mut self, player_count: u8, admit_owned: bool, mut callback: F)
        -> Result<CreaturePassReceipt, ActorWorldError>
    where F: FnMut(&mut Self, u16) -> Result<(Vec<u8>,Vec<u16>,bool,Option<MuleRetirementReceipt>,Vec<u16>,Vec<[i16;2]>), ActorWorldError> {
        if player_count > 4 { return Err(ActorWorldError::InvalidPlayer(player_count)); }
        let pool = self.creatures.as_ref().ok_or(ActorWorldError::UnsupportedCreatureSchedule)?;
        let declared = word(&self.backing, 0x3550);
        let active = pool.records().iter().filter(|r| r[6] != 0).count();
        if declared < 0 || declared as usize != active {
            return Err(ActorWorldError::CreatureCountMismatch { declared, active });
        }
        for (slot, record) in pool.records().iter().enumerate().filter(|(_, r)| r[6] != 0) {
            if record[6] != 1 || (if admit_owned {record[0x18]>4}else{record[0x18]!=4}) || record[0x15] & 1 != 0
                || pool.binding(slot)? != Some(if record[0x5e]>=26 {
                    CreatureAnimationBinding::BonesSelector4
                }else{CreatureAnimationBinding::MuleSelector4}) {
                return Err(ActorWorldError::UnsupportedCreatureSchedule);
            }
        }
        let initial_live:HashSet<u16>=pool.records().iter().enumerate().filter_map(|(i,r)|(r[6]!=0).then_some(i as u16)).collect();
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
        let mut pending_portrait_refreshes=Vec::new();let mut retirements=Vec::new();let mut newborn_slots=Vec::new();
        let mut pending_reveal_vertices=Vec::new();
        let mut remaining=declared as usize;
        for slot in 0..300usize {
            if remaining==0{break;}
            let record=staged.creatures.as_ref().unwrap().record(slot)?;
            if record[6]==0{continue;}
            if record[0x15]&1!=0{return Err(ActorWorldError::UnsupportedCreatureSchedule);}
            // 6A5A4..6A5D5: virtual68 is 1EC50 (state >= 24).
            // Count before virtual64; owner4 never contributes.
            if admit_owned && record[0x18]<4 && record[0x5e]<24 {
                let owner=usize::from(record[0x18]);
                let before=*staged.players[owner].bytes();let mut after=before;
                let count=u32::from_le_bytes(before[0x616..0x61a].try_into().unwrap());
                after[0x616..0x61a].copy_from_slice(&count.wrapping_add(1).to_le_bytes());
                staged.players[owner].replace_bookkeeping_checked(&before,&after)?;
            }
            let (audio,plant_retirements,portrait,retirement,newborns,reveals)=callback(&mut staged,slot as u16)?;
            pending_reveal_vertices.extend(reveals);
            newborn_slots.extend(newborns);
            if let Some(receipt)=retirement{retirements.push(receipt);}
            pending_audio.extend(audio);pending_plant_retirements.extend(plant_retirements);
            if portrait{pending_portrait_refreshes.push(slot as u16);}
            slots.push(slot as u16);remaining-=1;
        }
        if remaining!=0||staged.creatures.as_ref().unwrap().records().iter().any(|r|r[6]!=0&&r[0x15]&1!=0){return Err(ActorWorldError::UnsupportedCreatureSchedule);}
        let final_active=staged.creatures.as_ref().unwrap().records().iter().filter(|r|r[6]!=0).count();
        if final_active+retirements.len()!=active+newborn_slots.len()||word(&staged.backing,0x3550)as usize!=final_active {return Err(ActorWorldError::UnsupportedCreatureSchedule);}
        let mut reported=HashSet::new();
        for id in &newborn_slots{if initial_live.contains(id)||!reported.insert(*id)||staged.creatures.as_ref().unwrap().record(usize::from(*id))?[6]!=1{return Err(ActorWorldError::UnsupportedCreatureSchedule);}}
        staged.validate_links()?;
        *self = staged;
        Ok(CreaturePassReceipt { primary_slots: slots, pending_ui_animations: ui_animation_requests(), pending_audio, pending_plant_retirements, pending_portrait_refreshes, retirements, newborn_slots, pending_reveal_vertices })
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
        assert!(matches!(world.neutral_creature_pass_with(4,|_,_|Ok((Vec::new(),Vec::new(),false,None,vec![]))),Err(ActorWorldError::CreatureCountMismatch{..})));
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

#[cfg(test)]
mod multi_slot_tests {
    use super::*;
    const SLOTS: [u16;4] = [0,3,9,17];
    fn fixture(bones: bool, owner: u8, newborn: bool) -> ActorWorld {
        let mut bytes=vec![0;343_958];bytes[..4].copy_from_slice(&0x55336u32.to_le_bytes());bytes[337_054..337_058].copy_from_slice(&0x6beu32.to_le_bytes());
        let players=core::array::from_fn(|id| {let mut r=*PlayerState::fresh_campaign(id as u8,[4096,4096],b"fixture\0").unwrap().bytes();r[0x616..0x61a].copy_from_slice(&(100+id as u32).to_le_bytes());PlayerState::from_record(&r).unwrap()});
        let mut world=ActorWorld::from_parts(&ConqMissionSource::parse(&bytes).unwrap(),&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed:5428,ring:0,draw_count:0}).unwrap();
        let mut packed=vec![0;300*116];
        for slot in SLOTS {let i=slot as usize;let r=&mut packed[i*116..(i+1)*116];r[..2].copy_from_slice(&slot.to_le_bytes());r[6]=1;r[7..9].copy_from_slice(&(slot as i16*256).to_le_bytes());r[15..17].copy_from_slice(&slot.to_le_bytes());r[21]=u8::from(newborn);r[24]=owner;r[0x7a-8]=4;r[0x5e-8]=if bones{25}else{8};world.backing[MAP_START+i*14+6..MAP_START+i*14+9].copy_from_slice(&[slot as u8,(slot>>8)as u8,1]);}
        world.creatures=Some(Box::new(CreaturePool::from_packed(&packed,&[[0;124];300]).unwrap()));if bones { for slot in SLOTS { let old=*world.creatures.as_ref().unwrap().record(slot as usize).unwrap();let mut new=old;new[0x5e]=27;world.creatures.as_mut().unwrap().replace_record_checked(slot as usize,&old,&new).unwrap(); } }world.backing[0x3550..0x3552].copy_from_slice(&4i16.to_le_bytes());world.validate_links().unwrap();world
    }
    #[test]
    fn sparse_four_match_literal_scheduler_order_counters_and_ui() {
        let mut w=fixture(false,4,false);let mut seen=Vec::new();
        let r=w.neutral_creature_pass_with(4,|s,id|{seen.push(id);s.rng.draw_count+=1;Ok((vec![id as u8],vec![],true,None,vec![]))}).unwrap();
        assert_eq!(seen,SLOTS);assert_eq!(r.primary_slots,SLOTS);assert_eq!(r.pending_portrait_refreshes,SLOTS);assert_eq!(r.pending_audio,[0,3,9,17]);assert_eq!(w.rng.draw_count,4);
        for p in &w.players {assert_eq!(&p.bytes()[0x616..0x61a],&[0;4]);}
        assert_eq!(r.pending_ui_animations.map(|x|x.state_global),core::array::from_fn(|i|0x184cbc+i as u32*8));w.validate_links().unwrap();
    }
    fn mixed_fixture() -> ActorWorld {
        let mut world=fixture(false,4,false);
        // Admit each owner as initial packed identity; checked updates must
        // never transfer a live creature to another owner.
        let mut packed=Vec::with_capacity(300*116);
        for record in world.creatures.as_ref().unwrap().records() {
            packed.extend_from_slice(&record[..31]);packed.extend_from_slice(&record[39..]);
        }
        // Source virtual68 is state>=24; state15 is still counted here.
        for (slot,owner,state) in [(0,0,8),(3,4,8),(9,1,24),(17,2,15)] {
            packed[slot*116+0x18]=owner;packed[slot*116+0x5e-8]=state;
        }
        world.creatures=Some(Box::new(CreaturePool::from_packed(&packed,&[[0;124];300]).unwrap()));
        world.validate_links().unwrap();world
    }
    #[test]
    fn mixed_creature_primary_pass_preserves_source_order_predicate_and_counter_reset_scope() {
        let mut world=mixed_fixture();let before=world.clone();let mut seen=Vec::new();
        let receipt=world.creature_pass_with(1,true,|staged,slot| {
            // Each callback sees prior callbacks and its own pre-callback count.
            assert_eq!(staged.rng.draw_count,seen.len() as u32);
            let counters=staged.players.iter().map(|p|u32::from_le_bytes(p.bytes()[0x616..0x61a].try_into().unwrap())).collect::<Vec<_>>();
            assert_eq!(counters,if slot==17 {vec![1,101,103,103]}else{vec![1,101,102,103]});
            seen.push(slot);staged.rng.draw_count+=1;
            let reveals=if slot==0||slot==17 {vec![[slot as i16,256]]}else{vec![]};
            Ok((vec![],vec![],false,None,vec![],reveals))
        }).unwrap();
        assert_eq!(seen,SLOTS);assert_eq!(receipt.primary_slots,SLOTS);
        assert_eq!(receipt.pending_reveal_vertices,[[0,256],[17,256]]);
        assert_eq!(world.rng.draw_count,4);
        let mut expected=*before.players[0].bytes();expected[0x616..0x61a].copy_from_slice(&1u32.to_le_bytes());
        assert_eq!(world.players[0].bytes(),&expected);
        assert_eq!(world.players[1],before.players[1]);assert_eq!(world.players[3],before.players[3]);
        let mut expected=*before.players[2].bytes();expected[0x616..0x61a].copy_from_slice(&103u32.to_le_bytes());
        assert_eq!(world.players[2].bytes(),&expected);
        world.validate_links().unwrap();
    }
    #[test]
    fn mixed_creature_late_owned_failure_restores_all_world_state() {
        let mut world=mixed_fixture();let before=world.clone();let mut seen=Vec::new();
        let result=world.creature_pass_with(1,true,|staged,slot| {
            seen.push(slot);staged.rng.seed=12;staged.rng.draw_count+=1;staged.backing[MAP_START+12]=55;
            if slot==17 {return Err(CreatureError::UnsupportedNeighborhood.into());}
            Ok((vec![],vec![],false,None,vec![],vec![[0,256]]))
        });
        assert!(result.is_err());assert_eq!(seen,SLOTS);assert_eq!(world,before);
    }
    #[test]
    fn current_slot_retirement_does_not_shorten_initial_scan_count() {
        let mut w=fixture(true,4,false);let r=w.neutral_creature_pass_with(4,|s,id|{
            if id!=0{return Ok((vec![],vec![],false,None,vec![]));}
            let before=*s.creatures.as_ref().unwrap().record(0)?;
            s.backing[MAP_START+8]=0;s.creatures.as_mut().unwrap().retire_bones_checked(0,&before)?;s.backing[0x3550..0x3552].copy_from_slice(&3i16.to_le_bytes());
            let after=*s.creatures.as_ref().unwrap().record(0)?;
            let receipt=MuleRetirementReceipt{before,after,cleanup:vec![],count_after:3,pending_ui_cleanup:[MuleUiCleanupRequest::Kind1List{root_global:0x1860be,id:0},MuleUiCleanupRequest::ModuleReference{root_global:0x185dfc,reference:[0,0,1]}],pending_display_phase_1185a8:11};
            Ok((vec![],vec![],false,Some(receipt),vec![]))
        }).unwrap();assert_eq!(r.primary_slots,SLOTS);assert_eq!(r.retirements.len(),1);assert_eq!(word(&w.backing,0x3550),3);w.validate_links().unwrap();
    }
    #[test]
    fn late_error_restores_every_world_field() {
        let mut w=fixture(false,4,false);let before=w.clone();let mut seen=Vec::new();
        let r=w.neutral_creature_pass_with(4,|s,id|{seen.push(id);s.rng.seed=12;s.rng.draw_count+=1;s.backing[MAP_START+12]=99;if id==17{s.backing[MAP_START+8]=0;return Err(CreatureError::UnsupportedMotionState.into());}Ok((vec![],vec![],false,None,vec![]))});assert!(r.is_err());assert_eq!(seen,SLOTS);assert_eq!(w,before);
    }
    #[test]
    fn owned_and_initial_newborn_are_explicitly_blocked_without_callbacks() {
        for (owner,newborn) in [(0,false),(4,true)] {let mut w=fixture(false,owner,newborn);let before=w.clone();assert!(matches!(w.neutral_creature_pass_with(4,|_,_|panic!("blocked admission invoked callback")),Err(ActorWorldError::UnsupportedCreatureSchedule)));assert_eq!(w,before);}
    }
    #[test]
    fn callback_newborn_marker_rolls_back_whole_pass() {
        let mut w=fixture(false,4,false);let before=w.clone();let r=w.neutral_creature_pass_with(4,|s,id|{if id==0{let old=*s.creatures.as_ref().unwrap().record(3)?;let mut new=old;new[0x15]|=1;s.creatures.as_mut().unwrap().replace_record_checked(3,&old,&new)?;}Ok((vec![],vec![],false,None,vec![]))});assert!(matches!(r,Err(ActorWorldError::UnsupportedCreatureSchedule)));assert_eq!(w,before);
    }
    #[test]
    fn reported_unmarked_birth_behind_scan_waits_until_next_pass() {
        let mut w=fixture(false,4,false);let trig=TrigTables::generate_pinned().unwrap();let mut seen=Vec::new();
        let receipt=w.neutral_creature_pass_with(4,|s,id|{
            seen.push(id);let newborns=if id==17 {let born=s.create_mule(4,[8000,1000],50,None,&trig)?.unwrap();assert_eq!(born.slot,1);vec![born.slot]}else{vec![]};
            Ok((vec![],vec![],false,None,newborns))
        }).unwrap();
        assert_eq!(seen,SLOTS);assert_eq!(receipt.primary_slots,SLOTS);assert_eq!(receipt.newborn_slots,[1]);assert_eq!(word(&w.backing,0x3550),5);
        let newborn=w.creatures.as_ref().unwrap().record(1).unwrap();assert_eq!(newborn[6],1);assert_eq!(newborn[0x15]&1,0);w.validate_links().unwrap();
        let next=w.neutral_creature_pass_with(4,|_,_|Ok((vec![],vec![],false,None,vec![]))).unwrap();assert_eq!(next.primary_slots,[0,1,3,9,17]);
    }
    #[test]
    fn unreported_birth_rolls_back_pool_map_rng_and_count() {
        let mut w=fixture(false,4,false);let before=w.clone();let trig=TrigTables::generate_pinned().unwrap();let mut seen=Vec::new();
        let result=w.neutral_creature_pass_with(4,|s,id|{
            seen.push(id);if id==17 {assert_eq!(s.create_mule(4,[8000,1000],50,None,&trig)?.unwrap().slot,1);}
            Ok((vec![],vec![],false,None,vec![]))
        });
        assert!(matches!(result,Err(ActorWorldError::UnsupportedCreatureSchedule)));assert_eq!(seen,SLOTS);assert_eq!(w,before);
    }
}
