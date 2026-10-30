use crate::{RuleError, combat::Combat};
use centurions_content::WARRIOR_BYTES;
use centurions_content::combat::MONSTER_BYTES;
use centurions_content::{Warrior, records::FootprintCell};
use serde::{Deserialize, Serialize};

/// Whether death scripts are running, kept apart from the queue itself.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeathProcessing {
    /// Set while emptyque runs the queue; a new battle clears it even inside a
    /// death script.
    #[serde(rename = "queue")]
    pub doing_queue: u8,
    /// Set while killbody runs a monster's death script.
    pub callback: u8,
}

impl DeathProcessing {
    /// Whether a defeated monster's death script may run now. It waits in the queue while another
    /// death script is running or a spell is being cast.
    pub(crate) fn immediate(self, in_spell: i16) -> bool {
        self.callback == 0 && in_spell == 0
    }
}

/// A death script waiting to run: the monster that died, its side, the script and the cell where it
/// fell.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeathEntry {
    pub monster: i16,
    pub faction: i16,
    pub script: i16,
    pub x: i32,
    pub y: i32,
}
impl DeathEntry {
    /// The entry in the fourteen-byte form the queue stores.
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

/// The death scripts waiting to run (up to one hundred), and where the run has reached.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TodoQueue {
    pub entries: Vec<DeathEntry>,
    pub count: usize,
    pub cursor: usize,
    pub pointer: i32,
    pub processing: bool,
}
impl Default for TodoQueue {
    /// An empty queue.
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
impl TodoQueue {
    /// Checks the queue has its fixed size and positions within it.
    pub(crate) fn validate(&self) -> Result<(), RuleError> {
        if self.entries.len() != 100 || self.count > 100 || self.cursor > 100 {
            return Err("invalid retained death queue".into());
        }
        Ok(())
    }
    /// Adds a death script to the queue. A hundred-and-first is refused.
    pub fn enqueue(&mut self, entry: DeathEntry) -> Result<(), RuleError> {
        if self.entries.len() != 100 || self.count >= 100 {
            return Err(RuleError::Invalid("death queue capacity exceeded".into()));
        }
        self.entries[self.count] = entry;
        self.count += 1;
        Ok(())
    }
    /// Starts running the queue from its first entry.
    pub fn begin(&mut self) {
        self.processing = true;
        self.cursor = 0;
    }
    /// The next entry that still has a script to run, numbered from one; entries already run are
    /// skipped.
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
    /// A battle inside a death script empties the queue but keeps the outer cursor.
    pub fn reset_for_battle(&mut self) {
        self.count = 0;
        self.pointer &= 0xffff;
    }
    /// Marks the current entry's script as run and moves on to the next.
    pub fn acknowledge(&mut self) -> Result<(), RuleError> {
        let entry = self
            .entries
            .get_mut(self.cursor)
            .ok_or_else(|| RuleError::Invalid("invalid death queue cursor".into()))?;
        entry.script = 0;
        self.cursor += 1;
        Ok(())
    }
    /// Empties the queue and stops running it.
    pub fn finish(&mut self) {
        self.count = 0;
        // Only the low word of the queue marker clears.
        self.pointer &= 0xffff;
        self.processing = false;
    }
}

/// What a death story saves of the running story, its thirty-two action slots and the body that
/// died, to restore when the death story is done.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeathStoryFrame {
    pub actions: [u8; 32],
    #[serde(rename = "native_body")]
    pub body: i16,
}
impl DeathStoryFrame {
    /// Saves the story's action slots and the body that died.
    pub fn capture(header: &[u8], body: i16) -> Result<Self, RuleError> {
        let actions = header
            .get(8..40)
            .ok_or_else(|| RuleError::Invalid("invalid death story header".into()))?
            .try_into()
            .unwrap();
        Ok(Self { actions, body })
    }
    /// Puts the saved action slots back into the story header.
    pub fn restore(&self, header: &mut [u8]) -> Result<(), RuleError> {
        header
            .get_mut(8..40)
            .ok_or_else(|| RuleError::Invalid("invalid death story header".into()))?
            .copy_from_slice(&self.actions);
        Ok(())
    }
}

