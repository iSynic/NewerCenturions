//! Spells that resolve after the battle has ended.
use super::TurnFlow;
use crate::combat::party_buffs;
use crate::combat::warrior_alias::Owners;
use crate::combat::{outdoor_health, spells};
use crate::{Content, Event, RuleError, Session};
use centurions_content::records::{CharacterStatisticsMut, SpellInfoRecordMut, Statistic};

/// The cast being resolved: the spell and its level, its count (the attack count or a field's
/// power), whether a field collision caused it and the target a summon reads.
pub(super) struct Invocation {
    pub id: i16,
    pub level: i16,
    pub count: i16,
    pub collision: bool,
    /// The cast's current target, which a summon reads.
    pub current: Option<(i16, i16)>,
}

/// Resolves a spell after the battle has ended on the bodies the mask marks. Returns Terminated
/// when the party is lost.
///
/// Original: resolvespell
pub(super) fn apply(
    state: &mut Session,
    content: &Content,
    mask: &[i16; 110],
    usage: &mut spells::FieldUsage,
    invocation: Invocation,
    events: &mut Vec<Event>,
) -> Result<TurnFlow, RuleError> {
    let mission = state.land(content).ok_or("retained spell map missing")?;
    let packet = state.globals.spell_info.clone();
    let enemies = (0..100)
        .map(|index| {
            state
                .globals
                .monsters
                .slot(index)
                .map(|slot| slot.record.clone())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let current = state
        .globals
        .battle_order
        .current(state.globals.monster_up)?;
    let position = state
        .exploration
        .as_ref()
        .ok_or("retained spell position missing")?;
    let monster = state.globals.monster_pick.known();
    let outcome = spells::out_of_combat::resolve(
        &state.warriors,
        &enemies,
        mask,
        usage,
        &spells::out_of_combat::Context {
            spell: spells::DirectSpell {
                packet: &packet,
                level: invocation.level,
                count: invocation.count,
                armor_filter: state.globals.weapon_traits.armor_filter(),
                weapon_sound: state.globals.weapon_traits.sound(),
            },
            id: invocation.id,
            current: Some(current),
            memory: spells::RestrictionMemoryView::ordinary_startup(
                &mission.item_cache,
                spells::loaded_item_restriction(&state.globals.loaded_items)?,
            )
            .with_alias_owners(
                Owners::new(
                    state.globals.loaded_items.item.as_deref(),
                    &state.globals.the_shop,
                    Some(&position.event_headers),
                )
                .with_backup(&state.globals.warrior_table),
            )
            .with_tables(mission)
            .with_definitions(&content.items)
            // The caster is still the acting warrior (charup).
            .with_selected_party_caster(state.char_select_new as i16)
            .with_resist_caster(
                Some(current),
                (state.char_select_new, state.globals.monster_up),
                &state.warriors,
                &enemies,
            )
            .with_current_accuracy(
                Some(current),
                (&state.warriors, &enemies, monster.as_deref()),
                state.difficulty,
            ),
            location: (position.x as i16, position.y as i16),
            animation_delay: state.command_preferences.animation_delay,
            outdoor_health: outdoor_health::View::new(
                content,
                state.mission.as_deref(),
                state.scenario_level,
                state.globals.loaded_items.item.as_deref(),
                &state.globals.the_shop,
                Some(&position.event_headers),
                (&state.warriors, &state.globals.warrior_table),
            ),
            collision: invocation.collision,
            movement_gate: state.bandage_ready,
        },
        &state.globals.health_context,
        &mut state.random,
    )?;
    if let Some((effect, duration)) = outcome.party_effect {
        match effect {
            50 => {
                events.push(Event::Sound { resource_id: 601 });
                party_buffs::extend_from_count(&mut state.party_condition, invocation.count)?;
            }
            110 => party_buffs::extend_scenario_tail(state, duration)?,
            _ => party_buffs::extend(&mut state.party_condition, effect, duration)?,
        }
    }
    if outcome.monster_pick_written {
        state.invalidate_monster_pick();
    }
    // Outside combat, effect 68's fatigue of 1 becomes 4.
    if outcome.bodies.fatigue_reset
        && let Some(position) = state.exploration.as_mut()
    {
        position.fatigue = 4;
    }
    state.store_table_records(outcome.bodies.warriors)?;
    state.commit_writes(&outcome.bodies.alias_writes)?;
    state.globals.monsters.store_records(outcome.bodies.enemies);
    state
        .globals
        .kill_counts
        .add_num_enemy(outcome.bodies.total_delta);
    state
        .globals
        .battlefield
        .collide_check
        .clone_from(&usage.used);
    let mut info = SpellInfoRecordMut(&mut state.globals.spell_info);
    info.set_health_effect(outcome.health.effect as u8);
    // Effect66 stores its drawn selector in the packet's area byte.
    if let Some(shape) = outcome.bodies.area_shape {
        info.set_area_size(shape);
    }
    state.globals.health_context = outcome.health;
    if outcome.clear_departure {
        state.globals.neighbors.departure = 0;
    }
    if (0..6).contains(&current) {
        for credit in &outcome.monster_credit {
            let mut statistics =
                CharacterStatisticsMut(&mut state.warriors[state.char_select_new].bytes);
            statistics.add(Statistic::DamageDealt, i32::from(credit.damage));
            if credit.hit {
                statistics.add(Statistic::Hits, 1);
            }
            if credit.killed {
                statistics.add(Statistic::Kills, 1);
            }
        }
    }
    events.extend(outcome.events);
    if outcome.control == spells::out_of_combat::Control::PartyLost {
        events.extend(crate::mission::defeat(state, content)?);
        return Ok(TurnFlow::Terminated);
    }
    if let Some(arm) = outcome.arm {
        return super::out_of_combat_arms::run(
            state,
            content,
            arm,
            (&packet, invocation.id, invocation.current),
            events,
        );
    }
    Ok(TurnFlow::Returned)
}
