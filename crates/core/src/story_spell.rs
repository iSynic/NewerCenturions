use crate::{Content, Event, RuleError, Session, combat, feedback::NativeFeedback, story_health};
use centurions_content::{span, word};

/// newland 17/18: the existing selection mask is retained by 17 and replaced by 18.
pub(crate) fn apply(
    state: &mut Session,
    content: &Content,
    extra: &[u8],
    all: bool,
) -> Result<Vec<Event>, RuleError> {
    span(extra, 0, 10)?;
    let id = word(extra, 0)?;
    let count = word(extra, 2)?;
    let map = content
        .map_at(
            state.mission.as_deref(),
            state.scenario_level,
            state.exploration.as_ref().unwrap().map,
        )
        .ok_or_else(|| RuleError::Invalid("script spell map missing".into()))?;
    let mut packet = combat::spells::load_packet(&map.combat.as_ref().unwrap().attacks, id)?;
    packet[8] = packet[8].wrapping_add(word(extra, 4)? as u8);
    if word(extra, 6)? != 0 {
        packet[9] = 3;
    }
    let effect = word(&packet, 40)?.wrapping_abs();
    // The supplied scripted packet uses condition 23. Other effects retain their explicit adapters.
    if !matches!(effect, 0 | 23 | 57) {
        return Err(RuleError::Unsupported {
            action: "story.spell",
            dependency: "other scripted effect or tactical target selection",
        });
    }
    if all {
        state.party_mask.fill(0);
        state.party_mask[..state.warriors.len()].fill(1);
    }
    if let Some(combat) = state.combat.as_mut() {
        let mut mask = [0; 110];
        if all {
            mask[..state.warriors.len()].fill(1);
        } else {
            let story = state
                .combat_story
                .as_ref()
                .ok_or("scripted tactical spell origin missing")?;
            let target = combat::spells::SpellTarget {
                first: story.entry.x as i16,
                second: story.entry.y as i16,
                kind: 3,
            };
            let context = combat::spells::SpellContext {
                packet: &packet,
                id,
                count,
                caster: story.actor,
                initiative: combat.cursor as i16,
                collision: false,
            };
            // newland overwrites the two coordinates but leaves the cleared kind WORD.
            // spelltargets therefore reads the coordinate as an index into native party-position memory.
            let first = target.first;
            let point = if (0..state.warriors.len() as i16).contains(&first) {
                combat.squad.positions[first as usize]
            } else if first >= 8 {
                let index = (first as usize * 4 - 30) / 2;
                (
                    *combat
                        .order
                        .get(index)
                        .ok_or("script target position alias outside initiative")?,
                    *combat
                        .order
                        .get(index + 1)
                        .ok_or("script target position alias outside initiative")?,
                )
            } else {
                return Err(RuleError::Unsupported {
                    action: "story.spell",
                    dependency: "script target position in unrepresented selection work",
                });
            };
            let selected_target = combat::spells::SpellTarget {
                first: point.0 + combat.map.shift.0 + 5,
                second: point.1 + combat.map.shift.1 + 5,
                kind: 3,
            };
            let selected = combat::spells::target_mask_with_origin(
                combat,
                &state.warriors,
                &map.areas,
                (&selected_target, &target),
                &context,
                &combat.spell_fields,
                &mut state.random,
            )?;
            mask = selected.mask;
            combat.spell_fields = selected.fields;
        }
        state.party_mask.copy_from_slice(&mask[..6]);
        combat.attack_packet = Some(packet.clone());
        combat.attack_identity = Some([id / 1000 - 1, id % 1000 / 100 - 1, id % 100 - 1]);
        state.attack_count = count;
        return combat::runtime::resolve_spell_mask(
            combat,
            &mut state.warriors,
            &mut state.random,
            (id, &packet, count),
            &mask,
            state
                .combat_story
                .as_ref()
                .map_or(state.selected_warrior as i16, |s| s.actor),
            combat::spells::RestrictionMemoryView::ordinary_startup(
                &map.item_cache,
                state
                    .ammunition_work
                    .item
                    .as_ref()
                    .map(|item| word(item, 0x32))
                    .transpose()?,
            ),
        );
    }
    resolve_outdoors(state, &packet, id, count, state.selected_warrior, false)
}

