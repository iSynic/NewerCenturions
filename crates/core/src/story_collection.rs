use crate::{RuleError, combat};
use centurions_content::records::{InventorySlotsMut, ItemDefinition};
use centurions_content::{Content, InventoryRecord, Warrior, span};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Collection {
    pub warrior: Warrior,
    pub item: InventoryRecord,
    pub collected: bool,
    pub sounds: Vec<i16>,
    pub warnings: Vec<i16>,
}

pub fn collect_story_item(
    warrior: &Warrior,
    item: &InventoryRecord,
    content: &Content,
    race_movement: i8,
) -> Result<Collection, RuleError> {
    span(&warrior.bytes, 0, 3138)?;
    let definition = content
        .items
        .get(usize::from(item.id().unsigned_abs()))
        .ok_or_else(|| RuleError::Invalid("collected item definition is unavailable".into()))?;
    let definition = ItemDefinition(definition);
    if item.id() == 0 {
        return Err(RuleError::Invalid("collected item is empty".into()));
    }
    if (![804, 805, 806, 807, 808, 809, 833].contains(&item.id()) && definition.category()? != 23)
        || definition.reserve_ammunition_kind()? != 0
        || definition.unit_weight()? != 0
    {
        return Err(RuleError::Unsupported {
            action: "loot.take",
            dependency: "collection outside nonstacking quest items",
        });
    }
    let sound = match definition.pickup_sound()? {
        0 => 6002,
        authored => authored,
    };
    let mut result = Collection {
        warrior: warrior.clone(),
        item: item.clone(),
        collected: false,
        sounds: vec![sound],
        warnings: Vec::new(),
    };
    let count = warrior.inventory()?.len();
    if count >= 99 {
        result.warnings.push(147);
        return Ok(result);
    }
    let needed = definition
        .supplied_quantity()?
        .wrapping_mul(definition.unit_weight()?)
        .wrapping_add(warrior.record().carried_weight()?)
        .wrapping_add(definition.base_weight()?) as u16;
    if needed > warrior.record().weight_capacity()? as u16 {
        result.warnings.push(19);
        return Ok(result);
    }
    InventorySlotsMut(&mut result.warrior.bytes).append_collected(count, item, definition)?;
    crate::equipment::calculate_weight(&mut result.warrior, content)?;
    combat::calculate_movement(&mut result.warrior, race_movement)?;
    result.item.clear_id();
    result.collected = true;
    Ok(result)
}
