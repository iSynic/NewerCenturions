//! Wearing, removing and dropping items in a warrior's record.
use super::silent::{Fit, Force};
use crate::story::collection::Collection;
use crate::{RuleError, combat};
use centurions_content::ITEM_BYTES;
use centurions_content::mission::OpeningMission;
use centurions_content::span;
use centurions_content::{
    Content, InventoryRecord, Warrior,
    records::{
        EquipmentEligibility, InventorySlots, InventorySlotsMut, ItemDefinition, MosEquipment,
        ProfileTable, RaceEquipment,
    },
};

/// What wearing, removing or dropping an item changed: the warrior, the sounds and warnings that go
/// with it, and the party-wide values it wrote.
pub struct EquipmentResult {
    pub warrior: Warrior,
    pub changed: bool,
    pub sounds: Vec<i16>,
    pub warnings: Vec<i16>,
    pub record: Option<InventoryRecord>,
    /// Ordered party-wide writes for the Session caller to apply.
    pub(crate) global_writes: Vec<super::GlobalWrite>,
}

/// The refusal given when an item rule needs a service the reconstruction does not carry.
pub(super) fn unsupported(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "equipment",
        dependency,
    }
}

/// The definition of an item from the item table; the id's sign is ignored.
pub(super) fn item_definition(content: &Content, id: i16) -> Result<&[u8], RuleError> {
    content
        .items
        .get(usize::from(id.unsigned_abs()))
        .map(Vec::as_slice)
        .ok_or_else(|| RuleError::Invalid("equipment definition missing".into()))
}

/// Whether an item cannot be removed. The battery check comes after the
/// charge-minus-one shortcut.
///
/// Original: removeitem
pub(crate) fn removal_blocked(
    warrior: &Warrior,
    slot: usize,
    definition: &[u8],
) -> Result<bool, RuleError> {
    Ok(ItemDefinition(definition).item_type()?.wrapping_abs() == 20
        && warrior.record().inventory_quantity(slot)? != -1
        && InventorySlots(&warrior.bytes).battery_has_dependents()?)
}

/// Totals what the warrior carries: each item's weight plus its charge times the weight per charge.
pub(crate) fn calculate_weight(warrior: &mut Warrior, content: &Content) -> Result<(), RuleError> {
    let mut weight = 0_i16;
    for record in warrior.inventory()? {
        let definition = item_definition(content, record.id())?;
        weight = weight
            .wrapping_add(
                ItemDefinition(definition)
                    .charge_weight()?
                    .wrapping_mul(record.charge()?),
            )
            .wrapping_add(ItemDefinition(definition).weight()?);
    }
    warrior.record_mut().set_load(weight);
    Ok(())
}

/// Whether the warrior's race, MOS, race traits and psi school allow the item to be used.
pub fn can_use(
    warrior: &Warrior,
    definition: &[u8],
    mission: &OpeningMission,
) -> Result<bool, RuleError> {
    span(definition, 0, ITEM_BYTES)?;
    let race_id = warrior.record().race()?;
    let mos_id = warrior.record().mos()?;
    if race_id <= 0 || mos_id <= 0 {
        return Err(unsupported("unset Race or MOS profile"));
    }
    let race = RaceEquipment(ProfileTable::races(&mission.races).row(race_id as usize - 1)?);
    let mos = MosEquipment(ProfileTable::mos_profiles(&mission.profiles).row(mos_id as usize - 1)?);
    let eligibility = EquipmentEligibility(definition);
    if (eligibility.required_race()? != 0 && eligibility.required_race()? != race_id)
        || (eligibility.required_mos()? != 0 && eligibility.required_mos()? != mos_id)
    {
        return Ok(false);
    }
    let mut ability = 0;
    while ability < 58 && !eligibility.ability(ability)? {
        ability += 1;
    }
    if !mos.permits_ability(ability)? || !race.permits_ability(ability)? {
        return Ok(false);
    }
    for index in 0..9 {
        let race_trait = race.has_trait(index)?;
        if (eligibility.forbids_trait(index)? && race_trait)
            || (eligibility.requires_trait(index)? && !race_trait)
        {
            return Ok(false);
        }
    }
    let school = mos.psi_school()?;
    if (0..7).contains(&school.wrapping_sub(1)) {
        let index = (school - 1) as usize;
        if eligibility.forbids_school(index)?
            || (eligibility.has_required_schools()?
                && !eligibility.permits_required_school(index)?)
        {
            return Ok(false);
        }
    } else if eligibility.has_required_schools()? {
        return Ok(false);
    }
    Ok(true)
}

