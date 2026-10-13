//! Bounded runtime state and startup transactions for the CD candidate.
//!
//! This crate models a deliberately limited plant subsystem. It preserves
//! packed source bytes for every family and the map, mutates only proved
//! family-1 fields, and reports suspended work for the other subsystems.
//! Separate player/blueprint and activation helpers require explicit live
//! inputs; they do not establish the full startup or landing timeline.

pub mod world;
pub mod identity;
pub mod actor_command;
pub mod player_state;
pub mod blueprint;
pub mod actor_activation;
pub mod actor_initialization;

pub use world::{
    MissionWorld, PlantParameterTable, PlantParameters, PlantRenderRecord,
    PlantTickReport, RuntimeError, SuspendedSubsystems, TickEvent,
};
