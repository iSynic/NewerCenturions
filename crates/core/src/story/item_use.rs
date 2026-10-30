use crate::editor_items::{Fit, Force};
use crate::{RuleError, combat};
use centurions_content::records::{InventorySlots, InventorySlotsMut, ItemDefinition};
use centurions_content::{Content, InventoryRecord, WARRIOR_BYTES, Warrior, span};

/// What using an item named by a scenario did: the warriors afterward, the item's script return and
/// script, and the sounds and warnings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenarioItemUse {
    pub warriors: Vec<Warrior>,
    pub return_code: i16,
    pub script: Option<i16>,
    pub sounds: Vec<i16>,
    pub warnings: Vec<i16>,
    /// Party-wide writes from removing a worn discarded item.
    pub(crate) global_writes: Vec<crate::equipment::GlobalWrite>,
}

/// Where a scenario item is used: while exploring or in an encounter.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ScriptItemContext {
    Exploration,
    Encounter,
}

/// Refuses an item that does nothing when used: one that is not a scenario item (class 23) and
/// whose spell is not above 1100. Classes 24 and 25 with a spell below 1100 also give warnings 88
/// and 89.
///
/// Original: Items
pub(crate) fn reject_inert_item(definition: &[u8]) -> Result<(), RuleError> {
    let definition = ItemDefinition(definition);
    let class = definition.item_type()?.wrapping_abs();
    let effect = definition.spell()?;
    if class != 23 && effect <= 1100 {
        return Err(RuleError::Refused {
            feedback: if matches!(class, 24 | 25) && effect < 1100 {
                vec![
                    crate::feedback::GameFeedback::Warning { index: 88 },
                    crate::feedback::GameFeedback::Warning { index: 89 },
                ]
            } else {
                vec![]
            },
        });
    }
    Ok(())
}

/// Uses a scenario item (class 23): a charge is spent, the item is removed when its last charge
/// goes and it is set to be discarded, and the story script the item names is returned.
///
/// Original: Items, buttonchoice
pub fn use_scenario_item(
    warriors: &[Warrior],
    member: usize,
    slot: usize,
    content: &Content,
    race_movements: &[i8],
    context: ScriptItemContext,
    inventory_record: Option<&InventoryRecord>,
) -> Result<ScenarioItemUse, RuleError> {
    let warrior = warriors
        .get(member)
        .ok_or_else(|| RuleError::Invalid("selected item warrior unavailable".into()))?;
    let bytes = span(&warrior.bytes, 0, WARRIOR_BYTES)?;
    let count = usize::try_from(InventorySlots(bytes).count()?)
        .map_err(|_| RuleError::Invalid("selected inventory count is negative".into()))?;
    if slot >= count || count > 100 {
        return Err(RuleError::Invalid(
            "selected inventory item unavailable".into(),
        ));
    }
    let selected_item = InventorySlots(bytes).slot(slot)?;
    let id = selected_item.id();
    let definition = content
        .items
        .get(id.unsigned_abs() as usize)
        .ok_or_else(|| RuleError::Invalid("selected item definition unavailable".into()))?;
    reject_inert_item(definition)?;
    let definition = ItemDefinition(definition);
    if definition.item_type()?.wrapping_abs() != 23 {
        return Err(RuleError::Unsupported {
            action: "item.use",
            dependency: "non-script item use",
        });
    }
    let quantity = selected_item.charge()?;
    let quantity = if quantity > 0 { quantity - 1 } else { quantity };
    let remove = quantity == 0 && definition.discarded_when_empty()? != 0;
    let return_code = definition.story_script()?.wrapping_add(100).wrapping_neg();
    let mut result = ScenarioItemUse {
        warriors: warriors.to_vec(),
        return_code,
        script: (return_code <= -100).then(|| return_code.wrapping_abs().wrapping_sub(100)),
        sounds: vec![],
        warnings: if context == ScriptItemContext::Exploration && definition.spell()? < 1100 {
            vec![88, 89]
        } else {
            vec![]
        },
        global_writes: vec![],
    };
    result.warriors[member]
        .record_mut()
        .set_inventory_quantity(slot, quantity);
    let carried = result.warriors[member]
        .record()
        .load()?
        .wrapping_sub(definition.charge_weight()?);
    result.warriors[member].record_mut().set_load(carried);
    if remove {
        result.sounds.push(655);
        if selected_item.equipped() != 0 {
            // dropitem removes a worn item's attributes before shifting the slots.
            let selected = warrior.inventory()?[slot].clone();
            let removed = crate::editor_items::silent_fit(
                &result.warriors[member],
                content,
                slot,
                Fit::Remove,
                Force::Checked,
                inventory_record.unwrap_or(&selected),
            )?;
            result.warriors[member] = removed.warrior;
            result.warnings.extend(removed.warnings);
            result.global_writes = removed.global_writes;
            if removed.return_code as u16 == 0 {
                return Ok(result);
            }
        }
        let weight = definition
            .weight()?
            .wrapping_add(definition.charge_weight()?.wrapping_mul(quantity));
        let warrior = &mut result.warriors[member];
        let carried = warrior.record().load()?.wrapping_sub(weight);
        warrior.record_mut().set_load(carried);
        InventorySlotsMut(&mut warrior.bytes).remove_dropped_slot(slot, (count - 1) as i16);
        let race = warrior.record().race()?.wrapping_sub(1);
        let movement = race_movements
            .get(race as usize)
            .ok_or_else(|| RuleError::Invalid("selected item race movement unavailable".into()))?;
        combat::calculate_movement(&mut result.warriors[member], *movement)?;
    }
    Ok(result)
}