/// resolvespell151022 outdoors still runs resistance, but ignores its refusal result.
pub(crate) fn resolve_outdoors(
    state: &mut Session,
    packet: &[u8],
    id: i16,
    count: i16,
    caster: usize,
    include_allies: bool,
) -> Result<Vec<Event>, RuleError> {
    let effect = word(packet, 40)?.wrapping_abs();
    if !matches!(effect, 0..=40 | 57 | 64) && !(packet[1] == 7 && effect == 110) {
        return Err(RuleError::Unsupported {
            action: "psi.outdoor",
            dependency: "other outdoor spell effect",
        });
    }
    let mut duration = state
        .random
        .range(i16::from(packet[13] as i8), i16::from(packet[14] as i8));
    if packet[15] != 0 {
        for _ in 0..count {
            duration = duration.wrapping_add(
                state
                    .random
                    .range(i16::from(packet[15] as i8), i16::from(packet[16] as i8)),
            );
        }
    }
    if packet[1] == 7 {
        if effect == 110 {
            combat::party_buffs::extend_scenario_tail(state, duration)?;
        } else {
            combat::party_buffs::extend(&mut state.party_buffs, effect, duration)?;
        }
        state.attack_packet = Some(packet.to_vec());
        return Ok(vec![]);
    }
    let position = state.exploration.as_ref().unwrap();
    let (x, y) = (position.x as i16, position.y as i16);
    let mut health = story_health::context(
        &state.warriors,
        state.selected_warrior,
        state.health_context.as_ref(),
    );
    state.attack_packet = Some(packet.to_vec());
    health.effect = packet[18] as i8;
    let mut events = Vec::new();
    for member in 0..state.warriors.len() {
        if state.party_mask[member] <= 0 {
            continue;
        }
        let w = &mut state.warriors[member].bytes;
        // resist still executes outdoors; its return value does not suppress resolvedamage.
        let race = word(w, 0x32)?;
        let excluded = word(packet, 50)? != 0
            && (1..=16).contains(&race)
            && word(packet, 50)? as u16 & (0x8000 >> (race - 1)) != 0;
        if excluded {
            events.push(Event::Feedback(NativeFeedback::Flash {
                address: 0x4018b4,
                x: 50,
                y: 50,
                wait: 40,
                sound: 139,
            }));
        } else if !matches!(packet[9], 1 | 3..=255) {
            let mut saved = false;
            if packet[19] == 0 {
                let threshold = word(w, 0x112)?
                    .wrapping_add(if race == 3 { 1000 } else { 0 })
                    .wrapping_add(i16::from(packet[8] as i8).wrapping_mul(count));
                saved = state.random.bound(100) <= threshold;
            }
            if !saved && !(matches!(packet[19], 0 | 5) && word(w, 0x90)? != 0) {
                state.random.bound(100);
            }
        }
        let result = combat::damage::resolve_damage(
            w,
            false,
            &combat::damage::DamagePacket {
                bytes: packet.to_vec(),
                attack_count: count,
            },
            combat::damage::DamageFlags {
                shield: true,
                armor: true,
                saves: true,
                armor_filter: false,
            },
            &mut state.random,
        )?;
        if result.shield_sound {
            events.push(Event::Sound { resource_id: 206 });
        }
        let mut follow_up = None;
        if (1..=40).contains(&effect) {
            combat::spell_conditions::add_duration(w, false, effect, duration)?;
        }
        if matches!(effect, 1..=40 | 64) {
            follow_up = Some(if word(w, 0x9a)? != 0 && state.random.bound(100) < 34 {
                caster
            } else {
                member
            });
            if result.total == 0 {
                events.push(Event::Sound {
                    resource_id: word(packet, 38)?,
                });
                events.push(Event::OutdoorEffect {
                    effect: i16::from(packet[20] as i8).wrapping_abs(),
                    x,
                    y,
                    size: 32,
                    delay: 0,
                });
            }
        }
        if let Some(target) = follow_up {
            combat::spell_conditions::follow_up(&mut state.warriors[target].bytes, false, effect)?;
        }
        let w = &mut state.warriors[member].bytes;
        let before = word(w, 0x50)?;
        let defeated = before > 0 && before.wrapping_sub(result.total) < 1;
        if result.total != 0 && before > -10 {
            events.push(Event::Sound {
                resource_id: word(packet, 38)?,
            });
            if result.total < 0 {
                let outcome = story_health::heal_outdoors(
                    &state.warriors,
                    member,
                    result.total.wrapping_abs(),
                    true,
                    &health,
                    &mut state.random,
                )?;
                for event in outcome.events {
                    match event {
                        story_health::HealthEvent::Animation { effect, .. } => {
                            events.push(Event::OutdoorEffect {
                                effect: i16::from(effect),
                                x,
                                y,
                                size: 32,
                                delay: 0,
                            })
                        }
                        story_health::HealthEvent::Sound(resource_id) => {
                            events.push(Event::Sound { resource_id })
                        }
                        _ => {}
                    }
                }
                state.warriors = outcome.warriors;
                health = outcome.context;
            } else if before >= 0 {
                w[0x50..0x52].copy_from_slice(&before.wrapping_sub(result.total).to_be_bytes());
                health.affected[member] = -1;
                if defeated {
                    // Outdoor killbody increments availability and plays a death sound, without combat record cleanup.
                    health.unavailable = health.unavailable.wrapping_add(1);
                    health.last_body = member as i16;
                    events.push(Event::Sound {
                        resource_id: state.random.bound(2).wrapping_add(506),
                    });
                } else {
                    events.push(Event::OutdoorEffect {
                        effect: i16::from(packet[20] as i8).wrapping_abs(),
                        x,
                        y,
                        size: 32,
                        delay: 0,
                    });
                }
            }
        }
        events.push(Event::SpellResolved {
            caster: None,
            spell: id,
            target: member as i16,
            damage: result.total,
            defeated,
        });
    }
    if include_allies {
        for ally in state.allies[..state.ally_count * crate::story_allies::ALLY_BYTES]
            .as_chunks_mut::<{ crate::story_allies::ALLY_BYTES }>()
            .0
        {
            // Currently selectable party/multi outdoor packets have duration effects only.
            let result = combat::damage::resolve_damage(
                ally,
                true,
                &combat::damage::DamagePacket {
                    bytes: packet.to_vec(),
                    attack_count: count,
                },
                combat::damage::DamageFlags {
                    shield: true,
                    armor: true,
                    saves: true,
                    armor_filter: false,
                },
                &mut state.random,
            )?;
            if result.total != 0 {
                return Err(RuleError::Unsupported {
                    action: "psi.outdoor",
                    dependency: "outdoor allied-body damage continuation",
                });
            }
            if result.shield_sound {
                events.push(Event::Sound { resource_id: 206 });
            }
            if (1..=40).contains(&effect) {
                combat::spell_conditions::apply(ally, true, effect, duration, &mut state.random)?;
                events.push(Event::Sound {
                    resource_id: word(packet, 38)?,
                });
            }
        }
    }
    // heal1287dc writes the effect byte in the retained attack packet itself.
    state.attack_packet.as_mut().unwrap()[18] = health.effect as u8;
    state.health_context = Some(health);
    Ok(events)
}
