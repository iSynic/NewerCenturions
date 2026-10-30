use super::{SpellInfoRecord, StoryRecord, StoryRecordMut, TreasureRecord, put_word};
use crate::bytes::long;
use crate::mission::OpeningMission;
use crate::{ContentError, span, word};

impl StoryRecord<'_> {
    /// The location key, map * 10000 + y * 100 + x, that a walking event matches.
    pub fn location_key(&self) -> Result<i32, ContentError> {
        long(self.0, 0)
    }
    /// Parameter `cursor` of the event's eight actions.
    pub fn parameter(&self, cursor: usize) -> Result<i16, ContentError> {
        word(self.0, 24 + cursor * 2)
    }
}

impl StoryRecordMut<'_> {
    /// The walking event's chance, a signed byte; 255 (-1) retires a spent event.
    pub fn set_trigger_chance(&mut self, chance: u8) {
        self.0[7] = chance;
    }
    /// Replaces the eight opcodes and parameters, keeping the header and the bytes after them.
    pub fn write_actions(&mut self, actions: &[u8; 32]) {
        self.0[8..40].copy_from_slice(actions);
    }
    /// Copies the actions and byte 40, keeping the header and stamp.
    pub fn copy_actions_and_byte40(&mut self, source: &[u8]) {
        self.0[8..41].copy_from_slice(&source[8..41]);
    }
}

/// A map's 100 events (42-byte event records), changed as stories run.
pub struct EventHeaderTable<'a>(pub &'a [u8]);

impl<'a> EventHeaderTable<'a> {
    /// Size of the event table.
    pub const BYTES: usize = 100 * StoryRecord::BYTES;

    /// Event record `slot`.
    pub fn record(&self, slot: usize) -> Result<&'a [u8], ContentError> {
        span(self.0, slot * StoryRecord::BYTES, StoryRecord::BYTES)
    }
}

/// A map's event records as a view that can change them.
pub struct EventHeaderTableMut<'a>(pub &'a mut [u8]);

impl EventHeaderTableMut<'_> {
    /// Overwrites event `slot` with `record`.
    pub fn replace(&mut self, slot: usize, record: &[u8]) {
        self.0[slot * StoryRecord::BYTES..(slot + 1) * StoryRecord::BYTES].copy_from_slice(record);
    }
    /// Travel replacement keeps the event's eight-byte header and its dispatch stamp.
    pub fn replace_script(&mut self, slot: usize, replacement: &[u8]) {
        let start = slot * StoryRecord::BYTES;
        self.0[start + 8..start + 41].copy_from_slice(&replacement[8..41]);
    }
    /// Sets the walking chance of event `slot`; 255 retires the event.
    pub fn set_trigger_chance(&mut self, slot: usize, chance: u8) {
        self.0[slot * StoryRecord::BYTES + 7] = chance;
    }
}

/// A land's region table: random encounters, events, tile set and light for
/// twenty map rectangles, scanned from the highest region down.
pub struct RegionTable<'a>(pub &'a [u8]);

impl RegionTable<'_> {
    /// Size of a region table.
    pub const BYTES: usize = 644;
    /// Number of regions in a table.
    pub const REGIONS: usize = 20;
    /// Number of events a region can run.
    pub const EVENTS: usize = 3;

    /// The region's first row.
    pub fn first_y(&self, region: usize) -> Result<i16, ContentError> {
        word(self.0, region * 8)
    }
    /// The region's first column.
    pub fn first_x(&self, region: usize) -> Result<i16, ContentError> {
        word(self.0, region * 8 + 2)
    }
    /// The region's last row.
    pub fn last_y(&self, region: usize) -> Result<i16, ContentError> {
        word(self.0, region * 8 + 4)
    }
    /// The region's last column.
    pub fn last_x(&self, region: usize) -> Result<i16, ContentError> {
        word(self.0, region * 8 + 6)
    }
    /// Chance in 10000, divided by the caller's divisor.
    pub fn encounter_chance(&self, region: usize) -> Result<i16, ContentError> {
        word(self.0, 160 + region * 2)
    }
    /// A zero minimum disables the regional battle.
    pub fn battle_minimum(&self, region: usize) -> Result<i16, ContentError> {
        word(self.0, 200 + region * 4)
    }
    /// Highest battle number the region can draw.
    pub fn battle_maximum(&self, region: usize) -> Result<i16, ContentError> {
        word(self.0, 202 + region * 4)
    }
    /// The story script of event `event` of the region.
    pub fn event_script(&self, region: usize, event: usize) -> Result<i16, ContentError> {
        word(self.0, 280 + region * 6 + event * 2)
    }
    /// Percent compared by magnitude; a positive chance is cleared once used.
    pub fn event_chance(&self, region: usize, event: usize) -> Result<i16, ContentError> {
        word(self.0, 400 + region * 6 + event * 2)
    }
    /// The land's tile set, which selects its BD property file.
    pub fn terrain(&self) -> u8 {
        self.0[520]
    }
    /// The terrain byte of a saved table, or `None` when the table is short.
    pub fn stored_terrain(&self) -> Option<u8> {
        self.0.get(520).copied()
    }
    /// The land's light level.
    pub fn light_level(&self) -> Result<u8, ContentError> {
        Ok(span(self.0, 522, 1)?[0])
    }
    /// Stops the scan of lower regions and the retry after avoiding a battle.
    pub fn exclusive(&self, region: usize) -> bool {
        self.0[524 + region] != 0
    }
    /// Percent chance the party avoids a battle in the region.
    pub fn avoidance_chance(&self, region: usize) -> i8 {
        self.0[544 + region] as i8
    }
    /// The sound played when the party avoids a battle in the region.
    pub fn avoidance_sound(&self, region: usize) -> Result<i16, ContentError> {
        word(self.0, 564 + region * 2)
    }
    /// Signed dialogue ID; the caller uses its magnitude.
    pub fn avoidance_dialogue(&self, region: usize) -> Result<i16, ContentError> {
        word(self.0, 604 + region * 2)
    }
}

