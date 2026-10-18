//! Source10090 inactive family8 loader and typed native record ownership.
//! No world scheduler, renderer or startup policy is chosen here.
use crate::family8_creation::{CreationPlan, Family8Record};

pub const CAPACITY: usize = 16;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationBinding { Base, Offset80 }
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PoolError {
    PackedLength(usize), ActivePackedRecord(usize), InvalidSlot(usize),
    StaleRecord(usize), OccupiedSlot(usize), StaleBinding(usize),
    InvalidFactoryPlan, IdentityChanged(usize), PointerBytesChanged(usize),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Family8Pool {
    pub(crate) records: [Family8Record; CAPACITY],
    pub(crate) bindings: [Option<AnimationBinding>; CAPACITY],
}
impl Family8Pool {
    pub fn from_packed_inactive(packed: &[u8], prior: &[Family8Record; CAPACITY],
        bindings: [Option<AnimationBinding>; CAPACITY]) -> Result<Self, PoolError> {
        if packed.len() != CAPACITY * 94 { return Err(PoolError::PackedLength(packed.len())); }
        let mut records = *prior;
        for (slot, source) in packed.chunks_exact(94).enumerate() {
            if source[6] != 0 { return Err(PoolError::ActivePackedRecord(slot)); }
            records[slot][..31].copy_from_slice(&source[..31]);
            records[slot][39..].copy_from_slice(&source[31..]);
        }
        Ok(Self { records, bindings })
    }
    pub(crate) fn retire_checked(&mut self,slot:usize,before:&Family8Record)->Result<(),PoolError>{
        if self.record(slot)?!=before{return Err(PoolError::StaleRecord(slot));}
        if before[6]!=8||before[0x55]>1{return Err(PoolError::InvalidFactoryPlan);}
        let binding=if before[0x55]==1{AnimationBinding::Base}else{AnimationBinding::Offset80};
        if self.binding(slot)?!=Some(binding){return Err(PoolError::StaleBinding(slot));}
        self.records[slot][6]=0;self.bindings[slot]=None;Ok(())
    }
    pub fn records(&self) -> &[Family8Record; CAPACITY] { &self.records }
    pub fn record(&self, slot: usize) -> Result<&Family8Record, PoolError> {
        self.records.get(slot).ok_or(PoolError::InvalidSlot(slot))
    }
    pub fn binding(&self, slot: usize) -> Result<Option<AnimationBinding>, PoolError> {
        self.bindings.get(slot).copied().ok_or(PoolError::InvalidSlot(slot))
    }
    pub(crate) fn commit_creation(&mut self, slot: usize, before: &Family8Record,
        plan: &CreationPlan) -> Result<(), PoolError> {
        if self.record(slot)? != before { return Err(PoolError::StaleRecord(slot)); }
        if before[6] != 0 { return Err(PoolError::OccupiedSlot(slot)); }
        if self.binding(slot)?.is_some() { return Err(PoolError::StaleBinding(slot)); }
        let after = &plan.after;
        let binding = match (after[0x55], plan.animation_table_offset) {
            (1, 0) => AnimationBinding::Base,
            (0 | 2, 0x50) => AnimationBinding::Offset80,
            _ => return Err(PoolError::InvalidFactoryPlan),
        };
        if self.records.iter().position(|r| r[6] == 0) != Some(slot)
            || after[6] != 8 || u16::from_le_bytes([after[15], after[16]]) as usize != slot
            || after[24] > 4 || (after[0x55] == 2 && after[24] != 4)
            || plan.cell_head_after.index as usize != slot || plan.cell_head_after.kind != 8 {
            return Err(PoolError::InvalidFactoryPlan);
        }
        if after[31..39] != before[31..39] { return Err(PoolError::PointerBytesChanged(slot)); }
        self.records[slot] = *after; self.bindings[slot] = Some(binding); Ok(())
    }
    pub(crate) fn replace_record_checked(&mut self, slot: usize, before: &Family8Record,
        after: &Family8Record) -> Result<(), PoolError> {
        if self.record(slot)? != before { return Err(PoolError::StaleRecord(slot)); }
        if before[6] != after[6] || before[15..17] != after[15..17]
            || before[24] != after[24] || before[0x55] != after[0x55] {
            return Err(PoolError::IdentityChanged(slot));
        }
        if before[31..39] != after[31..39] { return Err(PoolError::PointerBytesChanged(slot)); }
        self.records[slot] = *after; Ok(())
    }
}
