//! The records and map a monster's resumed turn reads and writes.
use crate::combat::map_alias::Land;
use crate::combat::party_buffs;
use crate::combat::records;
use crate::combat::{CombatField, runtime::EnemyGlobals};
use crate::{Content, Event, RuleError, Session};
use centurions_content::{Warrior, combat::MonsterRecord, mission::OpeningMission};

/// The refusal for a monster turn that depends on an unrecovered part of the game.
pub(super) fn missing(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.npc_return",
        dependency,
    }
}

/// The monster whose turn it is.
pub(super) fn selected(state: &Session) -> Result<usize, RuleError> {
    let index = state
        .combat
        .as_ref()
        .map_or(state.globals.monster_up, |combat| combat.monster_up);
    if index >= 100 {
        return Err(missing("NPC selector outside represented records"));
    }
    Ok(index)
}
/// A monster's record, from the battle or from the party's globals after it.
pub(super) fn record(state: &Session, index: usize) -> Result<&MonsterRecord, RuleError> {
    if let Some(combat) = &state.combat {
        combat.story_record(index)
    } else {
        Ok(&state.globals.monsters.slot(index)?.record)
    }
}
/// A monster's record to change, from the battle or from the party's globals after it.
pub(super) fn record_mut(
    state: &mut Session,
    index: usize,
) -> Result<&mut MonsterRecord, RuleError> {
    if let Some(combat) = &mut state.combat {
        combat.story_record_mut(index)
    } else {
        Ok(&mut state.globals.monsters.slot_mut(index)?.record)
    }
}
/// The map of the field, from the battle or from the party's globals after it.
pub(super) fn map(state: &Session) -> Result<&CombatField, RuleError> {
    if let Some(combat) = &state.combat {
        Ok(&combat.field)
    } else {
        Ok(&state.globals.battlefield.field)
    }
}
/// Replaces the map of the field.
pub(super) fn set_map(state: &mut Session, map: CombatField) -> Result<(), RuleError> {
    if let Some(combat) = &mut state.combat {
        combat.field = map;
    } else {
        state.globals.battlefield.field = map;
    }
    Ok(())
}
/// The cell a monster stands in.
pub(super) fn position(state: &Session, index: usize) -> Result<(i16, i16), RuleError> {
    if let Some(combat) = &state.combat {
        return combat.story_position(index);
    }
    let position = state.globals.monsters.slot(index)?.position()?;
    let shift = map(state)?.map_shift;
    Ok(records::grid_position(position, shift))
}
/// Moves a monster's recorded cell.
pub(super) fn set_position(
    state: &mut Session,
    index: usize,
    point: (i16, i16),
) -> Result<(), RuleError> {
    let shift = map(state)?.map_shift;
    let absolute = records::grid_point(point, shift);
    if let Some(combat) = &mut state.combat {
        if let Some(position) = combat.roster.mon_pos.get_mut(index) {
            *position = point;
        } else {
            combat.monsters.slot_mut(index)?.position = Some(absolute);
        }
    } else {
        state.globals.monsters.slot_mut(index)?.position = Some(absolute);
    }
    Ok(())
}
/// The body whose turn it is, from the battle order.
pub(super) fn body(state: &Session) -> Result<i16, RuleError> {
    if let Some(combat) = &state.combat {
        combat
            .battle_order
            .get(combat.battle_order_index)
            .copied()
            .or(match combat.battle_order_index {
                110 => Some(110),
                111 => Some(combat.monster_up as i16),
                _ => None,
            })
            // Row 112 is the dispatch mode, past the battle order.
            .ok_or_else(|| {
                if combat.battle_order_index == 112 {
                    missing("NPC initiative row 112 reads the dispatch mode WORD")
                } else {
                    missing("NPC initiative row past the initiative work")
                }
            })
    } else {
        state.globals.battle_order.current(selected(state)?)
    }
}
/// The direction of the latest step.
pub(super) fn direction(state: &Session) -> Result<(i8, i8), RuleError> {
    let direction = state
        .exploration
        .as_ref()
        .ok_or("NPC movement map missing")?
        .last_move;
    Ok((direction[0], direction[1]))
}
/// Sets the direction of the latest step.
pub(super) fn set_direction(state: &mut Session, direction: (i8, i8)) -> Result<(), RuleError> {
    state
        .exploration
        .as_mut()
        .ok_or("NPC movement map missing")?
        .last_move = [direction.0, direction.1];
    Ok(())
}
/// The land the battle is fought on.
pub(super) fn mission<'a>(
    state: &Session,
    content: &'a Content,
) -> Result<&'a OpeningMission, RuleError> {
    state.land(content).ok_or("NPC mission missing".into())
}
/// Records in the health context the effect of the current attack packet, the warriors a spell in
/// `events` damaged (judged against the stamina in `before`) and the last body defeated.
pub(super) fn health(
    state: &mut Session,
    events: &[Event],
    mut before: Vec<i16>,
) -> Result<(), RuleError> {
    let kill_party = state.combat.as_ref().map(|combat| combat.kill_party);
    let packet = state
        .combat
        .as_ref()
        .map(|combat| &combat.spell_info)
        .unwrap_or(&state.globals.spell_info);
    let effect = packet[18] as i8;
    let health = &mut state.globals.health_context;
    if let Some(kill_party) = kill_party {
        health.kill_party = kill_party;
    }
    health.effect = effect;
    for event in events {
        match event {
            Event::SpellResolved {
                target,
                damage,
                defeated,
                ..
            } => {
                if let Some(hp) = usize::try_from(*target)
                    .ok()
                    .and_then(|index| before.get_mut(index))
                    && *damage > 0
                    && *hp >= 0
                {
                    health.affected[*target as usize] = -1;
                    *hp = hp.wrapping_sub(*damage);
                }
                if *defeated {
                    health.last_body = *target;
                }
            }
            Event::AttackResolved {
                target,
                defeated: true,
                ..
            } => health.last_body = *target,
            _ => {}
        }
    }
    super::super::store_last_defeated(state)
}
/// The stamina of every warrior of the party.
pub(super) fn party_hp(state: &Session) -> Result<Vec<i16>, RuleError> {
    Ok(state
        .warriors
        .iter()
        .map(Warrior::stamina)
        .collect::<Result<_, _>>()?)
}
/// Gathers the globals a monster's firing or casting reads from the party's state.
pub(super) fn enemy_globals(state: &Session) -> EnemyGlobals {
    EnemyGlobals {
        item_context: state.globals.loaded_item_field32,
        attack_count: state.globals.attack_count,
        packet: state
            .combat
            .as_ref()
            .map(|combat| combat.spell_info.clone())
            .unwrap_or_else(|| state.globals.spell_info.clone()),
        identity: state
            .combat
            .as_ref()
            .map(|combat| combat.cast_identity)
            .unwrap_or(state.globals.cast_identity),
        spell_attempt: 0,
        spell_disabled: state.enemy_spell_disabled,
        buffers: state.globals.loaded_items.clone(),
        display_writers: true,
        char_select_new: state.char_select_new,
        movement_gate: state.bandage_ready,
        neighbor_mask: Some(state.globals.party_track),
        party_condition: Some(state.party_condition.clone()),
        animation_delay: state.command_preferences.animation_delay,
        scenario_tail: party_buffs::scenario_tail(state).ok().flatten(),
        flags: Some(state.flags.clone()),
        land: state.exploration.as_ref().map(|position| Land {
            objects: position.objects.clone(),
            events: position.events.clone(),
        }),
        the_shop: Some(state.globals.the_shop.clone()),
        event_headers: state
            .exploration
            .as_ref()
            .map(|position| position.event_headers.clone()),
        warrior_table: Some(state.globals.warrior_table.clone()),
    }
}
/// Gives the globals back to the party's state after the monster has fired or cast.
pub(super) fn publish(state: &mut Session, globals: EnemyGlobals) {
    state.globals.loaded_item_field32 = globals.item_context;
    state.globals.attack_count = globals.attack_count;
    state.globals.loaded_items = globals.buffers;
    state.bandage_ready = globals.movement_gate;
    if let Some(mask) = globals.neighbor_mask {
        state.globals.party_track = mask;
    }
    if let Some(buffs) = globals.party_condition {
        state.party_condition = buffs;
    }
    if let Some(flags) = globals.flags {
        state.flags = flags;
    }
    if let Some(table) = globals.warrior_table {
        state.globals.warrior_table = table;
    }
    if let Some(the_shop) = globals.the_shop {
        state.globals.the_shop = the_shop;
    }
    if let (Some(headers), Some(position)) = (globals.event_headers, state.exploration.as_mut()) {
        position.event_headers = headers;
    }
    if globals.scenario_tail.is_some()
        && globals.scenario_tail != party_buffs::scenario_tail(state).ok().flatten()
    {
        state.scenario_duration_tail = globals.scenario_tail;
    }
    if let Some(combat) = state.combat.as_ref() {
        // An indexed mode-7 duration can write the difficulty.
        state.difficulty = combat.difficulty;
        state.globals.spell_info = combat.spell_info.clone();
        state.globals.cast_identity = combat.cast_identity;
        state.globals.weapon_traits = combat.weapon_traits;
    }
}
