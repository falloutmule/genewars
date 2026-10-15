//! Native81-byte numeric-family4 storage. The inactive49FD0 loader copies
//! packed[0,31) and[31,73) into native[0,31) and[39,81), preserving its gap.
//! Active serialized records need49E90 initialization and are not admitted.

use crate::landing::{LandingFactoryPlan, LandingRecord, FAMILY4_CAPACITY};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectAnimationBinding { EffAnimation49, EffAnimation6, EffAnimation47, EffAnimation48, EffAnimation26, EffAnimation21, EffAnimation22, EffAnimation44, EffAnimation33, EffAnimation32, EffAnimation60 }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectPoolError {
    PackedLength(usize), ActivePackedRecord(usize), InvalidSlot(usize),
    StaleRecord(usize), OccupiedSlot(usize), StaleBinding(usize),
    InvalidFactoryPlan, InvalidRetirement, IdentityChanged(usize), PointerBytesChanged(usize),
}
impl core::fmt::Display for EffectPoolError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f,"{self:?}") }
}
impl std::error::Error for EffectPoolError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectPool {
    records: [LandingRecord; FAMILY4_CAPACITY],
    bindings: [Option<EffectAnimationBinding>; FAMILY4_CAPACITY],
}

impl EffectPool {
    /// Test-only callback replay; caller validates identities, bindings, links,
    /// counts and exact captured-buffer roundtrip before invoking gameplay.
    #[cfg(test)]
    pub(crate) fn from_captured_records(records: [LandingRecord; 200], bindings: [Option<EffectAnimationBinding>; 200]) -> Self {
        Self { records, bindings }
    }

    pub fn from_packed_inactive(
        packed: &[u8], prior: &[LandingRecord; FAMILY4_CAPACITY],
        bindings: [Option<EffectAnimationBinding>; FAMILY4_CAPACITY],
    ) -> Result<Self, EffectPoolError> {
        if packed.len() != FAMILY4_CAPACITY * 73 { return Err(EffectPoolError::PackedLength(packed.len())); }
        let mut records = *prior;
        for (slot, source) in packed.chunks_exact(73).enumerate() {
            if source[6] != 0 { return Err(EffectPoolError::ActivePackedRecord(slot)); }
            records[slot][..31].copy_from_slice(&source[..31]);
            records[slot][39..81].copy_from_slice(&source[31..73]);
        }
        Ok(Self { records, bindings })
    }
    pub fn records(&self) -> &[LandingRecord; FAMILY4_CAPACITY] { &self.records }
    pub fn record(&self, slot: usize) -> Result<&LandingRecord, EffectPoolError> {
        self.records.get(slot).ok_or(EffectPoolError::InvalidSlot(slot))
    }
    pub fn binding(&self, slot: usize) -> Result<Option<EffectAnimationBinding>, EffectPoolError> {
        self.bindings.get(slot).copied().ok_or(EffectPoolError::InvalidSlot(slot))
    }

    pub(crate) fn commit_landing_factory(&mut self, plan: &LandingFactoryPlan) -> Result<(), EffectPoolError> {
        let slot = usize::from(plan.slot);
        self.commit_factory_record(slot, &plan.before, &plan.after_nonpointer, plan.animation_index, plan.animation_pointer_offset)
    }

    pub(crate) fn commit_building_factory(&mut self, plan: &crate::building_effect::BuildingEffectPlan) -> Result<(), EffectPoolError> {
        if plan.after[69] != 0x2c || plan.animation_index != 44 { return Err(EffectPoolError::InvalidFactoryPlan); }
        let slot = u16::from_le_bytes(plan.after[15..17].try_into().unwrap()) as usize;
        self.commit_factory_record(slot, &plan.before, &plan.after, plan.animation_index, 0x1f)
    }

