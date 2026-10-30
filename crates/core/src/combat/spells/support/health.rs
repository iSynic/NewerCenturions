//! Health changes after a spell's condition and reflection work.
use crate::combat::attack::AttackResult;
use crate::combat::spells::{DirectSpell, SupportSpellResult};
use crate::{RandomState, RuleError};
use centurions_content::records::{
    EnemyRecord, EnemyRecordMut, SpellInfoRecord, WarriorRecord, WarriorRecordMut,
};

/// Applies a spell's damage (healing, when negative) to a body's stamina and gives the result. A
/// warrior at -10 or below is untouched and only one at 0 or above can be harmed; healing stops at
/// his maximum, and a warrior healed from 0 or below to above 0 is counted as recovered. A hit
/// plays the spell's impact sound (except defense category 9).
pub(super) fn apply(
    result: &mut SupportSpellResult,
    body: usize,
    context: &DirectSpell<'_>,
    damage: i16,
    mut sounds: Vec<i16>,
    random: &mut RandomState,
) -> Result<AttackResult, RuleError> {
    let attack = SpellInfoRecord(context.packet);
    let mut killed = false;
    if body < 10 {
        let record = &mut result.warriors[body].bytes;
        let previous = WarriorRecord::from_bytes(record).stamina()?;
        if damage != 0 && previous > -10 {
            if attack.defense_category() != 9 {
                sounds.push(attack.impact_sound()?);
            }
            if damage < 0 {
                let after = previous
                    .wrapping_add(damage.wrapping_abs())
                    .max(-10)
                    .min(WarriorRecord::from_bytes(record).stamina_max()?);
                if previous <= 0 && after > 0 {
                    result.recovered = result.recovered.wrapping_add(1);
                }
                if previous <= 0 && after < -9 {
                    sounds.push(random.bound(2).wrapping_add(506));
                }
                WarriorRecordMut::from_bytes(record).set_stamina(after);
                killed = previous > 0 && after < 1;
                result.healed = true;
            } else if previous >= 0 {
                let after = previous.wrapping_sub(damage);
                WarriorRecordMut::from_bytes(record).set_stamina(after);
                WarriorRecordMut::from_bytes(record).mark_been_attacked();
                killed = previous > 0 && after < 1;
            }
        }
    } else {
        let record = &mut result.enemies[body - 10].bytes;
        if damage != 0 {
            let after = EnemyRecord::from_bytes(record)
                .stamina()?
                .wrapping_sub(damage);
            if attack.defense_category() != 9 {
                sounds.push(attack.impact_sound()?);
            }
            EnemyRecordMut::from_bytes(record).mark_been_attacked();
            EnemyRecordMut::from_bytes(record).set_stamina(after);
            killed = after < 1;
            if !killed {
                sounds.push(context.weapon_sound);
                if damage < 0 {
                    sounds.push(attack.impact_sound()?);
                }
            }
        }
    }
    Ok(AttackResult {
        icon: 0,
        feedback: None,
        damage,
        killed,
        sounds,
    })
}
