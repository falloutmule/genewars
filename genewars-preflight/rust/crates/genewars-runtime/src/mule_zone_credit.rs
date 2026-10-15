//! Bounded single-player Krell793C0/792C0 numeric credit and ordered tails.
//! Caller owns7A280 eligibility, creature scan, effect factories and exhaustion.
use crate::blueprint::BlueprintRng;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreditContext { pub frame: u32, pub participating_players: u8, pub selected_player: u8, pub approval_limit: u16 }
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreditError { UnsupportedContext, UnsupportedZone, UnsupportedMule, InvalidRng }
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Continuation {
    ///807CF virtual44: EDX39, EBX2, ECX1; pushed200 then30.
    Audio { creature_slot: u16, code: u8, ebx: u8, ecx: u8, stack: [i32; 2] },
    ///807FA6D500: EDX3C, EBX0, ECX10; stack order at call10,count,-1.
    /// This is a real RNG-consuming effect boundary, not presentation-only.
    Marker { creature_slot: u16, code: u8, ebx: u8, ecx: u8, stack: [i32; 3] },
    /// Execute only AFTER the preceding markers have updated world/RNG.
    ParticleTail(ParticleTail),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticleTail { pub creature_slot: u16, pub owner: u8, pub position: [i16; 2], pub physical_height: i32, pub yaw: i16 }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticlePlan { pub rng_after: BlueprintRng, pub journals: Vec<u16>, pub code: u8, pub owner: u8, pub position: [i16; 2], pub physical_height: i32, pub angles: [u16; 3], pub stack: [i32; 4] }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreditPlan { pub zone_after: [u8; 19], pub player_after: [u8; 1805], pub credits: [i32; 2], pub continuations: Vec<Continuation> }
fn word(bytes: &[u8], at: usize) -> i16 { i16::from_le_bytes(bytes[at..at+2].try_into().unwrap()) }
fn dword(bytes: &[u8], at: usize) -> i32 { i32::from_le_bytes(bytes[at..at+4].try_into().unwrap()) }
fn put(bytes: &mut [u8], at: usize, value: i32) { bytes[at..at+4].copy_from_slice(&value.to_le_bytes()); }
/// One Mule in source793C0 slot order. Only active single-player Krell's
/// owner0/neutral4 distribution is admitted; multiplayer penalties are separate.
pub fn plan_one_mule(zone: &[u8;19], player: &[u8;1805], mule: &[u8;124], context: CreditContext) -> Result<CreditPlan, CreditError> {
    if context.participating_players != 1 || context.selected_player != 0 { return Err(CreditError::UnsupportedContext); }
    let radius=dword(zone,6);
    if zone[4]!=12 || zone[18]as i8<=0 || !(0..=32767).contains(&radius) || dword(zone,10)<=0 || dword(zone,14)<0 { return Err(CreditError::UnsupportedZone); }
    if mule[6]!=1 || mule[0x7a]!=4 || !matches!(mule[24],0|4) || !(0..300).contains(&word(mule,15)) { return Err(CreditError::UnsupportedMule); }
    let mut plan=CreditPlan{zone_after:*zone,player_after:*player,credits:[0;2],continuations:Vec::new()};
    if context.frame&63!=0 || mule[0x5e]>=24 || mule[0x5e]==15 { return Ok(plan); }
    let delta=|at, z|i32::from(word(mule,at).wrapping_mul(2).wrapping_sub(word(zone,z).wrapping_mul(2))>>1);
    let (dx,dy)=(delta(7,0),delta(9,2));
    if dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy))as u32>(radius*radius)as u32 { return Ok(plan); }
    let slot=word(mule,15)as u16;
    for (index,weight) in [16i32,8].into_iter().enumerate() {
        let capped=weight.min(dword(&plan.zone_after,14));
        if capped<=0 { continue; }
        let credit=if mule[24]==4 { capped>>2 } else { capped };
        plan.credits[index]=credit;
        let remaining=dword(&plan.zone_after,14).wrapping_sub(credit);put(&mut plan.zone_after,14,remaining);
        let score=dword(&plan.player_after,13).wrapping_add(credit);put(&mut plan.player_after,13,score);
        let approval=dword(&plan.player_after,17).wrapping_add(credit).min(i32::from(context.approval_limit)*2);put(&mut plan.player_after,17,approval);
        // Even neutral credit0 from a positive capped amount invokes80780.
        plan.continuations.push(Continuation::Audio{creature_slot:slot,code:0x39,ebx:2,ecx:1,stack:[30,200]});
        plan.continuations.push(Continuation::Marker{creature_slot:slot,code:0x3c,ebx:0,ecx:10,stack:[10,(credit>>3).max(1)&255,-1]});
    }
    //79506 tests only the AL returned by the SECONDARY792C0 call.
    if plan.credits[1]>0 { plan.continuations.push(Continuation::ParticleTail(ParticleTail{creature_slot:slot,owner:mule[24],position:[word(mule,7),word(mule,9)],physical_height:dword(mule,39),yaw:word(mule,0x39)})); }
    Ok(plan)
}
///7950A..796CE after all score-marker continuations have executed. This
/// consumes only the explicit zone-tail draws; factory effects remain external.
pub fn plan_particle_tail(tail: &ParticleTail, mut rng: BlueprintRng) -> Result<ParticlePlan, CreditError> {
    if rng.ring>=32 || !matches!(tail.owner,0|4) || tail.creature_slot>=300 { return Err(CreditError::InvalidRng); }
    let mut journals=Vec::new();
    let mut draw=|journal| { journals.push(journal);rng.seed=rng.seed.wrapping_mul(0xe62d);rng.ring=rng.ring.wrapping_add(1)&31;rng.draw_count=rng.draw_count.wrapping_add(1);rng.seed>>8 };
    let yaw=tail.yaw.wrapping_add(1920).wrapping_add((draw(0xcf2)%256)as i16)as u16&2047;
    let pitch=draw(0xcf3)%128;
    let owner=if tail.owner==4 {(draw(0xcf5)%4)as u8} else {tail.owner};
    Ok(ParticlePlan{rng_after:rng,journals,code:0x2f,owner,position:tail.position,physical_height:tail.physical_height,angles:[yaw,pitch,512],stack:[0,i32::from(owner),8,0]})
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(owner:u8,quota:i32)->([u8;19],[u8;1805],[u8;124],CreditContext) {
        let mut zone=[0;19];zone[4]=12;zone[18]=1;put(&mut zone,6,1024);put(&mut zone,10,1024);put(&mut zone,14,quota);
        let mut mule=[0;124];mule[6]=1;mule[24]=owner;mule[0x7a]=4;mule[0x5e]=8;
        (zone,[0;1805],mule,CreditContext{frame:64,participating_players:1,selected_player:0,approval_limit:10})
    }
    #[test] fn sequential_caps_and_secondary_return_control_particle_tail() {
        let (zone,player,mule,context)=fixture(0,17);let plan=plan_one_mule(&zone,&player,&mule,context).unwrap();
        assert_eq!(plan.credits,[16,1]);assert_eq!(dword(&plan.zone_after,14),0);assert_eq!(dword(&plan.player_after,13),17);
        assert!(matches!(plan.continuations[0],Continuation::Audio{code:0x39,ebx:2,ecx:1,stack:[30,200],..}));
        assert!(matches!(plan.continuations.last(),Some(Continuation::ParticleTail(_))));
        let (zone,player,mule,context)=fixture(0,1);let plan=plan_one_mule(&zone,&player,&mule,context).unwrap();
        assert_eq!(plan.credits,[1,0]);assert_eq!(plan.continuations.len(),2);
    }
    #[test] fn neutral_rounding_keeps_zero_credit_markers_and_caps_approval() {
        let (zone,mut player,mule,context)=fixture(4,4);put(&mut player,17,20);
        let plan=plan_one_mule(&zone,&player,&mule,context).unwrap();
        assert_eq!(plan.credits,[1,0]);assert_eq!(dword(&plan.zone_after,14),3);assert_eq!(dword(&plan.player_after,17),20);
        assert_eq!(plan.continuations.len(),4);assert!(matches!(plan.continuations[3],Continuation::Marker{stack:[10,1,-1],..}));
    }
    #[test] fn unsupported_distribution_does_not_change_inputs() {
        let (zone,player,mule,mut context)=fixture(0,1024);let before=(zone,player,mule);context.participating_players=2;
        assert_eq!(plan_one_mule(&zone,&player,&mule,context),Err(CreditError::UnsupportedContext));assert_eq!((zone,player,mule),before);
        let tail=ParticleTail{creature_slot:0,owner:0,position:[0;2],physical_height:0,yaw:0};
        assert_eq!(plan_particle_tail(&tail,BlueprintRng{seed:1,ring:32,draw_count:0}),Err(CreditError::InvalidRng));
    }
}
