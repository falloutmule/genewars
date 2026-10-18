//! Source805E0's read-only construction-capability predicate, called by72ED0.
//! This does not evaluate the enclosing mission win/loss rules or transitions.
use crate::player_state::PLAYER_RECORD_BYTES;

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum MissionStatusError { ActorCount(i8), ActorId(i8), MissingBlueprint(i16) }

/// True is source805E0's AL=1 result. The meanings of numeric player fields
///616,600 and605 remain expressed by their offsets; they are not guessed
/// money, power or inventory defaults. Category0 is the admitted architect.
/// Actor blueprint lookup is lazy and follows the live signed-byte list.
pub fn construction_capability_exhausted(player:&[u8;PLAYER_RECORD_BYTES],
    mut actor_category:impl FnMut(i16)->Option<u8>)->Result<bool,MissionStatusError> {
    if i32::from_le_bytes(player[0x616..0x61a].try_into().unwrap())>0||player[0x600]!=0 {
        return Ok(false);
    }
    let count=player[3]as i8;
    if count>5{return Err(MissionStatusError::ActorCount(count));}
    for index in 0..count.max(0)as usize {
        let id=player[4+index]as i8;
        if !(0..20).contains(&id){return Err(MissionStatusError::ActorId(id));}
        if actor_category(i16::from(id)).ok_or(MissionStatusError::MissingBlueprint(i16::from(id)))?==0 {
            return Ok(false);
        }
    }
    if player[0x605]!=0 {
        for blueprint in player[0x16..0x496].chunks_exact(36) {
            if blueprint[0]!=0&&blueprint[0x16]==0{return Ok(false);}
        }
    }
    Ok(true)
}

#[cfg(test)]mod tests {
    use super::*;
    #[test]fn early_capability_does_not_resolve_unused_actor_handles(){
        let mut p=[0;PLAYER_RECORD_BYTES];p[3]=127;
        p[0x616..0x61a].copy_from_slice(&1i32.to_le_bytes());
        assert_eq!(construction_capability_exhausted(&p,|_|panic!("unqueried handle")),Ok(false));
        p[0x616..0x61a].copy_from_slice(&(-1i32).to_le_bytes());p[0x600]=1;
        assert_eq!(construction_capability_exhausted(&p,|_|None),Ok(false));
        p[0x600]=0;assert_eq!(construction_capability_exhausted(&p,|_|None),Err(MissionStatusError::ActorCount(127)));
    }
    #[test]fn actor_list_and_last_stored_blueprint_keep_source_short_circuit_order(){
        let mut p=[0;PLAYER_RECORD_BYTES];p[3]=2;p[4]=4;p[5]=255;
        let mut visits=Vec::new();
        assert_eq!(construction_capability_exhausted(&p,|id|{visits.push(id);Some(0)}),Ok(false));
        assert_eq!(visits,[4]);
        p[3]=255;p[0x16+31*36]=1;
        assert_eq!(construction_capability_exhausted(&p,|_|panic!("negative count")),Ok(true));
        p[0x605]=1;assert_eq!(construction_capability_exhausted(&p,|_|None),Ok(false));
        p[0x16+31*36+0x16]=1;assert_eq!(construction_capability_exhausted(&p,|_|None),Ok(true));
    }
}
