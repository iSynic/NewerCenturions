use super::*;

pub fn compact(warrior: &mut Warrior) -> Result<(), RuleError> {
    let count = warrior.inventory()?.len();
    let bytes = &mut warrior.bytes;
    let mut records: Vec<[u8; 22]> = bytes[0x132..0x132 + 2200].as_chunks::<22>().0.to_vec();
    for _ in 1..count {
        for index in 1..count {
            if records[index][9] != 0 && records[index - 1][9] == 0 {
                records.swap(index, index - 1);
            }
        }
    }
    for record in &mut records[count..] {
        record[..4].fill(0);
        record[9] = 0;
    }
    for (equipment, destination, first_only) in [(0xe4, 0, true), (0xca, 1, true), (0xee, 2, false)]
    {
        for index in 0..count {
            if i16::from_be_bytes(records[index][..2].try_into().unwrap())
                == word(bytes, equipment)?
                && records[index][9] != 0
            {
                records.swap(index, destination);
                if equipment == 0xe4 {
                    bytes[0x126..0x128].copy_from_slice(&records[0][4..6]);
                }
                if first_only {
                    break;
                }
            }
        }
    }
    for (index, record) in records.iter().enumerate() {
        bytes[0x132 + index * 22..0x132 + (index + 1) * 22].copy_from_slice(record);
    }
    Ok(())
}
use centurions_content::word;

/// randomitem with no forced enhancement (CODE 1 +0x2d558).
pub fn initialize_plain(
    id: i16,
    content: &Content,
    random: &mut RandomState,
) -> Result<InventoryRecord, RuleError> {
    initialize(id, 0, content, random)
}

/// randomitem for a fresh item record at difficulty zero.
pub fn initialize(
    id: i16,
    enhancement: i16,
    content: &Content,
    random: &mut RandomState,
) -> Result<InventoryRecord, RuleError> {
    initialize_at_difficulty(id, enhancement, 0, content, random)
}

/// randomitem uses the signed difficulty word at 0x442576 for draw thresholds and loop counts.
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
    if definition.len() != 150 {
        return Err(RuleError::Invalid(
            "item definition requires 150 bytes".into(),
        ));
    }
    let kind = word(definition, 0x1a)?.wrapping_abs();
    let mut record = InventoryRecord([0; 22]);
    record.0[0..2].copy_from_slice(&id.to_be_bytes());
    record.0[10] = kind as u8;
    for (target, source) in [
        (14, 0x4e),
        (15, 0x51),
        (11, 0x52),
        (16, 0x53),
        (17, 0x54),
        (18, 0x55),
        (19, 0x56),
        (21, 0x50),
        (20, 0x4f),
        (13, 0x8b),
    ] {
        record.0[target] = definition[source];
    }
    let base = word(definition, 0x14)?;
    let additional = word(definition, 0x18)?;
    let capacity = word(definition, 0x5a)?;
    let maximum = if capacity != 0 {
        capacity.wrapping_add(word(definition, 0x94)?)
    } else if base > 0 || additional != 0 {
        base.wrapping_add(additional)
    } else {
        0
    };
    let maximum = maximum.max(base);
    record.0[6..8].copy_from_slice(&maximum.to_be_bytes());
    if capacity == 0 && (base > 0 || additional != 0) {
        record.0[2..4].copy_from_slice(&base.to_be_bytes());
    }
    if maximum > 0 && [13, 14, 15].contains(&kind) {
        if random.bound(100) < 50_i16.wrapping_add(difficulty.wrapping_mul(10)) {
            let quantity = random.bound(maximum);
            record.0[2..4].copy_from_slice(&quantity.to_be_bytes());
            let ammunition_type = word(definition, 0x20)?;
            if ammunition_type != 14
                && let Some(ammo) = (200..400).find(|i| {
                    content
                        .items
                        .get(*i)
                        .is_some_and(|item| word(item, 0x22).ok() == Some(ammunition_type))
                })
            {
                record.0[4..6].copy_from_slice(&(ammo as i16).to_be_bytes());
            }
        }
    } else if base > 0 {
        let quantity = random.bound(maximum);
        record.0[2..4].copy_from_slice(&quantity.to_be_bytes());
    }
    if word(definition, 0x22)? != 0 {
        record.0[2..4].copy_from_slice(&base.to_be_bytes());
        record.0[4..8].fill(0);
    }
    let enhanced = random.bound(100) >= 94_i16.wrapping_sub(difficulty) || enhancement != 0;
    if enhanced && kind != 26 && kind != 27 && word(definition, 0x24)? == 0 {
        let mut changes = 0_i16;
        let mut upgrades = 0_i8;
        for iteration in 0..enhancement.max(1) {
            let remaining = enhancement.wrapping_sub(iteration);
            let damage = word(definition, 0x28)?;
            if damage.wrapping_add(word(definition, 0x3e)?) != 0 || [2, 15].contains(&kind) {
                let destination = if damage == 0 { 15 } else { 11 };
                for _ in 0..scale.wrapping_mul(5) {
                    if random.bound(100) < 7 {
                        record.0[destination] =
                            record.0[destination].wrapping_add(random.bound(5) as u8);
                        changes += 1;
                    }
                }
                for _ in 0..scale.wrapping_mul(4) {
                    if random.bound(100) < 5 {
                        record.0[18] =
                            record.0[18].wrapping_add(random.bound(6).wrapping_add(4) as u8);
                        changes += 1;
                    }
                }
            } else if definition[0x51] != 0 {
                for _ in 0..scale.wrapping_mul(5) {
                    if random.bound(100) < 7 {
                        record.0[15] = record.0[15].wrapping_add(random.bound(3) as u8);
                        changes += 1;
                    }
                }
            } else if definition[0x55] != 0 {
                for _ in 0..scale.wrapping_mul(5) {
                    if random.bound(100) < 5 {
                        record.0[18] =
                            record.0[18].wrapping_add(random.bound(6).wrapping_add(4) as u8);
                        changes += 1;
                    }
                }
            }
            for (offset, destination) in [(0x58, 14), (0x5e, 19)] {
                let maximum = word(definition, offset)?;
                for _ in 0..maximum.max(0).wrapping_mul(scale) {
                    if random.bound(100) < 2 {
                        record.0[destination] = record.0[destination].wrapping_add(1);
                        changes += 1;
                    }
                }
                if (record.0[destination] as i8 as i16) > maximum && maximum > 0 {
                    record.0[destination] = maximum as u8;
                }
            }
            for (offset, destination) in [(3, 16), (4, 17)] {
                let maximum = definition[offset] as i8;
                if maximum > 0 {
                    for _ in 0..scale.wrapping_mul(4) {
                        if random.bound(100) < 4 {
                            record.0[destination] =
                                record.0[destination].wrapping_add(random.bound(3) as u8);
                            changes += 1;
                        }
                    }
                    if record.0[destination] as i8 > maximum {
                        record.0[destination] = maximum as u8;
                    }
                }
            }
            let maximum = word(definition, 0x3c)?;
            if maximum > 0 {
                for _ in 0..scale.wrapping_mul(2) {
                    if random.bound(100) < 4 {
                        record.0[20] = record.0[20].wrapping_add(1);
                        changes += 1;
                    }
                }
            }
            if maximum > 0 && record.0[20] as i8 as i16 > maximum {
                record.0[20] = definition[0x4f];
            }
            if i16::from(upgrades) < remaining && changes != 0 {
                upgrades = changes as i8;
            }
            if upgrades == 0 && changes != 0 {
                upgrades = 1;
            }
            record.0[13] = definition[0x8b].wrapping_add(upgrades as u8);
        }
    }
    Ok(record)
}

