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
pub mod actor_motion;
pub mod player_state;
pub mod mission_player_funds;
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
pub mod bookkeeping;
pub mod target_resolver;
pub mod plant_spread;
pub mod family8_creation;
pub mod family8_pool;
pub mod countdown_expiry;
pub mod family8_motion;
pub mod family8_particle;
pub mod actor_construction;
pub mod actor_construction_lifecycle;
pub mod construction_terrain;
pub mod construction_site;
pub mod construction_setup;
pub mod construction_completion;
pub mod construction_maintenance;
pub mod construction_terraform_completion;
pub mod construction_terraform_resume;
pub mod construction_payment;
pub mod construction_stage4;
pub mod construction_work;
pub mod completed_building;
pub mod mission_status;
pub mod construction_motion;
pub mod construction_wait;
pub mod construction_cancellation;
pub mod actor_construction_animation;
pub mod actor_construction_recovery;
pub mod actor_construction_tail;
pub mod terrain_job_pool;
pub mod building;
pub mod building_effect;
pub mod building_effect_lifecycle;
pub mod building_callback;
pub mod building_progress;
pub mod building_obstacle;
pub mod building_pool;
pub mod building_resources;
pub mod water_effect;

pub use world::{
    MissionWorld, PlantParameterTable, PlantParameters, PlantRenderRecord,
    PlantTickReport, RuntimeError, SuspendedSubsystems, TickEvent,
};

pub mod turret_projectile;
