use crate::{RandomState, RuleError};
use centurions_content::WARRIOR_BYTES;
use centurions_content::combat::MONSTER_BYTES;
use centurions_content::{
    records::{Condition, EnemyRecord, EnemyRecordMut, SavingThrow, WarriorRecord},
    span,
};

mod attack_effects;
pub(crate) use attack_effects::{
    admit_weapon_condition, apply_weapon_condition, monster_attack_effect, warrior_attack_effect,
};

/// Disease from a natural attack: the save comes before the permanent-condition test.
///
/// Original: attack2
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

/// Poison from a natural attack: the chemical save comes before poison immunity.
///
/// Original: attack2
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

/// The longer duration a natural attack gives a warrior, or None if the warrior saves, already has
/// the condition permanently, or the new duration would reach thirty. A warrior of race 3 gets no
/// added strength.
fn condition_duration(
    warrior: &[u8],
    strength: i16,
    condition: Condition,
    save: SavingThrow,
    random: &mut RandomState,
) -> Result<Option<i16>, RuleError> {
    let warrior = WarriorRecord::from_bytes(span(warrior, 0, WARRIOR_BYTES)?);
    let saved = random.bound(100) <= warrior.save(save)?;
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

/// A warrior's or monster's stamina after a condition's damage, and whether it killed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiseaseTick {
    pub health: i16,
    pub killed: bool,
}

/// Disease damages a warrior before the round's durations count down.
///
/// Original: reduce
pub fn disease_tick(warrior: &[u8]) -> Result<Option<DiseaseTick>, RuleError> {
    condition_tick(warrior, Condition::Diseased)
}

/// Poison damages a warrior the same way disease does.
pub fn poison_tick(warrior: &[u8]) -> Result<Option<DiseaseTick>, RuleError> {
    condition_tick(warrior, Condition::Poisoned)
}

/// A monster's draining conditions, tested after its durations count down.
///
/// Original: getup
pub fn enemy_drain_tick(monster: &[u8]) -> Result<Option<DiseaseTick>, RuleError> {
    let monster = EnemyRecord::from_bytes(span(monster, 0, MONSTER_BYTES)?);
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

/// A monster's stamina after a heal, and the amount it healed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnemyHeal {
    pub health: i16,
    pub amount: i16,
}

/// A hurt monster with a timer on its poison condition regains stamina by the timer's size, up to
/// its maximum. The amount is reported in full even when the maximum caps it.
pub fn enemy_poison_heal(
    monster: &centurions_content::combat::MonsterRecord,
) -> Result<Option<EnemyHeal>, RuleError> {
    span(&monster.bytes, 0, MONSTER_BYTES)?;
    let duration = i16::from(monster.poison_timer()?);
    let health = monster.stamina()?;
    let maximum = monster.stamina_max()?;
    if duration == 0 || health <= 0 || health >= maximum {
        return Ok(None);
    }
    let amount = duration.abs();
    // Healing past the maximum is capped; the amount is still reported in full.
    let health = health.wrapping_add(amount).min(maximum);
    Ok(Some(EnemyHeal { health, amount }))
}

/// A monster with a timer on its poison condition, not under the computer's control, loses stamina
/// by the timer's size.
pub fn enemy_poison_tick(
    monster: &centurions_content::combat::MonsterRecord,
) -> Result<Option<DiseaseTick>, RuleError> {
    span(&monster.bytes, 0, MONSTER_BYTES)?;
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

/// A monster's psi drains each turn, never below zero. A recharging monster of the first category
/// regains the drained amount instead (see docs/source-notes/combat.md).
///
/// Original: getup
pub fn reduce_enemy_psi(monster: &mut [u8]) -> Result<(), RuleError> {
    span(monster, 0, MONSTER_BYTES)?;
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

/// A condition that damages a warrior each round by its duration; a warrior with stamina of zero or
/// less is skipped.
fn condition_tick(warrior: &[u8], condition: Condition) -> Result<Option<DiseaseTick>, RuleError> {
    let warrior = WarriorRecord::from_bytes(span(warrior, 0, WARRIOR_BYTES)?);
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

/// A warrior dies of disease outside battle: killbody marks the warrior and plays
/// the sound, without battle cleanup.
///
/// Original: killbody
pub fn disease_outdoors(
    warriors: &[centurions_content::Warrior],
    member: usize,
    context: &crate::story::health::HealthContext,
    random: &mut RandomState,
) -> Result<crate::story::health::HealthOutcome, RuleError> {
    condition_outdoors(warriors, member, context, Condition::Diseased, random)
}

/// A warrior dies of poison outside battle in the same way.
pub fn poison_outdoors(
    warriors: &[centurions_content::Warrior],
    member: usize,
    context: &crate::story::health::HealthContext,
    random: &mut RandomState,
) -> Result<crate::story::health::HealthOutcome, RuleError> {
    condition_outdoors(warriors, member, context, Condition::Poisoned, random)
}

/// Applies a condition's damage to a warrior outside battle. A warrior it defeats is marked down
/// with a sound, and the party is lost when the last one falls.
fn condition_outdoors(
    warriors: &[centurions_content::Warrior],
    member: usize,
    context: &crate::story::health::HealthContext,
    condition: Condition,
    random: &mut RandomState,
) -> Result<crate::story::health::HealthOutcome, RuleError> {
    if warriors.is_empty() || warriors.len() > 6 || member >= warriors.len() {
        return Err(RuleError::Invalid("condition target outside party".into()));
    }
    for warrior in warriors {
        span(&warrior.bytes, 0, WARRIOR_BYTES)?;
    }
    let mut outcome = crate::story::health::HealthOutcome {
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
            use crate::story::health::HealthEvent;
            outcome.context.last_body = member as i16;
            outcome.context.kill_party = outcome.context.kill_party.wrapping_add(1);
            outcome.events.push(HealthEvent::Defeated {
                warrior: member,
                permanent: tick.health < -9,
            });
            outcome
                .events
                .push(HealthEvent::Sound(random.bound(2).wrapping_add(506)));
            if outcome.context.kill_party > warriors.len() as i8 - 1 {
                outcome.events.push(HealthEvent::PartyLost);
            }
        }
    }
    Ok(outcome)
}
