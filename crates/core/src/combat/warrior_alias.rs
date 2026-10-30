//! Warrior fields read and written for bodies past the warrior table.
//!
//! Some routines index a monster body as if it were a warrior, so its fields land in
//! the records that follow the warrior table (see docs/source-notes/combat.md).
use crate::RuleError;
use crate::shop::TheShop;
use crate::state::WarriorTable;
use centurions_content::{ITEM_BYTES, WARRIOR_BYTES, Warrior};

/// Where the warrior table starts; a body number counts warrior-sized records from here.
const WARRIOR_TABLE: usize = 0x448aa0;
/// The startup editor backup record.
const EDITOR_BACKUP: (usize, usize) = (0x44ecb0, 18_828);
/// The ITEM work record; AMMO and WEAPON work follow it.
const LOADED_ITEM: (usize, usize) = (0x45363c, ITEM_BYTES);
/// loaditem keeps item definitions in banks out of numeric order.
const DEFINITION_BANKS: [(usize, usize); 5] = [
    (0x4537fe, 0),
    (0x45ad2e, 400),
    (0x46225e, 200),
    (0x46978e, 600),
    (0x470cbe, 800),
];
/// The shop's work record.
const THE_SHOP: (usize, usize) = (0x478288, 3002);
/// The current map record starts with its 4200-byte event headers.
const MAP_HEADER: (usize, usize) = (0x478e42, 4200);
/// The top of the memory the aliases can land in.
const A5_TOP: usize = 0x479ff0;

/// The record a warrior-indexed field lands in, with its offset there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Alias {
    /// Party records 0..5 and the Upgrade records 6..7.
    Record {
        body: usize,
        offset: usize,
    },
    EditorBackup(usize),
    LoadedItem(usize),
    Definition {
        item: usize,
        offset: usize,
    },
    TheShop(usize),
    MapHeader(usize),
    AboveA5,
    Unrepresented,
}

/// Finds the record that a field of the given body number lands in, counting warrior-sized records
/// from the start of the warrior table.
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
    } else if let Some(offset) = within(LOADED_ITEM) {
        Alias::LoadedItem(offset)
    } else if let Some((offset, first)) = DEFINITION_BANKS
        .into_iter()
        .find_map(|(base, first)| within((base, 200 * ITEM_BYTES)).map(|offset| (offset, first)))
    {
        Alias::Definition {
            item: first + offset / ITEM_BYTES,
            offset: offset % ITEM_BYTES,
        }
    } else if let Some(offset) = within(THE_SHOP) {
        Alias::TheShop(offset)
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
            $state.globals.loaded_items.item.as_deref(),
            &$state.globals.the_shop,
            $state
                .exploration
                .as_ref()
                .map(|position| position.event_headers.as_slice()),
        )
        .with_table(&$state.warriors, &$state.globals.warrior_table)
    };
}
pub(crate) use session_owners;

/// Owners backed by the warrior backup table, borrowing only the session fields read;
/// the item alias defaults to none.
macro_rules! backup_owners {
    ($state:expr) => {
        $crate::combat::warrior_alias::backup_owners!($state, None)
    };
    ($state:expr, $item:expr) => {
        $crate::combat::warrior_alias::Owners::new(
            $item,
            &$state.globals.the_shop,
            $state
                .exploration
                .as_ref()
                .map(|position| position.event_headers.as_slice()),
        )
        .with_backup(&$state.globals.warrior_table)
    };
}
pub(crate) use backup_owners;

/// Commits a live combat's pending writes while `$combat` stays borrowed from the session.
macro_rules! commit_pending {
    ($state:expr, $combat:expr) => {
        $crate::combat::warrior_alias::commit(
            $state.globals.loaded_items.item.as_mut(),
            Some(&mut $state.globals.the_shop),
            Some(&mut $state.globals.warrior_table),
            $state
                .exploration
                .as_mut()
                .map(|position| &mut position.event_headers),
            &std::mem::take(&mut $combat.alias_writes),
        )
    };
}
pub(crate) use commit_pending;

/// The retained owners a caller passes for warrior-indexed monster reads.
#[derive(Clone, Copy, Default)]
pub(crate) struct Owners<'a> {
    pub loaded_item: Option<&'a [u8]>,
    pub the_shop: Option<&'a TheShop>,
    pub header: Option<&'a [u8]>,
    /// The squad records and the table records outside it.
    pub warriors: Option<&'a [Warrior]>,
    pub table: Option<&'a WarriorTable>,
}

