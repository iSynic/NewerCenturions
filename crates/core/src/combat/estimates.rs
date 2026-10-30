//! The odds and damage estimates on the warrior panel. They differ from real
//! attack resolution on purpose: the panel shows what the original panel shows.
use crate::{RuleError, ammunition::LoadedItems};
use centurions_content::{
    Content, Warrior,
    records::{Condition, EnemyRecord, ItemDefinition},
};

/// The chance, damage range and average the warrior panel shows for the selected weapon against a
/// monster.
pub(crate) struct AttackEstimate {
    pub chance: i16,
    pub minimum: i16,
    pub maximum: i16,
    pub average: i16,
    pub loaded: LoadedItems,
}

/// Works out the chance to hit and the damage the panel shows. Chance is capped at 90 less five per
/// difficulty step, for the standard difficulties.
///
/// Original: updateodds, calcdamage
pub(crate) fn calculate(
    warrior: &Warrior,
    enemy_bytes: &[u8],
    difficulty: i16,
    loaded: &LoadedItems,
    content: &Content,
) -> Result<AttackEstimate, RuleError> {
    let actor = warrior.record();
    let monster = EnemyRecord::from_bytes(enemy_bytes);
    let mut chance = if !actor.toggle() {
        super::attack::melee_accuracy::against_enemy(&warrior.bytes, enemy_bytes)?
            .wrapping_add(i16::from(actor.inventory_slot(1).accuracy_modifier()))
            .max(5)
    } else {
        missile_chance(warrior, &monster, difficulty, loaded)?
    };
    // Only the five standard difficulties have this display cap.
    if (-2..=2).contains(&difficulty) {
        chance = chance.min(90 - difficulty * 5);
    }
    let mut loaded_after = loaded.clone();
    let (maximum, weapon_count) = damage_total(warrior, content, &mut loaded_after)?;
    let minimum = if actor.toggle() {
        weapon_count
    } else {
        actor.condition(Condition::BonusDamage)?.wrapping_add(1)
    };
    let mut average = if actor.toggle() {
        (maximum.wrapping_sub(minimum) / 2).wrapping_add(minimum)
    } else {
        maximum / 2
    };
    average = reduce_estimate(average, monster.ac()?);
    if actor.toggle() {
        average = reduce_estimate(average, monster.shield_chance()?);
    }
    Ok(AttackEstimate {
        chance,
        minimum,
        maximum,
        average,
        loaded: loaded_after,
    })
}

/// The panel's chance to hit with a missile weapon: 999 when the loaded ammunition or weapon
/// bypasses the hit roll, 100 against a monster that skips turns, otherwise the warrior's skill
/// less the monster's defenses, kept between 15 and 90 less five per difficulty step.
///
/// Original: updateodds
fn missile_chance(
    warrior: &Warrior,
    monster: &EnemyRecord<'_>,
    difficulty: i16,
    loaded: &LoadedItems,
) -> Result<i16, RuleError> {
    // The loaded ammunition and weapon count separately, before the displayed
    // weapon loads; summing them first would change the result.
    for bytes in [&loaded.ammo, &loaded.weapon].into_iter().flatten() {
        if ItemDefinition(bytes).bypasses_hit_roll()? {
            return Ok(999);
        }
    }
    let actor = warrior.record();
    let mut chance = 33_i16
        .wrapping_add(actor.missile_accuracy()?)
        .wrapping_add(i16::from(actor.inventory_slot(0).accuracy_modifier()));
    if monster.turn_skip_duration() != 0 {
        return Ok(100);
    }
    chance = chance
        .wrapping_sub(monster.missile_dodge_bonus()?)
        .wrapping_sub(monster.agility()? / 6)
        .wrapping_sub(monster.hit_dice()?.wrapping_mul(3))
        .wrapping_sub(i16::from(monster.guarding()).wrapping_mul(monster.hit_dice()?));
    chance = chance.wrapping_add(match monster.size()?.wrapping_abs() {
        0 => -5,
        1 | 2 => 5,
        3 => 10,
        _ => 0,
    });
    Ok(crate::rules::pin(
        chance,
        15_i16.wrapping_sub(difficulty.wrapping_mul(5)),
        90_i16.wrapping_sub(difficulty.wrapping_mul(5)),
    ))
}

