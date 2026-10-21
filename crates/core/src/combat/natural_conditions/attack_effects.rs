//! attack2 (0x1028a4) applies natural effects after physical damage. These rules
//! preserve its different warrior/enemy admission and duration behavior.
use crate::{RandomState, RuleError, combat::attack::AttackFeedback};
use centurions_content::{
    records::{Condition, ConditionTarget, DamageTargetKind, EnemyRecord, SavingThrow},
    span,
};

pub(crate) struct NaturalEffect {
    pub extra_damage: i16,
    pub applied: bool,
    pub feedback: Option<AttackFeedback>,
}

impl NaturalEffect {
    fn none() -> Self {
        Self {
            extra_damage: 0,
            applied: false,
            feedback: None,
        }
    }
    fn set_feedback(&mut self, condition: u8) {
        if self.applied {
            self.feedback = result_code(condition)
                .map(|code| AttackFeedback::defender(code, self.extra_damage));
        }
    }
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

/// None is magic immunity: the caller discards physical damage as well.
pub(crate) fn monster_attack_effect(
    target: &mut [u8],
    source: &[u8],
    index: usize,
    random: &mut RandomState,
) -> Result<Option<NaturalEffect>, RuleError> {
    let mut target = ConditionTarget::new(target, DamageTargetKind::Enemy)?;
    span(source, 0, 288)?;
    if index >= 6 {
        return Err(RuleError::Invalid("natural attack index".into()));
    }
    let source = EnemyRecord::from_bytes(source);
    let attack = source.natural_attack(index);
    let condition = attack.condition;
    let mut result = NaturalEffect::none();
    if condition == 0 {
        return Ok(Some(result));
    }
    let strength = random.range(source.level()? / 2, source.level()?).max(1);
    if target.magic_immunity()? > 100 {
        return Ok(None);
    }
    if matches!(condition, 1 | 2 | 4 | 5 | 6 | 9) {
        result.applied = apply_duration(&mut target, condition, strength, false, random)?;
        if result.applied && condition == 4 {
            target.halve_remaining_turn()?;
        }
    } else if condition == 10 {
        let threshold = target.saving_throw(SavingThrow::Mental)?;
        if random.bound(100) > threshold && threshold <= 99 {
            target.set_faction(source.faction()?);
            result.applied = true;
        }
    } else if (11..=17).contains(&condition) {
        result.extra_damage = extra_damage(&target, condition, attack.effect_maximum, random)?;
        result.applied = result.extra_damage != 0;
    } else {
        return Err(unsupported_condition());
    }
    result.set_feedback(condition);
    Ok(Some(result))
}

/// Called only when attack2 retained no physical weapon on entry.
pub(crate) fn warrior_attack_effect(
    target: &mut [u8],
    source: &[u8],
    index: usize,
    mind_shield: bool,
    random: &mut RandomState,
) -> Result<NaturalEffect, RuleError> {
    let mut target = ConditionTarget::new(target, DamageTargetKind::Warrior)?;
    span(source, 0, 288)?;
    if index >= 6 {
        return Err(RuleError::Invalid("natural attack index".into()));
    }
    let source = EnemyRecord::from_bytes(source);
    let attack = source.natural_attack(index);
    let condition = attack.condition;
    let prior_faction = target.faction()?;
    let mut result = NaturalEffect::none();
    if condition == 0 {
        return Ok(result);
    }
    let strength = random.range(source.level()? / 2, source.level()?).max(1);
    if matches!(condition, 1 | 2 | 4 | 5 | 6 | 9) {
        result.applied = apply_duration(&mut target, condition, strength, mind_shield, random)?;
        if result.applied && condition == 4 {
            target.halve_remaining_turn()?;
        }
    } else if condition == 10 {
        let threshold = target
            .saving_throw(SavingThrow::Mental)?
            .wrapping_add(if mind_shield { 50 } else { 0 });
        let saved = random.bound(100) <= threshold;
        if !saved && !target.is_cyborg()? {
            target.set_faction(source.faction()?);
            result.applied = true;
        }
    } else if matches!(condition, 11 | 14 | 17) {
        result.extra_damage = extra_damage(&target, condition, attack.effect_maximum, random)?;
        result.applied = result.extra_damage != 0;
    } else {
        return Err(unsupported_condition());
    }
    if condition != 10 || prior_faction == 0 {
        result.set_feedback(condition);
    }
    Ok(result)
}

fn unsupported_condition() -> RuleError {
    RuleError::Unsupported {
        action: "combat.attack.condition",
        dependency: "natural condition outside supplied actor families",
    }
}

/// Save before permanent-condition and cyborg gates, including refused effects.
fn apply_duration(
    target: &mut ConditionTarget<'_>,
    effect: u8,
    strength: i16,
    mind_shield: bool,
    random: &mut RandomState,
) -> Result<bool, RuleError> {
    let (save, condition, cyborg_immune) = match effect {
        1 => (SavingThrow::Mental, Condition::Retreating, true),
        2 => (SavingThrow::Biological, Condition::Helpless, true),
        3 => (SavingThrow::Psionic, Condition::ScanBlocked, false),
        4 => (SavingThrow::Entanglement, Condition::Entangled, false),
        5 => (SavingThrow::Chemical, Condition::Poisoned, true),
        6 => (SavingThrow::Mental, Condition::Confused, true),
        9 => (SavingThrow::Biological, Condition::Diseased, true),
        _ => return Err(RuleError::Invalid("natural duration condition".into())),
    };
    let threshold = target.saving_throw(save)?.wrapping_add(
        if !target.is_enemy() && save == SavingThrow::Mental && mind_shield {
            50
        } else {
            0
        },
    );
    let saved = random.bound(100) <= threshold || (target.is_enemy() && threshold > 99);
    let previous = target.duration(condition)?;
    if saved || previous < 0 {
        return Ok(false);
    }
    let magnitude = if cyborg_immune && target.is_cyborg()? {
        0
    } else {
        strength.wrapping_abs()
    };
    let duration = previous.wrapping_add(magnitude);
    if target.is_enemy() || duration < 30 {
        target.set_duration(condition, duration);
    }
    // Helplessness clears the turn even if the warrior duration exceeded its cap.
    if effect == 2 {
        target.clear_remaining_turn();
    }
    Ok(true)
}

/// The damage draw precedes the save and protection halves, including cyborg immunity.
fn extra_damage(
    target: &ConditionTarget<'_>,
    condition: u8,
    bound: i16,
    random: &mut RandomState,
) -> Result<i16, RuleError> {
    let save = match condition {
        11 => SavingThrow::Chemical,
        12 => SavingThrow::Heat,
        13 => SavingThrow::Cold,
        14 => SavingThrow::Electrical,
        15 => SavingThrow::Psionic,
        16 => SavingThrow::Atomic,
        17 => SavingThrow::Biological,
        _ => return Err(RuleError::Invalid("natural elemental condition".into())),
    };
    let mut amount = random.bound(bound);
    if target.is_cyborg()? {
        if condition == 14 {
            amount = amount.wrapping_mul(2);
        } else if condition == 17 {
            amount = 0;
        }
    }
    let threshold = target.saving_throw(save)?;
    if random.bound(100) <= threshold || (target.is_enemy() && threshold > 99) {
        amount /= 2;
    }
    if target.elemental_protection(usize::from(condition - 11))? {
        amount /= 2;
    }
    Ok(amount)
}
