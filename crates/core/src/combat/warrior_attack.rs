use crate::combat::attack;
use crate::combat::{damage, put_word};
use crate::{RandomState, RuleError};
use attack::AttackResult;
use centurions_content::{Warrior, word};
use damage::{DamageFlags, DamagePacket, resolve_damage};

/// Warrior-to-warrior branch of attack 0x100af0, including the unconditional reflection roll.
pub(crate) fn attack(
    warriors: &mut [Warrior],
    actor: usize,
    target: usize,
    attacks: &[u8],
    items: &[Vec<u8>],
    difficulty: i16,
    random: &mut RandomState,
) -> Result<(AttackResult, Vec<u8>, usize), RuleError> {
    let attacker = warriors
        .get(actor)
        .ok_or("melee actor outside party")?
        .bytes
        .clone();
    warriors.get(target).ok_or("melee target outside party")?;
    let weapon = weapon(&attacker, items)?;
    let mut packet = attack::attack_packet(attacks)?;
    if word(&attacker, 0xacc)? >= word(&attacker, 0xaca)? {
        return Ok((
            AttackResult {
                feedback: None,
                damage: 0,
                killed: false,
                sounds: vec![45],
            },
            packet,
            target,
        ));
    }
    put_word(
        &mut warriors[actor].bytes,
        0xacc,
        word(&attacker, 0xacc)?.wrapping_add(1),
    );
    let mut hit = 33_i16
        .wrapping_add(word(&attacker, 6)?)
        .wrapping_add(word(&attacker, 0xb5a)?);
    for offset in [0x62, 0xa6] {
        hit = hit.wrapping_sub(word(&attacker, offset)?.wrapping_abs());
    }
    for (offset, penalty) in [(0x6a, 15), (0x94, 15), (0x98, 10)] {
        if word(&attacker, offset)? != 0 {
            hit = hit.wrapping_sub(penalty);
        }
    }
    let defender = &warriors[target].bytes;
    hit = hit.wrapping_sub(word(defender, 0xa)?.wrapping_add(word(defender, 0xb5e)?));
    hit = hit.wrapping_sub(word(defender, 0x6c)?.wrapping_abs().wrapping_mul(2));
    for (offset, sign) in [(0x62, 1_i16), (0xa8, 1), (0xaa, -1)] {
        hit = hit.wrapping_add(word(defender, offset)?.wrapping_abs().wrapping_mul(sign));
    }
    for (offset, bonus) in [(0x6a, 15), (0x94, 15), (0x98, 10)] {
        if word(defender, offset)? != 0 {
            hit = hit.wrapping_add(bonus);
        }
    }
    let missed = random.bound(100)
        > crate::rules::pin(hit, 15 - difficulty * 5, 90 - difficulty * 5)
        && word(defender, 0x60)? == 0;
    if missed {
        increment(&mut warriors[actor].bytes, 0xa52, 1);
        return Ok((
            AttackResult {
                feedback: None,
                damage: 0,
                killed: false,
                sounds: vec![],
            },
            packet,
            target,
        ));
    }
    let actual = if random.bound(100) < 34 && word(defender, 0x9c)? != 0 {
        actor
    } else {
        target
    };
    prepare_packet(&mut packet, &attacker, weapon, &attacker)?;
    let w = &mut warriors[actual].bytes;
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
        random,
    )?;
    let mut damage = result.total;
    if damage <= 0 {
        increment(&mut warriors[actor].bytes, 0xa52, 1);
        return Ok((
            AttackResult {
                feedback: None,
                damage: 0,
                killed: false,
                sounds: vec![],
            },
            packet,
            actual,
        ));
    }
    if random.bound(100) <= word(&attacker, 0xfa)?.wrapping_add(word(&attacker, 0xb68)?) {
        damage = damage.wrapping_mul(2);
    }
    if word(w, 0x60)? != 0 {
        damage = word(w, 0x50)?;
    }
    put_word(w, 0x14, 1);
    let health = word(w, 0x50)?.wrapping_sub(damage);
    put_word(w, 0x50, health);
    increment(w, 0xa42, i32::from(damage));
    if health < 1 && health > -10 {
        w[0x9de] = 1;
    }
    Ok((
        AttackResult {
            feedback: None,
            damage,
            killed: health < 1,
            sounds: vec![match weapon {
                Some(weapon) => word(weapon, 14)?,
                None => random.bound(2).wrapping_add(401),
            }],
        },
        packet,
        actual,
    ))
}

pub(crate) fn weapon<'a>(
    warrior: &[u8],
    items: &'a [Vec<u8>],
) -> Result<Option<&'a [u8]>, RuleError> {
    let id = word(warrior, 0xca)?;
    if id == 0 {
        return Ok(None);
    }
    let definition = items
        .get(id.unsigned_abs() as usize)
        .ok_or("melee weapon definition missing")?;
    if word(definition, 26)?.wrapping_abs() != 2
        || word(definition, 42)? == -10
        || word(definition, 44)? > 1100
    {
        return Err(RuleError::Unsupported {
            action: "combat.attack",
            dependency: "spell-triggering melee weapon",
        });
    }
    Ok(Some(definition))
}

pub(crate) fn prepare_packet(
    packet: &mut [u8],
    warrior: &[u8],
    weapon: Option<&[u8]>,
    active: &[u8],
) -> Result<(), RuleError> {
    let bonus = word(warrior, 0x9e)?;
    put_word(packet, 32, bonus.wrapping_add(1));
    let maximum = bonus.wrapping_add(word(warrior, 0xc)?);
    put_word(packet, 34, maximum);
    if let Some(weapon) = weapon {
        // attack stores this percentage as Single before the widened multiply and truncation.
        let factor = f64::from((100. + f32::from(active[0x15a] as i8)) / 100.);
        put_word(
            packet,
            34,
            maximum.wrapping_add((f64::from(word(weapon, 40)?) * factor).trunc() as i32 as i16),
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
            if word(weapon, source)? != 0 {
                packet[destination] =
                    (f64::from(word(weapon, source)?) * factor).trunc() as i32 as u8;
            }
        }
    }
    Ok(())
}

fn increment(bytes: &mut [u8], offset: usize, value: i32) {
    let previous = i32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap());
    bytes[offset..offset + 4].copy_from_slice(&previous.wrapping_add(value).to_be_bytes());
}