/// A land's region table as a view that can change it.
pub struct RegionTableMut<'a>(pub &'a mut [u8]);

impl RegionTableMut<'_> {
    /// Sets the region's encounter chance.
    pub fn set_encounter_chance(&mut self, region: usize, chance: i16) {
        put_word(self.0, 160 + region * 2, chance);
    }
    /// Sets the region's lowest battle number.
    pub fn set_battle_minimum(&mut self, region: usize, value: i16) {
        put_word(self.0, 200 + region * 4, value);
    }
    /// Sets the region's highest battle number.
    pub fn set_battle_maximum(&mut self, region: usize, value: i16) {
        put_word(self.0, 202 + region * 4, value);
    }
    /// Rectangle bounds in record order: first y, first x, last y, last x.
    pub fn bound(&self, region: usize, field: usize) -> Result<i16, ContentError> {
        crate::word(self.0, region * 8 + field * 2)
    }
    /// Sets edge `field` of the region's rectangle, in record order.
    pub fn set_bound(&mut self, region: usize, field: usize, value: i16) {
        put_word(self.0, region * 8 + field * 2, value);
    }
    /// Sets the land's tile set.
    pub fn set_terrain(&mut self, terrain: u8) {
        self.0[520] = terrain;
    }
    /// Sets the land's light level.
    pub fn set_light_level(&mut self, level: u8) {
        self.0[522] = level;
    }
    /// Clears the chance of event `event` of the region, so it no longer runs.
    pub fn clear_event_chance(&mut self, region: usize, event: usize) {
        let offset = 400 + region * 6 + event * 2;
        self.0[offset..offset + 2].fill(0);
    }
}

/// A 426-byte encounter menu: four labelled choices and their branches. The
/// story menu rules decode the branch actions.
pub struct EncounterMenu<'a>(pub &'a [u8]);

impl<'a> EncounterMenu<'a> {
    /// The first entry's signed branch selector; -4 dispatches branch 4 directly.
    pub fn first_selector(&self) -> i8 {
        self.0[96] as i8
    }
    /// Signed low byte of word 100: the remaining count a continuation starts with.
    pub fn initial_remaining(&self) -> i8 {
        self.0[101] as i8
    }
    /// The dialogue shown with the menu.
    pub fn dialogue(&self) -> Result<i16, ContentError> {
        word(self.0, 104)
    }
    /// Pascal-string label bytes of one of the four 80-byte entries.
    pub fn label(&self, index: usize) -> Result<&'a [u8], ContentError> {
        let label = span(self.0, 106 + index * 80, 80)?;
        span(label, 1, usize::from(label[0]))
    }
}

/// An encounter menu as a view that can change it.
pub struct EncounterMenuMut<'a>(pub &'a mut [u8]);

impl EncounterMenuMut<'_> {
    /// Disables one-based `choice`.
    pub fn clear_choice(&mut self, choice: usize) {
        self.0[95 + choice] = 0;
    }
}

/// Map treasure table of 88-byte records.
pub struct TreasureTable<'a>(pub &'a [u8]);

impl<'a> TreasureTable<'a> {
    /// Treasure record `id`.
    pub fn record(&self, id: usize) -> Result<&'a [u8], ContentError> {
        span(self.0, id * TreasureRecord::BYTES, TreasureRecord::BYTES)
    }
}

impl SpellInfoRecord<'_> {
    /// Signed saving-throw bonus added whatever the power level.
    pub fn save_bonus(&self) -> i8 {
        self.0[6] as i8
    }
}

impl OpeningMission {
    /// Word 2: ticks a step takes while exploring.
    pub fn movement_time(&self, tile: i16) -> Result<i16, ContentError> {
        self.tile_word(tile, 2)
    }
    /// Word 8: 1 offers to board a boat; 2 marks a square only a boat may enter, which a walking
    /// party cannot step onto; other values have no known meaning.
    pub fn boarding_field(&self, tile: i16) -> Result<i16, ContentError> {
        self.tile_word(tile, 8)
    }
}
