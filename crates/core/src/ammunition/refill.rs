use super::{
    AmmunitionOutcome, AmmunitionResult, RefitFlags, WorkBuffers, definition, initial, unsupported,
};
use crate::feedback::{CLIP_SOUND_BASE, ERROR_SOUND};
use crate::{RandomState, RuleError};
use centurions_content::records::{ItemDefinition, item_class};
use centurions_content::{Content, InventoryRecord, Warrior};

/// Cargo Pod `STR#` 3 "Warnings", entry 49: the refit ran out of ammunition.
const REFIT_INCOMPLETE_WARNING: i16 = 49;

fn refill_sound(result: &mut AmmunitionResult, random: &mut RandomState, silent: bool) {
    if !silent {
        result.sounds.push(CLIP_SOUND_BASE + random.bound(2));
    }
}

fn partial_warning(result: &mut AmmunitionResult, enabled: bool) {
    if enabled {
        result.warnings.push(REFIT_INCOMPLETE_WARNING);
        result.sounds.push(ERROR_SOUND);
        result.feedback.push((
            result.sounds.len(),
            crate::Event::CollectionRejected {
                warning: REFIT_INCOMPLETE_WARNING,
            },
        ));
    }
}

fn refill_energy(
    result: &mut AmmunitionResult,
    content: &Content,
    slot: usize,
    amount: i16,
    remove_first: bool,
) -> Result<(), RuleError> {
    if i16::from(result.warrior.record().inventory_slot(slot).category_code())
        != item_class::SHIELD_BATTERY
    {
        return Ok(());
    }
    if result.warrior.record().inventory_quantity(slot)? > 0 {
        if remove_first {
            refit_equipment(result, content, slot, false)?;
        }
        result.warrior.record_mut().clear_equipped_battery();
        result
            .warrior
            .record_mut()
            .set_inventory_equipment_flag(slot, 0);
        refit_equipment(result, content, slot, true)?;
        if result
            .warrior
            .record()
            .inventory_slot(slot)
            .equipment_flag()
            != 0
        {
            let id = result.warrior.record().inventory_slot(slot).id();
            result
                .sounds
                .push(ItemDefinition(&definition(content, id)?).pickup_sound()?);
        }
    } else {
        let energy = result
            .warrior
            .record()
            .shield_points()?
            .wrapping_add(amount);
        result.warrior.record_mut().set_shield_points(energy);
    }
    Ok(())
}

fn refit_equipment(
    result: &mut AmmunitionResult,
    content: &Content,
    slot: usize,
    wearing: bool,
) -> Result<(), RuleError> {
    // Sessions always hold the retained row; refit's helper selects the battery row.
    let retained = result
        .retained_record
        .as_ref()
        .ok_or("battery re-wear without a retained inventory record")?;
    let equipped = crate::editor_items::retained_equipment(
        &result.warrior,
        content,
        slot,
        wearing,
        false,
        retained,
    )?;
    result.warrior = equipped.warrior;
    result.retained_record = Some(equipped.record);
    result.global_writes.extend(equipped.global_writes);
    result.work.item = Some(equipped.definition);
    for warning in equipped.warnings {
        result.warnings.push(warning);
        result.feedback.push((
            result.sounds.len(),
            crate::Event::CollectionRejected { warning },
        ));
    }
    if let Some(level) = equipped.level_refusal {
        result.feedback.push((
            result.sounds.len(),
            crate::Event::Feedback(crate::feedback::NativeFeedback::EquipmentLevel { level }),
        ));
    }
    Ok(())
}

fn admits_retained_item(
    result: &mut AmmunitionResult,
    id: i16,
    content: &Content,
    flags: RefitFlags,
) -> Result<bool, RuleError> {
    if flags.force || flags.in_combat {
        result.work.ammo = Some(definition(content, id)?);
        return Ok(matches!(
            ItemDefinition(
                result
                    .work
                    .item
                    .as_deref()
                    .ok_or_else(|| RuleError::from("ITEM work missing"))?
            )
            .category()?,
            item_class::MELEE_WEAPON | item_class::MISSILE_WEAPON
        ));
    }
    Ok(true)
}

pub fn refit(
    warrior: &Warrior,
    reserves: &[i32],
    content: &Content,
    flags: RefitFlags,
    work: &WorkBuffers,
    random: &mut RandomState,
) -> Result<AmmunitionResult, RuleError> {
    // The historical primitive API supplies its selected battery row. Session
    // callers supply the actual retained record through refit_with_record.
    let retained = warrior
        .inventory()?
        .into_iter()
        .find(|row| i16::from(row.category_code()) == item_class::SHIELD_BATTERY);
    refit_with_record(
        warrior,
        reserves,
        content,
        flags,
        work,
        random,
        retained.as_ref(),
    )
}

