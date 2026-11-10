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

/// Whether a warrior may cast, with the warnings that refuse him.
pub struct CastAdmission {
    pub allowed: bool,
    pub warnings: Vec<i16>,
}

/// Whether a squad member may cast now. No one outside the squad can, and no one while psi is
/// disabled (warning 113 says so unless `quiet`). A warrior who is confused, scan-blocked,
/// helpless or under computer control cannot, nor one without stamina or psi points. In a battle
/// a warrior who has already attacked cannot, nor one who has cast as many times as his limit
/// allows. Members past the six warriors are not tested.
///
/// Original: cancast
pub fn admission(
    warriors: &[Warrior],
    member: i16,
    disabled: u8,
    quiet: bool,
    in_combat: bool,
) -> Result<CastAdmission, RuleError> {
    let mut result = CastAdmission {
        allowed: false,
        warnings: vec![],
    };
    if member < 0 || member as usize >= warriors.len() {
        return Ok(result);
    }
    if disabled != 0 {
        if !quiet {
            result.warnings.push(113);
        }
        return Ok(result);
    }
    if member >= 6 {
        result.allowed = true;
        return Ok(result);
    }
    let w = &warriors[member as usize].bytes;
    centurions_content::span(w, 0, WARRIOR_BYTES)?;
    let record = WarriorRecord::from_bytes(w);
    result.allowed = [
        record.condition(Condition::Confused),
        record.condition(Condition::UnlabelledScanBlock),
        record.condition(Condition::Helpless),
        record.condition(Condition::ScanBlocked),
        record.computer_control(),
    ]
    .into_iter()
    .all(|value| value.is_ok_and(|value| value == 0))
        && record.stamina()? > 0
        && record.psi_points()? > 0
        && (!in_combat
            || record.been_attacked()? == 0
                && record.spells_so_far()? < record.max_spells_attacks()?);
    Ok(result)
}

/// Whether a squad member may cast now, asked without a warning.
///
/// Original: cancast
pub fn can_cast(
    warriors: &[Warrior],
    member: i16,
    disabled: u8,
    in_combat: bool,
) -> Result<bool, RuleError> {
    Ok(admission(warriors, member, disabled, true, in_combat)?.allowed)
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

/// The straight-line distance between two cells, cut to whole cells.
fn straight_distance(from: (i16, i16), to: (i16, i16)) -> f64 {
    let dx = f64::from(to.0.wrapping_sub(from.0));
    let dy = f64::from(to.1.wrapping_sub(from.1));
    (dx * dx + dy * dy).sqrt().trunc()
}

/// Whether a cell is within the range of another, measured as a straight line cut to whole cells.
///
/// Original: checkrange, getrangetobody
pub fn in_range(from: (i16, i16), to: (i16, i16), range: i16) -> bool {
    straight_distance(from, to) <= f64::from(range)
}

/// The distance in whole cells the range readout shows from one cell to another.
///
/// Original: flashrange
pub fn cell_distance(from: (i16, i16), to: (i16, i16)) -> i16 {
    straight_distance(from, to) as i32 as i16
}

/// The spell's attack packet from the mission's attack table.
pub(crate) fn packet(mission: &OpeningMission, spell: i16) -> Result<Vec<u8>, RuleError> {
    let tables = mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("psi content missing".into()))?;
    spells::load_packet(&tables.attacks, spell)
}

/// Pressing the Psi control asks whether the warrior may cast, warning 113 while psi is disabled:
/// the selected warrior outside a battle, the warrior whose turn it is in one. Outside a battle the
/// press sounds first and a refusal ends it there. In a battle a refusal gives back the psi a paid
/// fast spell from psi memory took, ends the fast spell and any spell under way, and sounds 143; an
/// admitted press sounds and clears the chosen targets, and a paid fast spell is then under way.
///
/// Original: buttonchoice, combatchoice
pub(crate) fn press(state: &mut crate::Session) -> Result<Vec<crate::Event>, RuleError> {
    use crate::{Event, feedback::GameFeedback};
    let in_combat = state.combat.is_some();
    let warrior = match &state.combat {
        Some(combat) => combat
            .active_body()
            .and_then(|body| usize::try_from(body).ok()),
        None => Some(state.char_select_new),
    }
    .filter(|&warrior| warrior < state.warriors.len() && state.exploration.is_some())
    .ok_or_else(|| RuleError::Invalid("the Psi control needs the acting warrior".into()))?;
    let admitted = admission(
        &state.warriors,
        warrior as i16,
        state.psi_disabled,
        false,
        in_combat,
    )?;
    let warnings = admitted
        .warnings
        .into_iter()
        .map(|index| GameFeedback::Warning { index });
    if !in_combat {
        let mut feedback = vec![GameFeedback::Sound { resource_id: 141 }];
        feedback.extend(warnings);
        if !admitted.allowed {
            return Err(RuleError::Refused { feedback });
        }
        return Ok(vec![Event::Sound { resource_id: 141 }]);
    }
    if !admitted.allowed {
        if state.globals.fast_spell != 0 && state.globals.memory_spell == 1 {
            let refund = cost(&state.globals.spell_info, state.globals.attack_count)?;
            let record = state.warriors[warrior].record();
            let points = record.psi_points()?.wrapping_add(refund);
            state.warriors[warrior].record_mut().set_psi_points(points);
        }
        state.globals.fast_spell = 0;
        state.set_in_spell(0);
        let mut feedback: Vec<_> = warnings.collect();
        feedback.push(GameFeedback::Sound { resource_id: 143 });
        return Err(RuleError::RefusedAfterEffects { feedback });
    }
    state.clear_targets();
    if state.globals.fast_spell != 0 {
        state.set_in_spell(1);
        state.globals.fast_spell = 0;
    }
    Ok(vec![Event::Sound { resource_id: 141 }])
}
