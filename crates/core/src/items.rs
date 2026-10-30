#[cfg(all(test, feature = "original-fixtures"))]
use crate::*;
use crate::{Event, Phase, RandomState, RuleError, Session};
use centurions_content::{
    Content, ITEM_BYTES, InventoryRecord, Warrior,
    records::{EquipmentStat, InventoryModifier as Modifier, InventorySlotsMut, ItemDefinition},
};
mod ammunition_menu;
pub(crate) use ammunition_menu::choose as choose_ammunition;
pub use ammunition_menu::{AmmunitionEntry, ammunition_menu};
mod inventory_detail;
pub(crate) use inventory_detail::quick_info;
mod inventory_opening;
mod item_information;
pub(crate) use inventory_opening::{
    calcw, highlight as highlight_inventory_row, open as open_inventory,
    scroll as scroll_inventory, switch as switch_inventory_owner,
};
pub use item_information::{ShownDamage, shown_damage};
pub(crate) use item_information::{
    list_users as list_item_users, retarget as retarget_item_information,
    show as show_item_information, show_dragged as show_dragged_item_information,
    show_loot as show_loot_item_information,
};

/// Sorts loot by category, highest first. Returns the last record moved up,
/// which booty leaves behind as the inventory record.
///
/// Original: booty
pub(crate) fn sort_loot(items: &mut [InventoryRecord]) -> Option<InventoryRecord> {
    let mut temporary = None;
    for _ in 0..items.len() {
        for slot in 0..items.len().saturating_sub(1) {
            if (items[slot].category_code() as i8) < items[slot + 1].category_code() as i8 {
                temporary = Some(items[slot + 1].clone());
                items.swap(slot, slot + 1);
            }
        }
    }
    temporary
}

/// Tidies a warrior's pack: worn items move to the front with the worn firearm first, the worn
/// melee weapon second and the worn shield battery (category 20) third, and the unused slots are
/// cleared.
pub fn compact(warrior: &mut Warrior) -> Result<(), RuleError> {
    compact_inner(warrior, None)
}

/// Tidies a pack as compact does, following one selected slot to its new place.
fn compact_inner(warrior: &mut Warrior, mut selected: Option<&mut usize>) -> Result<(), RuleError> {
    let count = warrior.inventory()?.len();
    let mut records: Vec<_> = (0..100)
        .map(|slot| warrior.record().inventory_slot(slot))
        .collect();
    let mut swap = |records: &mut [InventoryRecord], first, second| {
        records.swap(first, second);
        if let Some(slot) = selected.as_deref_mut() {
            if *slot == first {
                *slot = second;
            } else if *slot == second {
                *slot = first;
            }
        }
    };
    for _ in 1..count {
        for index in 1..count {
            if records[index].equipped() != 0 && records[index - 1].equipped() == 0 {
                swap(&mut records, index, index - 1);
            }
        }
    }
    for record in &mut records[count..] {
        record.set_id(0);
        record.set_charge(0);
        record.set_equipped(0);
    }
    // Firearm first, melee second, category 20 third. The third scan deliberately
    // does not stop after a match; duplicate equipped records retain that order.
    for (category, destination, first_only) in [(15, 0, true), (2, 1, true), (20, 2, false)] {
        for index in 0..count {
            if records[index].id() == EquipmentStat::Equipment(category).read(warrior)?
                && records[index].equipped() != 0
            {
                swap(&mut records, index, destination);
                if category == 15 {
                    warrior
                        .record_mut()
                        .set_loaded_ammunition(records[0].ammunition_id()?);
                }
                if first_only {
                    break;
                }
            }
        }
    }
    for (index, record) in records.iter().enumerate() {
        warrior.record_mut().set_inventory_slot(index, record);
    }
    Ok(())
}

/// A fresh item with no forced enhancement.
///
/// Original: randomitem
pub fn initialize_plain(
    id: i16,
    content: &Content,
    random: &mut RandomState,
) -> Result<InventoryRecord, RuleError> {
    initialize(id, 0, content, random)
}

/// A fresh item at difficulty zero.
///
/// Original: randomitem
pub fn initialize(
    id: i16,
    enhancement: i16,
    content: &Content,
    random: &mut RandomState,
) -> Result<InventoryRecord, RuleError> {
    initialize_at_difficulty(id, enhancement, 0, content, random)
}

