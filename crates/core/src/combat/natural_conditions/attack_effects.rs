//! attack2 (0x1028a4) applies natural effects after physical damage. These rules
//! preserve its different warrior/enemy admission and duration behavior.
use crate::{RandomState, RuleError, combat::attack::AttackFeedback};
use centurions_content::{
    records::{
        Condition, ConditionTarget, DamageTargetKind, EnemyRecord, EnemyRecordMut, ItemDefinition,
        SavingThrow, WeaponCondition,
    },
    span,
};

pub(crate) struct NaturalEffect {
    pub extra_damage: i16,
    pub applied: bool,
    pub feedback: Option<AttackFeedback>,
    /// Psi taken by condition 7, which attack2 adds to the attacker's live record.
    pub drained_psi: i16,
}

impl NaturalEffect {
    fn none() -> Self {
        Self {
            extra_damage: 0,
            applied: false,
            feedback: None,
            drained_psi: 0,
        }
    }
    /// A reflected attack credits the record it drained, as attack2's aliased bodies do.
    pub fn credit_attacker(&self, attacker: &mut [u8]) -> Result<(), RuleError> {
        let psi = EnemyRecord::from_bytes(attacker).psi_points()?;
        EnemyRecordMut::from_bytes(attacker).set_psi_points(psi.wrapping_add(self.drained_psi));
        Ok(())
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
        7 => -17,
        8 => -18,
        9 => -34,
        10 => -26,
        11 => -22,
        12 => -19,
        13 => -20,
        14 => -21,
        15 => -23,
        16 => -15,
        17 => -16,
        19 => -47,
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
    if matches!(condition, 1..=6 | 9) {
        result.applied = apply_duration(&mut target, condition, strength, false, random)?;
        if result.applied && condition == 4 {
            target.halve_remaining_turn()?;
        }
    } else if condition == 7 {
        drain_psi(&mut target, source.level()?, &mut result, random)?;
    } else if condition == 8 {
        // attack2 has no NPC-target arm for condition 8.
    } else if condition == 19 {
        result.applied = blind(&mut target, random)?;
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
    if matches!(condition, 1..=6 | 9) {
        result.applied = apply_duration(&mut target, condition, strength, mind_shield, random)?;
        if result.applied && condition == 4 {
            target.halve_remaining_turn()?;
        }
    } else if condition == 7 {
        drain_psi(&mut target, source.level()?, &mut result, random)?;
    } else if condition == 8 {
        // attack2 0x102d68: twenty per attacker maximum stamina, as a sign-extended word.
        if !saves(&target, SavingThrow::Entanglement, random)? {
            let amount = source.maximum_stamina()?.wrapping_mul(20);
            target.shift_experience(i32::from(amount))?;
            result.applied = true;
        }
    } else if condition == 19 {
        result.applied = blind(&mut target, random)?;
    } else if condition == 10 {
        let threshold = target
            .saving_throw(SavingThrow::Mental)?
            .wrapping_add(if mind_shield { 50 } else { 0 });
        let saved = random.bound(100) <= threshold;
        if !saved && !target.is_cyborg()? {
            target.set_faction(source.faction()?);
            result.applied = true;
        }
    } else if (11..=17).contains(&condition) {
        // attack2 0x1028a4: warrior cases 11-17 share one shape; heat, cold, psionic and
        // atomic (12, 13, 15, 16) are compared by natural-elements.json.
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

/// savevs: one Rand(100) against the threshold; NPC thresholds above 99 always save.
fn saves(
    target: &ConditionTarget<'_>,
    save: SavingThrow,
    random: &mut RandomState,
) -> Result<bool, RuleError> {
    let threshold = target.saving_throw(save)?;
    Ok(random.bound(100) <= threshold || (target.is_enemy() && threshold > 99))
}

/// attack2 0x102cd0: three Psi per attacker level, capped by the target's Psi.
fn drain_psi(
    target: &mut ConditionTarget<'_>,
    level: i16,
    result: &mut NaturalEffect,
    random: &mut RandomState,
) -> Result<(), RuleError> {
    let psi = target.psi_points()?;
    if !saves(target, SavingThrow::Psionic, random)? && psi != 0 {
        result.drained_psi = level.wrapping_mul(3).min(psi);
        target.set_psi_points(psi.wrapping_sub(result.drained_psi));
        result.applied = true;
    }
    Ok(())
}

/// Condition 19: the save precedes the warrior cyborg exemption; the blindness is permanent.
fn blind(target: &mut ConditionTarget<'_>, random: &mut RandomState) -> Result<bool, RuleError> {
    if saves(target, SavingThrow::Entanglement, random)? || target.is_cyborg()? {
        return Ok(false);
    }
    target.set_duration(Condition::Blind, -1);
    Ok(true)
}

/// attack and attack2 roll a selector -10 weapon's condition when the weapon loads,
/// before the hit roll: mode 1 admits it on a failed save, mode 2 on Rand(100) at
/// most the threshold, any other mode always.
pub(crate) fn admit_weapon_condition(
    weapon: &[u8],
    target: &mut [u8],
    kind: DamageTargetKind,
    mind_shield: bool,
    random: &mut RandomState,
) -> Result<Option<WeaponCondition>, RuleError> {
    let Some(condition) = ItemDefinition(weapon).weapon_condition()? else {
        return Ok(None);
    };
    let unsupported = || RuleError::Unsupported {
        action: "combat.attack.condition",
        dependency: "weapon condition slot or saving throw outside the record",
    };
    if !(0..40).contains(&condition.slot) {
        return Err(unsupported());
    }
    let target = ConditionTarget::new(target, kind)?;
    let admitted = match condition.mode {
        1 => {
            let save = SavingThrow::from_index(condition.threshold).ok_or_else(unsupported)?;
            let threshold = target.saving_throw(save)?.wrapping_add(
                if !target.is_enemy() && save == SavingThrow::Mental && mind_shield {
                    50
                } else {
                    0
                },
            );
            !(random.bound(100) <= threshold || (target.is_enemy() && threshold > 99))
        }
        2 => random.bound(100) <= condition.threshold,
        _ => true,
    };
    Ok(admitted.then_some(condition))
}

/// After nonzero damage a nonnegative slot gains the amount; result -40 either way.
pub(crate) fn apply_weapon_condition(
    target: &mut [u8],
    kind: DamageTargetKind,
    condition: WeaponCondition,
) -> Result<AttackFeedback, RuleError> {
    let mut target = ConditionTarget::new(target, kind)?;
    let slot = condition.slot as usize;
    let current = target.slot_duration(slot)?;
    if current >= 0 {
        target.set_slot_duration(slot, current.wrapping_add(condition.amount));
    }
    Ok(AttackFeedback::defender(-40, 0))
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
    if saves(target, save, random)? {
        amount /= 2;
    }
    if target.elemental_protection(usize::from(condition - 11))? {
        amount /= 2;
    }
    Ok(amount)
}
