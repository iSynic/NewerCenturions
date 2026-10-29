use crate::{RuleError, combat::Combat};
use centurions_content::{Warrior, records::FootprintCell};
use serde::{Deserialize, Serialize};

/// Literal native bytes, independent of saved caller depth and queue ownership.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeathProcessing {
    /// BYTE438584: emptyque sets1/0; combatsetup clears it even inside a callback.
    pub queue: u8,
    /// BYTE438585: killbody sets1 before newland and0 after returning, without a pop.
    pub callback: u8,
}

impl DeathProcessing {
    pub(crate) fn immediate(self, ready: i16) -> bool {
        self.callback == 0 && ready == 0
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeathEntry {
    pub monster: i16,
    pub faction: i16,
    pub script: i16,
    pub x: i32,
    pub y: i32,
}
impl DeathEntry {
    pub fn bytes(self) -> [u8; 14] {
        let mut bytes = [0; 14];
        bytes[..2].copy_from_slice(&self.monster.to_be_bytes());
        bytes[2..4].copy_from_slice(&self.faction.to_be_bytes());
        bytes[4..6].copy_from_slice(&self.script.to_be_bytes());
        bytes[6..10].copy_from_slice(&self.x.to_be_bytes());
        bytes[10..14].copy_from_slice(&self.y.to_be_bytes());
        bytes
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeathQueue {
    pub entries: Vec<DeathEntry>,
    pub count: usize,
    pub cursor: usize,
    pub pointer: i32,
    pub processing: bool,
}
impl Default for DeathQueue {
    fn default() -> Self {
        Self {
            entries: vec![DeathEntry::default(); 100],
            count: 0,
            cursor: 0,
            pointer: 0,
            processing: false,
        }
    }
}
impl DeathQueue {
    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        if self.entries.len() != 100 || self.count > 100 || self.cursor > 100 {
            return Err("invalid retained death queue".into());
        }
        Ok(())
    }
    pub fn enqueue(&mut self, entry: DeathEntry) -> Result<(), RuleError> {
        if self.entries.len() != 100 || self.count >= 100 {
            return Err(RuleError::Invalid("death queue capacity exceeded".into()));
        }
        self.entries[self.count] = entry;
        self.count += 1;
        Ok(())
    }
    pub fn begin(&mut self) {
        self.processing = true;
        self.cursor = 0;
    }
    pub fn current(&mut self) -> Option<(usize, DeathEntry)> {
        while self.cursor < self.count && self.cursor < self.entries.len() {
            let entry = self.entries[self.cursor];
            if entry.script != 0 {
                return Some((self.cursor + 1, entry));
            }
            self.cursor += 1;
        }
        None
    }
    // A nested combat resets the native count, leaving the outer local cursor intact.
    pub fn reset_for_battle(&mut self) {
        self.count = 0;
        self.pointer &= 0xffff;
    }
    pub fn acknowledge(&mut self) -> Result<(), RuleError> {
        let entry = self
            .entries
            .get_mut(self.cursor)
            .ok_or_else(|| RuleError::Invalid("invalid death queue cursor".into()))?;
        entry.script = 0;
        self.cursor += 1;
        Ok(())
    }
    pub fn finish(&mut self) {
        self.count = 0;
        // The native queue marker is a word; its adjacent word is untouched.
        self.pointer &= 0xffff;
        self.processing = false;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeathStoryFrame {
    pub actions: [u8; 32],
    pub native_body: i16,
}
impl DeathStoryFrame {
    pub fn capture(header: &[u8], native_body: i16) -> Result<Self, RuleError> {
        let actions = header
            .get(8..40)
            .ok_or_else(|| RuleError::Invalid("invalid death story header".into()))?
            .try_into()
            .unwrap();
        Ok(Self {
            actions,
            native_body,
        })
    }
    pub fn restore(&self, header: &mut [u8]) -> Result<(), RuleError> {
        header
            .get_mut(8..40)
            .ok_or_else(|| RuleError::Invalid("invalid death story header".into()))?
            .copy_from_slice(&self.actions);
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EncounterBattleFrame {
    pub actions: [u8; 32],
    pub level: u8,
    pub action_index: i16,
}
impl EncounterBattleFrame {
    pub fn capture(header: &[u8], action_index: i16) -> Result<Self, RuleError> {
        if header.len() != 42 {
            return Err(RuleError::Invalid("invalid encounter battle header".into()));
        }
        Ok(Self {
            actions: header[8..40].try_into().unwrap(),
            level: header[40],
            action_index,
        })
    }
    pub fn restore(&self, header: &mut [u8]) -> Result<i16, RuleError> {
        if header.len() != 42 {
            return Err(RuleError::Invalid("invalid encounter battle header".into()));
        }
        header[8..40].copy_from_slice(&self.actions);
        header[40] = self.level;
        Ok(self.action_index)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeathCleanup {
    pub entry: Option<DeathEntry>,
    pub increment_kills: bool,
}

pub fn cleanup_enemy(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    body: i16,
) -> Result<DeathCleanup, RuleError> {
    let result = cleanup_enemy_before_story(combat, warriors, body)?;
    clear_body_references(combat, warriors, body)?;
    Ok(result)
}

pub(crate) fn cleanup_enemy_before_story(
    combat: &mut Combat,
    warriors: &[Warrior],
    body: i16,
) -> Result<DeathCleanup, RuleError> {
    let index = usize::try_from(body.checked_sub(10).unwrap_or(-1))
        .map_err(|_| RuleError::Invalid("invalid defeated enemy".into()))?;
    let record = combat
        .enemies
        .records
        .get(index)
        .ok_or_else(|| RuleError::Invalid("invalid defeated enemy".into()))?;
    if combat.enemies.records.len() > 100
        || combat.enemies.records.iter().any(|m| m.bytes.len() != 288)
        || !(1..=6).contains(&warriors.len())
        || warriors.iter().any(|w| w.bytes.len() != 3138)
        || combat.map.tiles.len() != 8100
    {
        return Err(RuleError::Invalid("invalid death cleanup input".into()));
    }
    let record = record.record();
    let signed_size = record.size()?;
    let size = signed_size.wrapping_abs();
    if !(0..=3).contains(&size) {
        return Err(RuleError::Unsupported {
            action: "combat.death",
            dependency: "enemy body dimensions",
        });
    }
    let position = *combat
        .enemies
        .positions
        .get(index)
        .ok_or_else(|| RuleError::Invalid("enemy position missing".into()))?;
    let x = i32::from(position.0) + i32::from(combat.map.shift.0) + 5;
    let y = i32::from(position.1) + i32::from(combat.map.shift.1) + 5;
    let script = record.death_script()?;
    let faction = record.faction()?;
    let entry = (script != 0).then(|| DeathEntry {
        monster: i16::from(record.definition_id()),
        faction,
        script,
        x,
        y,
    });
    let increment_kills = faction != 0 && record.mechanical_state() != 255;
    let mut cells = vec![(x, y, record.footprint_tile(FootprintCell::Anchor)?)];
    if size == 3 {
        cells.push((
            x - 1,
            y - 1,
            record.footprint_tile(FootprintCell::Northwest)?,
        ));
    }
    if size > 1 {
        cells.push((x - 1, y, record.footprint_tile(FootprintCell::West)?));
    }
    if size == 3 || signed_size == 1 {
        cells.push((x, y - 1, record.footprint_tile(FootprintCell::North)?));
    }
    let changes = cells
        .into_iter()
        .map(|(x, y, value)| {
            centurions_content::mission::OpeningMission::index(x, y)
                .map(|index| (index, value))
                .map_err(RuleError::from)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if increment_kills {
        combat.victory_work.add_defeated(1);
    }
    combat.enemies.records[index]
        .record_mut()
        .set_movement_points(0);
    combat.enemies.records[index]
        .record_mut()
        .clear_conditions();
    for (index, value) in changes {
        combat.map.tiles[index] = value;
    }
    for entry in combat.order.iter_mut().skip(1) {
        if *entry == body {
            *entry = -1;
        }
    }
    Ok(DeathCleanup {
        entry,
        increment_kills,
    })
}

pub fn clear_body_references(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    body: i16,
) -> Result<(), RuleError> {
    if warriors.iter().any(|w| w.bytes.len() != 3138)
        || combat.enemies.records.iter().any(|m| m.bytes.len() != 288)
    {
        return Err(RuleError::Invalid("invalid death target input".into()));
    }
    for warrior in warriors {
        if warrior.record().target_body()? == body {
            warrior.record_mut().set_target_body(-1);
        }
    }
    for monster in &mut combat.enemies.records {
        if monster.record().target_body()? == body {
            monster.record_mut().set_target_body(-1);
        }
    }
    Ok(())
}
