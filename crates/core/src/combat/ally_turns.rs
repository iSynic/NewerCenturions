use super::*;
use attack::AttackResult;
use damage::{DamageFlags, DamagePacket, resolve_damage};

fn position(c: &Combat, body: i16) -> (i16, i16) {
    if body < 10 {
        c.squad.positions[body as usize]
    } else {
        c.enemies.positions[(body - 10) as usize]
    }
}
fn alive(c: &Combat, w: &[Warrior], body: i16) -> bool {
    if (0..6).contains(&body) {
        w[body as usize].bytes[0x9df] != 0
    } else if body >= 10 {
        c.enemies
            .records
            .get((body - 10) as usize)
            .is_some_and(|m| m.stamina().unwrap_or(0) > 0 && m.bytes[0xb0] != 255)
    } else {
        false
    }
}
fn faction(c: &Combat, w: &[Warrior], body: i16) -> Result<i16, RuleError> {
    if body < 10 {
        Ok(word(&w[body as usize].bytes, 0x10)?)
    } else {
        Ok(word(&c.enemies.records[(body - 10) as usize].bytes, 0xe)?)
    }
}

pub fn item_context_after_load(items: &[Vec<u8>], id: i16) -> Result<i16, RuleError> {
    let index = usize::try_from(id.wrapping_abs())
        .map_err(|_| RuleError::Invalid("invalid item context".into()))?;
    let item = items
        .get(index)
        .ok_or_else(|| RuleError::Invalid("item context missing".into()))?;
    Ok(word(item, 32)?)
}

pub fn item_context_after_weight_calc(
    warrior: &Warrior,
    items: &[Vec<u8>],
    previous: i16,
) -> Result<i16, RuleError> {
    let inventory = warrior.inventory()?;
    if let Some(last) = inventory.last() {
        item_context_after_load(items, last.id())
    } else {
        Ok(previous)
    }
}

pub fn natural_attack_enemy(
    c: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    attacker: i16,
    target: i16,
    index: usize,
) -> Result<AttackResult, RuleError> {
    Ok(natural_attack_enemy_inner(
        c,
        warriors,
        random,
        attacker,
        target,
        index,
        attacker,
        attack::default_attack_packet(),
    )?
    .0)
}

#[allow(clippy::too_many_arguments)]
pub fn natural_attack_enemy_with_packet(
    c: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    attacker: i16,
    target: i16,
    index: usize,
    attacks: &[u8],
    retained: &[u8],
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    natural_attack_enemy_with_actor(
        c,
        warriors,
        random,
        (attacker, target, index, attacker),
        attacks,
        retained,
    )
}

