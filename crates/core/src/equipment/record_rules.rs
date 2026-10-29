//! Record rules for equipment.
use crate::story_collection::Collection;
use crate::{RuleError, combat};
use centurions_content::mission::OpeningMission;
use centurions_content::span;
use centurions_content::{
    Content, InventoryRecord, Warrior,
    records::{
        EquipmentEligibility, InventorySlots, InventorySlotsMut, ItemDefinition,
        ProfessionEquipment, RaceEquipment,
    },
};

pub struct EquipmentResult {
    pub warrior: Warrior,
    pub changed: bool,
    pub sounds: Vec<i16>,
    pub warnings: Vec<i16>,
    pub record: Option<InventoryRecord>,
    /// Ordered party-wide writes for the Session caller to apply.
    pub(crate) global_writes: Vec<super::GlobalWrite>,
}

pub(super) fn unsupported(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "equipment",
        dependency,
    }
}

pub(super) fn item_definition(content: &Content, id: i16) -> Result<&[u8], RuleError> {
    content
        .items
        .get(usize::from(id.unsigned_abs()))
        .map(Vec::as_slice)
        .ok_or_else(|| RuleError::Invalid("equipment definition missing".into()))
}

/// removeitem checks the battery dependency after its charge-minus-one shortcut.
pub(crate) fn removal_blocked(
    warrior: &Warrior,
    slot: usize,
    definition: &[u8],
) -> Result<bool, RuleError> {
    Ok(ItemDefinition(definition).category()?.wrapping_abs() == 20
        && warrior.record().inventory_quantity(slot)? != -1
        && InventorySlots(&warrior.bytes).battery_has_dependents()?)
}

pub(crate) fn calculate_weight(warrior: &mut Warrior, content: &Content) -> Result<(), RuleError> {
    let mut weight = 0_i16;
    for record in warrior.inventory()? {
        let definition = item_definition(content, record.id())?;
        weight = weight
            .wrapping_add(
                ItemDefinition(definition)
                    .unit_weight()?
                    .wrapping_mul(record.quantity()?),
            )
            .wrapping_add(ItemDefinition(definition).base_weight()?);
    }
    warrior.record_mut().set_carried_weight(weight);
    Ok(())
}

