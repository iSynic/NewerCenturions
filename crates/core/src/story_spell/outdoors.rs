//! Outdoor resolvespell and buttonchoice's temporary ally-to-NPC record copies.
use crate::combat::{
    outdoor_health,
    runtime::{TurnFlow, TurnOutcome},
    spells,
};
use crate::{Content, Event, RuleError, Session, combat, story_allies};
use centurions_content::{
    combat::MonsterRecord,
    records::{AttackPacket, AttackPacketMut, CharacterStatisticsMut, Statistic},
};

/// The learned-spell suffix at buttonchoice10ad70, after party selection.
/// Entry has BYTE435662 clear; public cast preparation owns its memory update.
pub(crate) fn party(
    state: &mut Session,
    content: &Content,
    packet: &[u8],
    id: i16,
    count: i16,
) -> Result<TurnOutcome, RuleError> {
    let mode = AttackPacket(packet).targeting_mode();
    let copied = matches!(mode, 3 | 7 | 9) || mode == 0 && count > state.warriors.len() as i16;
    if copied && state.ally_count != 0 {
        let mut mask = state.spell_targets()?;
        for index in 0..state.ally_count {
            let start = index * story_allies::ALLY_BYTES;
            let record = MonsterRecord {
                bytes: state.allies[start..start + story_allies::ALLY_BYTES].to_vec(),
            };
            state.native_work.npc_backing.write_record(index, record)?;
            mask[index + 10] = 1;
        }
        state.publish_spell_targets(&mask);
    }
    let mut events = vec![Event::Sound {
        resource_id: AttackPacket(packet).launch_sound()?.wrapping_neg(),
    }];
    let outcome = resolve(state, content, packet, id, count)?;
    events.extend(outcome.events);
    if outcome.flow == TurnFlow::Terminated {
        return Ok(TurnOutcome {
            events,
            flow: TurnFlow::Terminated,
        });
    }
    // MainLoop entry from killbody does not return to this copy-out loop.
    if copied {
        for index in 0..state.ally_count {
            let record = &state.native_work.npc_backing.slot(index)?.record.bytes;
            let start = index * story_allies::ALLY_BYTES;
            state.allies[start..start + story_allies::ALLY_BYTES].copy_from_slice(record);
        }
    }
    if mode == 11 {
        events.push(Event::Sound {
            resource_id: AttackPacket(packet).impact_sound()?,
        });
    }
    state.set_ready(0);
    Ok(TurnOutcome {
        events,
        flow: TurnFlow::Returned,
    })
}

