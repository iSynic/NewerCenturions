//! Compatibility entry points for the maintained combat runtime.
mod actors;
pub use actors::neighbors;
use actors::{
    FireOutcome, alive, has_active_warrior, hostile, neighbors_at, position, refusal,
    spell_identity,
};
mod attacks;
use attacks::resolve_attack;
mod weapon_context;
pub(crate) use weapon_context::refresh_body_item;
use weapon_context::{is_melee_weapon, load_melee_work, load_natural_work};
mod reactions;
use reactions::check_neighbors_with_items;
pub(crate) use reactions::check_neighbors_with_work;
pub use reactions::{
    check_neighbors, check_neighbors_with_buffs, check_neighbors_with_context,
    check_neighbors_with_definitions,
};
mod departure;
pub(crate) use departure::{move_actor_with_reactions, warrior_departure_with_work};
pub use departure::{warrior_departure, warrior_departure_with_buffs};
mod enemy_actions;
pub use enemy_actions::{
    EnemyWork, enemy_action, enemy_action_with_work, enemy_action_with_work_and_buffs,
};
mod spell_results;
pub(crate) use spell_results::{append_spell_outcomes, resolve_spell_mask};
use spell_results::{enemy_death_sound, increment_stat};
mod enemy_spells;
use enemy_spells::{enemy_spell, enemy_spell_continue, refresh_enemy_item_alias};
mod enemy_fire;
use enemy_fire::enemy_fire;
mod player_actions;
pub use player_actions::{player_attack, player_attack_friend, player_guard, select_enemy_target};
mod actor_turns;
pub use actor_turns::{controlled_turn, controlled_warrior, enemy_turn};
mod player_psi;
pub use player_psi::{player_psi, player_psi_warrior};
mod medical;
pub(crate) use medical::refresh_active_item_context;
pub use medical::{begin_learned_heal, begin_medical_item, select_medical_target, stop_bleeding};
mod party_psi;
pub use party_psi::party_psi;
mod ammunition;
pub use ammunition::fit_ammunition;
mod shooting;
pub use shooting::{player_shoot, player_shoot_action, player_shoot_immediate};
pub(crate) use shooting::{player_shoot_at_action, player_shoot_targets_action};
mod turns;
pub(crate) use turns::resume_selection;
pub use turns::{end_turn, initialize_active_turn};
use turns::{select_turn, select_turn_bounded};
mod rounds;
use rounds::advance_round;
pub(crate) use rounds::{continue_round, finish_round_dispatch};
mod round_effects;
pub use round_effects::reduce_round;
use round_effects::{RoundContext, reduce_round_with_fields};
mod enemy_round;
pub use enemy_round::{EnemyRoundOutcome, reduce_enemy_round_with_fields};
use enemy_round::{record_spell_deaths, reduce_enemy_round_impl};
mod fields;
use fields::{FieldActor, collide_fields_impl};
pub use fields::{collide_fields, collide_fields_with_mask};
mod victory;
pub use victory::{victory_loot, victory_records};
mod battle_return;
pub use battle_return::confirm_survivors;
pub(crate) use battle_return::{begin_survivors, refit_after_combat};
use battle_return::{finish_battle_wrapper, finish_victory};
mod death;
use death::finish_action;
pub use death::{DeathStory, continue_death};
pub(crate) use death::{
    finish_death_story, finish_player_action, retain_death_parent, retain_last_defeated,
};
