//! Monsters firing after the battle has ended, as they do during it.
use super::{firearm_selection::OutOfCombatSelection, services::FirearmOutcome, turn_state};
use crate::combat::firearm::PreparedFirearm;
use crate::combat::spells::spell_descriptor;
use crate::combat::{
    enemy_weapons,
    runtime::{FireOutcome, TurnFlow},
    spells,
};
use crate::{Content, Event, RuleError, Session};
use centurions_content::records::{ItemDefinition, SpellInfoRecord};

/// A monster fires its weapon after the battle has ended as it does during it; the friendly
/// scientist fires its rifle instead.
///
/// Original: combat
pub(super) fn apply(state: &mut Session, content: &Content) -> Result<FirearmOutcome, RuleError> {
    let selected = turn_state::selected(state)?;
    let record = turn_state::record(state, selected)?.record();
    let scientist = record.faction()? == 0 && record.primary_weapon()? == 1;
    let context = state.globals.loaded_item_field32;
    // The friendly rifle caller keeps its own range scan and hit roll.
    if scientist {
        return super::scientist::fire(state, content, context);
    }
    let LoadedFirearm {
        mission,
        weapon_id,
        prepared,
        id,
        identity,
    } = load(state, content, selected, context)?;
    // A ray walks, physical or not; weapon fire only skips dead targets below
    // mode 2. Other modes mark targets, and only modes 3 and 4 have a shape.
    let ray = SpellInfoRecord(&prepared.packet).is_ray();
    let view = OutOfCombatSelection::new(
        &state.globals.monsters,
        &state.globals.battlefield,
        &state.globals.warrior_table,
    )?;
    let selection = if weapon_id == 0 || SpellInfoRecord(&prepared.packet).defense_category() != 9 {
        spells::select_spell_info_from_view(
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
    state.globals.attack_count = selection.count;
    let targets = &mut state.globals.target_picks;
    targets.clear();
    targets.write(&selection.targets);
    if !selection.cast {
        return returned(state, vec![], FireOutcome::Failed);
    }
    state.globals.round_scripts.blank_round = Some(0);
    *turn_state::record_mut(state, selected)? = selection.caster;
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
    state.globals.attack_count = if SpellInfoRecord(&prepared.packet).plays_impact_when_wounded() {
        1
    } else {
        state.globals.attack_count.max(1)
    };
    // A rotating packet turns by Rand(4) - 1.
    let rotation = if SpellInfoRecord(&prepared.packet).can_rotate() {
        state.random.bound(4) - 1
    } else {
        0
    };
    let mut first = true;
    let mut events = vec![];
    let total = selection.targets.len();
    let mut flights = vec![];
    let info = super::global_spells::spell_info(state);
    for (position, target) in selection.targets.into_iter().enumerate() {
        state.globals.target_picks.enter(position, total);
        if !alive(state, target.first)? {
            continue;
        }
        let jitter = spells::flight_jitter(&prepared.packet, &mut state.random);
        flights.push((
            events.len(),
            super::super::movement_return::out_of_combat_flight(
                state,
                selected as i16 + 10,
                &target,
                &prepared.packet,
                (total, jitter),
            ),
        ));
        events.push(Event::Sound {
            resource_id: SpellInfoRecord(&prepared.packet).launch_sound()?,
        });
        if ray {
            let invocation = (id, identity[1], state.globals.attack_count);
            let (flow, stop) = super::super::movement_return::out_of_combat_ray(
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
                return Ok(terminated(events, &flights, info, flow));
            }
            state.clear_spell_targets();
            continue;
        }
        let field = super::super::movement_return::OutOfCombatFieldOrigin {
            origin: if first {
                (target.first, target.second)
            } else {
                (-1, -1)
            },
            id,
            rotation,
            collision: state.globals.battlefield.collide_check[62] != 0,
        };
        first = false;
        let mask = super::super::movement_return::out_of_combat_field_target_mask(
            state,
            content,
            &target,
            &prepared.packet,
            field,
        )?;
        state.publish_spell_targets(&mask);
        let used = &state.globals.battlefield.collide_check;
        let index = usize::from(used[62]);
        if index >= used.len() {
            return Err(turn_state::missing(
                "retained NPC firearm usage index outside represented bytes",
            ));
        }
        let mut usage = spells::FieldUsage {
            slot: index.saturating_sub(1),
            used: used.clone(),
        };
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
            &mut events,
        )?;
        if flow == TurnFlow::Terminated {
            return Ok(terminated(events, &flights, info, flow));
        }
        state.set_in_spell(0);
        state.globals.target_picks.resolved(position as i16);
        // All 110 target marks clear after each target.
        state.clear_spell_targets();
        if state.globals.target_picks.loop_ended(position) {
            break;
        }
    }
    crate::event::Flight::attach(&mut events, &flights);
    crate::event::SpellInfo::attach(&mut events, 0, info);
    state.globals.target_picks.leave();
    returned(state, events, FireOutcome::Physical)
}

/// The monster's primary weapon and ammunition, loaded as during the battle: the
/// work pair, the prepared shot and the descriptor and identity the cast reads.
struct LoadedFirearm<'a> {
    mission: &'a centurions_content::mission::OpeningMission,
    weapon_id: i16,
    prepared: PreparedFirearm,
    id: i16,
    identity: [i16; 3],
}

