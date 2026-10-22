use crate::{story, typed_encounter};
use centurions_content::Content;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MovementContinuation {
    RegionalReturn {
        header: bool,
    },
    Region {
        x: i16,
        y: i16,
        check: bool,
        header: bool,
    },
    Header,
    Secrets,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimeContinuation {
    pub remaining_ticks: u8,
    pub minutes_per_tick: i16,
    pub schedule_cursor: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MedicalContinuation {
    #[serde(default)]
    pub origin: MedicalOrigin,
    pub actor: usize,
    pub item: i16,
    pub spell: i16,
    pub power: i16,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum MedicalOrigin {
    #[default]
    OwnedNeedle,
    LearnedHeal,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PsiMenuChoice {
    pub row: u8,
    pub column: u8,
    pub power: i16,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MedicalTarget {
    LivingPoint { x: i16, y: i16 },
    Portrait { warrior: usize },
    ConfirmedCancel,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScriptState {
    pub record: Vec<u8>,
    pub cursor: u8,
    pub map_slot: usize,
    #[serde(default)]
    pub scheduled: bool,
    /// The manual encounter caller decodes Items' negative script return.
    #[serde(default)]
    pub item_return: bool,
    #[serde(default)]
    pub menu: Option<MenuContinuation>,
    #[serde(default)]
    pub persist: bool,
    #[serde(default)]
    pub origin: Option<ScriptOrigin>,
    /// newland's encounter-caller action (local_4a), saved with the encounter
    /// record; opcode 36's restore loop also counts in it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caller_action: Option<i16>,
    /// An event started by the destination-tile lookup leaves the party in place
    /// when it ends (newland's bVar6).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub chained: bool,
}

impl ScriptState {
    /// A scheduled story run from `cursor` in map slot 0, without menu or origin.
    pub fn scheduled(record: Vec<u8>, cursor: u8) -> Self {
        Self {
            record,
            cursor,
            map_slot: 0,
            scheduled: true,
            item_return: false,
            menu: None,
            persist: false,
            origin: None,
            caller_action: None,
            chained: false,
        }
    }
    pub(crate) fn restart_from(&mut self, replacement: &[u8]) {
        centurions_content::records::StoryRecordMut(&mut self.record).replace_body(replacement);
        self.cursor = 0;
    }
}

impl ScriptState {
    pub(crate) fn valid(&self, content: &Content) -> bool {
        self.record.len() == 42
            && self.cursor <= 8
            && self.map_slot < 100
            && self.menu.as_ref().is_none_or(|menu| {
                menu.original.len() == 42
                    && menu.id >= 0
                    && content.map(0).is_some_and(|map| {
                        if menu.typed {
                            (menu.id as usize + 1) * typed_encounter::RECORD_BYTES
                                <= map.typed_encounters.len()
                        } else {
                            (menu.id as usize + 1) * story::MENU_RECORD_BYTES <= map.menus.len()
                        }
                    })
            })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScriptOrigin {
    pub map: i32,
    pub x: i32,
    pub y: i32,
    #[serde(default)]
    pub level: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MenuContinuation {
    pub id: i16,
    pub remaining: i16,
    pub original: Vec<u8>,
    #[serde(default)]
    pub typed: bool,
    /// Complex-encounter branches newland44 eliminated in the loaded buffer, one bit
    /// per branch; saveencounter writes them to the table when the encounter is left.
    #[serde(default, skip_serializing_if = "no_branches")]
    pub eliminated: u8,
}

fn no_branches(eliminated: &u8) -> bool {
    *eliminated == 0
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Presentation {
    ScriptPause,
    Messages {
        pages: Vec<(i16, String)>,
        next: Option<Box<Presentation>>,
    },
    Prologue {
        resource_id: i16,
    },
    Dialogue {
        id: i16,
        text: String,
    },
    Question {
        extra: i16,
        text: String,
    },
    Avoidance {
        region: u8,
        text: String,
    },
    Menu {
        id: i16,
        text: String,
        labels: Vec<String>,
    },
    Typed {
        id: i16,
        text: String,
    },
    Loot,
    Shop,
    Clinic {
        member: usize,
        percent: i16,
    },
    Survivors,
    StageChange {
        parameter: i16,
    },
    WarriorChoice {
        extra: i16,
    },
    /// getchoice for newland14 and -14: `remaining` more members to pick, dead ones
    /// too when `unconscious`; -14 inverts the finished pick mask.
    PartyPick {
        remaining: i16,
        unconscious: bool,
        invert: bool,
    },
}
