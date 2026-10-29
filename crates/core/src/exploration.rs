//! Exploration entry points; movement, stories and presentation have separate owners.
mod map_context;
pub(crate) use map_context::active_map;
use map_context::unsupported;
pub(crate) mod boat;
mod movement;
pub(crate) use movement::resume_movement;
pub use movement::{manual_encounter, move_squad};
mod item_use;
pub(crate) use item_use::check_item_actor;
pub use item_use::use_item;
mod regions;
pub(crate) use regions::check_region;
#[cfg(all(test, feature = "original-fixtures"))]
pub use regions::regional_encounter;
use regions::{Regional, apply_region, choose_avoidance, regional_selection};
mod continuations;
use continuations::continue_with_spells;
pub(crate) use continuations::finish_story_page;
pub use continuations::{
    continue_messages, continue_story, keep_current_spells, needs_spell_confirmation,
};
mod choices;
pub(crate) use choices::finish_typed_selection;
pub use choices::{cancel_menu, choose, choose_warrior, select_menu};
mod menus;
use menus::{present_menu, present_typed};
pub use menus::{submit_text, typed_record};
mod loot;
pub(crate) use loot::present_treasure;
use loot::present_treasure_record;
pub use loot::{BootyTally, take_loot};
mod script_return;
use script_return::{abandon_script, finish_dispatch, finish_script};
pub(crate) use script_return::{dialogue, retain_script_stamp, retain_turn_header};
mod battle_entry;
pub(crate) use battle_entry::{StoryBattle, begin_story_battle};
mod script_execution;
pub(crate) use script_execution::{Operation, run_script};

#[cfg(all(test, feature = "original-fixtures"))]
use crate::*;

#[cfg(all(test, feature = "original-fixtures"))]
#[path = "../tests/native/regional_reference.rs"]
mod tests;
