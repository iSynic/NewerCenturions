//! Global NPC casts use current retained records, without a tactical arena.
use super::{services::SpellOutcome, work};
use crate::combat::{runtime::TurnFlow, spells};
use crate::{Content, RuleError, Session};
use centurions_content::records::{AttackPacket, ItemDefinition};

pub(super) fn apply(state: &mut Session, content: &Content) -> Result<SpellOutcome, RuleError> {
    let mission = work::mission(state, content)?;
    let tables = mission.combat.as_ref().ok_or("spell content missing")?;
    let mut outcome = SpellOutcome {
        events: vec![],
        cast: false,
        flow: TurnFlow::Returned,
    };
    let mut attempt = 0i8;
    loop {
        let selected = work::selected(state)?;
        let caster = work::record(state, selected)?.clone();
        if attempt >= caster.record().spell_attempts() {
            break;
        }
        let Some((id, packet)) =
            spells::select_packet(&caster, &tables.attacks, &mut state.random)?
        else {
            break;
        };
        let mode = AttackPacket(&packet).targeting_mode();
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
        state.native_work.attack_packet = packet;
        state.native_work.attack_identity = identity;
        state.clear_spell_targets();
        let selected = work::selected(state)?;
        let side = work::record(state, selected)?.record().faction()?;
        let active = state.native_work.npc_backing.active_records()?;
        let mask = spells::global_target_mask(&state.warriors, &active, side, mode)?;
        state.publish_spell_targets(&mask);
        state.set_ready(1);
        let item = work::record(state, work::selected(state)?)?
            .record()
            .selected_item()?;
        super::super::enemy_spells::refresh_selected_item_alias(
            item,
            &content.items,
            &mut state.native_work.ammunition_work,
            &mut state.native_work.loaded_item_field32,
        )?;
        if mode != 12 {
            let selected = work::selected(state)?;
            work::record_mut(state, selected)?
                .record_mut()
                .set_reaction_ready(false);
        }
        let used = &state.native_work.tactical_work.field_used;
        let index = usize::from(used[62]);
        if index >= used.len() {
            return Err(work::missing(
                "retained NPC spell usage index outside represented bytes",
            ));
        }
        let mut usage = spells::FieldUsage {
            slot: index.saturating_sub(1),
            used: used.clone(),
        };
        let count = state.native_work.attack_count;
        // combat 11495e, 114a4c and 114b50 call showspellinfo before resolvespell.
        let info = spell_info(state);
        let start = outcome.events.len();
        let flow = super::super::retained_spell::apply(
            state,
            content,
            &mask,
            &mut usage,
            super::super::retained_spell::Invocation {
                id,
                level: identity[1],
                count,
                // Global modes 9, 10 and 12 select no target; the arms read an
                // earlier writer's WORD443722/443724.
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
    if let Some(item) = &state.native_work.ammunition_work.item {
        state.native_work.loaded_item_field32 = ItemDefinition(item).retained_field32()?;
    }
    Ok(outcome)
}

/// The enemy decision's targeted cast over retained records; None when nothing is in range.
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
    state.native_work.attack_packet = packet.to_vec();
    state.native_work.attack_identity = identity;
    let view = super::firearm_selection::RetainedSelection::new(
        &state.native_work.npc_backing,
        &state.native_work.tactical_work,
        &state.native_work.warrior_table,
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
    state.native_work.attack_count = selection.count;
    // combat 114b7e calls cleartarget before the decision writes its triples.
    let targets = &mut state.native_work.target_work;
    targets.clear();
    targets.write(&selection.targets);
    if selection.targets.is_empty() {
        return Ok(None);
    }
    *work::record_mut(state, selected)? = selection.caster;
    state.set_ready(1);
    let item = work::record(state, selected)?.record().selected_item()?;
    super::super::enemy_spells::refresh_selected_item_alias(
        item,
        &content.items,
        &mut state.native_work.ammunition_work,
        &mut state.native_work.loaded_item_field32,
    )?;
    state.native_work.attack_count = state.native_work.attack_count.max(1);
    let mut first = true;
    let total = selection.targets.len();
    let mut flights = vec![];
    // combat 11509e calls showspellinfo before cast.
    let info = spell_info(state);
    let start = outcome.events.len();
    for (position, target) in selection.targets.into_iter().enumerate() {
        state.native_work.target_work.enter(position, total);
        let jitter = spells::flight_jitter(packet, &mut state.random);
        flights.push((
            outcome.events.len(),
            super::super::movement_return::retained_flight(
                state,
                selected as i16 + 10,
                &target,
                packet,
                (total, jitter),
            ),
        ));
        outcome.events.push(crate::Event::Sound {
            resource_id: AttackPacket(packet).launch_sound()?,
        });
        // The NPC decision calls cast10d4fe, whose ray resolves crossed bodies with Ready kept.
        if AttackPacket(packet).targeting_mode() == 6 {
            let invocation = (id, identity[1], state.native_work.attack_count);
            let (flow, stop) = super::super::movement_return::retained_ray(
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
        let field = super::super::movement_return::RetainedFieldOrigin {
            origin: if first {
                (target.first, target.second)
            } else {
                (-1, -1)
            },
            id,
            rotation: i16::from(selection.rotation),
            collision: state.native_work.tactical_work.field_used[62] != 0,
        };
        first = false;
        let mask = super::super::movement_return::retained_field_target_mask(
            state, content, &target, packet, field,
        )?;
        state.publish_spell_targets(&mask);
        let used = &state.native_work.tactical_work.field_used;
        let index = usize::from(used[62]);
        let mut usage = spells::FieldUsage {
            slot: index.saturating_sub(1),
            used: used.clone(),
        };
        if AttackPacket(packet).targeting_mode() == 7 {
            global_durations(state, packet, &mut outcome.events)?;
            state.native_work.target_work.resolved(position as i16);
            continue;
        }
        state.set_ready(1);
        let count = state.native_work.attack_count;
        let flow = super::super::retained_spell::apply(
            state,
            content,
            &mask,
            &mut usage,
            super::super::retained_spell::Invocation {
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
        state.set_ready(0);
        state.native_work.target_work.resolved(position as i16);
        if state.native_work.target_work.loop_ended(position) {
            break;
        }
    }
    // cast ends with cleartarget (10e678).
    crate::event::Flight::attach(&mut outcome.events, &flights);
    crate::event::SpellInfo::attach(&mut outcome.events, start, info);
    state.native_work.target_work.leave();
    Ok(Some(TurnFlow::Returned))
}

/// showspellinfo's drawing for the retained NPC's packet in native work.
pub(super) fn spell_info(state: &Session) -> crate::event::SpellInfo {
    let work = &state.native_work;
    super::super::enemy_spells::npc_spell_info(
        &work.attack_packet,
        work.attack_identity,
        work.attack_count,
        &work.ammunition_work,
    )
}

/// resolvespell's mode7 arm for a retained NPC caster: the session's party durations.
fn global_durations(
    state: &mut Session,
    packet: &[u8],
    events: &mut Vec<crate::Event>,
) -> Result<(), RuleError> {
    state.set_ready(0);
    state.native_work.round_work.inactive_rounds = Some(0);
    let attack = AttackPacket(packet);
    let count = state.native_work.attack_count;
    let (minimum, maximum) = attack.duration_bounds();
    let mut duration = state.random.range(minimum, maximum);
    let (minimum, maximum) = attack.additional_duration_bounds();
    if minimum != 0 {
        for _ in 0..count {
            duration = duration.wrapping_add(state.random.range(minimum, maximum));
        }
    }
    match attack.effect()?.wrapping_abs() {
        50 => {
            events.push(crate::Event::Sound { resource_id: 601 });
            crate::combat::party_buffs::extend_from_count(&mut state.party_buffs, count)?;
        }
        effect @ 0..=9 => {
            crate::combat::party_buffs::extend(&mut state.party_buffs, effect, duration)?
        }
        110 => crate::combat::party_buffs::extend_scenario_tail(state, duration)?,
        effect => {
            if !crate::combat::party_buffs::extend_indexed(state, effect, duration)? {
                return Err(work::missing(
                    "NPC indexed duration outside the party durations",
                ));
            }
        }
    }
    Ok(())
}
