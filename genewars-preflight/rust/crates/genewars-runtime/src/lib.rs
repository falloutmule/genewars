//! Bounded runtime state and startup transactions for the CD candidate.
//!
//! This crate models a deliberately limited plant subsystem. It preserves
//! packed source bytes for every family and the map, mutates only proved
//! family-1 fields, and reports suspended work for the other subsystems.
//! The separate actor world owns native records, typed bindings, player/map
//! transactions and pending commands. It requires explicit live inputs and
//! does not establish the full startup or landing timeline.

pub mod world;
pub mod identity;
pub mod actor_command;
pub mod player_state;
pub mod blueprint;
pub mod actor_activation;
pub mod actor_initialization;
pub mod actor_pool;
pub mod actor_world;
pub mod actor_resources;
pub mod target_projection;
pub mod actor_render;
pub mod landing;
pub mod effect_pool;
pub mod terrain_target;
pub mod physics_tables;
pub mod landing_effect_lifecycle;
pub mod creature_world;

pub use world::{
    MissionWorld, PlantParameterTable, PlantParameters, PlantRenderRecord,
    PlantTickReport, RuntimeError, SuspendedSubsystems, TickEvent,
};