impl<'a> Owners<'a> {
    /// The owners a caller holds: item work, the shop and the map header.
    pub(crate) fn new(
        loaded_item: Option<&'a [u8]>,
        the_shop: &'a TheShop,
        header: Option<&'a [u8]>,
    ) -> Self {
        Self {
            loaded_item,
            the_shop: Some(the_shop),
            header,
            warriors: None,
            table: None,
        }
    }

    /// The table alone, for monster-indexed callers that lend the squad elsewhere.
    pub(crate) fn with_backup(self, table: &'a WarriorTable) -> Self {
        Self {
            table: Some(table),
            ..self
        }
    }

    /// These owners with the squad's warriors and the warrior table added.
    pub(crate) fn with_table(self, warriors: &'a [Warrior], table: &'a WarriorTable) -> Self {
        Self {
            warriors: Some(warriors),
            table: Some(table),
            ..self
        }
    }

    /// The value at `alias` in the warrior table, item work, shop work or map header,
    /// if this caller passed it.
    pub(crate) fn read(self, alias: Alias) -> Option<i16> {
        let word = |bytes: &[u8], offset: usize| {
            bytes
                .get(offset..offset + 2)
                .map(|word| i16::from_be_bytes([word[0], word[1]]))
        };
        match alias {
            Alias::LoadedItem(offset) => word(self.loaded_item?, offset),
            Alias::TheShop(offset) => self.the_shop?.read_word(offset).ok(),
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

/// A spelllist write to a monster field past the warrior table, committed by the
/// caller in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AliasWrite {
    pub body: usize,
    pub field: usize,
    pub value: i16,
}

/// Commits recorded writes to the session's item work, shop work and map header.
pub(crate) fn commit(
    loaded_item: Option<&mut Vec<u8>>,
    the_shop: Option<&mut TheShop>,
    table: Option<&mut WarriorTable>,
    header: Option<&mut Vec<u8>>,
    writes: &[AliasWrite],
) -> Result<(), RuleError> {
    let (mut loaded_item, mut the_shop, mut table, mut header) =
        (loaded_item, the_shop, table, header);
    for write in writes {
        let put = |bytes: Option<&mut Vec<u8>>, offset: usize| {
            let word = bytes
                .and_then(|bytes| bytes.get_mut(offset..offset + 2))
                .ok_or_else(|| RuleError::from("warrior-indexed write owner missing"))?;
            word.copy_from_slice(&write.value.to_be_bytes());
            Ok::<_, RuleError>(())
        };
        match locate(write.body, write.field) {
            Alias::LoadedItem(offset) => put(loaded_item.as_deref_mut(), offset)?,
            Alias::MapHeader(offset) => put(header.as_deref_mut(), offset)?,
            Alias::TheShop(offset) => the_shop
                .as_deref_mut()
                .ok_or_else(|| RuleError::from("warrior-indexed write owner missing"))?
                .write_word(offset, write.value)?,
            // Monster bodies 10..13 land in the editor backup; warriors never get here.
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
    loaded_item: Option<Vec<u8>>,
    the_shop: Option<TheShop>,
    header: Option<Vec<u8>>,
    table: Option<WarriorTable>,
}

impl Snapshot {
    /// Copies the owners so a caller can change its own record meanwhile.
    pub(crate) fn new(owners: Owners<'_>) -> Self {
        Self {
            loaded_item: owners.loaded_item.map(<[u8]>::to_vec),
            the_shop: owners.the_shop.cloned(),
            header: owners.header.map(<[u8]>::to_vec),
            table: owners.table.cloned(),
        }
    }

    /// The owners as the copy holds them.
    pub(crate) fn owners(&self) -> Owners<'_> {
        Owners {
            loaded_item: self.loaded_item.as_deref(),
            the_shop: self.the_shop.as_ref(),
            header: self.header.as_deref(),
            warriors: None,
            table: self.table.as_ref(),
        }
    }
}

impl crate::Session {
    /// Commits a cast's pending warrior-indexed writes.
    pub(crate) fn commit_alias_writes(&mut self) -> Result<(), RuleError> {
        let Some(combat) = self.combat.as_mut() else {
            return Ok(());
        };
        let writes = std::mem::take(&mut combat.alias_writes);
        self.commit_writes(&writes)
    }

    /// Commits pending warrior-indexed writes to the records they land in.
    pub(crate) fn commit_writes(&mut self, writes: &[AliasWrite]) -> Result<(), RuleError> {
        commit(
            self.globals.loaded_items.item.as_mut(),
            Some(&mut self.globals.the_shop),
            Some(&mut self.globals.warrior_table),
            self.exploration
                .as_mut()
                .map(|position| &mut position.event_headers),
            writes,
        )
    }
}
