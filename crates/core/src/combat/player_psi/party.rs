//! Party for combat/player_psi.
use super::{Event, RandomState, RuleError, cost, mode_allowed, packet, put_word};
use super::{OUTDOOR_PARTY_SPELLS, OUTDOOR_TARGET_SPELLS, PARTY_SPELLS};
use centurions_content::mission::OpeningMission;
use centurions_content::{Warrior, word};

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
            resource_id: word(&packet, 36)?.wrapping_neg(),
        }],
    };
    let remaining = actor.psi_points()?.wrapping_sub(charge);
    put_word(&mut result.warriors[request.warrior].bytes, 0x58, remaining);
    for (index, value) in identity.into_iter().chain([request.power]).enumerate() {
        put_word(
            &mut result.memory,
            request.warrior * 80 + 72 + index * 2,
            value,
        );
    }
    if packet[1] != 7 {
        return Ok(result);
    }
    let mut next_random = random.clone();
    let mut duration = next_random.range(i16::from(packet[13] as i8), i16::from(packet[14] as i8));
    if packet[15] != 0 {
        for _ in 0..request.power {
            duration = duration.wrapping_add(
                next_random.range(i16::from(packet[15] as i8), i16::from(packet[16] as i8)),
            );
        }
    }
    let effect = word(&packet, 40)?;
    if effect == 110 {
        result.scenario_duration = Some(duration);
    } else {
        crate::combat::party_buffs::extend(&mut result.buffs, effect, duration)?;
    }
    *random = next_random;
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
        || [0x60, 0x68, 0x90, 0x98, 0xac]
            .into_iter()
            .any(|offset| word(&actor.bytes, offset).unwrap_or(1) != 0)
        || word(&actor.bytes, 0x50)? <= 0
        || word(&actor.bytes, 0x58)? <= 0
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
    if !((packet[1] == 7 && matches!(word(&packet, 40)?, 0..=9 | 110))
        || (packet[1] == 11 && matches!(word(&packet, 40)?, 0 | 70))
        || (matches!(packet[1], 0 | 1 | 3 | 4 | 9)
            && (matches!(word(&packet, 40)?, 1..=40 | 57 | 64))))
    {
        return Err(RuleError::Invalid("psi unavailable outdoors".into()));
    }
    let charge = cost(&packet, request.power)?;
    if actor.psi_points()? < charge {
        return Err(RuleError::Invalid("insufficient psi".into()));
    }
    Ok(packet)
}
