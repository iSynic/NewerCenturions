//! Which packets the shared resolver handles, and the race and caster reads its
//! saving throws make.
use super::RuleError;
use crate::combat::Combat;
use crate::combat::warrior_alias::{self, Alias, AliasWrite, Owners};
use centurions_content::{
    ITEM_BYTES, Warrior,
    combat::MonsterRecord,
    records::{EnemyRecord, ItemDefinition, SpellInfoRecord},
    word,
};

/// Whether a packet combines damage with a condition or healing effect the shared
/// resolver knows.
pub(crate) fn is_support(packet: &[u8]) -> Result<bool, RuleError> {
    if packet.len() != 58 {
        return Err(RuleError::Invalid("spell packet size".into()));
    }
    let attack = SpellInfoRecord(packet);
    let effect = attack.effect()?.wrapping_abs();
    // A category-5 Psi drain has its own resolver.
    let drain = matches!(effect, 60 | 61) && attack.defense_category() != 5;
    // Effects without an arm of their own apply damage and report only.
    let generic = matches!(effect, 42..=48 | 50 | 62 | 63 | 65 | 67 | 69 | 71..=89 | 91..=99);
    Ok(drain
        || generic
        || matches!(effect, 0..=41 | 49 | 51..=54 | 57 | 59 | 64 | 66 | 68 | 70 | 90 | 100..))
}

/// Item work's restriction word, which is distinct from its field 32.
pub(crate) fn loaded_item_restriction(
    loaded: &crate::ammunition::LoadedItems,
) -> Result<Option<i16>, RuleError> {
    loaded
        .item
        .as_deref()
        .map(|item| {
            ItemDefinition(item)
                .restriction_alias()
                .map_err(RuleError::from)
        })
        .transpose()
}

/// Only non-support packets reach this. Every one is a movement or summon arm, a
/// category-5 drain or direct weapon damage, so monster selection admits each.
pub(super) fn validate_direct(packet: &[u8]) -> Result<(), RuleError> {
    if packet.len() != 58 {
        return Err(RuleError::Invalid("spell packet size".into()));
    }
    Ok(())
}

/// The race exclusion bit uses the signed race, so races outside 1..16 read other
/// packet bytes rather than a party race table.
///
/// Original: resist
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

/// The attack packet is followed directly by the scenario's spell table, so bits
/// past the packet read that table.
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

/// What a spell's race restrictions and resist rolls read of the party and the items beside the
/// battle: the item definitions, the selected warrior, the caster's level and accuracy, the
/// scenario spell table, and earlier writes of the same command.
#[derive(Clone, Copy)]
pub struct RestrictionMemoryView<'a> {
    item_cache: &'a [u8],
    ordinary_backup: bool,
    editor_backup: Option<&'a [u8]>,
    item_restriction: Option<i16>,
    definitions: Option<&'a [Vec<u8>]>,
    monster_records: Option<&'a [bool]>,
    selected_party_caster: Option<i16>,
    resist_caster_level: Option<i16>,
    physical_accuracy: Option<(i16, i16)>,
    /// The scenario spell table (Data S).
    spell_data: Option<&'a [u8]>,
    /// Item work, shop work and the map header for warrior-indexed monster fields.
    owners: Owners<'a>,
    /// Earlier writes of the same command that have not reached those owners yet.
    pending: &'a [AliasWrite],
    /// The selected warrior (`charup`), which effect 90 reads whatever body casts.
    char_up: Option<i16>,
}

