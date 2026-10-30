use centurions_content::{Content, InventoryRecord};
pub mod bestiary;
pub mod character_display;
pub mod confiscation;
pub mod creation;
pub mod feedback;
pub mod item_display;
pub mod loot_recovery;
pub mod original_save;
pub mod outdoor_psi;
pub mod reference_views;
pub mod roster;
pub mod shop;
pub mod trace;

#[cfg(all(test, feature = "original-fixtures"))]
#[path = "../tests/unit/support.rs"]
mod test_support;

pub mod rules {
    /// Clamps a value to a range. The two comparisons run in order, so reversed
    /// bounds give the maximum.
    ///
    /// Original: pin
    pub fn pin(mut value: i16, minimum: i16, maximum: i16) -> i16 {
        if value < minimum {
            value = minimum;
        }
        if value > maximum {
            value = maximum;
        }
        value
    }

    /// The number of spells for a level: its square plus 16, where the addition
    /// is 16-bit and leaves the square's upper word intact.
    ///
    /// Original: getnumspells
    pub fn spell_count(level: i16) -> i32 {
        let square = i32::from(level) * i32::from(level);
        (square & !0xffff) | i32::from((square as u16).wrapping_add(16))
    }

    /// The base of a selection's block of 200, rounding toward zero.
    ///
    /// Original: getselection
    pub fn selection_base(selection: i16) -> i16 {
        (selection / 200).wrapping_mul(200)
    }

    /// Whether a value lies between two limits, both included.
    pub fn inclusive_range(value: i16, minimum: i16, maximum: i16) -> bool {
        value >= minimum && value <= maximum
    }

    /// A random number from 1 up to the bound, from a drawn value.
    pub fn random_bound(bound: i16, quickdraw_value: i16) -> i16 {
        let magnitude = i32::from(quickdraw_value).abs();
        (magnitude.wrapping_mul(i32::from(bound)) / 32768).wrapping_add(1) as i16
    }

    /// A random number from the minimum up to the maximum, both included, from a drawn value.
    pub fn random_range(minimum: i16, maximum: i16, quickdraw_value: i16) -> i16 {
        minimum
            .wrapping_add(random_bound(
                maximum.wrapping_sub(minimum).wrapping_add(1),
                quickdraw_value,
            ))
            .wrapping_sub(1)
    }

    /// What a step costs: half the tile's cost (rounded toward zero) less one, no lower than zero,
    /// plus one for each axis the step moves along.
    pub fn movement_cost(tile_cost: i16, dx: i8, dy: i8) -> i16 {
        let sign = (tile_cost as u16 >> 15) as i16;
        let cost = (tile_cost.wrapping_add(sign) >> 1).wrapping_sub(1);
        let base = if cost < 1 { 0 } else { cost };
        base.wrapping_add(i16::from(dx != 0))
            .wrapping_add(i16::from(dy != 0))
    }

    /// Converts a floating result to a signed 32-bit value.
    ///
    /// Exceptional results differed between the original machines: an FPU Mac
    /// saturated while software floating point returned other values. The game
    /// follows modern IEEE-754 evaluation, as recompiled Realmz ports do:
    /// infinities and out-of-range values saturate to the 32-bit bounds and NaN
    /// becomes zero. In-range values truncate toward zero.
    pub fn sane_long(value: f64) -> i32 {
        value as i32
    }
}

/// Whether warriors can be managed now: no medical choice is pending, and either the squad
/// selection holds a single warrior or exploration shows nothing or only a clinic.
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
pub use exploration::{BootyTally, needs_spell_confirmation, typed_record};
pub mod clinic;
pub mod field_controls;
mod item_modifiers;
pub mod items;
pub mod mission;
pub mod story;
pub use story::actor as story_actor;
pub use story::allies as story_allies;
pub use story::cleanup as story_cleanup;
pub use story::clock as story_clock;
pub use story::collection as story_collection;
pub use story::delay as story_delay;
pub use story::destination as story_destination;
pub use story::health as story_health;
pub use story::inventory as story_inventory;
pub use story::item_use as story_item_use;
pub use story::manual_encounter as story_manual_encounter;
pub use story::map_picture as story_map_picture;
pub use story::move_away as story_move_away;
pub use story::npc_battle as story_npc_battle;
pub use story::party as story_party;
pub use story::promotion as story_promotion;
pub use story::reduce as story_reduce;
pub use story::remove_allies as story_remove_allies;
pub use story::schedule as story_schedule;
pub use story::skill as story_skill;
use story::spell as story_spell;
pub use story::treasure as story_treasure;
pub use story::variables as story_variables;
mod time;
pub use time::{ZeroOwners, ZeroTimeReturn};
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
pub use command::{Command, ItemInformationList};
use defaults::{
    AMMUNITION_SLOTS, DEFINE_SPELLS_BYTES, FLAG_BYTES, NOTE_BYTES, PARTY_CONDITION_BYTES,
    STORED_ITEM_RECORDS, empty_allies, empty_ammunition, empty_notes, empty_party_buffs,
    initial_flags, initial_library_seed, initial_psi_memory, menu_length, no_notes,
    normal_difficulty, zero_level, zero_word,
};
pub use error::RuleError;
pub use event::{
    Event, Flight, PresentationFlight, PresentationIcon, RayCrossing, SpellArea, SpellInfo,
};
pub use persistence::{atomic_write, load, save};
pub use random::RandomState;
pub use state::*;

mod commands;
pub use commands::apply;

use commands::insert;
