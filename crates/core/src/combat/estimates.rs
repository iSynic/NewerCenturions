//! Read-only updateodds (0x161068) and calcdamage estimates. These deliberately
//! retain the display algorithm's differences from actual attack resolution.
use crate::{RuleError, ammunition::WorkBuffers};
use centurions_content::{
    Content, Warrior,
    records::{Condition, EnemyRecord, ItemDefinition},
};

pub(crate) struct AttackEstimate {
    pub chance: i16,
    pub minimum: i16,
    pub maximum: i16,
    pub average: i16,
    pub work: WorkBuffers,
}

pub(crate) fn calculate(
    warrior: &Warrior,
    enemy_bytes: &[u8],
    difficulty: i16,
    work: &WorkBuffers,
    content: &Content,
) -> Result<AttackEstimate, RuleError> {
    let actor = warrior.record();
    let enemy = EnemyRecord::from_bytes(enemy_bytes);
    let mut chance = if !actor.firing_mode() {
        super::attack::melee_accuracy::against_enemy(&warrior.bytes, enemy_bytes)?
            .wrapping_add(i16::from(actor.inventory_slot(1).accuracy_modifier()))
            .max(5)
    } else {
        missile_chance(warrior, &enemy, difficulty, work)?
    };
    // Only the five original difficulties have this additional display cap.
    if (-2..=2).contains(&difficulty) {
        chance = chance.min(90 - difficulty * 5);
    }
    let mut retained = work.clone();
    let (maximum, weapon_count) = damage_total(warrior, content, &mut retained)?;
    let minimum = if actor.firing_mode() {
        weapon_count
    } else {
        actor.condition(Condition::BonusDamage)?.wrapping_add(1)
    };
    let mut average = if actor.firing_mode() {
        (maximum.wrapping_sub(minimum) / 2).wrapping_add(minimum)
    } else {
        maximum / 2
    };
    average = reduce_estimate(average, enemy.armor()?);
    if actor.firing_mode() {
        average = reduce_estimate(average, enemy.shield_chance()?);
    }
    Ok(AttackEstimate {
        chance,
        minimum,
        maximum,
        average,
        work: retained,
    })
}

fn missile_chance(
    warrior: &Warrior,
    enemy: &EnemyRecord<'_>,
    difficulty: i16,
    work: &WorkBuffers,
) -> Result<i16, RuleError> {
    // updateodds reads the retained ammo/weapon independently, before loading
    // the displayed weapon. Summing these fields would change cancellation.
    for bytes in [&work.ammo, &work.weapon].into_iter().flatten() {
        if ItemDefinition(bytes).bypasses_hit_roll()? {
            return Ok(999);
        }
    }
    let actor = warrior.record();
    let mut chance = 33_i16
        .wrapping_add(actor.missile_accuracy()?)
        .wrapping_add(i16::from(actor.inventory_slot(0).accuracy_modifier()));
    if enemy.turn_skip_duration() != 0 {
        return Ok(100);
    }
    chance = chance
        .wrapping_sub(enemy.missile_dodge_bonus()?)
        .wrapping_sub(enemy.speed()? / 6)
        .wrapping_sub(enemy.level()?.wrapping_mul(3))
        .wrapping_sub(i16::from(enemy.guarding()).wrapping_mul(enemy.level()?));
    chance = chance.wrapping_add(match enemy.size()?.wrapping_abs() {
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

fn damage_total(
    warrior: &Warrior,
    content: &Content,
    work: &mut WorkBuffers,
) -> Result<(i16, i16), RuleError> {
    let actor = warrior.record();
    let firing = actor.firing_mode();
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
            work.ammo = Some(bytes.clone());
        }
        if weapon.rounds_per_shot()? > 1 {
            damage = damage.wrapping_mul(weapon.rounds_per_shot()?);
        }
        if !firing && weapon_count > 1 {
            damage = damage.wrapping_mul(weapon_count);
        }
        let mut multiplier = 1.0_f32;
        // The original searches all stored slots, independently of item count.
        for slot in 0..99 {
            let item = actor.inventory_slot(slot);
            if item.id() == id && item.equipment_flag() != 0 {
                let ammo_bonus = if firing {
                    ammunition
                        .as_deref()
                        .or(work.ammo.as_deref())
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
        // SANE stores Single before widening and truncating to a signed Long.
        damage = ((f64::from(damage) * f64::from(multiplier)) as f32).trunc() as i32 as i16;
        work.item = Some(bytes);
    }
    if !firing {
        damage = damage.wrapping_add(
            actor
                .bare_hand_damage()?
                .wrapping_add(actor.bare_hand_bonus()?),
        );
    }
    let mut total = (actor.actions_per_round()? / 2).wrapping_mul(damage);
    if firing && weapon_count > 1 {
        total = total.wrapping_mul(weapon_count);
    }
    Ok((total, weapon_count))
}

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

fn reduce_estimate(damage: i16, resistance: i16) -> i16 {
    // Preserve each Single rounding in updateodds' two separate reductions.
    let fraction = (f64::from(f32::from(resistance)) / 100.0) as f32;
    let remainder = (1.0 - f64::from(fraction)) as f32;
    let reduced = (f64::from(remainder) * f64::from(f32::from(damage))) as f32;
    ((f64::from(reduced) + 0.5) as f32).trunc() as i32 as i16
}
