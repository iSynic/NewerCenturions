use centurions_content::{Content, InventoryRecord};
pub mod character_display;
pub mod confiscation;
pub mod creation;
pub mod feedback;
pub mod item_display;
pub mod native_save;
pub mod outdoor_psi;
pub mod reference_views;
pub mod roster;
pub mod shop;

#[cfg(all(test, feature = "original-fixtures"))]
#[path = "../tests/unit/support.rs"]
mod test_support;

pub mod rules {
    /// CODE 1 pin, 0x13f65e. Sequential comparisons also define reversed bounds.
    pub fn pin(mut value: i16, minimum: i16, maximum: i16) -> i16 {
        if value < minimum {
            value = minimum;
        }
        if value > maximum {
            value = maximum;
        }
        value
    }

    /// CODE 1 getnumspells, 0x125512: ADD.W leaves the product's upper word intact.
    pub fn spell_count(level: i16) -> i32 {
        let square = i32::from(level) * i32::from(level);
        (square & !0xffff) | i32::from((square as u16).wrapping_add(16))
    }

    /// CODE 1 getselection, 0x125a08: signed quotient, rounded toward zero.
    pub fn selection_base(selection: i16) -> i16 {
        (selection / 200).wrapping_mul(200)
    }

    pub fn inclusive_range(value: i16, minimum: i16, maximum: i16) -> bool {
        value >= minimum && value <= maximum
    }

    pub fn random_bound(bound: i16, quickdraw_value: i16) -> i16 {
        let magnitude = i32::from(quickdraw_value).abs();
        (magnitude.wrapping_mul(i32::from(bound)) / 32768).wrapping_add(1) as i16
    }

    pub fn random_range(minimum: i16, maximum: i16, quickdraw_value: i16) -> i16 {
        minimum
            .wrapping_add(random_bound(
                maximum.wrapping_sub(minimum).wrapping_add(1),
                quickdraw_value,
            ))
            .wrapping_sub(1)
    }

    pub fn movement_cost(tile_cost: i16, dx: i8, dy: i8) -> i16 {
        let sign = (tile_cost as u16 >> 15) as i16;
        let cost = (tile_cost.wrapping_add(sign) >> 1).wrapping_sub(1);
        let base = if cost < 1 { 0 } else { cost };
        base.wrapping_add(i16::from(dx != 0))
            .wrapping_add(i16::from(dy != 0))
    }
}

fn warrior_management_available(state: &Session) -> bool {
    state.medical_continuation.is_none()
        && (state.phase == Phase::SquadSelection && state.warriors.len() == 1
            || state.phase == Phase::Exploration
                && state.exploration.as_ref().is_some_and(|p| {
                    p.presentation.is_none()
                        || matches!(p.presentation, Some(Presentation::Clinic { .. }))
                }))
}

pub mod ammunition;
pub mod character;
pub mod combat;
pub mod editor_items;
pub mod equipment;
mod exploration;
pub mod party;
pub mod reconstruction;
pub mod recovered_ai;
pub mod recovered_application_control;
pub mod recovered_attack;
pub mod recovered_audio;
pub mod recovered_battle_order;
pub mod recovered_bodies;
pub mod recovered_character_creation;
pub mod recovered_character_panels;
pub mod recovered_character_ui;
pub mod recovered_character_upgrade;
pub mod recovered_combat_center;
pub mod recovered_combat_panel;
pub mod recovered_combat_results;
pub mod recovered_combat_setup;
pub mod recovered_combat_ui;
pub mod recovered_detail_ui;
pub mod recovered_drawing;
pub mod recovered_economy;
pub mod recovered_equipment;
pub mod recovered_equipment_ui;
pub mod recovered_fade_ui;
pub mod recovered_feedback;
pub mod recovered_field_control;
pub mod recovered_files;
pub mod recovered_gamma;
pub mod recovered_gamma_control;
pub mod recovered_geometry;
pub mod recovered_install;
pub mod recovered_interface;
pub mod recovered_inventory;
pub mod recovered_inventory_edit;
pub mod recovered_inventory_interaction;
pub mod recovered_inventory_rules;
pub mod recovered_inventory_selection;
pub mod recovered_item_ui;
pub mod recovered_load_control;
pub mod recovered_loot_control;
pub mod recovered_native_services;
pub mod recovered_navigation;
pub mod recovered_order_form;
pub mod recovered_preferences;
pub mod recovered_preparation;
pub mod recovered_prompt_control;
pub mod recovered_random_item;
pub mod recovered_records;
pub mod recovered_rendering;
pub mod recovered_runtime;
pub mod recovered_scenario_menus;
pub mod recovered_selection_control;
pub mod recovered_session_control;
pub mod recovered_shop_ui;
pub mod recovered_spatial;
pub mod recovered_spell_animation;
pub mod recovered_spell_ui;
pub mod recovered_startup;
pub mod recovered_status_panels;
pub mod recovered_storage;
pub mod recovered_system_control;
pub mod recovered_text;
pub mod recovered_timing;
pub mod recovered_verification;
pub mod recovered_world_display;
pub use exploration::{needs_spell_confirmation, typed_record};
pub mod clinic;
pub mod field_controls;
mod item_modifiers;
pub mod items;
pub mod mission;
pub mod story;
pub mod story_actor;
pub mod story_allies;
pub mod story_cleanup;
pub mod story_collection;
mod story_combat;
pub mod story_delay;
pub mod story_destination;
pub mod story_health;
pub mod story_inventory;
pub mod story_item_use;
pub mod story_manual_encounter;
pub mod story_map_picture;
pub mod story_move_away;
pub mod story_npc_battle;
pub mod story_party;
pub mod story_promotion;
pub mod story_reduce;
pub mod story_registration;
pub mod story_remove_allies;
pub mod story_schedule;
pub mod story_skill;
mod story_spell;
pub mod story_treasure;
pub mod story_variables;
mod time;
pub mod travel;
pub mod typed_encounter;
pub mod walking_header;

mod command;
mod defaults;
mod error;
mod event;
mod persistence;
mod random;
mod session;
mod state;
pub use command::Command;
use defaults::{
    empty_allies, empty_ammunition, empty_notes, empty_party_buffs, initial_flags,
    initial_library_seed, initial_psi_memory, menu_length, no_notes, normal_difficulty, zero_level,
    zero_word,
};
pub use error::RuleError;
pub use event::Event;
pub use persistence::{atomic_write, load, save};
pub use random::RandomState;
pub use state::*;

mod commands;
pub use commands::apply;

use commands::insert;
