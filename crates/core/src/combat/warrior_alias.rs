//! Warrior-indexed WORDs read and written for bodies past the warrior table.
//!
//! Party records live at 0x448aa0 + body * 3138. Native code indexes NPC bodies 10.. the
//! same way, so their fields land in the memory that follows the table.
use crate::RuleError;
use crate::shop::ShopWork;
use crate::state::WarriorTable;
use centurions_content::{ITEM_BYTES, WARRIOR_BYTES, Warrior};

const WARRIOR_TABLE: usize = 0x448aa0;
/// The startup editor backup record.
const EDITOR_BACKUP: (usize, usize) = (0x44ecb0, 18_828);
/// The ITEM work record; AMMO and WEAPON work follow it.
const ITEM_WORK: (usize, usize) = (0x45363c, ITEM_BYTES);
/// loaditem 0x130afe caches definitions in banks out of logical order.
const DEFINITION_BANKS: [(usize, usize); 5] = [
    (0x4537fe, 0),
    (0x45ad2e, 400),
    (0x46225e, 200),
    (0x46978e, 600),
    (0x470cbe, 800),
];
const SHOP_WORK: (usize, usize) = (0x478288, 3002);
/// The current map record starts with its 4200-byte event headers.
const MAP_HEADER: (usize, usize) = (0x478e42, 4200);
const A5_TOP: usize = 0x479ff0;

/// The owner of a warrior-indexed WORD, with the WORD's offset inside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Alias {
    /// Party records 0..5 and the Upgrade records 6..7 (0x44d42c, 0x44e06e).
    Record {
        body: usize,
        offset: usize,
    },
    EditorBackup(usize),
    ItemWork(usize),
    Definition {
        item: usize,
        offset: usize,
    },
    ShopWork(usize),
    MapHeader(usize),
    AboveA5,
    Unrepresented,
}

pub(crate) fn locate(body: usize, field: usize) -> Alias {
    let Some(address) = body
        .checked_mul(WARRIOR_BYTES)
        .and_then(|stride| (WARRIOR_TABLE + field).checked_add(stride))
    else {
        return Alias::AboveA5;
    };
    let within = |(base, bytes): (usize, usize)| {
        address
            .checked_sub(base)
            .filter(|offset| offset + 2 <= bytes)
    };
    if let Some(offset) = within((WARRIOR_TABLE, EDITOR_BACKUP.0 - WARRIOR_TABLE)) {
        Alias::Record {
            body: offset / WARRIOR_BYTES,
            offset: offset % WARRIOR_BYTES,
        }
    } else if let Some(offset) = within(EDITOR_BACKUP) {
        Alias::EditorBackup(offset)
    } else if let Some(offset) = within(ITEM_WORK) {
        Alias::ItemWork(offset)
    } else if let Some((offset, first)) = DEFINITION_BANKS
        .into_iter()
        .find_map(|(base, first)| within((base, 200 * ITEM_BYTES)).map(|offset| (offset, first)))
    {
        Alias::Definition {
            item: first + offset / ITEM_BYTES,
            offset: offset % ITEM_BYTES,
        }
    } else if let Some(offset) = within(SHOP_WORK) {
        Alias::ShopWork(offset)
    } else if let Some(offset) = within(MAP_HEADER) {
        Alias::MapHeader(offset)
    } else if address + 2 > A5_TOP {
        Alias::AboveA5
    } else {
        Alias::Unrepresented
    }
}

/// The session's owners, borrowed field by field so other fields stay mutable.
macro_rules! session_owners {
    ($state:expr) => {
        $crate::combat::warrior_alias::Owners::new(
            $state.native_work.ammunition_work.item.as_deref(),
            &$state.native_work.shop_work,
            $state
                .exploration
                .as_ref()
                .map(|position| position.event_headers.as_slice()),
        )
        .with_table(&$state.warriors, &$state.native_work.warrior_table)
    };
}
pub(crate) use session_owners;

/// Commits a live combat's pending writes while `$combat` stays borrowed from the session.
macro_rules! commit_pending {
    ($state:expr, $combat:expr) => {
        $crate::combat::warrior_alias::commit(
            $state.native_work.ammunition_work.item.as_mut(),
            Some(&mut $state.native_work.shop_work),
            Some(&mut $state.native_work.warrior_table),
            $state
                .exploration
                .as_mut()
                .map(|position| &mut position.event_headers),
            &std::mem::take(&mut $combat.alias_writes),
        )
    };
}
pub(crate) use commit_pending;

/// The retained owners a caller passes for warrior-indexed NPC reads.
#[derive(Clone, Copy, Default)]
pub(crate) struct Owners<'a> {
    pub item_work: Option<&'a [u8]>,
    pub shop_work: Option<&'a ShopWork>,
    pub header: Option<&'a [u8]>,
    /// The squad records and the table records outside it.
    pub warriors: Option<&'a [Warrior]>,
    pub table: Option<&'a WarriorTable>,
}

impl<'a> Owners<'a> {
    pub(crate) fn new(
        item_work: Option<&'a [u8]>,
        shop_work: &'a ShopWork,
        header: Option<&'a [u8]>,
    ) -> Self {
        Self {
            item_work,
            shop_work: Some(shop_work),
            header,
            warriors: None,
            table: None,
        }
    }

