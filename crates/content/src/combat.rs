use crate::{ContentError, span, word};
use serde::{Deserialize, Serialize};

/// Size of one battle record in the scenario's `Data BD` file.
pub const BATTLE_BYTES: usize = 1372;
/// Size of one monster record in the scenario's `Data MD` file.
pub const MONSTER_BYTES: usize = 288;

/// The monster's name, a Str29 in its Data MD record; bytes after the length
/// are padding kept with the record.
pub fn monster_name(record: &[u8]) -> Result<String, ContentError> {
    crate::ui::pascal_text(span(record, 218, 30)?)
}

/// The tables a battle is built from, each loaded as raw bytes.
#[derive(Clone, Debug)]
pub struct CombatContent {
    /// The scenario's `Data EDCD` EXTRA table, 10-byte encounter records that story parameters
    /// select.
    pub encounters: Vec<u8>,
    /// The scenario's `Data BD`: battle records of `BATTLE_BYTES` each.
    pub battles: Vec<u8>,
    /// The scenario's `Data MD`: monster records of `MONSTER_BYTES` each.
    pub monsters: Vec<u8>,
    /// The movement rating of each of the ten races, from `Data Race`; it sets
    /// the party's movement at the start of a battle.
    pub race_moves: Vec<i8>,
    /// The installation's `Data S` file of spell and attack records.
    pub attacks: Vec<u8>,
}

/// One battle from `Data BD`: where each monster stands, what the story says
/// around it and how far away the party starts.
#[derive(Clone, Debug)]
pub struct BattleRecord {
    /// Monster ids placed on the 26 by 26 battle grid, indexed `x * 26 + y`; zero is empty
    /// and a negative id puts that monster on the opposing side.
    pub battle_grid: Vec<i16>,
    /// Dialogue shown when the battle begins; zero shows none.
    pub before_message: i16,
    /// Dialogue shown when the party wins; zero shows none.
    pub after_message: i16,
    /// Where the party's squad is placed on the battle field.
    pub squad_start: (i16, i16),
    /// Upper bound of the random distance between the party and the monsters when the
    /// battle begins.
    pub distance: i8,
}
impl BattleRecord {
    /// Reads battle `id` from the `Data BD` table. The sign of `id` is ignored.
    ///
    /// Original: combatsetup
    pub fn read(table: &[u8], id: i16) -> Result<Self, ContentError> {
        let bytes = span(
            table,
            usize::from(id.unsigned_abs()) * BATTLE_BYTES,
            BATTLE_BYTES,
        )?;
        Ok(Self {
            battle_grid: bytes[..1352]
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
/// One monster from `Data MD` as stored, before a battle copies it and rolls its
/// stamina.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MonsterRecord {
    /// The record's bytes, `MONSTER_BYTES` long.
    pub bytes: Vec<u8>,
}
impl MonsterRecord {
    /// Reads monster `id` from the `Data MD` table. The sign of `id` is ignored.
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
    /// The monster's current stamina.
    pub fn stamina(&self) -> Result<i16, ContentError> {
        self.record().stamina()
    }
    /// The monster's maximum stamina.
    pub fn stamina_max(&self) -> Result<i16, ContentError> {
        word(&self.bytes, 0x112)
    }
    /// The monster's poison condition, a signed byte whose magnitude is the stamina
    /// it loses (or regains) each round; zero is none.
    pub fn poison_timer(&self) -> Result<i8, ContentError> {
        Ok(span(&self.bytes, 0xb5, 1)?[0] as i8)
    }
    /// The monster's agility, which orders it in the battle.
    pub fn agility(&self) -> Result<i16, ContentError> {
        word(&self.bytes, 4)
    }
    /// Whether the monster fights the party (its faction is set).
    pub fn hostile(&self) -> Result<bool, ContentError> {
        Ok(word(&self.bytes, 0xe)? != 0)
    }
}
