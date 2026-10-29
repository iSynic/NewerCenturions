//! Ordered equipment effects. Removal is intentionally not the inverse of wear.
use super::globals::GlobalWrite;
use super::record_rules::unsupported;
use crate::{RuleError, combat};
use centurions_content::records::{
    EquipmentModifiers, EquipmentStat as Stat, RetainedEquipmentModifiers,
};
use centurions_content::{Content, InventoryRecord, Warrior};

pub(super) fn change(
    warrior: &mut Warrior,
    stat: Stat,
    amount: i16,
    removing: bool,
    clamp: bool,
) -> Result<(), RuleError> {
    let old = stat.read(warrior)?;
    let value = if removing {
        old.wrapping_sub(amount)
    } else {
        old.wrapping_add(amount)
    };
    stat.write(warrior, if clamp { value.max(0) } else { value })?;
    Ok(())
}

pub(super) fn movement(warrior: &mut Warrior, content: &Content) -> Result<(), RuleError> {
    let race = warrior.record().race()?.wrapping_sub(1);
    let table = &content
        .map(0)
        .and_then(|map| map.combat.as_ref())
        .ok_or_else(|| unsupported("equipment movement profile missing"))?
        .race_moves;
    // The silent caller initializes unused race-table entries to zero.
    let speed = table.get(race as usize).copied().unwrap_or(0);
    combat::calculate_movement(warrior, speed)
}

pub(super) fn attributes(
    warrior: &mut Warrior,
    definition: &EquipmentModifiers,
    globals: &mut Vec<GlobalWrite>,
    removing: bool,
) -> Result<(), RuleError> {
    for selector in definition.attributes {
        match selector {
            0 => (),
            value if value < 0 => change(
                warrior,
                Stat::CreatureBonus(value.wrapping_abs().wrapping_sub(1)),
                definition.attribute_amount,
                removing,
                false,
            )?,
            1..16 => change(
                warrior,
                Stat::Skill(selector - 1),
                definition.attribute_amount,
                removing,
                false,
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

pub(super) fn defenses(
    warrior: &mut Warrior,
    item: &InventoryRecord,
    definition: &EquipmentModifiers,
    retained: &RetainedEquipmentModifiers,
    removing: bool,
) -> Result<(), RuleError> {
    change(
        warrior,
        Stat::ShieldChance,
        retained
            .shield_chance
            .wrapping_add(definition.shield_chance),
        removing,
        !removing,
    )?;
    if !removing {
        change(
            warrior,
            Stat::ShieldCost,
            retained.shield_cost.wrapping_add(definition.shield_cost),
            false,
            true,
        )?;
    }
    if definition.adds_energy {
        change(
            warrior,
            Stat::ShieldCapacity,
            definition.energy_capacity.wrapping_add(item.capacity()?),
            removing,
            true,
        )?;
        change(
            warrior,
            Stat::ShieldCharge,
            item.quantity()?,
            removing,
            true,
        )?;
    }
    Ok(())
}

pub(super) fn statistics(
    warrior: &mut Warrior,
    definition: &EquipmentModifiers,
    retained: &RetainedEquipmentModifiers,
    removing: bool,
) -> Result<(), RuleError> {
    let (accuracy, modifier) = if definition.kind == 2 {
        (
            Stat::MeleeAccuracy,
            if removing {
                retained.accuracy
            } else {
                retained.wear_melee_accuracy
            },
        )
    } else {
        (Stat::MissileAccuracy, retained.accuracy)
    };
    change(warrior, accuracy, modifier, removing, false)?;
    for (stat, amount, clamp) in [
        (Stat::Strength, definition.strength, false),
        (Stat::Quickness, definition.quickness, false),
        (
            Stat::MissileDodge,
            retained
                .missile_dodge
                .wrapping_add(definition.missile_dodge),
            removing,
        ),
        (
            Stat::MeleeDodge,
            retained.melee_dodge.wrapping_add(definition.melee_dodge),
            removing,
        ),
        (Stat::MovementBonus, definition.movement, false),
        (Stat::PsiResistance, definition.psi_resistance, false),
        (Stat::MaximumPsi, definition.maximum_psi, false),
        (
            Stat::WeaponRange,
            retained.range.wrapping_add(definition.range),
            false,
        ),
        (
            Stat::Armor,
            retained.armor.wrapping_add(definition.armor),
            removing,
        ),
    ] {
        change(warrior, stat, amount, removing, clamp)?;
    }
    if removing {
        change(
            warrior,
            Stat::ShieldCost,
            retained
                .shield_cost
                .wrapping_add(definition.removal_shield_cost),
            true,
            true,
        )?;
    }
    for (index, amount) in definition.resistances.iter().copied().enumerate() {
        if amount < 0 {
            change(warrior, Stat::Resistance(index), amount, !removing, false)?;
        }
    }
    Ok(())
}

pub(super) fn conditions(
    warrior: &mut Warrior,
    definition: &EquipmentModifiers,
    removing: bool,
) -> Result<(), RuleError> {
    // The original removal excludes selector 99; wear includes it.
    if definition.effect > 59 && definition.effect < if removing { 99 } else { 100 } {
        Stat::Condition(definition.effect_amount).write(warrior, 0)?;
    }
    if !removing && definition.effect == 122 {
        change(
            warrior,
            Stat::SpecialEffect122,
            definition.effect_amount,
            false,
            false,
        )?;
    }
    if definition.effect > 19 && definition.effect < 60 {
        let condition = Stat::Condition(definition.effect - 20);
        if removing {
            if condition.read(warrior)? < 0 {
                change(warrior, condition, definition.effect_amount, true, false)?;
            }
            if condition.read(warrior)? > 0 {
                condition.write(warrior, 0)?;
            }
        } else {
            if condition.read(warrior)? > -1 {
                condition.write(warrior, 0)?;
            }
            change(warrior, condition, definition.effect_amount, false, false)?;
        }
    }
    Ok(())
}
