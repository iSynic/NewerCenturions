use crate::{ContentError, span, word};
use serde::{Deserialize, Serialize};

pub const BATTLE_BYTES: usize = 1372;
pub const MONSTER_BYTES: usize = 288;

/// Data MD keeps a Str29; bytes after its length are retained record padding.
pub fn monster_name(record: &[u8]) -> Result<String, ContentError> {
    crate::ui::pascal_text(span(record, 218, 30)?)
}

#[derive(Clone, Debug)]
pub struct CombatContent {
    pub encounters: Vec<u8>,
    pub battles: Vec<u8>,
    pub monsters: Vec<u8>,
    pub race_moves: Vec<i8>,
    pub attacks: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct BattleRecord {
    pub cells: Vec<i16>,
    pub before_message: i16,
    pub after_message: i16,
    pub squad_start: (i16, i16),
    pub distance: i8,
}
impl BattleRecord {
    pub fn read(table: &[u8], id: i16) -> Result<Self, ContentError> {
        let bytes = span(
            table,
            usize::from(id.unsigned_abs()) * BATTLE_BYTES,
            BATTLE_BYTES,
        )?;
        Ok(Self {
            cells: bytes[..1352]
                .as_chunks::<2>()
                .0
                .iter()
                .map(|v| i16::from_be_bytes(*v))
                .collect(),
            before_message: word(bytes, 1352)?,
            after_message: word(bytes, 1354)?,
            squad_start: (word(bytes, 1356)?, word(bytes, 1358)?),
            distance: bytes[1370] as i8,
        })
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MonsterRecord {
    pub bytes: Vec<u8>,
}
impl MonsterRecord {
    pub fn read(table: &[u8], id: i16) -> Result<Self, ContentError> {
        Ok(Self {
            bytes: span(
                table,
                usize::from(id.unsigned_abs()) * MONSTER_BYTES,
                MONSTER_BYTES,
            )?
            .to_vec(),
        })
    }
    pub fn stamina(&self) -> Result<i16, ContentError> {
        word(&self.bytes, 0x110)
    }
    pub fn maximum_stamina(&self) -> Result<i16, ContentError> {
        word(&self.bytes, 0x112)
    }
    pub fn poison_timer(&self) -> Result<i8, ContentError> {
        Ok(span(&self.bytes, 0xb5, 1)?[0] as i8)
    }
    pub fn speed(&self) -> Result<i16, ContentError> {
        word(&self.bytes, 4)
    }
    pub fn hostile(&self) -> Result<bool, ContentError> {
        Ok(word(&self.bytes, 0xe)? != 0)
    }
}