/// Loads the monster's weapon and prepares its shot.
fn load<'a>(
    state: &mut Session,
    content: &'a Content,
    selected: usize,
    context: i16,
) -> Result<LoadedFirearm<'a>, RuleError> {
    let record = turn_state::record(state, selected)?.record();
    let mission = turn_state::mission(state, content)?;
    let tables = mission.combat.as_ref().ok_or("combat content missing")?;
    let weapon_id = record.primary_weapon()?;
    // A negative weapon id loads the definition of its absolute value; preparing
    // the shot refuses the negative id itself.
    let weapon = content
        .items
        .get(weapon_id.unsigned_abs() as usize)
        .ok_or_else(|| turn_state::missing("retained NPC firearm weapon content"))?;
    let ammo_id = enemy_weapons::ammo_id(turn_state::record(state, selected)?, context)?;
    let ammo = content
        .items
        .get(ammo_id.unsigned_abs() as usize)
        .ok_or("enemy ammunition outside content")?;
    state.globals.loaded_items.prepare_pair(weapon, ammo)?;
    let mut caster = turn_state::record(state, selected)?.clone();
    // A natural weapon keeps the previous packet and identity.
    let prepared = if weapon_id == 0 {
        let packet = state.globals.spell_info.clone();
        caster.record_mut().set_weapon(0);
        PreparedFirearm {
            sound: SpellInfoRecord(&packet).impact_sound()?,
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
    *turn_state::record_mut(state, selected)? = caster;
    let (id, identity) = if weapon_id == 0 {
        state.globals.weapon_traits.load_pair(weapon, ammo)?;
        let identity = state.globals.cast_identity;
        let id = spell_descriptor(identity);
        (id, identity)
    } else {
        state.globals.weapon_traits.prepare(weapon, ammo)?;
        state.globals.spell_info = prepared.packet.clone();
        let id = ItemDefinition(weapon).spell()?;
        let identity = super::super::spell_identity(id);
        state.globals.cast_identity = identity;
        state.globals.attack_count = prepared.shots;
        (id, identity)
    };
    Ok(LoadedFirearm {
        mission,
        weapon_id,
        prepared,
        id,
        identity,
    })
}

/// A cast that terminated combat returns its events with the flights and spell info.
fn terminated(
    mut events: Vec<Event>,
    flights: &[(usize, Option<crate::event::Flight>)],
    info: crate::event::SpellInfo,
    flow: TurnFlow,
) -> FirearmOutcome {
    crate::event::Flight::attach(&mut events, flights);
    crate::event::SpellInfo::attach(&mut events, 0, info);
    FirearmOutcome {
        events,
        outcome: FireOutcome::Physical,
        flow,
    }
}

/// Whether a body can still be hit: a warrior in the battle, or a monster with stamina left that
/// has not been removed.
fn alive(state: &Session, body: i16) -> Result<bool, RuleError> {
    if (0..6).contains(&body) {
        Ok(state
            .warriors
            .get(body as usize)
            .is_some_and(|warrior| warrior.record().in_battle() != 0))
    } else if body >= 10 {
        let record = turn_state::record(state, (body - 10) as usize)?;
        Ok(record.stamina()? > 0 && !record.record().removed())
    } else {
        Ok(false)
    }
}

/// Ends the shot: the loaded weapon's caliber becomes the item context for the next shot.
fn returned(
    state: &mut Session,
    events: Vec<Event>,
    outcome: FireOutcome,
) -> Result<FirearmOutcome, RuleError> {
    if let Some(item) = &state.globals.loaded_items.item {
        state.globals.loaded_item_field32 = ItemDefinition(item).ammunition_caliber()?;
    }
    Ok(FirearmOutcome {
        events,
        outcome,
        flow: TurnFlow::Returned,
    })
}