pub(crate) fn refit_with_record(
    warrior: &Warrior,
    reserves: &[i32],
    content: &Content,
    flags: RefitFlags,
    work: &WorkBuffers,
    random: &mut RandomState,
    retained: Option<&InventoryRecord>,
) -> Result<AmmunitionResult, RuleError> {
    let inventory = warrior.inventory()?;
    let mut result = initial(warrior, reserves, work)?;
    result.retained_record = retained.cloned();
    let mut next_random = random.clone();
    if !refill_existing_magazines(&mut result, &inventory, content, flags, &mut next_random)? {
        select_ammunition_for_empty_magazine(
            &mut result,
            &inventory,
            content,
            flags,
            &mut next_random,
        )?;
    }
    Ok(result.finish(random, next_random))
}

/// Refill selected ammunition before considering a different supply. The native
/// partial branch does not debit stock; exact stock completes this first pass.
fn refill_existing_magazines(
    result: &mut AmmunitionResult,
    inventory: &[InventoryRecord],
    content: &Content,
    flags: RefitFlags,
    random: &mut RandomState,
) -> Result<bool, RuleError> {
    for (slot, record) in inventory.iter().enumerate() {
        let loaded = result.warrior.record().inventory_quantity(slot)?;
        let capacity = record.capacity()?;
        let ammo = record.ammunition_id()?;
        if record.equipment_flag() == 0 || loaded >= capacity || ammo == 0 {
            continue;
        }
        if !admits_retained_item(result, record.id(), content, flags)? {
            continue;
        }
        if !(200..400).contains(&ammo) {
            return Err(unsupported(
                "selected ammunition outside represented reserves",
            ));
        }
        result.work.ammo = Some(definition(content, ammo)?);
        let index = (ammo - 200) as usize;
        let stock = result.reserves[index];
        if stock == 0 {
            continue;
        }
        let needed = capacity.wrapping_sub(loaded);
        if (stock as i16) < needed {
            refill_sound(result, random, flags.silent);
            result
                .warrior
                .record_mut()
                .set_inventory_quantity(slot, loaded.wrapping_add(stock as i16));
            refill_energy(result, content, slot, stock as i16, false)?;
            partial_warning(result, flags.warnings);
        } else {
            result
                .warrior
                .record_mut()
                .set_inventory_quantity(slot, loaded.wrapping_add(needed));
            result.reserves[index] = stock.wrapping_sub(i32::from(needed));
            refill_energy(result, content, slot, needed, false)?;
            refill_sound(result, random, flags.silent);
            if flags.force || flags.in_combat {
                result.outcome = AmmunitionOutcome::RefitCompleted;
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Empty-magazine selection stops at the first compatible nonzero reserve.
/// Unlike the first pass, exact stock follows the partial branch and records
/// 200 + inventory slot, not the selected reserve. Preserve refit14e5e4's writes.
fn select_ammunition_for_empty_magazine(
    result: &mut AmmunitionResult,
    inventory: &[InventoryRecord],
    content: &Content,
    flags: RefitFlags,
    random: &mut RandomState,
) -> Result<bool, RuleError> {
    for (slot, record) in inventory.iter().enumerate() {
        let loaded = result.warrior.record().inventory_quantity(slot)?;
        let capacity = record.capacity()?;
        if record.equipment_flag() == 0 || loaded > 0 || capacity <= 0 {
            continue;
        }
        if !admits_retained_item(result, record.id(), content, flags)? {
            continue;
        }
        result.work.item = Some(definition(content, record.id())?);
        let caliber = ItemDefinition(result.work.item.as_deref().unwrap()).ammunition_caliber()?;
        for index in 0..200 {
            let stock = result.reserves[index];
            if stock == 0 {
                continue;
            }
            let ammo = (index + 200) as i16;
            result.work.ammo = Some(definition(content, ammo)?);
            if ItemDefinition(result.work.ammo.as_deref().unwrap()).reserve_ammunition_kind()?
                != caliber
            {
                continue;
            }
            let needed = capacity.wrapping_sub(loaded);
            if (stock as i16) <= needed {
                refill_sound(result, random, flags.silent);
                result
                    .warrior
                    .record_mut()
                    .set_inventory_quantity(slot, loaded.wrapping_add(stock as i16));
                result
                    .warrior
                    .record_mut()
                    .set_inventory_ammunition(slot, (slot as i16).wrapping_add(200));
                refill_energy(result, content, slot, stock as i16, true)?;
                partial_warning(result, flags.warnings);
            } else {
                result
                    .warrior
                    .record_mut()
                    .set_inventory_quantity(slot, loaded.wrapping_add(needed));
                result
                    .warrior
                    .record_mut()
                    .set_inventory_ammunition(slot, ammo);
                result.reserves[index] = stock.wrapping_sub(i32::from(needed));
                refill_energy(result, content, slot, needed, true)?;
                refill_sound(result, random, flags.silent);
                if flags.force || flags.in_combat {
                    result.outcome = AmmunitionOutcome::RefitCompleted;
                }
            }
            return Ok(true);
        }
    }
    Ok(false)
}
