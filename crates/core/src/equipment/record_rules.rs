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
    let mut result = Collection {
        warrior: warrior.clone(),
        item: item.clone(),
        collected: false,
        sounds: vec![ItemDefinition(definition).pickup_sound()?],
        warnings: vec![],
    };
    let count = warrior.inventory()?.len();
    let stacked = if item.id() == 700 {
        warrior
            .inventory()?
            .iter()
            .position(|record| record.id() == 700)
    } else {
        None
    };
    if let Some(slot) = stacked {
        // Native stacking writes quantity and weight before the rejection gates.
        let quantity = item.quantity()?;
        let total = result
            .warrior
            .record()
            .inventory_quantity(slot)?
            .wrapping_add(quantity);
        result
            .warrior
            .record_mut()
            .set_inventory_quantity(slot, total);
        let weight = result
            .warrior
            .record()
            .carried_weight()?
            .wrapping_add(quantity);
        result.warrior.record_mut().set_carried_weight(weight);
    }
    if count >= 99 {
        if earned_unloaded_firearm(item) {
            result.sounds.push(6000);
        }
        result.warnings.push(147);
        return Ok(result);
    }
    let needed = ItemDefinition(definition)
        .supplied_quantity()?
        .wrapping_mul(ItemDefinition(definition).unit_weight()?)
        .wrapping_add(result.warrior.record().carried_weight()?)
        .wrapping_add(ItemDefinition(definition).base_weight()?) as u16;
    if needed > warrior.record().weight_capacity()? as u16 {
        if earned_unloaded_firearm(item) {
            result.sounds.push(6000);
        }
        result.warnings.push(19);
        return Ok(result);
    }
    if stacked.is_some() {
        combat::calculate_movement(&mut result.warrior, race_movement(warrior, mission)?)?;
        result.item.clear_id();
        result.collected = true;
        return Ok(result);
    }
    InventorySlotsMut(&mut result.warrior.bytes).append_collected(
        count,
        item,
        ItemDefinition(definition),
    )?;
    calculate_weight(&mut result.warrior, content)?;
    combat::calculate_movement(&mut result.warrior, race_movement(warrior, mission)?)?;
    result.item.clear_id();
    result.collected = true;
    Ok(result)
}

pub(crate) fn remove_inner(
    warrior: &mut Warrior,
    slot: usize,
    content: &Content,
    mission: &OpeningMission,
) -> Result<(), RuleError> {
    let shortcut = warrior.record().inventory_quantity(slot)? == -1;
    *warrior = crate::editor_items::equipment_record(warrior, content, slot, false)?.0;
    if !shortcut {
        combat::calculate_movement(warrior, race_movement(warrior, mission)?)?;
    }
    Ok(())
}

/// dropitem 0x121152 retains the original 29-slot shift and empty-record fill.
pub(crate) fn drop(
    warrior: &mut Warrior,
    slot: usize,
    content: &Content,
    mission: &OpeningMission,
) -> Result<(), RuleError> {
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
) -> Result<(), RuleError> {
    let item = InventorySlots(&warrior.bytes).slot(slot)?;
    drop_record(warrior, slot, &item, content, mission, true)
}

pub(super) fn drop_record(
    warrior: &mut Warrior,
    slot: usize,
    item: &InventoryRecord,
    content: &Content,
    mission: &OpeningMission,
    forced: bool,
) -> Result<(), RuleError> {
    if item.equipment_flag() != 0 {
        if forced {
            *warrior = crate::editor_items::equipment_record_forced(warrior, content, slot)?.0;
        } else {
            remove_inner(warrior, slot, content, mission)?;
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
    combat::calculate_movement(warrior, race_movement(warrior, mission)?)
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
        });
    }
    let definition = item_definition(content, item.id())?;
    let mut result = EquipmentResult {
        warrior: warrior.clone(),
        changed: false,
        sounds: vec![],
        warnings: vec![],
        record: retained.cloned(),
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
        });
    }
    let mut result = EquipmentResult {
        warrior: warrior.clone(),
        changed: true,
        sounds: vec![ItemDefinition(item_definition(content, item.id())?).pickup_sound()?],
        warnings: vec![],
        record: retained.cloned(),
    };
    if let Some(record) = retained {
        let removed =
            crate::editor_items::retained_equipment(warrior, content, slot, false, false, record)?;
        result.warrior = removed.warrior;
        result.record = Some(removed.record);
    } else {
        remove_inner(&mut result.warrior, slot, content, mission)?;
    }
    Ok(result)
}