/// The movement allowance of the warrior's race.
pub(super) fn race_movement(warrior: &Warrior, mission: &OpeningMission) -> Result<i8, RuleError> {
    let race = warrior.record().race()?;
    mission
        .combat
        .as_ref()
        .and_then(|tables| tables.race_moves.get(race.wrapping_sub(1) as usize))
        .copied()
        .ok_or_else(|| RuleError::Invalid("equipment movement profile missing".into()))
}

/// Whether the item is the unloaded firearm a starting warrior earns.
pub(super) fn earned_unloaded_firearm(item: &InventoryRecord) -> bool {
    item.0
        == [
            0, 165, 0, 0, 0, 0, 0, 1, 0, 0, 15, 0, 0, 4, 0, 6, 0, 0, 0, 0, 0, 0,
        ]
}

/// Puts a found item into a warrior's pack, returning the sound and warnings that go with it;
/// movement is recalculated when the item was taken.
pub fn collect_item(
    warrior: &Warrior,
    item: &InventoryRecord,
    content: &Content,
    mission: &OpeningMission,
) -> Result<Collection, RuleError> {
    let definition = item_definition(content, item.id())?;
    let mut result = crate::story::collection::collect_inventory_item(
        warrior,
        item,
        content,
        ItemDefinition(definition).sound()?,
        earned_unloaded_firearm(item),
    )?;
    if result.collected {
        combat::calculate_movement(&mut result.warrior, race_movement(warrior, mission)?)?;
    }
    Ok(result)
}

/// Takes the item in one slot off the warrior, undoing its effects, and returns the party-wide
/// values it wrote. Movement is recalculated unless the item has a charge of -1.
pub(crate) fn remove_inner(
    warrior: &mut Warrior,
    slot: usize,
    content: &Content,
    mission: &OpeningMission,
) -> Result<Vec<super::GlobalWrite>, RuleError> {
    let shortcut = warrior.record().inventory_quantity(slot)? == -1;
    let (removed, _, globals) =
        crate::editor_items::equipment_record(warrior, content, slot, Fit::Remove)?;
    *warrior = removed;
    if !shortcut {
        combat::calculate_movement(warrior, race_movement(warrior, mission)?)?;
    }
    Ok(globals)
}

/// The drop-all-equipment script operation empties a warrior's inventory. Each
/// slot in turn is removed, worn or not, then slot 0 is dropped with slot 1's
/// item loaded for the weight; a worn slot 0 reloads its own definition when
/// removed. Each drop plays sound 655, and a blocked removal warns 145 and
/// keeps the item. Returns the member and the last item loaded, if any.
///
/// Original: newland, removeitem, dropitem
pub(crate) fn drop_all(
    warrior: &Warrior,
    content: &Content,
    mission: &OpeningMission,
    events: &mut Vec<crate::Event>,
) -> Result<(Warrior, Option<i16>, Vec<super::GlobalWrite>), RuleError> {
    let mut warrior = warrior.clone();
    let mut loaded = None;
    let mut globals = vec![];
    let mut remove = |warrior: &mut Warrior, slot: usize, events: &mut Vec<crate::Event>| {
        let record = warrior.record().inventory_slot(slot);
        let removed = crate::editor_items::silent_fit(
            warrior,
            content,
            slot,
            Fit::Remove,
            Force::Checked,
            &record,
        )?;
        *warrior = removed.warrior;
        globals.extend(removed.global_writes);
        events.extend(removed.warnings.iter().map(|&index| {
            crate::Event::Feedback(crate::feedback::GameFeedback::Warning { index })
        }));
        Ok::<_, RuleError>((record.id(), removed.warnings.is_empty()))
    };
    for index in 0.. {
        let count = InventorySlots(&warrior.bytes).count()?;
        if count == 0 {
            break;
        }
        if index >= 100 {
            return Err(unsupported("removal past the inventory"));
        }
        remove(&mut warrior, index, events)?;
        let mut item = InventorySlots(&warrior.bytes).item_id(1)?;
        events.push(crate::Event::Sound { resource_id: 655 });
        if warrior.record().inventory_slot(0).equipped() != 0 {
            let (worn, removed) = remove(&mut warrior, 0, events)?;
            item = worn;
            if !removed {
                loaded = Some(item);
                continue;
            }
        }
        loaded = Some(item);
        let definition = ItemDefinition(item_definition(content, item)?);
        let quantity = warrior.record().inventory_slot(0).charge()?;
        let weight = definition
            .charge_weight()?
            .wrapping_mul(quantity)
            .wrapping_add(definition.weight()?);
        let total = warrior.record().load()?.wrapping_sub(weight);
        InventorySlotsMut(&mut warrior.bytes).remove_dropped_slot(0, count.wrapping_sub(1));
        warrior.record_mut().set_load(total);
        let movement = race_movement(&warrior, mission)?;
        combat::calculate_movement(&mut warrior, movement)?;
    }
    Ok((warrior, loaded, globals))
}

