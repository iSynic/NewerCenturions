//! Support rules for combat/spells.
use super::{
    AttackResult, BodySpellResult, DamageFlags, DamagePacket, DirectSpell, RandomState,
    RestrictionMemoryView, RuleError, monster_race_is_excluded, put_word, race_is_excluded,
    resolve_damage_with_amount,
};
use centurions_content::combat::MonsterRecord;
use centurions_content::{Warrior, word};

/// resist150588..150742 for nonphysical status categories7,8,10.
pub(super) fn status_resisted(
    target: &[u8],
    monster: bool,
    context: &DirectSpell<'_>,
    random: &mut RandomState,
) -> Result<bool, RuleError> {
    if context.armor_filter || !matches!(context.packet[9], 0 | 2) {
        return Ok(false);
    }
    if context.packet[19] <= 6
        && monster
        && (target[0x8b + usize::from(context.packet[19])] as i8 > 100 || word(target, 0xa)? > 100)
    {
        return Ok(true);
    }
    if context.packet[19] == 5
        && if monster {
            target[0xcb] != 0
        } else {
            word(target, 0x90)? != 0
        }
    {
        return Ok(true);
    }
    let immune = (context.level..7).any(|level| {
        if monster {
            target[0xc2 + level as usize] != 0
        } else {
            word(target, 0x7e + level as usize * 2).unwrap_or(1) != 0
        }
    });
    let threshold = word(target, if monster { 0xa } else { 0x22 })?
        .wrapping_add(i16::from(context.packet[10] as i8).wrapping_mul(context.count));
    let roll = random.bound(100);
    Ok(if monster {
        roll <= threshold || immune
    } else {
        roll > threshold && immune
    })
}

/// resolvespell151022 and spelllist15bd50 retain the original Psi-drain ordering.
/// Party targets clear the amount before Siphon's transfer; NPC targets transfer
/// the negative half before clearing it. Neither branch applies stamina damage.
#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_psi_drain(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    caster: i16,
    memory: RestrictionMemoryView<'_>,
    random: &mut RandomState,
) -> Result<BodySpellResult, RuleError> {
    let packet = context.packet;
    if packet.len() != 58
        || warriors.len() > 6
        || enemies.len() > 100
        || mask.len() < 10 + enemies.len()
        || !(1..=7).contains(&context.count)
        || !(0..7).contains(&context.level)
    {
        return Err(RuleError::Invalid("Psi drain inputs".into()));
    }
    let effect = word(packet, 40)?;
    if !matches!(effect, 60 | 61) || packet[19] != 5 {
        return Err(RuleError::Unsupported {
            action: "combat.spell.drain",
            dependency: "other Psi-drain packet category",
        });
    }
    let caster =
        usize::try_from(caster).map_err(|_| RuleError::Invalid("negative Psi caster".into()))?;
    if caster >= warriors.len() && caster < 10
        || caster >= 10 + enemies.len()
        || warriors.iter().any(|record| record.bytes.len() != 3138)
        || enemies.iter().any(|record| record.bytes.len() != 288)
        || mask[warriors.len()..10].iter().any(|flag| *flag > 0)
    {
        return Err(RuleError::Invalid("Psi drain body records".into()));
    }
    let mut next_random = random.clone();
    let mut result = BodySpellResult {
        warriors: warriors.to_vec(),
        enemies: enemies.to_vec(),
        sounds: vec![],
        targets: vec![],
        race_excluded: vec![],
    };
    next_random.range(i16::from(packet[13] as i8), i16::from(packet[14] as i8));
    if packet[15] != 0 {
        for _ in 0..context.count {
            next_random.range(i16::from(packet[15] as i8), i16::from(packet[16] as i8));
        }
    }
    for body in (0..warriors.len()).chain(10..10 + enemies.len()) {
        if mask[body] <= 0 {
            continue;
        }
        let monster = body >= 10;
        let excluded = if monster {
            word(packet, 50)? != 0 && monster_race_is_excluded(packet, body, Some(memory))?
        } else {
            race_is_excluded(packet, &result.warriors[body].bytes, "combat.spell.drain")?
        };
        if excluded {
            result.race_excluded.push(body);
            continue;
        }
        let target = spell_body(&mut result.warriors, &mut result.enemies, body)?;
        if status_resisted(target, monster, context, &mut next_random)? {
            continue;
        }
        let damage = resolve_damage_with_amount(
            target,
            monster,
            &DamagePacket {
                bytes: packet.to_vec(),
                attack_count: context.count,
            },
            DamageFlags {
                shield: true,
                armor: true,
                saves: true,
                armor_filter: context.armor_filter,
            },
            0,
            &mut next_random,
        )?;
        let reflected = !monster && word(target, 0x9a)? != 0 && next_random.bound(100) < 34;
        let actual = if reflected { caster } else { body };
        let target = spell_body(&mut result.warriors, &mut result.enemies, actual)?;
        let offset = if actual < 10 { 0x58 } else { 0x10e };
        let amount = damage.total.wrapping_neg();
        let mut points = word(target, offset)?.wrapping_add(amount).max(0);
        if actual < 10 {
            points = points.min(word(target, 0x5a)?);
        }
        put_word(target, offset, points);
        if effect == 61 && actual >= 10 {
            let target = spell_body(&mut result.warriors, &mut result.enemies, caster)?;
            let offset = if caster < 10 { 0x58 } else { 0x10e };
            let points = word(target, offset)?.wrapping_add(amount / 2);
            put_word(target, offset, points);
        }
        result.targets.push((
            body,
            AttackResult {
                feedback: None,
                damage: 0,
                killed: false,
                sounds: if damage.shield_sound {
                    vec![206]
                } else {
                    vec![]
                },
            },
        ));
    }
    *random = next_random;
    Ok(result)
}

pub(super) fn spell_body<'a>(
    warriors: &'a mut [Warrior],
    enemies: &'a mut [MonsterRecord],
    body: usize,
) -> Result<&'a mut [u8], RuleError> {
    if body < 10 {
        Ok(&mut warriors
            .get_mut(body)
            .ok_or("Psi party body missing")?
            .bytes)
    } else {
        Ok(&mut enemies
            .get_mut(body - 10)
            .ok_or("Psi enemy body missing")?
            .bytes)
    }
}
