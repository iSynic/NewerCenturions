use crate::{RandomState, RuleError};
use centurions_content::{span, word};

pub(crate) struct NaturalEffect {
    pub extra_damage: i16,
    pub applied: bool,
    pub feedback: Option<super::attack::AttackFeedback>,
}

fn result_code(condition: u8) -> Option<i16> {
    Some(match condition {
        1 => -33,
        2 => -32,
        3 => -30,
        4 => -29,
        5 => -28,
        6 => -27,
        9 => -34,
        10 => -26,
        11 => -22,
        12 => -19,
        13 => -20,
        14 => -21,
        15 => -23,
        16 => -15,
        17 => -16,
        _ => return None,
    })
}

/// attack2 0x1028a4 shares natural effects between armed fallthroughs and unarmed NPCs.
/// None is the native magic-immunity branch, which discards physical damage too.
pub(crate) fn monster_attack_effect(
    target: &mut [u8],
    source: &[u8],
    index: usize,
    random: &mut RandomState,
) -> Result<Option<NaturalEffect>, RuleError> {
    span(target, 0, 288)?;
    span(source, 0, 288)?;
    if index >= 6 {
        return Err(RuleError::Invalid("natural attack index".into()));
    }
    let condition = source[0x96 + index * 4];
    let mut result = NaturalEffect {
        extra_damage: 0,
        applied: false,
        feedback: None,
    };
    if condition == 0 {
        return Ok(Some(result));
    }
    let strength = random.range(word(source, 0)? / 2, word(source, 0)?).max(1);
    if word(target, 0xa)? > 100 {
        return Ok(None);
    }
    if matches!(condition, 1 | 2 | 4 | 5 | 6 | 9) {
        result.applied = attack_duration(target, true, condition, strength, false, random)?;
        if result.applied && condition == 4 {
            target[0x109] = ((target[0x109] as i8) / 2) as u8;
            target[0x106] = ((target[0x106] as i8) / 2) as u8;
        }
    } else if condition == 10 {
        let threshold = i16::from(target[0x8b] as i8);
        if random.bound(100) > threshold && threshold <= 99 {
            target[0xe..0x10].copy_from_slice(&source[0xe..0x10]);
            result.applied = true;
        }
    } else if (11..=17).contains(&condition) {
        result.extra_damage = attack_extra_damage(
            target,
            true,
            condition,
            i16::from(source[0x94 + index * 4] as i8),
            random,
        )?;
        result.applied = result.extra_damage != 0;
    } else {
        return Err(RuleError::Unsupported {
            action: "combat.attack.condition",
            dependency: "natural condition outside supplied actor families",
        });
    }
    if result.applied {
        result.feedback = result_code(condition)
            .map(|code| super::attack::AttackFeedback::defender(code, result.extra_damage));
    }
    Ok(Some(result))
}

/// attack2 applies these effects only when no physical weapon was retained on entry.
pub(crate) fn warrior_attack_effect(
    target: &mut [u8],
    source: &[u8],
    index: usize,
    mind_shield: bool,
    random: &mut RandomState,
) -> Result<NaturalEffect, RuleError> {
    span(target, 0, 3138)?;
    span(source, 0, 288)?;
    if index >= 6 {
        return Err(RuleError::Invalid("natural attack index".into()));
    }
    let condition = source[0x96 + index * 4];
    let prior_faction = word(target, 0x10)?;
    let mut result = NaturalEffect {
        extra_damage: 0,
        applied: false,
        feedback: None,
    };
    if condition == 0 {
        return Ok(result);
    }
    let strength = random.range(word(source, 0)? / 2, word(source, 0)?).max(1);
    if matches!(condition, 1 | 2 | 4 | 5 | 6 | 9) {
        result.applied = attack_duration(target, false, condition, strength, mind_shield, random)?;
        if result.applied && condition == 4 {
            for offset in [0x3c, 0x40] {
                let remaining = word(target, offset)? / 2;
                target[offset..offset + 2].copy_from_slice(&remaining.to_be_bytes());
            }
        }
    } else if condition == 10 {
        let threshold = word(target, 0x112)?.wrapping_add(if mind_shield { 50 } else { 0 });
        let saved = random.bound(100) <= threshold;
        if !saved && word(target, 0x32)? != 3 {
            target[0x10..0x12].copy_from_slice(&source[0xe..0x10]);
            result.applied = true;
        }
    } else if matches!(condition, 11 | 14 | 17) {
        result.extra_damage = attack_extra_damage(
            target,
            false,
            condition,
            i16::from(source[0x94 + index * 4] as i8),
            random,
        )?;
        result.applied = result.extra_damage != 0;
    } else {
        return Err(RuleError::Unsupported {
            action: "combat.attack.condition",
            dependency: "natural condition outside supplied actor families",
        });
    }
    if result.applied && (condition != 10 || prior_faction == 0) {
        result.feedback = result_code(condition)
            .map(|code| super::attack::AttackFeedback::defender(code, result.extra_damage));
    }
    Ok(result)
}

