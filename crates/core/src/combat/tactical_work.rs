//! Tactical arrays that survive the native combat-mode flag.
use super::{Combat, TacticalMap};
use crate::{MapState, RuleError, Session};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TacticalWork {
    /// WORD map4085b8, its origin442780/442784 and the adjacent edge alias.
    pub map: TacticalMap,
    /// The single current-map saveland record, overwritten by nested entry.
    #[serde(default)]
    pub saved_land: Option<BattleLand>,
    /// Represented WORD pairs at44709a; shorter squads leave the tail unchanged.
    pub positions: [(i16, i16); 6],
    /// Six under-tile WORDs at437a6e, read by bodyfield105366.
    pub under_tiles: [i16; 6],
    /// Sixty 20-byte records at442bd6 and the independent usage work at435884.
    pub spell_fields: Vec<u8>,
    pub field_used: Vec<u8>,
    /// The battle record combatsetup loaded; its grid tells spelllist's dead-slot
    /// scan whether a skipped level-zero record left a slot. None before any battle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub battle: Option<i16>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BattleLand {
    pub level: u8,
    pub map: i32,
    pub records: MapState,
}

impl TacticalWork {
    /// Before the first battle the original reads cold DATA: the map, origin,
    /// edge alias, party positions, under tiles, fields and usage are all zero.
    pub(crate) fn startup() -> Self {
        Self {
            map: TacticalMap {
                tiles: vec![0; 8100],
                edge_word: 0,
                shift: (0, 0),
            },
            saved_land: None,
            positions: [(0, 0); 6],
            under_tiles: [0; 6],
            spell_fields: vec![0; 1200],
            field_used: vec![0; 128],
            battle: None,
        }
    }

    pub(crate) fn capture(combat: &Combat, previous: &Self) -> Self {
        let mut positions = previous.positions;
        let mut under_tiles = previous.under_tiles;
        for (slot, point) in positions.iter_mut().zip(&combat.squad.positions) {
            *slot = *point;
        }
        for (slot, tile) in under_tiles.iter_mut().zip(&combat.squad.under_tiles) {
            *slot = *tile;
        }
        Self {
            map: combat.map.clone(),
            saved_land: previous.saved_land.clone(),
            positions,
            under_tiles,
            spell_fields: combat.spell_fields.clone(),
            field_used: combat.field_used.clone(),
            battle: Some(combat.battle),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        if self.map.tiles.len() != 8100
            || self.spell_fields.len() != 1200
            || self.field_used.len() != 128
        {
            return Err("invalid retained tactical arrays".into());
        }
        if let Some(saved) = &self.saved_land {
            saved.records.native_bytes()?;
        }
        Ok(())
    }
}

impl Session {
    /// newland1400fe clears BYTE4358c2, adjacent to the sixty usage bytes.
    pub(crate) fn clear_story_entry_work(&mut self) {
        let used = if let Some(combat) = &mut self.combat {
            &mut combat.field_used
        } else {
            &mut self.native_work.tactical_work.field_used
        };
        used[62] = 0;
    }

    /// booty1059ec clears only these two BYTEs in each of sixty records.
    pub(crate) fn clear_tactical_fields(&mut self) {
        let fields = if let Some(combat) = self.combat.as_mut() {
            &mut combat.spell_fields
        } else {
            &mut self.native_work.tactical_work.spell_fields
        };
        for field in fields.as_chunks_mut::<20>().0 {
            let mut field = centurions_content::records::SpellFieldMut(field);
            field.set_initiative(0);
            field.set_duration(0);
        }
    }

    /// Each newland land change saves the old map's land slot and loads the new one;
    /// during a battle the loaded slot is what loadland13107a restores at teardown.
    pub(crate) fn record_battle_land(&mut self) {
        let (Some(_), Some(position)) = (&self.combat, &self.exploration) else {
            return;
        };
        if let Some(saved) = self.native_work.tactical_work.saved_land.as_mut() {
            *saved = BattleLand {
                level: self.scenario_level,
                map: position.map,
                records: position.map_state(),
            };
        }
    }

    /// loadland13107a restores map records, not the old combat roster or origin.
    pub(crate) fn reload_battle_land(&mut self) -> Result<(), RuleError> {
        let work = &mut self.native_work.tactical_work;
        let position = self.exploration.as_mut().ok_or("battle map missing")?;
        let Some(saved) = work
            .saved_land
            .as_ref()
            .filter(|saved| saved.level == self.scenario_level && saved.map == position.map)
        else {
            return Ok(());
        };
        position.restore_map(saved.map, saved.records.clone());
        work.map.tiles.clone_from(&saved.records.floor);
        Ok(())
    }
}
