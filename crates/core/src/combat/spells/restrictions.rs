//! Restrictions for combat/spells.
use super::RuleError;
use crate::combat::Combat;
use crate::combat::warrior_alias::{self, Alias, AliasWrite, Owners};
use centurions_content::{
    Warrior,
    combat::MonsterRecord,
    records::{AttackPacket, EnemyRecord, ItemDefinition},
    word,
};

/// resolvespell combines damage with recognized condition/healing selectors.
pub(crate) fn is_support(packet: &[u8]) -> Result<bool, RuleError> {
    if packet.len() != 58 {
        return Err(RuleError::Invalid("spell packet size".into()));
    }
    let attack = AttackPacket(packet);
    let effect = attack.effect()?.wrapping_abs();
    // Category5 Psi drain keeps its separately verified resolver.
    let drain = matches!(effect, 60 | 61) && attack.defense_category() != 5;
    // Selectors without a resolvespell or spelllist arm apply damage and report only.
    let generic = matches!(effect, 42..=48 | 50 | 62 | 63 | 65 | 67 | 69 | 71..=89 | 91..=99);
    Ok(drain
        || generic
        || matches!(effect, 0..=41 | 49 | 51..=54 | 57 | 59 | 64 | 66 | 68 | 70 | 90 | 100..))
}

/// WORD45366e belongs to the retained item record, not its field32 (WORD45365c).
pub(crate) fn retained_item_alias(
    work: &crate::ammunition::WorkBuffers,
) -> Result<Option<i16>, RuleError> {
    work.item
        .as_deref()
        .map(|item| {
            ItemDefinition(item)
                .restriction_alias()
                .map_err(RuleError::from)
        })
        .transpose()
}

/// Only non-support packets reach this. Every one is a movement or summon arm, a
/// category-5 drain (spelllist 15ba5a) or direct weapon damage (the per-body weapon
/// resolver), so NPC selection admits each.
pub(super) fn validate_direct(packet: &[u8]) -> Result<(), RuleError> {
    if packet.len() != 58 {
        return Err(RuleError::Invalid("spell packet size".into()));
    }
    Ok(())
}

/// resist150154 tests BitTst(443780, race-1) with the signed race, so races
/// outside 1..16 read other packet bytes rather than a party race table.
pub(super) fn race_is_excluded(
    packet: &[u8],
    warrior: &[u8],
    memory: Option<RestrictionMemoryView<'_>>,
) -> Result<bool, RuleError> {
    if word(packet, 50)? == 0 {
        return Ok(false);
    }
    restriction_bit(
        packet,
        word(warrior, 0x32)?.wrapping_sub(1),
        memory.and_then(|memory| memory.spell_data),
    )
}

/// The packet at 44374e ends where Data S, loaded whole at 443788 by ToolBoxInit
/// (132614), begins, so bytes past the packet read the scenario's spell table.
pub(super) fn restriction_bit(
    packet: &[u8],
    index: i16,
    spell_data: Option<&[u8]>,
) -> Result<bool, RuleError> {
    let offset = 50 + (i32::from(index) >> 3);
    let byte = usize::try_from(offset)
        .ok()
        .and_then(|offset| match offset.checked_sub(58) {
            None => packet.get(offset),
            Some(past) => spell_data?.get(past),
        })
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
    definitions: Option<&'a [Vec<u8>]>,
    npc_records: Option<&'a [bool]>,
    selected_party_caster: Option<i16>,
    resist_caster_level: Option<i16>,
    physical_accuracy: Option<(i16, i16)>,
    /// Data S at 443788, the scenario spell table.
    spell_data: Option<&'a [u8]>,
    /// ITEM work, shop work and the map header for warrior-indexed NPC WORDs.
    owners: Owners<'a>,
    /// Earlier writes of the same command that have not reached those owners yet.
    pending: &'a [AliasWrite],
    /// WORD4435d2, which effect90 reads whatever body casts.
    selected_warrior: Option<i16>,
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
    restriction_bit(
        packet,
        memory.race_word(body)?.wrapping_sub(1),
        memory.spell_data,
    )
}

impl<'a> RestrictionMemoryView<'a> {
    /// spelllist reads WORD4435d2 for the party caster's faction and transfer,
    /// independently of the current initiative body used for reflection.
    pub(crate) fn with_selected_party_caster(mut self, selected: i16) -> Self {
        self.selected_party_caster = Some(selected);
        self
    }

    pub(in crate::combat::spells) fn selected_party_caster(self) -> Option<i16> {
        self.selected_party_caster
    }

    /// spelllist15c10a reads the level and faction of WORD4435d2 for any caster.
    pub(crate) fn with_selected_warrior(mut self, selected: i16) -> Self {
        self.selected_warrior = Some(selected);
        self
    }

    pub(in crate::combat::spells) fn selected_warrior(self) -> Option<i16> {
        self.selected_warrior.or(self.selected_party_caster)
    }

