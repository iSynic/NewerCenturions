//! Resolving spells outside combat, with allies copied into monster records while
//! the spell runs.
use crate::combat::spells::spell_level;
use crate::combat::warrior_alias::Owners;
use crate::combat::{
    outdoor_health,
    runtime::{TurnFlow, TurnOutcome},
    spells,
};
use crate::{Content, Event, RuleError, Session, combat, story_allies};
use centurions_content::{
    combat::MonsterRecord,
    records::{CharacterStatisticsMut, SpellInfoRecord, SpellInfoRecordMut, Statistic},
};

/// Casts a learned spell after party selection. Cast preparation updates the
/// Psi memory.
///
/// Original: buttonchoice
pub(crate) fn party(
    state: &mut Session,
    content: &Content,
    packet: &[u8],
    id: i16,
    count: i16,
) -> Result<TurnOutcome, RuleError> {
    let mode = SpellInfoRecord(packet).target_type();
    let copied = matches!(mode, 3 | 7 | 9) || mode == 0 && count > state.warriors.len() as i16;
    if copied && state.held_over != 0 {
        let mut mask = state.spell_targets()?;
        for index in 0..state.held_over {
            let start = index * story_allies::ALLY_BYTES;
            let record = MonsterRecord {
                bytes: state.holdover[start..start + story_allies::ALLY_BYTES].to_vec(),
            };
            state.globals.monsters.write_record(index, record)?;
            mask[index + 10] = 1;
        }
        state.publish_spell_targets(&mask);
    }
    let mut events = vec![Event::Sound {
        resource_id: SpellInfoRecord(packet).launch_sound()?.wrapping_neg(),
    }];
    let outcome = resolve(state, content, packet, id, count)?;
    events.extend(outcome.events);
    if outcome.flow == TurnFlow::Terminated {
        return Ok(TurnOutcome {
            events,
            flow: TurnFlow::Terminated,
        });
    }
    // A killbody that returns to the main loop skips copying the allies back.
    if copied {
        for index in 0..state.held_over {
            let record = &state.globals.monsters.slot(index)?.record.bytes;
            let start = index * story_allies::ALLY_BYTES;
            state.holdover[start..start + story_allies::ALLY_BYTES].copy_from_slice(record);
        }
    }
    if mode == 11 {
        events.push(Event::Sound {
            resource_id: SpellInfoRecord(packet).impact_sound()?,
        });
    }
    state.set_in_spell(0);
    Ok(TurnOutcome {
        events,
        flow: TurnFlow::Returned,
    })
}

