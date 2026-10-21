//! Admission for combat/player_psi.
use super::{RuleError, spells};
use centurions_content::mission::OpeningMission;
use centurions_content::{
    Warrior,
    records::{AttackPacket, Condition, WarriorRecord},
};

#[derive(Clone, Copy, Debug)]
pub struct CastRequest {
    pub rotation: u8,
    pub spell: i16,
    pub power: i16,
    pub target: Option<(i16, i16)>,
}

pub struct CastAdmission {
    pub allowed: bool,
    pub sounds: Vec<i16>,
}

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
    centurions_content::span(w, 0, 3138)?;
    let record = WarriorRecord::from_bytes(w);
    result.allowed = [
        record.condition(Condition::Confused),
        record.condition(Condition::UnlabelledScanBlock),
        record.condition(Condition::Helpless),
        record.condition(Condition::ScanBlocked),
        record.automatic_mode(),
        record.damage_marker(),
    ]
    .into_iter()
    .all(|value| value.is_ok_and(|value| value == 0))
        && record.stamina()? > 0
        && record.psi_points()? > 0
        && record.scans_used()? < record.scan_capacity()?;
    Ok(result)
}

pub fn can_cast(warriors: &[Warrior], member: i16, disabled: u8) -> Result<bool, RuleError> {
    Ok(admission(warriors, member, disabled)?.allowed)
}

pub fn cost(packet: &[u8], power: i16) -> Result<i16, RuleError> {
    centurions_content::span(packet, 0, 58)?;
    Ok(AttackPacket(packet)
        .psi_cost()
        .wrapping_mul(power)
        .wrapping_abs())
}

pub fn mode_allowed(packet: &[u8], combat: bool) -> Result<bool, RuleError> {
    centurions_content::span(packet, 0, 58)?;
    Ok(AttackPacket(packet).mode_allowed(combat))
}

pub fn range(packet: &[u8], power: i16) -> Result<i16, RuleError> {
    centurions_content::span(packet, 0, 58)?;
    Ok(AttackPacket(packet)
        .base_range()
        .wrapping_add(AttackPacket(packet).range_per_power().wrapping_mul(power))
        .wrapping_abs())
}

pub fn in_range(from: (i16, i16), to: (i16, i16), range: i16) -> bool {
    let dx = f64::from(to.0.wrapping_sub(from.0));
    let dy = f64::from(to.1.wrapping_sub(from.1));
    (dx * dx + dy * dy).sqrt().trunc() <= f64::from(range)
}

pub(super) fn packet(mission: &OpeningMission, spell: i16) -> Result<Vec<u8>, RuleError> {
    let tables = mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("psi content missing".into()))?;
    spells::load_packet(&tables.attacks, spell)
}
