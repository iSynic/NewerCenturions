pub mod payment;
use crate::editor_items::{Fit, Force};
use crate::{Event, RandomState, RuleError, combat, items};
use centurions_content::{
    Content, InventoryRecord, WARRIOR_BYTES, Warrior,
    records::{
        CharacterInformation, EquipmentStat, InventorySlotsMut, ItemConsumptionExtra,
        ItemDefinition, ItemPrerequisiteExtra, TreasureRecord,
    },
    span,
};

/// Where an item was found in the party: the warrior and slot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartyItem {
    pub warrior: usize,
    pub slot: usize,
}

/// Finds the first warrior, in party order, who carries an item.
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

/// Whether the party's carrying of an item meets an item prerequisite: required absent, required
/// present, or either.
pub fn item_prerequisite_matches(warriors: &[Warrior], extra: &[u8]) -> Result<bool, RuleError> {
    let extra = ItemPrerequisiteExtra::new(extra)?;
    let found = query_party_item(warriors, extra.item()?)?.is_some();
    Ok(match extra.mode()? {
        0 => !found,
        1 => found,
        2 => true,
        _ => false,
    })
}

/// What taking items from the party did: the warriors afterward, how many items were taken, the
/// events and any replacement item.
#[derive(Debug, PartialEq, Eq)]
pub struct Consumption {
    pub warriors: Vec<Warrior>,
    pub consumed: usize,
    pub events: Vec<Event>,
    pub record: Option<InventoryRecord>,
    pub definition: Option<Vec<u8>>,
    /// Party-wide writes from equipped removal and replacement wear.
    pub(crate) global_writes: Vec<crate::equipment::GlobalWrite>,
}

/// Takes items from the party's packs as an item-consumption extra asks.
pub fn consume_party_items(
    warriors: &[Warrior],
    content: &Content,
    race_movements: &[i8],
    extra: &[u8],
) -> Result<Consumption, RuleError> {
    consume_party_items_with_record(warriors, content, race_movements, extra, None)
}

/// Works through the party's packs in party order on up to the extra's maximum of an item: mode 1
/// removes it, mode 2 changes its quantity (removing it when none are left and it is discarded when
/// empty) and mode 3 replaces it with another item. A worn item is taken off first, and a worn
/// replacement is put on.
pub fn consume_party_items_with_record(
    warriors: &[Warrior],
    content: &Content,
    race_movements: &[i8],
    extra: &[u8],
    inventory_record: Option<&InventoryRecord>,
) -> Result<Consumption, RuleError> {
    let extra = ItemConsumptionExtra::new(extra)?;
    let id = extra.item()?;
    let maximum = extra.maximum()?;
    let mode = extra.mode()?;
    for warrior in warriors {
        span(&warrior.bytes, 0, WARRIOR_BYTES)?;
    }
    let mut result = Consumption {
        warriors: warriors.to_vec(),
        consumed: 0,
        events: Vec::new(),
        record: inventory_record.cloned(),
        definition: None,
        global_writes: vec![],
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
                result.definition = Some(crate::ammunition::definition(
                    content,
                    extra.replacement()?,
                )?);
                if item.equipped() != 0 {
                    let record = result
                        .record
                        .as_ref()
                        .ok_or("equipped replacement without a retained inventory record")?;
                    let removed = crate::editor_items::silent_fit(
                        warrior,
                        content,
                        slot,
                        Fit::Remove,
                        Force::Checked,
                        record,
                    )?;
                    *warrior = removed.warrior;
                    result.record = Some(removed.record);
                    result.definition = Some(removed.definition);
                    result.global_writes.extend(removed.global_writes);
                    result
                        .events
                        .extend(removed.raised_warnings.into_iter().map(|index| {
                            Event::Feedback(crate::feedback::GameFeedback::Warning { index })
                        }));
                }
                let replacement = extra.replacement()?;
                let definition = result.definition.as_ref().unwrap();
                let mut replacement_item = warrior.record().inventory_slot(slot);
                replacement_item.set_id(replacement);
                replacement_item.set_charge(ItemDefinition(definition).supplied_quantity()?);
                warrior
                    .record_mut()
                    .set_inventory_slot(slot, &replacement_item);
                if item.equipped() != 0 {
                    let worn = crate::editor_items::silent_fit(
                        warrior,
                        content,
                        slot,
                        Fit::Wear,
                        Force::Checked,
                        result.record.as_ref().unwrap(),
                    )?;
                    *warrior = worn.warrior;
                    result.record = Some(worn.record);
                    result.definition = Some(worn.definition);
                    result.global_writes.extend(worn.global_writes);
                    result
                        .events
                        .extend(worn.raised_warnings.into_iter().map(|index| {
                            Event::Feedback(crate::feedback::GameFeedback::Warning { index })
                        }));
                }
                result.consumed += 1;
                slot += 1;
                continue;
            } else if mode == 2 {
                let quantity = warrior
                    .record()
                    .inventory_quantity(slot)?
                    .wrapping_add(extra.quantity_change()?);
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
            if item.equipped() != 0 {
                if warrior.record().inventory_quantity(slot)? == -1 {
                    let class = ItemDefinition(definition).item_type()?.wrapping_abs();
                    // i16::MIN survives wrapping_abs and keeps its own error.
                    usize::try_from(class).map_err(|_| "invalid removed class")?;
                    EquipmentStat::Equipment(class)
                        .write(warrior, 0)
                        .map_err(|_| "removed class outside warrior")?;
                    warrior.record_mut().set_inventory_equipment_flag(slot, 0);
                } else if crate::equipment::removal_blocked(warrior, slot, definition)? {
                    result.events.extend([
                        Event::Sound { resource_id: 655 },
                        Event::Feedback(crate::feedback::GameFeedback::Warning { index: 145 }),
                    ]);
                    result.consumed += 1;
                    if mode == 2 {
                        slot += 1;
                    }
                    continue;
                } else {
                    let record = result
                        .record
                        .as_ref()
                        .ok_or("equipped removal without a retained inventory record")?;
                    let removed = crate::editor_items::silent_fit(
                        warrior,
                        content,
                        slot,
                        Fit::Remove,
                        Force::Checked,
                        record,
                    )?;
                    *warrior = removed.warrior;
                    result.record = Some(removed.record);
                    result.definition = Some(removed.definition);
                    result.global_writes.extend(removed.global_writes);
                    if removed.return_code == 0 {
                        return Err("equipped story removal unexpectedly refused".into());
                    }
                }
            }
            let count = warrior.inventory()?.len();
            let weight = ItemDefinition(definition).weight()?.wrapping_add(
                ItemDefinition(definition)
                    .charge_weight()?
                    .wrapping_mul(warrior.record().inventory_quantity(slot)?),
            );
            let carried = warrior.record().load()?.wrapping_sub(weight);
            warrior.record_mut().set_load(carried);
            InventorySlotsMut(&mut warrior.bytes).remove_dropped_slot(slot, (count - 1) as i16);
            let race = warrior.record().race()?.wrapping_sub(1);
            let movement = race_movements.get(race as usize).ok_or_else(|| {
                RuleError::Invalid("consumed item race movement is unavailable".into())
            })?;
            combat::calculate_movement(warrior, *movement)?;
            result.consumed += 1;
            result.events.push(Event::Sound { resource_id: 655 });
            // After a stack runs out, mode 2 moves on and mode 1 retries the
            // compacted slot.
            if mode == 2 {
                slot += 1;
            }
        }
    }
    Ok(result)
}

