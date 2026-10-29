//! Shared roster/work transfer when booty clears the active combat roster.
use crate::{Event, Phase, RuleError, Session};

pub(crate) fn survivors(state: &mut Session) -> Result<Vec<Event>, RuleError> {
    let arena = state.combat.as_mut().ok_or("survivor combat missing")?;
    let result = if let Some(selection) = state.survivors.take() {
        crate::combat::allies::commit_survivors(
            &state.allies,
            state.ally_count,
            &mut arena.enemies.records,
            Some(&selection),
        )?
    } else if state.suppress_allies != 0 {
        crate::combat::allies::AllySynchronization {
            menu: state.allies.clone(),
            count: state.ally_count,
            modal: false,
            sounds: vec![],
        }
    } else {
        crate::combat::allies::synchronize(
            &state.allies,
            state.ally_count,
            &mut arena.enemies.records,
            &mut state.random,
        )?
    };
    state.allies = result.menu;
    state.ally_count = result.count;
    restore_weapons(state)?;
    Ok(result
        .sounds
        .into_iter()
        .map(|resource_id| Event::Sound { resource_id })
        .collect())
}

pub(crate) fn restore_weapons(state: &mut Session) -> Result<(), RuleError> {
    for record in state
        .allies
        .as_chunks_mut::<{ crate::story_allies::ALLY_BYTES }>()
        .0
        .iter_mut()
        .take(state.ally_count)
    {
        let weapon =
            centurions_content::records::EnemyRecord::from_bytes(record).primary_weapon()?;
        if weapon < 200 {
            centurions_content::records::EnemyRecordMut::from_bytes(record)
                .set_selected_item(weapon);
        }
    }
    Ok(())
}

pub(crate) fn detach(state: &mut Session) -> Result<(), RuleError> {
    state.retain_monster_work();
    let combat = state.combat.as_ref().ok_or("reward combat missing")?;
    state.native_work.round_work.capture(combat);
    state.native_work.turn_round = Some(combat.round);
    state.native_work.attack_packet = combat.attack_packet.clone();
    state.native_work.attack_identity = combat.attack_identity;
    state.native_work.weapon_work = combat.weapon_work;
    state.native_work.selected_enemy = combat.selected_enemy;
    state.native_work.script_enemy = combat.script_enemy;
    state.native_work.npc_backing.capture(combat);
    state
        .native_work
        .spell_target_tail
        .clone_from(&combat.target_tail);
    state.native_work.target_work = combat.target_work;
    state.native_work.tactical_work = crate::combat::tactical_work::TacticalWork::capture(
        combat,
        &state.native_work.tactical_work,
    );
    state.native_work.victory_work = combat.victory_work;
    state.native_work.neighbor_work = combat.neighbor_work.clone();
    state.native_work.initiative_work = crate::combat::retained::InitiativeWork::capture(combat);
    state.native_work.retained_death_queue = Some(combat.death_queue.clone());
    state.native_work.ready = combat.ready;
    state.store_spell_fatigue();
    state.combat = None;
    state.handled_deaths.clear();
    state.phase = Phase::Exploration;
    Ok(())
}
