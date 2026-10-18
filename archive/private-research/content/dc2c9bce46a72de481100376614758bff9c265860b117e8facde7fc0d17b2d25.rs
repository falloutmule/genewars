//! Owned 25-slot terrain-job store; inactive signed counters preserve opaque bytes.
use crate::construction_terrain::{TerrainJob, TerrainJobAllocation};
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerrainJobPoolError { CountMismatch, InvalidSlot, StaleRecord, InvalidTransition }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainJobPool { records: [TerrainJob; 25], count: u8 }
fn active(record: &TerrainJob) -> bool {
    i16::from_le_bytes(record[12..14].try_into().unwrap()) >= 0
}
#[cfg(test)]
mod tests {
    use super::*;
    fn empty() -> [TerrainJob; 25] { std::array::from_fn(|_| { let mut r=[0xa5;176];r[12..14].copy_from_slice(&(-1i16).to_le_bytes());r }) }
    #[test]
    fn allocation_retirement_preserves_opaque_record_and_rejects_stale_commit() {
        let mut pool=TerrainJobPool::admit(empty(),0).unwrap();
        let before=pool.records[3];let mut after=before;after[12..14].copy_from_slice(&2i16.to_le_bytes());
        let plan=TerrainJobAllocation{slot:3,before,after,count_after:1};pool.apply_allocation(&plan).unwrap();
        let snapshot=pool.clone();assert_eq!(pool.apply_allocation(&plan),Err(TerrainJobPoolError::StaleRecord));assert_eq!(pool,snapshot);
        let mut stepped=after;stepped[12..14].copy_from_slice(&0i16.to_le_bytes());pool.apply_step(3,&after,&stepped,true).unwrap();
        assert_eq!(pool.count(),0);assert_eq!(pool.records[3],before);
    }
    #[test]
    fn inconsistent_count_is_rejected() { assert_eq!(TerrainJobPool::admit(empty(),1),Err(TerrainJobPoolError::CountMismatch)); }
}
impl TerrainJobPool {
    pub fn admit(records: [TerrainJob; 25], count: u8) -> Result<Self, TerrainJobPoolError> {
        if usize::from(count) != records.iter().filter(|r| active(r)).count() {
            return Err(TerrainJobPoolError::CountMismatch);
        }
        Ok(Self { records, count })
    }
    pub fn records(&self) -> &[TerrainJob; 25] { &self.records }
    pub fn count(&self) -> u8 { self.count }
    pub fn apply_allocation(&mut self, plan: &TerrainJobAllocation) -> Result<(), TerrainJobPoolError> {
        let slot = usize::from(plan.slot);
        let prior = self.records.get(slot).ok_or(TerrainJobPoolError::InvalidSlot)?;
        if prior != &plan.before { return Err(TerrainJobPoolError::StaleRecord); }
        if active(prior) || !active(&plan.after) || plan.count_after != self.count + 1 {
            return Err(TerrainJobPoolError::InvalidTransition);
        }
        self.records[slot] = plan.after;
        self.count = plan.count_after;
        Ok(())
    }
    /// Apply a live 78E30 record; 79220 retirement changes counter to -1 and decrements count.
    pub fn apply_step(&mut self, slot: u8, before: &TerrainJob, after: &TerrainJob, retire: bool) -> Result<(), TerrainJobPoolError> {
        let prior = self.records.get(usize::from(slot)).ok_or(TerrainJobPoolError::InvalidSlot)?;
        if prior != before { return Err(TerrainJobPoolError::StaleRecord); }
        if !active(before) || (!retire && !active(after)) || (retire && self.count == 0) {
            return Err(TerrainJobPoolError::InvalidTransition);
        }
        let mut committed = *after;
        if retire { committed[12..14].copy_from_slice(&(-1i16).to_le_bytes()); }
        self.records[usize::from(slot)] = committed;
        if retire { self.count -= 1; }
        Ok(())
    }
}