/// Drops an item: the 29 slots after it shift up and the last is blanked.
///
/// Original: dropitem
pub(crate) fn drop(
    warrior: &mut Warrior,
    slot: usize,
    content: &Content,
    mission: &OpeningMission,
) -> Result<Vec<super::GlobalWrite>, RuleError> {
    let item = warrior
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or("drop slot missing")?;
    drop_record(warrior, slot, &item, content, mission, false)
}

/// Drops the item in a slot, taking it off first without the usual refusal when it is worn.
pub(crate) fn force_drop(
    warrior: &mut Warrior,
    slot: usize,
    content: &Content,
    mission: &OpeningMission,
) -> Result<Vec<super::GlobalWrite>, RuleError> {
    let item = InventorySlots(&warrior.bytes).slot(slot)?;
    drop_record(warrior, slot, &item, content, mission, true)
}

/// Drops an item weighed by the definition its caller loaded. A worn slot's
/// removal reloads its own definition, and a blocked removal warns and keeps
/// the item. Returns the definition left loaded.
///
/// Original: dropitem
pub(crate) fn drop_loaded(
    warrior: &mut Warrior,
    slot: usize,
    loaded: i16,
    content: &Content,
    mission: &OpeningMission,
    events: &mut Vec<crate::Event>,
) -> Result<(i16, Vec<super::GlobalWrite>), RuleError> {
    let record = InventorySlots(&warrior.bytes).slot(slot)?;
    let mut globals = vec![];
    let mut weighed = loaded;
    if record.equipped() != 0 {
        let removed = crate::editor_items::silent_fit(
            warrior,
            content,
            slot,
            Fit::Remove,
            Force::Checked,
            &record,
        )?;
        *warrior = removed.warrior;
        globals = removed.global_writes;
        weighed = record.id();
        if !removed.warnings.is_empty() {
            events.extend(removed.warnings.iter().map(|&index| {
                crate::Event::Feedback(crate::feedback::GameFeedback::Warning { index })
            }));
            return Ok((weighed, globals));
        }
    }
    let mut item = InventorySlots(&warrior.bytes).slot(slot)?;
    item.0[..2].copy_from_slice(&weighed.to_be_bytes());
    globals.extend(drop_record(warrior, slot, &item, content, mission, false)?);
    Ok((weighed, globals))
}

/// Drops the item in a slot; a worn item is taken off first, forced or checked.
pub(super) fn drop_record(
    warrior: &mut Warrior,
    slot: usize,
    item: &InventoryRecord,
    content: &Content,
    mission: &OpeningMission,
    forced: bool,
) -> Result<Vec<super::GlobalWrite>, RuleError> {
    let mut globals = vec![];
    if item.equipped() != 0 {
        if forced {
            let (removed, _, writes) =
                crate::editor_items::equipment_record_forced(warrior, content, slot)?;
            *warrior = removed;
            globals = writes;
        } else {
            globals = remove_inner(warrior, slot, content, mission)?;
        }
    }
    let definition = item_definition(content, item.id())?;
    let weight = ItemDefinition(definition)
        .charge_weight()?
        .wrapping_mul(item.charge()?)
        .wrapping_add(ItemDefinition(definition).weight()?);
    let remaining = InventorySlots(&warrior.bytes).count()?.wrapping_sub(1);
    let total = warrior.record().load()?.wrapping_sub(weight);
    InventorySlotsMut(&mut warrior.bytes).remove_dropped_slot(slot, remaining);
    warrior.record_mut().set_load(total);
    combat::calculate_movement(warrior, race_movement(warrior, mission)?)?;
    Ok(globals)
}

