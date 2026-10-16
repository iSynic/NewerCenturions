use super::*;
use attack::AttackResult;
use damage::{DamageFlags, DamagePacket, resolve_damage};
use firearm::PreparedFirearm;

pub fn ammo_id(enemy: &MonsterRecord, item_context: i16) -> Result<i16, RuleError> {
    Ok(if item_context == 0 {
        0
    } else {
        word(&enemy.bytes, 0x4c)?
    })
}

pub fn prepare(
    enemy: &mut MonsterRecord,
    weapon: &[u8],
    ammo: &[u8],
    attacks: &[u8],
) -> Result<PreparedFirearm, RuleError> {
    if enemy.bytes.len() != 288 || weapon.len() != 150 || ammo.len() != 150 {
        return Err(RuleError::Invalid("enemy weapon record size".into()));
    }
    let id = word(&enemy.bytes, 0x26)?;
    if id <= 0 || !matches!(word(weapon, 44)?, 4101 | 4102) {
        return Err(RuleError::Unsupported {
            action: "combat.enemy_firearm",
            dependency: "enemy firearm outside recovered definitions",
        });
    }
    let packet = firearm::prepare_packet(weapon, ammo, attacks)?;
    if packet[19] != 9 || packet[20] != 9 || word(&packet, 40)? != 0 {
        return Err(RuleError::Unsupported {
            action: "combat.enemy_firearm",
            dependency: "enemy firearm status effects",
        });
    }
    put_word(&mut enemy.bytes, 0x4e, id);
    Ok(PreparedFirearm {
        sound: word(&packet, 38)?,
        packet,
        slot: 0,
        shots: i16::from(weapon[77] as i8),
        rounds: word(weapon, 22)?,
        weight: 0,
        armor_filter: word(weapon, 136)?.wrapping_add(word(ammo, 136)?) != 0,
    })
}

pub fn resolve(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    prepared: &PreparedFirearm,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    if warrior.bytes.len() != 3138 || enemy.bytes.len() != 288 || prepared.packet.len() != 58 {
        return Err(RuleError::Invalid("enemy firearm record size".into()));
    }
    let mut victim = warrior.clone();
    let mut attacker = enemy.clone();
    let mut next = random.clone();
    let result =
        firearm::resolve_enemy(&mut victim, &mut attacker, prepared, &mut next, difficulty)?;
    *warrior = victim;
    *enemy = attacker;
    *random = next;
    Ok(result)
}

fn increment(bytes: &mut [u8], offset: usize, value: i32) {
    let current = i32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap());
    bytes[offset..offset + 4].copy_from_slice(&current.wrapping_add(value).to_be_bytes());
}

pub fn melee(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    weapon: &[u8],
    random: &mut RandomState,
    difficulty: i16,
    attacker_x: i16,
    target_x: i16,
) -> Result<AttackResult, RuleError> {
    Ok(melee_inner(
        warrior,
        enemy,
        weapon,
        random,
        difficulty,
        attacker_x,
        target_x,
        0,
        attack::default_attack_packet(),
        &[0; 20],
    )?
    .0)
}

#[allow(clippy::too_many_arguments)]
pub fn melee_warrior_with_packet(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    weapon: &[u8],
    difficulty: i16,
    attacks: &[u8],
    retained: &[u8],
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    melee_warrior_with_buffs(
        combat, warriors, random, bodies, weapon, difficulty, attacks, retained, &[0; 20],
    )
}