/// Whether the spell's race restriction spares this monster, by its race word; unsupported without
/// the party memory.
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
    /// The party caster's faction and transfer come from `charup`, independently of
    /// the battle order's current body used for reflection.
    pub(crate) fn with_selected_party_caster(mut self, selected: i16) -> Self {
        self.selected_party_caster = Some(selected);
        self
    }

    /// The warrior selected as the party's caster, if one was given.
    pub(in crate::combat::spells) fn selected_party_caster(self) -> Option<i16> {
        self.selected_party_caster
    }

    /// Conversion reads the level and faction of `charup` for any caster.
    pub(crate) fn with_char_up(mut self, selected: i16) -> Self {
        self.char_up = Some(selected);
        self
    }

    /// The selected warrior whose level and faction a conversion reads; the party's caster when
    /// none is set.
    pub(in crate::combat::spells) fn char_up(self) -> Option<i16> {
        self.char_up.or(self.selected_party_caster)
    }

    /// The save loses five times the caster's level: the level of `charup` when the current body is
    /// below 9, otherwise the hit dice of `monsterup`.
    ///
    /// Original: resist
    pub(crate) fn with_resist_caster(
        mut self,
        current: Option<i16>,
        (warrior, monster): (usize, usize),
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
                enemies.get(monster)?.record().hit_dice().ok()
            }
        });
        self
    }

    /// The category-9 gate starts at 33 plus the current body's accuracy; body 9 and
    /// empty entries add nothing. Row 110 of the battle order is the cursor `up`
    /// itself, which indexes monster record 100, the monster template (`monster`).
    ///
    /// Original: resist
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
                    // Missile accuracy of a warrior-table record past the squad.
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
                            .hit_dice()
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

    /// A warrior-table field through its owner; the editor backup reads zero.
    fn table_word(self, body: usize, field: usize) -> Option<i16> {
        self.alias_word(body, field, |_| RuleError::Invalid(String::new()))
            .ok()
    }

    /// In live combat getup sets `charup` or `monsterup` from the current body.
    pub(crate) fn with_live_caster(self, combat: &Combat, warriors: &[Warrior]) -> Self {
        let enemies = &combat.roster.records;
        let current = combat.active_body();
        let this = self.with_current_accuracy(
            Some(
                combat
                    .battle_order
                    .get(combat.battle_order_index)
                    .copied()
                    .unwrap_or(combat.battle_order_index as i16),
            ),
            (warriors, enemies, combat.monster_template()),
            combat.difficulty,
        );
        // A party turn leaves the current body in `charup`.
        let party = current.filter(|body| *body < 9).or(this.char_up());
        this.with_live_resist_caster(combat, warriors, (party, None))
    }

    /// The caster level comes from the battle order's current body (row 110 is the
    /// cursor itself): below 9 the level of `charup` (`party`), otherwise that of
    /// `monsterup` (`monster`), which getup sets on a monster turn.
    ///
    /// Original: resist
    pub(crate) fn with_live_resist_caster(
        mut self,
        combat: &Combat,
        warriors: &[Warrior],
        (party, monster): (Option<i16>, Option<usize>),
    ) -> Self {
        let current = combat
            .battle_order
            .get(combat.battle_order_index)
            .copied()
            .unwrap_or(combat.battle_order_index as i16);
        self.resist_caster_level = if current < 9 {
            party
                .and_then(|warrior| warriors.get(usize::try_from(warrior).ok()?))
                .and_then(|record| record.record().level().ok())
        } else {
            let monster = monster.unwrap_or(if current >= 10 {
                current as usize - 10
            } else {
                combat.monster_up
            });
            combat
                .story_record(monster)
                .ok()
                .and_then(|record| record.record().hit_dice().ok())
        };
        self
    }

    /// The current body's accuracy and the category-9 gate, if worked out.
    pub(super) fn physical_accuracy(self) -> Option<(i16, i16)> {
        self.physical_accuracy
    }

    /// The level whose five-times value the target's save loses, if worked out.
    pub(super) fn resist_caster_level(self) -> Option<i16> {
        self.resist_caster_level
    }

    /// Reads the scenario spell table from the story tables.
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

    /// Reads item definitions from the given list.
    pub(crate) fn with_definitions(mut self, definitions: &'a [Vec<u8>]) -> Self {
        self.definitions = Some(definitions);
        self
    }

    /// Says which monster records exist.
    pub(crate) fn with_monster_records(mut self, known: &'a [bool]) -> Self {
        self.monster_records = Some(known);
        self
    }

    /// A monster whose record was never set up is unsupported.
    pub(in crate::combat::spells) fn require_monster(self, body: usize) -> Result<(), RuleError> {
        if body >= 10
            && self
                .monster_records
                .is_some_and(|known| !known.get(body - 10).copied().unwrap_or(false))
        {
            return Err(RuleError::Unsupported {
                action: "combat.retained_npc",
                dependency: "inactive NPC record absent from legacy context",
            });
        }
        Ok(())
    }

    /// Lends the records that warrior-indexed monster fields live in.
    pub(crate) fn with_alias_owners<'b>(self, owners: Owners<'b>) -> RestrictionMemoryView<'b>
    where
        'a: 'b,
    {
        RestrictionMemoryView { owners, ..self }
    }

    /// Adds the writes of the same command that have not yet reached those records.
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

    /// A warrior-indexed field of any body through the owner of that memory.
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
            // Item work's race word is the held item alias.
            Alias::LoadedItem(0x32) if self.owners.loaded_item.is_none() => self.item_restriction,
            Alias::Definition { item, offset } if item < 200 => (self.item_cache.len()
                == ITEM_CACHE_BYTES)
                .then(|| word(self.item_cache, item * ITEM_BYTES + offset))
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

    /// A view with only the item definitions cached.
    pub fn item_cache_only(item_cache: &'a [u8]) -> Self {
        Self {
            item_cache,
            ordinary_backup: false,
            editor_backup: None,
            item_restriction: None,
            definitions: None,
            monster_records: None,
            selected_party_caster: None,
            resist_caster_level: None,
            physical_accuracy: None,
            spell_data: None,
            owners: Owners::default(),
            pending: &[],
            char_up: None,
        }
    }

    /// Registered editing never writes the demo backup, so only alias writes change
    /// it; a caller without the session table reads startup zeros. Captured demo
    /// histories use `with_editor_backup`. `item_restriction` is item work's race
    /// word, distinct from its field 32.
    pub fn ordinary_startup(item_cache: &'a [u8], item_restriction: Option<i16>) -> Self {
        Self {
            item_cache,
            ordinary_backup: true,
            editor_backup: None,
            item_restriction,
            definitions: None,
            monster_records: None,
            selected_party_caster: None,
            resist_caster_level: None,
            physical_accuracy: None,
            spell_data: None,
            owners: Owners::default(),
            pending: &[],
            char_up: None,
        }
    }

    /// A view for an editing session's captured party memory; the backup record must be the right
    /// size.
    pub fn with_editor_backup(
        item_cache: &'a [u8],
        item_restriction: Option<i16>,
        backup: &'a [u8],
    ) -> Result<Self, RuleError> {
        if backup.len() != EDITOR_BACKUP_BYTES {
            return Err(RuleError::Invalid("editor backup record size".into()));
        }
        Ok(Self {
            item_cache,
            ordinary_backup: false,
            editor_backup: Some(backup),
            item_restriction,
            definitions: None,
            monster_records: None,
            selected_party_caster: None,
            resist_caster_level: None,
            physical_accuracy: None,
            spell_data: None,
            owners: Owners::default(),
            pending: &[],
            char_up: None,
        })
    }

    /// The race word of a body, read through the owner of that memory.
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

/// The editor backup record's size, which `with_editor_backup` checks.
const EDITOR_BACKUP_BYTES: usize = 18_828;
/// The first 200 item definitions, cached after the item work records.
const ITEM_CACHE_BYTES: usize = 30_000;
