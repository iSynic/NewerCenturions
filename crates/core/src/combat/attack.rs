use super::*;
use damage::{DamageFlags, DamagePacket, resolve_damage};

#[derive(Debug, PartialEq, Eq)]
pub struct AttackResult {
    pub damage: i16,
    pub killed: bool,
    pub sounds: Vec<i16>,
}

fn increment_long(bytes: &mut [u8], offset: usize, value: i32) {
    let current = i32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap());
    bytes[offset..offset + 4].copy_from_slice(&current.wrapping_add(value).to_be_bytes());
}

pub(crate) fn default_attack_packet() -> Vec<u8> {
    let mut packet = vec![0; 58];
    packet[19] = 8;
    packet[20] = 8;
    packet
}

pub(crate) fn attack_packet(attacks: &[u8]) -> Result<Vec<u8>, RuleError> {
    Ok(centurions_content::span(attacks, 3 * 2842 + 2 * 58, 58)?.to_vec())
}

pub fn unarmed_attack(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    Ok(unarmed_attack_inner(
        warrior,
        enemy,
        random,
        default_attack_packet(),
        &[0; 58],
        difficulty,
    )?
    .0)
}

pub fn unarmed_attack_with_packet(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    random: &mut RandomState,
    attacks: &[u8],
    retained: &[u8],
    difficulty: i16,
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    if retained.len() != 58 {
        return Err(RuleError::Invalid("retained attack packet size".into()));
    }
    let base = attack_packet(attacks)?;
    let (mut next_warrior, mut next_enemy, mut next_random) =
        (warrior.clone(), enemy.clone(), random.clone());
    let result = unarmed_attack_inner(
        &mut next_warrior,
        &mut next_enemy,
        &mut next_random,
        base,
        retained,
        difficulty,
    )?;
    *warrior = next_warrior;
    *enemy = next_enemy;
    *random = next_random;
    Ok(result)
}

