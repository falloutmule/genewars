//! Bounded mission-entry player preparation using the recovered shared systems.
//! The caller supplies retained state, native priors and original stack scratch.
//! This plan does not invent their provenance or execute landing/reveal callbacks.
use crate::blueprint::{BlueprintGeneration, BlueprintNames, BlueprintRng};
use crate::player_state::{ActorOriginSample, PlayerState, PlayerStateError, PreparedPlayerActors};
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootstrapError { Player(PlayerStateError), ImmediateActivationRequired, FreshCampaignRequired }
impl From<PlayerStateError> for BootstrapError { fn from(value:PlayerStateError)->Self{Self::Player(value)} }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedMissionPlayer {
 pub generated: BlueprintGeneration,
 pub prepared: PreparedPlayerActors,
 /// 7F950 performs no backfill when every category is present.
 pub category_backfill_is_noop: bool,
}
/// Compose7F580,801C0 and the fresh no-unit prefix of80830.
/// Call after71B70/80590 restoration and7D090 identity/fund initialization.
/// Landing factory,72AD0 reveal/shade refresh and7F950 occur later in source order.
pub fn prepare_player(
 loaded: &PlayerState, names: &BlueprintNames, scratch:[u8;36], rng:BlueprintRng,
 actor_priors:&[[u8;200];5], mask:u8, origin:ActorOriginSample, target:[i16;2],
 special_activation:u32,
)->Result<PreparedMissionPlayer,BootstrapError>{
 if special_activation!=0{return Err(BootstrapError::ImmediateActivationRequired)}
 if loaded.creation_mode()!=0 || loaded.progress()!=0 || loaded.actor_count()!=0 {
  return Err(BootstrapError::FreshCampaignRequired)
 }
 let mut player=loaded.clone();
 let generated=player.regenerate_blueprints(scratch,names,rng,true)?;
 let prepared=player.prepare_fresh_actor_partition(actor_priors,mask,origin,target)?;
 let category_backfill_is_noop=(0..4u8).all(|category|generated.records.iter().any(|r|r[0]!=0&&r[0x16]==category));
 Ok(PreparedMissionPlayer{generated,prepared,category_backfill_is_noop})
}
