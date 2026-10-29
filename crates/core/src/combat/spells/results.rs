//! Results for combat/spells.
use super::AttackResult;
use centurions_content::Warrior;
use centurions_content::combat::MonsterRecord;

pub struct WarriorSpellResult {
    pub warriors: Vec<Warrior>,
    pub targets: Vec<(usize, AttackResult)>,
    pub race_excluded: Vec<usize>,
}

pub struct BodySpellResult {
    pub total_delta: i16,
    pub warriors: Vec<Warrior>,
    pub enemies: Vec<MonsterRecord>,
    pub sounds: Vec<i16>,
    pub targets: Vec<(usize, AttackResult)>,
    pub race_excluded: Vec<usize>,
}

pub struct SupportSpellResult {
    pub total_delta: i16,
    pub race_excluded: Vec<usize>,
    pub warriors: Vec<Warrior>,
    pub enemies: Vec<MonsterRecord>,
    pub sounds: Vec<i16>,
    pub targets: Vec<(usize, AttackResult)>,
    pub healed: bool,
    pub recovered: u8,
    /// spelllist15b9f6 effect68 stored fatigue 1 for the caller's updatefat.
    pub fatigue_reset: bool,
    /// Effect66 replaced a zero packet byte31 with its draw.
    pub area_shape: Option<u8>,
    /// Effect90 bodies killed inside spelllist, before any health change.
    pub forced_kills: Vec<usize>,
    /// Effect26/64 writes to warrior-indexed NPC WORDs, in spelllist order.
    pub(crate) alias_writes: Vec<crate::combat::warrior_alias::AliasWrite>,
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