pub fn natural_attack_enemy_with_actor(
    c: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    attacks: &[u8],
    retained: &[u8],
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    let (attacker, target, index, actor) = bodies;
    if retained.len() != 58 {
        return Err(RuleError::Invalid("retained attack packet size".into()));
    }
    let base = attack::attack_packet(attacks)?;
    let (mut next_combat, mut next_warriors, mut next_random) =
        (c.clone(), warriors.to_vec(), random.clone());
    let result = natural_attack_enemy_inner(
        &mut next_combat,
        &mut next_warriors,
        &mut next_random,
        attacker,
        target,
        index,
        actor,
        base,
    )?;
    *c = next_combat;
    warriors.clone_from_slice(&next_warriors);
    *random = next_random;
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn natural_attack_enemy_inner(
    c: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    attacker: i16,
    target: i16,
    index: usize,
    actor: i16,
    mut packet: Vec<u8>,
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    if attacker < 10 || target < 10 || attacker == target || index >= 6 {
        return Err(RuleError::Invalid("invalid monster attack".into()));
    }
    let a = (attacker - 10) as usize;
    let t = (target - 10) as usize;
    let source = c
        .enemies
        .records
        .get(a)
        .ok_or_else(|| RuleError::Invalid("attacker missing".into()))?
        .bytes
        .clone();
    let victim = &c
        .enemies
        .records
        .get(t)
        .ok_or_else(|| RuleError::Invalid("target missing".into()))?
        .bytes
        .clone();
    let selected = if source[0x93 + index * 4] == 0 {
        0x93
    } else {
        0x93 + index * 4
    };
    let condition = source[0x96 + index * 4];
    let current = word(&source, 0x4e)?;
    let guardian_reaction = current == 187 && word(&source, 0x26)? == 0;
    if (!matches!(current, 0 | 1 | 28 | 29 | 680 | 681) && !guardian_reaction)
        || word(&source, 0x28)? != 0
        || !matches!(condition, 0 | 2 | 4 | 6 | 9 | 14)
        || victim[0xd1] != 0
        || word(victim, 0xc)? != 0
    {
        return Err(RuleError::Unsupported {
            action: "combat.monster_attack",
            dependency: "monster weapon, attack condition, reflection or immunity",
        });
    }
    if word(&source, 0x4e)? != 0 {
        let actor = usize::try_from(actor - 10)
            .map_err(|_| RuleError::Invalid("invalid retained enemy actor".into()))?;
        let record = c
            .enemies
            .records
            .get_mut(actor)
            .ok_or_else(|| RuleError::Invalid("retained enemy actor missing".into()))?;
        put_word(&mut record.bytes, 0x4e, 0);
    }
    let p = position(c, attacker);
    let q = position(c, target);
    if p.0 < q.0 && source[0x107] == 0 {
        c.enemies.records[a].bytes[0x107] = 1;
    } else if q.0 < p.0 && source[0x107] == 1 {
        c.enemies.records[a].bytes[0x107] = 0;
    }
    let mut hit = 33_i16
        .wrapping_add(word(&source, 0)?.wrapping_mul(5))
        .wrapping_add(word(&source, 0x40)?.wrapping_mul(5));
    for offset in [0xb4, 0xd6] {
        hit = hit.wrapping_sub(i16::from(source[offset] as i8).wrapping_abs());
    }
    for (offset, penalty) in [(0xb8, 15), (0xcd, 15), (0xcf, 10)] {
        if source[offset] != 0 {
            hit = hit.wrapping_sub(penalty);
        }
    }
    for (offset, sign) in [(0xb4, 1_i16), (0xd7, 1), (0xd8, -1)] {
        hit = hit.wrapping_add(
            i16::from(victim[offset] as i8)
                .wrapping_abs()
                .wrapping_mul(sign),
        );
    }
    for (offset, bonus) in [(0xb8, 15), (0xcd, 15), (0xcf, 10)] {
        if victim[offset] != 0 {
            hit = hit.wrapping_add(bonus);
        }
    }
    hit = hit.wrapping_sub(word(victim, 8)?);
    if (source[0xac] != 0 || source[0xad] != 0) && victim[0xc8] != 0 {
        hit = hit.wrapping_sub(10);
    }
    if random.bound(100)
        > hit.clamp(
            15_i16.wrapping_sub(c.difficulty.wrapping_mul(5)),
            90_i16.wrapping_sub(c.difficulty.wrapping_mul(5)),
        )
    {
        return Ok((
            AttackResult {
                damage: 0,
                killed: false,
                sounds: vec![],
            },
            packet,
        ));
    }
    let bonus = word(&source, 0x40)?.wrapping_add(i16::from(source[0xd2] as i8));
    put_word(
        &mut packet,
        32,
        bonus.wrapping_add(i16::from(source[selected] as i8)),
    );
    put_word(
        &mut packet,
        34,
        bonus.wrapping_add(i16::from(source[selected + 1] as i8)),
    );
    let impact = i16::from(source[selected + 2] as i8);
    put_word(&mut packet, 38, if impact == 0 { 100 } else { impact });
    let resolved = resolve_damage(
        &mut c.enemies.records[t].bytes,
        true,
        &DamagePacket {
            bytes: packet.clone(),
            attack_count: 0,
        },
        DamageFlags {
            armor: true,
            saves: true,
            ..Default::default()
        },
        random,
    )?;
    let mut sounds = Vec::new();
    let mut extra_damage = 0_i16;
    if resolved.total != 0 && condition != 0 {
        let strength = random
            .range(word(&source, 0)? / 2, word(&source, 0)?)
            .max(1);
        if condition == 14 {
            extra_damage = random.bound(i16::from(source[0x94 + index * 4] as i8));
            let victim = &c.enemies.records[t].bytes;
            let threshold = i16::from(victim[0x8f] as i8);
            if random.bound(100) <= threshold || threshold > 99 {
                extra_damage /= 2;
            }
            if victim[0xbe] != 0 {
                extra_damage /= 2;
            }
            if extra_damage != 0 {
                sounds.push(344);
            }
        } else {
            let (save, offset) = match condition {
                2 => (7, 0xb3),
                4 => (8, 0xb4),
                6 => (0, 0xcf),
                9 => (7, 0xce),
                _ => unreachable!(),
            };
            let m = &mut c.enemies.records[t].bytes;
            let threshold = i16::from(m[0x8b + save] as i8);
            let saved = random.bound(100) <= threshold || threshold > 99;
            if !saved && (m[offset] as i8) >= 0 {
                m[offset] = (m[offset] as i8).wrapping_add(strength.wrapping_abs() as i8) as u8;
                if condition == 2 {
                    m[0x109] = 0;
                    m[0x106] = 0;
                }
                if condition == 4 {
                    m[0x109] = ((m[0x109] as i8) / 2) as u8;
                    m[0x106] = ((m[0x106] as i8) / 2) as u8;
                }
                if condition == 9 {
                    sounds.push(104);
                }
            }
        }
    }
    let health = word(&c.enemies.records[t].bytes, 0x110)?
        .wrapping_sub(resolved.total)
        .wrapping_sub(extra_damage);
    if resolved.total != 0 {
        put_word(&mut c.enemies.records[t].bytes, 0x110, health);
        c.enemies.records[t].bytes[0x105] = 1;
    }
    if health < 1 {
        lifecycle::kill_body(c, warriors, target)?;
    }
    Ok((
        AttackResult {
            damage: resolved.total.wrapping_add(extra_damage),
            killed: health < 1,
            sounds,
        },
        packet,
    ))
}

/// Select result audio from the retained attack index and loaded impact word.
pub fn result_audio(
    attacker: &MonsterRecord,
    index: usize,
    damage: i16,
    loaded_impact: i16,
) -> Result<i16, RuleError> {
    if attacker.bytes.len() != 288 || index >= 6 {
        return Err(RuleError::Invalid(
            "monster result audio record or index".into(),
        ));
    }
    if damage == 0 {
        return Ok(650);
    }
    if word(&attacker.bytes, 0x4e)? != 0 {
        return Ok(loaded_impact);
    }
    let selected = if attacker.bytes[0x93 + index * 4] == 0 {
        0
    } else {
        index
    };
    Ok(i16::from(attacker.bytes[0x95 + selected * 4] as i8))
}

fn packet(weapon: &[u8], ammo: &[u8], attacks: &[u8]) -> Result<Vec<u8>, RuleError> {
    let mut p = centurions_content::span(attacks, 3 * 2842, 58)?.to_vec();
    p[0] = weapon[77];
    p[17] = weapon[55];
    p[18] = weapon[53];
    p[2] = p[2].wrapping_add(word(weapon, 60)?.wrapping_add(word(ammo, 60)?) as u8);
    p[5] = p[5].wrapping_add(weapon[81]).wrapping_add(ammo[81]);
    put_word(&mut p, 34, word(weapon, 62)?.wrapping_add(word(ammo, 62)?));
    if word(&p, 34)? != 0 {
        put_word(&mut p, 32, 1);
    }
    for (dest, offset) in [
        (23, 140),
        (24, 64),
        (25, 66),
        (26, 68),
        (27, 70),
        (28, 72),
        (29, 74),
    ] {
        p[dest] = word(weapon, offset)?.wrapping_add(word(ammo, offset)?) as u8;
    }
    for (dest, offset) in [(36, 12), (38, 14)] {
        put_word(
            &mut p,
            dest,
            if word(ammo, offset)? != 0 {
                word(ammo, offset)?
            } else {
                word(weapon, offset)?
            },
        );
    }
    p[1] = word(weapon, 56)?.wrapping_add(word(ammo, 56)?) as u8;
    if word(ammo, 56)? != 0 {
        p[1] = word(ammo, 56)? as u8;
    }
    p[31] = weapon[59];
    if word(ammo, 58)? != 0 {
        p[31] = word(ammo, 58)? as u8;
    }
    let rounds = word(weapon, 22)?;
    let factor = f32::from(100_i16.wrapping_add(i16::from(ammo[85] as i8))) / 100.0;
    for offset in [32, 34] {
        let value = (f32::from(word(&p, offset)?.wrapping_mul(rounds)) * factor).trunc() as i16;
        put_word(&mut p, offset, value);
    }
    for value in &mut p[23..30] {
        *value = (f32::from(value.wrapping_mul(rounds as u8)) * factor).trunc() as u8;
    }
    Ok(p)
}

#[allow(clippy::too_many_arguments)]
fn fire(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
    events: &mut Vec<Event>,
    actor: (i16, i16),
    buffers: &mut crate::ammunition::WorkBuffers,
    display_writers: bool,
) -> Result<bool, RuleError> {
    let (item_context, body) = actor;
    let a = (body - 10) as usize;
    let source = c.enemies.records[a].bytes.clone();
    if word(&source, 0x26)? != 1 || word(&source, 0x4c)? != 201 {
        return Err(RuleError::Unsupported {
            action: "combat.scientist_firearm",
            dependency: "other scientist weapon or ammunition",
        });
    }
    let weapon = items
        .get(1)
        .ok_or_else(|| RuleError::Invalid("scientist weapon missing".into()))?;
    let ammo = items
        .get(if item_context == 0 { 0 } else { 201 })
        .ok_or_else(|| RuleError::Invalid("scientist ammunition missing".into()))?;
    let attacks = &mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("combat content missing".into()))?
        .attacks;
    put_word(&mut c.enemies.records[a].bytes, 0x4e, 1);
    let p = packet(weapon, ammo, attacks)?;
    c.weapon_work.prepare(weapon, ammo)?;
    buffers.prepare_pair(weapon, ammo)?;
    c.attack_packet = Some(p.clone());
    c.attack_identity = Some([3, 0, 0]);
    random.bound(7);
    let shots = i16::from(weapon[77] as i8);
    let range = i16::from(p[0] as i8)
        .wrapping_mul(shots)
        .wrapping_add(i16::from(p[2] as i8))
        .wrapping_abs();
    let start = position(c, body);
    let mut distances = [127_i16; 110];
    let mut count = 0;
    let mut maximum = 0;
    for target in 0..10 + c.enemies.records.len() as i16 {
        if !alive(c, w, target) || faction(c, w, body)? == faction(c, w, target)? {
            continue;
        }
        let end = position(c, target);
        let dx = f32::from(start.0 - end.0);
        let dy = f32::from(start.1 - end.1);
        let distance = (dx * dx + dy * dy).sqrt().trunc() as i16;
        maximum = maximum.max(distance);
        if distance <= range && lifecycle::can_see(c, mission, start, end, 0)? {
            distances[target as usize] = distance;
            count += 1;
        }
    }
    if count == 0 || range < maximum {
        return Ok(false);
    }
    let mut selected = None;
    for _ in 0..100 {
        let target = (random.bound(c.enemies.records.len() as i16 + 10) - 1) as usize;
        if distances[target] <= range {
            selected = Some(target);
            break;
        }
    }
    let Some(target) = selected else {
        return Ok(false);
    };
    if target < 10 {
        return Err(RuleError::Unsupported {
            action: "combat.scientist_firearm",
            dependency: "scientist targeting warrior",
        });
    }
    let m = &mut c.enemies.records[a].bytes;
    m[0x109] = (m[0x109] as i8).wrapping_sub(2) as u8;
    m[0x106] = (m[0x106] as i8).wrapping_sub(3) as u8;
    m[0x104] = u8::from((m[0x109] as i8) >= 1);
    if display_writers {
        runtime::refresh_body_item(c, w, items, body, buffers)?;
    }
    events.push(Event::Sound {
        resource_id: word(&p, 36)?,
    });
    random.range(i16::from(p[13] as i8), i16::from(p[14] as i8));
    let t = target - 10;
    let victim = &c.enemies.records[t].bytes;
    let mut hit = 33_i16;
    if c.active_body().is_some_and(|actor| actor > 9) {
        hit = hit
            .wrapping_add(word(&source, 0)?.wrapping_mul(5))
            .wrapping_add(word(&source, 0x66)?);
    }
    if victim[0xb3] == 0 {
        hit = hit
            .wrapping_sub(word(victim, 0x62)?)
            .wrapping_sub(random.bound(word(victim, 4)? / 3))
            .wrapping_sub(word(victim, 0)?.wrapping_mul(3))
            .wrapping_sub(i16::from(victim[0x104] as i8).wrapping_mul(word(victim, 0)?));
        hit = hit.wrapping_add(match word(victim, 0x10)?.wrapping_abs() {
            0 => -5,
            1 | 2 => 5,
            3 => 10,
            _ => 0,
        });
        if random.bound(100)
            > hit.clamp(
                15_i16.wrapping_sub(c.difficulty.wrapping_mul(5)),
                90_i16.wrapping_sub(c.difficulty.wrapping_mul(5)),
            )
        {
            return Ok(true);
        }
    }
    let result = resolve_damage(
        &mut c.enemies.records[t].bytes,
        true,
        &DamagePacket {
            bytes: p.clone(),
            attack_count: shots,
        },
        DamageFlags {
            shield: true,
            armor: true,
            saves: true,
            ..Default::default()
        },
        random,
    )?;
    if result.shield_sound {
        events.push(Event::Sound { resource_id: 206 });
    }
    if result.total != 0 {
        let hp = word(&c.enemies.records[t].bytes, 0x110)?.wrapping_sub(result.total);
        put_word(&mut c.enemies.records[t].bytes, 0x110, hp);
        if hp < 1 {
            lifecycle::kill_body(c, w, target as i16)?;
        } else {
            events.push(Event::Sound {
                resource_id: word(&p, 38)?,
            });
            events.push(Event::Sound {
                resource_id: result_audio(&c.enemies.records[a], 0, result.total, word(&p, 38)?)?,
            });
        }
        events.push(Event::AttackResolved {
            attacker: body,
            target: target as i16,
            damage: result.total,
            defeated: hp < 1,
        });
    }
    Ok(true)
}

