//! Runtime-owned initialization writes made by7D090, outside serialized data.
use super::*;
impl ActorWorld {
    pub fn initialize_fresh_runtime_countdown(&mut self, context: &crate::mission_context::MissionRuntimeContext) -> Result<(), &'static str> {
        if !matches!(context.origin(),crate::mission_context::ContextOrigin::FreshCdConq001|crate::mission_context::ContextOrigin::CarriedCdKrell) || context.frame() != 0 {
            return Err("countdown initialization requires admitted source mission-entry context");
        }
        self.backing[0x355c..0x355e].copy_from_slice(&context.countdown().to_le_bytes());
        Ok(())
    }
}