    /// resist150480 subtracts five times the level of WORD4435d2 when the current
    /// initiative body is below 9, otherwise of NPC WORD447196.
    pub(crate) fn with_resist_caster(
        mut self,
        current: Option<i16>,
        (warrior, npc): (usize, usize),
        warriors: &[Warrior],
        enemies: &[MonsterRecord],
    ) -> Self {
        self.resist_caster_level = current.and_then(|body| {
            if body < 9 {
                match warriors.get(warrior) {
                    Some(record) => record.record().level().ok(),
                    None => self.table_word(warrior, 0x3a),
                }
            } else {
                enemies.get(npc)?.record().level().ok()
            }
        });
        self
    }

    /// resist1501a0: the category-9 gate starts at 33 plus the accuracy of the body
    /// at 4470b8[447194]; body 9 and empty entries add nothing. Body 110, the cursor
    /// WORD at row 110, indexes NPC record 100: MONSTER work 440dbc (`monster`).
    pub(crate) fn with_current_accuracy(
        mut self,
        body: Option<i16>,
        (warriors, enemies, monster): (&[Warrior], &[MonsterRecord], Option<&[u8]>),
        difficulty: i16,
    ) -> Self {
        self.physical_accuracy = body
            .and_then(|body| match body {
                0..=8 => match warriors.get(body as usize) {
                    Some(record) => record.record().missile_accuracy().ok(),
                    // Missile accuracy WORDs +4 and +0xb58 of a record past the squad.
                    None => Some(
                        self.table_word(body as usize, 4)?
                            .wrapping_add(self.table_word(body as usize, 0xb58)?),
                    ),
                },
                10.. => {
                    let record = match enemies.get(body as usize - 10) {
                        Some(record) => record.record(),
                        None if body == 110 => EnemyRecord::from_bytes(monster?),
                        None => return None,
                    };
                    Some(
                        record
                            .level()
                            .ok()?
                            .wrapping_mul(5)
                            .wrapping_add(record.missile_accuracy_bonus().ok()?),
                    )
                }
                _ => Some(0),
            })
            .map(|accuracy| (33_i16.wrapping_add(accuracy), difficulty));
        self
    }

    /// A warrior-table WORD through its owner; the editor backup reads zero.
    fn table_word(self, body: usize, field: usize) -> Option<i16> {
        self.alias_word(body, field, |_| RuleError::Invalid(String::new()))
            .ok()
    }

    /// In live combat getup writes WORD4435d2 or WORD447196 from the current body.
    pub(crate) fn with_live_caster(self, combat: &Combat, warriors: &[Warrior]) -> Self {
        let enemies = &combat.enemies.records;
        let current = combat.active_body();
        let this = self.with_current_accuracy(
            Some(
                combat
                    .order
                    .get(combat.cursor)
                    .copied()
                    .unwrap_or(combat.cursor as i16),
            ),
            (warriors, enemies, combat.monster_template()),
            combat.difficulty,
        );
        // A party turn's getup leaves the current body in WORD4435d2.
        let party = current.filter(|body| *body < 9).or(this.selected_warrior());
        this.with_live_resist_caster(combat, warriors, (party, None))
    }

    /// resist150480 tests the body at 4470b8[447194] (row 110 is the cursor WORD
    /// itself): below 9 it reads the level of warrior WORD4435d2 (`party`), otherwise
    /// that of NPC WORD447196 (`npc`), which getup writes from an NPC turn's body.
    pub(crate) fn with_live_resist_caster(
        mut self,
        combat: &Combat,
        warriors: &[Warrior],
        (party, npc): (Option<i16>, Option<usize>),
    ) -> Self {
        let current = combat
            .order
            .get(combat.cursor)
            .copied()
            .unwrap_or(combat.cursor as i16);
        self.resist_caster_level = if current < 9 {
            party
                .and_then(|warrior| warriors.get(usize::try_from(warrior).ok()?))
                .and_then(|record| record.record().level().ok())
        } else {
            let npc = npc.unwrap_or(if current >= 10 {
                current as usize - 10
            } else {
                combat.selected_enemy
            });
            combat
                .story_record(npc)
                .ok()
                .and_then(|record| record.record().level().ok())
        };
        self
    }

    pub(super) fn physical_accuracy(self) -> Option<(i16, i16)> {
        self.physical_accuracy
    }

    pub(super) fn resist_caster_level(self) -> Option<i16> {
        self.resist_caster_level
    }

    pub(crate) fn with_tables(
        mut self,
        tables: &'a centurions_content::mission::StoryTables,
    ) -> Self {
        self.spell_data = tables
            .combat
            .as_ref()
            .map(|combat| combat.attacks.as_slice());
        self
    }

    pub(crate) fn with_definitions(mut self, definitions: &'a [Vec<u8>]) -> Self {
        self.definitions = Some(definitions);
        self
    }

