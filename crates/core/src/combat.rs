use crate::{Command, RuleError, Session};
use centurions_content::Warrior;
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
pub mod neighbor_work;
pub mod npc_work;
mod occupied_cell;
pub(crate) mod outdoor_health;
pub mod party_buffs;
pub mod player_psi;
pub mod psi_selection;
mod record_state;
mod records;
pub mod retained;
pub mod retreat;
mod reward_mode;
pub mod rewards;
pub mod round_stories;
pub mod round_work;
pub mod runtime;
pub mod shooting;
pub mod spell_conditions;
pub mod spells;
pub mod tactical_work;
pub mod target_work;
pub mod targeting;
pub mod victory_work;
pub mod warrior_alias;
mod warrior_attack;
pub mod weapon_work;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Combat {
    #[serde(default)]
    pub victory_work: victory_work::VictoryWork,
    #[serde(default)]
    pub neighbor_work: neighbor_work::NeighborWork,
    /// Live copy of signed WORD438760, retained to NativeWorkContext at teardown.
    #[serde(default)]
    pub ready: i16,
    #[serde(default, skip_serializing_if = "crate::normal_difficulty")]
    pub difficulty: i16,
    pub battle: i16,
    pub map: TacticalMap,
    pub squad: SquadPlacement,
    pub enemies: EnemySetup,
    #[serde(default)]
    pub retained_npcs: retained::NpcBacking,
    #[serde(default)]
    pub direct_deaths: bool,
    /// A support spell's effect68 within this command; the command boundary stores it.
    #[serde(skip)]
    pub fatigue_reset: bool,
    /// Effect26/64 warrior-indexed NPC writes awaiting their retained owners.
    #[serde(skip)]
    pub alias_writes: Vec<warrior_alias::AliasWrite>,
    /// Scoped resolvespell caller input while an item uses the shared cast/ray
    /// kernels; restored before any command or save boundary.
    #[serde(skip)]
    pub item_condition_caller: Option<i16>,
    /// Transient notification of an actual MONSTER scratch copy/build.
    #[serde(skip)]
    pub monster_work_written: bool,
    /// Known MONSTER work lent for the current command; never saved.
    #[serde(skip)]
    pub monster_template: Option<Vec<u8>>,
    /// Actual attack writer, drained into shared WORD43877e before callbacks.
    #[serde(skip)]
    pub npc_inhibit_write: Option<i16>,
    #[serde(default)]
    pub direct_death: Option<(i16, death_scripts::DeathEntry)>,
    pub order: Vec<i16>,
    pub cursor: usize,
    pub round: i16,
    /// Native WORD4427a6: rounds since attack/cast; ten wraps force retreat.
    #[serde(default)]
    pub inactive_rounds: i16,
    #[serde(default)]
    pub selection_pending: bool,
    /// getup's retained WORD43800e; an NPC script death skips its reset.
    #[serde(default)]
    pub script_enemy: usize,
    /// WORD447196 is written by getup, independently of the retained script origin.
    #[serde(default)]
    pub selected_enemy: usize,
    #[serde(default)]
    pub round_scripts: round_stories::RoundScripts,
    #[serde(default)]
    pub death_queue: death_scripts::DeathQueue,
    #[serde(default = "empty_spell_fields")]
    pub spell_fields: Vec<u8>,
    #[serde(default = "empty_field_used")]
    pub field_used: Vec<u8>,
    #[serde(default = "attack_packet_startup")]
    pub attack_packet: Vec<u8>,
    #[serde(default)]
    pub attack_identity: [i16; 3],
    #[serde(default)]
    pub weapon_work: weapon_work::WeaponWork,
    #[serde(default)]
    pub unavailable: i8,
    /// killbody's retained body, consumed into the session before script dispatch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending_last_defeated: Option<i16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firearm_selection: Option<shooting::Selection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capture_actor: Option<i16>,
    #[serde(default, skip_serializing_if = "automatic_off")]
    pub automatic: [bool; 6],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_origin: Option<(i16, i16)>,
    /// Target-mask WORDs 6..109 at 4434e0; words 0..5 stay in the session party mask.
    #[serde(default = "target_tail_startup")]
    pub target_tail: Vec<i16>,
    /// WORDs 443722/443724, 4435c0 and 442bd4; schema 60.
    #[serde(default, skip_serializing_if = "target_work::TargetWork::is_startup")]
    pub target_work: target_work::TargetWork,
}

/// The 58-byte attack packet work is zero before the first prepared attack.
pub(crate) fn attack_packet_startup() -> Vec<u8> {
    vec![0; 58]
}

