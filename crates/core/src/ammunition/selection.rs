use super::{definition, unsupported};
use crate::RuleError;
use centurions_content::records::ItemDefinition;
use centurions_content::{Content, InventoryRecord, word};

/// Checks that the item in a record is a melee, missile or category 20 item, and returns the
/// number of the ammunition it holds: the loaded one, which must be a reserve number from 200 to
/// 399, or when nothing is loaded the first ammunition that fits its caliber.
pub(super) fn validate_instance(
    record: &InventoryRecord,
    content: &Content,
) -> Result<i16, RuleError> {
    let weapon = definition(content, record.id())?;
    let category = ItemDefinition(&weapon).item_type()?;
    if !matches!(category, 2 | 15 | 20) {
        return Err(unsupported("refill effects for this equipment category"));
    }
    let loaded = record.ammunition_id()?;
    let compatible = |id| -> Result<bool, RuleError> {
        let ammo = definition(content, id)?;
        // Refit matches the caliber even when the ammunition has a use effect;
        // fitting handles the effect after the candidate is chosen.
        Ok(ItemDefinition(&ammo).item_type()? == 26
            && ItemDefinition(&weapon).ammunition_caliber()?
                == ItemDefinition(&ammo).reserve_ammunition_kind()?)
    };
    if loaded != 0 {
        // A loaded magazine names its reserve directly; refit's empty-magazine
        // branch can itself store 200 + inventory slot here.
        if !(200..400).contains(&loaded) {
            return Err("incompatible loaded ammunition".into());
        }
        definition(content, loaded)?;
        return Ok(loaded);
    }
    for id in 200..400 {
        if compatible(id)? {
            return Ok(id);
        }
    }
    Err(unsupported("no compatible ammunition definition"))
}

/// The numbers (200 to 399) of the ammunition items that fit a weapon's caliber.
pub fn compatible_ammunition(
    content: &Content,
    record: &InventoryRecord,
) -> Result<Vec<i16>, RuleError> {
    let item = definition(content, record.id())?;
    let caliber = ItemDefinition(&item).ammunition_caliber()?;
    (200..400)
        .filter_map(|id| {
            let ammo = match definition(content, id) {
                Ok(ammo) => ammo,
                Err(error) => return Some(Err(error)),
            };
            match (word(&ammo, 26), word(&ammo, 34)) {
                (Ok(26), Ok(value)) if value == caliber => Some(Ok(id)),
                (Ok(_), Ok(_)) => None,
                (Err(error), _) | (_, Err(error)) => Some(Err(error.into())),
            }
        })
        .collect()
}