pub fn can_use(
    warrior: &Warrior,
    definition: &[u8],
    mission: &OpeningMission,
) -> Result<bool, RuleError> {
    span(definition, 0, 150)?;
    let race_id = warrior.record().race()?;
    let mos_id = warrior.record().profession()?;
    if race_id <= 0 || mos_id <= 0 {
        return Err(unsupported("unset Race or MOS profile"));
    }
    let race = RaceEquipment(span(&mission.races, (race_id as usize - 1) * 408, 408)?);
    let mos = ProfessionEquipment(span(&mission.profiles, (mos_id as usize - 1) * 576, 576)?);
    let eligibility = EquipmentEligibility(definition);
    if (eligibility.required_race()? != 0 && eligibility.required_race()? != race_id)
        || (eligibility.required_profession()? != 0 && eligibility.required_profession()? != mos_id)
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

pub(super) fn race_movement(warrior: &Warrior, mission: &OpeningMission) -> Result<i8, RuleError> {
    let race = warrior.record().race()?;
    mission
        .combat
        .as_ref()
        .and_then(|tables| tables.race_moves.get(race.wrapping_sub(1) as usize))
        .copied()
        .ok_or_else(|| RuleError::Invalid("equipment movement profile missing".into()))
}

pub(super) fn earned_unloaded_firearm(item: &InventoryRecord) -> bool {
    item.0
        == [
            0, 165, 0, 0, 0, 0, 0, 1, 0, 0, 15, 0, 0, 4, 0, 6, 0, 0, 0, 0, 0, 0,
        ]
}

pub fn collect_item(
    warrior: &Warrior,
    item: &InventoryRecord,
    content: &Content,
    mission: &OpeningMission,
) -> Result<Collection, RuleError> {
    let definition = item_definition(content, item.id())?;
    let mut result = crate::story_collection::collect_inventory_item(
        warrior,
        item,
        content,
        ItemDefinition(definition).pickup_sound()?,
        earned_unloaded_firearm(item),
    )?;
    if result.collected {
        combat::calculate_movement(&mut result.warrior, race_movement(warrior, mission)?)?;
    }
    Ok(result)
}

pub(crate) fn remove_inner(
    warrior: &mut Warrior,
    slot: usize,
    content: &Content,
    mission: &OpeningMission,
) -> Result<Vec<super::GlobalWrite>, RuleError> {
    let shortcut = warrior.record().inventory_quantity(slot)? == -1;
    let (removed, _, globals) =
        crate::editor_items::equipment_record(warrior, content, slot, false)?;
    *warrior = removed;
    if !shortcut {
        combat::calculate_movement(warrior, race_movement(warrior, mission)?)?;
    }
    Ok(globals)
}

/// newland91 0x141d0e empties an inventory through removeitem on a rising slot
/// index, worn or not, then dropitem on slot 0 with slot 1's item loaded for the
/// weight; a worn slot 0 reloads its own definition through its removal. Each
/// dropitem plays sound 655, and a blocked removal warns 145 and keeps the item.
/// Returns the member and the last item loaded, if any.
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
        let removed =
            crate::editor_items::retained_equipment(warrior, content, slot, false, false, &record)?;
        *warrior = removed.warrior;
        globals.extend(removed.global_writes);
        events.extend(removed.warnings.iter().map(|&index| {
            crate::Event::Feedback(crate::feedback::NativeFeedback::Warning { index })
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
        if warrior.record().inventory_slot(0).equipment_flag() != 0 {
            let (worn, removed) = remove(&mut warrior, 0, events)?;
            item = worn;
            if !removed {
                loaded = Some(item);
                continue;
            }
        }
        loaded = Some(item);
        let definition = ItemDefinition(item_definition(content, item)?);
        let quantity = warrior.record().inventory_slot(0).quantity()?;
        let weight = definition
            .unit_weight()?
            .wrapping_mul(quantity)
            .wrapping_add(definition.base_weight()?);
        let total = warrior.record().carried_weight()?.wrapping_sub(weight);
        InventorySlotsMut(&mut warrior.bytes).remove_dropped_slot(0, count.wrapping_sub(1));
        warrior.record_mut().set_carried_weight(total);
        let movement = race_movement(&warrior, mission)?;
        combat::calculate_movement(&mut warrior, movement)?;
    }
    Ok((warrior, loaded, globals))
}

/// dropitem 0x121152 retains the original 29-slot shift and empty-record fill.
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

pub(crate) fn force_drop(
    warrior: &mut Warrior,
    slot: usize,
    content: &Content,
    mission: &OpeningMission,
) -> Result<Vec<super::GlobalWrite>, RuleError> {
    let item = InventorySlots(&warrior.bytes).slot(slot)?;
    drop_record(warrior, slot, &item, content, mission, true)
}

/// dropitem 0x121152 after its caller's loaditem: an unworn slot is weighed by
/// that loaded definition. A worn slot's removal reloads its own definition, and
/// a blocked removal warns and keeps the item. Returns the definition left loaded.
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
    if record.equipment_flag() != 0 {
        let removed =
            crate::editor_items::retained_equipment(warrior, content, slot, false, false, &record)?;
        *warrior = removed.warrior;
        globals = removed.global_writes;
        weighed = record.id();
        if !removed.warnings.is_empty() {
            events.extend(removed.warnings.iter().map(|&index| {
                crate::Event::Feedback(crate::feedback::NativeFeedback::Warning { index })
            }));
            return Ok((weighed, globals));
        }
    }
    let mut item = InventorySlots(&warrior.bytes).slot(slot)?;
    item.0[..2].copy_from_slice(&weighed.to_be_bytes());
    globals.extend(drop_record(warrior, slot, &item, content, mission, false)?);
    Ok((weighed, globals))
}

pub(super) fn drop_record(
    warrior: &mut Warrior,
    slot: usize,
    item: &InventoryRecord,
    content: &Content,
    mission: &OpeningMission,
    forced: bool,
) -> Result<Vec<super::GlobalWrite>, RuleError> {
    let mut globals = vec![];
    if item.equipment_flag() != 0 {
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
        .unit_weight()?
        .wrapping_mul(item.quantity()?)
        .wrapping_add(ItemDefinition(definition).base_weight()?);
    let remaining = InventorySlots(&warrior.bytes).count()?.wrapping_sub(1);
    let total = warrior.record().carried_weight()?.wrapping_sub(weight);
    InventorySlotsMut(&mut warrior.bytes).remove_dropped_slot(slot, remaining);
    warrior.record_mut().set_carried_weight(total);
    combat::calculate_movement(warrior, race_movement(warrior, mission)?)?;
    Ok(globals)
}

pub fn wear(
    warrior: &Warrior,
    content: &Content,
    mission: &OpeningMission,
    slot: usize,
    in_combat: bool,
) -> Result<EquipmentResult, RuleError> {
    wear_retained(warrior, content, mission, slot, in_combat, None)
}

pub(super) fn wear_retained(
    warrior: &Warrior,
    content: &Content,
    mission: &OpeningMission,
    slot: usize,
    in_combat: bool,
    retained: Option<&InventoryRecord>,
) -> Result<EquipmentResult, RuleError> {
    let item = warrior
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or_else(|| RuleError::Invalid("equipment slot missing".into()))?;
    if item.equipment_flag() != 0 {
        return Ok(EquipmentResult {
            warrior: warrior.clone(),
            changed: false,
            sounds: vec![],
            warnings: vec![],
            record: retained.cloned(),
            global_writes: vec![],
        });
    }
    let definition = item_definition(content, item.id())?;
    let mut result = EquipmentResult {
        warrior: warrior.clone(),
        changed: false,
        sounds: vec![],
        warnings: vec![],
        record: retained.cloned(),
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
    let worn = crate::editor_items::retained_equipment(
        warrior,
        content,
        slot,
        true,
        false,
        retained.unwrap_or(&item),
    )?;
    result.warrior = worn.warrior;
    result.warnings = worn.warnings;
    result.global_writes = worn.global_writes;
    result.record = retained.map(|_| worn.record);
    if InventorySlots(&result.warrior.bytes).equipment_flag(slot) == 0 {
        result.changed = result.warrior != *warrior;
        return Ok(result);
    }
    if in_combat {
        let ap = result.warrior.record().action_points()?.wrapping_sub(1);
        result.warrior.record_mut().set_action_points(ap);
    }
    result.changed = true;
    result
        .sounds
        .push(ItemDefinition(definition).pickup_sound()?);
    Ok(result)
}

pub fn remove(
    warrior: &Warrior,
    content: &Content,
    mission: &OpeningMission,
    slot: usize,
) -> Result<EquipmentResult, RuleError> {
    remove_retained(warrior, content, mission, slot, None)
}

pub(super) fn remove_retained(
    warrior: &Warrior,
    content: &Content,
    mission: &OpeningMission,
    slot: usize,
    retained: Option<&InventoryRecord>,
) -> Result<EquipmentResult, RuleError> {
    let item = warrior
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or_else(|| RuleError::Invalid("equipment slot missing".into()))?;
    if item.equipment_flag() == 0 {
        return Err(unsupported("unequipped selection"));
    }
    if removal_blocked(warrior, slot, item_definition(content, item.id())?)? {
        return Ok(EquipmentResult {
            warrior: warrior.clone(),
            changed: false,
            sounds: vec![],
            warnings: vec![145],
            record: retained.cloned(),
            global_writes: vec![],
        });
    }
    let mut result = EquipmentResult {
        warrior: warrior.clone(),
        changed: true,
        sounds: vec![ItemDefinition(item_definition(content, item.id())?).pickup_sound()?],
        warnings: vec![],
        record: retained.cloned(),
        global_writes: vec![],
    };
    if let Some(record) = retained {
        let removed =
            crate::editor_items::retained_equipment(warrior, content, slot, false, false, record)?;
        result.warrior = removed.warrior;
        result.record = Some(removed.record);
        result.global_writes = removed.global_writes;
    } else {
        result.global_writes = remove_inner(&mut result.warrior, slot, content, mission)?;
    }
    Ok(result)
}
