use crate::RuleError;
use crate::combat::spells::FIRST_SPELL_DESCRIPTOR;
use centurions_content::ITEM_BYTES;
use centurions_content::{Content, word};
use serde::{Deserialize, Serialize};

/// The item, ammunition and weapon definitions the game has loaded to work with.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadedItems {
    pub item: Option<Vec<u8>>,
    pub ammo: Option<Vec<u8>>,
    pub weapon: Option<Vec<u8>>,
}

impl LoadedItems {
    /// Checks that each loaded definition has the full length of an item.
    pub fn validate(&self) -> Result<(), RuleError> {
        if [&self.item, &self.ammo, &self.weapon]
            .into_iter()
            .flatten()
            .any(|record| record.len() != ITEM_BYTES)
        {
            return Err(RuleError::Invalid("ammunition work record length".into()));
        }
        Ok(())
    }

    /// Loads an item's definition as the item being worked on.
    pub(crate) fn load_item(&mut self, content: &Content, id: i16) -> Result<i16, RuleError> {
        self.load_item_from(&content.items, id)
    }

    /// Loads an item's definition from a table and returns its caliber word (byte 32 of the
    /// definition).
    pub(crate) fn load_item_from(&mut self, items: &[Vec<u8>], id: i16) -> Result<i16, RuleError> {
        let record = item_definition(items, id)?;
        let context = word(&record, 32)?;
        self.item = Some(record);
        Ok(context)
    }

    /// Loads item 0 as the item, its ammunition and its weapon, the weapon copy having no
    /// immunity class.
    pub(crate) fn load_natural(&mut self, items: &[Vec<u8>]) -> Result<(), RuleError> {
        let record = item_definition(items, 0)?;
        let mut weapon = record.clone();
        put_word(&mut weapon, 100, 0);
        self.weapon = Some(weapon);
        self.ammo = Some(record.clone());
        self.item = Some(record);
        Ok(())
    }

    /// Loads an item's definition as the weapon being worked on.
    pub(crate) fn load_weapon(&mut self, items: &[Vec<u8>], id: i16) -> Result<(), RuleError> {
        self.weapon = Some(item_definition(items, id)?);
        Ok(())
    }

    /// Pairs a weapon with its ammunition: both get a positive type, and a weapon that carries a
    /// spell takes the ammunition's words 12, 14 and 56 where the ammunition sets them.
    pub(crate) fn prepare_pair(&mut self, weapon: &[u8], ammo: &[u8]) -> Result<(), RuleError> {
        if weapon.len() != ITEM_BYTES || ammo.len() != ITEM_BYTES {
            return Err(RuleError::Invalid("ammunition work record length".into()));
        }
        let mut prepared = weapon.to_vec();
        put_word(&mut prepared, 26, word(weapon, 26)?.wrapping_abs());
        let mut ammunition = ammo.to_vec();
        put_word(&mut ammunition, 26, word(ammo, 26)?.wrapping_abs());
        if word(weapon, 44)? >= FIRST_SPELL_DESCRIPTOR {
            for offset in [12, 14] {
                let value = word(ammo, offset)?;
                if value != 0 {
                    put_word(&mut prepared, offset, value);
                }
            }
            if word(ammo, 56)? != 0 {
                put_word(&mut prepared, 56, i16::from(ammo[57] as i8));
            }
        }
        self.weapon = Some(prepared);
        self.ammo = Some(ammunition);
        Ok(())
    }
}

/// Sets a two-byte value in a definition.
fn put_word(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

/// The definition of an item, with its type made positive.
pub fn definition(content: &Content, id: i16) -> Result<Vec<u8>, RuleError> {
    item_definition(&content.items, id)
}
/// The definition of an item from a table, with its type made positive.
fn item_definition(items: &[Vec<u8>], id: i16) -> Result<Vec<u8>, RuleError> {
    let bytes = items
        .get(usize::from(id.unsigned_abs()))
        .ok_or_else(|| RuleError::Invalid("ammunition definition missing".into()))?;
    if bytes.len() != ITEM_BYTES {
        return Err(RuleError::Invalid("ammunition definition length".into()));
    }
    let mut result = bytes.clone();
    let class = word(&result, 26)?.wrapping_abs();
    put_word(&mut result, 26, class);
    Ok(result)
}
