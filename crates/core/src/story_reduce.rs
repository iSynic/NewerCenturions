use crate::{RandomState, RuleError, story_health};
use centurions_content::{Warrior, span, word};

pub fn reduce_outdoors(
    warriors: &[Warrior],
    context: &story_health::HealthContext,
    random: &mut RandomState,
) -> Result<story_health::HealthOutcome, RuleError> {
    if warriors.is_empty() || warriors.len() > 6 {
        return Err(RuleError::Invalid(
            "outdoor reduction requires one to six warriors".into(),
        ));
    }
    for warrior in warriors {
        span(&warrior.bytes, 0, 3138)?;
        if [0x8a, 0x8c, 0x98, 0xa0, 0xa2]
            .iter()
            .any(|offset| word(&warrior.bytes, *offset).unwrap_or(1) != 0)
        {
            return Err(RuleError::Unsupported {
                action: "time.reduce",
                dependency: "active damage, stun recovery or energy effects",
            });
        }
    }
    let mut outcome = story_health::HealthOutcome {
        warriors: warriors.to_vec(),
        context: context.clone(),
        events: Vec::new(),
        amounts: Vec::new(),
    };
    let mut next_random = random.clone();
    for member in 0..outcome.warriors.len() {
        let bytes = &outcome.warriors[member].bytes;
        let stamina = word(bytes, 0x50)?;
        let regeneration = word(bytes, 0x66)?;
        if regeneration != 0
            && stamina > -10
            && stamina < word(bytes, 0x52)?
            && word(bytes, 0x90)? == 0
        {
            let healed = story_health::heal_outdoors(
                &outcome.warriors,
                member,
                regeneration.wrapping_abs(),
                false,
                &outcome.context,
                &mut next_random,
            )?;
            outcome.warriors = healed.warriors;
            outcome.context = healed.context;
            outcome.events.extend(healed.events);
            outcome.amounts.extend(healed.amounts);
        }
        let disease = crate::combat::natural_conditions::disease_outdoors(
            &outcome.warriors,
            member,
            &outcome.context,
            &mut next_random,
        )?;
        outcome.warriors = disease.warriors;
        outcome.context = disease.context;
        outcome.events.extend(disease.events);
        outcome.amounts.extend(disease.amounts);
        let poison = crate::combat::natural_conditions::poison_outdoors(
            &outcome.warriors,
            member,
            &outcome.context,
            &mut next_random,
        )?;
        outcome.warriors = poison.warriors;
        outcome.context = poison.context;
        outcome.events.extend(poison.events);
        outcome.amounts.extend(poison.amounts);
        let bytes = &mut outcome.warriors[member].bytes;
        for offset in (0x5e..0xae).step_by(2) {
            let duration = word(bytes, offset)?;
            if duration > 0 {
                bytes[offset..offset + 2].copy_from_slice(&(duration - 1).to_be_bytes());
            }
        }
    }
    *random = next_random;
    Ok(outcome)
}
