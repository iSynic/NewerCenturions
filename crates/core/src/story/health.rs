use crate::{RandomState, RuleError};
use centurions_content::{
    WARRIOR_BYTES, Warrior,
    records::{PartyHealthExtra, WarriorRecord, WarriorRecordMut},
    span,
};

/// The values health and healing leave behind for the next action: how many warriors are out of the
/// fight, the selected warrior, the last effect shown, the last body and the warriors marked
/// affected.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HealthContext {
    /// How many warriors are out of the fight.
    #[serde(rename = "unavailable")]
    pub kill_party: i8,
    /// The warrior selected.
    #[serde(rename = "selected_warrior")]
    pub char_select_new: usize,
    /// The last effect shown.
    pub effect: i8,
    /// The last warrior to fall.
    pub last_body: i16,
    /// Whether warriors affected are being marked.
    pub marks_affected: bool,
    /// The warriors marked as affected.
    pub affected: [i16; 6],
}

impl HealthContext {
    /// The values a new game starts with: all zero.
    pub(crate) fn startup() -> Self {
        Self {
            kill_party: 0,
            char_select_new: 0,
            effect: 0,
            last_body: 0,
            marks_affected: false,
            affected: [0; 6],
        }
    }
}

/// Copies the last health outcome with the newly selected warrior.
pub(crate) fn reselected_context(
    char_select_new: usize,
    previous: &HealthContext,
) -> HealthContext {
    let mut context = previous.clone();
    context.char_select_new = char_select_new;
    context
}

/// What a healing or damaging action shows the player: an animation on a warrior, a sound, a
/// warrior falling, the party being lost, a dialogue, or a flash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HealthEvent {
    Animation { warrior: usize, effect: i8 },
    Sound(i16),
    Defeated { warrior: usize, permanent: bool },
    PartyLost,
    Dialogue(i16),
    Flash { address: u32, sound: i16 },
}

/// The result of a health action: the warriors after it, the context it left, the events to show
/// and the amount each warrior received.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HealthOutcome {
    pub warriors: Vec<Warrior>,
    pub context: HealthContext,
    pub events: Vec<HealthEvent>,
    pub amounts: Vec<(usize, i16)>,
}

/// The health context for the selected warrior, continuing from the previous one when there is one.
pub fn context(
    warriors: &[Warrior],
    char_select_new: usize,
    previous: Option<&HealthContext>,
) -> HealthContext {
    let mut context = previous.cloned().unwrap_or(HealthContext {
        kill_party: warriors
            .iter()
            .filter(|warrior| {
                warrior.stamina().unwrap_or(0) < 1
                    || warrior.record().computer_control().unwrap_or(0) != 0
            })
            .count() as i8,
        char_select_new,
        effect: 0,
        last_body: 0,
        marks_affected: false,
        affected: [0; 6],
    });
    context.char_select_new = char_select_new;
    context
}

/// Checks that a health action has one to six full warrior records.
fn validate(warriors: &[Warrior]) -> Result<(), RuleError> {
    if warriors.is_empty() || warriors.len() > 6 {
        return Err(RuleError::Invalid(
            "health action requires one to six warriors".into(),
        ));
    }
    for warrior in warriors {
        span(&warrior.bytes, 0, WARRIOR_BYTES)?;
    }
    Ok(())
}

/// Changes one member's stamina by an amount, between -10 and the maximum. A warrior already at -10
/// or below stays dead; one raised above 0 from knocked out is no longer counted out of the fight;
/// one dropped to 0 or below falls and is counted out, which can lose the party.
fn heal_member(
    outcome: &mut HealthOutcome,
    party: usize,
    member: usize,
    amount: i16,
    animate: bool,
    random: &mut RandomState,
) {
    let warrior = &mut outcome.warriors[member].bytes;
    let before = WarriorRecord::from_bytes(warrior).stamina().unwrap();
    let maximum = WarriorRecord::from_bytes(warrior).stamina_max().unwrap();
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
        outcome.context.kill_party = outcome.context.kill_party.wrapping_sub(1);
    }
    after = after.min(maximum);
    WarriorRecordMut::from_bytes(warrior).set_stamina(after);
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
        outcome.context.kill_party = outcome.context.kill_party.wrapping_add(1);
        outcome
            .events
            .push(HealthEvent::Sound(random.bound(2).wrapping_add(506)));
        if outcome.context.kill_party > party as i8 - 1 {
            outcome.events.push(HealthEvent::PartyLost);
        }
    }
    if outcome.context.kill_party > party as i8 - 1 {
        outcome.events.push(HealthEvent::PartyLost);
    }
    if outcome.context.marks_affected {
        outcome.context.affected[member] = -1;
    }
}

/// Heals or harms the party outside combat; removing a body in combat is a separate
/// command.
///
/// Original: heal
pub fn heal_outdoors(
    warriors: &[Warrior],
    member: usize,
    amount: i16,
    animate: bool,
    context: &HealthContext,
    random: &mut RandomState,
) -> Result<HealthOutcome, RuleError> {
    heal_record(
        warriors,
        warriors.len(),
        member,
        amount,
        animate,
        context,
        random,
    )
}

/// Heals over the warrior table: `records` runs past the `party` members when the
/// member is a held record, and party loss counts only the members.
///
/// Original: heal
pub(crate) fn heal_record(
    warriors: &[Warrior],
    party: usize,
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
    heal_member(
        &mut outcome,
        party,
        member,
        amount,
        animate,
        &mut next_random,
    );
    *random = next_random;
    Ok(outcome)
}

/// Party Health visits every member; Selected Health visits the picked members.
pub fn party_health(
    warriors: &[Warrior],
    extra: &[u8],
    all: bool,
    selected: &[i16; 6],
    context: &HealthContext,
    random: &mut RandomState,
) -> Result<HealthOutcome, RuleError> {
    validate(warriors)?;
    let extra = PartyHealthExtra::new(extra)?;
    let multiplier = extra.multiplier()?;
    let low = extra.low()?;
    let high = extra.high()?;
    let sound = extra.sound()?;
    let dialogue = extra.dialogue()?;
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
        heal_member(
            &mut outcome,
            warriors.len(),
            member,
            amount,
            true,
            &mut next_random,
        );
    }
    if dialogue != 0 {
        outcome
            .events
            .push(HealthEvent::Dialogue(dialogue.wrapping_abs()));
    }
    *random = next_random;
    Ok(outcome)
}
