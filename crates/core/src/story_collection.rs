use crate::{RuleError, combat};
use centurions_content::{Content, InventoryRecord, Warrior, span, word};

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
    if item.id() == 0 {
        return Err(RuleError::Invalid("collected item is empty".into()));
    }
    if (![804, 805, 806, 807, 808, 809, 833].contains(&item.id()) && word(definition, 0x1a)? != 23)
        || word(definition, 0x22)? != 0
        || word(definition, 0x24)? != 0
    {
        return Err(RuleError::Unsupported {
            action: "loot.take",
            dependency: "collection outside nonstacking quest items",
        });
    }
    let sound = match word(definition, 0xc)? {
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
    let needed = word(definition, 0x14)?
        .wrapping_mul(word(definition, 0x24)?)
        .wrapping_add(word(&warrior.bytes, 0x9d6)?)
        .wrapping_add(word(definition, 0x10)?) as u16;
    if needed > word(&warrior.bytes, 0x9d8)? as u16 {
        result.warnings.push(19);
        return Ok(result);
    }
    let start = 0x132 + count * 22;
    let destination = &mut result.warrior.bytes[start..start + 22];
    destination[..8].copy_from_slice(&item.0[..8]);
    destination[..2].copy_from_slice(&word(definition, 0x66)?.to_be_bytes());
    destination[8] = word(definition, 0x22)? as u8;
    destination[9] = 0;
    destination[10] = word(definition, 0x1a)? as u8;
    destination[13..].copy_from_slice(&item.0[13..]);
    result.warrior.bytes[0x1a..0x1c].copy_from_slice(&((count + 1) as i16).to_be_bytes());
    crate::equipment::calculate_weight(&mut result.warrior, content)?;
    combat::calculate_movement(&mut result.warrior, race_movement)?;
    result.item.0[..2].fill(0);
    result.collected = true;
    Ok(result)
}
