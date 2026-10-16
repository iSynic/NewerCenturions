use crate::{RandomState, RuleError, story_health};
use centurions_content::{Warrior, span, word};

pub fn reduce_outdoors(
    warriors: &[Warrior],
    context: &story_health::HealthContext,
    random: &mut RandomState,
    energy_recovery_disabled: u8,
) -> Result<story_health::HealthOutcome, RuleError> {
    if warriors.is_empty() || warriors.len() > 6 {
        return Err(RuleError::Invalid(
            "outdoor reduction requires one to six warriors".into(),
        ));
    }
    for warrior in warriors {
        span(&warrior.bytes, 0, 3138)?;
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
        for (offset, amount, address) in [(0x8a, None, 0x401948), (0x8c, Some(2), 0x40193a)] {
            let bytes = &mut outcome.warriors[member].bytes;
            let before = word(bytes, 0x50)?;
            let condition = word(bytes, offset)?;
            if condition != 0 && before > 0 {
                let amount = amount.unwrap_or(condition.wrapping_abs());
                bytes[0x50..0x52].copy_from_slice(&before.wrapping_sub(amount).to_be_bytes());
                outcome.context.effect = 7;
                if before.wrapping_sub(amount) < 1 {
                    outcome.context.unavailable = outcome.context.unavailable.wrapping_add(1);
                    outcome.context.last_body = member as i16;
                    outcome.events.push(story_health::HealthEvent::Sound(
                        next_random.bound(2).wrapping_add(506),
                    ));
                    outcome.events.push(story_health::HealthEvent::Flash {
                        address,
                        sound: 10122,
                    });
                    if outcome.context.unavailable >= outcome.warriors.len() as i8 {
                        outcome.events.push(story_health::HealthEvent::PartyLost);
                    }
                }
            }
        }
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
        energy(bytes, energy_recovery_disabled)?;
        if word(bytes, 0x98)? != 0 && word(bytes, 0x10)? != 0 {
            bytes[0x10..0x12].fill(0);
            outcome.context.unavailable = outcome.context.unavailable.wrapping_sub(1);
        }
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

pub(crate) fn energy(bytes: &mut [u8], recovery_disabled: u8) -> Result<(), RuleError> {
    if word(bytes, 0x50)? > 0 {
        let drain = word(bytes, 0xa2)?.wrapping_abs();
        if drain != 0 {
            let points = word(bytes, 0x58)?.wrapping_sub(drain).max(0);
            bytes[0x58..0x5a].copy_from_slice(&points.to_be_bytes());
        }
        let regeneration = word(bytes, 0xa0)?.wrapping_abs();
        if recovery_disabled == 0 && regeneration != 0 && word(bytes, 0x5a)? != 0 {
            let points = word(bytes, 0x58)?
                .wrapping_add(regeneration)
                .min(word(bytes, 0x5a)?);
            bytes[0x58..0x5a].copy_from_slice(&points.to_be_bytes());
        }
    }
    Ok(())
}
