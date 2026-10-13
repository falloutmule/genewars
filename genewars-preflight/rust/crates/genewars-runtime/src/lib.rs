//! Bounded mutable runtime state for the admitted first-mission plant slice.
//!
//! This crate models a deliberately limited plant subsystem. It preserves
//! packed source bytes for every family and the map, mutates only proved
//! family-1 fields, and reports suspended work rather than inventing behavior
//! for the other subsystems.

pub mod world;
pub mod identity;
pub mod actor_command;

pub use world::{
    MissionWorld, PlantParameterTable, PlantParameters, PlantRenderRecord,
    PlantTickReport, RuntimeError, SuspendedSubsystems, TickEvent,
};
