//! The stamina killbody reads outside battle, where every body indexes the warrior
//! table.
use super::warrior_alias::{self, Alias, Owners};
use crate::{RuleError, Session, state::WarriorTable};
use centurions_content::{Content, Warrior, word};

use centurions_content::records::warrior_layout::STAMINA;

/// A body's stamina outside battle. Monster bodies still index the warrior table,
/// so they read whatever lies past it.
///
/// Original: killbody
pub(crate) fn health(state: &Session, content: &Content, body: i16) -> Result<i16, RuleError> {
    View::new(
        content,
        state.mission.as_deref(),
        state.scenario_level,
        state.globals.loaded_items.item.as_deref(),
        &state.globals.the_shop,
        state
            .exploration
            .as_ref()
            .map(|position| position.event_headers.as_slice()),
        (&state.warriors, &state.globals.warrior_table),
    )
    .health(body)
}

/// The records past the warrior table that a monster body's reads land in.
#[derive(Clone, Copy)]
pub(crate) struct View<'a> {
    content: &'a Content,
    mission: Option<&'a str>,
    level: u8,
    loaded_item: Option<&'a [u8]>,
    the_shop: &'a crate::shop::TheShop,
    header: Option<&'a [u8]>,
    table: (&'a [Warrior], &'a WarriorTable),
}

impl<'a> View<'a> {
    /// Gathers the records a body's stamina can land in.
    pub(crate) fn new(
        content: &'a Content,
        mission: Option<&'a str>,
        level: u8,
        loaded_item: Option<&'a [u8]>,
        the_shop: &'a crate::shop::TheShop,
        header: Option<&'a [u8]>,
        table: (&'a [Warrior], &'a WarriorTable),
    ) -> Self {
        Self {
            content,
            mission,
            level,
            loaded_item,
            the_shop,
            header,
            table,
        }
    }

    /// The warrior table record of a body past the squad, zero when nothing wrote it.
    pub(crate) fn scratch_character(self, body: usize) -> Option<Warrior> {
        self.table.1.bytes(body).map(|bytes| Warrior {
            name: String::new(),
            original_sha256: String::new(),
            bytes,
        })
    }

    /// The stamina of a body outside battle, from whichever record its index lands in.
    pub(crate) fn health(self, body: i16) -> Result<i16, RuleError> {
        let alias = usize::try_from(body)
            .ok()
            .filter(|body| *body < 110)
            .map_or(Alias::Unrepresented, |body| {
                warrior_alias::locate(body, STAMINA)
            });
        match alias {
            Alias::Record { .. } | Alias::EditorBackup(_) => Owners {
                warriors: Some(self.table.0),
                table: Some(self.table.1),
                ..Owners::default()
            }
            .read(alias)
            .ok_or_else(|| RuleError::from("warrior table stamina alias missing")),
            Alias::LoadedItem(offset) => read_alias_word(
                self.loaded_item
                    .ok_or_else(|| RuleError::from("ITEM work missing"))?,
                offset,
                "outdoor death ITEM stamina alias",
            ),
            Alias::Definition { item, offset } => self.definition_health(item, offset),
            Alias::TheShop(offset) => self.the_shop.read_word(offset),
            Alias::MapHeader(offset) => read_alias_word(
                self.header
                    .ok_or_else(|| RuleError::from("outdoor death outside a mission"))?,
                offset,
                "outdoor death current map header stamina alias",
            ),
            Alias::AboveA5 => Err(unavailable(
                "outdoor death stamina alias beyond native A5 allocation",
            )),
            Alias::Unrepresented => Err(unavailable(
                "outdoor death unrepresented warrior-storage alias",
            )),
        }
    }

    /// The stamina read from an item definition record.
    fn definition_health(self, item: usize, offset: usize) -> Result<i16, RuleError> {
        if item >= 800 {
            let mission = self
                .mission
                .ok_or_else(|| RuleError::from("outdoor death outside a mission"))?;
            let context = self
                .content
                .context(Some(mission), self.level)
                .map_err(|_| RuleError::from("outdoor death outside a mission"))?;
            return definition_word(&context, item, offset);
        }
        definition_word(self.content, item, offset)
    }
}

/// The word at the offset of an item definition; a definition the scenario did not load is not
/// supported.
fn definition_word(content: &Content, item: usize, offset: usize) -> Result<i16, RuleError> {
    read_alias_word(
        content
            .items
            .get(item)
            // Item definitions 800..999 keep rows a shorter scenario item file
            // leaves unwritten.
            .ok_or_else(|| {
                unavailable("outdoor death alias past the scenario's loaded item records")
            })?,
        offset,
        "outdoor death definition cache stamina alias",
    )
}

/// Reads a word from a record, or reports it as unavailable.
fn read_alias_word(
    bytes: &[u8],
    offset: usize,
    dependency: &'static str,
) -> Result<i16, RuleError> {
    word(bytes, offset).map_err(|_| RuleError::Unsupported {
        action: "combat.outdoor_health",
        dependency,
    })
}

/// Reports a stamina this build cannot read, without changing anything.
fn unavailable(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.outdoor_health",
        dependency,
    }
}
