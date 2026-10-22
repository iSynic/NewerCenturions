use super::{AttackPacket, StoryRecord, StoryRecordMut, TreasureRecord, put_word};
use crate::mission::{OpeningMission, long};
use crate::{ContentError, span, word};

impl StoryRecord<'_> {
    /// The map/x/y key (map * 10000 + y * 100 + x) matched by walking headers.
    pub fn location_key(&self) -> Result<i32, ContentError> {
        long(self.0, 0)
    }
    pub fn parameter(&self, cursor: usize) -> Result<i16, ContentError> {
        word(self.0, 24 + cursor * 2)
    }
}

impl StoryRecordMut<'_> {
    /// Signed walking-header chance; 255 retires an exhausted event.
    pub fn set_trigger_chance(&mut self, chance: u8) {
        self.0[7] = chance;
    }
    /// Replaces the eight opcodes and parameters, keeping the header and bytes 40/41.
    pub fn write_actions(&mut self, actions: &[u8; 32]) {
        self.0[8..40].copy_from_slice(actions);
    }
    /// newland 0x1434d4 copies actions and byte 40, keeping the header and stamp.
    pub fn copy_actions_and_byte40(&mut self, source: &[u8]) {
        self.0[8..41].copy_from_slice(&source[8..41]);
    }
}

/// The map's mutable table of 100 story headers (42-byte event records).
pub struct EventHeaderTable<'a>(pub &'a [u8]);

impl<'a> EventHeaderTable<'a> {
    pub const BYTES: usize = 100 * StoryRecord::BYTES;

    pub fn record(&self, slot: usize) -> Result<&'a [u8], ContentError> {
        span(self.0, slot * StoryRecord::BYTES, StoryRecord::BYTES)
    }
}

pub struct EventHeaderTableMut<'a>(pub &'a mut [u8]);

impl EventHeaderTableMut<'_> {
    pub fn replace(&mut self, slot: usize, record: &[u8]) {
        self.0[slot * StoryRecord::BYTES..(slot + 1) * StoryRecord::BYTES].copy_from_slice(record);
    }
    /// Travel replacement keeps the event's eight-byte header and its dispatch stamp.
    pub fn replace_script(&mut self, slot: usize, replacement: &[u8]) {
        let start = slot * StoryRecord::BYTES;
        self.0[start + 8..start + 41].copy_from_slice(&replacement[8..41]);
    }
    pub fn set_trigger_chance(&mut self, slot: usize, chance: u8) {
        self.0[slot * StoryRecord::BYTES + 7] = chance;
    }
}

/// Mutable regional-encounter table for twenty map rectangles, scanned from
/// the highest region down.
pub struct RegionTable<'a>(pub &'a [u8]);

impl RegionTable<'_> {
    pub const BYTES: usize = 644;
    pub const REGIONS: usize = 20;
    pub const EVENTS: usize = 3;

    pub fn first_y(&self, region: usize) -> Result<i16, ContentError> {
        word(self.0, region * 8)
    }
    pub fn first_x(&self, region: usize) -> Result<i16, ContentError> {
        word(self.0, region * 8 + 2)
    }
    pub fn last_y(&self, region: usize) -> Result<i16, ContentError> {
        word(self.0, region * 8 + 4)
    }
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
    pub fn battle_maximum(&self, region: usize) -> Result<i16, ContentError> {
        word(self.0, 202 + region * 4)
    }
    pub fn event_script(&self, region: usize, event: usize) -> Result<i16, ContentError> {
        word(self.0, 280 + region * 6 + event * 2)
    }
    /// Percent compared by magnitude; a positive chance is cleared once used.
    pub fn event_chance(&self, region: usize, event: usize) -> Result<i16, ContentError> {
        word(self.0, 400 + region * 6 + event * 2)
    }
    /// Stops the scan of lower regions and the retry after avoiding a battle.
    /// The land's tile set, which selects its BD property file.
    pub fn terrain(&self) -> u8 {
        self.0[520]
    }
    pub fn exclusive(&self, region: usize) -> bool {
        self.0[524 + region] != 0
    }
    pub fn avoidance_chance(&self, region: usize) -> i8 {
        self.0[544 + region] as i8
    }
    pub fn avoidance_sound(&self, region: usize) -> Result<i16, ContentError> {
        word(self.0, 564 + region * 2)
    }
    /// Signed dialogue ID; the caller uses its magnitude.
    pub fn avoidance_dialogue(&self, region: usize) -> Result<i16, ContentError> {
        word(self.0, 604 + region * 2)
    }
}