/// What a battle started by a story saves of the story that started it: its action slots, its level
/// and where it had reached.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EncounterBattleFrame {
    pub actions: [u8; 32],
    pub level: u8,
    pub action_index: i16,
}
impl EncounterBattleFrame {
    /// Saves the story's action slots, level and place before the battle begins.
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
    /// Puts the story back as it was and returns the place to resume at.
    pub fn restore(&self, header: &mut [u8]) -> Result<i16, RuleError> {
        if header.len() != 42 {
            return Err(RuleError::Invalid("invalid encounter battle header".into()));
        }
        header[8..40].copy_from_slice(&self.actions);
        header[40] = self.level;
        Ok(self.action_index)
    }
}

/// What removing a defeated monster leaves to do: the death script to queue, if it has one, and
/// whether it counts toward the battle's kills.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeathCleanup {
    pub entry: Option<DeathEntry>,
    pub increment_kills: bool,
}

/// Removes a defeated monster from the battle, returns its death script for the caller to queue,
/// and makes every warrior and monster that was aiming at it choose no target.
///
/// Original: killbody
pub fn cleanup_enemy(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    body: i16,
) -> Result<DeathCleanup, RuleError> {
    let result = cleanup_enemy_before_story(combat, warriors, body)?;
    clear_body_references(combat, warriors, body)?;
    Ok(result)
}

/// Removes a defeated monster's body from the map and the battle order, stops it moving and clears
/// its conditions. A monster with a nonzero faction counts as a kill unless it is a disabled
/// machine. Targets keep pointing at the body until the story has run.
///
/// Original: killbody
pub(crate) fn cleanup_enemy_before_story(
    combat: &mut Combat,
    warriors: &[Warrior],
    body: i16,
) -> Result<DeathCleanup, RuleError> {
    let index = usize::try_from(body.checked_sub(10).unwrap_or(-1))
        .map_err(|_| RuleError::Invalid("invalid defeated enemy".into()))?;
    let record = combat
        .roster
        .records
        .get(index)
        .ok_or_else(|| RuleError::Invalid("invalid defeated enemy".into()))?;
    if combat.roster.records.len() > 100
        || combat
            .roster
            .records
            .iter()
            .any(|m| m.bytes.len() != MONSTER_BYTES)
        || !(1..=6).contains(&warriors.len())
        || warriors.iter().any(|w| w.bytes.len() != WARRIOR_BYTES)
        || combat.field.tiles.len() != 8100
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
        .roster
        .mon_pos
        .get(index)
        .ok_or_else(|| RuleError::Invalid("enemy position missing".into()))?;
    let (x, y) = crate::combat::records::grid_point(position, combat.field.map_shift);
    let script = record.todo_on_death()?;
    let faction = record.faction()?;
    let entry = (script != 0).then(|| DeathEntry {
        monster: i16::from(record.monster_id()),
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
        combat.kill_counts.add_kill_mon(1);
    }
    combat.roster.records[index].record_mut().set_movement(0);
    combat.roster.records[index].record_mut().clear_conditions();
    for (index, value) in changes {
        combat.field.tiles[index] = value;
    }
    for entry in combat.battle_order.iter_mut().skip(1) {
        if *entry == body {
            *entry = -1;
        }
    }
    Ok(DeathCleanup {
        entry,
        increment_kills,
    })
}

/// Makes every warrior and monster that was aiming at the body choose no target.
pub fn clear_body_references(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    body: i16,
) -> Result<(), RuleError> {
    if warriors.iter().any(|w| w.bytes.len() != WARRIOR_BYTES)
        || combat
            .roster
            .records
            .iter()
            .any(|m| m.bytes.len() != MONSTER_BYTES)
    {
        return Err(RuleError::Invalid("invalid death target input".into()));
    }
    for warrior in warriors {
        if warrior.record().target()? == body {
            warrior.record_mut().set_target(-1);
        }
    }
    for monster in &mut combat.roster.records {
        if monster.record().target()? == body {
            monster.record_mut().set_target(-1);
        }
    }
    Ok(())
}