fn unarmed_attack_inner(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    random: &mut RandomState,
    base: Vec<u8>,
    retained: &[u8],
    difficulty: i16,
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    let mut packet = retained.to_vec();
    let w = &mut warrior.bytes;
    let m = &mut enemy.bytes;
    if word(w, 0xca)? != 0 || m[0xd1] != 0 || word(m, 0xc)? != 0 {
        return Err(RuleError::Unsupported {
            action: "combat.attack",
            dependency: "equipped weapon, damage reflection or weapon immunity",
        });
    }
    if word(w, 0xacc)? >= word(w, 0xaca)? {
        return Ok((
            AttackResult {
                damage: 0,
                killed: false,
                sounds: vec![45],
            },
            packet,
        ));
    }
    packet = base;
    let attacks = word(w, 0xacc)?.wrapping_add(1);
    put_word(w, 0xacc, attacks);
    let mut hit = 33_i16
        .wrapping_add(word(w, 6)?)
        .wrapping_add(word(w, 0xb5a)?);
    for offset in [0x62, 0xa6] {
        hit = hit.wrapping_sub(word(w, offset)?.wrapping_abs());
    }
    for (offset, penalty) in [(0x6a, 15), (0x94, 15), (0x98, 10)] {
        if word(w, offset)? != 0 {
            hit = hit.wrapping_sub(penalty);
        }
    }
    for i in 0..8 {
        if m[0xa8 + i] != 0 {
            hit = hit.wrapping_add(word(w, 0xae + i * 2)?);
        }
    }
    hit = hit.wrapping_sub(word(m, 8)?);
    hit = hit.wrapping_sub(i16::from(m[0xb9] as i8).abs().wrapping_mul(2));
    for (offset, sign) in [(0xb4, 1_i16), (0xd7, 1), (0xd8, -1)] {
        hit = hit.wrapping_add(i16::from(m[offset] as i8).abs().wrapping_mul(sign));
    }
    for (offset, bonus) in [(0xb8, 15), (0xcd, 15), (0xcf, 10)] {
        if m[offset] != 0 {
            hit = hit.wrapping_add(bonus);
        }
    }
    if random.bound(100)
        > hit.clamp(
            15_i16.wrapping_sub(difficulty.wrapping_mul(5)),
            90_i16.wrapping_sub(difficulty.wrapping_mul(5)),
        )
        && m[0xb3] == 0
    {
        increment_long(w, 0xa52, 1);
        return Ok((
            AttackResult {
                damage: 0,
                killed: false,
                sounds: vec![],
            },
            packet,
        ));
    }
    put_word(&mut packet, 32, word(w, 0x9e)?.wrapping_add(1));
    put_word(&mut packet, 34, word(w, 0x9e)?.wrapping_add(word(w, 0xc)?));
    let resolved = resolve_damage(
        m,
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
    let mut damage = resolved.total;
    if damage <= 0 {
        increment_long(w, 0xa52, 1);
        return Ok((
            AttackResult {
                damage: 0,
                killed: false,
                sounds: vec![],
            },
            packet,
        ));
    }
    if random.bound(100) <= word(w, 0xfa)?.wrapping_add(word(w, 0xb68)?) {
        damage = damage.wrapping_mul(2);
    }
    if m[0xb3] != 0 {
        damage = word(m, 0x110)?;
    }
    m[0x105] = 1;
    let health = word(m, 0x110)?.wrapping_sub(damage);
    put_word(m, 0x110, health);
    increment_long(w, 0xa46, i32::from(damage));
    increment_long(w, 0xa4a, 1);
    let killed = health <= 0;
    if killed {
        increment_long(w, 0xa5a, 1);
    }
    let sound = random.bound(2).wrapping_add(401);
    Ok((
        AttackResult {
            damage,
            killed,
            sounds: vec![sound],
        },
        packet,
    ))
}

pub fn natural_attack(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    attack_index: usize,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<AttackResult, RuleError> {
    Ok(natural_attack_inner(
        warrior,
        enemy,
        attack_index,
        random,
        default_attack_packet(),
        &[0; 20],
        difficulty,
    )?
    .0)
}

pub fn natural_attack_with_packet(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    attack_index: usize,
    random: &mut RandomState,
    attacks: &[u8],
    retained: &[u8],
    difficulty: i16,
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    natural_attack_with_buffs(
        warrior,
        enemy,
        attack_index,
        random,
        attacks,
        retained,
        &[0; 20],
        difficulty,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn natural_attack_with_buffs(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    attack_index: usize,
    random: &mut RandomState,
    attacks: &[u8],
    retained: &[u8],
    buffs: &[u8],
    difficulty: i16,
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    super::party_buffs::validate(buffs)?;
    if retained.len() != 58 {
        return Err(RuleError::Invalid("retained attack packet size".into()));
    }
    let base = attack_packet(attacks)?;
    let (mut next_warrior, mut next_enemy, mut next_random) =
        (warrior.clone(), enemy.clone(), random.clone());
    let result = natural_attack_inner(
        &mut next_warrior,
        &mut next_enemy,
        attack_index,
        &mut next_random,
        base,
        buffs,
        difficulty,
    )?;
    *warrior = next_warrior;
    *enemy = next_enemy;
    *random = next_random;
    Ok(result)
}

fn natural_attack_inner(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    attack_index: usize,
    random: &mut RandomState,
    mut packet: Vec<u8>,
    buffs: &[u8],
    difficulty: i16,
) -> Result<(AttackResult, Vec<u8>), RuleError> {
    let w = &mut warrior.bytes;
    let m = &mut enemy.bytes;
    if attack_index >= 6 {
        return Err(RuleError::Invalid("natural attack outside record".into()));
    }
    let attack_offset = if m[0x93 + attack_index * 4] == 0 {
        0x93
    } else {
        0x93 + attack_index * 4
    };
    let condition = m[0x96 + attack_index * 4];
    if !matches!(
        word(m, 0x4e)?,
        0 | 2 | 3 | 5 | 9 | 12 | 26 | 28 | 29 | 37 | 38 | 50 | 100 | 165 | 680 | 681 | 683
    ) || !matches!(word(m, 0x28)?, 0 | 9)
        || (word(m, 0x28)? == 9 && !matches!(word(m, 0x4e)?, 0 | 9))
        || !matches!(condition, 0 | 4 | 5 | 9 | 14)
        || word(w, 0x9c)? != 0
        || word(w, 0x60)? != 0
    {
        return Err(RuleError::Unsupported {
            action: "combat.attack",
            dependency: "enemy weapon, attack condition, reflection or unconscious target",
        });
    }
    put_word(m, 0x4e, 0);
    let mut hit = 33_i16
        .wrapping_add(word(m, 0)?.wrapping_mul(5))
        .wrapping_add(word(m, 0x40)?.wrapping_mul(5));
    for offset in [0xb4, 0xd6] {
        hit = hit.wrapping_sub(i16::from(m[offset] as i8).abs());
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
    if random.bound(100)
        > hit.clamp(
            15_i16.wrapping_sub(difficulty.wrapping_mul(5)),
            90_i16.wrapping_sub(difficulty.wrapping_mul(5)),
        )
    {
        increment_long(w, 0xa56, 1);
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
    // The rejected class-0 default still selects reloaded weapon-0 damage.
    let rejected_weapon = word(m, 0x28)? == 9;
    put_word(
        &mut packet,
        32,
        bonus.wrapping_add(if rejected_weapon {
            1
        } else {
            i16::from(m[attack_offset] as i8)
        }),
    );
    put_word(
        &mut packet,
        34,
        bonus.wrapping_add(if rejected_weapon {
            0
        } else {
            i16::from(m[attack_offset + 1] as i8)
        }),
    );
    let impact = i16::from(m[attack_offset + 2] as i8);
    put_word(
        &mut packet,
        38,
        if rejected_weapon {
            0
        } else if impact == 0 {
            100
        } else {
            impact
        },
    );
    let resolved = resolve_damage(
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
        random,
    )?;
    if resolved.total == 0 {
        increment_long(w, 0xa56, 1);
        return Ok((
            AttackResult {
                damage: 0,
                killed: false,
                sounds: vec![],
            },
            packet,
        ));
    }
    let mut damage = resolved.total;
    if condition != 0 {
        let strength = random.range(word(m, 0)? / 2, word(m, 0)?).max(1);
        if condition == 9
            && let Some(duration) =
                super::natural_conditions::disease_duration(w, strength, random)?
        {
            put_word(w, 0x96, duration);
        }
        if condition == 5
            && let Some(duration) = super::natural_conditions::poison_duration(w, strength, random)?
        {
            put_word(w, 0x64, duration);
        }
        if condition == 4 {
            let saved = random.bound(100) <= word(w, 0x122)?;
            let previous = word(w, 0x62)?;
            if !saved && previous >= 0 {
                let duration = previous.wrapping_add(strength.wrapping_abs());
                if duration < 30 {
                    put_word(w, 0x62, duration);
                }
                for offset in [0x3c, 0x40] {
                    let remaining = word(w, offset)? / 2;
                    put_word(w, offset, remaining);
                }
            }
        }
    }
    let mut sounds = vec![];
    if let Some(adjusted) = super::party_buffs::attack2_damage(buffs, 0, damage)? {
        damage = adjusted;
        sounds.push(-694);
    }
    if condition == 14 {
        let mut extra = random.bound(i16::from(m[0x94 + attack_index * 4] as i8));
        if word(w, 0x32)? == 3 {
            extra = extra.wrapping_mul(2);
        }
        if random.bound(100) <= word(w, 0x11a)? {
            extra /= 2;
        }
        if word(w, 0x76)? != 0 {
            extra /= 2;
        }
        damage = damage.wrapping_add(extra);
    }
    increment_long(w, 0xa42, i32::from(damage));
    increment_long(w, 0xa4e, 1);
    let health = word(w, 0x50)?.wrapping_sub(damage);
    put_word(w, 0x50, health);
    if health < 1 && health > -10 {
        w[0x9de] = 1;
    }
    put_word(w, 0x14, 1);
    Ok((
        AttackResult {
            damage,
            killed: health < 1,
            sounds,
        },
        packet,
    ))
}
