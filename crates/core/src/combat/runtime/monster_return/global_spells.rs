//! Monster spells cast after the battle has ended.
use super::{services::SpellOutcome, turn_state};
use crate::combat::party_buffs;
use crate::combat::{runtime::TurnFlow, spells};
use crate::{Content, RuleError, Session};
use centurions_content::records::{ItemDefinition, SpellInfoRecord};

/// A monster casts after the battle has ended as it does during it: up to its number of magic
/// attacks, each picks a power at random. Targeting modes 9, 10 and 12 take a mask of every body
/// of a side or of all bodies; the other modes select their own targets.
///
/// Original: combat
pub(super) fn apply(state: &mut Session, content: &Content) -> Result<SpellOutcome, RuleError> {
    let mission = turn_state::mission(state, content)?;
    let tables = mission.combat.as_ref().ok_or("spell content missing")?;
    let mut outcome = SpellOutcome {
        events: vec![],
        cast: false,
        flow: TurnFlow::Returned,
    };
    let mut attempt = 0i8;
    loop {
        let selected = turn_state::selected(state)?;
        let caster = turn_state::record(state, selected)?.clone();
        if attempt >= caster.record().magic_attack_count() {
            break;
        }
        let Some((id, packet)) =
            spells::select_packet(&caster, &tables.attacks, &mut state.random)?
        else {
            break;
        };
        let mode = SpellInfoRecord(&packet).target_type();
        if !matches!(mode, 9 | 10 | 12) {
            match targeted(state, content, mission, selected, id, &packet, &mut outcome)? {
                Some(TurnFlow::Terminated) => {
                    outcome.flow = TurnFlow::Terminated;
                    return Ok(outcome);
                }
                Some(_) => {
                    outcome.cast = true;
                    attempt = attempt.wrapping_add(1);
                    continue;
                }
                None => break,
            }
        }
        let identity = super::super::spell_identity(id);
        state.globals.spell_info = packet;
        state.globals.cast_identity = identity;
        state.clear_spell_targets();
        let selected = turn_state::selected(state)?;
        let side = turn_state::record(state, selected)?.record().faction()?;
        let active = state.globals.monsters.active_records()?;
        let mask = spells::global_target_mask(&state.warriors, &active, side, mode)?;
        state.publish_spell_targets(&mask);
        state.set_in_spell(1);
        let item = turn_state::record(state, turn_state::selected(state)?)?
            .record()
            .weapon()?;
        super::super::enemy_spells::refresh_selected_item_alias(
            item,
            &content.items,
            &mut state.globals.loaded_items,
            &mut state.globals.loaded_item_field32,
        )?;
        if mode != 12 {
            let selected = turn_state::selected(state)?;
            turn_state::record_mut(state, selected)?
                .record_mut()
                .set_reaction_ready(false);
        }
        let used = &state.globals.battlefield.collide_check;
        let index = usize::from(used[62]);
        if index >= used.len() {
            return Err(turn_state::missing(
                "retained NPC spell usage index outside represented bytes",
            ));
        }
        let mut usage = spells::FieldUsage {
            slot: index.saturating_sub(1),
            used: used.clone(),
        };
        let count = state.globals.attack_count;
        let info = spell_info(state);
        let start = outcome.events.len();
        let flow = super::super::out_of_combat_spell::apply(
            state,
            content,
            &mask,
            &mut usage,
            super::super::out_of_combat_spell::Invocation {
                id,
                level: identity[1],
                count,
                // Global modes 9, 10 and 12 choose no target; movement and summons
                // read whatever target an earlier spell left.
                collision: index != 0,
                current: None,
            },
            &mut outcome.events,
        )?;
        crate::event::SpellInfo::attach(&mut outcome.events, start, info);
        if flow == TurnFlow::Terminated {
            outcome.flow = flow;
            return Ok(outcome);
        }
        outcome.cast = true;
        attempt = attempt.wrapping_add(1);
    }
    if let Some(item) = &state.globals.loaded_items.item {
        state.globals.loaded_item_field32 = ItemDefinition(item).ammunition_caliber()?;
    }
    Ok(outcome)
}

