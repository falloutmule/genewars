//! Source7A280 scheduler admission before actors (1C760).
use super::*;
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ZoneUpdateError { UnsupportedActiveZoneSchedule }
impl ActorWorld {
    /// Countdown nonzero returns at7A298; an empty list returns at7A2BC.
    /// Active zone scoring/exhaustion is not a no-op and remains explicit.
    pub fn step_admitted_zone_callbacks(&mut self,source_frame:u32)->Result<(),ZoneUpdateError>{
        if word(&self.backing,0x355c)!=0||self.backing[0x34b8]==0{return Ok(());}
        let unsupported=ZoneUpdateError::UnsupportedActiveZoneSchedule;
        let count=usize::from(self.backing[0x34b8]);if count>4{return Err(unsupported);}
        let pool=self.creatures.as_ref().ok_or(unsupported)?;
        let active=pool.records().iter().filter(|record|record[6]!=0).count();
        if word(&self.backing,0x3550)<0||word(&self.backing,0x3550)as usize!=active{return Err(unsupported);}
        for record in pool.records().iter().filter(|record|record[6]!=0){
            if record[6]!=1||record[0x7a]!=4||record[0x18]>4{return Err(unsupported);}
        }
        for index in 0..count{
            let zone=&self.backing[0x2ff8+index*19..0x2ff8+(index+1)*19];
            let signed_dword=|at|i32::from_le_bytes(zone[at..at+4].try_into().unwrap());
            if zone[4]!=12||signed_dword(10)<=0||signed_dword(14)<=0{return Err(unsupported);}
            let radius=signed_dword(6);if !(0..=32767).contains(&radius){return Err(unsupported);}
            if zone[18]as i8<=0{continue;}
            //793CF does not add creature identity to the frame gate.
            if source_frame&63!=0{continue;}
            for record in pool.records().iter().filter(|record|record[6]!=0){
                //Type1 virtual68 and the separate frozen-state exclusion.
                if record[0x5e]>=24||record[0x5e]==15{continue;}
                let delta=|at|i32::from(word(record,at).wrapping_mul(2).wrapping_sub(word(zone,at-7).wrapping_mul(2))>>1);
                let (dx,dy)=(delta(7),delta(9));
                let distance=dx*dx+dy*dy;
                //Selector4 maps to category4: either match enters792C0.
                //Scoring, quota exhaustion, effects and RNG remain unsupported.
                if (distance as u32)<=(radius*radius)as u32{return Err(unsupported);}
            }
        }
        Ok(())
    }
}

#[cfg(test)] mod tests {
 use super::*;
 fn active_fixture()->ActorWorld{
  let mut raw=vec![0;343958];raw[..4].copy_from_slice(&0x55336u32.to_le_bytes());raw[337054..337058].copy_from_slice(&0x6beu32.to_le_bytes());
  raw[0x34b8]=1;raw[0x2ffc]=12;raw[0x300a]=1;
  for offset in [0x2ffe,0x3002,0x3006]{raw[offset..offset+4].copy_from_slice(&1024i32.to_le_bytes());}
  let mut world=ActorWorld::from_parts(&ConqMissionSource::parse(&raw).unwrap(),&[[0;200];20],[ActorHandleBindings::empty();20],core::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,[4096,4096],b"fixture\0").unwrap()),BlueprintRng{seed:47117,ring:17,draw_count:33620}).unwrap();
  let mut packed=vec![0;300*116];
  for slot in 0..4{let r=&mut packed[slot*116..(slot+1)*116];r[6]=1;r[15..17].copy_from_slice(&(slot as i16).to_le_bytes());r[7..9].copy_from_slice(&8192i16.to_le_bytes());r[9..11].copy_from_slice(&8192i16.to_le_bytes());r[24]=4;r[0x7a-8]=4;r[0x5e-8]=8;}
  world.creatures=Some(Box::new(CreaturePool::from_packed(&packed,&[[0;124];300]).unwrap()));world.backing[0x3550..0x3552].copy_from_slice(&4i16.to_le_bytes());world
 }
 #[test]fn active_outside_and_non_scan_frames_preserve_world(){
  let mut world=active_fixture();for frame in [0,63,64,256,1000,1024]{let before=world.clone();assert_eq!(world.step_admitted_zone_callbacks(frame),Ok(()));assert_eq!(world,before);}
 }
 #[test]fn wrapped_boundary_and_invalid_metadata_reject_atomically(){
  let base=active_fixture();let mut world=base.clone();let old=*world.creatures.as_ref().unwrap().record(0).unwrap();let mut near=old;near[7..9].copy_from_slice(&32767i16.to_le_bytes());near[9..11].copy_from_slice(&0i16.to_le_bytes());world.creatures.as_mut().unwrap().replace_record_checked(0,&old,&near).unwrap();let before=world.clone();assert_eq!(world.step_admitted_zone_callbacks(1024),Err(ZoneUpdateError::UnsupportedActiveZoneSchedule));assert_eq!(world,before);assert_eq!(world.step_admitted_zone_callbacks(1000),Ok(()));assert_eq!(world,before);
  for offset in [0x3550,0x2ffc,0x3003,0x3007]{let mut world=base.clone();world.backing[offset]=0;let before=world.clone();assert_eq!(world.step_admitted_zone_callbacks(1024),Err(ZoneUpdateError::UnsupportedActiveZoneSchedule));assert_eq!(world,before);}
 }
 #[test] fn countdown_gate_is_unchanged_and_active_zone_is_explicitly_unsupported(){
  let mut raw=vec![0;343958];raw[..4].copy_from_slice(&0x55336u32.to_le_bytes());raw[337054..337058].copy_from_slice(&0x6beu32.to_le_bytes());
  let source=ConqMissionSource::parse(&raw).unwrap();
  let mut world=ActorWorld::from_parts(&source,&[[0;200];20],[ActorHandleBindings::empty();20],core::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,[4096,4096],b"fixture\0").unwrap()),BlueprintRng{seed:0,ring:31,draw_count:0}).unwrap();
  world.backing[0x34b8]=1;
  for countdown in [1u16,1000,65535]{world.backing[0x355c..0x355e].copy_from_slice(&countdown.to_le_bytes());let before=world.clone();assert_eq!(world.step_admitted_zone_callbacks(1024),Ok(()));assert_eq!(world,before);}
  world.backing[0x355c..0x355e].fill(0);let before=world.clone();assert_eq!(world.step_admitted_zone_callbacks(1024),Err(ZoneUpdateError::UnsupportedActiveZoneSchedule));assert_eq!(world,before);
  world.backing[0x34b8]=0;let before=world.clone();assert_eq!(world.step_admitted_zone_callbacks(1024),Ok(()));assert_eq!(world,before);
 }
}


