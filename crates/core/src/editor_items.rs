use crate::{Event, Phase, RuleError, Session, rules};
use centurions_content::{Content, InventoryRecord, Warrior, word};

/// The refusal given when an item edit reaches something outside item editing.
fn unavailable() -> RuleError {
    RuleError::Unsupported {
        action: "editor.item",
        dependency: "the reached presentation or arithmetic service is not part of item editing",
    }
}

/// One editor control's change to an item: byte edits wrap before the signed
/// limits are applied.
///
/// Original: edititems
pub fn change_record(record: &mut InventoryRecord, control: u16) -> Result<(), RuleError> {
    let mut candidate = record.clone();
    match control {
        40..=49 => candidate.adjust_editor_modifier(usize::from(control - 40), -1),
        52..=61 => candidate.adjust_editor_modifier(usize::from(control - 52), 1),
        50 => candidate.set_charge(0),
        62 => candidate.set_charge(candidate.capacity()?),
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
    let quantity = rules::pin(candidate.charge()?, 0, candidate.capacity()?);
    candidate.set_charge(quantity);
    *record = candidate;
    Ok(())
}

/// Changes the record silently; the command reports feedback and spends the action.
pub(crate) fn equipment_record(
    warrior: &Warrior,
    content: &Content,
    slot: usize,
    fit: Fit,
) -> Result<(Warrior, Vec<i16>, Vec<GlobalWrite>), RuleError> {
    equipment_record_inner(warrior, content, slot, fit, Force::Checked)
}

/// Takes an item off a warrior as the editor does: on a blank item record, and without the refusal
/// that keeps a shield battery on while a power shield (class 13 or 14) is worn. Returns the
/// warrior, the warnings raised and the party-wide values written.
pub(crate) fn equipment_record_forced(
    warrior: &Warrior,
    content: &Content,
    slot: usize,
) -> Result<(Warrior, Vec<i16>, Vec<GlobalWrite>), RuleError> {
    equipment_record_inner(warrior, content, slot, Fit::Remove, Force::Forced)
}

/// Puts an edited item back on a warrior as the editor does, on a blank item record; the wearing
/// checks still apply. Returns the warrior, the warnings raised and the party-wide values written.
pub(crate) fn equipment_record_restored(
    warrior: &Warrior,
    content: &Content,
    slot: usize,
) -> Result<(Warrior, Vec<i16>, Vec<GlobalWrite>), RuleError> {
    equipment_record_inner(warrior, content, slot, Fit::Wear, Force::Forced)
}

/// Removes or wears the item in one slot. A forced change works on a blank item record in place of
/// the slot's own.
fn equipment_record_inner(
    warrior: &Warrior,
    content: &Content,
    slot: usize,
    fit: Fit,
    force: Force,
) -> Result<(Warrior, Vec<i16>, Vec<GlobalWrite>), RuleError> {
    let item = warrior
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or("equipment slot missing")?;
    let inventory_record = if force == Force::Forced {
        InventoryRecord([0; 22])
    } else {
        item
    };
    let result = silent_fit(warrior, content, slot, fit, force, &inventory_record)?;
    Ok((result.warrior, result.warnings, result.global_writes))
}

pub(crate) use crate::equipment::{Fit, Force, GlobalWrite, SilentFit};

/// Removes or wears one slot's item with no messages, reporting the warnings it would have raised
/// and the party-wide values it wrote.
pub(crate) fn silent_fit(
    warrior: &Warrior,
    content: &Content,
    slot: usize,
    fit: Fit,
    force: Force,
    inventory_record: &InventoryRecord,
) -> Result<SilentFit, RuleError> {
    crate::equipment::silent_fit(warrior, content, slot, fit, force, inventory_record)
}

/// Applies one item-editor control to the item in a carried slot. A worn item is taken off and
/// worn again around the change so the warrior's totals follow it.
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
    let (next, inventory_record, globals) =
        crate::equipment::edit_record(warrior, content, slot, control)?;
    // Removal then re-wear write the party-wide values, including difficulty.
    state.apply_equipment_globals(&globals)?;
    state.warriors[member] = next;
    state.globals.inventory_record = inventory_record;
    // An ID outside the five item banks leaves the loaded item unchanged.
    if item.id() != 0
        && let Some(definition) = content.items.get(usize::from(item.id().unsigned_abs()))
    {
        state.globals.loaded_item_field32 = word(definition, 32)?;
    }
    Ok(vec![Event::Sound { resource_id: 141 }])
}
