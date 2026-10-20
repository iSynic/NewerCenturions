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
    pub warriors: Vec<Warrior>,
    pub enemies: Vec<MonsterRecord>,
    pub sounds: Vec<i16>,
    pub targets: Vec<(usize, AttackResult)>,
    pub race_excluded: Vec<usize>,
}

pub struct SupportSpellResult {
    pub warriors: Vec<Warrior>,
    pub enemies: Vec<MonsterRecord>,
    pub sounds: Vec<i16>,
    pub targets: Vec<(usize, AttackResult)>,
    pub healed: bool,
    pub recovered: u8,
}
