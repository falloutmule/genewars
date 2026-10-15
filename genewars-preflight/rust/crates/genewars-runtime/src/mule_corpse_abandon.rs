//!67780..67821/6789F low-reserve corpse abandonment with false5B100 predicate.
//! Caller proves source state5 distance/target binding and postphysics cell.
//! No feeding, relocation, physics, animation or common timer decrement here.
use crate::{blueprint::BlueprintRng, creature_world::CreatureRecord};
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorpseAbandonError { Identity, Current, Target, References, Timing, Rng, EscapeRequired }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpseAbandonPlan {
    pub current_after: CreatureRecord,
    pub target_after: CreatureRecord,
    pub rng_after: BlueprintRng,
}
/// Caller already selected the source low-reserve branch and proved distance
/// <=4096. Existing corpse/Bones targets24..27 share the low-reserve reset;
/// rich-reserve consumption and true-cell relocation remain unsupported.
/// The live association50 survives even though target55's kind becomes null.
pub fn plan_abandon(current: &CreatureRecord, corpse: &CreatureRecord,
    rng_before: BlueprintRng, current_cell_requires_escape: bool)
    -> Result<CorpseAbandonPlan, CorpseAbandonError>
{
    let word = |r: &CreatureRecord, offset| i16::from_le_bytes([r[offset],r[offset+1]]);
    let id = word(current,15);let target_id = word(corpse,15);
    if !(0..300).contains(&id) || !(0..300).contains(&target_id) || id==target_id {
        return Err(CorpseAbandonError::Identity);
    }
    if current[6]!=1 || current[24]!=4 || current[0x7a]!=4 || current[0x5e]!=5
        || current[0x76]>90 || current[0x78]!=0 || current[0x61]!=0x40
        || current[0x60]&0x22!=0 || word(current,0x6a)<=0 {
        return Err(CorpseAbandonError::Current);
    }
    if corpse[6]!=1 || corpse[24]!=4 || corpse[0x7a]!=4 || !matches!(corpse[0x5e],24..=27)
        || word(corpse,0x62)<=0 || word(corpse,0x68)<0
        || word(corpse,0x68)>word(corpse,0x62)>>1 {
        return Err(CorpseAbandonError::Target);
    }
    if current[0x4f]!=0 || current[0x5d]!=0 || current[0x52]!=1 || current[0x57]!=1
        || word(current,0x50)!=target_id || word(current,0x55)!=target_id {
        return Err(CorpseAbandonError::References);
    }
    if word(current,0x6c)<0 || word(current,0x6c)&15!=0 {return Err(CorpseAbandonError::Timing);}
    if rng_before.ring>=32 {return Err(CorpseAbandonError::Rng);}
    if current_cell_requires_escape {return Err(CorpseAbandonError::EscapeRequired);}
    let mut current_after=*current;let mut target_after=*corpse;
    //5CCEF/5CD14 releases reservation on the kind1 target.
    target_after[0x60]&=0xfe;
    //5CCA0 common local reset; null indices and association50 remain opaque.
    current_after[0x15]&=0xdf;current_after[0x33..0x37].fill(0);
    let flags=(word(current,0x60)as u16)&0xd71a;
    current_after[0x60..0x62].copy_from_slice(&flags.to_le_bytes());
    current_after[0x5d]=0;current_after[0x57]=0;
    let mut rng_after=rng_before;
    rng_after.seed=rng_after.seed.wrapping_mul(0xe62d);
    rng_after.ring=rng_after.ring.wrapping_add(1)&31;
    rng_after.draw_count=rng_after.draw_count.wrapping_add(1);
    let delay=20+i16::from((rng_after.seed>>8)as u8%20);
    current_after[0x5e]=8;current_after[0x6c..0x6e].copy_from_slice(&delay.to_le_bytes());
    Ok(CorpseAbandonPlan { current_after,target_after,rng_after })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture()->(CreatureRecord,CreatureRecord,BlueprintRng) {
        let mut a=[0xa5;124];let mut b=[0x5a;124];
        for(r,id)in[(&mut a,8i16),(&mut b,1i16)]{r[6]=1;r[24]=4;r[0x7a]=4;r[15..17].copy_from_slice(&id.to_le_bytes());}
        a[0x5e]=5;a[0x76]=50;a[0x78]=0;a[0x60]=8;a[0x61]=0x40;
        a[0x6a..0x6c].copy_from_slice(&30i16.to_le_bytes());a[0x6c..0x6e].copy_from_slice(&32752i16.to_le_bytes());
        a[0x4f]=0;a[0x5d]=0;a[0x50..0x53].copy_from_slice(&[1,0,1]);a[0x55..0x58].copy_from_slice(&[1,0,1]);
        b[0x5e]=25;b[0x60]=9;b[0x62..0x64].copy_from_slice(&143i16.to_le_bytes());b[0x68..0x6a].copy_from_slice(&35i16.to_le_bytes());
        (a,b,BlueprintRng {seed:1,ring:31,draw_count:u32::MAX})
    }
    #[test]
    fn abandonment_releases_reservation_preserves_association_and_wraps_one_draw() {
        let(a,b,rng)=fixture();let p=plan_abandon(&a,&b,rng,false).unwrap();
        assert_eq!(p.current_after[0x5e],8);assert_eq!(p.current_after[0x57],0);assert_eq!(p.target_after[0x60],8);
        assert_eq!(&p.current_after[0x50..0x53],&a[0x50..0x53]);assert_eq!(p.current_after[0x61],0x40);
        assert_eq!(p.rng_after,BlueprintRng {seed:0xe62d,ring:0,draw_count:0});
        assert_eq!(i16::from_le_bytes(p.current_after[0x6c..0x6e].try_into().unwrap()),30);
        for i in 0..124 {
            if !matches!(i,0x15|0x33..=0x36|0x57|0x5d|0x5e|0x60|0x61|0x6c|0x6d){assert_eq!(p.current_after[i],a[i],"current{i:x}");}
            if i!=0x60{assert_eq!(p.target_after[i],b[i],"target{i:x}");}
        }
    }
    #[test]
    fn decayed_bones_target_releases_only_reservation_and_keeps_live_association() {
        // Source-parametric3645 callback: target25 becomes26 before the
        // current state5 processing gate. No reacquisition or consumption.
        let(a,mut bones,rng)=fixture();bones[0x5e]=26;bones[0x19]=4;bones[0x1b]=0;
        let p=plan_abandon(&a,&bones,rng,false).unwrap();
        let mut released=bones;released[0x60]&=0xfe;
        assert_eq!(p.target_after,released);
        assert_eq!(p.target_after[0x5e],26);
        assert_eq!(&p.current_after[0x50..0x53],&a[0x50..0x53]);
        assert_eq!(p.current_after[0x57],0);
        assert_eq!(p.current_after[0x5e],8);
        assert_eq!(p.rng_after.draw_count,rng.draw_count.wrapping_add(1));
        // The admitted decay cannot turn a rich Bones target into an
        // abandonable one, or silently admit the escape continuation.
        bones[0x68..0x6a].copy_from_slice(&72i16.to_le_bytes());
        assert_eq!(plan_abandon(&a,&bones,rng,false),Err(CorpseAbandonError::Target));
        bones[0x68..0x6a].copy_from_slice(&35i16.to_le_bytes());
        assert_eq!(plan_abandon(&a,&bones,rng,true),Err(CorpseAbandonError::EscapeRequired));
    }
    #[test]
    fn consumption_live_targets_escape_and_unknown_associations_remain_unsupported() {
        let(a,b,rng)=fixture();assert_eq!(plan_abandon(&a,&b,rng,true),Err(CorpseAbandonError::EscapeRequired));
        let mut rich=b;rich[0x68..0x6a].copy_from_slice(&72i16.to_le_bytes());assert_eq!(plan_abandon(&a,&rich,rng,false),Err(CorpseAbandonError::Target));
        let mut live=b;live[0x5e]=23;assert_eq!(plan_abandon(&a,&live,rng,false),Err(CorpseAbandonError::Target));
        for(off,v)in[(0x52,0),(0x57,2),(0x4f,4),(0x5d,5),(0x6c,1),(0x60,0x28),(0x78,1)]{let mut current=a;current[off]=v;assert!(plan_abandon(&current,&b,rng,false).is_err());}
        assert!(plan_abandon(&a,&b,BlueprintRng {ring:32,..rng},false).is_err());
        let mut half=b;half[0x68..0x6a].copy_from_slice(&71i16.to_le_bytes());assert!(plan_abandon(&a,&half,rng,false).is_ok());
    }
}