/// Wears the item in a slot, with the sounds and warnings the player would get. A worn item
/// changes nothing; in a battle a successful wear costs an action point.
pub fn wear(
    warrior: &Warrior,
    content: &Content,
    mission: &OpeningMission,
    slot: usize,
    in_combat: bool,
) -> Result<EquipmentResult, RuleError> {
    wear_silently(warrior, content, mission, slot, in_combat, None)
}

/// Wears the item in a slot, using the caller's retained item record in place of the slot's own
/// when given. Returns the warrior, sounds, warnings and party-wide writes.
pub(super) fn wear_silently(
    warrior: &Warrior,
    content: &Content,
    mission: &OpeningMission,
    slot: usize,
    in_combat: bool,
    inventory_record: Option<&InventoryRecord>,
) -> Result<EquipmentResult, RuleError> {
    let item = warrior
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or_else(|| RuleError::Invalid("equipment slot missing".into()))?;
    if item.equipped() != 0 {
        return Ok(EquipmentResult {
            warrior: warrior.clone(),
            changed: false,
            sounds: vec![],
            warnings: vec![],
            record: inventory_record.cloned(),
            global_writes: vec![],
        });
    }
    let definition = item_definition(content, item.id())?;
    let mut result = EquipmentResult {
        warrior: warrior.clone(),
        changed: false,
        sounds: vec![],
        warnings: vec![],
        record: inventory_record.cloned(),
        global_writes: vec![],
    };
    if !can_use(warrior, definition, mission)? {
        result.warnings.push(29);
        result.sounds.push(6000);
        return Ok(result);
    }
    if warrior.record().level()? < i16::from(item.required_level()) {
        result.sounds.push(6000);
        return Ok(result);
    }
    let worn = crate::editor_items::silent_fit(
        warrior,
        content,
        slot,
        Fit::Wear,
        Force::Checked,
        inventory_record.unwrap_or(&item),
    )?;
    result.warrior = worn.warrior;
    result.warnings = worn.warnings;
    result.global_writes = worn.global_writes;
    result.record = inventory_record.map(|_| worn.record);
    if InventorySlots(&result.warrior.bytes).equipped(slot) == 0 {
        result.changed = result.warrior != *warrior;
        return Ok(result);
    }
    if in_combat {
        let ap = result.warrior.record().action_points()?.wrapping_sub(1);
        result.warrior.record_mut().set_action_points(ap);
    }
    result.changed = true;
    result.sounds.push(ItemDefinition(definition).sound()?);
    Ok(result)
}

/// Takes the item in a slot off, with the sounds and warnings the player would get. A shield
/// battery stays on, with warning 145, while a power shield is worn.
pub fn remove(
    warrior: &Warrior,
    content: &Content,
    mission: &OpeningMission,
    slot: usize,
) -> Result<EquipmentResult, RuleError> {
    remove_silently(warrior, content, mission, slot, None)
}

/// Takes the item in a slot off, using the caller's retained item record in place of the slot's
/// own when given. Returns the warrior, sounds, warnings and party-wide writes.
pub(super) fn remove_silently(
    warrior: &Warrior,
    content: &Content,
    mission: &OpeningMission,
    slot: usize,
    inventory_record: Option<&InventoryRecord>,
) -> Result<EquipmentResult, RuleError> {
    let item = warrior
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or_else(|| RuleError::Invalid("equipment slot missing".into()))?;
    if item.equipped() == 0 {
        return Err(unsupported("unequipped selection"));
    }
    if removal_blocked(warrior, slot, item_definition(content, item.id())?)? {
        return Ok(EquipmentResult {
            warrior: warrior.clone(),
            changed: false,
            sounds: vec![],
            warnings: vec![145],
            record: inventory_record.cloned(),
            global_writes: vec![],
        });
    }
    let mut result = EquipmentResult {
        warrior: warrior.clone(),
        changed: true,
        sounds: vec![ItemDefinition(item_definition(content, item.id())?).sound()?],
        warnings: vec![],
        record: inventory_record.cloned(),
        global_writes: vec![],
    };
    if let Some(record) = inventory_record {
        let removed = crate::editor_items::silent_fit(
            warrior,
            content,
            slot,
            Fit::Remove,
            Force::Checked,
            record,
        )?;
        result.warrior = removed.warrior;
        result.record = Some(removed.record);
        result.global_writes = removed.global_writes;
    } else {
        result.global_writes = remove_inner(&mut result.warrior, slot, content, mission)?;
    }
    Ok(result)
}
