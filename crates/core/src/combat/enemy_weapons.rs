use crate::combat::{Combat, attack, damage, firearm, put_word};
use crate::{RandomState, RuleError};
use attack::AttackResult;
use centurions_content::combat::MonsterRecord;
use centurions_content::{Warrior, word};
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
    let (result, packet, _) = melee_warrior_with_target(
        combat,
        warriors,
        random,
        bodies,
        weapon,
        difficulty,
        Some(attacks),
        retained,
        buffs,
    )?;
    Ok((result, packet))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn melee_warrior_with_target(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    weapon: &[u8],
    difficulty: i16,
    attacks: Option<&[u8]>,
    retained: &[u8],
    buffs: &[u8],
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
    let (attacker, target, index, actor) = bodies;
    let body_index = |body: i16| {
        usize::try_from(i32::from(body) - 10)
            .ok()
            .filter(|index| *index < combat.enemies.records.len())
            .ok_or_else(|| RuleError::Invalid("enemy melee body outside records".into()))
    };
    let a = body_index(attacker)?;
    let actor = usize::try_from(actor - 10)
        .ok()
        .filter(|index| *index < 100)
        .ok_or("invalid retained melee actor")?;
    let target = usize::try_from(target)
        .ok()
        .filter(|index| *index < 6 && *index < warriors.len())
        .ok_or_else(|| RuleError::Invalid("enemy melee warrior outside party".into()))?;
    if retained.len() != 58
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
        put_word(&mut next.story_record_mut(actor)?.bytes, 0x4e, id);
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
        attacks
            .map(attack::attack_packet)
            .transpose()?
            .unwrap_or_else(attack::default_attack_packet),
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
    let (result, packet, _) = melee_enemy_with_target(
        combat,
        random,
        bodies,
        weapon,
        difficulty,
        Some(attacks),
        retained,
    )?;
    Ok((result, packet))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn melee_enemy_with_target(
    combat: &mut Combat,
    random: &mut RandomState,
    bodies: (i16, i16, usize, i16),
    weapon: &[u8],
    difficulty: i16,
    attacks: Option<&[u8]>,
    retained: &[u8],
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
    if retained.len() != 58 {
        return Err(RuleError::Invalid("retained attack packet size".into()));
    }
    melee_enemy_inner(
        combat,
        random,
        bodies,
        weapon,
        difficulty,
        attacks
            .map(attack::attack_packet)
            .transpose()?
            .unwrap_or_else(attack::default_attack_packet),
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
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
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
        usize::try_from(actor - 10)
            .ok()
            .filter(|index| *index < 100)
            .ok_or("invalid retained melee actor")?,
    );
    if a == t || index >= 6 || weapon.len() != 150 || combat.enemies.positions.len() <= a.max(t) {
        return Err(RuleError::Invalid("enemy melee body or definition".into()));
    }
    let source = &combat.enemies.records[a].bytes;
    let victim = &combat.enemies.records[t].bytes;
    if source.len() != 288 || victim.len() != 288 {
        return Err(RuleError::Invalid("enemy melee record size".into()));
    }
    let id = word(source, 0x28)?;
    let loaded = word(source, 0x4e)? != 0;
    let bonus = word(source, 0x40)?.wrapping_add(i16::from(source[0xd2] as i8));
    let condition = source[0x96 + index * 4];
    if word(weapon, 26)? != 2 || word(weapon, 42)? != 0 {
        return Err(RuleError::Unsupported {
            action: "combat.enemy_melee",
            dependency: "nonphysical melee weapon definition",
        });
    }
    let mut next = combat.clone();
    let mut rng = random.clone();
    let source = source.clone();
    if loaded {
        put_word(&mut next.story_record_mut(actor)?.bytes, 0x4e, id);
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
    let immunity = word(victim, 0xc)?;
    let rejected = immunity == -1 && loaded && word(weapon, 100)? != -1
        || immunity == -2 && loaded && word(weapon, 100)? != -2
        || !matches!(immunity, -2..=0)
            && (if loaded { word(weapon, 102)? } else { 0 }).wrapping_sub(1024) != immunity;
    if hit && rejected {
        let feedback = Some(attack::AttackFeedback {
            code: match immunity {
                -1 => -50,
                -2 => -51,
                _ => -45,
            },
            amount: 0,
            recipient: attack::AttackRecipient::Attacker,
        });
        if loaded && matches!(immunity, -1 | -2) {
            put_word(&mut next.enemies.records[a].bytes, 0x4e, 0);
        }
        *combat = next;
        *random = rng;
        return Ok((
            AttackResult {
                feedback,
                damage: 0,
                killed: false,
                sounds: vec![],
            },
            packet,
            false,
        ));
    }
    let reflected = victim[0xd1] != 0 && rng.bound(100) < 34 && word(&source, 0xc)? == 0;
    let target = &mut next.enemies.records[if reflected { a } else { t }].bytes;
    let mut damage = if hit {
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
        if reflected {
            let result =
                attack::resolve_reflected_attack(target, &packet, index, !loaded, &mut rng)?;
            *combat = next;
            *random = rng;
            return Ok((result, packet, true));
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
    let mut clear_target = false;
    let mut feedback = None;
    if damage != 0 {
        if !loaded {
            if let Some(effect) =
                super::natural_conditions::monster_attack_effect(target, &source, index, &mut rng)?
            {
                feedback = effect.feedback;
                damage = damage.wrapping_add(effect.extra_damage);
                if effect.applied && condition == 5 {
                    sounds.push(667);
                }
                clear_target = effect.applied && condition == 10;
            } else {
                damage = 0;
            }
        }
        target[0x105] = 1;
        let health = word(target, 0x110)?.wrapping_sub(damage);
        put_word(target, 0x110, health);
    }
    let result = AttackResult {
        feedback,
        damage,
        killed: word(target, 0x110)? < 1,
        sounds,
    };
    if clear_target {
        put_word(&mut next.enemies.records[a].bytes, 0x10c, -1);
    }
    *combat = next;
    *random = rng;
    Ok((result, packet, reflected))
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
    let (result, packet, _) = melee_inner(
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
    )?;
    Ok((result, packet))
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
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
    super::party_buffs::validate(buffs)?;
    if warrior.bytes.len() != 3138 || enemy.bytes.len() != 288 || weapon.len() != 150 || index >= 6
    {
        return Err(RuleError::Invalid("enemy melee record size".into()));
    }
    let id = word(&enemy.bytes, 0x28)?;
    let loaded = word(&enemy.bytes, 0x4e)? != 0;
    if !matches!(
        enemy.bytes[0x96 + index * 4],
        0 | 1 | 2 | 4 | 5 | 6 | 9 | 10 | 11 | 14 | 17
    ) || word(weapon, 26)? != 2
        || word(weapon, 42)? != 0
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
    let hit = next.bound(100) <= hit.clamp(minimum, maximum);
    let reflected = word(w, 0x9c)? != 0 && next.bound(100) < 34 && word(m, 0xc)? == 0;
    if !hit {
        if !reflected {
            increment(w, 0xa56, 1);
        }
        *warrior = victim;
        *enemy = attacker;
        *random = next;
        return Ok((
            AttackResult {
                feedback: None,
                damage: 0,
                killed: false,
                sounds: vec![],
            },
            packet,
            reflected,
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
    if reflected {
        let result = attack::resolve_reflected_attack(m, &packet, index, !loaded, &mut next)?;
        *warrior = victim;
        *enemy = attacker;
        *random = next;
        return Ok((result, packet, true));
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
    let mut feedback = None;
    if damage == 0 {
        increment(w, 0xa56, 1);
    } else {
        let natural = if !loaded {
            Some(super::natural_conditions::warrior_attack_effect(
                w,
                m,
                index,
                word(buffs, 14)? != 0,
                &mut next,
            )?)
        } else {
            None
        };
        if natural.as_ref().is_some_and(|effect| effect.applied) && m[0x96 + index * 4] == 5 {
            sounds.push(667);
        }
        if let Some(adjusted) = super::party_buffs::attack2_damage(buffs, 0, damage)? {
            damage = adjusted;
            sounds.push(-694);
        }
        if let Some(natural) = natural {
            feedback = natural.feedback;
            damage = damage.wrapping_add(natural.extra_damage);
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
        feedback,
        damage,
        killed: word(w, 0x50)? < 1,
        sounds,
    };
    *warrior = victim;
    *enemy = attacker;
    *random = next;
    Ok((result, packet, false))
}
