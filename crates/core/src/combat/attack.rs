use super::*;
use damage::{DamageFlags, DamagePacket, resolve_damage};

#[derive(Debug, PartialEq, Eq)]
pub struct AttackResult {
    pub feedback: Option<AttackFeedback>,
    pub damage: i16,
    pub killed: bool,
    pub sounds: Vec<i16>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AttackFeedback {
    pub code: i16,
    pub amount: i16,
    pub recipient: AttackRecipient,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AttackRecipient {
    Attacker,
    Defender,
}

impl AttackFeedback {
    pub(crate) fn defender(code: i16, amount: i16) -> Self {
        Self {
            code,
            amount,
            recipient: AttackRecipient::Defender,
        }
    }
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
        None,
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
        None,
    )?;
    *warrior = next_warrior;
    *enemy = next_enemy;
    *random = next_random;
    Ok((result.0, result.1))
}

pub(crate) fn warrior_attack_with_packet(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    random: &mut RandomState,
    attacks: &[u8],
    retained: &[u8],
    difficulty: i16,
    weapon: Option<&[u8]>,
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
    unarmed_attack_inner(
        warrior,
        enemy,
        random,
        attack_packet(attacks)?,
        retained,
        difficulty,
        weapon,
    )
}

fn unarmed_attack_inner(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    random: &mut RandomState,
    base: Vec<u8>,
    retained: &[u8],
    difficulty: i16,
    weapon: Option<&[u8]>,
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
    let mut packet = retained.to_vec();
    let w = &mut warrior.bytes;
    let m = &mut enemy.bytes;
    if word(w, 0xca)? != 0 && weapon.is_none() {
        return Err(RuleError::Unsupported {
            action: "combat.attack",
            dependency: "equipped weapon, damage reflection or weapon immunity",
        });
    }
    if word(w, 0xacc)? >= word(w, 0xaca)? {
        return Ok((
            AttackResult {
                feedback: None,
                damage: 0,
                killed: false,
                sounds: vec![45],
            },
            packet,
            false,
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
                feedback: None,
                damage: 0,
                killed: false,
                sounds: vec![],
            },
            packet,
            false,
        ));
    }
    let immunity = word(m, 0xc)?;
    let armed = weapon.is_some();
    let rejected = immunity == -1 && armed && word(weapon.unwrap(), 100)? != -1
        || immunity == -2 && armed && word(weapon.unwrap(), 100)? != -2
        || !matches!(immunity, -2..=0)
            && weapon.map_or(Ok(0), |w| word(w, 102))?.wrapping_sub(1024) != immunity;
    if rejected {
        return Ok((
            AttackResult {
                feedback: None,
                damage: 0,
                killed: false,
                sounds: vec![],
            },
            packet,
            false,
        ));
    }
    let reflected = m[0xd1] != 0 && random.bound(100) < 34;
    super::warrior_attack::prepare_packet(&mut packet, w, weapon, w)?;
    let resolved = resolve_damage(
        if reflected { &mut *w } else { &mut *m },
        !reflected,
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
    if weapon.is_some() && !reflected {
        for i in 0..8 {
            if m[0xa8 + i] != 0 {
                damage = damage.wrapping_add(word(w, 0xae + i * 2)? / 5);
            }
        }
    }
    if damage <= 0 {
        increment_long(w, 0xa52, 1);
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
    if random.bound(100) <= word(w, 0xfa)?.wrapping_add(word(w, 0xb68)?) {
        damage = damage.wrapping_mul(2);
    }
    if reflected {
        if word(w, 0x60)? != 0 {
            damage = word(w, 0x50)?;
        }
        let health = word(w, 0x50)?.wrapping_sub(damage);
        put_word(w, 0x50, health);
        put_word(w, 0x14, 1);
        increment_long(w, 0xa42, i32::from(damage));
        if (-9..=0).contains(&health) {
            w[0x9de] = 1;
        }
        let sound = match weapon {
            Some(weapon) => word(weapon, 14)?,
            None => random.bound(2).wrapping_add(401),
        };
        return Ok((
            AttackResult {
                feedback: None,
                damage,
                killed: health < 1,
                sounds: vec![sound],
            },
            packet,
            true,
        ));
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
    let sound = match weapon {
        Some(weapon) => word(weapon, 14)?,
        None => random.bound(2).wrapping_add(401),
    };
    Ok((
        AttackResult {
            feedback: None,
            damage,
            killed,
            sounds: vec![sound],
        },
        packet,
        false,
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
    let (result, packet, _) = natural_attack_with_target(
        warrior,
        enemy,
        attack_index,
        random,
        Some(attacks),
        retained,
        buffs,
        difficulty,
    )?;
    Ok((result, packet))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn natural_attack_with_target(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    attack_index: usize,
    random: &mut RandomState,
    attacks: Option<&[u8]>,
    retained: &[u8],
    buffs: &[u8],
    difficulty: i16,
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
    super::party_buffs::validate(buffs)?;
    if retained.len() != 58 {
        return Err(RuleError::Invalid("retained attack packet size".into()));
    }
    let base = attacks
        .map(attack_packet)
        .transpose()?
        .unwrap_or_else(default_attack_packet);
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

pub(crate) fn resolve_reflected_attack(
    target: &mut [u8],
    packet: &[u8],
    index: usize,
    natural: bool,
    random: &mut RandomState,
) -> Result<AttackResult, RuleError> {
    let source = target.to_vec();
    target[0x105] = 1;
    let resolved = resolve_damage(
        target,
        true,
        &DamagePacket {
            bytes: packet.to_vec(),
            attack_count: 0,
        },
        DamageFlags {
            armor: true,
            saves: true,
            ..Default::default()
        },
        random,
    )?;
    let condition = source[0x96 + index * 4];
    let mut damage = resolved.total;
    let mut feedback = None;
    if damage != 0 && natural {
        if let Some(effect) =
            super::natural_conditions::monster_attack_effect(target, &source, index, random)?
        {
            feedback = effect.feedback;
            damage = damage.wrapping_add(effect.extra_damage);
            if condition == 10 && effect.applied {
                put_word(target, 0x10c, -1);
            }
        } else {
            damage = 0;
        }
    }
    let health = word(target, 0x110)?.wrapping_sub(damage);
    put_word(target, 0x110, health);
    Ok(AttackResult {
        feedback,
        damage,
        killed: health < 1,
        sounds: vec![],
    })
}

fn natural_attack_inner(
    warrior: &mut Warrior,
    enemy: &mut MonsterRecord,
    attack_index: usize,
    random: &mut RandomState,
    mut packet: Vec<u8>,
    buffs: &[u8],
    difficulty: i16,
) -> Result<(AttackResult, Vec<u8>, bool), RuleError> {
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
    // attack2102038 clears any previously loaded ranged ID before natural damage.
    if !matches!(word(m, 0x28)?, 0 | 9)
        || (word(m, 0x28)? == 9 && !matches!(word(m, 0x4e)?, 0 | 9))
        || !matches!(condition, 0 | 1 | 2 | 4 | 5 | 6 | 9 | 10 | 11 | 14 | 17)
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
    let hit = random.bound(100)
        <= hit.clamp(
            15_i16.wrapping_sub(difficulty.wrapping_mul(5)),
            90_i16.wrapping_sub(difficulty.wrapping_mul(5)),
        );
    let reflected = word(w, 0x9c)? != 0 && random.bound(100) < 34 && word(m, 0xc)? == 0;
    if !hit {
        if !reflected {
            increment_long(w, 0xa56, 1);
        }
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
    if reflected {
        let result = resolve_reflected_attack(m, &packet, attack_index, true, random)?;
        return Ok((result, packet, true));
    }
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
                feedback: None,
                damage: 0,
                killed: false,
                sounds: vec![],
            },
            packet,
            reflected,
        ));
    }
    let natural = super::natural_conditions::warrior_attack_effect(
        w,
        m,
        attack_index,
        word(buffs, 14)? != 0,
        random,
    )?;
    let mut damage = resolved.total;
    let mut sounds = vec![];
    if let Some(adjusted) = super::party_buffs::attack2_damage(buffs, 0, damage)? {
        damage = adjusted;
        sounds.push(-694);
    }
    damage = damage.wrapping_add(natural.extra_damage);
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
            feedback: natural.feedback,
            damage,
            killed: health < 1,
            sounds,
        },
        packet,
        false,
    ))
}