    fn commit_factory_record(&mut self, slot: usize, prior: &LandingRecord, after: &LandingRecord,
        animation_index: u16, pointer_offset: u8) -> Result<(), EffectPoolError> {
        let before = self.record(slot)?;
        if before != prior { return Err(EffectPoolError::StaleRecord(slot)); }
        if before[6] != 0 { return Err(EffectPoolError::OccupiedSlot(slot)); }
        if self.binding(slot)?.is_some() { return Err(EffectPoolError::StaleBinding(slot)); }
        let binding = match (after[69],animation_index) {
            (0x31,49) => EffectAnimationBinding::EffAnimation49,
            (6,6) => EffectAnimationBinding::EffAnimation6,
            (0x30,48) => EffectAnimationBinding::EffAnimation48,
            (0x2F,47) => EffectAnimationBinding::EffAnimation47,
            (0x1A,26) => EffectAnimationBinding::EffAnimation26,
            (0x15,21) => EffectAnimationBinding::EffAnimation21,
            (0x16,22) => EffectAnimationBinding::EffAnimation22,
            (0x2c,44) => EffectAnimationBinding::EffAnimation44,
            (0x21,33) => EffectAnimationBinding::EffAnimation33,
            (0x20,32) => EffectAnimationBinding::EffAnimation32,
            (0x3c,60) => EffectAnimationBinding::EffAnimation60,
            _ => return Err(EffectPoolError::InvalidFactoryPlan),
        };
        if self.records.iter().position(|r| r[6] == 0) != Some(slot)
            || after[6] != 5 || word(after,15) != slot as i16
            || (after[24] >= 4 && !(after[69] == 0x3c && after[24] == 4))
            || pointer_offset != 0x1F {
            return Err(EffectPoolError::InvalidFactoryPlan);
        }
        if after[31..35] != [0;4] || after[35..39] != before[35..39] {
            return Err(EffectPoolError::PointerBytesChanged(slot));
        }
        self.records[slot] = *after;
        self.bindings[slot] = Some(binding);
        Ok(())
    }

    pub(crate) fn replace_record_checked(&mut self, slot: usize, before: &LandingRecord, after: &LandingRecord) -> Result<(), EffectPoolError> {
        if self.record(slot)? != before { return Err(EffectPoolError::StaleRecord(slot)); }
        if before[6] != after[6] || before[15..17] != after[15..17] || before[24] != after[24] || before[69] != after[69] {
            return Err(EffectPoolError::IdentityChanged(slot));
        }
        if before[31..39] != after[31..39] { return Err(EffectPoolError::PointerBytesChanged(slot)); }
        self.records[slot] = *after;
        Ok(())
    }
    pub(crate) fn retire_checked(&mut self,slot:usize,before:&LandingRecord,after:&LandingRecord)->Result<(),EffectPoolError> {
        if self.record(slot)?!=before { return Err(EffectPoolError::StaleRecord(slot)); }
        let expected_binding=match before[69] {0x3c=>EffectAnimationBinding::EffAnimation60,0x20=>EffectAnimationBinding::EffAnimation32,0x21=>EffectAnimationBinding::EffAnimation33,0x2c=>EffectAnimationBinding::EffAnimation44,0x15=>EffectAnimationBinding::EffAnimation21,0x16=>EffectAnimationBinding::EffAnimation22,0x1A=>EffectAnimationBinding::EffAnimation26,6=>EffectAnimationBinding::EffAnimation6,0x2F=>EffectAnimationBinding::EffAnimation47,0x30=>EffectAnimationBinding::EffAnimation48,0x31=>EffectAnimationBinding::EffAnimation49,_=>return Err(EffectPoolError::InvalidRetirement) };
        if before[6]!=5||self.binding(slot)?!=Some(expected_binding) { return Err(EffectPoolError::InvalidRetirement); }
        let mut expected=*before;expected[6]=0;
        if expected[73]==5&&word(&expected,71)==slot as i16 { expected[73]=0; }
        if *after!=expected { return Err(EffectPoolError::InvalidRetirement); }
        self.records[slot]=*after;self.bindings[slot]=None;Ok(())
    }
}

fn word(record: &LandingRecord, offset: usize) -> i16 { i16::from_le_bytes([record[offset],record[offset+1]]) }

