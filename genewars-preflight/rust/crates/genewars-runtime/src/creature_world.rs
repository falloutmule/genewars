//! Native family0/kind1 storage for the exact admitted first-mission source.
//!
//! This is the 5A850 copy/resource-binding boundary, not a complete AI tick.
//! It owns no map. Native pointers are logical handles and are never invoked.

use genewars_assets::ConqMissionSource;

pub const CREATURE_CAPACITY: usize = 300;
pub const NATIVE_CREATURE_BYTES: usize = 124;
pub const PACKED_CREATURE_BYTES: usize = 116;
pub type CreatureRecord = [u8; NATIVE_CREATURE_BYTES];
const MISSION_SHA256: &str = "72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreatureAnimationBinding {
    /// 5A1C0 selector4 -> creature loader slot4; exact CD MULE.ANB.
    MuleSelector4,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureError {
    MissionSource(genewars_assets::MissionSourceError),
    MissionHash,
    PackedLength(usize),
    UnsupportedActiveRecord(usize),
    InvalidIdentity(usize),
    InvalidSlot(usize),
    StaleRecord(usize),
    IdentityChanged(usize),
    PointerBytesChanged(usize),
}
impl core::fmt::Display for CreatureError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f,"{self:?}") }
}
impl std::error::Error for CreatureError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreaturePool {
    records: [CreatureRecord; CREATURE_CAPACITY],
    bindings: [Option<CreatureAnimationBinding>; CREATURE_CAPACITY],
}
impl CreaturePool {
    /// The caller supplies already-constructed native priors. The serialized
    /// reader leaves [31,39) untouched except for the active 5A1C0 animation
    /// pointer assignment, represented here by a zero placeholder plus handle.
    /// The vtable bytes [35,39) remain opaque; the known Kind1 class is enforced
    /// by this type rather than by a forged process address.
    pub fn from_mission_source(source: &ConqMissionSource<'_>, prior: &[CreatureRecord; CREATURE_CAPACITY]) -> Result<Self,CreatureError> {
        if crate::identity::sha256_hex(source.bytes()) != MISSION_SHA256 { return Err(CreatureError::MissionHash); }
        let group=source.group(0).map_err(CreatureError::MissionSource)?;
        Self::from_packed(group.bytes(),prior)
    }
    /// Exact loader copy plus the explicitly admitted Mule binding. Other
    /// active resource selectors and the state>=26 Bones route reject.
    pub fn from_packed(packed: &[u8], prior: &[CreatureRecord; CREATURE_CAPACITY]) -> Result<Self,CreatureError> {
        if packed.len()!=CREATURE_CAPACITY*PACKED_CREATURE_BYTES { return Err(CreatureError::PackedLength(packed.len())); }
        let mut records=*prior;
        let mut bindings=[None;CREATURE_CAPACITY];
        for (i,source) in packed.chunks_exact(PACKED_CREATURE_BYTES).enumerate() {
            records[i][..31].copy_from_slice(&source[..31]);
            records[i][39..].copy_from_slice(&source[31..]);
            if records[i][6]!=0 {
                if records[i][6]!=1 || records[i][0x5e]>=26 || records[i][0x7a] as i8 !=4 { return Err(CreatureError::UnsupportedActiveRecord(i)); }
                if u16::from_le_bytes([records[i][15],records[i][16]]) as usize !=i || records[i][24]>4 { return Err(CreatureError::InvalidIdentity(i)); }
                records[i][31..35].fill(0);
                bindings[i]=Some(CreatureAnimationBinding::MuleSelector4);
            }
        }
        Ok(Self{records,bindings})
    }
    pub fn records(&self)->&[CreatureRecord;CREATURE_CAPACITY] { &self.records }
    pub fn record(&self,slot:usize)->Result<&CreatureRecord,CreatureError> { self.records.get(slot).ok_or(CreatureError::InvalidSlot(slot)) }
    pub fn binding(&self,slot:usize)->Result<Option<CreatureAnimationBinding>,CreatureError> { self.bindings.get(slot).copied().ok_or(CreatureError::InvalidSlot(slot)) }
    /// Transaction boundary used by the world adapter for source-proved
    /// native-field/link updates. No packed mirror or second map is written.
    pub(crate) fn replace_record_checked(&mut self,slot:usize,before:&CreatureRecord,after:&CreatureRecord)->Result<(),CreatureError> {
        if self.record(slot)?!=before { return Err(CreatureError::StaleRecord(slot)); }
        if before[6]!=after[6] || before[15..17]!=after[15..17] || before[24]!=after[24] || before[0x7a]!=after[0x7a] { return Err(CreatureError::IdentityChanged(slot)); }
        if before[31..39]!=after[31..39] { return Err(CreatureError::PointerBytesChanged(slot)); }
        self.records[slot]=*after;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn split_copy_preserves_runtime_gap_and_binds_active_record() {
        let prior=core::array::from_fn(|slot|core::array::from_fn(|i|(slot*37+i*19)as u8));
        let mut raw=vec![0;CREATURE_CAPACITY*PACKED_CREATURE_BYTES];
        raw[6]=1;raw[24]=4;raw[0x7a-8]=4;raw[0x5e-8]=8;
        let pool=CreaturePool::from_packed(&raw,&prior).unwrap();
        assert_eq!(&pool.records[0][31..35],&[0;4]);assert_eq!(&pool.records[0][35..39],&prior[0][35..39]);
        assert_eq!(&pool.records[1][31..39],&prior[1][31..39]);assert_eq!(pool.binding(0).unwrap(),Some(CreatureAnimationBinding::MuleSelector4));
        assert_eq!(pool.binding(1).unwrap(),None);
    }
    #[test]
    fn stale_and_pointer_changes_reject_without_mutation() {
        let prior=[[0;NATIVE_CREATURE_BYTES];CREATURE_CAPACITY];let mut pool=CreaturePool::from_packed(&vec![0;CREATURE_CAPACITY*PACKED_CREATURE_BYTES],&prior).unwrap();let before=*pool.record(0).unwrap();let snapshot=pool.clone();let mut after=before;after[35]=1;
        assert!(pool.replace_record_checked(0,&before,&after).is_err());assert_eq!(pool,snapshot);
        after=before;after[0]=9;pool.replace_record_checked(0,&before,&after).unwrap();assert!(pool.replace_record_checked(0,&before,&after).is_err());
    }
}
