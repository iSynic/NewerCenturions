//! One selected body: damage channels, spelllist, then health.
use crate::combat::attack::AttackResult;
use crate::combat::{
    damage::{DamageFlags, DamagePacket, resolve_damage_with_amount},
    spells::{DirectSpell, RestrictionMemoryView, SupportSpellResult},
};
use crate::{RandomState, RuleError};
use centurions_content::records::SpellInfoRecord;

/// Resolves the spell on one body: its damage and conditions, then its health change. A conversion
/// that kills inside the spell is credited and followed by a second kill.
#[allow(clippy::too_many_arguments)]
pub(super) fn resolve(
    result: &mut SupportSpellResult,
    body: usize,
    context: &DirectSpell<'_>,
    effect: i16,
    duration: i16,
    amount: &mut i16,
    caller: (Option<i16>, Option<RestrictionMemoryView<'_>>),
    random: &mut RandomState,
) -> Result<(), RuleError> {
    let kills = result.forced_kills.len();
    let (total, sounds, _, icon) = damage_and_conditions(
        result, body, context, effect, duration, amount, caller, random,
    )?;
    // Conversion kills inside spelllist, after the shield sounds; the damage is then
    // still subtracted from the dead body, credited and followed by a second killbody.
    // Each entry is one killbody for the caller.
    let repeated = total != 0 && result.forced_kills.len() != kills;
    let sounds = if repeated {
        let spelllist = AttackResult {
            icon: 0,
            feedback: None,
            damage: 0,
            killed: true,
            sounds,
        };
        result.targets.push((body, spelllist));
        vec![]
    } else {
        sounds
    };
    let mut outcome = super::health::apply(result, body, context, total, sounds, random)?;
    outcome.killed |= !repeated && result.forced_kills.contains(&body);
    outcome.icon = icon;
    result.targets.push((body, outcome));
    Ok(())
}

/// Rolls the spell's damage on one body through its shield, armor and saves, then applies its
/// conditions.
#[allow(clippy::too_many_arguments)]
pub(in crate::combat::spells) fn damage_and_conditions(
    result: &mut SupportSpellResult,
    body: usize,
    context: &DirectSpell<'_>,
    effect: i16,
    duration: i16,
    amount: &mut i16,
    caller: (Option<i16>, Option<RestrictionMemoryView<'_>>),
    random: &mut RandomState,
) -> Result<(i16, Vec<i16>, bool, i16), RuleError> {
    let record = result.body_bytes_mut(body);
    let damage = resolve_damage_with_amount(
        record,
        body >= 10,
        &DamagePacket {
            bytes: context.packet.to_vec(),
            attack_count: context.count,
        },
        DamageFlags {
            shield: true,
            armor: true,
            saves: true,
            armor_filter: context.armor_filter,
        },
        *amount,
        random,
    )?;
    let mut total = if effect == 28 { duration } else { damage.total };
    let reported = super::conditions::apply(
        result,
        body,
        (effect, context.count),
        duration,
        &mut total,
        caller,
        random,
    )?;
    *amount = total;
    let mut sounds = if damage.shield_sound {
        vec![206]
    } else {
        vec![]
    };
    if reported && total == 0 {
        sounds.push(SpellInfoRecord(context.packet).impact_sound()?);
    }
    // The fourth value is the damage icon showresults displays.
    Ok((total, sounds, reported, damage.sound))
}
