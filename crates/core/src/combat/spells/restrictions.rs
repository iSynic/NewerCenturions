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
        if backup.len() != 18_828 {
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
        let address = body
            .checked_mul(3138)
            .and_then(|stride| 0x448aa0_usize.checked_add(stride))
            .and_then(|address| address.checked_add(0x32))
            .ok_or_else(|| RuleError::Invalid("restriction alias address overflow".into()))?;
        if self.ordinary_backup && (0x44ecb0..0x45363b).contains(&address) {
            return Ok(0);
        }
        if let Some(backup) = self.editor_backup
            && let Some(offset) = address
                .checked_sub(0x44ecb0)
                .filter(|offset| *offset <= backup.len() - 2)
        {
            return Ok(word(backup, offset)?);
        }
        if address == 0x45366e
            && let Some(alias) = self.item_work_alias
        {
            return Ok(alias);
        }
        if self.item_cache.len() == 30_000
            && let Some(offset) = address
                .checked_sub(0x4537fe)
                .filter(|offset| *offset <= self.item_cache.len() - 2)
        {
            return Ok(word(self.item_cache, offset)?);
        }
        Err(RuleError::Unsupported {
            action: "combat.spell",
            dependency: "restricted monster race alias outside the represented memory owners",
        })
    }
}
