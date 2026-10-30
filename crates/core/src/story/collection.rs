use crate::{RuleError, combat};
use centurions_content::records::{InventorySlotsMut, ItemDefinition};
use centurions_content::{Content, InventoryRecord, WARRIOR_BYTES, Warrior, span};

/// What a warrior did with an item: the warrior afterward, the item, whether it was collected, and
/// the sounds and warnings that went with it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Collection {
    pub warrior: Warrior,
    pub item: InventoryRecord,
    pub collected: bool,
    pub sounds: Vec<i16>,
    pub warnings: Vec<i16>,
}

/// Offers a warrior a quest item a script gives: it is collected if the warrior can carry it, and
/// movement is recalculated. Other items are not supported.
pub fn collect_story_item(
    warrior: &Warrior,
    item: &InventoryRecord,
    content: &Content,
    race_movement: i8,
) -> Result<Collection, RuleError> {
    span(&warrior.bytes, 0, WARRIOR_BYTES)?;
    let definition = content
        .items
        .get(usize::from(item.id().unsigned_abs()))
        .ok_or_else(|| RuleError::Invalid("collected item definition is unavailable".into()))?;
    let definition = ItemDefinition(definition);
    if item.id() == 0 {
        return Err(RuleError::Invalid("collected item is empty".into()));
    }
    if (![804, 805, 806, 807, 808, 809, 833].contains(&item.id()) && definition.item_type()? != 23)
        || definition.reserve_ammunition_kind()? != 0
    {
        return Err(RuleError::Unsupported {
            action: "loot.take",
            dependency: "collection outside quest items",
        });
    }
    let sound = match definition.sound()? {
        0 => 6002,
        authored => authored,
    };
    let mut result = collect_inventory_item(warrior, item, content, sound, false)?;
    if result.collected {
        combat::calculate_movement(&mut result.warrior, race_movement)?;
    }
    Ok(result)
}

/// Merges a stacking item into an existing row before the rejection tests. A
/// merge keeps the existing row and skips calcw, so the load grows by the raw
/// quantity rather than quantity times unit weight.
///
/// Original: booty
pub(crate) fn collect_inventory_item(
    warrior: &Warrior,
    item: &InventoryRecord,
    content: &Content,
    sound: i16,
    refusal_sound: bool,
) -> Result<Collection, RuleError> {
    let definition = ItemDefinition(
        content
            .items
            .get(usize::from(item.id().unsigned_abs()))
            .ok_or("collected item definition is unavailable")?,
    );
    let mut result = Collection {
        warrior: warrior.clone(),
        item: item.clone(),
        collected: false,
        sounds: vec![sound],
        warnings: Vec::new(),
    };
    let count = warrior.inventory()?.len();
    let stacked = if definition.charge_weight()? != 0 {
        warrior
            .inventory()?
            .iter()
            .position(|owned| owned.id() == item.id())
    } else {
        None
    };
    if let Some(slot) = stacked {
        let quantity = item.charge()?;
        let total = result
            .warrior
            .record()
            .inventory_quantity(slot)?
            .wrapping_add(quantity);
        result
            .warrior
            .record_mut()
            .set_inventory_quantity(slot, total);
        let weight = result.warrior.record().load()?.wrapping_add(quantity);
        result.warrior.record_mut().set_load(weight);
    }
    if count >= 99 {
        if refusal_sound {
            result.sounds.push(6000);
        }
        result.warnings.push(147);
        return Ok(result);
    }
    let needed = definition
        .supplied_quantity()?
        .wrapping_mul(definition.charge_weight()?)
        .wrapping_add(result.warrior.record().load()?)
        .wrapping_add(definition.weight()?) as u16;
    if needed > warrior.record().load_max()? as u16 {
        if refusal_sound {
            result.sounds.push(6000);
        }
        result.warnings.push(19);
        return Ok(result);
    }
    if stacked.is_none() {
        InventorySlotsMut(&mut result.warrior.bytes).append_collected(count, item, definition)?;
        crate::equipment::calculate_weight(&mut result.warrior, content)?;
    }
    result.item.clear_id();
    result.collected = true;
    Ok(result)
}
