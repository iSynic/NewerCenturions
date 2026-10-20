//! Admission for combat/player_psi.
use super::{RuleError, spells};
use centurions_content::mission::OpeningMission;
use centurions_content::{Warrior, word};

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
    result.allowed = [0x98, 0xac, 0x60, 0x68, 0x90, 0x14]
        .into_iter()
        .all(|offset| word(w, offset).is_ok_and(|value| value == 0))
        && word(w, 0x50)? > 0
        && word(w, 0x58)? > 0
        && word(w, 0xac8)? < word(w, 0xac6)?;
    Ok(result)
}

pub fn can_cast(warriors: &[Warrior], member: i16, disabled: u8) -> Result<bool, RuleError> {
    Ok(admission(warriors, member, disabled)?.allowed)
}

pub fn cost(packet: &[u8], power: i16) -> Result<i16, RuleError> {
    centurions_content::span(packet, 0, 58)?;
    Ok(i16::from(packet[30]).wrapping_mul(power).wrapping_abs())
}

pub fn mode_allowed(packet: &[u8], combat: bool) -> Result<bool, RuleError> {
    centurions_content::span(packet, 0, 58)?;
    Ok(packet[if combat { 21 } else { 22 }] != 0)
}

pub fn range(packet: &[u8], power: i16) -> Result<i16, RuleError> {
    centurions_content::span(packet, 0, 58)?;
    Ok(i16::from(packet[2] as i8)
        .wrapping_add(i16::from(packet[3] as i8).wrapping_mul(power))
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