pub(crate) fn resolve(
    state: &mut Session,
    content: &Content,
    packet: &[u8],
    id: i16,
    count: i16,
) -> Result<TurnOutcome, RuleError> {
    let attack = AttackPacket(packet);
    let effect = attack.effect()?.wrapping_abs();
    // The global-duration arm returns before target, field-usage and health work.
    if attack.targeting_mode() == 7 {
        let duration = spells::detached::duration(packet, count, &mut state.random);
        let mut events = vec![];
        match effect {
            50 => {
                events.push(Event::Sound { resource_id: 601 });
                combat::party_buffs::extend_from_count(&mut state.party_buffs, count)?;
            }
            110 => combat::party_buffs::extend_scenario_tail(state, duration)?,
            0..=9 => combat::party_buffs::extend(&mut state.party_buffs, effect, duration)?,
            _ => {
                if !combat::party_buffs::extend_indexed(state, effect, duration)? {
                    return Err(unavailable(
                        "indexed spell effect WORD 0x442562 + 2 * effect without an owner",
                    ));
                }
            }
        }
        state.native_work.attack_packet = packet.to_vec();
        state.set_ready(0);
        return Ok(TurnOutcome {
            events,
            flow: TurnFlow::Returned,
        });
    }
    let mut mask = [0; 110];
    mask[..6].copy_from_slice(&state.native_work.party_mask);
    let tail = &state.native_work.spell_target_tail;
    if tail.len() != 104 {
        return Err("invalid retained spell target mask".into());
    }
    mask[6..].copy_from_slice(tail);
    // Opaque scratch for absent records is guarded before every selected or
    // reflected NPC read and is never committed to the retained array.
    let mut known = [false; 100];
    let enemies: Vec<_> = state
        .native_work
        .npc_backing
        .slots
        .iter()
        .enumerate()
        .map(|(index, slot)| {
            if let Some(slot) = slot {
                known[index] = true;
                slot.record.clone()
            } else {
                MonsterRecord {
                    bytes: vec![0; 288],
                }
            }
        })
        .collect();
    let current = state
        .native_work
        .initiative_work
        .current(state.native_work.selected_enemy)
        .ok();
    let selected = attack.targeting_mode() != 7 && mask.iter().any(|selected| *selected > 0);
    let filter = if selected {
        state.native_work.weapon_work.armor_filter()
    } else {
        false
    };
    let sound = state.native_work.weapon_work.sound();
    let location = state
        .exploration
        .as_ref()
        .ok_or("outdoor spell map missing")?;
    let mission = state.land(content).ok_or("outdoor spell content missing")?;
    let mut usage = spells::FieldUsage {
        slot: 0,
        used: vec![0; 128],
    };
    let monster = state.native_work.monster_work.known();
    let outcome = spells::detached::resolve(
        &state.warriors,
        &enemies,
        &mask,
        &mut usage,
        &spells::detached::Context {
            spell: spells::DirectSpell {
                packet,
                level: id % 1000 / 100 - 1,
                count,
                armor_filter: filter,
                weapon_sound: sound,
            },
            id,
            current,
            memory: spells::RestrictionMemoryView::ordinary_startup(
                &mission.item_cache,
                spells::retained_item_alias(&state.native_work.ammunition_work)?,
            )
            .with_alias_owners(
                crate::combat::warrior_alias::Owners::new(
                    state.native_work.ammunition_work.item.as_deref(),
                    &state.native_work.shop_work,
                    Some(&location.event_headers),
                )
                .with_backup(&state.native_work.warrior_table),
            )
            .with_tables(mission)
            .with_definitions(&content.items)
            .with_npc_records(&known)
            // The charm arm reads the retained party caster WORD4435d2.
            .with_selected_party_caster(state.selected_warrior as i16)
            .with_resist_caster(
                current,
                (state.selected_warrior, state.native_work.selected_enemy),
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
                state.native_work.ammunition_work.item.as_deref(),
                &state.native_work.shop_work,
                Some(&location.event_headers),
                (&state.warriors, &state.native_work.warrior_table),
            ),
            collision: false,
            movement_gate: state.bandage_ready,
        },
        &state.native_work.health_context,
        &mut state.random,
    )?;
    if outcome.npc_credit.iter().any(|credit| !credit.killed) {
        state.native_work.weapon_work.sound();
    }
    let credit_owner = if outcome.npc_credit.is_empty() {
        None
    } else {
        Some(
            state
                .native_work
                .initiative_work
                .current(state.native_work.selected_enemy)?,
        )
    };
    if outcome.monster_work_written {
        state.invalidate_monster_work();
    }
    // updatefat clamps effect68's fatigue 1 to 4 outside combat.
    if outcome.bodies.fatigue_reset {
        position_fatigue(state);
    }
    state.store_table_records(outcome.bodies.warriors)?;
    state.commit_writes(&outcome.bodies.alias_writes)?;
    for (index, record) in outcome.bodies.enemies.into_iter().enumerate() {
        if known[index] {
            state.native_work.npc_backing.write_record(index, record)?;
        }
    }
    let mut events = vec![];
    state.native_work.attack_packet = packet.to_vec();
    let mut retained = AttackPacketMut(&mut state.native_work.attack_packet);
    retained.set_health_effect(outcome.health.effect as u8);
    // Effect66 stores its drawn selector in the packet's area byte.
    if let Some(shape) = outcome.bodies.area_shape {
        retained.set_area_shape(shape);
    }
    state.native_work.health_context = outcome.health;
    if attack.targeting_mode() != 7 {
        let work = &mut state.native_work.tactical_work;
        work.field_used[0] = usage.used[0];
        work.field_used[61] = usage.used[61];
    }
    if outcome.clear_departure {
        state.native_work.neighbor_work.departure = 0;
    }
    if credit_owner.is_some_and(|body| (0..6).contains(&body)) {
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
    state
        .native_work
        .victory_work
        .add_total(outcome.bodies.total_delta);
    events.extend(outcome.events);
    if outcome.control == spells::detached::Control::PartyLost {
        events.extend(crate::mission::defeat(state, content)?);
        return Ok(TurnOutcome {
            events,
            flow: TurnFlow::Terminated,
        });
    }
    // The movement and summon arms act on the retained tactical arrays at the
    // retained cell WORD443722/443724; buttonchoice and Items clear it first.
    let flow = match outcome.arm {
        Some(arm) => {
            combat::runtime::retained_arm(state, content, arm, (packet, id, None), &mut events)?
        }
        None => TurnFlow::Returned,
    };
    Ok(TurnOutcome { events, flow })
}

fn position_fatigue(state: &mut Session) {
    if let Some(position) = state.exploration.as_mut() {
        position.fatigue = 4;
    }
}

fn unavailable(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "psi.outdoor",
        dependency,
    }
}
