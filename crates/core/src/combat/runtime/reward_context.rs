//! Handing the roster to booty when the battle ends.
use crate::combat::allies::AllySynchronization;
use crate::combat::battlefield::Battlefield;
use crate::combat::globals::BattleOrder;
use crate::{Event, Phase, RuleError, Session};

/// Settles the survivors with the held-over allies before the reward. The player's choice from the
/// survivors screen is committed; with allies suspended they stay as they were; otherwise the
/// roster's survivors are matched against them. Held-over allies then get their weapons back.
pub(crate) fn survivors(state: &mut Session) -> Result<Vec<Event>, RuleError> {
    let arena = state.combat.as_mut().ok_or("survivor combat missing")?;
    let result = if let Some(selection) = state.survivors.take() {
        crate::combat::allies::commit_survivors(
            &state.holdover,
            state.held_over,
            &mut arena.roster.records,
            Some(&selection),
        )?
    } else if state.suspend_allies != 0 {
        AllySynchronization {
            menu: state.holdover.clone(),
            count: state.held_over,
            modal: false,
            sounds: vec![],
        }
    } else {
        crate::combat::allies::synchronize(
            &state.holdover,
            state.held_over,
            &mut arena.roster.records,
            &mut state.random,
        )?
    };
    state.holdover = result.menu;
    state.held_over = result.count;
    restore_weapons(state)?;
    Ok(result
        .sounds
        .into_iter()
        .map(|resource_id| Event::Sound { resource_id })
        .collect())
}

/// Sets the weapon of each held-over ally to its primary weapon when that is below 200.
pub(crate) fn restore_weapons(state: &mut Session) -> Result<(), RuleError> {
    for record in state
        .holdover
        .as_chunks_mut::<{ crate::story::allies::ALLY_BYTES }>()
        .0
        .iter_mut()
        .take(state.held_over)
    {
        let weapon =
            centurions_content::records::EnemyRecord::from_bytes(record).primary_weapon()?;
        if weapon < 200 {
            centurions_content::records::EnemyRecordMut::from_bytes(record).set_weapon(weapon);
        }
    }
    Ok(())
}

/// Leaves the battle: what the party's globals keep of the battle (round scripts, positions, kill
/// counts, battle order, death queue and the rest) is copied from it, and the game returns to
/// exploration.
pub(crate) fn leave_battle(state: &mut Session) -> Result<(), RuleError> {
    state.store_monster_pick();
    let combat = state.combat.as_ref().ok_or("reward combat missing")?;
    state.globals.round_scripts.capture(combat);
    state.globals.turn_round = Some(combat.combat_round);
    state.globals.spell_info = combat.spell_info.clone();
    state.globals.cast_identity = combat.cast_identity;
    state.globals.weapon_traits = combat.weapon_traits;
    state.globals.monster_up = combat.monster_up;
    state.globals.macro_monster = combat.macro_monster;
    state.globals.monsters.capture(combat);
    state.globals.track_tail.clone_from(&combat.track_tail);
    state.globals.target_picks = combat.target_picks;
    state.globals.battlefield = Battlefield::capture(combat, &state.globals.battlefield);
    state.globals.kill_counts = combat.kill_counts;
    state.globals.neighbors = combat.neighbors.clone();
    state.globals.battle_order = BattleOrder::capture(combat);
    state.globals.outer_todo_queue = Some(combat.todo_queue.clone());
    state.globals.in_spell = combat.in_spell;
    state.store_spell_fatigue();
    state.combat = None;
    state.handled_deaths.clear();
    state.phase = Phase::Exploration;
    Ok(())
}
