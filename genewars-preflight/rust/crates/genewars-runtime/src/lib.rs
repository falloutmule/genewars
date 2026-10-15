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
pub mod airborne_building_collision;
pub mod mule_zone_credit;
pub mod zone_score_marker;
pub mod zone_score_marker_motion;
pub mod mule_human_producer;
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
pub mod creature_pick;
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
pub mod building_availability;
pub mod building_obstacle;
pub mod building_pool;
pub mod building_resources;
pub mod water_effect;

pub use world::{
    MissionWorld, PlantParameterTable, PlantParameters, PlantRenderRecord,
    PlantTickReport, RuntimeError, SuspendedSubsystems, TickEvent,
};

pub mod source_building_resources;

pub mod projectile_impact;
pub mod extractor_production;

pub mod terrain_height;

pub mod mission_context;

pub mod building_power;

pub mod terrain_border_table;

pub mod mission_outcome;

pub mod mission_bootstrap;
pub mod result_transition;
pub mod actor_construction_placement;
pub mod result_carry;

pub mod ui_animation;

pub mod genepod_idle;

pub mod mule_cell_entry;

pub mod genepod_command;

pub mod mule_waypoint_wait;

pub mod mule_lost_food;

pub mod mule_pair_init;
pub mod mule_idle_pair_init;

pub mod mule_pair_rendezvous;

pub mod genepod_human_producer;

pub mod mule_pair_center;

pub mod mule_pair_stop;

pub mod genepod_positive_work;

pub mod mule_factory;
pub mod mule_pair_birth;

pub mod mule_pair_birth_tail;

pub mod mule_retry_relocation;

pub mod genepod_capture;

pub mod genepod_work_callback;
pub mod genepod_terminal;

pub mod family8_zone_target;

pub mod effect20_factory;

pub mod effect20_motion;

pub mod owned_mule_ai;
pub mod owned_mule_common_choice;
pub mod owned_mule_callback;
pub mod mule_owned_command;

pub mod mule_corpse_arrival;
pub mod mule_corpse_abandon;
pub mod mule_bones_feeding;

pub mod mule_self_water_damage;
