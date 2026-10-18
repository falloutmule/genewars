//! Bounded runtime state and startup transactions for the CD candidate.
//!
//! This crate models a deliberately limited plant subsystem. It preserves
//! packed source bytes for every family and the map, mutates only proved
//! family-1 fields, and reports suspended work for the other subsystems.
//! The separate actor world owns native records, typed bindings, player/map
//! transactions and pending commands. It requires explicit live inputs and
//! does not establish the full startup or landing timeline.

#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/world.rs"]pub mod world;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/identity.rs"]pub mod identity;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/actor_command.rs"]pub mod actor_command;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/actor_motion.rs"]pub mod actor_motion;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/player_state.rs"]pub mod player_state;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/blueprint.rs"]pub mod blueprint;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/actor_activation.rs"]pub mod actor_activation;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/actor_initialization.rs"]pub mod actor_initialization;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/actor_pool.rs"]pub mod actor_pool;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/actor_world.rs"]pub mod actor_world;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/actor_resources.rs"]pub mod actor_resources;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/target_projection.rs"]pub mod target_projection;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/actor_render.rs"]pub mod actor_render;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/landing.rs"]pub mod landing;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/effect_pool.rs"]pub mod effect_pool;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/terrain_target.rs"]pub mod terrain_target;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/physics_tables.rs"]pub mod physics_tables;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/landing_effect_lifecycle.rs"]pub mod landing_effect_lifecycle;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/private/review/mule-bones/candidate.rs"]pub mod creature_world;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/bookkeeping.rs"]pub mod bookkeeping;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/target_resolver.rs"]pub mod target_resolver;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/plant_spread.rs"]pub mod plant_spread;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/family8_creation.rs"]pub mod family8_creation;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/family8_pool.rs"]pub mod family8_pool;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/countdown_expiry.rs"]pub mod countdown_expiry;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/family8_motion.rs"]pub mod family8_motion;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/family8_particle.rs"]pub mod family8_particle;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/actor_construction.rs"]pub mod actor_construction;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/actor_construction_lifecycle.rs"]pub mod actor_construction_lifecycle;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/construction_terrain.rs"]pub mod construction_terrain;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/construction_site.rs"]pub mod construction_site;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/construction_setup.rs"]pub mod construction_setup;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/construction_completion.rs"]pub mod construction_completion;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/construction_maintenance.rs"]pub mod construction_maintenance;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/construction_terraform_completion.rs"]pub mod construction_terraform_completion;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/construction_terraform_resume.rs"]pub mod construction_terraform_resume;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/construction_payment.rs"]pub mod construction_payment;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/construction_motion.rs"]pub mod construction_motion;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/construction_wait.rs"]pub mod construction_wait;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/construction_cancellation.rs"]pub mod construction_cancellation;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/actor_construction_animation.rs"]pub mod actor_construction_animation;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/actor_construction_recovery.rs"]pub mod actor_construction_recovery;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/actor_construction_tail.rs"]pub mod actor_construction_tail;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/terrain_job_pool.rs"]pub mod terrain_job_pool;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/building.rs"]pub mod building;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/building_effect.rs"]pub mod building_effect;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/building_effect_lifecycle.rs"]pub mod building_effect_lifecycle;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/building_callback.rs"]pub mod building_callback;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/building_progress.rs"]pub mod building_progress;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/building_obstacle.rs"]pub mod building_obstacle;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/building_pool.rs"]pub mod building_pool;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/building_resources.rs"]pub mod building_resources;
#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/water_effect.rs"]pub mod water_effect;

pub use world::{
    MissionWorld, PlantParameterTable, PlantParameters, PlantRenderRecord,
    PlantTickReport, RuntimeError, SuspendedSubsystems, TickEvent,
};
