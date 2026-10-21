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
pub mod lifecycle;
pub mod medical;
pub mod mission_deaths;
pub mod monster;
pub mod natural_conditions;
mod occupied_cell;
pub mod party_buffs;
pub mod player_psi;
pub mod psi_selection;
mod records;
pub mod retained;
pub mod retreat;
pub mod rewards;
pub mod round_stories;
pub mod runtime;
pub mod shooting;
pub mod spell_conditions;
pub mod spells;
pub mod targeting;
mod warrior_attack;
pub mod weapon_work;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Combat {
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
    #[serde(default)]
    pub attack_packet: Option<Vec<u8>>,
    #[serde(default)]
    pub attack_identity: Option<[i16; 3]>,
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
            || self.round < 0
            || !(0..=9).contains(&self.inactive_rounds)
            || (self.selection_pending
                && (self.cursor != 110 || self.round_scripts.pending.is_some()))
            || self.spell_fields.len() != 1200
            || self.field_used.len() != 128
            || self
                .attack_packet
                .as_ref()
                .is_some_and(|packet| packet.len() != 58)
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
#[path = "../tests/unit/combat_initial_turn_tests.rs"]
mod initial_turn_tests;

fn put_word(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}
mod movement;
pub(crate) use movement::move_controlled;
pub use movement::move_warrior;
mod turn_setup;
pub use turn_setup::{
    EncounterBattle, calculate_movement, direction_offset, initialize_turn, initiative,
};
pub(crate) use turn_setup::{initialize_enemy_record, initialize_warrior_record};
mod battle_entry;
pub use battle_entry::{
    EnemySetup, SquadPlacement, TacticalMap, begin, begin_battle, begin_group, begin_npc,
    begin_selected_battle, encounter_battle,
};
mod placement;
use placement::place_enemies_with_allies;
pub use placement::{place_enemies, place_enemies_at_difficulty, place_squad};
mod map;
pub use map::build_tactical_map;
