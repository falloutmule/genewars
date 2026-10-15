//! Bounded65AEF..65B44 corpse arrival, before the common callback tail.
//! Caller proves target's MuleSelector4 resource binding and current common
//! physics/arrival result. No feeding, RNG, physics, animation or retirement.
use crate::creature_world::CreatureRecord;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorpseArrivalError { Identity, Current, Target, References, Arrival }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpseArrivalPlan {
    pub current_after: CreatureRecord,
    pub target_after: CreatureRecord,
}
/// State1's kind1 target branch only. Bones state>=26 takes a different state4
/// branch and is deliberately excluded. Hunger HIGH61 bit40 is preserved;
/// source65B44 clears LOW60 bit40 (the approach flag), not hunger.
pub fn plan_arrival(current: &CreatureRecord, corpse: &CreatureRecord)
    -> Result<CorpseArrivalPlan, CorpseArrivalError>
{
    let word = |r: &CreatureRecord, offset| i16::from_le_bytes([r[offset], r[offset+1]]);
    let id = word(current, 15); let target_id = word(corpse, 15);
    if !(0..300).contains(&id) || !(0..300).contains(&target_id) || id == target_id {
        return Err(CorpseArrivalError::Identity);
    }
    if current[6] != 1 || current[24] != 4 || current[0x7a] != 4
        || current[0x5e] != 1 || current[0x76] > 90 || current[0x78] != 0
        || word(current, 0x6a) <= 0 || word(current, 0x6c) < 0
        || current[0x61] != 0x40 || current[0x60] & 2 != 0 {
        return Err(CorpseArrivalError::Current);
    }
    if corpse[6] != 1 || corpse[24] != 4 || corpse[0x7a] != 4
        || !matches!(corpse[0x5e],24|25) || word(corpse,0x49) as u16 & 0x400 != 0 {
        return Err(CorpseArrivalError::Target);
    }
    if current[0x4f] != 0 || current[0x5d] != 0
        || current[0x52] != 1 || current[0x57] != 1
        || word(current,0x50) != target_id || word(current,0x55) != target_id {
        return Err(CorpseArrivalError::References);
    }
    if word(current,0x49) as u16 & 0x200 == 0 || current[0x60] & 0x80 != 0 {
        return Err(CorpseArrivalError::Arrival);
    }
    let mut current_after = *current; let mut target_after = *corpse;
    current_after[0x5e] = 5;
    target_after[0x60] |= 1;
    current_after[0x6c..0x6e].copy_from_slice(&32767i16.to_le_bytes());
    current_after[0x72..0x76].copy_from_slice(&current[7..11]);
    current_after[0x60] = (current[0x60] | 8) & 0xbf;
    Ok(CorpseArrivalPlan { current_after, target_after })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (CreatureRecord,CreatureRecord) {
        let mut a=[0xa5;124];let mut b=[0x5a;124];
        for (r,id) in [(&mut a,8i16),(&mut b,1i16)] {
            r[6]=1;r[24]=4;r[0x7a]=4;r[15..17].copy_from_slice(&id.to_le_bytes());
            r[0x49..0x4b].copy_from_slice(&0x200u16.to_le_bytes());
        }
        a[0x5e]=1;a[0x76]=50;a[0x78]=0;a[0x61]=0x40;a[0x60]=0x48;
        a[0x6a..0x6c].copy_from_slice(&24i16.to_le_bytes());a[0x6c..0x6e].copy_from_slice(&20i16.to_le_bytes());
        a[0x4f]=0;a[0x5d]=0;a[0x50..0x53].copy_from_slice(&[1,0,1]);a[0x55..0x58].copy_from_slice(&[1,0,1]);
        a[7..11].copy_from_slice(&[0x7f,0x7c,0xe5,0x53]);b[0x5e]=25;b[0x60]=8;
        (a,b)
    }
    #[test]
    fn arrival_reserves_target_and_preserves_hunger_goal_association_and_pointers() {
        let (a,b)=fixture();let p=plan_arrival(&a,&b).unwrap();
        assert_eq!(p.current_after[0x5e],5);assert_eq!(p.current_after[0x60],8);
        assert_eq!(p.current_after[0x61],0x40);assert_eq!(p.target_after[0x60],9);
        assert_eq!(&p.current_after[0x72..0x76],&a[7..11]);
        for i in 0..124 {
            if !matches!(i,0x5e|0x60|0x6c|0x6d|0x72..=0x75) {assert_eq!(p.current_after[i],a[i],"current offset{i:x}");}
            if i!=0x60 {assert_eq!(p.target_after[i],b[i],"target offset{i:x}");}
        }
        let mut reserved=b;reserved[0x60]|=1;assert_eq!(plan_arrival(&a,&reserved).unwrap().target_after,reserved);
    }
    #[test]
    fn bones_nonarrival_and_external_associations_reject_without_mutation() {
        let (a,b)=fixture();
        for (off,v) in [(0x5e,26),(24,0),(0x7a,5)] {
            let mut target=b;target[off]=v;assert!(plan_arrival(&a,&target).is_err());assert_eq!(a,fixture().0);
        }
        for (off,v) in [(0x52,0),(0x57,2),(0x4f,4),(0x5d,5),(0x60,0xc8),(0x4a,0),(0x78,1),(0x76,91)] {
            let mut current=a;current[off]=v;assert!(plan_arrival(&current,&b).is_err());
        }
    }
}