#[cfg(test)]
mod tests {
    use super::*;
    fn score_marker(pool: &EffectPool, owner: u8) -> crate::zone_score_marker::MarkerPlan {
        let mut mule=[0;124];mule[6]=1;mule[24]=owner;mule[122]=4;
        crate::zone_score_marker::plan_one(&mule,pool.records(),0,10,10,-1,
            0,0,None,1024,crate::landing::LandingRng{seed:5428,ring:31,draw_count:0},
            &[0;2048],&[0;2048]).unwrap()
    }
    #[test]
    fn source_score_marker_binds_and_retires_for_owned_and_neutral_mules() {
        for owner in [0,4] {
            let mut pool=EffectPool::from_packed_inactive(&vec![0;200*73],&[[0;81];200],[None;200]).unwrap();
            let marker=score_marker(&pool,owner);
            pool.commit_landing_factory(&marker.factory).unwrap();
            assert_eq!(pool.binding(0).unwrap(),Some(EffectAnimationBinding::EffAnimation60));
            assert_eq!(pool.record(0).unwrap()[24],owner);
            let before=*pool.record(0).unwrap();let mut after=before;after[6]=0;
            let snapshot=pool.clone();
            pool.bindings[0]=Some(EffectAnimationBinding::EffAnimation6);
            assert_eq!(pool.retire_checked(0,&before,&after),Err(EffectPoolError::InvalidRetirement));
            pool=snapshot;
            pool.retire_checked(0,&before,&after).unwrap();
            assert_eq!(pool.binding(0).unwrap(),None);
        }
    }
    #[test]
    fn score_marker_mismatched_selector_owner_and_pool_admission_reject_atomically() {
        let pool=EffectPool::from_packed_inactive(&vec![0;200*73],&[[0;81];200],[None;200]).unwrap();
        let marker=score_marker(&pool,0);
        for (code,animation,owner) in [(6,60,0),(0x3c,6,0),(0x3c,60,5),(6,6,4)] {
            let mut changed=marker.factory.clone();changed.after_nonpointer[69]=code;
            changed.animation_index=animation;changed.after_nonpointer[24]=owner;
            let mut candidate=pool.clone();
            assert_eq!(candidate.commit_landing_factory(&changed),Err(EffectPoolError::InvalidFactoryPlan));
            assert_eq!(candidate,pool);
        }
        let mut occupied=pool.clone();occupied.commit_landing_factory(&marker.factory).unwrap();
        let snapshot=occupied.clone();
        assert!(occupied.commit_landing_factory(&marker.factory).is_err());assert_eq!(occupied,snapshot);
        let mut mule=[0;124];mule[6]=1;mule[122]=4;
        assert_eq!(crate::zone_score_marker::plan_one(&mule,pool.records(),200,10,10,-1,
            0,0,None,1024,crate::landing::LandingRng{seed:1,ring:0,draw_count:0},
            &[0;2048],&[0;2048]),Err(crate::zone_score_marker::MarkerError::FullPoolReplacement));
    }
    #[test]
    fn inactive_loader_preserves_native_gap_for_every_slot() {
        let mut source = vec![0;200*73];
        for (i, value) in source.iter_mut().enumerate() { *value = (i * 37) as u8; }
        for slot in 0..200 { source[slot * 73 + 6] = 0; }
        let prior = std::array::from_fn(|slot| std::array::from_fn(|offset| (slot + offset * 11) as u8));
        let pool = EffectPool::from_packed_inactive(&source,&prior,[None;200]).unwrap();
        for slot in 0..200 {
            assert_eq!(&pool.record(slot).unwrap()[..31], &source[slot*73..slot*73+31]);
            assert_eq!(&pool.record(slot).unwrap()[31..39], &prior[slot][31..39]);
            assert_eq!(&pool.record(slot).unwrap()[39..], &source[slot*73+31..slot*73+73]);
        }
        source[199*73+6] = 5;
        assert_eq!(EffectPool::from_packed_inactive(&source,&prior,[None;200]),Err(EffectPoolError::ActivePackedRecord(199)));
    }
    #[test]
    fn factory_binding_and_replacements_preserve_identity_and_before_image() {
        let mut pool = EffectPool::from_packed_inactive(&vec![0;200*73],&[[0xA5;81];200],[None;200]).unwrap();
        let plan = crate::landing::plan_landing_factory(pool.records(),0,0,0,[20736,13568],6272,30,None).unwrap();
        pool.commit_landing_factory(&plan).unwrap();
        assert_eq!(pool.binding(0).unwrap(),Some(EffectAnimationBinding::EffAnimation49));
        let before = pool.clone();
        assert_eq!(pool.commit_landing_factory(&plan),Err(EffectPoolError::StaleRecord(0)));
        assert_eq!(pool,before);
        let old = *pool.record(0).unwrap();
        let mut changed = old;
        changed[31] ^= 1;
        assert_eq!(pool.replace_record_checked(0,&old,&changed),Err(EffectPoolError::PointerBytesChanged(0)));
        changed = old;
        changed[69] = 6;
        assert_eq!(pool.replace_record_checked(0,&old,&changed),Err(EffectPoolError::IdentityChanged(0)));
        assert_eq!(pool,before);
    }
}