    /// The table alone, for NPC-indexed callers that lend the squad elsewhere.
    pub(crate) fn with_backup(self, table: &'a WarriorTable) -> Self {
        Self {
            table: Some(table),
            ..self
        }
    }

    pub(crate) fn with_table(self, warriors: &'a [Warrior], table: &'a WarriorTable) -> Self {
        Self {
            warriors: Some(warriors),
            table: Some(table),
            ..self
        }
    }

    /// The warrior table, ITEM work, shop work or map header WORD at `alias`, if this
    /// caller passed it.
    pub(crate) fn read(self, alias: Alias) -> Option<i16> {
        let word = |bytes: &[u8], offset: usize| {
            bytes
                .get(offset..offset + 2)
                .map(|word| i16::from_be_bytes([word[0], word[1]]))
        };
        match alias {
            Alias::ItemWork(offset) => word(self.item_work?, offset),
            Alias::ShopWork(offset) => self.shop_work?.read_word(offset).ok(),
            Alias::MapHeader(offset) => word(self.header?, offset),
            Alias::Record { body, offset } => match self.warriors?.get(body) {
                Some(warrior) => word(&warrior.bytes, offset),
                None => self.table?.word(body, offset),
            },
            Alias::EditorBackup(offset) => self
                .table?
                .word(8 + offset / WARRIOR_BYTES, offset % WARRIOR_BYTES),
            _ => None,
        }
    }
}

/// A spelllist write to a warrior-indexed NPC WORD, committed by the caller in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AliasWrite {
    pub body: usize,
    pub field: usize,
    pub value: i16,
}

/// Commits recorded writes to the session's ITEM work, shop work and map header.
pub(crate) fn commit(
    item_work: Option<&mut Vec<u8>>,
    shop_work: Option<&mut ShopWork>,
    table: Option<&mut WarriorTable>,
    header: Option<&mut Vec<u8>>,
    writes: &[AliasWrite],
) -> Result<(), RuleError> {
    let (mut item_work, mut shop_work, mut table, mut header) =
        (item_work, shop_work, table, header);
    for write in writes {
        let put = |bytes: Option<&mut Vec<u8>>, offset: usize| {
            let word = bytes
                .and_then(|bytes| bytes.get_mut(offset..offset + 2))
                .ok_or_else(|| RuleError::from("warrior-indexed write owner missing"))?;
            word.copy_from_slice(&write.value.to_be_bytes());
            Ok::<_, RuleError>(())
        };
        match locate(write.body, write.field) {
            Alias::ItemWork(offset) => put(item_work.as_deref_mut(), offset)?,
            Alias::MapHeader(offset) => put(header.as_deref_mut(), offset)?,
            Alias::ShopWork(offset) => shop_work
                .as_deref_mut()
                .ok_or_else(|| RuleError::from("warrior-indexed write owner missing"))?
                .write_word(offset, write.value)?,
            // NPC bodies 10..13 index the backup; party bodies never record alias writes.
            Alias::EditorBackup(offset) => table
                .as_deref_mut()
                .ok_or_else(|| RuleError::from("warrior-indexed write owner missing"))?
                .set_word(
                    8 + offset / WARRIOR_BYTES,
                    offset % WARRIOR_BYTES,
                    write.value,
                )?,
            _ => return Err("warrior-indexed write outside a retained owner".into()),
        }
    }
    Ok(())
}

/// Owned copies of the owners for a caller that also mutates its work record.
#[derive(Clone, Default)]
pub(crate) struct Snapshot {
    item_work: Option<Vec<u8>>,
    shop_work: Option<ShopWork>,
    header: Option<Vec<u8>>,
    table: Option<WarriorTable>,
}

impl Snapshot {
    pub(crate) fn new(owners: Owners<'_>) -> Self {
        Self {
            item_work: owners.item_work.map(<[u8]>::to_vec),
            shop_work: owners.shop_work.cloned(),
            header: owners.header.map(<[u8]>::to_vec),
            table: owners.table.cloned(),
        }
    }

    pub(crate) fn owners(&self) -> Owners<'_> {
        Owners {
            item_work: self.item_work.as_deref(),
            shop_work: self.shop_work.as_ref(),
            header: self.header.as_deref(),
            warriors: None,
            table: self.table.as_ref(),
        }
    }
}

impl crate::Session {
    /// Commits a live cast's pending warrior-indexed writes to the retained owners.
    pub(crate) fn commit_alias_writes(&mut self) -> Result<(), RuleError> {
        let Some(combat) = self.combat.as_mut() else {
            return Ok(());
        };
        let writes = std::mem::take(&mut combat.alias_writes);
        self.commit_writes(&writes)
    }

    pub(crate) fn commit_writes(&mut self, writes: &[AliasWrite]) -> Result<(), RuleError> {
        commit(
            self.native_work.ammunition_work.item.as_mut(),
            Some(&mut self.native_work.shop_work),
            Some(&mut self.native_work.warrior_table),
            self.exploration
                .as_mut()
                .map(|position| &mut position.event_headers),
            writes,
        )
    }
}