/// Bounded live7A280 scheduler. Exhaustion's7A2F3 continuation is separate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KrellZoneScheduleError {
    World(ActorWorldError),
    Credit(super::zone_credit_updates::ZoneCreditError),
    UnsupportedContext,
    UnsupportedZone,
    InvalidCreatureCount,
    InvalidCreature(u16),
    UnsupportedExhaustion,
}
impl From<ActorWorldError> for KrellZoneScheduleError {
    fn from(error: ActorWorldError) -> Self { Self::World(error) }
}
impl From<super::zone_credit_updates::ZoneCreditError> for KrellZoneScheduleError {
    fn from(error: super::zone_credit_updates::ZoneCreditError) -> Self { Self::Credit(error) }
}
impl ActorWorld {
    /// Source7A280 visits zones and then creature slots in ascending order.
    /// One outer transaction also rolls back earlier zones if a later factory,
    /// binding, or unported exhaustion continuation cannot be admitted.
    pub fn step_krell_zone_callbacks(
        &mut self,
        frame: u32,
        players: u8,
        selected: u8,
        context: LandingPhysicsContext,
        trig: &TrigTables,
    ) -> Result<Vec<super::zone_credit_updates::ZoneCreditReceipt>, KrellZoneScheduleError> {
        use crate::creature_world::CreatureAnimationBinding;
        use KrellZoneScheduleError as Error;
        //7A298 and7A2BC return before any callback/context admission.
        let count = usize::from(self.backing[0x34b8]);
        if word(&self.backing, 0x355c) != 0 || count == 0 { return Ok(Vec::new()); }
        if players != 1 || selected != 0 { return Err(Error::UnsupportedContext); }
        if count > 4 { return Err(Error::UnsupportedZone); }
        let linked = self.validate_links()?;
        let pool = self.creatures.as_ref().ok_or(Error::InvalidCreatureCount)?;
        let active = pool.records().iter().filter(|record| record[6] != 0).count();
        if word(&self.backing, 0x3550) < 0 || word(&self.backing, 0x3550) as usize != active {
            return Err(Error::InvalidCreatureCount);
        }
        // Virtual68 is only established here for the actual typed Mule/Bones
        // family. Do not silently skip unknown living species or owners.
        for (slot, record) in pool.records().iter().enumerate().filter(|(_, record)| record[6] != 0) {
            let binding = pool.binding(slot).map_err(|error| Error::World(error.into()))?;
            let expected = if record[0x5e] >= 26 {
                CreatureAnimationBinding::BonesSelector4
            } else { CreatureAnimationBinding::MuleSelector4 };
            if record[6] != 1 || record[0x7a] != 4 || !matches!(record[24], 0 | 4)
                || word(record, 15) != slot as i16 || binding != Some(expected)
                || !linked.contains(&(1, slot as i16))
            { return Err(Error::InvalidCreature(slot as u16)); }
        }
        let mut staged = self.clone();
        let mut receipts = Vec::new();
        for index in 0..count {
            let offset = 0x2ff8 + index * 19;
            let zone: [u8; 19] = staged.backing[offset..offset + 19].try_into().unwrap();
            let signed = |at: usize| i32::from_le_bytes(zone[at..at + 4].try_into().unwrap());
            //7A2C2 ignores exhausted/inactive entries before calling793C0.
            if signed(14) <= 0 { continue; }
            let radius = signed(6);
            if zone[4] != 12 || !(0..=32767).contains(&radius) || signed(10) <= 0 {
                return Err(Error::UnsupportedZone);
            }
            if zone[18] as i8 <= 0 { continue; }
            //793CF's frame gate does not include the creature slot.
            if frame & 63 == 0 {
                for slot in 0..300 {
                    let record = *staged.creatures.as_ref().unwrap().record(slot)
                        .map_err(|error| Error::World(error.into()))?;
                    // Type1 virtual68 excludes dead24..27; state15 is a
                    // separate source frozen-state exclusion.
                    if record[6] == 0 || record[0x5e] >= 24 || record[0x5e] == 15 { continue; }
                    let delta = |at: usize, center: usize| i32::from(
                        word(&record, at).wrapping_mul(2)
                            .wrapping_sub(word(&zone, center).wrapping_mul(2)) >> 1);
                    let (dx, dy) = (delta(7, 0), delta(9, 2));
                    if dx * dx + dy * dy > radius * radius { continue; }
                    receipts.push(staged.credit_one_krell_mule(
                        index as u8, slot as u16, frame, players, selected, context, trig,
                    )?);
                }
            }
            //7A2F3.. has activation/exhaustion effects beyond single-Mule
            // credit. A failure here must roll back every preceding credit.
            if i32::from_le_bytes(staged.backing[offset + 14..offset + 18].try_into().unwrap()) == 0 {
                return Err(Error::UnsupportedExhaustion);
            }
        }
        *self = staged;
        Ok(receipts)
    }
}


