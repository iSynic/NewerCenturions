//! Ordered equipment effects. Removal is intentionally not the inverse of wear.
use super::globals::GlobalWrite;
use super::record_rules::unsupported;
use super::silent::Fit;
use crate::{RuleError, combat};
use centurions_content::records::{EquipmentModifiers, EquipmentStat as Stat, InstanceModifiers};
use centurions_content::{Content, InventoryRecord, Warrior};

/// Whether a changed stat stops at zero.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Floor {
    Unbounded,
    Zero,
}

/// Changes one stat by an item's amount, adding it when the item is worn and subtracting it when
/// removed, optionally stopping at zero.
pub(super) fn change(
    warrior: &mut Warrior,
    stat: Stat,
    amount: i16,
    fit: Fit,
    floor: Floor,
) -> Result<(), RuleError> {
    let old = stat.read(warrior)?;
    let value = if fit == Fit::Remove {
        old.wrapping_sub(amount)
    } else {
        old.wrapping_add(amount)
    };
    stat.write(
        warrior,
        if floor == Floor::Zero {
            value.max(0)
        } else {
            value
        },
    )?;
    Ok(())
}

/// Recomputes the warrior's movement from the race's movement allowance; races beyond the table
/// move at speed zero.
pub(super) fn movement(warrior: &mut Warrior, content: &Content) -> Result<(), RuleError> {
    let race = warrior.record().race()?.wrapping_sub(1);
    let table = &content
        .map(0)
        .and_then(|map| map.combat.as_ref())
        .ok_or_else(|| unsupported("equipment movement profile missing"))?
        .race_moves;
    // Races beyond the table move at speed zero.
    let speed = table.get(race as usize).copied().unwrap_or(0);
    combat::calculate_movement(warrior, speed)
}

/// Applies an item's attribute effects: a negative selector changes a creature bonus, 1 to 15
/// change a skill, and larger selectors name party-wide values, which wearing subtracts from and
/// removal clears.
pub(super) fn attributes(
    warrior: &mut Warrior,
    definition: &EquipmentModifiers,
    globals: &mut Vec<GlobalWrite>,
    fit: Fit,
) -> Result<(), RuleError> {
    let removing = fit == Fit::Remove;
    for selector in definition.attributes {
        match selector {
            0 => (),
            value if value < 0 => change(
                warrior,
                Stat::CreatureBonus(value.wrapping_abs().wrapping_sub(1)),
                definition.attribute_amount,
                fit,
                Floor::Unbounded,
            )?,
            1..16 => change(
                warrior,
                Stat::Skill(selector - 1),
                definition.attribute_amount,
                fit,
                Floor::Unbounded,
            )?,
            _ if removing => globals.push(GlobalWrite::Clear { selector }),
            // Wearing a zero amount subtracts nothing and needs no prior value.
            _ if definition.attribute_amount == 0 => (),
            _ => globals.push(GlobalWrite::Subtract {
                selector,
                amount: definition.attribute_amount.wrapping_abs(),
            }),
        }
    }
    Ok(())
}

/// Applies the shield and defense effects of an item. Wearing raises the shield chance and cost;
/// removal lowers the chance, and an item that adds energy changes the shield's capacity and
/// charge.
pub(super) fn defenses(
    warrior: &mut Warrior,
    item: &InventoryRecord,
    definition: &EquipmentModifiers,
    instance: &InstanceModifiers,
    fit: Fit,
) -> Result<(), RuleError> {
    let removing = fit == Fit::Remove;
    change(
        warrior,
        Stat::ShieldChance,
        instance
            .shield_chance
            .wrapping_add(definition.shield_chance),
        fit,
        if removing {
            Floor::Unbounded
        } else {
            Floor::Zero
        },
    )?;
    if !removing {
        change(
            warrior,
            Stat::ShieldCost,
            instance.shield_cost.wrapping_add(definition.shield_cost),
            Fit::Wear,
            Floor::Zero,
        )?;
    }
    if definition.adds_energy {
        change(
            warrior,
            Stat::ShieldCapacity,
            definition.energy_capacity.wrapping_add(item.capacity()?),
            fit,
            Floor::Zero,
        )?;
        change(
            warrior,
            Stat::ShieldCharge,
            item.charge()?,
            fit,
            Floor::Zero,
        )?;
    }
    Ok(())
}

