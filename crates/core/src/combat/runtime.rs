//! Compatibility entry points for the maintained combat runtime.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum TurnFlow {
    #[default]
    Returned,
    Terminated,
}

#[derive(Default)]
pub(crate) struct TurnOutcome {
    pub events: Vec<crate::Event>,
    pub flow: TurnFlow,
}

mod actors;
pub use actors::neighbors;
use actors::{
    FireOutcome, alive, has_active_warrior, hostile, neighbors_at, position, refusal,
    spell_identity,
};
mod attacks;
mod detached_attack;
mod detached_warrior;
mod outdoor_death;
mod weapon_context;
pub(crate) use weapon_context::refresh_body_item;
use weapon_context::{is_melee_weapon, load_melee_work, load_natural_work};
mod movement_return;
mod reaction_calls;
mod reaction_frame;
mod retained_arms;
pub(crate) use retained_arms::run as retained_arm;
mod retained_spell;
mod retained_turn;
pub(crate) mod reward_cleanup;
pub(crate) use movement_return::answer_retained_walk;
pub use movement_return::{CombatReturn, MoveFrame, RetainedWalkChoice};
pub(crate) use movement_return::{MovePlan, begin as begin_move};
mod reactions;
use reactions::check_neighbors_with_items;
pub(crate) use reactions::check_neighbors_with_work;
pub use reactions::{
    check_neighbors, check_neighbors_with_buffs, check_neighbors_with_context,
    check_neighbors_with_definitions,
};
mod departure;
pub(crate) use departure::move_actor_with_reactions;
pub use departure::{warrior_departure, warrior_departure_with_buffs};
mod enemy_actions;
pub use enemy_actions::{
    EnemyWork, enemy_action, enemy_action_with_work, enemy_action_with_work_and_buffs,
};
mod spell_results;
pub(crate) use spell_results::{append_spell_outcomes, resolve_spell_mask};
mod enemy_spells;
pub(crate) use enemy_spells::enemy_spell;
pub(crate) use enemy_spells::live_arm;
use enemy_spells::{enemy_spell_continue, refresh_enemy_item_alias};
mod enemy_fire;
use enemy_fire::enemy_fire;
mod player_actions;
pub use player_actions::{player_attack, player_attack_friend, player_guard, select_enemy_target};
mod actor_turns;
pub use actor_turns::{controlled_turn, controlled_warrior, enemy_turn};
mod npc_return;
pub use npc_return::NpcActionFrame;
pub(crate) use npc_return::resume_pending as resume_npc_action;
mod player_psi;
pub use player_psi::{player_psi, player_psi_warrior};
mod medical;
pub use medical::{begin_learned_heal, begin_medical_item, select_medical_target, stop_bleeding};
pub(crate) use medical::{refresh_active_item_context, refresh_item_context};
mod party_psi;
pub use party_psi::party_psi;
mod ammunition;
pub use ammunition::fit_ammunition;
mod shooting;
pub use shooting::{player_shoot, player_shoot_action, player_shoot_immediate};
pub(crate) use shooting::{
    player_shoot_at_action, player_shoot_locations_action, player_shoot_targets_action,
};
mod turn_stories;
pub use turn_stories::{TurnCaller, TurnChildReturn, TurnDispatch};
pub(crate) use turn_stories::{
    TurnInitialization, begin_child as begin_turn_child, finish as finish_turn_dispatch,
    initialize_session as initialize_session_turn, mark_cleanup as mark_turn_cleanup,
    resume_child as resume_turn_child,
};
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
pub use fields::collide_fields;
pub(crate) use fields::collide_fields_with_work;
use fields::{FieldActor, collide_fields_impl};
mod victory;
pub use victory::{victory_loot, victory_records};
mod retained_rewards;
mod reward_context;
pub(crate) use retained_rewards::{
    begin as begin_retained_rewards, confirm as confirm_retained_rewards, resume_deferred_getup,
    validate_count as validate_retained_reward_count, validate_deferred_getup,
};
pub(crate) use reward_context::{
    detach as detach_reward_combat, survivors as synchronize_reward_survivors,
};
mod battle_return;
mod ordinary_rewards;
pub use ordinary_rewards::OrdinaryBattleRewards;
pub(crate) use ordinary_rewards::after_close as close_ordinary_rewards;
pub(crate) use ordinary_rewards::{finish as finish_early_rewards, install as install_rewards};
mod selected_rewards;
pub use battle_return::confirm_survivors;
pub(crate) use battle_return::{begin_survivors, refit_after_combat};
use battle_return::{finish_battle_wrapper, finish_victory};
pub use selected_rewards::SelectedBattleTreasure;
pub(crate) use selected_rewards::after_close as close_selected_rewards;
pub(crate) use selected_rewards::begin_cleanup as begin_selected_cleanup;
pub(crate) use selected_rewards::retreat as retreat_selected_rewards;
mod death;
mod death_return;
use death::finish_action;
pub use death::{DeathStory, continue_death};
pub(crate) use death::{
    finish_death_story, finish_player_action, retain_death_header, retain_death_parent,
    retain_last_defeated,
};