    pub(crate) fn with_npc_records(mut self, known: &'a [bool]) -> Self {
        self.npc_records = Some(known);
        self
    }

    pub(in crate::combat::spells) fn require_npc(self, body: usize) -> Result<(), RuleError> {
        if body >= 10
            && self
                .npc_records
                .is_some_and(|known| !known.get(body - 10).copied().unwrap_or(false))
        {
            return Err(RuleError::Unsupported {
                action: "combat.retained_npc",
                dependency: "inactive NPC record absent from legacy context",
            });
        }
        Ok(())
    }

    pub(crate) fn with_alias_owners<'b>(self, owners: Owners<'b>) -> RestrictionMemoryView<'b>
    where
        'a: 'b,
    {
        RestrictionMemoryView { owners, ..self }
    }

    pub(crate) fn with_pending<'b>(self, pending: &'b [AliasWrite]) -> RestrictionMemoryView<'b>
    where
        'a: 'b,
    {
        RestrictionMemoryView { pending, ..self }
    }

    /// Whether the caller lent the session's warrior table.
    pub(crate) fn holds_table(self) -> bool {
        self.owners.table.is_some()
    }

    /// A warrior-indexed WORD (0x448aa0 + body * 3138 + field) through its owner.
    pub(crate) fn alias_word(
        self,
        body: usize,
        field: usize,
        missing: fn(Alias) -> RuleError,
    ) -> Result<i16, RuleError> {
        if let Some(write) = self
            .pending
            .iter()
            .rfind(|write| write.body == body && write.field == field)
        {
            return Ok(write.value);
        }
        let alias = warrior_alias::locate(body, field);
        let value = match alias {
            // Registered sessions change the backup only through alias writes.
            Alias::EditorBackup(_) if self.ordinary_backup => self.owners.read(alias).or(Some(0)),
            Alias::EditorBackup(offset) => self
                .editor_backup
                .map(|backup| word(backup, offset))
                .transpose()?,
            // The race WORD 0x32 of ITEM work is the retained item alias.
            Alias::ItemWork(0x32) if self.owners.item_work.is_none() => self.item_work_alias,
            Alias::Definition { item, offset } if item < 200 => (self.item_cache.len()
                == ITEM_CACHE_BYTES)
                .then(|| word(self.item_cache, item * 150 + offset))
                .transpose()?,
            Alias::Definition { item, offset } => self
                .definitions
                .and_then(|definitions| definitions.get(item))
                .map(|record| word(record, offset))
                .transpose()?,
            _ => self.owners.read(alias),
        };
        value.ok_or_else(|| missing(alias))
    }

    pub fn item_cache_only(item_cache: &'a [u8]) -> Self {
        Self {
            item_cache,
            ordinary_backup: false,
            editor_backup: None,
            item_work_alias: None,
            definitions: None,
            npc_records: None,
            selected_party_caster: None,
            resist_caster_level: None,
            physical_accuracy: None,
            spell_data: None,
            owners: Owners::default(),
            pending: &[],
            selected_warrior: None,
        }
    }

    /// Registered editing bypasses the DEMO backup writer at 16224c..16225e.
    /// The reconstruction always uses registered behavior, so only alias writes
    /// change the backup; a caller without the session table reads startup zeros.
    /// Captured DEMO histories use with_editor_backup.
    /// item_work_alias is WORD at offset 0x32, distinct from field32 at 0x20.
    pub fn ordinary_startup(item_cache: &'a [u8], item_work_alias: Option<i16>) -> Self {
        Self {
            item_cache,
            ordinary_backup: true,
            editor_backup: None,
            item_work_alias,
            definitions: None,
            npc_records: None,
            selected_party_caster: None,
            resist_caster_level: None,
            physical_accuracy: None,
            spell_data: None,
            owners: Owners::default(),
            pending: &[],
            selected_warrior: None,
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
            definitions: None,
            npc_records: None,
            selected_party_caster: None,
            resist_caster_level: None,
            physical_accuracy: None,
            spell_data: None,
            owners: Owners::default(),
            pending: &[],
            selected_warrior: None,
        })
    }

    fn race_word(self, body: usize) -> Result<i16, RuleError> {
        self.alias_word(body, 0x32, |alias| match alias {
            Alias::AboveA5 => RuleError::Unsupported {
                action: "combat.spell",
                dependency: "warrior-indexed NPC alias above the A5 world",
            },
            _ => RuleError::Unsupported {
                action: "combat.spell",
                dependency: "warrior-indexed NPC alias outside the represented owners",
            },
        })
    }
}

/// The editor backup 0x44ecb0; `with_editor_backup` checks its size.
const EDITOR_BACKUP_BYTES: usize = 18_828;
/// The first 200 item definitions, cached after the item work records.
const ITEM_CACHE_BYTES: usize = 30_000;
