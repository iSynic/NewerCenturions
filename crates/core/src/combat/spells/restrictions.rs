//! Restrictions for combat/spells.
use super::RuleError;
use centurions_content::word;

pub(super) fn validate_direct(packet: &[u8]) -> Result<(), RuleError> {
    if packet.len() != 58 {
        return Err(RuleError::Invalid("spell packet size".into()));
    }
    let unresisted_damage =
        packet[9] == 3 && packet[19] == 8 && packet[20] == 8 && word(packet, 40)? == 0;
    let poison = word(packet, 40)? == 4 && packet[19] == 1 && packet[20] == 1;
    let scripted = word(packet, 40)? == 23 && packet[9] == 3 && matches!(packet[19], 7 | 8);
    if (word(packet, 40)? != 0 && !poison && !scripted)
        || (!(0..=6).contains(&packet[19]) && !unresisted_damage && !scripted)
        || packet[20] as i8 <= 0
        || packet[23..30].iter().any(|value| *value != 0)
        || !matches!(word(packet, 50)?, 0 | 0x2000)
    {
        return Err(RuleError::Unsupported {
            action: "combat.spell",
            dependency: "spell effects, other race restrictions or elemental damage",
        });
    }
    Ok(())
}

pub(super) fn race_is_excluded(
    packet: &[u8],
    warrior: &[u8],
    action: &'static str,
) -> Result<bool, RuleError> {
    let race_mask = word(packet, 50)? as u16;
    if race_mask == 0 {
        return Ok(false);
    }
    let race = word(warrior, 0x32)?;
    if !(1..=16).contains(&race) {
        return Err(RuleError::Unsupported {
            action,
            dependency: "racial mask beyond recovered party races",
        });
    }
    Ok(race_mask & (0x8000_u16 >> (race - 1)) != 0)
}

pub(super) fn restriction_bit(packet: &[u8], index: i16) -> Result<bool, RuleError> {
    let offset = 50 + (i32::from(index) >> 3);
    let byte = usize::try_from(offset)
        .ok()
        .and_then(|offset| packet.get(offset))
        .ok_or(RuleError::Unsupported {
            action: "combat.spell",
            dependency: "racial bit test outside the loaded spell packet",
        })?;
    Ok(byte & (0x80 >> (index as u16 & 7)) != 0)
}

#[derive(Clone, Copy)]
pub struct RestrictionMemoryView<'a> {
    item_cache: &'a [u8],
    ordinary_backup: bool,
    editor_backup: Option<&'a [u8]>,
    item_work_alias: Option<i16>,
}

pub(super) fn monster_race_is_excluded(
    packet: &[u8],
    body: usize,
    memory: Option<RestrictionMemoryView<'_>>,
) -> Result<bool, RuleError> {
    let memory = memory.ok_or(RuleError::Unsupported {
        action: "combat.spell",
        dependency: "restricted monster target uses unrepresented party memory",
    })?;
    restriction_bit(packet, memory.race_word(body)?.wrapping_sub(1))
}

impl<'a> RestrictionMemoryView<'a> {
    pub fn item_cache_only(item_cache: &'a [u8]) -> Self {
        Self {
            item_cache,
            ordinary_backup: false,
            editor_backup: None,
            item_work_alias: None,
        }
    }

    /// Supported ordinary history preserves the original startup editor backup.
    /// item_work_alias is WORD at offset 0x32, distinct from field32 at 0x20.
    pub fn ordinary_startup(item_cache: &'a [u8], item_work_alias: Option<i16>) -> Self {
        Self {
            item_cache,
            ordinary_backup: true,
            editor_backup: None,
            item_work_alias,
        }
    }

    pub fn with_editor_backup(
        item_cache: &'a [u8],
        item_work_alias: Option<i16>,
        backup: &'a [u8],
    ) -> Result<Self, RuleError> {
        if backup.len() != EDITOR_BACKUP_BYTES {
            return Err(RuleError::Invalid("editor backup record size".into()));
        }
        Ok(Self {
            item_cache,
            ordinary_backup: false,
            editor_backup: Some(backup),
            item_work_alias,
        })
    }

    fn race_word(self, body: usize) -> Result<i16, RuleError> {
        let unrepresented = || RuleError::Unsupported {
            action: "combat.spell",
            dependency: "restricted monster race alias outside the represented memory owners",
        };
        match RaceAlias::of(body)? {
            RaceAlias::EditorBackup { .. } if self.ordinary_backup => Ok(0),
            RaceAlias::EditorBackup { offset } => match self.editor_backup {
                Some(backup) => Ok(word(backup, offset)?),
                None => Err(unrepresented()),
            },
            RaceAlias::ItemWorkAlias => self.item_work_alias.ok_or_else(unrepresented),
            RaceAlias::ItemCache { offset } if self.item_cache.len() == ITEM_CACHE_BYTES => {
                Ok(word(self.item_cache, offset)?)
            }
            RaceAlias::ItemCache { .. } | RaceAlias::Unrepresented => Err(unrepresented()),
        }
    }
}

/// Party race reads use the warrior table at 0x448aa0 (3138-byte records, race
/// WORD 0x32). An enemy body index runs past that table into these owners.
const WARRIOR_TABLE: usize = 0x448aa0;
const WARRIOR_BYTES: usize = 3138;
const RACE_WORD: usize = 0x32;
/// Startup editor backup; `with_editor_backup` checks the same size.
const EDITOR_BACKUP: usize = 0x44ecb0;
const EDITOR_BACKUP_BYTES: usize = 18_828;
/// WORD 0x32 of the item work record that follows the editor backup at 0x45363c.
const ITEM_WORK_ALIAS: usize = 0x45366e;
/// The first 200 item definitions, cached after the item work records.
const ITEM_CACHE: usize = 0x4537fe;
const ITEM_CACHE_BYTES: usize = 30_000;

/// The memory owner, and the WORD offset inside it, that a race read reaches.
enum RaceAlias {
    EditorBackup { offset: usize },
    ItemWorkAlias,
    ItemCache { offset: usize },
    Unrepresented,
}

impl RaceAlias {
    fn of(body: usize) -> Result<Self, RuleError> {
        let address = body
            .checked_mul(WARRIOR_BYTES)
            .and_then(|stride| WARRIOR_TABLE.checked_add(stride))
            .and_then(|address| address.checked_add(RACE_WORD))
            .ok_or_else(|| RuleError::Invalid("restriction alias address overflow".into()))?;
        let word_within = |base: usize, bytes: usize| {
            address
                .checked_sub(base)
                .filter(|offset| *offset <= bytes - 2)
        };
        Ok(
            if let Some(offset) = word_within(EDITOR_BACKUP, EDITOR_BACKUP_BYTES) {
                Self::EditorBackup { offset }
            } else if address == ITEM_WORK_ALIAS {
                Self::ItemWorkAlias
            } else if let Some(offset) = word_within(ITEM_CACHE, ITEM_CACHE_BYTES) {
                Self::ItemCache { offset }
            } else {
                Self::Unrepresented
            },
        )
    }
}
