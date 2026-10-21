pub mod payment;
use crate::{Event, RandomState, RuleError, combat, items};
use centurions_content::{
    Content, InventoryRecord, Warrior,
    mission::long,
    records::{InventorySlotsMut, ItemDefinition},
    span, word,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartyItem {
    pub warrior: usize,
    pub slot: usize,
}

pub fn query_party_item(warriors: &[Warrior], id: i16) -> Result<Option<PartyItem>, RuleError> {
    for (warrior, record) in warriors.iter().enumerate() {
        for (slot, item) in record.inventory()?.iter().enumerate() {
            if item.id() == id {
                return Ok(Some(PartyItem { warrior, slot }));
            }
        }
    }
    Ok(None)
}

pub fn item_prerequisite_matches(warriors: &[Warrior], extra: &[u8]) -> Result<bool, RuleError> {
    let extra = span(extra, 0, 10)?;
    let found = query_party_item(warriors, word(extra, 0)?)?.is_some();
    Ok(match word(extra, 2)? {
        0 => !found,
        1 => found,
        2 => true,
        _ => false,
    })
}

#[derive(Debug, PartialEq, Eq)]
pub struct Consumption {
    pub warriors: Vec<Warrior>,
    pub consumed: usize,
    pub events: Vec<Event>,
    pub record: Option<InventoryRecord>,
    pub definition: Option<Vec<u8>>,
}

pub fn consume_party_items(
    warriors: &[Warrior],
    content: &Content,
    race_movements: &[i8],
    extra: &[u8],
) -> Result<Consumption, RuleError> {
    consume_party_items_retained(warriors, content, race_movements, extra, None)
}

pub fn consume_party_items_retained(
    warriors: &[Warrior],
    content: &Content,
    race_movements: &[i8],
    extra: &[u8],
    retained: Option<&InventoryRecord>,
) -> Result<Consumption, RuleError> {
    let extra = span(extra, 0, 10)?;
    let id = word(extra, 0)?;
    let maximum = word(extra, 2)?;
    let mode = word(extra, 4)?;
    for warrior in warriors {
        span(&warrior.bytes, 0, 3138)?;
    }
    let mut result = Consumption {
        warriors: warriors.to_vec(),
        consumed: 0,
        events: Vec::new(),
        record: retained.cloned(),
        definition: None,
    };
    if maximum <= 0 {
        return Ok(result);
    }
    for warrior in &mut result.warriors {
        let mut slot = 0;
        while slot < warrior.inventory()?.len() && result.consumed < maximum as usize {
            let item = warrior.inventory()?[slot].clone();
            if item.id() != id {
                slot += 1;
                continue;
            }
            if mode == 3 {
                result.definition = Some(crate::ammunition::definition(content, word(extra, 8)?)?);
                if item.equipment_flag() != 0 {
                    let record = result.record.as_ref().ok_or(RuleError::Unsupported {
                        action: "story.consume_item",
                        dependency: "equipped replacement retained inventory context",
                    })?;
                    let removed = crate::editor_items::retained_equipment(
                        warrior, content, slot, false, false, record,
                    )?;
                    *warrior = removed.warrior;
                    result.record = Some(removed.record);
                    result.definition = Some(removed.definition);
                    result
                        .events
                        .extend(removed.native_warnings.into_iter().map(|index| {
                            Event::Feedback(crate::feedback::NativeFeedback::Warning { index })
                        }));
                }
                let replacement = word(extra, 8)?;
                let definition = result.definition.as_ref().unwrap();
                let mut replacement_item = warrior.record().inventory_slot(slot);
                replacement_item.0[..2].copy_from_slice(&replacement.to_be_bytes());
                replacement_item.set_quantity(ItemDefinition(definition).supplied_quantity()?);
                warrior
                    .record_mut()
                    .set_inventory_slot(slot, &replacement_item);
                if item.equipment_flag() != 0 {
                    let worn = crate::editor_items::retained_equipment(
                        warrior,
                        content,
                        slot,
                        true,
                        false,
                        result.record.as_ref().unwrap(),
                    )?;
                    *warrior = worn.warrior;
                    result.record = Some(worn.record);
                    result.definition = Some(worn.definition);
                    result
                        .events
                        .extend(worn.native_warnings.into_iter().map(|index| {
                            Event::Feedback(crate::feedback::NativeFeedback::Warning { index })
                        }));
                }
                result.consumed += 1;
                slot += 1;
                continue;
            } else if mode == 2 {
                let quantity = warrior
                    .record()
                    .inventory_quantity(slot)?
                    .wrapping_add(word(extra, 6)?);
                warrior.record_mut().set_inventory_quantity(slot, quantity);
                result.definition = Some(crate::ammunition::definition(content, id)?);
                let definition = content
                    .items
                    .get(id.unsigned_abs() as usize)
                    .ok_or("altered item definition unavailable")?;
                if quantity > 0 || ItemDefinition(definition).discarded_when_empty()? == 0 {
                    result.consumed += 1;
                    slot += 1;
                    continue;
                }
            } else if mode != 1 {
                return Err(RuleError::Unsupported {
                    action: "story.consume_item",
                    dependency: "item replacement operation",
                });
            }
            let definition = content
                .items
                .get(id.unsigned_abs() as usize)
                .ok_or_else(|| {
                    RuleError::Invalid("consumed item definition is unavailable".into())
                })?;
            result.definition = Some(crate::ammunition::definition(content, id)?);
            if item.equipment_flag() != 0 {
                if warrior.record().inventory_quantity(slot)? == -1 {
                    let class = ItemDefinition(definition).category()?.wrapping_abs();
                    let offset =
                        0xc6 + usize::try_from(class).map_err(|_| "invalid removed class")? * 2;
                    warrior
                        .bytes
                        .get_mut(offset..offset + 2)
                        .ok_or("removed class outside warrior")?
                        .fill(0);
                    warrior.record_mut().set_inventory_equipment_flag(slot, 0);
                } else if crate::equipment::removal_blocked(warrior, slot, definition)? {
                    result.events.extend([
                        Event::Sound { resource_id: 655 },
                        Event::Feedback(crate::feedback::NativeFeedback::Warning { index: 145 }),
                    ]);
                    result.consumed += 1;
                    if mode == 2 {
                        slot += 1;
                    }
                    continue;
                } else {
                    let record = result.record.as_ref().ok_or(RuleError::Unsupported {
                        action: "story.consume_item",
                        dependency: "equipped removal retained inventory context",
                    })?;
                    let removed = crate::editor_items::retained_equipment(
                        warrior, content, slot, false, false, record,
                    )?;
                    *warrior = removed.warrior;
                    result.record = Some(removed.record);
                    result.definition = Some(removed.definition);
                    if removed.native_return == 0 {
                        return Err("equipped story removal unexpectedly refused".into());
                    }
                }
            }
            let count = warrior.inventory()?.len();
            let weight = ItemDefinition(definition).base_weight()?.wrapping_add(
                ItemDefinition(definition)
                    .unit_weight()?
                    .wrapping_mul(warrior.record().inventory_quantity(slot)?),
            );
            let carried = warrior.record().carried_weight()?.wrapping_sub(weight);
            warrior.record_mut().set_carried_weight(carried);
            InventorySlotsMut(&mut warrior.bytes).remove_dropped_slot(slot, (count - 1) as i16);
            let race = warrior.record().race()?.wrapping_sub(1);
            let movement = race_movements.get(race as usize).ok_or_else(|| {
                RuleError::Invalid("consumed item race movement is unavailable".into())
            })?;
            combat::calculate_movement(warrior, *movement)?;
            result.consumed += 1;
            result.events.push(Event::Sound { resource_id: 655 });
            // newland14666e advances after depletion; mode1 retries the compacted slot.
            if mode == 2 {
                slot += 1;
            }
        }
    }
    Ok(result)
}

pub fn normalize_story_loot(warriors: &[Warrior]) -> Result<Vec<Warrior>, RuleError> {
    let mut result = warriors.to_vec();
    for warrior in &mut result {
        span(&warrior.bytes, 0, 3138)?;
        normalize_warrior_after_battle(warrior)?;
    }
    Ok(result)
}

/// booty and retreat share this cleanup, but decide active status separately.
pub(crate) fn normalize_warrior_after_battle(warrior: &mut Warrior) -> Result<(), RuleError> {
    use centurions_content::records::Condition;
    warrior.record_mut().set_faction(0);
    warrior.record_mut().set_bleeding_flag(0);
    warrior.record_mut().set_turn_skip_duration(0);
    for condition in [
        Condition::Retreating,
        Condition::Helpless,
        Condition::Entangled,
        Condition::Poisoned,
    ] {
        if warrior.record().condition(condition)? > 0 {
            warrior.record_mut().set_condition(condition, 0);
        }
    }
    if warrior.record().automatic_mode()? > 0 {
        warrior.record_mut().set_automatic_mode(0);
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExperienceAward {
    pub warriors: Vec<Warrior>,
    pub share: i32,
}

pub fn award_story_experience(
    warriors: &[Warrior],
    amount: i32,
) -> Result<ExperienceAward, RuleError> {
    let mut result = normalize_story_loot(warriors)?;
    for warrior in &mut result {
        warrior.record_mut().set_active_flag(1);
    }
    let eligible = result
        .iter()
        .filter(|warrior| {
            warrior.stamina().unwrap_or(0) > 0
                && warrior.record().automatic_mode().unwrap_or(-1) >= 0
        })
        .count() as i32;
    let rounded = amount as f32;
    if rounded as f64 >= i32::MAX as f64 || (rounded as f64) < i32::MIN as f64 {
        return Err(RuleError::Unsupported {
            action: "story.experience",
            dependency: "SANE reward conversion overflow",
        });
    }
    let share = if eligible == 0 {
        0
    } else {
        (rounded as i32 / eligible).wrapping_abs()
    };
    for warrior in &mut result {
        if warrior.stamina()? <= 0 || warrior.record().automatic_mode()? < 0 {
            continue;
        }
        let experience =
            long(&warrior.bytes, 0x9d2)?
                .checked_add(share)
                .ok_or(RuleError::Unsupported {
                    action: "story.experience",
                    dependency: "SANE experience overflow",
                })?;
        let experience = if experience < 0 {
            i32::from(((experience >> 16) as i16).wrapping_abs())
        } else {
            experience
        };
        let remaining = warrior
            .record()
            .remaining_experience()?
            .checked_sub(share)
            .ok_or(RuleError::Unsupported {
                action: "story.experience",
                dependency: "SANE experience balance overflow",
            })?;
        warrior.record_mut().set_upgrade_points(experience);
        warrior.record_mut().set_remaining_experience(remaining);
    }
    Ok(ExperienceAward {
        warriors: result,
        share,
    })
}

pub fn grant_story_treasure(
    treasure: &[u8],
    content: &Content,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<Vec<InventoryRecord>, RuleError> {
    let treasure = span(treasure, 0, 88)?;
    if treasure[80..88].iter().any(|value| *value != 0) {
        return Err(RuleError::Unsupported {
            action: "story.treasure",
            dependency: "treasure credit and experience awards",
        });
    }
    let mut result = Vec::new();
    let mut candidate_random = random.clone();
    for slot in 0..20 {
        let id = word(treasure, slot * 2)?;
        if id > 0 {
            let enhancement = word(treasure, 40 + slot * 2)?;
            let enhancement = if enhancement == 0 {
                0
            } else {
                enhancement.wrapping_add(difficulty).max(1)
            };
            result.push(items::initialize_at_difficulty(
                id,
                enhancement,
                difficulty,
                content,
                &mut candidate_random,
            )?);
        }
    }
    *random = candidate_random;
    Ok(result)
}
