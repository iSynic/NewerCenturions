//! Melee hit modifiers shared by player attacks. Keep the monster armor-based
//! defense distinct from a warrior's calculated dodge statistic.
use crate::RuleError;
use centurions_content::records::{Condition, EnemyRecord, WarriorRecord};

/// The attacker's chance after what hinders its swing: entangled and hindered attacks subtract
/// their strength, and slowed or blind (15) or confused (10) subtract a flat amount.
fn attacker_penalties(attacker: &WarriorRecord<'_>, mut chance: i16) -> Result<i16, RuleError> {
    for condition in [Condition::Entangled, Condition::HinderedAttacks] {
        chance = chance.wrapping_sub(attacker.condition(condition)?.wrapping_abs());
    }
    for (condition, penalty) in [
        (Condition::MovingSlowly, 15),
        (Condition::Blind, 15),
        (Condition::Confused, 10),
    ] {
        if attacker.condition(condition)? != 0 {
            chance = chance.wrapping_sub(penalty);
        }
    }
    Ok(chance)
}

/// A warrior's chance to hit a monster: the base 33 and melee accuracy, less the attacker's
/// hindrances, plus the warrior's bonus against each creature category the monster belongs to, less
/// the monster's armor rating and its shielding, and adjusted by the monster's defensive conditions.
pub(in crate::combat) fn against_enemy(attacker: &[u8], defender: &[u8]) -> Result<i16, RuleError> {
    let attacker = WarriorRecord::from_bytes(attacker);
    let defender = EnemyRecord::from_bytes(defender);
    let mut chance =
        attacker_penalties(&attacker, 33_i16.wrapping_add(attacker.melee_accuracy()?))?;
    for category in 0..8 {
        if defender.has_creature_category(category) {
            chance = chance.wrapping_add(attacker.special(category)?);
        }
    }
    chance = chance.wrapping_sub(defender.armor_rating()?);
    chance = chance.wrapping_sub(
        defender
            .condition(Condition::ShieldedFromHits)
            .abs()
            .wrapping_mul(2),
    );
    for (condition, sign) in [
        (Condition::Entangled, 1_i16),
        (Condition::HinderedDefense, 1),
        (Condition::IncreasedDefense, -1),
    ] {
        chance = chance.wrapping_add(defender.condition(condition).abs().wrapping_mul(sign));
    }
    for (condition, bonus) in [
        (Condition::MovingSlowly, 15),
        (Condition::Blind, 15),
        (Condition::Confused, 10),
    ] {
        if defender.condition(condition) != 0 {
            chance = chance.wrapping_add(bonus);
        }
    }
    Ok(chance)
}

/// A warrior's chance to hit another warrior: as against a monster, but the defender's melee dodge
/// replaces armor rating and there are no creature categories.
pub(in crate::combat) fn against_warrior(
    attacker: &[u8],
    defender: &[u8],
) -> Result<i16, RuleError> {
    let attacker = WarriorRecord::from_bytes(attacker);
    let defender = WarriorRecord::from_bytes(defender);
    let mut chance =
        attacker_penalties(&attacker, 33_i16.wrapping_add(attacker.melee_accuracy()?))?;
    chance = chance.wrapping_sub(defender.melee_dodge()?);
    chance = chance.wrapping_sub(
        defender
            .condition(Condition::ShieldedFromHits)?
            .wrapping_abs()
            .wrapping_mul(2),
    );
    for (condition, sign) in [
        (Condition::Entangled, 1_i16),
        (Condition::HinderedDefense, 1),
        (Condition::IncreasedDefense, -1),
    ] {
        chance = chance.wrapping_add(
            defender
                .condition(condition)?
                .wrapping_abs()
                .wrapping_mul(sign),
        );
    }
    for (condition, bonus) in [
        (Condition::MovingSlowly, 15),
        (Condition::Blind, 15),
        (Condition::Confused, 10),
    ] {
        if defender.condition(condition)? != 0 {
            chance = chance.wrapping_add(bonus);
        }
    }
    Ok(chance)
}

