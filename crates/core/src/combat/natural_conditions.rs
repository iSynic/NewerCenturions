use crate::{RandomState, RuleError};
use centurions_content::{span, word};

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
