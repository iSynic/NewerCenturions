//! The battle map and the arrays beside it, which outlast the battle.
use crate::combat::{Combat, CombatField};
use crate::{MapState, RuleError, Session};
use serde::{Deserialize, Serialize};

/// What outlasts a battle: the tactical map, the saved land, the party's positions and the spell
/// and collision arrays.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Battlefield {
    /// The battle map, its origin and the word before it.
    #[serde(rename = "map")]
    pub field: CombatField,
    /// The land saved for this battle; a nested battle overwrites it.
    #[serde(default)]
    pub saved_land: Option<BattleLand>,
    /// Party positions; a smaller squad leaves the later ones unchanged.
    #[serde(rename = "positions")]
    pub pos: [(i16, i16); 6],
    /// The tiles under the six warriors, read by bodyfield.
    #[serde(rename = "under_tiles")]
    pub char_under: [i16; 6],
    /// The sixty spell fields and their usage bytes.
    #[serde(rename = "spell_fields")]
    pub spell_queue: Vec<u8>,
    #[serde(rename = "field_used")]
    pub collide_check: Vec<u8>,
    /// The battle record combatsetup loaded; its grid tells spelllist's dead-slot
    /// scan whether a skipped level-zero record left a slot. None before any battle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub battle: Option<i16>,
}

/// The land the party stood on when the battle began, restored when it ends.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BattleLand {
    pub level: u8,
    pub map: i32,
    pub records: MapState,
}

impl Battlefield {
    /// Before the first battle everything here is zero.
    pub(crate) fn startup() -> Self {
        Self {
            field: CombatField {
                tiles: vec![0; 8100],
                edge_word: 0,
                map_shift: (0, 0),
            },
            saved_land: None,
            pos: [(0, 0); 6],
            char_under: [0; 6],
            spell_queue: vec![0; 1200],
            collide_check: vec![0; 128],
            battle: None,
        }
    }

    /// Saves the battle's map and arrays. A squad smaller than six leaves the later positions as
    /// they were.
    pub(crate) fn capture(combat: &Combat, previous: &Self) -> Self {
        let mut positions = previous.pos;
        let mut under_tiles = previous.char_under;
        for (slot, point) in positions.iter_mut().zip(&combat.squad.pos) {
            *slot = *point;
        }
        for (slot, tile) in under_tiles.iter_mut().zip(&combat.squad.char_under) {
            *slot = *tile;
        }
        Self {
            field: combat.field.clone(),
            saved_land: previous.saved_land.clone(),
            pos: positions,
            char_under: under_tiles,
            spell_queue: combat.spell_queue.clone(),
            collide_check: combat.collide_check.clone(),
            battle: Some(combat.battle),
        }
    }

    /// Checks the saved arrays have their fixed sizes and the saved land is valid.
    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        if self.field.tiles.len() != 8100
            || self.spell_queue.len() != 1200
            || self.collide_check.len() != 128
        {
            return Err("invalid retained tactical arrays".into());
        }
        if let Some(saved) = &self.saved_land {
            saved.records.to_bytes()?;
        }
        Ok(())
    }
}

impl Session {
    /// A story clears the byte after the sixty usage bytes.
    pub(crate) fn clear_story_entry_byte(&mut self) {
        let used = if let Some(combat) = &mut self.combat {
            &mut combat.collide_check
        } else {
            &mut self.globals.battlefield.collide_check
        };
        used[62] = 0;
    }

    /// Booty clears these two bytes in each of the sixty fields.
    pub(crate) fn clear_tactical_fields(&mut self) {
        let fields = if let Some(combat) = self.combat.as_mut() {
            &mut combat.spell_queue
        } else {
            &mut self.globals.battlefield.spell_queue
        };
        for field in fields.as_chunks_mut::<20>().0 {
            let mut field = centurions_content::records::SpellFieldMut(field);
            field.set_initiative(0);
            field.set_duration(0);
        }
    }

    /// Each land change saves the old land and loads the new; during a battle the
    /// loaded land is what returns when it ends.
    pub(crate) fn record_battle_land(&mut self) {
        let (Some(_), Some(position)) = (&self.combat, &self.exploration) else {
            return;
        };
        if let Some(saved) = self.globals.battlefield.saved_land.as_mut() {
            *saved = BattleLand {
                level: self.scenario_level,
                map: position.map,
                records: position.map_state(),
            };
        }
    }

    /// Restores the land, not the old roster or origin.
    ///
    /// Original: loadland
    pub(crate) fn reload_battle_land(&mut self) -> Result<(), RuleError> {
        let battlefield = &mut self.globals.battlefield;
        let position = self.exploration.as_mut().ok_or("battle map missing")?;
        let Some(saved) = battlefield
            .saved_land
            .as_ref()
            .filter(|saved| saved.level == self.scenario_level && saved.map == position.map)
        else {
            return Ok(());
        };
        position.restore_map(saved.map, saved.records.clone());
        battlefield.field.tiles.clone_from(&saved.records.floor);
        Ok(())
    }
}
