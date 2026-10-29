//! Party for combat/player_psi.
use super::{Event, RandomState, RuleError, cost, mode_allowed, packet};
use super::{OUTDOOR_PARTY_SPELLS, OUTDOOR_TARGET_SPELLS, PARTY_SPELLS};
use centurions_content::mission::OpeningMission;
use centurions_content::{
    Warrior,
    records::{AttackPacket, Condition, PsiSelectionMemoryMut},
};

#[derive(Clone, Copy, Debug)]
pub struct PartyCastRequest {
    pub warrior: usize,
    pub spell: i16,
    pub power: i16,
}

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
    if AttackPacket(&packet).targeting_mode() != 7 {
        return Ok(result);
    }
    let mut next_random = random.clone();
    let (minimum, maximum) = AttackPacket(&packet).duration_bounds();
    let mut duration = next_random.range(minimum, maximum);
    let (minimum, maximum) = AttackPacket(&packet).additional_duration_bounds();
    if minimum != 0 {
        for _ in 0..request.power {
            duration = duration.wrapping_add(next_random.range(minimum, maximum));
        }
    }
    let effect = AttackPacket(&packet).effect()?;
    if effect == 110 {
        result.scenario_duration = Some(duration);
    } else {
        crate::combat::party_buffs::extend(&mut result.buffs, effect, duration)?;
    }
    *random = next_random;
    Ok(result)
}

/// Charges the menu selection; buttonchoice owns the later resolution and ally copies.
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
    let identity = [
        request.spell / 1000 - 1,
        request.spell % 1000 / 100 - 1,
        request.spell % 100 - 1,
    ];
    let mut result = PartyCastResult {
        warriors: warriors.to_vec(),
        memory: memory.to_vec(),
        buffs: buffs.to_vec(),
        packet: packet.clone(),
        identity,
        count: request.power,
        scenario_duration: None,
        events: vec![Event::Sound {
            resource_id: AttackPacket(&packet).launch_sound()?.wrapping_neg(),
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
        centurions_content::span(&warrior.bytes, 0, 3138)?;
    }
    if disabled != 0
        || [
            actor.record().condition(Condition::Helpless),
            actor.record().condition(Condition::ScanBlocked),
            actor.record().automatic_mode(),
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
            feedback: vec![crate::feedback::NativeFeedback::Warning { index: 22 }],
        });
    }
    if !((AttackPacket(&packet).targeting_mode() == 7
        && matches!(AttackPacket(&packet).effect()?, 0..=9 | 110))
        || (AttackPacket(&packet).targeting_mode() == 11
            && matches!(AttackPacket(&packet).effect()?, 0 | 70))
        || (matches!(AttackPacket(&packet).targeting_mode(), 0 | 1 | 3 | 4 | 9)
            && (matches!(AttackPacket(&packet).effect()?, 1..=40 | 57 | 64))))
    {
        return Err(RuleError::Invalid("psi unavailable outdoors".into()));
    }
    let charge = cost(&packet, request.power)?;
    if actor.psi_points()? < charge {
        return Err(RuleError::Invalid("insufficient psi".into()));
    }
    Ok(packet)
}
