use crate::combat::spells::spell_identity;
use crate::{Phase, RuleError, Session, combat, story, typed_encounter};
use centurions_content::Content;
use centurions_content::records::{ItemDefinition, SpellInfoRecord};
use serde::{Deserialize, Serialize};

/// What a step across the land still has to do when it resumes: finish a boat step, check the
/// region at a square, return from a regional event, look up the walking header, or search for
/// secrets.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MovementContinuation {
    Boating(BoatMovement),
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

/// A boat step in progress: the tile ahead, its property and the stage the step has reached.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BoatMovement {
    /// The tile ahead of the boat.
    pub tile: i16,
    /// The terrain property of the tile ahead, which decides whether the boat can enter.
    pub property: i16,
    /// The stage the step has reached.
    pub phase: BoatPhase,
}

/// The stages of a boat step: the region check, the regional return, the header lookup and what
/// follows it, finishing the step, and the questions to board or leave the boat.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BoatPhase {
    Region { x: i16, y: i16, check: bool },
    RegionalReturn,
    Header,
    AfterHeader,
    Finish,
    Board,
    Leave,
}

/// Time still to pass: the ticks remaining, the minutes each takes and where the calendar schedule
/// stopped.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimeContinuation {
    /// Ticks of time still to pass.
    pub remaining_ticks: u8,
    /// Minutes each tick takes.
    pub minutes_per_tick: i16,
    /// The entry of the calendar schedule the scan has reached.
    pub schedule_cursor: usize,
}

/// A healing cast waiting for the player to choose its target: the caster, the item and spell used
/// and the spell's power.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MedicalContinuation {
    /// What began the cast.
    #[serde(default)]
    pub origin: MedicalOrigin,
    /// The warrior casting.
    pub actor: usize,
    /// The item used, or zero for a learned spell.
    pub item: i16,
    /// The healing spell being cast.
    pub spell: i16,
    /// The spell's power.
    pub power: i16,
}

/// What began a medical cast: a needle the warrior owns, the learned heal spell, or another owned
/// item.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum MedicalOrigin {
    /// A needle in a warrior's pack.
    #[default]
    OwnedNeedle,
    /// The heal spell a warrior has learned.
    LearnedHeal,
    /// Another item that casts a healing spell.
    OwnedItem,
}

impl MedicalContinuation {
    /// Checks that a pending medical cast is made in a battle by a warrior of the squad, with the
    /// item, spell and power its origin allows and the matching spell prepared in the battle.
    pub(crate) fn validate(&self, session: &Session, content: &Content) -> Result<(), RuleError> {
        let descriptor = match self.origin {
            MedicalOrigin::OwnedNeedle => [0, 3, 0],
            MedicalOrigin::LearnedHeal => [0, 0, 0],
            MedicalOrigin::OwnedItem => spell_identity(self.spell),
        };
        let valid_identity = match self.origin {
            MedicalOrigin::OwnedNeedle => {
                (700..=703).contains(&self.item) && self.spell == 1401 && self.power == 1
            }
            MedicalOrigin::LearnedHeal => {
                self.item == 0 && self.spell == 1101 && (1..=7).contains(&self.power)
            }
            MedicalOrigin::OwnedItem => {
                self.item > 0
                    && self.spell > 1100
                    && content
                        .items
                        .get(self.item as usize)
                        .is_some_and(|definition| {
                            ItemDefinition(definition).spell().ok() == Some(self.spell)
                        })
            }
        };
        if session.phase != Phase::Combat
            || self.actor >= session.warriors.len()
            || !valid_identity
            || session.globals.attack_count != self.power
            || session.combat.as_ref().is_none_or(|arena| {
                (match self.origin {
                    MedicalOrigin::OwnedItem => {
                        self.actor != session.char_select_new
                            || arena.active_body().is_none_or(|body| {
                                body < 0 || body >= session.warriors.len() as i16
                            })
                    }
                    _ => arena.active_body() != Some(self.actor as i16),
                }) || arena.cast_identity != descriptor
                    || {
                        let packet = &arena.spell_info;
                        packet.len() != combat::ATTACK_PACKET_BYTES
                            || match self.origin {
                                MedicalOrigin::OwnedItem => {
                                    SpellInfoRecord(packet).target_type() != 1
                                        || SpellInfoRecord(packet)
                                            .effect()
                                            .ok()
                                            .is_none_or(|effect| effect != 57)
                                }
                                _ => SpellInfoRecord(packet).effect().ok() != Some(57),
                            }
                    }
            })
        {
            return Err(RuleError::Invalid("invalid medical continuation".into()));
        }
        Ok(())
    }
}

