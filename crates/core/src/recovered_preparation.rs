use crate::{
    RuleError,
    recovered_records::{self, ItemTables, RecordWrite, SPELL_BYTES},
};
use centurions_content::{ITEM_BYTES, WARRIOR_BYTES, Warrior, span, word};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeaponPreparation {
    pub warrior: Warrior,
    pub item_work: [u8; ITEM_BYTES],
    pub writes: Vec<RecordWrite>,
}

/// CODE 1 toweap, 0x15ef50. The mode BYTE is cleared before the optional load.
pub fn to_weapon(
    warrior: &Warrior,
    warrior_index: u16,
    tables: &ItemTables<'_>,
    item_work: &[u8; ITEM_BYTES],
) -> Result<WeaponPreparation, RuleError> {
    span(&warrior.bytes, 0, WARRIOR_BYTES)?;
    let address = 0x448aa0 + u32::from(warrior_index) * WARRIOR_BYTES as u32;
    let mut next = warrior.clone();
    next.bytes[0x9dd] = 0;
    let mut writes = vec![RecordWrite {
        address: address + 0x9dd,
        bytes: vec![0],
    }];
    let id = word(&warrior.bytes, 0xca)?;
    let mut item = *item_work;
    let speed = if id == 0 {
        word(&warrior.bytes, 0x38)?.wrapping_mul(8).wrapping_add(30)
    } else {
        let loaded = recovered_records::loaditem(tables, item_work, id)?;
        item = loaded.bytes;
        writes.extend(loaded.writes);
        word(&item, 0xe)?
    };
    next.bytes[0x128..0x12a].copy_from_slice(&speed.to_be_bytes());
    writes.push(RecordWrite {
        address: address + 0x128,
        bytes: speed.to_be_bytes().to_vec(),
    });
    Ok(WeaponPreparation {
        warrior: next,
        item_work: item,
        writes,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissilePreparation {
    pub available: bool,
    pub item_work: [u8; ITEM_BYTES],
    pub spell_work: [u8; SPELL_BYTES],
    pub spell_coordinates: [i16; 3],
    pub writes: Vec<RecordWrite>,
}

/// CODE 1 tomissle, 0x15efee. No equipment leaves all scratch records untouched.
pub fn to_missile(
    warrior: &Warrior,
    tables: &ItemTables<'_>,
    spells: &[u8],
    item_work: &[u8; ITEM_BYTES],
    spell_work: &[u8; SPELL_BYTES],
    spell_coordinates: [i16; 3],
) -> Result<MissilePreparation, RuleError> {
    let id = word(&warrior.bytes, 0xe4)?;
    let mut result = MissilePreparation {
        available: id != 0,
        item_work: *item_work,
        spell_work: *spell_work,
        spell_coordinates,
        writes: Vec::new(),
    };
    if id != 0 {
        let loaded = recovered_records::loaditem(tables, item_work, id)?;
        let spell = recovered_records::loadspell2(spells, word(&loaded.bytes, 0x2c)?)?;
        result.item_work = loaded.bytes;
        result.spell_work = spell.record.bytes;
        result.spell_coordinates = spell.coordinates;
        result.writes.extend(loaded.writes);
        result.writes.extend(spell.record.writes);
    }
    Ok(result)
}

/// CODE 1 getmovecost, 0x153de6: WORD tile normalization, then MULU.W addressing.
pub fn movement_cost(properties: &[u8], tile: i16, dx: i8, dy: i8) -> Result<i16, RuleError> {
    let normalized = if tile > 999 {
        tile.wrapping_sub(1000)
    } else {
        tile
    };
    let offset = usize::from(normalized as u16) * 40 + 2;
    Ok(crate::rules::movement_cost(
        word(properties, offset)?,
        dx,
        dy,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unarmed_and_missing_missile_preserve_scratch_and_wrap_word_speed() {
        let mut warrior = Warrior {
            name: "A".into(),
            original_sha256: String::new(),
            bytes: vec![0; WARRIOR_BYTES],
        };
        warrior.bytes[0x38..0x3a].copy_from_slice(&i16::MAX.to_be_bytes());
        warrior.bytes[0x9dd] = 3;
        let tables = ItemTables { families: [&[]; 5] };
        let item = [0xa5; ITEM_BYTES];
        let spell = [0x5a; SPELL_BYTES];
        let weapon = to_weapon(&warrior, 2, &tables, &item).unwrap();
        assert_eq!(word(&weapon.warrior.bytes, 0x128).unwrap(), 22);
        assert_eq!(weapon.warrior.bytes[0x9dd], 0);
        assert_eq!(warrior.bytes[0x9dd], 3);
        assert_eq!(weapon.item_work, item);
        assert_eq!(weapon.writes.len(), 2);
        let missile = to_missile(&warrior, &tables, &[], &item, &spell, [2, 3, 4]).unwrap();
        assert!(!missile.available);
        assert_eq!(missile.spell_coordinates, [2, 3, 4]);
        assert_eq!(missile.item_work, item);
        assert_eq!(missile.spell_work, spell);
        assert!(missile.writes.is_empty());
        let mut props = vec![0; 80];
        props[42..44].copy_from_slice(&9_i16.to_be_bytes());
        assert_eq!(movement_cost(&props, 1001, -1, 1).unwrap(), 5);
        assert_eq!(movement_cost(&props, 1, -1, 1).unwrap(), 5);
        assert!(movement_cost(&props, -1, 0, 0).is_err());
    }
}
