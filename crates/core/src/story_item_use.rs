use crate::{RuleError, combat};
use centurions_content::records::{InventorySlots, InventorySlotsMut, ItemDefinition};
use centurions_content::{Content, InventoryRecord, Warrior, span};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenarioItemUse {
    pub warriors: Vec<Warrior>,
    pub return_code: i16,
    pub script: Option<i16>,
    pub sounds: Vec<i16>,
    pub warnings: Vec<i16>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ScriptItemContext {
    Exploration,
    Encounter,
}

/// Items12ac14 warns for non-activating samples;12ad9a skips their effects.
pub(crate) fn reject_inert_item(definition: &[u8]) -> Result<(), RuleError> {
    let definition = ItemDefinition(definition);
    let class = definition.category()?.wrapping_abs();
    let effect = definition.spell()?;
    if class != 23 && effect <= 1100 {
        return Err(RuleError::Refused {
            feedback: if matches!(class, 24 | 25) && effect < 1100 {
                vec![
                    crate::feedback::NativeFeedback::Warning { index: 88 },
                    crate::feedback::NativeFeedback::Warning { index: 89 },
                ]
            } else {
                vec![]
            },
        });
    }
    Ok(())
}

/// items 12ac90 uses class 23 and buttonchoice 10a9f4 decodes its script return.
pub fn use_scenario_item(
    warriors: &[Warrior],
    member: usize,
    slot: usize,
    content: &Content,
    race_movements: &[i8],
    context: ScriptItemContext,
    retained: Option<&InventoryRecord>,
) -> Result<ScenarioItemUse, RuleError> {
    let warrior = warriors
        .get(member)
        .ok_or_else(|| RuleError::Invalid("selected item warrior unavailable".into()))?;
    let bytes = span(&warrior.bytes, 0, 3138)?;
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
    if definition.category()?.wrapping_abs() != 23 {
        return Err(RuleError::Unsupported {
            action: "item.use",
            dependency: "non-script item use",
        });
    }
    let quantity = selected_item.quantity()?;
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
    };
    result.warriors[member]
        .record_mut()
        .set_inventory_quantity(slot, quantity);
    let carried = result.warriors[member]
        .record()
        .carried_weight()?
        .wrapping_sub(definition.unit_weight()?);
    result.warriors[member]
        .record_mut()
        .set_carried_weight(carried);
    if remove {
        result.sounds.push(655);
        if selected_item.equipment_flag() != 0 {
            // dropitem121152 removes worn attributes before its fixed-slot shift.
            let selected = warrior.inventory()?[slot].clone();
            let removed = crate::editor_items::retained_equipment(
                &result.warriors[member],
                content,
                slot,
                false,
                false,
                retained.unwrap_or(&selected),
            )?;
            result.warriors[member] = removed.warrior;
            result.warnings.extend(removed.warnings);
            if removed.native_return as u16 == 0 {
                return Ok(result);
            }
        }
        let weight = definition
            .base_weight()?
            .wrapping_add(definition.unit_weight()?.wrapping_mul(quantity));
        let warrior = &mut result.warriors[member];
        let carried = warrior.record().carried_weight()?.wrapping_sub(weight);
        warrior.record_mut().set_carried_weight(carried);
        InventorySlotsMut(&mut warrior.bytes).remove_dropped_slot(slot, (count - 1) as i16);
        let race = warrior.record().race()?.wrapping_sub(1);
        let movement = race_movements
            .get(race as usize)
            .ok_or_else(|| RuleError::Invalid("selected item race movement unavailable".into()))?;
        combat::calculate_movement(&mut result.warriors[member], *movement)?;
    }
    Ok(result)
}