pub struct RegionTableMut<'a>(pub &'a mut [u8]);

impl RegionTableMut<'_> {
    pub fn set_encounter_chance(&mut self, region: usize, chance: i16) {
        put_word(self.0, 160 + region * 2, chance);
    }
    pub fn set_battle_minimum(&mut self, region: usize, value: i16) {
        put_word(self.0, 200 + region * 4, value);
    }
    pub fn set_battle_maximum(&mut self, region: usize, value: i16) {
        put_word(self.0, 202 + region * 4, value);
    }
    /// Rectangle bounds in record order: first y, first x, last y, last x.
    pub fn bound(&self, region: usize, field: usize) -> Result<i16, ContentError> {
        crate::word(self.0, region * 8 + field * 2)
    }
    pub fn set_bound(&mut self, region: usize, field: usize, value: i16) {
        put_word(self.0, region * 8 + field * 2, value);
    }
    pub fn set_terrain(&mut self, terrain: u8) {
        self.0[520] = terrain;
    }
    pub fn set_light_level(&mut self, level: u8) {
        self.0[522] = level;
    }
    pub fn clear_event_chance(&mut self, region: usize, event: usize) {
        let offset = 400 + region * 6 + event * 2;
        self.0[offset..offset + 2].fill(0);
    }
}

/// Original 426-byte encounter menu record. Branch actions are decoded by
/// the story menu adapter.
pub struct EncounterMenu<'a>(pub &'a [u8]);

impl<'a> EncounterMenu<'a> {
    /// The first entry's signed branch selector; -4 dispatches branch 4 directly.
    pub fn first_selector(&self) -> i8 {
        self.0[96] as i8
    }
    /// Signed low byte of word 100; seeds the continuation's remaining count.
    pub fn initial_remaining(&self) -> i8 {
        self.0[101] as i8
    }
    pub fn dialogue(&self) -> Result<i16, ContentError> {
        word(self.0, 104)
    }
    /// Pascal-string label bytes of one of the four 80-byte entries.
    pub fn label(&self, index: usize) -> Result<&'a [u8], ContentError> {
        let label = span(self.0, 106 + index * 80, 80)?;
        span(label, 1, usize::from(label[0]))
    }
}

/// Map treasure table of 88-byte records.
pub struct TreasureTable<'a>(pub &'a [u8]);

impl<'a> TreasureTable<'a> {
    pub fn record(&self, id: usize) -> Result<&'a [u8], ContentError> {
        span(self.0, id * TreasureRecord::BYTES, TreasureRecord::BYTES)
    }
}

impl AttackPacket<'_> {
    /// Signed flat save bonus read by the opcode 52 party save.
    pub fn save_bonus(&self) -> i8 {
        self.0[6] as i8
    }
}

impl OpeningMission {
    /// Word 2: exploration move time in ticks.
    pub fn movement_time(&self, tile: i16) -> Result<i16, ContentError> {
        self.tile_word(tile, 2)
    }
    /// Word 8: move_squad treats 1 as the boarding choice and 2 as impassable;
    /// no wider meaning is confirmed.
    pub fn boarding_field(&self, tile: i16) -> Result<i16, ContentError> {
        self.tile_word(tile, 8)
    }
}
