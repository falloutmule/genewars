use super::*;

impl ActorWorld {
    /// Read-only805E0 adapter resolving native actor blueprint handles.
    /// The complete72ED0 mission tail is not implied by this predicate.
    pub fn construction_capability_exhausted(&self,owner:u8)->Result<bool,ActorWorldError> {
        crate::mission_status::construction_capability_exhausted(self.player(owner)?.bytes(),|id|{
            let actor=self.entity([id as u8,(id as u16>>8)as u8,3]).ok()?;
            let handle=self.actors.bindings(id as usize).ok()?.blueprint?;
            if actor[24]!=handle.player_id{return None;}
            Some(self.player(handle.player_id).ok()?.blueprint(usize::from(handle.blueprint_index)).ok()?[0x16])
        }).map_err(ActorWorldError::MissionStatus)
    }
}