/// attack2 natural duration branches use a save before their permanent-condition gate.
pub(crate) fn attack_duration(
    target: &mut [u8],
    monster: bool,
    condition: u8,
    strength: i16,
    mind_shield: bool,
    random: &mut RandomState,
) -> Result<bool, RuleError> {
    span(target, 0, if monster { 288 } else { 3138 })?;
    let (save, party_offset, monster_offset, cyborg_immune) = match condition {
        1 => (0, 0x5e, 0xb2, true),
        2 => (7, 0x60, 0xb3, true),
        3 => (5, 0x68, 0xb7, false),
        4 => (8, 0x62, 0xb4, false),
        5 => (1, 0x64, 0xb5, true),
        6 => (0, 0x98, 0xcf, true),
        9 => (7, 0x96, 0xce, true),
        _ => return Err(RuleError::Invalid("natural duration condition".into())),
    };
    let threshold = if monster {
        i16::from(target[0x8b + save] as i8)
    } else {
        word(target, 0x112 + save * 2)?.wrapping_add(if save == 0 && mind_shield { 50 } else { 0 })
    };
    let saved = random.bound(100) <= threshold || (monster && threshold > 99);
    let previous = if monster {
        i16::from(target[monster_offset] as i8)
    } else {
        word(target, party_offset)?
    };
    if saved || previous < 0 {
        return Ok(false);
    }
    let magnitude = if !monster && cyborg_immune && word(target, 0x32)? == 3 {
        0
    } else {
        strength.wrapping_abs()
    };
    let duration = previous.wrapping_add(magnitude);
    if monster {
        target[monster_offset] = duration as u8;
    } else if duration < 30 {
        target[party_offset..party_offset + 2].copy_from_slice(&duration.to_be_bytes());
    }
    if condition == 2 {
        if monster {
            target[0x109] = 0;
            target[0x106] = 0;
        } else {
            target[0x3c..0x3e].fill(0);
            target[0x40..0x42].fill(0);
        }
    }
    Ok(true)
}

/// attack2 cases11..17 retain the extra damage draw before save and resistance halves.
pub(crate) fn attack_extra_damage(
    target: &[u8],
    monster: bool,
    condition: u8,
    bound: i16,
    random: &mut RandomState,
) -> Result<i16, RuleError> {
    if !(11..=17).contains(&condition) {
        return Err(RuleError::Invalid("natural elemental condition".into()));
    }
    span(target, 0, if monster { 288 } else { 3138 })?;
    let save = usize::from(condition - 10);
    let mut amount = random.bound(bound);
    if !monster && word(target, 0x32)? == 3 {
        if condition == 14 {
            amount = amount.wrapping_mul(2);
        } else if condition == 17 {
            amount = 0;
        }
    }
    let threshold = if monster {
        i16::from(target[0x8b + save] as i8)
    } else {
        word(target, 0x112 + save * 2)?
    };
    if random.bound(100) <= threshold || (monster && threshold > 99) {
        amount /= 2;
    }
    let protected = if monster {
        target[0xbb + usize::from(condition - 11)] != 0
    } else {
        word(target, 0x70 + usize::from(condition - 11) * 2)? != 0
    };
    if protected {
        amount /= 2;
    }
    Ok(amount)
}

/// attack2 case 9 consumes the disease save before testing an existing permanent condition.
pub fn disease_duration(
    warrior: &[u8],
    strength: i16,
    random: &mut RandomState,
) -> Result<Option<i16>, RuleError> {
    let warrior = span(warrior, 0, 3138)?;
    let saved = random.bound(100) <= word(warrior, 0x120)?;
    let previous = word(warrior, 0x96)?;
    if saved || previous < 0 {
        return Ok(None);
    }
    let strength = if word(warrior, 0x32)? == 3 {
        0
    } else {
        strength.wrapping_abs()
    };
    let duration = previous.wrapping_add(strength);
    Ok((duration < 30).then_some(duration))
}