/// A monster's targeted spell; None when no target is selected.
fn targeted(
    state: &mut Session,
    content: &Content,
    mission: &centurions_content::mission::OpeningMission,
    selected: usize,
    id: i16,
    packet: &[u8],
    outcome: &mut SpellOutcome,
) -> Result<Option<TurnFlow>, RuleError> {
    let identity = super::super::spell_identity(id);
    state.globals.spell_info = packet.to_vec();
    state.globals.cast_identity = identity;
    let view = super::firearm_selection::OutOfCombatSelection::new(
        &state.globals.monsters,
        &state.globals.battlefield,
        &state.globals.warrior_table,
    )?;
    let selection = spells::select_spell_from_view(
        &view,
        &state.warriors,
        mission,
        &mission.areas,
        selected,
        packet,
        &mut state.random,
    )?;
    state.globals.attack_count = selection.count;
    let targets = &mut state.globals.target_picks;
    targets.clear();
    targets.write(&selection.targets);
    if selection.targets.is_empty() {
        return Ok(None);
    }
    *turn_state::record_mut(state, selected)? = selection.caster;
    state.set_in_spell(1);
    let item = turn_state::record(state, selected)?.record().weapon()?;
    super::super::enemy_spells::refresh_selected_item_alias(
        item,
        &content.items,
        &mut state.globals.loaded_items,
        &mut state.globals.loaded_item_field32,
    )?;
    state.globals.attack_count = state.globals.attack_count.max(1);
    let mut first = true;
    let total = selection.targets.len();
    let mut flights = vec![];
    let info = spell_info(state);
    let start = outcome.events.len();
    for (position, target) in selection.targets.into_iter().enumerate() {
        state.globals.target_picks.enter(position, total);
        let jitter = spells::flight_jitter(packet, &mut state.random);
        flights.push((
            outcome.events.len(),
            super::super::movement_return::out_of_combat_flight(
                state,
                selected as i16 + 10,
                &target,
                packet,
                (total, jitter),
            ),
        ));
        outcome.events.push(crate::Event::Sound {
            resource_id: SpellInfoRecord(packet).launch_sound()?,
        });
        // A ray resolves the bodies it crosses while in_spell stays set.
        if SpellInfoRecord(packet).is_ray() {
            let invocation = (id, identity[1], state.globals.attack_count);
            let (flow, stop) = super::super::movement_return::out_of_combat_ray(
                state,
                content,
                selected + 10,
                &target,
                packet,
                invocation,
                &mut outcome.events,
            )?;
            spells::stop_flight(&mut flights, stop);
            if flow == TurnFlow::Terminated {
                crate::event::Flight::attach(&mut outcome.events, &flights);
                crate::event::SpellInfo::attach(&mut outcome.events, start, info);
                return Ok(Some(flow));
            }
            continue;
        }
        let field = super::super::movement_return::OutOfCombatFieldOrigin {
            origin: if first {
                (target.first, target.second)
            } else {
                (-1, -1)
            },
            id,
            rotation: i16::from(selection.rotation),
            collision: state.globals.battlefield.collide_check[62] != 0,
        };
        first = false;
        let mask = super::super::movement_return::out_of_combat_field_target_mask(
            state, content, &target, packet, field,
        )?;
        state.publish_spell_targets(&mask);
        let used = &state.globals.battlefield.collide_check;
        let index = usize::from(used[62]);
        let mut usage = spells::FieldUsage {
            slot: index.saturating_sub(1),
            used: used.clone(),
        };
        if SpellInfoRecord(packet).is_party_duration() {
            global_durations(state, packet, &mut outcome.events)?;
            state.globals.target_picks.resolved(position as i16);
            continue;
        }
        state.set_in_spell(1);
        let count = state.globals.attack_count;
        let flow = super::super::out_of_combat_spell::apply(
            state,
            content,
            &mask,
            &mut usage,
            super::super::out_of_combat_spell::Invocation {
                id,
                level: identity[1],
                count,
                collision: index != 0,
                current: Some((target.first, target.second)),
            },
            &mut outcome.events,
        )?;
        if flow == TurnFlow::Terminated {
            crate::event::Flight::attach(&mut outcome.events, &flights);
            crate::event::SpellInfo::attach(&mut outcome.events, start, info);
            return Ok(Some(flow));
        }
        state.set_in_spell(0);
        state.globals.target_picks.resolved(position as i16);
        if state.globals.target_picks.loop_ended(position) {
            break;
        }
    }
    crate::event::Flight::attach(&mut outcome.events, &flights);
    crate::event::SpellInfo::attach(&mut outcome.events, start, info);
    state.globals.target_picks.leave();
    Ok(Some(TurnFlow::Returned))
}

/// The spell information shown for the monster's packet.
///
/// Original: showspellinfo
pub(super) fn spell_info(state: &Session) -> crate::event::SpellInfo {
    let globals = &state.globals;
    super::super::enemy_spells::monster_spell_info(
        &globals.spell_info,
        globals.cast_identity,
        globals.attack_count,
        &globals.loaded_items,
    )
}

/// A monster's global (mode 7) spell writes the party durations.
///
/// Original: resolvespell
fn global_durations(
    state: &mut Session,
    packet: &[u8],
    events: &mut Vec<crate::Event>,
) -> Result<(), RuleError> {
    state.set_in_spell(0);
    state.globals.round_scripts.blank_round = Some(0);
    let attack = SpellInfoRecord(packet);
    let count = state.globals.attack_count;
    let (minimum, maximum) = attack.duration_range();
    let mut duration = state.random.range(minimum, maximum);
    let (minimum, maximum) = attack.power_duration_range();
    if minimum != 0 {
        for _ in 0..count {
            duration = duration.wrapping_add(state.random.range(minimum, maximum));
        }
    }
    match attack.effect()?.wrapping_abs() {
        50 => {
            events.push(crate::Event::Sound { resource_id: 601 });
            party_buffs::extend_from_count(&mut state.party_condition, count)?;
        }
        effect @ 0..=9 => party_buffs::extend(&mut state.party_condition, effect, duration)?,
        110 => party_buffs::extend_scenario_tail(state, duration)?,
        effect => {
            if !party_buffs::extend_indexed(state, effect, duration)? {
                return Err(turn_state::missing(
                    "NPC indexed duration outside the party durations",
                ));
            }
        }
    }
    Ok(())
}