/// The spell the player picked in the psi menu: its row, its column and its power.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PsiMenuChoice {
    /// The row of the psi menu.
    pub row: u8,
    /// The column of the psi menu.
    pub column: u8,
    /// The power of the spell picked.
    pub power: i16,
}

/// Where the player aimed a medical cast: a living point of the battlefield, a warrior's portrait,
/// or a confirmed cancel.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MedicalTarget {
    /// A square of the battlefield.
    LivingPoint { x: i16, y: i16 },
    /// A warrior's portrait.
    Portrait { warrior: usize },
    /// The player confirmed cancelling.
    ConfirmedCancel,
}

/// A running land script: the story record it was read from, the action it is at, the encounter
/// menu it repeats, and where it began.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScriptState {
    /// The story record the script runs: its header and eight actions.
    pub record: Vec<u8>,
    /// The action the script is at.
    pub cursor: u8,
    /// The event slot of the map the script came from.
    pub map_slot: usize,
    /// Whether the calendar started the script.
    #[serde(default)]
    pub scheduled: bool,
    /// The manual encounter caller decodes Items' negative script return.
    #[serde(default)]
    pub item_return: bool,
    /// The encounter menu the script repeats, if any.
    #[serde(default)]
    pub menu: Option<MenuContinuation>,
    /// Whether the event is copied into the map's event headers, at `persist_cell`, when the script
    /// ends.
    #[serde(default)]
    pub persist: bool,
    /// Where the script began.
    #[serde(default)]
    pub origin: Option<ScriptOrigin>,
    /// The encounter action that started the script, saved with the encounter
    /// record; the confiscation restore loop also counts in it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caller_action: Option<i16>,
    /// An event started by the destination-tile lookup leaves the party in place
    /// when it ends.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub chained: bool,
    /// The position the script was started with: the party cell for a walking
    /// header, 0,0 for other callers. Travel and teleport operations replace each
    /// coordinate with a nonnegative target. Save schema 59.
    #[serde(default, skip_serializing_if = "zero_cell")]
    pub registers: (i16, i16),
    /// The position the persist-event operation saved; when the script returns,
    /// the event is copied into the map header row at that position.
    #[serde(default, skip_serializing_if = "zero_cell")]
    pub persist_cell: (i16, i16),
}

/// Whether a cell is the origin, so it can be left out of a save.
fn zero_cell(cell: &(i16, i16)) -> bool {
    *cell == (0, 0)
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
            registers: (0, 0),
            persist_cell: (0, 0),
        }
    }
    /// Replaces the script's actions with a new body and starts again at the first action.
    pub(crate) fn restart_from(&mut self, replacement: &[u8]) {
        centurions_content::records::StoryRecordMut(&mut self.record).replace_body(replacement);
        self.cursor = 0;
    }
}

impl ScriptState {
    /// Whether the record is 42 bytes, the action is at most 8, the map slot is below 100 and any
    /// menu names a menu or typed encounter that exists.
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

/// Where a script began: the level, map and square of its event, so the event can be retired when
/// the script ends.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScriptOrigin {
    /// The map the event is on.
    pub map: i32,
    /// The event's column.
    pub x: i32,
    /// The event's row.
    pub y: i32,
    /// The scenario level the map is on.
    #[serde(default)]
    pub level: u8,
}

/// An encounter menu a script is repeating: the menu shown, how many shows remain, the record that
/// began it, and whether it is a typed prompt.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MenuContinuation {
    /// The menu's number.
    pub id: i16,
    /// How many more times the menu is shown; for a typed prompt, the attempts left.
    pub remaining: i16,
    /// The menu's record as it was when the script began.
    pub original: Vec<u8>,
    /// Whether the menu is a typed prompt.
    #[serde(default)]
    pub typed: bool,
    /// Branch marks kept so older saves still load. The eliminate-branch
    /// operation now writes the shared encounter header, which later loads may
    /// replace.
    #[serde(default, skip_serializing_if = "no_branches")]
    pub eliminated: u8,
}

/// Whether no branches are marked eliminated, so the mark can be left out of a save.
fn no_branches(eliminated: &u8) -> bool {
    *eliminated == 0
}
/// What the player is looking at while exploring: messages, a prologue, a boat question, a
/// dialogue, a question, an avoidance, a menu or typed prompt, the loot window, a shop, a clinic,
/// survivors to pick, a stage change, a warrior choice or a party pick, or a pause in a script.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Presentation {
    BoatQuestion {
        boarding: bool,
    },
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
    /// The clinic's introductory text returns to temple before credits are pooled.
    ClinicEntry {
        percent: i16,
    },
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
    /// Choosing party members for a script: `remaining` more members to pick,
    /// dead ones too when `unconscious`; the inverted form flips the finished
    /// pick.
    PartyPick {
        remaining: i16,
        unconscious: bool,
        invert: bool,
    },
}