/// attack2 case 5 tests save 1 before permanent poison immunity.
pub fn poison_duration(
    warrior: &[u8],
    strength: i16,
    random: &mut RandomState,
) -> Result<Option<i16>, RuleError> {
    let warrior = span(warrior, 0, 3138)?;
    let saved = random.bound(100) <= word(warrior, 0x114)?;
    let previous = word(warrior, 0x64)?;
    if saved || previous < 0 {
        return Ok(None);
    }
    let strength = if word(warrior, 0x32)? == 3 {
        0
    } else {
        strength.wrapping_abs()
    };
    let duration = previous.wrapping_add(strength);
    Ok((duration < 30).then_some(duration))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiseaseTick {
    pub health: i16,
    pub killed: bool,
}

/// reduce applies disease damage before reducing positive duration counters.
pub fn disease_tick(warrior: &[u8]) -> Result<Option<DiseaseTick>, RuleError> {
    condition_tick(warrior, 0x96)
}

pub fn poison_tick(warrior: &[u8]) -> Result<Option<DiseaseTick>, RuleError> {
    condition_tick(warrior, 0x64)
}

/// getup tests CE and CB after decrementing positive condition bytes.
pub fn enemy_drain_tick(monster: &[u8]) -> Result<Option<DiseaseTick>, RuleError> {
    let monster = span(monster, 0, 288)?;
    let duration = i16::from(monster[0xce] as i8);
    let health = word(monster, 0x110)?;
    if duration == 0 || monster[0xcb] != 0 || health <= 0 {
        return Ok(None);
    }
    let health = health.wrapping_sub(duration.abs());
    Ok(Some(DiseaseTick {
        health,
        killed: health < 1,
    }))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnemyHeal {
    pub health: i16,
    pub amount: i16,
}

pub fn enemy_poison_heal(
    monster: &centurions_content::combat::MonsterRecord,
) -> Result<Option<EnemyHeal>, RuleError> {
    span(&monster.bytes, 0, 288)?;
    let duration = i16::from(monster.poison_timer()?);
    let health = monster.stamina()?;
    let maximum = monster.maximum_stamina()?;
    if duration == 0 || health <= 0 || health >= maximum {
        return Ok(None);
    }
    let amount = duration.abs();
    // The original cap adjustment subtracts HP from itself, retaining the amount.
    let health = health.wrapping_add(amount).min(maximum);
    Ok(Some(EnemyHeal { health, amount }))
}

pub fn enemy_poison_tick(
    monster: &centurions_content::combat::MonsterRecord,
) -> Result<Option<DiseaseTick>, RuleError> {
    span(&monster.bytes, 0, 288)?;
    let duration = i16::from(monster.poison_timer()?);
    let health = monster.stamina()?;
    if duration == 0 || monster.bytes[0xcb] != 0 || health <= 0 {
        return Ok(None);
    }
    let health = health.wrapping_sub(duration.abs());
    Ok(Some(DiseaseTick {
        health,
        killed: health < 1,
    }))
}

/// getup 0x126244 preserves its signed drain and the original regeneration alias.
pub fn reduce_enemy_psi(monster: &mut [u8]) -> Result<(), RuleError> {
    span(monster, 0, 288)?;
    let amount = i16::from(monster[0xd4] as i8);
    let mut psi = word(monster, 0x10e)?;
    if amount != 0 {
        psi = psi.wrapping_sub(amount).max(0);
    }
    if monster[0xd3] != 0 && monster[0xa8] != 0 {
        psi = psi.wrapping_add(amount);
    }
    monster[0x10e..0x110].copy_from_slice(&psi.to_be_bytes());
    Ok(())
}

fn condition_tick(warrior: &[u8], offset: usize) -> Result<Option<DiseaseTick>, RuleError> {
    let warrior = span(warrior, 0, 3138)?;
    let duration = word(warrior, offset)?;
    let health = word(warrior, 0x50)?;
    if duration == 0 || health <= 0 || word(warrior, 0x90)? < 0 {
        return Ok(None);
    }
    let health = health.wrapping_sub(duration.wrapping_abs());
    Ok(Some(DiseaseTick {
        health,
        killed: health < 1,
    }))
}

/// Outdoor killbody updates availability and sound without combat cleanup.
pub fn disease_outdoors(
    warriors: &[centurions_content::Warrior],
    member: usize,
    context: &crate::story_health::HealthContext,
    random: &mut RandomState,
) -> Result<crate::story_health::HealthOutcome, RuleError> {
    condition_outdoors(warriors, member, context, 0x96, random)
}

pub fn poison_outdoors(
    warriors: &[centurions_content::Warrior],
    member: usize,
    context: &crate::story_health::HealthContext,
    random: &mut RandomState,
) -> Result<crate::story_health::HealthOutcome, RuleError> {
    condition_outdoors(warriors, member, context, 0x64, random)
}

fn condition_outdoors(
    warriors: &[centurions_content::Warrior],
    member: usize,
    context: &crate::story_health::HealthContext,
    offset: usize,
    random: &mut RandomState,
) -> Result<crate::story_health::HealthOutcome, RuleError> {
    if warriors.is_empty() || warriors.len() > 6 || member >= warriors.len() {
        return Err(RuleError::Invalid("condition target outside party".into()));
    }
    for warrior in warriors {
        span(&warrior.bytes, 0, 3138)?;
    }
    let mut outcome = crate::story_health::HealthOutcome {
        warriors: warriors.to_vec(),
        context: context.clone(),
        events: Vec::new(),
        amounts: Vec::new(),
    };
    if let Some(tick) = condition_tick(&warriors[member].bytes, offset)? {
        outcome.warriors[member].bytes[0x50..0x52].copy_from_slice(&tick.health.to_be_bytes());
        outcome.context.effect = 7;
        if tick.killed {
            use crate::story_health::HealthEvent;
            outcome.context.last_body = member as i16;
            outcome.context.unavailable = outcome.context.unavailable.wrapping_add(1);
            outcome.events.push(HealthEvent::Defeated {
                warrior: member,
                permanent: tick.health < -9,
            });
            outcome
                .events
                .push(HealthEvent::Sound(random.bound(2).wrapping_add(506)));
            if outcome.context.unavailable > warriors.len() as i8 - 1 {
                outcome.events.push(HealthEvent::PartyLost);
            }
        }
    }
    Ok(outcome)
}