/// Applies an item's changes to accuracy, strength, quickness, dodge, movement, psi resistance,
/// psi, range, armor and resistances. Removal is not the inverse of wear: it floors dodge and armor
/// at zero, takes off the removal shield cost, uses the instance's accuracy for a melee weapon
/// where wearing uses its melee accuracy, and applies only negative resistances, in the opposite
/// direction to the other stats.
pub(super) fn statistics(
    warrior: &mut Warrior,
    definition: &EquipmentModifiers,
    instance: &InstanceModifiers,
    fit: Fit,
) -> Result<(), RuleError> {
    let removing = fit == Fit::Remove;
    let (accuracy, modifier) = if definition.kind == 2 {
        (
            Stat::MeleeAccuracy,
            if removing {
                instance.accuracy
            } else {
                instance.wear_melee_accuracy
            },
        )
    } else {
        (Stat::MissileAccuracy, instance.accuracy)
    };
    change(warrior, accuracy, modifier, fit, Floor::Unbounded)?;
    let removal_floor = if removing {
        Floor::Zero
    } else {
        Floor::Unbounded
    };
    for (stat, amount, floor) in [
        (Stat::Strength, definition.strength, Floor::Unbounded),
        (Stat::Quickness, definition.quickness, Floor::Unbounded),
        (
            Stat::MissileDodge,
            instance
                .missile_dodge
                .wrapping_add(definition.missile_dodge),
            removal_floor,
        ),
        (
            Stat::MeleeDodge,
            instance.melee_dodge.wrapping_add(definition.melee_dodge),
            removal_floor,
        ),
        (Stat::MovementBonus, definition.movement, Floor::Unbounded),
        (
            Stat::PsiResistance,
            definition.psi_resistance,
            Floor::Unbounded,
        ),
        (Stat::MaximumPsi, definition.maximum_psi, Floor::Unbounded),
        (
            Stat::WeaponRange,
            instance.range.wrapping_add(definition.range),
            Floor::Unbounded,
        ),
        (
            Stat::Armor,
            instance.armor.wrapping_add(definition.armor),
            removal_floor,
        ),
    ] {
        change(warrior, stat, amount, fit, floor)?;
    }
    if removing {
        change(
            warrior,
            Stat::ShieldCost,
            instance
                .shield_cost
                .wrapping_add(definition.removal_shield_cost),
            Fit::Remove,
            Floor::Zero,
        )?;
    }
    for (index, amount) in definition.resistances.iter().copied().enumerate() {
        if amount < 0 {
            change(
                warrior,
                Stat::Resistance(index),
                amount,
                if removing { Fit::Wear } else { Fit::Remove },
                Floor::Unbounded,
            )?;
        }
    }
    Ok(())
}

/// Applies an item's effects on the warrior's conditions. Removal is not the inverse of wear:
/// effects 60 to 98 clear the condition numbered by the effect's amount on wear and on removal,
/// while effect 99 clears it only on wear (see docs/source-notes/rules.md, equipment). Effects 20
/// to 59 adjust the condition numbered effect minus 20 by the amount.
pub(super) fn conditions(
    warrior: &mut Warrior,
    definition: &EquipmentModifiers,
    fit: Fit,
) -> Result<(), RuleError> {
    let removing = fit == Fit::Remove;
    // Removal leaves condition 99 alone while wearing sets it, a quirk the game
    // has; removal is not the inverse of wear.
    if definition.effect > 59 && definition.effect < if removing { 99 } else { 100 } {
        Stat::Condition(definition.effect_amount).write(warrior, 0)?;
    }
    if !removing && definition.effect == 122 {
        change(
            warrior,
            Stat::SpecialEffect122,
            definition.effect_amount,
            Fit::Wear,
            Floor::Unbounded,
        )?;
    }
    if definition.effect > 19 && definition.effect < 60 {
        let condition = Stat::Condition(definition.effect - 20);
        if removing {
            if condition.read(warrior)? < 0 {
                change(
                    warrior,
                    condition,
                    definition.effect_amount,
                    Fit::Remove,
                    Floor::Unbounded,
                )?;
            }
            if condition.read(warrior)? > 0 {
                condition.write(warrior, 0)?;
            }
        } else {
            if condition.read(warrior)? > -1 {
                condition.write(warrior, 0)?;
            }
            change(
                warrior,
                condition,
                definition.effect_amount,
                Fit::Wear,
                Floor::Unbounded,
            )?;
        }
    }
    Ok(())
}
