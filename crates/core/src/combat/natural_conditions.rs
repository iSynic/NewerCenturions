use crate::{RandomState, RuleError};
use centurions_content::{
    records::{Condition, EnemyRecord, EnemyRecordMut, SavingThrow, WarriorRecord},
    span,
};

mod attack_effects;
pub(crate) use attack_effects::{
    admit_weapon_condition, apply_weapon_condition, monster_attack_effect, warrior_attack_effect,
};

/// attack2 case 9 consumes the disease save before testing an existing permanent condition.
pub fn disease_duration(
    warrior: &[u8],
    strength: i16,
    random: &mut RandomState,
) -> Result<Option<i16>, RuleError> {
    condition_duration(
        warrior,
        strength,
        Condition::Diseased,
        SavingThrow::Biological,
        random,
    )
}

/// attack2 case 5 tests the chemical save before permanent poison immunity.
pub fn poison_duration(
    warrior: &[u8],
    strength: i16,
    random: &mut RandomState,
) -> Result<Option<i16>, RuleError> {
    condition_duration(
        warrior,
        strength,
        Condition::Poisoned,
        SavingThrow::Chemical,
        random,
    )
}

fn condition_duration(
    warrior: &[u8],
    strength: i16,
    condition: Condition,
    save: SavingThrow,
    random: &mut RandomState,
) -> Result<Option<i16>, RuleError> {
    let warrior = WarriorRecord::from_bytes(span(warrior, 0, 3138)?);
    let saved = random.bound(100) <= warrior.saving_throw(save)?;
    let previous = warrior.condition(condition)?;
    if saved || previous < 0 {
        return Ok(None);
    }
    let strength = if warrior.race()? == 3 {
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
    condition_tick(warrior, Condition::Diseased)
}

pub fn poison_tick(warrior: &[u8]) -> Result<Option<DiseaseTick>, RuleError> {
    condition_tick(warrior, Condition::Poisoned)
}

/// getup tests CE and CB after decrementing positive condition bytes.
pub fn enemy_drain_tick(monster: &[u8]) -> Result<Option<DiseaseTick>, RuleError> {
    let monster = EnemyRecord::from_bytes(span(monster, 0, 288)?);
    let duration = monster.condition(Condition::Diseased);
    let health = monster.stamina()?;
    if duration == 0 || monster.condition(Condition::ComputerControl) != 0 || health <= 0 {
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
    if duration == 0 || monster.record().condition(Condition::ComputerControl) != 0 || health <= 0 {
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
    let record = EnemyRecord::from_bytes(monster);
    let amount = record.condition(Condition::DrainPsi);
    let mut psi = record.psi_points()?;
    if amount != 0 {
        psi = psi.wrapping_sub(amount).max(0);
    }
    if record.condition(Condition::RechargePsi) != 0 && record.has_creature_category(0) {
        psi = psi.wrapping_add(amount);
    }
    EnemyRecordMut::from_bytes(monster).set_psi_points(psi);
    Ok(())
}

fn condition_tick(warrior: &[u8], condition: Condition) -> Result<Option<DiseaseTick>, RuleError> {
    let warrior = WarriorRecord::from_bytes(span(warrior, 0, 3138)?);
    let duration = warrior.condition(condition)?;
    let health = warrior.stamina()?;
    if duration == 0 || health <= 0 || warrior.condition(Condition::ComputerControl)? < 0 {
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
    condition_outdoors(warriors, member, context, Condition::Diseased, random)
}

pub fn poison_outdoors(
    warriors: &[centurions_content::Warrior],
    member: usize,
    context: &crate::story_health::HealthContext,
    random: &mut RandomState,
) -> Result<crate::story_health::HealthOutcome, RuleError> {
    condition_outdoors(warriors, member, context, Condition::Poisoned, random)
}

fn condition_outdoors(
    warriors: &[centurions_content::Warrior],
    member: usize,
    context: &crate::story_health::HealthContext,
    condition: Condition,
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
    if let Some(tick) = condition_tick(&warriors[member].bytes, condition)? {
        outcome.warriors[member]
            .record_mut()
            .set_stamina(tick.health);
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
