//! Source7A280 scheduler admission before actors (1C760).
use super::*;
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ZoneUpdateError { UnsupportedActiveZoneSchedule }
impl ActorWorld {
    /// Countdown nonzero returns at7A298; an empty list returns at7A2BC.
    /// Active zone scoring/exhaustion is not a no-op and remains explicit.
    pub fn step_admitted_zone_callbacks(&mut self)->Result<(),ZoneUpdateError>{
        if word(&self.backing,0x355c)!=0||self.backing[0x34b8]==0{Ok(())}
        else{Err(ZoneUpdateError::UnsupportedActiveZoneSchedule)}
    }
}

#[cfg(test)] mod tests {
 use super::*;
 #[test] fn countdown_gate_is_unchanged_and_active_zone_is_explicitly_unsupported(){
  let mut raw=vec![0;343958];raw[..4].copy_from_slice(&0x55336u32.to_le_bytes());raw[337054..337058].copy_from_slice(&0x6beu32.to_le_bytes());
  let source=ConqMissionSource::parse(&raw).unwrap();
  let mut world=ActorWorld::from_parts(&source,&[[0;200];20],[ActorHandleBindings::empty();20],core::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,[4096,4096],b"fixture\0").unwrap()),BlueprintRng{seed:0,ring:31,draw_count:0}).unwrap();
  world.backing[0x34b8]=1;
  for countdown in [1u16,1000,65535]{world.backing[0x355c..0x355e].copy_from_slice(&countdown.to_le_bytes());let before=world.clone();assert_eq!(world.step_admitted_zone_callbacks(),Ok(()));assert_eq!(world,before);}
  world.backing[0x355c..0x355e].fill(0);let before=world.clone();assert_eq!(world.step_admitted_zone_callbacks(),Err(ZoneUpdateError::UnsupportedActiveZoneSchedule));assert_eq!(world,before);
  world.backing[0x34b8]=0;let before=world.clone();assert_eq!(world.step_admitted_zone_callbacks(),Ok(()));assert_eq!(world,before);
 }
}
