use super::{
    AmmunitionOutcome, AmmunitionResult, LoadedItems, RefitFlags, definition, initial, unsupported,
};
use crate::editor_items::{Fit, Force};
use crate::feedback::{CLIP_SOUND_BASE, ERROR_SOUND};
use crate::{RandomState, RuleError};
use centurions_content::records::{ItemDefinition, item_class};
use centurions_content::{Content, InventoryRecord, Warrior};

/// Cargo Pod `STR#` 3 "Warnings", entry 49: the refit ran out of ammunition.
const REFIT_INCOMPLETE_WARNING: i16 = 49;

/// Plays one of the two reload sounds unless the refit is silent.
fn refill_sound(result: &mut AmmunitionResult, random: &mut RandomState, silent: bool) {
    if !silent {
        result.sounds.push(CLIP_SOUND_BASE + random.bound(2));
    }
}

/// Warns that the refit could not be completed, with the error sound.
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

/// Refills a shield battery slot. A battery that still holds charge is taken off (when asked) and
/// worn again, and its sound plays if it ends up worn; an empty one adds the energy straight to
/// the warrior's shield points. A slot that is not a shield battery is left alone.
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
            refit_equipment(result, content, slot, Fit::Remove)?;
        }
        result.warrior.record_mut().clear_equipped_battery();
        result
            .warrior
            .record_mut()
            .set_inventory_equipment_flag(slot, 0);
        refit_equipment(result, content, slot, Fit::Wear)?;
        if result.warrior.record().inventory_slot(slot).equipped() != 0 {
            let id = result.warrior.record().inventory_slot(slot).id();
            result
                .sounds
                .push(ItemDefinition(&definition(content, id)?).sound()?);
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

/// Takes a battery off or puts it back on to match a refill.
fn refit_equipment(
    result: &mut AmmunitionResult,
    content: &Content,
    slot: usize,
    fit: Fit,
) -> Result<(), RuleError> {
    let inventory_record = result
        .inventory_record
        .as_ref()
        .ok_or("battery re-wear without a retained inventory record")?;
    let equipped = crate::editor_items::silent_fit(
        &result.warrior,
        content,
        slot,
        fit,
        Force::Checked,
        inventory_record,
    )?;
    result.warrior = equipped.warrior;
    result.inventory_record = Some(equipped.record);
    result.global_writes.extend(equipped.global_writes);
    result.loaded.item = Some(equipped.definition);
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
            crate::Event::Feedback(crate::feedback::GameFeedback::EquipmentLevel { level }),
        ));
    }
    Ok(())
}

/// Whether a weapon may be refilled now: when forced or in a battle, it must be a melee or missile
/// weapon.
fn admits_loaded_item(
    result: &mut AmmunitionResult,
    id: i16,
    content: &Content,
    flags: RefitFlags,
) -> Result<bool, RuleError> {
    if flags.force || flags.in_combat {
        result.loaded.ammo = Some(definition(content, id)?);
        return Ok(matches!(
            ItemDefinition(
                result
                    .loaded
                    .item
                    .as_deref()
                    .ok_or_else(|| RuleError::from("ITEM work missing"))?
            )
            .item_type()?,
            item_class::MELEE_WEAPON | item_class::MISSILE_WEAPON
        ));
    }
    Ok(true)
}

/// Refits the warrior's weapons and batteries from the party's reserves: worn weapons that hold
/// ammunition are topped up first, and if that does not complete the refit, one worn empty weapon
/// is given ammunition. Used where no session carries the warrior's battery record.
pub fn refit(
    warrior: &Warrior,
    reserves: &[i32],
    content: &Content,
    flags: RefitFlags,
    loaded: &LoadedItems,
    random: &mut RandomState,
) -> Result<AmmunitionResult, RuleError> {
    // Without a session, the warrior's battery row stands in for the inventory
    // record; session callers pass the real one through refit_with_record.
    let inventory_record = warrior
        .inventory()?
        .into_iter()
        .find(|row| i16::from(row.category_code()) == item_class::SHIELD_BATTERY);
    refit_with_record(
        warrior,
        reserves,
        content,
        flags,
        loaded,
        random,
        inventory_record.as_ref(),
    )
}

/// Refits as refit does, with the session's battery record.
pub(crate) fn refit_with_record(
    warrior: &Warrior,
    reserves: &[i32],
    content: &Content,
    flags: RefitFlags,
    loaded: &LoadedItems,
    random: &mut RandomState,
    inventory_record: Option<&InventoryRecord>,
) -> Result<AmmunitionResult, RuleError> {
    let inventory = warrior.inventory()?;
    let mut result = initial(warrior, reserves, loaded)?;
    result.inventory_record = inventory_record.cloned();
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

/// Refills from the loaded ammunition before trying another supply. A partial
/// refill does not take from stock; exact stock completes this first pass.
///
/// Original: refit
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
        if record.equipped() == 0 || loaded >= capacity || ammo == 0 {
            continue;
        }
        if !admits_loaded_item(result, record.id(), content, flags)? {
            continue;
        }
        if !(200..400).contains(&ammo) {
            return Err(unsupported(
                "selected ammunition outside represented reserves",
            ));
        }
        result.loaded.ammo = Some(definition(content, ammo)?);
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

/// An empty magazine takes the first compatible reserve with rounds. Unlike the
/// first pass, exact stock follows the partial branch and records 200 +
/// inventory slot, not the chosen reserve; the game does this, so keep it.
///
/// Original: refit
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
        if record.equipped() == 0 || loaded > 0 || capacity <= 0 {
            continue;
        }
        if !admits_loaded_item(result, record.id(), content, flags)? {
            continue;
        }
        result.loaded.item = Some(definition(content, record.id())?);
        let caliber =
            ItemDefinition(result.loaded.item.as_deref().unwrap()).ammunition_caliber()?;
        for index in 0..200 {
            let stock = result.reserves[index];
            if stock == 0 {
                continue;
            }
            let ammo = (index + 200) as i16;
            result.loaded.ammo = Some(definition(content, ammo)?);
            if ItemDefinition(result.loaded.ammo.as_deref().unwrap()).reserve_ammunition_kind()?
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
