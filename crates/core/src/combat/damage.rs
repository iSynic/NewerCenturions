use super::*;

#[derive(Clone, Debug)]
pub struct DamagePacket {
    pub bytes: Vec<u8>,
    pub attack_count: i16,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DamageFlags {
    pub shield: bool,
    pub armor: bool,
    pub saves: bool,
    pub armor_filter: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DamageResult {
    pub total: i16,
    pub components: [i16; 9],
    pub sound: i16,
    pub shield_sound: bool,
}

pub fn resolve_damage(
    target: &mut [u8],
    monster: bool,
    packet: &DamagePacket,
    flags: DamageFlags,
    random: &mut RandomState,
) -> Result<DamageResult, RuleError> {
    let p = &packet.bytes;
    if p.len() != 58 || target.len() != if monster { 288 } else { 3138 } {
        return Err(RuleError::Invalid("damage record size".into()));
    }
    if flags.saves
        && monster
        && target[0x8c..0x93]
            .iter()
            .zip(&p[23..30])
            .any(|(v, damage)| (*v as i8) < 0 && *damage != 0)
    {
        return Err(RuleError::Unsupported {
            action: "combat.damage",
            dependency: "monster percentage resistance",
        });
    }
    let mut components = [0_i16; 9];
    let maximum = word(p, 34)?;
    if maximum != 0 {
        components[0] = random.range(word(p, 32)?, maximum);
    }
    for _ in 0..packet.attack_count {
        if p[12] != 0 {
            components[1] = components[1]
                .wrapping_add(random.range(i16::from(p[11] as i8), i16::from(p[12] as i8)));
        }
    }
    for (i, maximum) in p[23..30].iter().enumerate() {
        if *maximum != 0 {
            components[i + 2] = random.bound(i16::from(*maximum));
        }
    }
    if monster && p[42] != 0 {
        let multiplier = word(target, 0x10)?.wrapping_abs();
        for damage in &mut components {
            *damage = damage.wrapping_mul(multiplier);
        }
    }
    let armor = word(target, if monster { 8 } else { 0x2c })?.min(95);
    if flags.armor_filter {
        for damage in &mut components {
            let mut kept = 0_i16;
            for _ in 0..*damage {
                if random.bound(100) > armor {
                    kept = kept.wrapping_add(1);
                }
            }
            *damage = kept;
        }
    }
    let sum = |damage: &[i16; 9]| damage.iter().fold(0_i16, |a, b| a.wrapping_add(*b));
    if word(p, 40)? == 57 {
        return Ok(DamageResult {
            total: sum(&components).wrapping_neg(),
            components,
            sound: 0,
            shield_sound: false,
        });
    }
    let mut shield_sound = false;
    if flags.shield && (!monster || target[0xb0] != 0xff) {
        let chance = word(target, if monster { 0x56 } else { 0x24 })?.min(95);
        let offset = if monster { 0x58 } else { 0x26 };
        let mut charge = word(target, offset)?;
        let mut cost = if monster {
            i16::from(target[0x10a] as i8)
        } else {
            word(target, 0x2a)?
        };
        if monster && chance != 0 && charge != 0 && cost == 0 {
            cost = 1;
        }
        if monster || (charge != 0 && charge >= cost) {
            'shield: for damage in &mut components {
                let initial = *damage;
                for _ in 0..initial {
                    if random.bound(100) < chance && charge >= cost && *damage > 1 {
                        *damage = damage.wrapping_sub(1);
                        charge = charge.wrapping_sub(cost);
                        shield_sound = true;
                    } else if !monster && charge < cost {
                        break 'shield;
                    }
                }
            }
            put_word(target, offset, charge);
            if !monster {
                put_word(target, 0x160, charge);
            }
        }
    }
    if flags.armor {
        for (component, damage) in components.iter_mut().enumerate() {
            let initial = if component == 6 { 0 } else { *damage };
            for _ in 0..initial {
                if random.bound(100) < armor && *damage > 1 {
                    *damage = damage.wrapping_sub(1);
                }
            }
        }
    }
    if flags.saves {
        if (p[9] as i8) < 2 {
            let bonus = i16::from(p[8] as i8)
                .wrapping_mul(packet.attack_count)
                .wrapping_add(i16::from(p[6] as i8));
            for (component, damage) in components.iter_mut().enumerate().skip(2) {
                if *damage != 0 {
                    let save = if monster {
                        i16::from(target[0x8a + component] as i8)
                    } else {
                        word(target, 0x112 + (component - 1) * 2)?
                    };
                    if random.bound(100) <= save.wrapping_add(bonus) || monster && save > 99 {
                        *damage /= 2;
                    }
                }
            }
        }
        for (i, damage) in components.iter_mut().enumerate().skip(2) {
            let protected = if monster {
                target[0xb9 + i] != 0
            } else {
                word(target, 0x70 + (i - 2) * 2)? != 0
            };
            if protected {
                *damage /= 2;
            }
        }
        if monster {
            for (i, damage) in components.iter_mut().enumerate().skip(2) {
                if (target[0x8a + i] as i8) > 100 {
                    *damage = 0;
                }
            }
            if target[0xab] != 0 {
                components[8] = 0;
            }
        } else if word(target, 0x32)? == 3 {
            components[8] = 0;
            components[5] = components[5].wrapping_mul(2);
        }
    }
    let first = components.iter().position(|v| *v != 0);
    let sound = first.map_or(0, |i| [157, 157, 159, 160, 161, 162, 163, 164, 159][i]);
    if first.is_some() {
        if monster {
            target[0x105] = 1;
        } else {
            put_word(target, 0x14, 1);
        }
    }
    Ok(DamageResult {
        total: sum(&components),
        components,
        sound,
        shield_sound,
    })
}
