//! Current record/map views and synchronous service commits for an NPC caller.
use crate::combat::{TacticalMap, runtime::EnemyWork};
use crate::{Content, Event, RuleError, Session};
use centurions_content::{Warrior, combat::MonsterRecord, mission::OpeningMission};

pub(super) fn missing(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.npc_return",
        dependency,
    }
}

pub(super) fn selected(state: &Session) -> Result<usize, RuleError> {
    let index = state
        .combat
        .as_ref()
        .map_or(state.native_work.selected_enemy, |combat| {
            combat.selected_enemy
        });
    if index >= 100 {
        return Err(missing("NPC selector outside represented records"));
    }
    Ok(index)
}
pub(super) fn record(state: &Session, index: usize) -> Result<&MonsterRecord, RuleError> {
    if let Some(combat) = &state.combat {
        combat.story_record(index)
    } else {
        Ok(&state.native_work.npc_backing.slot(index)?.record)
    }
}
pub(super) fn record_mut(
    state: &mut Session,
    index: usize,
) -> Result<&mut MonsterRecord, RuleError> {
    if let Some(combat) = &mut state.combat {
        combat.story_record_mut(index)
    } else {
        Ok(&mut state.native_work.npc_backing.slot_mut(index)?.record)
    }
}
pub(super) fn map(state: &Session) -> Result<&TacticalMap, RuleError> {
    if let Some(combat) = &state.combat {
        Ok(&combat.map)
    } else {
        Ok(&state.native_work.tactical_work.map)
    }
}
pub(super) fn set_map(state: &mut Session, map: TacticalMap) -> Result<(), RuleError> {
    if let Some(combat) = &mut state.combat {
        combat.map = map;
    } else {
        state.native_work.tactical_work.map = map;
    }
    Ok(())
}
pub(super) fn position(state: &Session, index: usize) -> Result<(i16, i16), RuleError> {
    if let Some(combat) = &state.combat {
        return combat.story_position(index);
    }
    let position = state.native_work.npc_backing.slot(index)?.position()?;
    let shift = map(state)?.shift;
    Ok((
        position.0.wrapping_sub(i32::from(shift.0) + 5) as i16,
        position.1.wrapping_sub(i32::from(shift.1) + 5) as i16,
    ))
}
pub(super) fn set_position(
    state: &mut Session,
    index: usize,
    point: (i16, i16),
) -> Result<(), RuleError> {
    let shift = map(state)?.shift;
    let absolute = (
        i32::from(point.0) + i32::from(shift.0) + 5,
        i32::from(point.1) + i32::from(shift.1) + 5,
    );
    if let Some(combat) = &mut state.combat {
        if let Some(position) = combat.enemies.positions.get_mut(index) {
            *position = point;
        } else {
            combat.retained_npcs.slot_mut(index)?.position = Some(absolute);
        }
    } else {
        state.native_work.npc_backing.slot_mut(index)?.position = Some(absolute);
    }
    Ok(())
}
pub(super) fn body(state: &Session) -> Result<i16, RuleError> {
    if let Some(combat) = &state.combat {
        combat
            .order
            .get(combat.cursor)
            .copied()
            .or(match combat.cursor {
                110 => Some(110),
                111 => Some(combat.selected_enemy as i16),
                _ => None,
            })
            // Row 112 is dispatch mode WORD 0x447198, past the initiative work.
            .ok_or_else(|| {
                if combat.cursor == 112 {
                    missing("NPC initiative row 112 reads the dispatch mode WORD")
                } else {
                    missing("NPC initiative row past the initiative work")
                }
            })
    } else {
        state.native_work.initiative_work.current(selected(state)?)
    }
}
pub(super) fn direction(state: &Session) -> Result<(i8, i8), RuleError> {
    let direction = state
        .exploration
        .as_ref()
        .ok_or("NPC movement map missing")?
        .last_move;
    Ok((direction[0], direction[1]))
}
pub(super) fn set_direction(state: &mut Session, direction: (i8, i8)) -> Result<(), RuleError> {
    state
        .exploration
        .as_mut()
        .ok_or("NPC movement map missing")?
        .last_move = [direction.0, direction.1];
    Ok(())
}
pub(super) fn mission<'a>(
    state: &Session,
    content: &'a Content,
) -> Result<&'a OpeningMission, RuleError> {
    state.land(content).ok_or("NPC mission missing".into())
}
pub(super) fn health(
    state: &mut Session,
    events: &[Event],
    mut before: Vec<i16>,
) -> Result<(), RuleError> {
    let unavailable = state.combat.as_ref().map(|combat| combat.unavailable);
    let packet = state
        .combat
        .as_ref()
        .map(|combat| &combat.attack_packet)
        .unwrap_or(&state.native_work.attack_packet);
    let effect = packet[18] as i8;
    let health = &mut state.native_work.health_context;
    if let Some(unavailable) = unavailable {
        health.unavailable = unavailable;
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
    super::super::retain_last_defeated(state)
}
pub(super) fn party_hp(state: &Session) -> Result<Vec<i16>, RuleError> {
    Ok(state
        .warriors
        .iter()
        .map(Warrior::stamina)
        .collect::<Result<_, _>>()?)
}
pub(super) fn enemy_work(state: &Session) -> EnemyWork {
    EnemyWork {
        item_context: state.native_work.loaded_item_field32,
        attack_count: state.native_work.attack_count,
        packet: state
            .combat
            .as_ref()
            .map(|combat| combat.attack_packet.clone())
            .unwrap_or_else(|| state.native_work.attack_packet.clone()),
        identity: state
            .combat
            .as_ref()
            .map(|combat| combat.attack_identity)
            .unwrap_or(state.native_work.attack_identity),
        spell_attempt: 0,
        spell_disabled: state.enemy_spell_disabled,
        buffers: state.native_work.ammunition_work.clone(),
        display_writers: true,
        selected_warrior: state.selected_warrior,
        movement_gate: state.bandage_ready,
        neighbor_mask: Some(state.native_work.party_mask),
        party_buffs: Some(state.party_buffs.clone()),
        animation_delay: state.command_preferences.animation_delay,
        scenario_tail: crate::combat::party_buffs::scenario_tail(state)
            .ok()
            .flatten(),
        flags: Some(state.flags.clone()),
        land: state
            .exploration
            .as_ref()
            .map(|position| crate::combat::map_alias::Land {
                objects: position.objects.clone(),
                events: position.events.clone(),
            }),
        shop_work: Some(state.native_work.shop_work.clone()),
        event_headers: state
            .exploration
            .as_ref()
            .map(|position| position.event_headers.clone()),
        warrior_table: Some(state.native_work.warrior_table.clone()),
    }
}
pub(super) fn publish(state: &mut Session, work: EnemyWork) {
    state.native_work.loaded_item_field32 = work.item_context;
    state.native_work.attack_count = work.attack_count;
    state.native_work.ammunition_work = work.buffers;
    state.bandage_ready = work.movement_gate;
    if let Some(mask) = work.neighbor_mask {
        state.native_work.party_mask = mask;
    }
    if let Some(buffs) = work.party_buffs {
        state.party_buffs = buffs;
    }
    if let Some(flags) = work.flags {
        state.flags = flags;
    }
    if let Some(table) = work.warrior_table {
        state.native_work.warrior_table = table;
    }
    if let Some(shop_work) = work.shop_work {
        state.native_work.shop_work = shop_work;
    }
    if let (Some(headers), Some(position)) = (work.event_headers, state.exploration.as_mut()) {
        position.event_headers = headers;
    }
    if work.scenario_tail.is_some()
        && work.scenario_tail
            != crate::combat::party_buffs::scenario_tail(state)
                .ok()
                .flatten()
    {
        state.scenario_duration_tail = work.scenario_tail;
    }
    if let Some(combat) = state.combat.as_ref() {
        // An indexed mode-7 duration writes WORD442576 through the live combat copy.
        state.difficulty = combat.difficulty;
        state.native_work.attack_packet = combat.attack_packet.clone();
        state.native_work.attack_identity = combat.attack_identity;
        state.native_work.weapon_work = combat.weapon_work;
    }
}
