//! Source 74A50 numeric backward target search. Environmental predicates are
//! supplied by a separately admitted caller; no point is assumed walkable.
use crate::{actor_activation::TrigTables,physics_tables::PhysicsTables};

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum TargetResolverError<E> { Query(E), IterationLimit }

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct TargetResolution { pub point:[i16;2], pub queried_points:u16 }

/// Walk in source fixed-point steps from requested toward current. Distance
/// uses the signed15 torus and the radius64 terminal snap. The terminal point
/// is still queried, then returned even if rejected, matching 74C73..74CA1.
/// A finite guard rejects if the original search has not terminated; this
/// avoids inventing a point when repeated environment rejection would loop.
pub fn resolve_target<E,F>(current:[i16;2],requested:[i16;2],physics:&PhysicsTables,trig:&TrigTables,mut accepts:F)
    ->Result<TargetResolution,TargetResolverError<E>>
where F:FnMut([i16;2])->Result<bool,E> {
    let doubled=core::array::from_fn::<_,2,_>(|i|current[i].wrapping_sub(requested[i]).wrapping_mul(2));
    let angle=crate::actor_motion::angle(i32::from(doubled[0]),i32::from(doubled[1]),physics.angles());
    let direction=usize::from(angle.wrapping_add(1536)&2047);
    let delta=[trig.cosine()[direction].wrapping_shl(6),trig.sine()[direction].wrapping_shl(6)];
    let mut fixed=requested.map(|v|i32::from(v).wrapping_shl(16));
    for iteration in 1..=4096u16 {
        let mut point=fixed.map(|v|(v>>16)as i16);
        let distance=genewars_core::cd_candidate::distance_squared_wrapping_15(current,point);
        if distance<=4096 {point=current;}
        let accepted=accepts(point).map_err(TargetResolverError::Query)?;
        if accepted||point==current {return Ok(TargetResolution{point,queried_points:iteration});}
        for i in 0..2 {fixed[i]=fixed[i].wrapping_add(delta[i]);}
    }
    Err(TargetResolverError::IterationLimit)
}