/// Clears every warrior's battle state: faction, bleeding, skipped turns, the retreating, helpless,
/// entangled and poisoned conditions, and positive computer control.
pub fn normalize_story_loot(warriors: &[Warrior]) -> Result<Vec<Warrior>, RuleError> {
    let mut result = warriors.to_vec();
    for warrior in &mut result {
        span(&warrior.bytes, 0, WARRIOR_BYTES)?;
        normalize_warrior_after_battle(warrior)?;
    }
    Ok(result)
}

/// booty and retreat share this cleanup, but decide active status separately.
pub(crate) fn normalize_warrior_after_battle(warrior: &mut Warrior) -> Result<(), RuleError> {
    use centurions_content::records::Condition;
    warrior.record_mut().set_faction(0);
    warrior.record_mut().set_bleeding(0);
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
    if warrior.record().computer_control()? > 0 {
        warrior.record_mut().set_computer_control(0);
    }
    Ok(())
}

/// Victory points given by a story: the warriors afterward and each one's share.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VictoryPointsAward {
    pub warriors: Vec<Warrior>,
    pub share: i32,
}

/// Gives victory points to the warriors with stamina above 0 and no negative computer control, dividing
/// it evenly among them; battle state is cleared first.
pub fn award_story_victory_points(
    warriors: &[Warrior],
    amount: i32,
) -> Result<VictoryPointsAward, RuleError> {
    let mut result = normalize_story_loot(warriors)?;
    for warrior in &mut result {
        warrior.record_mut().set_in_battle(1);
    }
    let eligible = result
        .iter()
        .filter(|warrior| {
            warrior.stamina().unwrap_or(0) > 0
                && warrior.record().computer_control().unwrap_or(-1) >= 0
        })
        .count() as i32;
    let rounded = crate::rules::sane_long(f64::from(amount as f32));
    let share = if eligible == 0 {
        0
    } else {
        (rounded / eligible).wrapping_abs()
    };
    for warrior in &mut result {
        if warrior.stamina()? <= 0 || warrior.record().computer_control()? < 0 {
            continue;
        }
        // The sum is exact; only storing it as a 32-bit value saturates.
        let victory_points = crate::rules::sane_long(
            f64::from(CharacterInformation(&warrior.bytes).upgrade_points()?) + f64::from(share),
        );
        let victory_points = if victory_points < 0 {
            i32::from(((victory_points >> 16) as i16).wrapping_abs())
        } else {
            victory_points
        };
        let remaining = crate::rules::sane_long(
            f64::from(warrior.record().remaining_victory_points()?) - f64::from(share),
        );
        warrior.record_mut().set_upgrade_points(victory_points);
        warrior.record_mut().set_remaining_victory_points(remaining);
    }
    Ok(VictoryPointsAward {
        warriors: result,
        share,
    })
}

/// Generates the items of a treasure record a story hands out, enhanced for the difficulty. A
/// record that also awards credits or victory points is not supported here.
pub fn grant_story_treasure(
    treasure: &[u8],
    content: &Content,
    random: &mut RandomState,
    difficulty: i16,
) -> Result<Vec<InventoryRecord>, RuleError> {
    let treasure = TreasureRecord::new(treasure)?;
    if treasure.has_awards() {
        return Err(RuleError::Unsupported {
            action: "story.treasure",
            dependency: "treasure credit and experience awards",
        });
    }
    let mut result = Vec::new();
    let mut candidate_random = random.clone();
    for slot in 0..TreasureRecord::SLOTS {
        let id = treasure.item(slot)?;
        if id > 0 {
            let enhancement = treasure.enhancement(slot)?;
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
