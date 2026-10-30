use crate::{Command, RuleError, Session};
use centurions_content::Warrior;
use centurions_content::combat::MONSTER_BYTES;
use serde::{Deserialize, Serialize};

pub mod allies;
pub mod ally_turns;
pub mod attack;
pub mod capture;
pub mod controls;
pub mod damage;
pub mod death_scripts;
pub mod enemy;
pub mod enemy_weapons;
pub mod entry_party;
pub(crate) mod estimates;
pub mod final_turn_stories;
pub mod firearm;
pub mod footprint;
pub mod interaction;
pub(crate) mod item_activation;
pub mod lifecycle;
pub mod map_alias;
pub mod medical;
pub mod mission_deaths;
pub mod monster;
pub(crate) mod monster_weapons;
pub mod natural_conditions;
pub use runtime::monster_turn_flags;
pub use runtime::neighbor_scan;
mod occupied_cell;
pub(crate) mod outdoor_health;
pub mod party_buffs;
pub mod player_psi;
pub use player_psi::selection as psi_selection;
pub mod globals;
mod record_state;
mod records;
pub mod retreat;
mod reward_mode;
pub mod rewards;
pub mod round_stories;
pub use runtime::round_scripts;
pub mod runtime;
pub mod shooting;
pub use spells::conditions as spell_conditions;
pub mod spells;
pub use targeting::target_picks;
pub mod battlefield;
pub mod targeting;
pub use runtime::kill_counts;
pub mod warrior_alias;
mod warrior_attack;
pub use firearm::weapon_traits;

/// A battle in progress: the field, the squad's places, the monsters and their positions, the
/// battle order, and the values the battle keeps while it runs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Combat {
    #[serde(default)]
    #[serde(rename = "victory_work")]
    pub kill_counts: kill_counts::KillCounts,
    #[serde(default)]
    #[serde(rename = "neighbor_work")]
    pub neighbors: neighbor_scan::Neighbors,
    /// The battle's `in_spell`, copied back to the session when the battle ends.
    #[serde(default)]
    #[serde(rename = "ready")]
    pub in_spell: i16,
    #[serde(default, skip_serializing_if = "crate::normal_difficulty")]
    pub difficulty: i16,
    pub battle: i16,
    #[serde(rename = "map")]
    pub field: CombatField,
    pub squad: SquadPlacement,
    /// The monsters on the battle roster and their positions.
    #[serde(rename = "enemies")]
    pub roster: EnemySetup,
    #[serde(default)]
    #[serde(rename = "retained_npcs")]
    pub monsters: globals::Monsters,
    #[serde(default)]
    pub direct_deaths: bool,
    /// A support spell's fatigue reset (effect 68) within this command; the
    /// command boundary stores it.
    #[serde(skip)]
    pub fatigue_reset: bool,
    /// Writes by effects 26 and 64 that index monsters by warrior number, waiting to
    /// be applied to the records they land in.
    #[serde(skip)]
    pub alias_writes: Vec<warrior_alias::AliasWrite>,
    /// The caller resolvespell sees while an item uses the shared cast and ray
    /// rules; restored before any command ends or the game is saved.
    #[serde(skip)]
    pub item_condition_caller: Option<i16>,
    /// Set when this command copied or built a monster record.
    #[serde(skip)]
    #[serde(rename = "monster_work_written")]
    pub monster_pick_written: bool,
    /// The monster record lent to the current command; never saved.
    #[serde(skip)]
    pub monster_template: Option<Vec<u8>>,
    /// The attack's own `did_attack` write, applied to the shared value before
    /// script callbacks.
    #[serde(skip)]
    pub did_attack_write: Option<i16>,
    #[serde(default)]
    pub direct_death: Option<(i16, death_scripts::DeathEntry)>,
    #[serde(rename = "order")]
    pub battle_order: Vec<i16>,
    #[serde(rename = "cursor")]
    pub battle_order_index: usize,
    #[serde(rename = "round")]
    pub combat_round: i16,
    /// Rounds since the last attack or cast (`blank_round`); ten such rounds
    /// force a retreat.
    #[serde(default)]
    #[serde(rename = "inactive_rounds")]
    pub blank_round: i16,
    #[serde(default)]
    pub selection_pending: bool,
    /// The monster whose script getup last ran; a monster's script death does not
    /// reset it.
    #[serde(default)]
    #[serde(rename = "script_enemy")]
    pub macro_monster: usize,
    /// The monster whose turn getup started, set apart from `macro_monster`.
    #[serde(default)]
    #[serde(rename = "selected_enemy")]
    pub monster_up: usize,
    #[serde(default)]
    #[serde(rename = "round_scripts")]
    pub round_macros: round_stories::RoundMacros,
    #[serde(default)]
    #[serde(rename = "death_queue")]
    pub todo_queue: death_scripts::TodoQueue,
    #[serde(default = "empty_spell_queue")]
    #[serde(rename = "spell_fields")]
    pub spell_queue: Vec<u8>,
    #[serde(default = "empty_collide_check")]
    #[serde(rename = "field_used")]
    pub collide_check: Vec<u8>,
    #[serde(default = "spell_info_startup")]
    #[serde(rename = "attack_packet")]
    pub spell_info: Vec<u8>,
    #[serde(default)]
    #[serde(rename = "attack_identity")]
    pub cast_identity: [i16; 3],
    #[serde(default)]
    #[serde(rename = "weapon_work")]
    pub weapon_traits: weapon_traits::WeaponTraits,
    #[serde(default)]
    #[serde(rename = "unavailable")]
    pub kill_party: i8,
    /// The last body killbody handled, passed to the session before its script
    /// runs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending_last_defeated: Option<i16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firearm_selection: Option<shooting::Selection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capture_actor: Option<i16>,
    #[serde(default, skip_serializing_if = "automatic_off")]
    #[serde(rename = "automatic")]
    pub do_auto: [bool; 6],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_origin: Option<(i16, i16)>,
    /// Entries 6..109 of the spell target list; entries 0..5 stay in the
    /// session's `party_track`.
    #[serde(default = "track_tail_startup")]
    #[serde(rename = "target_tail")]
    pub track_tail: Vec<i16>,
    /// The battle's target selections (save schema 60).
    #[serde(default, skip_serializing_if = "target_picks::TargetPicks::is_startup")]
    #[serde(rename = "target_work")]
    pub target_picks: target_picks::TargetPicks,
}