#[allow(clippy::too_many_arguments)]
pub fn melee_warrior_with_buffs(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    weapon: &[u8],
    difficulty: i16,
    attacks: &[u8],
    retained: &[u8],
    buffs: &[u8],
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    let (attacker, target, index, actor) = bodies;
    let body_index = |body: i16| {
        usize::try_from(i32::from(body) - 10)
            .ok()
            .filter(|index| *index < combat.enemies.records.len())
            .ok_or_else(|| RuleError::Invalid("enemy melee body outside records".into()))
    };
    let (a, actor) = (body_index(attacker)?, body_index(actor)?);
    let target = usize::try_from(target)
        .ok()
        .filter(|index| *index < 6 && *index < warriors.len())
        .ok_or_else(|| RuleError::Invalid("enemy melee warrior outside party".into()))?;
    if retained.len() != 58
        || combat.enemies.records[actor].bytes.len() != 288
        || combat.enemies.positions.len() <= a
        || combat.squad.positions.len() <= target
    {
        return Err(RuleError::Invalid("enemy melee context size".into()));
    }
    let source = &combat.enemies.records[a].bytes;
    let loaded = word(source, 0x4e)? != 0;
    let id = word(source, 0x28)?;
    let mut next = combat.clone();
    let mut next_warriors = warriors.to_vec();
    let mut rng = random.clone();
    if loaded {
        put_word(&mut next.enemies.records[actor].bytes, 0x4e, id);
    }
    let result = melee_inner(
        &mut next_warriors[target],
        &mut next.enemies.records[a],
        weapon,
        &mut rng,
        difficulty,
        combat.enemies.positions[a].0,
        combat.squad.positions[target].0,
        index,
        attack::attack_packet(attacks)?,
        buffs,
    )?;
    *combat = next;
    warriors.clone_from_slice(&next_warriors);
    *random = rng;
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
pub fn melee_enemy_with_packet(
    combat: &mut Combat,
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    weapon: &[u8],
    difficulty: i16,
    attacks: &[u8],
    retained: &[u8],
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    if retained.len() != 58 {
        return Err(RuleError::Invalid("retained attack packet size".into()));
    }
    melee_enemy_inner(
        combat,
        random,
        bodies,
        weapon,
        difficulty,
        attack::attack_packet(attacks)?,
    )
}

pub fn melee_enemy(
    combat: &mut Combat,
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    weapon: &[u8],
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    Ok(melee_enemy_inner(
        combat,
        random,
        bodies,
        weapon,
        difficulty,
        attack::default_attack_packet(),
    )?
    .0)
}

fn melee_enemy_inner(
    combat: &mut Combat,
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    weapon: &[u8],
    difficulty: i16,
    mut packet: Vec<u8>,
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    let (attacker, target, index, actor) = bodies;
    let body_index = |body: i16| {
        usize::try_from(i32::from(body) - 10)
            .ok()
            .filter(|index| *index < combat.enemies.records.len())
            .ok_or_else(|| RuleError::Invalid("enemy melee body outside records".into()))
    };
    let (a, t, actor) = (
        body_index(attacker)?,
        body_index(target)?,
        body_index(actor)?,
    );
    if a == t || index >= 6 || weapon.len() != 150 || combat.enemies.positions.len() <= a.max(t) {
        return Err(RuleError::Invalid("enemy melee body or definition".into()));
    }
    let source = &combat.enemies.records[a].bytes;
    let victim = &combat.enemies.records[t].bytes;
    if source.len() != 288
        || victim.len() != 288
        || combat.enemies.records[actor].bytes.len() != 288
    {
        return Err(RuleError::Invalid("enemy melee record size".into()));
    }
    let id = word(source, 0x28)?;
    let loaded = word(source, 0x4e)? != 0;
    let bonus = word(source, 0x40)?.wrapping_add(i16::from(source[0xd2] as i8));
    let no_effect_charm =
        matches!(id, 177 | 178) && bonus == 0 && matches!(source[0x96 + index * 4], 0 | 10);
    let condition = source[0x96 + index * 4];
    let guardian_poison = id == 187 && condition == 5;
    if !matches!(id, 177 | 178 | 181 | 187 | 192 | 194)
        || (!loaded && !no_effect_charm && !guardian_poison && condition != 0)
        || word(weapon, 26)? != 2
        || word(weapon, 42)? != 0
        || word(victim, 0xc)? != 0
        || victim[0xd1] != 0
    {
        return Err(RuleError::Unsupported {
            action: "combat.enemy_melee",
            dependency: "enemy victim reflection, immunity or effective attack condition",
        });
    }
    let mut next = combat.clone();
    let mut rng = random.clone();
    let source = source.clone();
    if loaded {
        put_word(&mut next.enemies.records[actor].bytes, 0x4e, id);
    }
    let (attacker_x, target_x) = (next.enemies.positions[a].0, next.enemies.positions[t].0);
    if attacker_x < target_x && source[0x107] == 0 {
        next.enemies.records[a].bytes[0x107] = 1;
    } else if target_x < attacker_x && source[0x107] == 1 {
        next.enemies.records[a].bytes[0x107] = 0;
    }
    let mut hit = 33_i16
        .wrapping_add(word(&source, 0)?.wrapping_mul(5))
        .wrapping_add(word(&source, 0x40)?.wrapping_mul(5))
        .wrapping_add(if loaded {
            i16::from(weapon[81] as i8)
        } else {
            0
        });
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
    for (offset, adjustment) in [(0xb8, 15), (0xcd, 15), (0xcf, 10)] {
        if victim[offset] != 0 {
            hit = hit.wrapping_add(adjustment);
        }
    }
    hit = hit.wrapping_sub(word(victim, 8)?);
    if (source[0xac] != 0 || source[0xad] != 0) && victim[0xc8] != 0 {
        hit = hit.wrapping_sub(10);
    }
    let hit = rng.bound(100)
        <= hit.clamp(
            15_i16.wrapping_sub(difficulty.wrapping_mul(5)),
            90_i16.wrapping_sub(difficulty.wrapping_mul(5)),
        );
    let target = &mut next.enemies.records[t].bytes;
    let damage = if hit {
        target[0x105] = 1;
        put_word(&mut packet, 32, bonus.wrapping_add(1));
        put_word(
            &mut packet,
            34,
            bonus.wrapping_add(if loaded { word(weapon, 40)? } else { 0 }),
        );
        for (destination, offset) in [
            (23, 140),
            (24, 64),
            (25, 66),
            (26, 68),
            (27, 70),
            (28, 72),
            (29, 74),
        ] {
            packet[destination] = if loaded {
                word(weapon, offset)? as u8
            } else {
                0
            };
        }
        resolve_damage(
            target,
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
            &mut rng,
        )?
        .total
    } else {
        0
    };
    let mut sounds = vec![];
    if damage != 0 {
        if !loaded && guardian_poison {
            let strength = rng.range(word(&source, 0)? / 2, word(&source, 0)?).max(1);
            let saved = rng.bound(100) <= i16::from(target[0x8c] as i8);
            if !saved && (target[0xb5] as i8) >= 0 {
                target[0xb5] =
                    (target[0xb5] as i8).wrapping_add(strength.wrapping_abs() as i8) as u8;
                sounds.push(667);
            }
        }
        target[0x105] = 1;
        let health = word(target, 0x110)?.wrapping_sub(damage);
        put_word(target, 0x110, health);
    }
    let result = AttackResult {
        damage,
        killed: word(target, 0x110)? < 1,
        sounds,
    };
    *combat = next;
    *random = rng;
    Ok((result, packet))
}

#[allow(clippy::too_many_arguments)]
pub fn melee_with_packet(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    weapon: &[u8],
    random: &mut RandomState,
    difficulty: i16,
    attacker_x: i16,
    target_x: i16,
    attacks: &[u8],
    retained: &[u8],
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    melee_with_buffs(
        warrior, enemy, weapon, random, difficulty, attacker_x, target_x, attacks, retained,
        &[0; 20],
    )
}

#[allow(clippy::too_many_arguments)]
pub fn melee_with_buffs(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    weapon: &[u8],
    random: &mut RandomState,
    difficulty: i16,
    attacker_x: i16,
    target_x: i16,
    attacks: &[u8],
    retained: &[u8],
    buffs: &[u8],
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    if retained.len() != 58 {
        return Err(RuleError::Invalid("retained attack packet size".into()));
    }
    melee_inner(
        warrior,
        enemy,
        weapon,
        random,
        difficulty,
        attacker_x,
        target_x,
        0,
        attack::attack_packet(attacks)?,
        buffs,
    )
}

#[allow(clippy::too_many_arguments)]
fn melee_inner(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    weapon: &[u8],
    random: &mut RandomState,
    difficulty: i16,
    attacker_x: i16,
    target_x: i16,
    index: usize,
    mut packet: Vec<u8>,
    buffs: &[u8],
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    super::party_buffs::validate(buffs)?;
    if warrior.bytes.len() != 3138 || enemy.bytes.len() != 288 || weapon.len() != 150 || index >= 6
    {
        return Err(RuleError::Invalid("enemy melee record size".into()));
    }
    let id = word(&enemy.bytes, 0x28)?;
    let loaded = word(&enemy.bytes, 0x4e)? != 0;
    let no_effect_charm = matches!(id, 177 | 178)
        && word(&enemy.bytes, 0x40)?.wrapping_add(i16::from(enemy.bytes[0xd2] as i8)) == 0
        && (0..6).all(|index| matches!(enemy.bytes[0x96 + index * 4], 0 | 10));
    let guardian_poison = id == 187 && enemy.bytes[0x96 + index * 4] == 5;
    if !matches!(id, 177..=181 | 187 | 192 | 194)
        || (!loaded
            && !no_effect_charm
            && !guardian_poison
            && (0..6).any(|index| enemy.bytes[0x96 + index * 4] != 0))
        || word(weapon, 26)? != 2
        || word(weapon, 42)? != 0
        || word(&warrior.bytes, 0x9c)? != 0
        || word(&warrior.bytes, 0x60)? != 0
    {
        return Err(RuleError::Unsupported {
            action: "combat.enemy_melee",
            dependency: "other enemy melee weapon or reflection/unconscious target",
        });
    }
    let mut victim = warrior.clone();
    let mut attacker = enemy.clone();
    let mut next = random.clone();
    let w = &mut victim.bytes;
    let m = &mut attacker.bytes;
    if loaded {
        put_word(m, 0x4e, id);
    }
    if attacker_x < target_x && m[0x107] == 0 {
        m[0x107] = 1;
    } else if target_x < attacker_x && m[0x107] == 1 {
        m[0x107] = 0;
    }
    let mut hit = 33_i16
        .wrapping_add(word(m, 0)?.wrapping_mul(5))
        .wrapping_add(word(m, 0x40)?.wrapping_mul(5))
        .wrapping_add(if loaded {
            i16::from(weapon[81] as i8)
        } else {
            0
        });
    for offset in [0xb4, 0xd6] {
        hit = hit.wrapping_sub(i16::from(m[offset] as i8).wrapping_abs());
    }
    for (offset, penalty) in [(0xb8, 15), (0xcd, 15), (0xcf, 10)] {
        if m[offset] != 0 {
            hit = hit.wrapping_sub(penalty);
        }
    }
    for (offset, sign) in [(0x62, 1_i16), (0xa8, 1), (0xaa, -1)] {
        hit = hit.wrapping_add(word(w, offset)?.wrapping_abs().wrapping_mul(sign));
    }
    for (offset, bonus) in [(0x6a, 15), (0x94, 15), (0x98, 10)] {
        if word(w, offset)? != 0 {
            hit = hit.wrapping_add(bonus);
        }
    }
    hit = hit.wrapping_sub(word(w, 0xa)?);
    if (m[0xac] != 0 || m[0xad] != 0) && word(w, 0x8a)? != 0 {
        hit = hit.wrapping_sub(10);
    }
    let minimum = 15_i16.wrapping_sub(difficulty.wrapping_mul(5));
    let maximum = 90_i16.wrapping_sub(difficulty.wrapping_mul(5));
    if next.bound(100) > hit.clamp(minimum, maximum) {
        increment(w, 0xa56, 1);
        *warrior = victim;
        *enemy = attacker;
        *random = next;
        return Ok((
            AttackResult {
                damage: 0,
                killed: false,
                sounds: vec![],
            },
            packet,
        ));
    }
    let bonus = word(m, 0x40)?.wrapping_add(i16::from(m[0xd2] as i8));
    put_word(&mut packet, 32, bonus.wrapping_add(1));
    put_word(
        &mut packet,
        34,
        bonus.wrapping_add(if loaded { word(weapon, 40)? } else { 0 }),
    );
    for (destination, source) in [
        (23, 140),
        (24, 64),
        (25, 66),
        (26, 68),
        (27, 70),
        (28, 72),
        (29, 74),
    ] {
        packet[destination] = if loaded {
            word(weapon, source)? as u8
        } else {
            0
        };
    }
    let result = resolve_damage(
        w,
        false,
        &DamagePacket {
            bytes: packet.clone(),
            attack_count: 0,
        },
        DamageFlags {
            armor: true,
            saves: true,
            ..Default::default()
        },
        &mut next,
    )?;
    let mut sounds = vec![];
    let mut damage = result.total;
    if damage == 0 {
        increment(w, 0xa56, 1);
    } else {
        if !loaded && guardian_poison {
            let strength = next.range(word(m, 0)? / 2, word(m, 0)?).max(1);
            let saved = next.bound(100) <= word(w, 0x114)?;
            let previous = word(w, 0x64)?;
            if !saved && previous >= 0 {
                let strength = if word(w, 0x32)? == 3 {
                    0
                } else {
                    strength.wrapping_abs()
                };
                let duration = previous.wrapping_add(strength);
                if duration < 30 {
                    put_word(w, 0x64, duration);
                }
                sounds.push(667);
            }
        }
        if let Some(adjusted) = super::party_buffs::attack2_damage(buffs, 0, damage)? {
            damage = adjusted;
            sounds.push(-694);
        }
        increment(w, 0xa42, i32::from(damage));
        increment(w, 0xa4e, 1);
        let hp = word(w, 0x50)?.wrapping_sub(damage);
        put_word(w, 0x50, hp);
        if hp < 1 && hp > -10 {
            w[0x9de] = 1;
        }
        put_word(w, 0x14, 1);
    }
    let result = AttackResult {
        damage,
        killed: word(w, 0x50)? < 1,
        sounds,
    };
    *warrior = victim;
    *enemy = attacker;
    *random = next;
    Ok((result, packet))
}
