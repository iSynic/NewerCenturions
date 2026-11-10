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
    average = reduce_estimate(average, monster.armor_rating()?);
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

/// One hit of the displayed weapon: its damage, the weapon's attack count, and the item and
/// ammunition definitions it loaded.
struct Hit {
    damage: i16,
    attack_count: i16,
    item: Option<Vec<u8>>,
    ammunition: Option<Vec<u8>>,
}

/// The damage of one hit with the firearm or the melee weapon on display. The firearm adds its
/// loaded ammunition; each is multiplied by its rounds per shot and the melee weapon by its attack
/// count, then scaled by the enhancement of the first worn slot holding the same item, the
/// firearm's with the ammunition's enhancement added (the ammunition loaded before when the
/// firearm has none). With no worn slot holding the weapon the scale is `unmatched`. The melee hit
/// adds the bare-hand damage and bonus, with or without a weapon.
///
/// Original: calcdamage
fn hit(
    warrior: &Warrior,
    content: &Content,
    loaded_ammunition: Option<&[u8]>,
    firing: bool,
    unmatched: f32,
) -> Result<Hit, RuleError> {
    let actor = warrior.record();
    let id = actor.displayed_weapon(firing)?;
    let mut hit = Hit {
        damage: 0,
        attack_count: 0,
        item: None,
        ammunition: None,
    };
    if id != 0 {
        let bytes = crate::ammunition::definition(content, id)?;
        let weapon = ItemDefinition(&bytes);
        hit.attack_count = weapon.attack_count()?;
        let mut damage = combined_damage(weapon, firing)?;
        if firing && actor.loaded_ammunition()? != 0 {
            let ammunition = crate::ammunition::definition(content, actor.loaded_ammunition()?)?;
            damage = damage.wrapping_add(combined_damage(ItemDefinition(&ammunition), true)?);
            hit.ammunition = Some(ammunition);
        }
        if weapon.rounds_per_shot()? > 1 {
            damage = damage.wrapping_mul(weapon.rounds_per_shot()?);
        }
        if !firing && hit.attack_count > 1 {
            damage = damage.wrapping_mul(hit.attack_count);
        }
        let mut scale = unmatched;
        // Every inventory slot is searched, whatever the item count.
        for slot in 0..99 {
            let item = actor.inventory_slot(slot);
            if item.id() == id && item.equipped() != 0 {
                let ammo_bonus = if firing {
                    hit.ammunition
                        .as_deref()
                        .or(loaded_ammunition)
                        .map(|bytes| i16::from(ItemDefinition(bytes).damage_enhancement()))
                        .unwrap_or(0)
                } else {
                    0
                };
                let bonus = i16::from(item.damage_enhancement()).wrapping_add(ammo_bonus);
                scale = (f64::from(100.0_f32 + f32::from(bonus)) / 100.0) as f32;
                break;
            }
        }
        // Rounded to single precision before truncating.
        hit.damage = ((f64::from(damage) * f64::from(scale)) as f32).trunc() as i32 as i16;
        hit.item = Some(bytes);
    }
    if !firing {
        hit.damage = hit.damage.wrapping_add(
            actor
                .bare_hand_damage()?
                .wrapping_add(actor.bare_hand_bonus()?),
        );
    }
    Ok(hit)
}

/// The most damage the displayed weapon can do in a turn, and the weapon's attack count: half the
/// warrior's normal attacks times one hit, times the attack count again when firing. The weapon
/// and its ammunition become the loaded items.
///
/// Original: calcdamage
fn damage_total(
    warrior: &Warrior,
    content: &Content,
    loaded: &mut LoadedItems,
) -> Result<(i16, i16), RuleError> {
    let firing = warrior.record().toggle();
    let hit = hit(
        warrior,
        content,
        loaded.ammo.as_deref(),
        firing,
        UNMATCHED_SCALE,
    )?;
    if let Some(ammunition) = hit.ammunition {
        loaded.ammo = Some(ammunition);
    }
    if let Some(item) = hit.item {
        loaded.item = Some(item);
    }
    let mut total = (warrior.record().actions_per_round()? / 2).wrapping_mul(hit.damage);
    if firing && hit.attack_count > 1 {
        total = total.wrapping_mul(hit.attack_count);
    }
    Ok((total, hit.attack_count))
}

/// The damage per hit the Items window's warrior panel shows for the firearm or the melee weapon
/// on display. It only reads: the loads it would make are made by the commands that refresh the
/// panel.
///
/// Original: calcdamage
pub(crate) fn sheet_damage(
    warrior: &Warrior,
    content: &Content,
    loaded: &LoadedItems,
    firearm: bool,
) -> Result<i16, RuleError> {
    Ok(hit(
        warrior,
        content,
        loaded.ammo.as_deref(),
        firearm,
        UNMATCHED_SCALE,
    )?
    .damage)
}

/// The scale of a weapon that no worn slot holds.
const UNMATCHED_SCALE: f32 = 1.0;

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