/// Size of `spell_info`.
pub(crate) const ATTACK_PACKET_BYTES: usize = 58;
/// Combat body slots: warriors 0..5, enemies 10..109. The battle order and the
/// target list hold one entry per slot.
pub(crate) const BODY_SLOTS: usize = 110;
/// Target list entries 6..109, after the party's six.
pub(crate) const TARGET_TAIL_WORDS: usize = BODY_SLOTS - 6;
/// Tactical map cells, 90 by 90.
pub(crate) const MAP_CELLS: usize = 8100;
/// Spell field bytes.
pub(crate) const SPELL_FIELD_BYTES: usize = 1200;
/// Spell field usage bytes.
pub(crate) const FIELD_USED_BYTES: usize = 128;

/// `spell_info` is empty before the first attack is prepared.
pub(crate) fn spell_info_startup() -> Vec<u8> {
    vec![0; ATTACK_PACKET_BYTES]
}

/// The tail of the target list (entries 6 onward) starts all zero.
fn track_tail_startup() -> Vec<i16> {
    vec![0; TARGET_TAIL_WORDS]
}
impl Combat {
    /// Clears target entries 6..109, as clearing targets does and as cast does
    /// after each target.
    ///
    /// Original: cleartarget, cast
    pub(crate) fn clear_track_tail(&mut self) {
        self.track_tail = vec![0; TARGET_TAIL_WORDS];
    }
}
/// Whether no warrior is on automatic.
fn automatic_off(flags: &[bool; 6]) -> bool {
    flags.iter().all(|flag| !flag)
}
/// A battle's spell queue before anything is queued: blank.
fn empty_spell_queue() -> Vec<u8> {
    vec![0; SPELL_FIELD_BYTES]
}
/// A battle's record of occupied cells before any are marked: blank.
fn empty_collide_check() -> Vec<u8> {
    vec![0; FIELD_USED_BYTES]
}
impl Combat {
    /// Checks the battle against the squad: list sizes, difficulty and round ranges, battle order
    /// entries, the capture and firearm selections, and that every position lies on the map.
    pub fn validate(&self, warriors: &[Warrior]) -> Result<(), RuleError> {
        self.neighbors.validate()?;
        self.monsters.validate()?;
        self.weapon_traits.validate()?;
        if self.capture_actor.is_some_and(|actor| {
            Some(actor) != self.active_body()
                || !(0..warriors.len() as i16).contains(&actor)
                || self.firearm_selection.is_some()
        }) {
            return Err(RuleError::Invalid(
                "invalid mechanical targeting state".into(),
            ));
        }
        self.todo_queue.validate()?;
        if let Some(selection) = &self.firearm_selection {
            selection.validate(self, warriors)?;
        }
        if !(1..=6).contains(&warriors.len())
            || !(-2..=2).contains(&self.difficulty)
            || self.field.tiles.len() != MAP_CELLS
            || self.squad.pos.len() != warriors.len()
            || self.squad.char_under.len() != warriors.len()
            || self.roster.records.len() > 100
            || self.roster.mon_pos.len() != self.roster.records.len()
            || self.battle_order.len() != BODY_SLOTS
            || self.battle_order_index > BODY_SLOTS
            || self.macro_monster >= 100
            || self.monster_up >= 100
            || self.pending_last_defeated.is_some_and(|body| {
                body < 0
                    || (body < 10 && body as usize >= warriors.len())
                    || (body >= 10 && body as usize - 10 >= self.roster.records.len())
            })
            || self.direct_death.as_ref().is_some_and(|(body, entry)| {
                *body < 10
                    || usize::from(body.unsigned_abs()) - 10 >= self.roster.records.len()
                    || entry.script <= 0
            })
            || self.do_auto[warriors.len()..].iter().any(|flag| *flag)
            || self.turn_origin.is_some_and(|(x, y)| {
                !(0..90).contains(&(i32::from(x) + i32::from(self.field.map_shift.0) + 5))
                    || !(0..90).contains(&(i32::from(y) + i32::from(self.field.map_shift.1) + 5))
            })
            || self.turn_origin.is_some()
                && self
                    .active_body()
                    .is_none_or(|body| !(0..warriors.len() as i16).contains(&body))
            || self.track_tail.len() != TARGET_TAIL_WORDS
            || self.combat_round < 0
            || !(0..=9).contains(&self.blank_round)
            || (self.selection_pending
                && (self.battle_order_index != BODY_SLOTS || self.round_macros.pending.is_some()))
            || self.spell_queue.len() != SPELL_FIELD_BYTES
            || self.collide_check.len() != FIELD_USED_BYTES
            || self.spell_info.len() != ATTACK_PACKET_BYTES
            || self
                .roster
                .records
                .iter()
                .any(|m| m.bytes.len() != MONSTER_BYTES)
            || self.battle_order.iter().any(|id| {
                *id < -1
                    || (*id >= warriors.len() as i16 && *id < 10)
                    || *id >= self.roster.records.len() as i16 + 10
            })
            || self.squad.pos.iter().enumerate().any(|(index, (x, y))| {
                if (*x, *y) == (-1, -1)
                    && warriors[index].record().in_battle() == 0
                    && warriors[index].record().position().unwrap_or(0) == -1
                {
                    return false;
                }
                let x = i32::from(*x) + i32::from(self.field.map_shift.0) + 5;
                let y = i32::from(*y) + i32::from(self.field.map_shift.1) + 5;
                !(0..90).contains(&x) || !(0..90).contains(&y)
            })
            || self.roster.mon_pos.iter().any(|(x, y)| {
                let x = i32::from(*x) + i32::from(self.field.map_shift.0) + 5;
                let y = i32::from(*y) + i32::from(self.field.map_shift.1) + 5;
                !(0..90).contains(&x) || !(0..90).contains(&y)
            })
        {
            return Err(RuleError::Invalid("invalid tactical state".into()));
        }
        Ok(())
    }
    /// The body whose turn it is, if any: the current entry of the battle order.
    pub fn active_body(&self) -> Option<i16> {
        self.battle_order
            .get(self.battle_order_index)
            .copied()
            .filter(|id| *id >= 0)
    }
}

/// The command a click on the battle screen makes, without acting on it; see
/// `interaction` for which control takes precedence.
///
/// Original: combatchoice
pub fn click_command(state: &Session, x: i16, y: i16) -> Result<Option<Command>, RuleError> {
    interaction::click_command(state, x, y)
}
#[cfg(all(test, feature = "original-fixtures"))]
#[path = "../tests/native/initial_turn_reference.rs"]
mod initial_turn_tests;

mod movement;
pub(crate) use movement::move_controlled;
pub use movement::move_warrior;
mod turn_setup;
pub use turn_setup::{
    EncounterBattle, calculate_movement, direction_offset, initialize_turn, initiative,
};
pub(crate) use turn_setup::{initialize_enemy_record, initialize_warrior_record};
mod battle_entry;
pub(crate) use battle_entry::begin_selected_child;
pub use battle_entry::{
    CombatField, EnemySetup, SquadPlacement, begin, begin_battle, begin_group, begin_npc,
    begin_selected_battle, encounter_battle,
};
mod placement;
use placement::place_enemies_with_allies;
pub use placement::{place_enemies, place_enemies_at_difficulty, place_squad};
mod map;
pub use globals::monster_pick::MonsterPick;
pub use map::build_tactical_map;
