//! Warrior-indexed stamina reads made by outdoor killbody at 0x12e4e4.
use super::warrior_alias::{self, Alias, Owners};
use crate::{RuleError, Session, state::WarriorTable};
use centurions_content::{Content, Warrior, word};

const STAMINA: usize = 0x50;

/// Combat mode zero does not redirect NPC bodies to the NPC record array.
/// Resolve only the actual WORD owner; a missing owner is required only here.
pub(crate) fn health(state: &Session, content: &Content, body: i16) -> Result<i16, RuleError> {
    View::new(
        content,
        state.mission.as_deref(),
        state.scenario_level,
        state.native_work.ammunition_work.item.as_deref(),
        &state.native_work.shop_work,
        state
            .exploration
            .as_ref()
            .map(|position| position.event_headers.as_slice()),
        (&state.warriors, &state.native_work.warrior_table),
    )
    .health(body)
}

/// A lazy view over the disjoint retained owners used by NPC status reads.
#[derive(Clone, Copy)]
pub(crate) struct View<'a> {
    content: &'a Content,
    mission: Option<&'a str>,
    level: u8,
    item_work: Option<&'a [u8]>,
    shop_work: &'a crate::shop::ShopWork,
    header: Option<&'a [u8]>,
    table: (&'a [Warrior], &'a WarriorTable),
}

impl<'a> View<'a> {
    pub(crate) fn new(
        content: &'a Content,
        mission: Option<&'a str>,
        level: u8,
        item_work: Option<&'a [u8]>,
        shop_work: &'a crate::shop::ShopWork,
        header: Option<&'a [u8]>,
        table: (&'a [Warrior], &'a WarriorTable),
    ) -> Self {
        Self {
            content,
            mission,
            level,
            item_work,
            shop_work,
            header,
            table,
        }
    }

    /// The warrior table record of a body past the squad, zero when nothing wrote it.
    pub(crate) fn held_record(self, body: usize) -> Option<Warrior> {
        self.table.1.bytes(body).map(|bytes| Warrior {
            name: String::new(),
            original_sha256: String::new(),
            bytes,
        })
    }

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
            Alias::ItemWork(offset) => read_alias_word(
                self.item_work
                    .ok_or_else(|| RuleError::from("ITEM work missing"))?,
                offset,
                "outdoor death ITEM stamina alias",
            ),
            Alias::Definition { item, offset } => self.definition_health(item, offset),
            Alias::ShopWork(offset) => self.shop_work.read_word(offset),
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

fn definition_word(content: &Content, item: usize, offset: usize) -> Result<i16, RuleError> {
    read_alias_word(
        content
            .items
            .get(item)
            // The 800..999 bank keeps rows a shorter scenario item file leaves unwritten.
            .ok_or_else(|| {
                unavailable("outdoor death alias past the scenario's loaded item records")
            })?,
        offset,
        "outdoor death definition cache stamina alias",
    )
}

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

fn unavailable(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.outdoor_health",
        dependency,
    }
}