/// A fresh item; the difficulty sets the draw thresholds and loop counts.
///
/// Original: randomitem
pub fn initialize_at_difficulty(
    id: i16,
    enhancement: i16,
    difficulty: i16,
    content: &Content,
    random: &mut RandomState,
) -> Result<InventoryRecord, RuleError> {
    let scale = difficulty.wrapping_add(3);
    let definition = content
        .items
        .get(usize::from(id.unsigned_abs()))
        .ok_or_else(|| RuleError::Invalid("item id outside loaded bands".into()))?;
    if definition.len() != ITEM_BYTES {
        return Err(RuleError::Invalid(
            "item definition requires 150 bytes".into(),
        ));
    }
    let definition = ItemDefinition(definition);
    let kind = definition.item_type()?.wrapping_abs();
    let mut record = InventoryRecord([0; 22]);
    record.set_id(id);
    record.set_category(kind as u8);
    definition.install_generated_modifiers(&mut record);
    let base = definition.supplied_quantity()?;
    let additional = definition.magazine_capacity()?;
    let capacity = definition.battery_capacity()?;
    let maximum = if capacity != 0 {
        capacity.wrapping_add(definition.capacity_bonus()?)
    } else if base > 0 || additional != 0 {
        base.wrapping_add(additional)
    } else {
        0
    };
    let maximum = maximum.max(base);
    record.set_capacity(maximum);
    if capacity == 0 && (base > 0 || additional != 0) {
        record.set_charge(base);
    }
    if maximum > 0 && [13, 14, 15].contains(&kind) {
        if random.bound(100) < 50_i16.wrapping_add(difficulty.wrapping_mul(10)) {
            let quantity = random.bound(maximum);
            record.set_charge(quantity);
            let ammunition_type = definition.ammunition_caliber()?;
            if ammunition_type != 14
                && let Some(ammo) = (200..400).find(|i| {
                    content.items.get(*i).is_some_and(|item| {
                        ItemDefinition(item).reserve_ammunition_kind().ok() == Some(ammunition_type)
                    })
                })
            {
                record.set_ammunition(ammo as i16);
            }
        }
    } else if base > 0 {
        let quantity = random.bound(maximum);
        record.set_charge(quantity);
    }
    if definition.reserve_ammunition_kind()? != 0 {
        record.set_charge(base);
        record.set_ammunition(0);
        record.set_capacity(0);
    }
    let enhanced = random.bound(100) >= 94_i16.wrapping_sub(difficulty) || enhancement != 0;
    if enhanced && kind != 26 && kind != 27 && definition.charge_weight()? == 0 {
        let mut changes = 0_i16;
        let mut upgrades = 0_i8;
        for iteration in 0..enhancement.max(1) {
            let remaining = enhancement.wrapping_sub(iteration);
            let damage = definition.melee_damage()?;
            if damage.wrapping_add(definition.missile_damage()?) != 0 || [2, 15].contains(&kind) {
                let destination = if damage == 0 {
                    Modifier::Accuracy
                } else {
                    Modifier::MeleeAccuracy
                };
                for _ in 0..scale.wrapping_mul(5) {
                    if random.bound(100) < 7 {
                        destination.add(&mut record, random.bound(5) as i8);
                        changes += 1;
                    }
                }
                for _ in 0..scale.wrapping_mul(4) {
                    if random.bound(100) < 5 {
                        Modifier::Damage.add(&mut record, random.bound(6).wrapping_add(4) as i8);
                        changes += 1;
                    }
                }
            } else if definition.base_accuracy() != 0 {
                for _ in 0..scale.wrapping_mul(5) {
                    if random.bound(100) < 7 {
                        Modifier::Accuracy.add(&mut record, random.bound(3) as i8);
                        changes += 1;
                    }
                }
            } else if definition.damage_enhancement() != 0 {
                for _ in 0..scale.wrapping_mul(5) {
                    if random.bound(100) < 5 {
                        Modifier::Damage.add(&mut record, random.bound(6).wrapping_add(4) as i8);
                        changes += 1;
                    }
                }
            }
            for (maximum, destination) in [
                (definition.shield_chance()?, Modifier::ShieldChance),
                (definition.armor()?, Modifier::Armor),
            ] {
                for _ in 0..maximum.max(0).wrapping_mul(scale) {
                    if random.bound(100) < 2 {
                        destination.add(&mut record, 1);
                        changes += 1;
                    }
                }
                if i16::from(destination.read(&record)) > maximum && maximum > 0 {
                    destination.write(&mut record, maximum as i8);
                }
            }
            for (maximum, destination) in [
                (definition.missile_dodge_limit(), Modifier::MissileDodge),
                (definition.melee_dodge_limit(), Modifier::MeleeDodge),
            ] {
                if maximum > 0 {
                    for _ in 0..scale.wrapping_mul(4) {
                        if random.bound(100) < 4 {
                            destination.add(&mut record, random.bound(3) as i8);
                            changes += 1;
                        }
                    }
                    if destination.read(&record) > maximum {
                        destination.write(&mut record, maximum);
                    }
                }
            }
            let maximum = definition.range()?;
            if maximum > 0 {
                for _ in 0..scale.wrapping_mul(2) {
                    if random.bound(100) < 4 {
                        Modifier::Range.add(&mut record, 1);
                        changes += 1;
                    }
                }
            }
            if maximum > 0 && i16::from(Modifier::Range.read(&record)) > maximum {
                Modifier::Range.write(&mut record, definition.base_range_modifier());
            }
            if i16::from(upgrades) < remaining && changes != 0 {
                upgrades = changes as i8;
            }
            if upgrades == 0 && changes != 0 {
                upgrades = 1;
            }
            Modifier::RequiredLevel.write(
                &mut record,
                definition.generated_required_level().wrapping_add(upgrades),
            );
        }
    }
    Ok(record)
}

