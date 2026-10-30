//! Whether a warrior may cast a psi power in battle.
use super::{RuleError, spells};
use centurions_content::WARRIOR_BYTES;
use centurions_content::mission::OpeningMission;
use centurions_content::{
    Warrior,
    records::{Condition, SpellInfoRecord, WarriorRecord},
};

/// A psi power to cast: the spell and its power level, the cell it is aimed at, and the quarter
/// turns it is rotated.
#[derive(Clone, Copy, Debug)]
pub struct CastRequest {
    pub rotation: u8,
    pub spell: i16,
    pub power: i16,
    pub target: Option<(i16, i16)>,
}

/// Whether a warrior may cast, with the sounds that refuse him.
pub struct CastAdmission {
    pub allowed: bool,
    pub sounds: Vec<i16>,
}

/// Whether a warrior may cast now. He cannot while psi is disabled (a refusal sound plays),
/// confused, scan-blocked, helpless, under computer control or already attacked, nor without
/// stamina or psi points, nor after casting as many times as his limit allows.
pub fn admission(
    warriors: &[Warrior],
    member: i16,
    disabled: u8,
) -> Result<CastAdmission, RuleError> {
    let mut result = CastAdmission {
        allowed: false,
        sounds: vec![],
    };
    if !(0..6).contains(&member) {
        return Ok(result);
    }
    if disabled != 0 {
        result.sounds.push(113);
        return Ok(result);
    }
    let w = &warriors
        .get(member as usize)
        .ok_or_else(|| RuleError::Invalid("psi warrior missing".into()))?
        .bytes;
    centurions_content::span(w, 0, WARRIOR_BYTES)?;
    let record = WarriorRecord::from_bytes(w);
    result.allowed = [
        record.condition(Condition::Confused),
        record.condition(Condition::UnlabelledScanBlock),
        record.condition(Condition::Helpless),
        record.condition(Condition::ScanBlocked),
        record.computer_control(),
        record.been_attacked(),
    ]
    .into_iter()
    .all(|value| value.is_ok_and(|value| value == 0))
        && record.stamina()? > 0
        && record.psi_points()? > 0
        && record.spells_so_far()? < record.max_spells_attacks()?;
    Ok(result)
}

/// Whether a warrior may cast now.
pub fn can_cast(warriors: &[Warrior], member: i16, disabled: u8) -> Result<bool, RuleError> {
    Ok(admission(warriors, member, disabled)?.allowed)
}

/// The psi points a power costs at a power level.
pub fn cost(packet: &[u8], power: i16) -> Result<i16, RuleError> {
    centurions_content::span(packet, 0, 58)?;
    Ok(SpellInfoRecord(packet)
        .cost()
        .wrapping_mul(power)
        .wrapping_abs())
}

/// Whether the spell may be cast in this setting, battle or out of combat.
pub fn mode_allowed(packet: &[u8], combat: bool) -> Result<bool, RuleError> {
    centurions_content::span(packet, 0, 58)?;
    Ok(SpellInfoRecord(packet).mode_allowed(combat))
}

/// How far a power reaches at a power level: the base range plus the range for each level.
pub fn range(packet: &[u8], power: i16) -> Result<i16, RuleError> {
    centurions_content::span(packet, 0, 58)?;
    Ok(SpellInfoRecord(packet)
        .range_base()
        .wrapping_add(
            SpellInfoRecord(packet)
                .range_per_power()
                .wrapping_mul(power),
        )
        .wrapping_abs())
}

/// Whether a cell is within the range of another, measured as a straight line cut to whole cells.
pub fn in_range(from: (i16, i16), to: (i16, i16), range: i16) -> bool {
    let dx = f64::from(to.0.wrapping_sub(from.0));
    let dy = f64::from(to.1.wrapping_sub(from.1));
    (dx * dx + dy * dy).sqrt().trunc() <= f64::from(range)
}

/// The spell's attack packet from the mission's attack table.
pub(super) fn packet(mission: &OpeningMission, spell: i16) -> Result<Vec<u8>, RuleError> {
    let tables = mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("psi content missing".into()))?;
    spells::load_packet(&tables.attacks, spell)
}