#[cfg(test)]
mod krell_schedule_admission_tests {
    use super::*;
    fn fixture() -> ActorWorld {
        let mut raw = vec![0; 343958];
        raw[..4].copy_from_slice(&0x55336u32.to_le_bytes());
        raw[337054..337058].copy_from_slice(&0x6beu32.to_le_bytes());
        ActorWorld::from_parts(
            &ConqMissionSource::parse(&raw).unwrap(), &[[0; 200]; 20],
            [ActorHandleBindings::empty(); 20],
            core::array::from_fn(|p| PlayerState::fresh_campaign(p as u8, [4096, 4096], b"fixture\0").unwrap()),
            BlueprintRng { seed: 0, ring: 31, draw_count: 0 },
        ).unwrap()
    }
    #[test]
    fn source_early_returns_precede_unsupported_context() {
        let trig = TrigTables::generate_pinned().unwrap();
        let context = LandingPhysicsContext { water_height: 0, gravity: 1024 };
        let mut world = fixture();
        // A countdown protects even unadmitted active metadata/context.
        world.backing[0x34b8] = 5;
        world.backing[0x355c..0x355e].copy_from_slice(&1u16.to_le_bytes());
        let before = world.clone();
        assert_eq!(world.step_krell_zone_callbacks(64, 2, 1, context, &trig), Ok(Vec::new()));
        assert_eq!(world, before);
        world.backing[0x355c..0x355e].fill(0);
        world.backing[0x34b8] = 0;
        let before = world.clone();
        assert_eq!(world.step_krell_zone_callbacks(64, 2, 1, context, &trig), Ok(Vec::new()));
        assert_eq!(world, before);
    }
    #[test]
    fn unsupported_active_context_and_count_preserve_entire_world() {
        let trig = TrigTables::generate_pinned().unwrap();
        let context = LandingPhysicsContext { water_height: 0, gravity: 1024 };
        let mut world = fixture();
        world.backing[0x34b8] = 1;
        for (players, selected) in [(0, 0), (2, 0), (1, 1)] {
            let before = world.clone();
            assert_eq!(world.step_krell_zone_callbacks(64, players, selected, context, &trig),
                Err(KrellZoneScheduleError::UnsupportedContext));
            assert_eq!(world, before);
        }
        world.backing[0x34b8] = 5;
        let before = world.clone();
        assert_eq!(world.step_krell_zone_callbacks(64, 1, 0, context, &trig),
            Err(KrellZoneScheduleError::UnsupportedZone));
        assert_eq!(world, before);
    }
}
