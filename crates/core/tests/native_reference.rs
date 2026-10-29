include!("native/modules.rs");
mod campaign;
mod enemy_callers;
#[path = "hex.rs"]
mod fixture_hex;
mod native_common;
mod prelude;
mod snapshots;
mod support;

mod command_clinic;
mod command_encounter_replacement;
mod command_encounter_work;
mod command_equipment_globals;
mod command_item_activation;
mod command_movement_duration;
mod command_reaction_return;
mod command_script_item_returns;
mod command_spell_routes;
mod source_field_collisions;

mod command_turn_story;
mod source_npc_initialization;
