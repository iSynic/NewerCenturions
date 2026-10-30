//! What a spell did to each body, recorded for the caller to commit in order.
use super::AttackResult;
use crate::combat::warrior_alias::AliasWrite;
use centurions_content::Warrior;
use centurions_content::combat::MonsterRecord;

/// What a spell did to the warriors: their records after it, the result for each, and the warriors
/// a race restriction spared.
pub struct WarriorSpellResult {
    pub warriors: Vec<Warrior>,
    pub targets: Vec<(usize, AttackResult)>,
    pub race_excluded: Vec<usize>,
}

/// What a spell did to warriors and monsters: the change to the hostile count, their records after
/// it, the sounds, the result for each body, and the bodies a race restriction spared.
pub struct BodySpellResult {
    pub total_delta: i16,
    pub warriors: Vec<Warrior>,
    pub enemies: Vec<MonsterRecord>,
    pub sounds: Vec<i16>,
    pub targets: Vec<(usize, AttackResult)>,
    pub race_excluded: Vec<usize>,
}

/// What a spell did, beyond damage: as the body result, plus whether it healed and how many
/// warriors it recovered, and the other changes it made (fatigue, area shape, forced kills, writes
/// to warrior-indexed monster fields).
pub struct SupportSpellResult {
    pub total_delta: i16,
    pub race_excluded: Vec<usize>,
    pub warriors: Vec<Warrior>,
    pub enemies: Vec<MonsterRecord>,
    pub sounds: Vec<i16>,
    pub targets: Vec<(usize, AttackResult)>,
    pub healed: bool,
    pub recovered: u8,
    /// Effect 68 stored fatigue 1 for the caller's updatefat.
    pub fatigue_reset: bool,
    /// Effect 66 replaced a zero area byte with its draw.
    pub area_shape: Option<u8>,
    /// Effect 90 bodies killed inside spelllist, before any health change.
    pub forced_kills: Vec<usize>,
    /// Effect 26 and 64 writes to warrior-indexed monster fields, in spelllist order.
    pub(crate) alias_writes: Vec<AliasWrite>,
}

impl SupportSpellResult {
    /// Copies of the body records with no effects recorded yet.
    pub(crate) fn new(warriors: &[Warrior], enemies: &[MonsterRecord], sounds: Vec<i16>) -> Self {
        Self {
            total_delta: 0,
            warriors: warriors.to_vec(),
            enemies: enemies.to_vec(),
            sounds,
            targets: vec![],
            healed: false,
            recovered: 0,
            race_excluded: vec![],
            fatigue_reset: false,
            area_shape: None,
            forced_kills: vec![],
            alias_writes: vec![],
        }
    }

    /// Record bytes of a warrior body below 10 or the enemy at `body - 10`;
    /// panics outside either table.
    pub(crate) fn body_bytes(&self, body: usize) -> &Vec<u8> {
        if body < 10 {
            &self.warriors[body].bytes
        } else {
            &self.enemies[body - 10].bytes
        }
    }

    /// Mutable record bytes of a warrior body below 10 or the enemy at `body - 10`;
    /// panics outside either table.
    pub(crate) fn body_bytes_mut(&mut self, body: usize) -> &mut Vec<u8> {
        if body < 10 {
            &mut self.warriors[body].bytes
        } else {
            &mut self.enemies[body - 10].bytes
        }
    }
}

impl crate::Session {
    /// Combat updatefat skips its clamp, so effect68 leaves fatigue 1 until a later change.
    pub(crate) fn store_spell_fatigue(&mut self) {
        if let Some(combat) = self.combat.as_mut()
            && std::mem::take(&mut combat.fatigue_reset)
            && let Some(position) = self.exploration.as_mut()
        {
            position.fatigue = 1;
        }
    }
}
