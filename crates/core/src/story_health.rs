use crate::{RandomState, RuleError};
use centurions_content::{Warrior, span, word};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HealthContext {
    pub unavailable: i8,
    pub selected_warrior: usize,
    pub effect: i8,
    pub last_body: i16,
    pub marks_affected: bool,
    pub affected: [i16; 6],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HealthEvent {
    Animation { warrior: usize, effect: i8 },
    Sound(i16),
    Defeated { warrior: usize, permanent: bool },
    PartyLost,
    Dialogue(i16),
    Flash { address: u32, sound: i16 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HealthOutcome {
    pub warriors: Vec<Warrior>,
    pub context: HealthContext,
    pub events: Vec<HealthEvent>,
    pub amounts: Vec<(usize, i16)>,
}

pub fn context(
    warriors: &[Warrior],
    selected_warrior: usize,
    previous: Option<&HealthContext>,
) -> HealthContext {
    let mut context = previous.cloned().unwrap_or(HealthContext {
        unavailable: warriors
            .iter()
            .filter(|warrior| warrior.stamina().unwrap_or(0) < 1)
            .count() as i8,
        selected_warrior,
        effect: 0,
        last_body: 0,
        marks_affected: false,
        affected: [0; 6],
    });
    context.selected_warrior = selected_warrior;
    context
}

fn validate(warriors: &[Warrior]) -> Result<(), RuleError> {
    if warriors.is_empty() || warriors.len() > 6 {
        return Err(RuleError::Invalid(
            "health action requires one to six warriors".into(),
        ));
    }
    for warrior in warriors {
        span(&warrior.bytes, 0, 3138)?;
    }
    Ok(())
}

fn heal_member(
    outcome: &mut HealthOutcome,
    member: usize,
    amount: i16,
    animate: bool,
    random: &mut RandomState,
) {
    let warrior = &mut outcome.warriors[member].bytes;
    let before = word(warrior, 0x50).unwrap();
    let maximum = word(warrior, 0x52).unwrap();
    let living = before > 0;
    let permanent = before < -9;
    let unconscious = !living && !permanent;
    if animate {
        outcome.context.effect = 4;
        outcome.events.push(HealthEvent::Animation {
            warrior: member,
            effect: 4,
        });
    }
    let mut after = if permanent {
        before
    } else {
        before.wrapping_add(amount)
    };
    after = after.max(-10);
    if after > 0 && unconscious {
        outcome.context.unavailable = outcome.context.unavailable.wrapping_sub(1);
    }
    after = after.min(maximum);
    warrior[0x50..0x52].copy_from_slice(&after.to_be_bytes());
    outcome.amounts.push((member, amount));
    if after < -9 && !permanent && !living {
        outcome
            .events
            .push(HealthEvent::Sound(random.bound(2).wrapping_add(506)));
    }
    if after < 1 && living {
        outcome.context.last_body = member as i16;
        outcome.events.push(HealthEvent::Defeated {
            warrior: member,
            permanent: after < -9,
        });
        outcome.context.unavailable = outcome.context.unavailable.wrapping_add(1);
        outcome
            .events
            .push(HealthEvent::Sound(random.bound(2).wrapping_add(506)));
        if outcome.context.unavailable > outcome.warriors.len() as i8 - 1 {
            outcome.events.push(HealthEvent::PartyLost);
        }
    }
    if outcome.context.unavailable > outcome.warriors.len() as i8 - 1 {
        outcome.events.push(HealthEvent::PartyLost);
    }
    if outcome.context.marks_affected {
        outcome.context.affected[member] = -1;
    }
}

/// Original outdoor heal 0x1287c2; combat body removal is a separate command boundary.
pub fn heal_outdoors(
    warriors: &[Warrior],
    member: usize,
    amount: i16,
    animate: bool,
    context: &HealthContext,
    random: &mut RandomState,
) -> Result<HealthOutcome, RuleError> {
    validate(warriors)?;
    if member >= warriors.len() {
        return Err(RuleError::Invalid("health target outside party".into()));
    }
    let mut outcome = HealthOutcome {
        warriors: warriors.to_vec(),
        context: context.clone(),
        events: Vec::new(),
        amounts: Vec::new(),
    };
    let mut next_random = random.clone();
    heal_member(&mut outcome, member, amount, animate, &mut next_random);
    *random = next_random;
    Ok(outcome)
}

/// Opcode16 visits every member; opcode15 visits nonzero selection-mask entries.
pub fn party_health(
    warriors: &[Warrior],
    extra: &[u8],
    all: bool,
    selected: &[i16; 6],
    context: &HealthContext,
    random: &mut RandomState,
) -> Result<HealthOutcome, RuleError> {
    validate(warriors)?;
    let extra = span(extra, 0, 10)?;
    let multiplier = word(extra, 0)?;
    let low = word(extra, 2)?;
    let high = word(extra, 4)?;
    let sound = word(extra, 6)?;
    let dialogue = word(extra, 8)?;
    let mut outcome = HealthOutcome {
        warriors: warriors.to_vec(),
        context: context.clone(),
        events: Vec::new(),
        amounts: Vec::new(),
    };
    outcome.context.effect = 15;
    let mut next_random = random.clone();
    for (member, mask) in selected.iter().take(warriors.len()).enumerate() {
        if !all && *mask == 0 {
            continue;
        }
        if sound != 0 {
            outcome.events.push(HealthEvent::Sound(sound));
        }
        let amount = next_random.range(low, high).wrapping_mul(multiplier);
        heal_member(&mut outcome, member, amount, true, &mut next_random);
    }
    if dialogue != 0 {
        outcome
            .events
            .push(HealthEvent::Dialogue(dialogue.wrapping_abs()));
    }
    *random = next_random;
    Ok(outcome)
}