/// Resolves a spell cast outdoors on its targets and returns what it did.
pub(crate) fn resolve(
    state: &mut Session,
    content: &Content,
    packet: &[u8],
    id: i16,
    count: i16,
) -> Result<TurnOutcome, RuleError> {
    let attack = SpellInfoRecord(packet);
    let effect = attack.effect()?.wrapping_abs();
    // A party-duration spell returns before targets, fields and health.
    if attack.is_party_duration() {
        let duration = spells::out_of_combat::duration(packet, count, &mut state.random);
        let mut events = vec![];
        match effect {
            50 => {
                events.push(Event::Sound { resource_id: 601 });
                combat::party_buffs::extend_from_count(&mut state.party_condition, count)?;
            }
            110 => combat::party_buffs::extend_scenario_tail(state, duration)?,
            0..=9 => combat::party_buffs::extend(&mut state.party_condition, effect, duration)?,
            _ => {
                if !combat::party_buffs::extend_indexed(state, effect, duration)? {
                    return Err(unavailable(
                        "indexed spell effect WORD 0x442562 + 2 * effect without an owner",
                    ));
                }
            }
        }
        state.globals.spell_info = packet.to_vec();
        state.set_in_spell(0);
        return Ok(TurnOutcome {
            events,
            flow: TurnFlow::Returned,
        });
    }
    let mut mask = [0; 110];
    mask[..6].copy_from_slice(&state.globals.party_track);
    let tail = &state.globals.track_tail;
    if tail.len() != 104 {
        return Err("invalid retained spell target mask".into());
    }
    mask[6..].copy_from_slice(tail);
    // Absent monster records are refused before any selected or reflected read and
    // are never stored back.
    let mut known = [false; 100];
    let enemies: Vec<_> = state
        .globals
        .monsters
        .slots
        .iter()
        .enumerate()
        .map(|(index, slot)| {
            if let Some(slot) = slot {
                known[index] = true;
                slot.record.clone()
            } else {
                MonsterRecord {
                    bytes: vec![0; centurions_content::combat::MONSTER_BYTES],
                }
            }
        })
        .collect();
    let current = state
        .globals
        .battle_order
        .current(state.globals.monster_up)
        .ok();
    let selected = !attack.is_party_duration() && mask.iter().any(|selected| *selected > 0);
    let filter = if selected {
        state.globals.weapon_traits.armor_filter()
    } else {
        false
    };
    let sound = state.globals.weapon_traits.sound();
    let location = state
        .exploration
        .as_ref()
        .ok_or("outdoor spell map missing")?;
    let mission = state.land(content).ok_or("outdoor spell content missing")?;
    let mut usage = spells::FieldUsage {
        slot: 0,
        used: vec![0; 128],
    };
    let monster = state.globals.monster_pick.known();
    let outcome = spells::out_of_combat::resolve(
        &state.warriors,
        &enemies,
        &mask,
        &mut usage,
        &spells::out_of_combat::Context {
            spell: spells::DirectSpell {
                packet,
                level: spell_level(id),
                count,
                armor_filter: filter,
                weapon_sound: sound,
            },
            id,
            current,
            memory: spells::RestrictionMemoryView::ordinary_startup(
                &mission.item_cache,
                spells::loaded_item_restriction(&state.globals.loaded_items)?,
            )
            .with_alias_owners(
                Owners::new(
                    state.globals.loaded_items.item.as_deref(),
                    &state.globals.the_shop,
                    Some(&location.event_headers),
                )
                .with_backup(&state.globals.warrior_table),
            )
            .with_tables(mission)
            .with_definitions(&content.items)
            .with_monster_records(&known)
            // Charm reads the party caster `char_up`.
            .with_selected_party_caster(state.char_select_new as i16)
            .with_resist_caster(
                current,
                (state.char_select_new, state.globals.monster_up),
                &state.warriors,
                &enemies,
            )
            .with_current_accuracy(
                current,
                (&state.warriors, &enemies, monster.as_deref()),
                state.difficulty,
            ),
            location: (location.x as i16, location.y as i16),
            animation_delay: state.command_preferences.animation_delay,
            outdoor_health: outdoor_health::View::new(
                content,
                state.mission.as_deref(),
                state.scenario_level,
                state.globals.loaded_items.item.as_deref(),
                &state.globals.the_shop,
                Some(&location.event_headers),
                (&state.warriors, &state.globals.warrior_table),
            ),
            collision: false,
            movement_gate: state.bandage_ready,
        },
        &state.globals.health_context,
        &mut state.random,
    )?;
    if outcome.monster_credit.iter().any(|credit| !credit.killed) {
        state.globals.weapon_traits.sound();
    }
    let credit_owner = if outcome.monster_credit.is_empty() {
        None
    } else {
        Some(
            state
                .globals
                .battle_order
                .current(state.globals.monster_up)?,
        )
    };
    if outcome.monster_pick_written {
        state.invalidate_monster_pick();
    }
    // Outside combat the fatigue reset's 1 becomes 4.
    if outcome.bodies.fatigue_reset {
        position_fatigue(state);
    }
    state.store_table_records(outcome.bodies.warriors)?;
    state.commit_writes(&outcome.bodies.alias_writes)?;
    for (index, record) in outcome.bodies.enemies.into_iter().enumerate() {
        if known[index] {
            state.globals.monsters.write_record(index, record)?;
        }
    }
    let mut events = vec![];
    state.globals.spell_info = packet.to_vec();
    let mut info = SpellInfoRecordMut(&mut state.globals.spell_info);
    info.set_health_effect(outcome.health.effect as u8);
    // The random-area effect stores its drawn shape in `spell_info`.
    if let Some(shape) = outcome.bodies.area_shape {
        info.set_area_size(shape);
    }
    state.globals.health_context = outcome.health;
    if !attack.is_party_duration() {
        let battlefield = &mut state.globals.battlefield;
        battlefield.collide_check[0] = usage.used[0];
        battlefield.collide_check[61] = usage.used[61];
    }
    if outcome.clear_departure {
        state.globals.neighbors.departure = 0;
    }
    if credit_owner.is_some_and(|body| (0..6).contains(&body)) {
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
    state
        .globals
        .kill_counts
        .add_num_enemy(outcome.bodies.total_delta);
    events.extend(outcome.events);
    if outcome.control == spells::out_of_combat::Control::PartyLost {
        events.extend(crate::mission::defeat(state, content)?);
        return Ok(TurnOutcome {
            events,
            flow: TurnFlow::Terminated,
        });
    }
    // Movement and summon spells act on the tactical arrays at the target cell,
    // which buttonchoice and Items clear first.
    let flow = match outcome.arm {
        Some(arm) => combat::runtime::out_of_combat_arm(
            state,
            content,
            arm,
            (packet, id, None),
            &mut events,
        )?,
        None => TurnFlow::Returned,
    };
    Ok(TurnOutcome { events, flow })
}

/// Resets the party's fatigue to 4.
fn position_fatigue(state: &mut Session) {
    if let Some(position) = state.exploration.as_mut() {
        position.fatigue = 4;
    }
}

/// The refusal given when an outdoor spell needs something the reconstruction does not carry.
fn unavailable(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "psi.outdoor",
        dependency,
    }
}