/// Adds the rounds of an ammunition item to the party's reserve for that item (item number minus
/// 200), with its sound.
pub(crate) fn add_ammunition(
    state: &mut Session,
    content: &Content,
    item: &InventoryRecord,
) -> Result<Vec<Event>, RuleError> {
    if !(200..400).contains(&item.id()) || item.category_code() != 26 {
        return Err(RuleError::Unsupported {
            action: "ammunition.collect",
            dependency: "other ammunition types",
        });
    }
    let quantity = item.charge()?;
    if quantity < 0 {
        return Err(RuleError::Invalid("ammunition pack is exhausted".into()));
    }
    let id = item.id() as usize;
    let definition = content
        .items
        .get(id)
        .ok_or_else(|| RuleError::Invalid("ammunition definition unavailable".into()))?;
    if ItemDefinition(definition).reserve_ammunition_kind()? == 0
        || ItemDefinition(definition).item_type()? != 26
    {
        return Err(RuleError::Invalid(
            "item is not an ammunition supply".into(),
        ));
    }
    state.ammunition[id - 200] = state.ammunition[id - 200].wrapping_add(i32::from(quantity));
    let authored = ItemDefinition(definition).sound()?;
    let sound = if authored != 0 {
        authored
    } else {
        state
            .random
            .bound(2)
            .wrapping_add(crate::feedback::CLIP_SOUND_BASE)
    };
    Ok(vec![
        Event::Sound { resource_id: sound },
        Event::AmmunitionAdded {
            ammunition: item.id(),
            quantity,
            reserve: state.ammunition[id - 200],
        },
    ])
}

/// Removes the item in one slot of the pack, which must not be worn, and tidies the pack.
pub(crate) fn remove_inventory_slot(warrior: &mut Warrior, slot: usize) -> Result<(), RuleError> {
    let count = warrior.inventory()?.len();
    if slot >= count {
        return Err(RuleError::Invalid("inventory slot outside warrior".into()));
    }
    if warrior.record().inventory_slot(slot).equipped() != 0 {
        return Err(RuleError::Unsupported {
            action: "inventory.remove",
            dependency: "removing equipped items",
        });
    }
    InventorySlotsMut(&mut warrior.bytes).remove_occupied_slot(slot, count);
    compact(warrior)
}

/// Converts an ammunition pack in a warrior's pack into rounds in the party's reserve, using the
/// pack up.
pub fn use_ammunition_pack(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    if state.phase != Phase::Exploration
        || state
            .exploration
            .as_ref()
            .is_none_or(|p| p.presentation.is_some())
    {
        return Err(RuleError::Invalid(
            "ammunition supply conversion requires unobstructed exploration".into(),
        ));
    }
    let recipient = state
        .warriors
        .get(warrior)
        .ok_or_else(|| RuleError::Invalid("warrior outside squad".into()))?;
    let item = recipient
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or_else(|| RuleError::Invalid("inventory slot outside warrior".into()))?;
    if item.equipped() != 0 {
        return Err(RuleError::Invalid("ammunition pack is equipped".into()));
    }
    if !(200..400).contains(&item.id()) || item.category_code() != 26 {
        return Err(RuleError::Unsupported {
            action: "ammunition.use",
            dependency: "inventory record is not an ammunition supply",
        });
    }
    if item.charge()? <= 0 {
        return Err(RuleError::Invalid("ammunition pack is exhausted".into()));
    }
    let effects = add_ammunition(state, content, &item)?;
    remove_inventory_slot(&mut state.warriors[warrior], slot)?;
    Ok(effects)
}
#[cfg(all(test, feature = "original-fixtures"))]
#[path = "../tests/unit/items_reference.rs"]
mod tests;
