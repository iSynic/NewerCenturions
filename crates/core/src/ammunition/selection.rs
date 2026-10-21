use super::{definition, unsupported};
use crate::RuleError;
use centurions_content::records::ItemDefinition;
use centurions_content::{Content, InventoryRecord, word};

pub(super) fn validate_instance(
    record: &InventoryRecord,
    content: &Content,
) -> Result<i16, RuleError> {
    let weapon = definition(content, record.id())?;
    let category = ItemDefinition(&weapon).category()?;
    if !matches!(category, 2 | 15 | 20) {
        return Err(unsupported("refill effects for this equipment category"));
    }
    let loaded = record.ammunition_id()?;
    let compatible = |id| -> Result<bool, RuleError> {
        let ammo = definition(content, id)?;
        // refit14e5e4 matches the caliber even when ammunition has a use effect.
        // fit handles those effects separately after its candidate is selected.
        Ok(ItemDefinition(&ammo).category()? == 26
            && ItemDefinition(&weapon).ammunition_caliber()?
                == ItemDefinition(&ammo).reserve_ammunition_kind()?)
    };
    if loaded != 0 {
        // refit14e5e4 uses an existing magazine's reserve index directly. Its
        // empty-magazine branch can itself write 200 + inventory slot here.
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
