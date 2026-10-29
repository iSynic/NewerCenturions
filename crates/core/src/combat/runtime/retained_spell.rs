//! Session commits for resolvespell over retained arrays outside combat mode.
use super::TurnFlow;
use crate::combat::{outdoor_health, spells};
use crate::{Content, Event, RuleError, Session};
use centurions_content::records::{AttackPacketMut, CharacterStatisticsMut, Statistic};

pub(super) struct Invocation {
    pub id: i16,
    pub level: i16,
    pub count: i16,
    pub collision: bool,
    /// The cast loop's triple at WORD442bd4, which the summon arm reads.
    pub current: Option<(i16, i16)>,
}

pub(super) fn apply(
    state: &mut Session,
    content: &Content,
    mask: &[i16; 110],
    usage: &mut spells::FieldUsage,
    invocation: Invocation,
    events: &mut Vec<Event>,
) -> Result<TurnFlow, RuleError> {
    let mission = state.land(content).ok_or("retained spell map missing")?;
    let packet = state.native_work.attack_packet.clone();
    let enemies = (0..100)
        .map(|index| {
            state
                .native_work
                .npc_backing
                .slot(index)
                .map(|slot| slot.record.clone())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let current = state
        .native_work
        .initiative_work
        .current(state.native_work.selected_enemy)?;
    let position = state
        .exploration
        .as_ref()
        .ok_or("retained spell position missing")?;
    let monster = state.native_work.monster_work.known();
    let outcome = spells::detached::resolve(
        &state.warriors,
        &enemies,
        mask,
        usage,
        &spells::detached::Context {
            spell: spells::DirectSpell {
                packet: &packet,
                level: invocation.level,
                count: invocation.count,
                armor_filter: state.native_work.weapon_work.armor_filter(),
                weapon_sound: state.native_work.weapon_work.sound(),
            },
            id: invocation.id,
            current: Some(current),
            memory: spells::RestrictionMemoryView::ordinary_startup(
                &mission.item_cache,
                spells::retained_item_alias(&state.native_work.ammunition_work)?,
            )
            .with_alias_owners(
                crate::combat::warrior_alias::Owners::new(
                    state.native_work.ammunition_work.item.as_deref(),
                    &state.native_work.shop_work,
                    Some(&position.event_headers),
                )
                .with_backup(&state.native_work.warrior_table),
            )
            .with_tables(mission)
            .with_definitions(&content.items)
            // spelllist15c10a reads the retained party caster WORD4435d2.
            .with_selected_party_caster(state.selected_warrior as i16)
            .with_resist_caster(
                Some(current),
                (state.selected_warrior, state.native_work.selected_enemy),
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
                state.native_work.ammunition_work.item.as_deref(),
                &state.native_work.shop_work,
                Some(&position.event_headers),
                (&state.warriors, &state.native_work.warrior_table),
            ),
            collision: invocation.collision,
            movement_gate: state.bandage_ready,
        },
        &state.native_work.health_context,
        &mut state.random,
    )?;
    if let Some((effect, duration)) = outcome.party_effect {
        match effect {
            50 => {
                events.push(Event::Sound { resource_id: 601 });
                crate::combat::party_buffs::extend_from_count(
                    &mut state.party_buffs,
                    invocation.count,
                )?;
            }
            110 => crate::combat::party_buffs::extend_scenario_tail(state, duration)?,
            _ => crate::combat::party_buffs::extend(&mut state.party_buffs, effect, duration)?,
        }
    }
    if outcome.monster_work_written {
        state.invalidate_monster_work();
    }
    // updatefat clamps effect68's fatigue 1 to 4 once combat has ended.
    if outcome.bodies.fatigue_reset
        && let Some(position) = state.exploration.as_mut()
    {
        position.fatigue = 4;
    }
    state.store_table_records(outcome.bodies.warriors)?;
    state.commit_writes(&outcome.bodies.alias_writes)?;
    state
        .native_work
        .npc_backing
        .store_records(outcome.bodies.enemies);
    state
        .native_work
        .victory_work
        .add_total(outcome.bodies.total_delta);
    state
        .native_work
        .tactical_work
        .field_used
        .clone_from(&usage.used);
    let mut retained = AttackPacketMut(&mut state.native_work.attack_packet);
    retained.set_health_effect(outcome.health.effect as u8);
    // Effect66 stores its drawn selector in the packet's area byte.
    if let Some(shape) = outcome.bodies.area_shape {
        retained.set_area_shape(shape);
    }
    state.native_work.health_context = outcome.health;
    if outcome.clear_departure {
        state.native_work.neighbor_work.departure = 0;
    }
    if (0..6).contains(&current) {
        for credit in &outcome.npc_credit {
            let mut statistics =
                CharacterStatisticsMut(&mut state.warriors[state.selected_warrior].bytes);
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
    if outcome.control == spells::detached::Control::PartyLost {
        events.extend(crate::mission::defeat(state, content)?);
        return Ok(TurnFlow::Terminated);
    }
    if let Some(arm) = outcome.arm {
        return super::retained_arms::run(
            state,
            content,
            arm,
            (&packet, invocation.id, invocation.current),
            events,
        );
    }
    Ok(TurnFlow::Returned)
}
