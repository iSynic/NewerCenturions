//! Retained NPC fire shares preparation, selection and cast with the live caller.
use super::{firearm_selection::RetainedSelection, services::FirearmOutcome, work};
use crate::combat::{
    enemy_weapons,
    runtime::{FireOutcome, TurnFlow},
    spells,
};
use crate::{Content, Event, RuleError, Session};
use centurions_content::records::{AttackPacket, ItemDefinition};

pub(super) fn apply(state: &mut Session, content: &Content) -> Result<FirearmOutcome, RuleError> {
    let selected = work::selected(state)?;
    let record = work::record(state, selected)?.record();
    let scientist = record.faction()? == 0 && record.primary_weapon()? == 1;
    let context = state.native_work.loaded_item_field32;
    // The friendly rifle caller keeps its own range scan and hit roll.
    if scientist {
        return super::scientist::fire(state, content, context);
    }
    let record = work::record(state, selected)?.record();
    let mission = work::mission(state, content)?;
    let tables = mission.combat.as_ref().ok_or("combat content missing")?;
    let weapon_id = record.primary_weapon()?;
    // loadweapon 130d38 caches the definition at abs(id); the shared preparation
    // refuses the signed selected-item copy of a negative weapon.
    let weapon = content
        .items
        .get(weapon_id.unsigned_abs() as usize)
        .ok_or_else(|| work::missing("retained NPC firearm weapon content"))?;
    let ammo_id = enemy_weapons::ammo_id(work::record(state, selected)?, context)?;
    let ammo = content
        .items
        .get(ammo_id.unsigned_abs() as usize)
        .ok_or("enemy ammunition outside content")?;
    state
        .native_work
        .ammunition_work
        .prepare_pair(weapon, ammo)?;
    let mut caster = work::record(state, selected)?.clone();
    // enemy_fire's natural weapon keeps the retained packet and identity.
    let prepared = if weapon_id == 0 {
        let packet = state.native_work.attack_packet.clone();
        caster.record_mut().set_selected_item(0);
        crate::combat::firearm::PreparedFirearm {
            sound: AttackPacket(&packet).impact_sound()?,
            packet,
            slot: 0,
            shots: i16::from(ItemDefinition(weapon).selected_attack_count()),
            rounds: ItemDefinition(weapon).rounds_per_shot()?,
            weight: 0,
            armor_filter: ItemDefinition(weapon).bypasses_hit_roll()?,
        }
    } else {
        enemy_weapons::prepare(&mut caster, weapon, ammo, &tables.attacks)?
    };
    // cast10d4fe branches only on the ray, physical or not (its category-9 tests at
    // 10d890/10d96a only skip dead targets below mode 2); other modes reach
    // spelltargets, which shapes only modes 3 and 4.
    let ray = AttackPacket(&prepared.packet).targeting_mode() == 6;
    *work::record_mut(state, selected)? = caster;
    let (id, identity) = if weapon_id == 0 {
        state.native_work.weapon_work.load_pair(weapon, ammo)?;
        let identity = state.native_work.attack_identity;
        let id = (identity[0] + 1) * 1000 + (identity[1] + 1) * 100 + identity[2] + 1;
        (id, identity)
    } else {
        state.native_work.weapon_work.prepare(weapon, ammo)?;
        state.native_work.attack_packet = prepared.packet.clone();
        let id = ItemDefinition(weapon).spell()?;
        let identity = super::super::spell_identity(id);
        state.native_work.attack_identity = identity;
        state.native_work.attack_count = prepared.shots;
        (id, identity)
    };
    let view = RetainedSelection::new(
        &state.native_work.npc_backing,
        &state.native_work.tactical_work,
        &state.native_work.warrior_table,
    )?;
    let selection = if weapon_id == 0 || AttackPacket(&prepared.packet).defense_category() != 9 {
        spells::select_retained_from_view(
            &view,
            &state.warriors,
            mission,
            &mission.areas,
            selected,
            &prepared,
            &mut state.random,
        )?
    } else {
        spells::select_firearm_from_view(
            &view,
            &state.warriors,
            mission,
            &mission.areas,
            selected,
            &prepared,
            &mut state.random,
        )?
    };
    state.native_work.attack_count = selection.count;
    // combat 114b7e calls cleartarget before the decision writes its triples.
    let targets = &mut state.native_work.target_work;
    targets.clear();
    targets.write(&selection.targets);
    if !selection.cast {
        return returned(state, vec![], FireOutcome::Failed);
    }
    state.native_work.round_work.inactive_rounds = Some(0);
    *work::record_mut(state, selected)? = selection.caster;
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
    state.native_work.attack_count = if AttackPacket(&prepared.packet).plays_impact_when_wounded() {
        1
    } else {
        state.native_work.attack_count.max(1)
    };
    // combat1150a4 stores Rand(4)-1 in WORD44331a for a byte-7 packet.
    let rotation = if AttackPacket(&prepared.packet).has_randomized_animation() {
        state.random.bound(4) - 1
    } else {
        0
    };
    let mut first = true;
    let mut events = vec![];
    let total = selection.targets.len();
    let mut flights = vec![];
    // combat 11509e calls showspellinfo before cast.
    let info = super::global_spells::spell_info(state);
    for (position, target) in selection.targets.into_iter().enumerate() {
        state.native_work.target_work.enter(position, total);
        if !alive(state, target.first)? {
            continue;
        }
        let jitter = spells::flight_jitter(&prepared.packet, &mut state.random);
        flights.push((
            events.len(),
            super::super::movement_return::retained_flight(
                state,
                selected as i16 + 10,
                &target,
                &prepared.packet,
                (total, jitter),
            ),
        ));
        events.push(Event::Sound {
            resource_id: AttackPacket(&prepared.packet).launch_sound()?,
        });
        if ray {
            let invocation = (id, identity[1], state.native_work.attack_count);
            let (flow, stop) = super::super::movement_return::retained_ray(
                state,
                content,
                selected + 10,
                &target,
                &prepared.packet,
                invocation,
                &mut events,
            )?;
            spells::stop_flight(&mut flights, stop);
            if flow == TurnFlow::Terminated {
                crate::event::Flight::attach(&mut events, &flights);
                crate::event::SpellInfo::attach(&mut events, 0, info);
                return Ok(FirearmOutcome {
                    events,
                    outcome: FireOutcome::Physical,
                    flow,
                });
            }
            state.clear_spell_targets();
            continue;
        }
        let field = super::super::movement_return::RetainedFieldOrigin {
            origin: if first {
                (target.first, target.second)
            } else {
                (-1, -1)
            },
            id,
            rotation,
            collision: state.native_work.tactical_work.field_used[62] != 0,
        };
        first = false;
        let mask = super::super::movement_return::retained_field_target_mask(
            state,
            content,
            &target,
            &prepared.packet,
            field,
        )?;
        state.publish_spell_targets(&mask);
        let used = &state.native_work.tactical_work.field_used;
        let index = usize::from(used[62]);
        if index >= used.len() {
            return Err(work::missing(
                "retained NPC firearm usage index outside represented bytes",
            ));
        }
        let mut usage = spells::FieldUsage {
            slot: index.saturating_sub(1),
            used: used.clone(),
        };
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
            &mut events,
        )?;
        if flow == TurnFlow::Terminated {
            crate::event::Flight::attach(&mut events, &flights);
            crate::event::SpellInfo::attach(&mut events, 0, info);
            return Ok(FirearmOutcome {
                events,
                outcome: FireOutcome::Physical,
                flow,
            });
        }
        state.set_ready(0);
        state.native_work.target_work.resolved(position as i16);
        // cast10e642 clears all110 WORDs after the target returns.
        state.clear_spell_targets();
        if state.native_work.target_work.loop_ended(position) {
            break;
        }
    }
    // cast ends with cleartarget (10e678).
    crate::event::Flight::attach(&mut events, &flights);
    crate::event::SpellInfo::attach(&mut events, 0, info);
    state.native_work.target_work.leave();
    returned(state, events, FireOutcome::Physical)
}

fn alive(state: &Session, body: i16) -> Result<bool, RuleError> {
    if (0..6).contains(&body) {
        Ok(state
            .warriors
            .get(body as usize)
            .is_some_and(|warrior| warrior.record().active_flag() != 0))
    } else if body >= 10 {
        let record = work::record(state, (body - 10) as usize)?;
        Ok(record.stamina()? > 0 && !record.record().removed())
    } else {
        Ok(false)
    }
}

fn returned(
    state: &mut Session,
    events: Vec<Event>,
    outcome: FireOutcome,
) -> Result<FirearmOutcome, RuleError> {
    if let Some(item) = &state.native_work.ammunition_work.item {
        state.native_work.loaded_item_field32 = ItemDefinition(item).retained_field32()?;
    }
    Ok(FirearmOutcome {
        events,
        outcome,
        flow: TurnFlow::Returned,
    })
}
