//! Psi powers cast on the party from the menu.
use super::{Event, RandomState, RuleError, cost, mode_allowed, packet};
use super::{OUTDOOR_PARTY_SPELLS, OUTDOOR_TARGET_SPELLS, PARTY_SPELLS};
use crate::combat::spells::spell_identity;
use centurions_content::WARRIOR_BYTES;
use centurions_content::mission::OpeningMission;
use centurions_content::{
    Warrior,
    records::{Condition, PsiSelectionMemoryMut, SpellInfoRecord},
};

/// A psi power cast on the party from the menu: who casts, which power, and its level.
#[derive(Clone, Copy, Debug)]
pub struct PartyCastRequest {
    pub warrior: usize,
    pub spell: i16,
    pub power: i16,
}

/// What casting on the party left: the warriors, the psi memory, the party's buffs, the packet and
/// its identity, the count, any scenario duration, and the events.
pub struct PartyCastResult {
    pub warriors: Vec<Warrior>,
    pub memory: Vec<u8>,
    pub buffs: Vec<u8>,
    pub packet: Vec<u8>,
    pub identity: [i16; 3],
    pub count: i16,
    pub scenario_duration: Option<i16>,
    pub events: Vec<Event>,
}

/// Casts a psi power from the menu on the party. A power that lasts on the party draws its
/// duration, the base plus a draw for each power level, and adds it to the buffs or to the scenario
/// duration (effect 110).
pub fn cast_party(
    warriors: &[Warrior],
    memory: &[u8],
    buffs: &[u8],
    mission: &OpeningMission,
    random: &mut RandomState,
    disabled: u8,
    request: PartyCastRequest,
) -> Result<PartyCastResult, RuleError> {
    let mut result = prepare_party(warriors, memory, buffs, mission, disabled, request)?;
    let packet = result.packet.clone();
    if !SpellInfoRecord(&packet).is_party_duration() {
        return Ok(result);
    }
    let mut next_random = random.clone();
    let (minimum, maximum) = SpellInfoRecord(&packet).duration_range();
    let mut duration = next_random.range(minimum, maximum);
    let (minimum, maximum) = SpellInfoRecord(&packet).power_duration_range();
    if minimum != 0 {
        for _ in 0..request.power {
            duration = duration.wrapping_add(next_random.range(minimum, maximum));
        }
    }
    let effect = SpellInfoRecord(&packet).effect()?;
    if effect == 110 {
        result.scenario_duration = Some(duration);
    } else {
        crate::combat::party_buffs::extend(&mut result.buffs, effect, duration)?;
    }
    *random = next_random;
    Ok(result)
}

/// Charges the power chosen from the menu; buttonchoice resolves it later.
///
/// Original: buttonchoice
pub(crate) fn prepare_party(
    warriors: &[Warrior],
    memory: &[u8],
    buffs: &[u8],
    mission: &OpeningMission,
    disabled: u8,
    request: PartyCastRequest,
) -> Result<PartyCastResult, RuleError> {
    if memory.len() != 480 || buffs.len() != 20 {
        return Err(RuleError::Invalid("outdoor psi input size".into()));
    }
    let packet = validate_party(warriors, mission, disabled, request)?;
    let actor = &warriors[request.warrior];
    let charge = cost(&packet, request.power)?;
    let identity = spell_identity(request.spell);
    let mut result = PartyCastResult {
        warriors: warriors.to_vec(),
        memory: memory.to_vec(),
        buffs: buffs.to_vec(),
        packet: packet.clone(),
        identity,
        count: request.power,
        scenario_duration: None,
        events: vec![Event::Sound {
            resource_id: SpellInfoRecord(&packet).launch_sound()?.wrapping_neg(),
        }],
    };
    let remaining = actor.psi_points()?.wrapping_sub(charge);
    result.warriors[request.warrior]
        .record_mut()
        .set_psi_points(remaining);
    PsiSelectionMemoryMut(&mut result.memory).remember_cast(
        request.warrior,
        identity,
        request.power,
    );
    Ok(result)
}

/// Checks that a warrior can cast this power from the menu: a known spell the menu offers, a power
/// level from 1 to 7, and a caster who is not helpless, scan-blocked, confused or under computer
/// control and has stamina and enough psi points, for a spell that may be cast out of combat.
/// Returns the spell's packet.
pub fn validate_party(
    warriors: &[Warrior],
    mission: &OpeningMission,
    disabled: u8,
    request: PartyCastRequest,
) -> Result<Vec<u8>, RuleError> {
    if (!PARTY_SPELLS.contains(&request.spell)
        && !OUTDOOR_PARTY_SPELLS.contains(&request.spell)
        && !OUTDOOR_TARGET_SPELLS.contains(&request.spell))
        || !(1..=7).contains(&request.power)
    {
        return Err(RuleError::Unsupported {
            action: "psi.party",
            dependency: "other outdoor learned spell or power",
        });
    }
    if !(1..=6).contains(&warriors.len()) {
        return Err(RuleError::Invalid("outdoor psi input size".into()));
    }
    let actor = warriors
        .get(request.warrior)
        .ok_or_else(|| RuleError::Invalid("outdoor psi warrior".into()))?;
    for warrior in warriors {
        centurions_content::span(&warrior.bytes, 0, WARRIOR_BYTES)?;
    }
    if disabled != 0
        || [
            actor.record().condition(Condition::Helpless),
            actor.record().condition(Condition::ScanBlocked),
            actor.record().computer_control(),
            actor.record().condition(Condition::Confused),
            actor.record().condition(Condition::UnlabelledScanBlock),
        ]
        .into_iter()
        .any(|value| value.unwrap_or(1) != 0)
        || actor.record().stamina()? <= 0
        || actor.record().psi_points()? <= 0
    {
        return Err(RuleError::Invalid("warrior cannot cast outdoor psi".into()));
    }
    if !actor.knows_spell(request.spell)? {
        return Err(RuleError::Invalid(
            "warrior has not learned this psi spell".into(),
        ));
    }
    let packet = packet(mission, request.spell)?;
    if !mode_allowed(&packet, false)? {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::GameFeedback::Warning { index: 22 }],
        });
    }
    if !((SpellInfoRecord(&packet).is_party_duration()
        && matches!(SpellInfoRecord(&packet).effect()?, 0..=9 | 110))
        || (SpellInfoRecord(&packet).target_type() == 11
            && matches!(SpellInfoRecord(&packet).effect()?, 0 | 70))
        || (matches!(SpellInfoRecord(&packet).target_type(), 0 | 1 | 3 | 4 | 9)
            && (matches!(SpellInfoRecord(&packet).effect()?, 1..=40 | 57 | 64))))
    {
        return Err(RuleError::Invalid("psi unavailable outdoors".into()));
    }
    let charge = cost(&packet, request.power)?;
    if actor.psi_points()? < charge {
        return Err(RuleError::Invalid("insufficient psi".into()));
    }
    Ok(packet)
}