/// A monster's chance to hit a warrior, natural or armed. Unlike a warrior's melee, the defender's
/// attribute dodge bonus does not count.
///
/// Original: attack2
pub(in crate::combat) fn enemy_against_warrior(
    attacker: &EnemyRecord<'_>,
    defender: &WarriorRecord<'_>,
    weapon_accuracy: i16,
) -> Result<i16, RuleError> {
    let mut chance = 33_i16
        .wrapping_add(attacker.hit_dice()?.wrapping_mul(5))
        .wrapping_add(attacker.damage_plus()?.wrapping_mul(5))
        .wrapping_add(weapon_accuracy);
    for condition in [Condition::Entangled, Condition::HinderedAttacks] {
        chance = chance.wrapping_sub(attacker.condition(condition).abs());
    }
    for (condition, penalty) in [
        (Condition::MovingSlowly, 15),
        (Condition::Blind, 15),
        (Condition::Confused, 10),
    ] {
        if attacker.condition(condition) != 0 {
            chance = chance.wrapping_sub(penalty);
        }
    }
    for (condition, sign) in [
        (Condition::Entangled, 1_i16),
        (Condition::HinderedDefense, 1),
        (Condition::IncreasedDefense, -1),
    ] {
        chance = chance.wrapping_add(
            defender
                .condition(condition)?
                .wrapping_abs()
                .wrapping_mul(sign),
        );
    }
    for (condition, bonus) in [
        (Condition::MovingSlowly, 15),
        (Condition::Blind, 15),
        (Condition::Confused, 10),
    ] {
        if defender.condition(condition)? != 0 {
            chance = chance.wrapping_add(bonus);
        }
    }
    chance = chance.wrapping_sub(defender.base_melee_dodge()?);
    if (attacker.has_creature_category(4) || attacker.has_creature_category(5))
        && defender.condition(Condition::SeventhLevelProtection)? != 0
    {
        chance = chance.wrapping_sub(10);
    }
    Ok(chance)
}

/// A monster's chance to hit another monster: 33 plus 5 per hit die and per point of damage plus,
/// less the attacker's hindrances, adjusted by the defender's conditions and armor rating. An attacker of
/// creature category 4 or 5 loses 10 against a seventh-level-protected defender.
pub(in crate::combat) fn enemy_against_enemy(
    attacker: &EnemyRecord<'_>,
    defender: &EnemyRecord<'_>,
    weapon_accuracy: i16,
) -> Result<i16, RuleError> {
    let mut chance = 33_i16
        .wrapping_add(attacker.hit_dice()?.wrapping_mul(5))
        .wrapping_add(attacker.damage_plus()?.wrapping_mul(5))
        .wrapping_add(weapon_accuracy);
    for condition in [Condition::Entangled, Condition::HinderedAttacks] {
        chance = chance.wrapping_sub(attacker.condition(condition).abs());
    }
    for (condition, penalty) in [
        (Condition::MovingSlowly, 15),
        (Condition::Blind, 15),
        (Condition::Confused, 10),
    ] {
        if attacker.condition(condition) != 0 {
            chance = chance.wrapping_sub(penalty);
        }
    }
    for (condition, sign) in [
        (Condition::Entangled, 1_i16),
        (Condition::HinderedDefense, 1),
        (Condition::IncreasedDefense, -1),
    ] {
        chance = chance.wrapping_add(defender.condition(condition).abs().wrapping_mul(sign));
    }
    for (condition, bonus) in [
        (Condition::MovingSlowly, 15),
        (Condition::Blind, 15),
        (Condition::Confused, 10),
    ] {
        if defender.condition(condition) != 0 {
            chance = chance.wrapping_add(bonus);
        }
    }
    chance = chance.wrapping_sub(defender.armor_rating()?);
    if (attacker.has_creature_category(4) || attacker.has_creature_category(5))
        && defender.condition(Condition::SeventhLevelProtection) != 0
    {
        chance = chance.wrapping_sub(10);
    }
    Ok(chance)
}