fn target_tail_startup() -> Vec<i16> {
    vec![0; 104]
}
impl Combat {
    /// cleartarget13822a, and cast10e642's clear after each target.
    pub(crate) fn clear_target_tail(&mut self) {
        self.target_tail = vec![0; 104];
    }
}
fn automatic_off(flags: &[bool; 6]) -> bool {
    flags.iter().all(|flag| !flag)
}
fn empty_spell_fields() -> Vec<u8> {
    vec![0; 1200]
}
fn empty_field_used() -> Vec<u8> {
    vec![0; 128]
}
impl Combat {
    pub fn validate(&self, warriors: &[Warrior]) -> Result<(), RuleError> {
        self.neighbor_work.validate()?;
        self.retained_npcs.validate()?;
        self.weapon_work.validate()?;
        if self.capture_actor.is_some_and(|actor| {
            Some(actor) != self.active_body()
                || !(0..warriors.len() as i16).contains(&actor)
                || self.firearm_selection.is_some()
        }) {
            return Err(RuleError::Invalid(
                "invalid mechanical targeting state".into(),
            ));
        }
        self.death_queue.validate()?;
        if let Some(selection) = &self.firearm_selection {
            selection.validate(self, warriors)?;
        }
        if !(1..=6).contains(&warriors.len())
            || !(-2..=2).contains(&self.difficulty)
            || self.map.tiles.len() != 8100
            || self.squad.positions.len() != warriors.len()
            || self.squad.under_tiles.len() != warriors.len()
            || self.enemies.records.len() > 100
            || self.enemies.positions.len() != self.enemies.records.len()
            || self.order.len() != 110
            || self.cursor > 110
            || self.script_enemy >= 100
            || self.selected_enemy >= 100
            || self.pending_last_defeated.is_some_and(|body| {
                body < 0
                    || (body < 10 && body as usize >= warriors.len())
                    || (body >= 10 && body as usize - 10 >= self.enemies.records.len())
            })
            || self.direct_death.as_ref().is_some_and(|(body, entry)| {
                *body < 10
                    || usize::from(body.unsigned_abs()) - 10 >= self.enemies.records.len()
                    || entry.script <= 0
            })
            || self.automatic[warriors.len()..].iter().any(|flag| *flag)
            || self.turn_origin.is_some_and(|(x, y)| {
                !(0..90).contains(&(i32::from(x) + i32::from(self.map.shift.0) + 5))
                    || !(0..90).contains(&(i32::from(y) + i32::from(self.map.shift.1) + 5))
            })
            || self.turn_origin.is_some()
                && self
                    .active_body()
                    .is_none_or(|body| !(0..warriors.len() as i16).contains(&body))
            || self.target_tail.len() != 104
            || self.round < 0
            || !(0..=9).contains(&self.inactive_rounds)
            || (self.selection_pending
                && (self.cursor != 110 || self.round_scripts.pending.is_some()))
            || self.spell_fields.len() != 1200
            || self.field_used.len() != 128
            || self.attack_packet.len() != 58
            || self.enemies.records.iter().any(|m| m.bytes.len() != 288)
            || self.order.iter().any(|id| {
                *id < -1
                    || (*id >= warriors.len() as i16 && *id < 10)
                    || *id >= self.enemies.records.len() as i16 + 10
            })
            || self
                .squad
                .positions
                .iter()
                .enumerate()
                .any(|(index, (x, y))| {
                    if (*x, *y) == (-1, -1)
                        && warriors[index].record().active_flag() == 0
                        && warriors[index].record().defeated_marker().unwrap_or(0) == -1
                    {
                        return false;
                    }
                    let x = i32::from(*x) + i32::from(self.map.shift.0) + 5;
                    let y = i32::from(*y) + i32::from(self.map.shift.1) + 5;
                    !(0..90).contains(&x) || !(0..90).contains(&y)
                })
            || self.enemies.positions.iter().any(|(x, y)| {
                let x = i32::from(*x) + i32::from(self.map.shift.0) + 5;
                let y = i32::from(*y) + i32::from(self.map.shift.1) + 5;
                !(0..90).contains(&x) || !(0..90).contains(&y)
            })
        {
            return Err(RuleError::Invalid("invalid tactical state".into()));
        }
        Ok(())
    }
    pub fn active_body(&self) -> Option<i16> {
        self.order.get(self.cursor).copied().filter(|id| *id >= 0)
    }
}

/// Read-only combatchoice routing; see interaction for the native precedence.
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
    EnemySetup, SquadPlacement, TacticalMap, begin, begin_battle, begin_group, begin_npc,
    begin_selected_battle, encounter_battle,
};
mod placement;
use placement::place_enemies_with_allies;
pub use placement::{place_enemies, place_enemies_at_difficulty, place_squad};
mod map;
pub use map::build_tactical_map;
mod monster_work;
pub(crate) mod reward_work;
mod spell_summon;
pub use monster_work::MonsterWork;