pub fn scientist_turn_with_item_context(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
    item_context: i16,
) -> Result<Vec<Event>, RuleError> {
    scientist_turn_with_item_context_and_buffs(c, w, mission, random, items, item_context, &[0; 20])
}

pub fn scientist_turn_with_item_context_and_buffs(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
    item_context: i16,
    buffs: &[u8],
) -> Result<Vec<Event>, RuleError> {
    scientist_turn_inner(
        c,
        w,
        mission,
        random,
        items,
        item_context,
        buffs,
        &mut Default::default(),
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn scientist_turn_with_work(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
    item_context: i16,
    buffs: &[u8],
    buffers: &mut crate::ammunition::WorkBuffers,
) -> Result<Vec<Event>, RuleError> {
    scientist_turn_inner(
        c,
        w,
        mission,
        random,
        items,
        item_context,
        buffs,
        buffers,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
fn scientist_turn_inner(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
    mut item_context: i16,
    buffs: &[u8],
    buffers: &mut crate::ammunition::WorkBuffers,
    display_writers: bool,
) -> Result<Vec<Event>, RuleError> {
    let body = c
        .active_body()
        .ok_or_else(|| RuleError::Invalid("active scientist missing".into()))?;
    if body < 10 {
        return Err(RuleError::Invalid("active scientist is warrior".into()));
    }
    let e = (body - 10) as usize;
    if c.enemies.records[e].bytes[0x103] != 65 {
        return Err(RuleError::Unsupported {
            action: "combat.scientist",
            dependency: "other ally definition",
        });
    }
    let attacks = &mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("combat tables missing".into()))?
        .attacks;
    let mut events = vec![];
    let mut attempt = 0;
    for _ in 0..32 {
        let m = &mut c.enemies.records[e].bytes;
        if (m[0x109] as i8) < 2 {
            return Ok(events);
        }
        m[0x104] = u8::from((m[0x109] as i8) >= 1);
        if m[0xb2] != 0 {
            return Err(RuleError::Unsupported {
                action: "combat.scientist",
                dependency: "fleeing ally",
            });
        }
        if random.bound(100) <= word(m, 0x48)? {
            if fire(
                c,
                w,
                mission,
                random,
                items,
                &mut events,
                (item_context, body),
                buffers,
                display_writers,
            )? {
                if let Some(item) = buffers.item.as_ref() {
                    item_context = word(item, 32)?;
                }
                continue;
            }
            attempt += 1;
        } else if random.bound(100) <= word(m, 0x42)? {
            return Err(RuleError::Unsupported {
                action: "combat.scientist",
                dependency: "ally spell",
            });
        }
        events.extend(if display_writers {
            runtime::check_neighbors_with_work(c, w, random, body, items, attacks, buffs, buffers)?
        } else {
            runtime::check_neighbors_with_buffs(c, w, random, body, items, attacks, buffs)?
        });
        if let Some(item) = buffers.item.as_ref() {
            item_context = word(item, 32)?;
        }
        for _ in 0..128 {
            if !alive(c, w, body) || !w.iter().any(|w| w.bytes[0x9df] != 0) {
                break;
            }
            let m = &c.enemies.records[e].bytes;
            if (m[0x106] as i8) <= 0 || (m[0x109] as i8) <= 1 || m[0xb3] != 0 {
                break;
            }
            runtime::select_enemy_target(c, w, mission, random, body)?;
            let moved = enemy::move_enemy(c, mission, e, random)?;
            events.extend(
                moved
                    .sounds
                    .into_iter()
                    .map(|resource_id| Event::Sound { resource_id }),
            );
            if moved.moved {
                let p = position(c, body);
                events.push(Event::TacticalMoved {
                    body,
                    x: p.0,
                    y: p.1,
                });
            }
            events.extend(if display_writers {
                runtime::check_neighbors_with_work(
                    c, w, random, body, items, attacks, buffs, buffers,
                )?
            } else {
                runtime::check_neighbors_with_buffs(c, w, random, body, items, attacks, buffs)?
            });
            if let Some(item) = buffers.item.as_ref() {
                item_context = word(item, 32)?;
            }
            if !moved.moved {
                break;
            }
        }
        let m = &c.enemies.records[e].bytes;
        if word(m, 0x48)? == 0 || (m[0x109] as i8) < 2 || attempt > 1 {
            return Ok(events);
        }
        attempt += 1;
        if fire(
            c,
            w,
            mission,
            random,
            items,
            &mut events,
            (item_context, body),
            buffers,
            display_writers,
        )? {
            if let Some(item) = buffers.item.as_ref() {
                item_context = word(item, 32)?;
            }
            continue;
        }
        return Ok(events);
    }
    Err(RuleError::Invalid("scientist action point budget".into()))
}

/// Execute the scientist with a zero loaditem work-record ammo selector.
pub fn scientist_turn(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
    items: &[Vec<u8>],
) -> Result<Vec<Event>, RuleError> {
    scientist_turn_with_item_context(c, w, mission, random, items, 0)
}
