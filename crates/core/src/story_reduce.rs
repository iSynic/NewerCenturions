use crate::{RandomState, RuleError, story_health};
use centurions_content::{
    WARRIOR_BYTES, Warrior,
    records::{Condition, WarriorRecord, WarriorRecordMut},
    span,
};

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
        span(&warrior.bytes, 0, WARRIOR_BYTES)?;
    }
    let mut outcome = story_health::HealthOutcome {
        warriors: warriors.to_vec(),
        context: context.clone(),
        events: Vec::new(),
        amounts: Vec::new(),
    };
    let mut next_random = random.clone();
    for member in 0..outcome.warriors.len() {
        let warrior = outcome.warriors[member].record();
        let stamina = warrior.stamina()?;
        let regeneration = warrior.condition(Condition::Regenerating)?;
        if regeneration != 0
            && stamina > -10
            && stamina < warrior.maximum_stamina()?
            && warrior.automatic_mode()? == 0
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
        for (condition, amount, address) in [
            (Condition::SeventhLevelProtection, None, 0x401948),
            (Condition::Speedy, Some(2), 0x40193a),
        ] {
            let warrior = &mut outcome.warriors[member];
            let before = warrior.record().stamina()?;
            let condition = warrior.record().condition(condition)?;
            if condition != 0 && before > 0 {
                let amount = amount.unwrap_or(condition.wrapping_abs());
                warrior
                    .record_mut()
                    .set_stamina(before.wrapping_sub(amount));
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
        let warrior = &mut outcome.warriors[member];
        energy(&mut warrior.bytes, energy_recovery_disabled)?;
        if warrior.record().confusion_duration()? != 0 && warrior.record().faction()? != 0 {
            warrior.record_mut().set_faction(0);
            outcome.context.unavailable = outcome.context.unavailable.wrapping_sub(1);
        }
        warrior.record_mut().decrement_temporary_conditions()?;
    }
    *random = next_random;
    Ok(outcome)
}

/// reduce drains energy before regeneration, while an unconscious warrior does neither.
pub(crate) fn energy(bytes: &mut [u8], recovery_disabled: u8) -> Result<(), RuleError> {
    if WarriorRecord::from_bytes(bytes).stamina()? > 0 {
        let drain = WarriorRecord::from_bytes(bytes)
            .condition(Condition::DrainPsi)?
            .wrapping_abs();
        if drain != 0 {
            let points = WarriorRecord::from_bytes(bytes)
                .psi_points()?
                .wrapping_sub(drain)
                .max(0);
            WarriorRecordMut::from_bytes(bytes).set_psi_points(points);
        }
        let warrior = WarriorRecord::from_bytes(bytes);
        let regeneration = warrior.condition(Condition::RechargePsi)?.wrapping_abs();
        if recovery_disabled == 0 && regeneration != 0 && warrior.maximum_psi()? != 0 {
            let points = warrior
                .psi_points()?
                .wrapping_add(regeneration)
                .min(warrior.maximum_psi()?);
            WarriorRecordMut::from_bytes(bytes).set_psi_points(points);
        }
    }
    Ok(())
}
