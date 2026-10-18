//! Linked-cell portions of84960/84B40/8B840. Call on a staged world.
use super::*;
use genewars_core::cd_candidate::{interpolate_entity_shade, interpolate_height};

impl ActorWorld {
    ///84960 uses masks C,8,C,4;84B40 uses C for all four corners.
    pub(super) fn refresh_terrain_vertex(&mut self, position:[i16;2], slope:bool)
        -> Result<(),ActorWorldError> {
        let x=(i32::from(position[0])>>8)&127;
        let y=(i32::from(position[1])>>8)&127;
        for (n,(dx,dy)) in [(0,0),(-1,0),(-1,-1),(0,-1)].into_iter().enumerate() {
            let cell=(((y+dy)&127)*128+((x+dx)&127)) as usize;
            self.backing[MAP_START+cell*14+11]|=if slope{12}else{[12,8,12,4][n]};
            let mut raw=reference(self.map(),cell*14+6);
            let mut visited=HashSet::new();
            while raw[2]!=0 {
                if !visited.insert(raw){return Err(ActorWorldError::DuplicateLink(raw));}
                let before=self.entity(raw)?.to_vec();
                let next=reference(&before,3);
                // Actual reference vtables: plants/buildings1F280, others1F240.
                match raw[2] {
                    1|3|5|8=>{},
                    2|4=>if before[0x15]&2==0 {
                        let shade=interpolate_entity_shade(self.map(),word(&before,7),word(&before,9))
                            .map_err(|e|ActorWorldError::Terrain(e.to_string()))?;
                        let mut after=before.clone();after[0x17]=shade;
                        self.replace_terrain_entity(raw,&before,&after)?;
                    },
                    kind=>return Err(ActorWorldError::UnsupportedRevealFamily(kind)),
                }
                raw=next;
            }
        }
        Ok(())
    }

    pub(super) fn refresh_terrain_object_heights(&mut self,cell:u16)->Result<(),ActorWorldError>{
        let mut raw=reference(self.map(),usize::from(cell)*14+6);
        let mut visited=HashSet::new();
        while raw[2]!=0 {
            if !visited.insert(raw){return Err(ActorWorldError::DuplicateLink(raw));}
            let before=self.entity(raw)?.to_vec();let next=reference(&before,3);
            let ground=interpolate_height(self.map(),word(&before,7),word(&before,9))
                .map_err(|e|ActorWorldError::Terrain(e.to_string()))?;
            let mut after=before.clone();after[17..21].copy_from_slice(&ground.to_le_bytes());
            match raw[2] {
                //1F440 returns0, so these have no physical-height write.
                2|4=>{},
                //1F260 subtracts the just-written ground from physical height.
                1|3|5|8=>{
                    if before.len()<43{return Err(ActorWorldError::EnvironmentUnavailable);}
                    let height=i32::from_le_bytes(before[39..43].try_into().unwrap());
                    let forced=raw[2]==5&&before.get(0x45)==Some(&0x2e);
                    if height.wrapping_sub(ground)<0||forced{after[39..43].copy_from_slice(&ground.to_le_bytes());}
                },
                kind=>return Err(ActorWorldError::UnsupportedRevealFamily(kind)),
            }
            self.replace_terrain_entity(raw,&before,&after)?;raw=next;
        }
        Ok(())
    }

    pub(super) fn replace_terrain_entity(&mut self,raw:[u8;3],before:&[u8],after:&[u8])->Result<(),ActorWorldError>{
        let id=usize::from(u16::from_le_bytes([raw[0],raw[1]]));
        match raw[2] {
            1=>self.creatures.as_mut().ok_or(ActorWorldError::EnvironmentUnavailable)?
                .replace_record_checked(id,before.try_into().map_err(|_|ActorWorldError::EnvironmentUnavailable)?,after.try_into().map_err(|_|ActorWorldError::EnvironmentUnavailable)?)?,
            3=>self.actors.replace_record_checked(id,before.try_into().map_err(|_|ActorWorldError::EnvironmentUnavailable)?,after.try_into().map_err(|_|ActorWorldError::EnvironmentUnavailable)?)?,
            4=>self.buildings.as_mut().ok_or(ActorWorldError::BuildingsNotAdmitted)?
                .replace_record_checked(id,before.try_into().map_err(|_|ActorWorldError::EnvironmentUnavailable)?,after.try_into().map_err(|_|ActorWorldError::EnvironmentUnavailable)?)?,
            5=>self.effects.as_mut().ok_or(ActorWorldError::EnvironmentUnavailable)?
                .replace_record_checked(id,before.try_into().map_err(|_|ActorWorldError::EnvironmentUnavailable)?,after.try_into().map_err(|_|ActorWorldError::EnvironmentUnavailable)?)?,
            8=>self.family8.as_mut().ok_or(ActorWorldError::EnvironmentUnavailable)?
                .replace_record_checked(id,before.try_into().map_err(|_|ActorWorldError::EnvironmentUnavailable)?,after.try_into().map_err(|_|ActorWorldError::EnvironmentUnavailable)?)?,
            2=>{let layout=self.layouts[1];let start=layout.file_offset+id*layout.packed_stride;
                if before.len()!=31||after.len()!=31||self.backing[start..start+31]!=*before{return Err(ActorWorldError::StaleLink);}
                self.backing[start..start+31].copy_from_slice(after);
            },
            kind=>return Err(ActorWorldError::UnsupportedRevealFamily(kind)),
        }
        Ok(())
    }
}
