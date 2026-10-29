use crate::{Event, Phase, RuleError, Session, rules};
use centurions_content::{Content, InventoryRecord, Warrior, word};

fn unavailable() -> RuleError {
    RuleError::Unsupported {
        action: "editor.item",
        dependency: "the reached presentation or arithmetic service is not part of item editing",
    }
}

/// edititems 0x134034: wrapped BYTE edits precede signed pin calls.
pub fn change_record(record: &mut InventoryRecord, control: u16) -> Result<(), RuleError> {
    let mut candidate = record.clone();
    match control {
        40..=49 => candidate.adjust_editor_modifier(usize::from(control - 40), -1),
        52..=61 => candidate.adjust_editor_modifier(usize::from(control - 52), 1),
        50 => candidate.set_quantity(0),
        62 => candidate.set_quantity(candidate.capacity()?),
        51 | 63 => {
            let capacity = candidate
                .capacity()?
                .wrapping_add(if control == 51 { -1 } else { 1 });
            candidate.set_capacity(capacity);
        }
        _ => {
            return Err(RuleError::Invalid(
                "item editor control is outside the original modifier controls".into(),
            ));
        }
    }
    for (row, minimum, maximum) in [
        (0, 1, 200),
        (1, 0, 200),
        (2, 0, 100),
        (3, 0, 100),
        (4, 0, 100),
        (5, 0, 200),
        (6, 0, 50),
        (7, 0, 50),
        (8, -10, 10),
        (9, 0, 50),
    ] {
        let value = rules::pin(i16::from(candidate.editor_modifier(row)), minimum, maximum);
        candidate.set_editor_modifier(row, value as u8);
    }
    let quantity = rules::pin(candidate.quantity()?, 0, candidate.capacity()?);
    candidate.set_quantity(quantity);
    *record = candidate;
    Ok(())
}

/// Execute the recovered record mutation silently; the command caller owns feedback and AP.
pub(crate) fn equipment_record(
    warrior: &Warrior,
    content: &Content,
    slot: usize,
    wearing: bool,
) -> Result<(Warrior, Vec<i16>, Vec<GlobalWrite>), RuleError> {
    equipment_record_inner(warrior, content, slot, wearing, false)
}

pub(crate) fn equipment_record_forced(
    warrior: &Warrior,
    content: &Content,
    slot: usize,
) -> Result<(Warrior, Vec<i16>, Vec<GlobalWrite>), RuleError> {
    equipment_record_inner(warrior, content, slot, false, true)
}

pub(crate) fn equipment_record_restored(
    warrior: &Warrior,
    content: &Content,
    slot: usize,
) -> Result<(Warrior, Vec<i16>, Vec<GlobalWrite>), RuleError> {
    equipment_record_inner(warrior, content, slot, true, true)
}

fn equipment_record_inner(
    warrior: &Warrior,
    content: &Content,
    slot: usize,
    wearing: bool,
    forced: bool,
) -> Result<(Warrior, Vec<i16>, Vec<GlobalWrite>), RuleError> {
    let item = warrior
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or("equipment slot missing")?;
    let retained = if forced {
        InventoryRecord([0; 22])
    } else {
        item
    };
    let result = retained_equipment(warrior, content, slot, wearing, forced, &retained)?;
    Ok((result.warrior, result.warnings, result.global_writes))
}

pub(crate) use crate::equipment::{GlobalWrite, RetainedEquipment};

pub(crate) fn retained_equipment(
    warrior: &Warrior,
    content: &Content,
    slot: usize,
    wearing: bool,
    forced: bool,
    retained: &InventoryRecord,
) -> Result<RetainedEquipment, RuleError> {
    crate::equipment::retained_equipment(warrior, content, slot, wearing, forced, retained)
}

pub fn edit(
    state: &mut Session,
    content: &Content,
    member: usize,
    slot: usize,
    control: u16,
) -> Result<Vec<Event>, RuleError> {
    if state.phase != Phase::Exploration {
        return Err(RuleError::Invalid(
            "item editing requires exploration".into(),
        ));
    }
    let warrior = state.warriors.get(member).ok_or_else(unavailable)?;
    if slot > warrior.inventory()?.len() || slot >= 100 {
        return Err(unavailable());
    }
    let item = warrior.record().inventory_slot(slot);
    let (next, retained, globals) = crate::equipment::edit_record(warrior, content, slot, control)?;
    // Removal then re-wear write the party-wide WORDs, including difficulty40.
    state.apply_equipment_globals(&globals)?;
    state.warriors[member] = next;
    state.native_work.inventory_record = retained;
    // loaditem copies nothing for an ID outside its five banks, keeping the work record.
    if item.id() != 0
        && let Some(definition) = content.items.get(usize::from(item.id().unsigned_abs()))
    {
        state.native_work.loaded_item_field32 = word(definition, 32)?;
    }
    Ok(vec![Event::Sound { resource_id: 141 }])
}