pub(crate) fn add_ammunition(
    state: &mut Session,
    content: &Content,
    item: &InventoryRecord,
) -> Result<Vec<Event>, RuleError> {
    if !(200..400).contains(&item.id()) || item.0[10] != 26 {
        return Err(RuleError::Unsupported {
            action: "ammunition.collect",
            dependency: "other ammunition types",
        });
    }
    let quantity = word(&item.0, 2)?;
    if quantity < 0 {
        return Err(RuleError::Invalid("ammunition pack is exhausted".into()));
    }
    let id = item.id() as usize;
    let definition = content
        .items
        .get(id)
        .ok_or_else(|| RuleError::Invalid("ammunition definition unavailable".into()))?;
    if word(definition, 0x22)? == 0 || word(definition, 0x1a)? != 26 {
        return Err(RuleError::Invalid(
            "item is not an ammunition supply".into(),
        ));
    }
    state.ammunition[id - 200] = state.ammunition[id - 200].wrapping_add(i32::from(quantity));
    let authored = word(definition, 0xc)?;
    let sound = if authored != 0 {
        authored
    } else {
        state.random.bound(2).wrapping_add(403)
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

pub(crate) fn remove_inventory_slot(warrior: &mut Warrior, slot: usize) -> Result<(), RuleError> {
    let count = warrior.inventory()?.len();
    if slot >= count {
        return Err(RuleError::Invalid("inventory slot outside warrior".into()));
    }
    if warrior.bytes[0x132 + slot * 22 + 9] != 0 {
        return Err(RuleError::Unsupported {
            action: "inventory.remove",
            dependency: "removing equipped items",
        });
    }
    warrior.bytes.copy_within(
        0x132 + (slot + 1) * 22..0x132 + count * 22,
        0x132 + slot * 22,
    );
    warrior.bytes[0x132 + (count - 1) * 22..0x132 + count * 22].fill(0);
    warrior.bytes[0x1a..0x1c].copy_from_slice(&((count - 1) as i16).to_be_bytes());
    compact(warrior)
}

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
    if item.0[9] != 0 {
        return Err(RuleError::Invalid("ammunition pack is equipped".into()));
    }
    if !(200..400).contains(&item.id()) || item.0[10] != 26 {
        return Err(RuleError::Unsupported {
            action: "ammunition.use",
            dependency: "inventory record is not an ammunition supply",
        });
    }
    if word(&item.0, 2)? <= 0 {
        return Err(RuleError::Invalid("ammunition pack is exhausted".into()));
    }
    let effects = add_ammunition(state, content, &item)?;
    remove_inventory_slot(&mut state.warriors[warrior], slot)?;
    Ok(effects)
}
#[cfg(all(test, feature = "original-fixtures"))]
#[path = "../tests/unit/items_reference.rs"]
mod tests;