/// The most damage the displayed weapon can do in a turn, and the weapon's attack count. The total
/// is half the warrior's normal attacks times the weapon's damage with its elements, loaded
/// ammunition and enhancement, plus bare-hand damage when not firing.
///
/// Original: calcdamage
fn damage_total(
    warrior: &Warrior,
    content: &Content,
    loaded: &mut LoadedItems,
) -> Result<(i16, i16), RuleError> {
    let actor = warrior.record();
    let firing = actor.toggle();
    let id = actor.displayed_weapon(firing)?;
    let mut damage = 0_i16;
    let mut weapon_count = 0;
    if id != 0 {
        let bytes = crate::ammunition::definition(content, id)?;
        let weapon = ItemDefinition(&bytes);
        weapon_count = weapon.attack_count()?;
        damage = combined_damage(weapon, firing)?;
        let ammunition = if firing && actor.loaded_ammunition()? != 0 {
            Some(crate::ammunition::definition(
                content,
                actor.loaded_ammunition()?,
            )?)
        } else {
            None
        };
        if let Some(bytes) = &ammunition {
            damage = damage.wrapping_add(combined_damage(ItemDefinition(bytes), true)?);
            loaded.ammo = Some(bytes.clone());
        }
        if weapon.rounds_per_shot()? > 1 {
            damage = damage.wrapping_mul(weapon.rounds_per_shot()?);
        }
        if !firing && weapon_count > 1 {
            damage = damage.wrapping_mul(weapon_count);
        }
        let mut multiplier = 1.0_f32;
        // Every inventory slot is searched, whatever the item count.
        for slot in 0..99 {
            let item = actor.inventory_slot(slot);
            if item.id() == id && item.equipped() != 0 {
                let ammo_bonus = if firing {
                    ammunition
                        .as_deref()
                        .or(loaded.ammo.as_deref())
                        .map(|bytes| i16::from(ItemDefinition(bytes).damage_enhancement()))
                        .unwrap_or(0)
                } else {
                    0
                };
                let bonus = i16::from(item.damage_enhancement()).wrapping_add(ammo_bonus);
                multiplier = (f64::from(100.0_f32 + f32::from(bonus)) / 100.0) as f32;
                break;
            }
        }
        // Rounded to single precision before truncating.
        damage = ((f64::from(damage) * f64::from(multiplier)) as f32).trunc() as i32 as i16;
        loaded.item = Some(bytes);
    }
    if !firing {
        damage = damage.wrapping_add(
            actor
                .bare_hand_damage()?
                .wrapping_add(actor.bare_hand_bonus()?),
        );
    }
    let mut total = (actor.normal_attacks()? / 2).wrapping_mul(damage);
    if firing && weapon_count > 1 {
        total = total.wrapping_mul(weapon_count);
    }
    Ok((total, weapon_count))
}

/// A weapon's damage including every positive element.
///
/// Original: calcdamage
fn combined_damage(item: ItemDefinition<'_>, firing: bool) -> Result<i16, RuleError> {
    let mut damage = if firing {
        item.missile_damage()?
    } else {
        item.melee_damage()?
    };
    for element in 0..7 {
        let amount = item.elemental_damage(element)?;
        if amount > 0 {
            damage = damage.wrapping_add(amount);
        }
    }
    Ok(damage)
}

/// Reduces damage by a percentage, rounding to the nearest whole point.
///
/// Original: calcdamage
fn reduce_estimate(damage: i16, resistance: i16) -> i16 {
    // Each of the two reductions rounds to single precision.
    let fraction = (f64::from(f32::from(resistance)) / 100.0) as f32;
    let remainder = (1.0 - f64::from(fraction)) as f32;
    let reduced = (f64::from(remainder) * f64::from(f32::from(damage))) as f32;
    ((f64::from(reduced) + 0.5) as f32).trunc() as i32 as i16
}
