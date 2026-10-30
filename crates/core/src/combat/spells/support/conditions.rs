//! A spell's condition effects (spelllist), after the duration and before health.
use crate::combat::spells::effects;
use crate::combat::warrior_alias::{self, Alias, AliasWrite};
use crate::combat::{
    spell_conditions,
    spells::{RestrictionMemoryView, SupportSpellResult},
};
use crate::{RandomState, RuleError};
use centurions_content::records::warrior_layout::{COMPUTER_CONTROL, STAMINA, STAMINA_MAX};
use centurions_content::records::{Condition, EnemyRecord, EnemyRecordMut, WarriorRecord};

/// Refuses a condition case the reconstruction does not carry.
fn unavailable(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.spell.support",
        dependency,
    }
}

/// Applies a spell's condition effect to a body after its duration: adds the duration, clears the
/// condition an effect above 99 names, and sends a spell back at its caster when the body reflects
/// scans (34 percent). Plain damage and healing add no condition.
#[allow(clippy::too_many_arguments)]
pub(super) fn apply(
    result: &mut SupportSpellResult,
    body: usize,
    (effect, count): (i16, i16),
    duration: i16,
    damage: &mut i16,
    caller: (Option<i16>, Option<RestrictionMemoryView<'_>>),
    random: &mut RandomState,
) -> Result<bool, RuleError> {
    if effect == 0 || effect == effects::HEALING {
        return Ok(false);
    }
    let (caster, memory) = caller;
    if (1..=40).contains(&effect) {
        let record = result.body_bytes_mut(body);
        spell_conditions::add_duration(record, body >= 10, effect, duration)?;
    } else if effect > 99 {
        // Effects above 99 clear condition `effect - 101` before spelllist.
        let record = result.body_bytes_mut(body);
        spell_conditions::clear_selector_slot(record, body >= 10, effect)?;
    }
    let reflected = body < 10
        && WarriorRecord::from_bytes(&result.warriors[body].bytes)
            .condition(Condition::ReflectingScans)?
            != 0
        && random.bound(100) < 34;
    if effect == effects::FATIGUE_RESET {
        result.fatigue_reset = true;
        return Ok(false);
    }
    if reflected && caster.is_none() && effects::acts_on_target(effect) {
        return Err(unavailable("reflected condition caster is unavailable"));
    }
    let target = if reflected {
        usize::try_from(caster.unwrap_or(body as i16))
            .map_err(|_| RuleError::Invalid("negative condition caster".into()))?
    } else {
        body
    };
    if target >= result.warriors.len() && target < 10 || target >= 10 + result.enemies.len() {
        return Err(RuleError::Unsupported {
            action: "combat.spell.support",
            dependency: "condition target is an inactive warrior or unloaded NPC record",
        });
    }
    if let Some(memory) = memory
        && effects::reads_monster_record(effect)
    {
        memory.require_monster(target)?;
    }
    if effects::reads_target_stamina(effect) {
        *damage = if target < 10 {
            WarriorRecord::from_bytes(&result.warriors[target].bytes).stamina()?
        } else {
            EnemyRecord::from_bytes(&result.enemies[target - 10].bytes).stamina()?
        }
        .wrapping_add(10);
    }
    if effect == 90 {
        return convert(result, target, count, (caster, memory), random);
    }
    if matches!(effect, 59..=61) {
        // The amount moves Psi and returns unreported, without damage.
        let amount = if effect == 59 {
            *damage
        } else {
            damage.wrapping_neg()
        };
        let caster = memory
            .and_then(RestrictionMemoryView::selected_party_caster)
            .or(caster);
        spell_conditions::move_psi(&mut result.warriors, &mut result.enemies, target, amount)?;
        if effect == 61 && target >= 10 {
            let caster = usize::try_from(caster.ok_or("siphon caller missing")?)
                .map_err(|_| RuleError::Invalid("negative siphon caster".into()))?;
            spell_conditions::add_psi(
                &mut result.warriors,
                &mut result.enemies,
                caster,
                amount / 2,
            )?;
        }
        *damage = 0;
        return Ok(false);
    }
    if matches!(effect, 51 | 52) {
        let selected = memory
            .and_then(RestrictionMemoryView::selected_party_caster)
            .or(caster);
        let caster = usize::try_from(selected.ok_or("charm caller missing")?)
            .map_err(|_| RuleError::Invalid("negative charm caster".into()))?;
        if let Some(memory) = memory {
            memory.require_monster(caster)?;
        }
        let prior = if target < 10 {
            result.warriors[target].record().faction()?
        } else {
            result.enemies[target - 10].record().faction()?
        };
        let delta = spell_conditions::change_allegiance(
            &mut result.warriors,
            &mut result.enemies,
            target,
            caster,
        )?;
        result.recovered = result.recovered.wrapping_sub(delta as u8);
        let after = if target < 10 {
            result.warriors[target].record().faction()?
        } else {
            result.enemies[target - 10].record().faction()?
        };
        result.total_delta = result
            .total_delta
            .wrapping_add(i16::from(after != 0) - i16::from(prior != 0));
    } else if matches!(effect, 53 | 54) {
        let record = result.body_bytes_mut(target);
        spell_conditions::stun(record, target >= 10, duration)?;
    } else if target >= 10 && matches!(effect, effects::QUARTER_REVIVAL | effects::REVIVAL) {
        // Revival and necromancy address the warrior table even for a monster body, so
        // their writes land past it (docs/memory-alias-audit.md).
        let memory = memory.ok_or_else(|| {
            unavailable("warrior-indexed NPC alias outside the represented owners")
        })?;
        warrior_indexed(result, memory, target, effect)?;
    } else {
        let record = result.body_bytes_mut(target);
        spell_conditions::follow_up(record, target >= 10, effect)?;
    }
    Ok(true)
}

/// A party caster may convert or kill a hostile category-1 or 2 monster.
///
/// Original: spelllist
fn convert(
    result: &mut SupportSpellResult,
    target: usize,
    count: i16,
    caller: (Option<i16>, Option<RestrictionMemoryView<'_>>),
    random: &mut RandomState,
) -> Result<bool, RuleError> {
    if target < 10 {
        return Err(unavailable(
            "effect90 aimed at a party body reads Toolbox globals before the NPC array",
        ));
    }
    let (caster, memory) = caller;
    let caster = memory
        .and_then(RestrictionMemoryView::char_up)
        .or(caster)
        .and_then(|body| usize::try_from(body).ok())
        .filter(|body| *body < result.warriors.len())
        .ok_or_else(|| unavailable("effect90 selected warrior outside the squad records"))?;
    let monster = EnemyRecord::from_bytes(&result.enemies[target - 10].bytes);
    if !(monster.has_creature_category(1) || monster.has_creature_category(2))
        || monster.faction()? == 0
        || monster.survivor_marker()? == 0xff
        || monster.stamina()? <= 0
    {
        return Ok(true);
    }
    let warrior = WarriorRecord::from_bytes(&result.warriors[caster].bytes);
    let power = warrior
        .level()?
        .wrapping_mul(3)
        .wrapping_add(count.wrapping_mul(5));
    let chance = monster
        .hit_dice()?
        .wrapping_mul(5)
        .wrapping_add(100i16.wrapping_sub(power))
        .max(25);
    let roll = random.bound(100).wrapping_sub(chance);
    // A failed conversion returns unreported, before the impact feedback.
    if roll < 1 {
        return Ok(false);
    }
    let faction = warrior.faction()?;
    let record = &mut result.enemies[target - 10];
    if roll < 30 {
        EnemyRecordMut::from_bytes(&mut record.bytes).set_stamina(0);
        result.forced_kills.push(target);
    } else {
        result.total_delta = result.total_delta.wrapping_sub(1);
        result.sounds.push(630);
        EnemyRecordMut::from_bytes(&mut record.bytes).set_faction(faction);
    }
    Ok(true)
}

/// Necromancy reads stamina and, below -9, puts the body on automatic (-1) with a
/// quarter of its maximum stamina. Revival reads stamina then automatic and, when
/// either admits it, clears automatic and sets stamina to -9. Writes are recorded
/// for the caller in order.
///
/// Original: spelllist
fn warrior_indexed(
    result: &mut SupportSpellResult,
    memory: RestrictionMemoryView<'_>,
    body: usize,
    effect: i16,
) -> Result<(), RuleError> {
    let read = |result: &SupportSpellResult, field: usize| match result
        .alias_writes
        .iter()
        .rfind(|write| write.body == body && write.field == field)
    {
        Some(write) => Ok(write.value),
        None => memory.alias_word(body, field, |alias| match alias {
            Alias::AboveA5 => unavailable("warrior-indexed NPC alias above the A5 world"),
            _ => unavailable("warrior-indexed NPC alias outside the represented owners"),
        }),
    };
    let writes = if effect == 26 {
        if read(result, STAMINA)? >= -9 {
            return Ok(());
        }
        let maximum = read(result, STAMINA_MAX)?;
        [(COMPUTER_CONTROL, -1), (STAMINA, maximum / 4)]
    } else {
        if read(result, STAMINA)? >= -9 && read(result, COMPUTER_CONTROL)? == 0 {
            return Ok(());
        }
        [(COMPUTER_CONTROL, 0), (STAMINA, -9)]
    };
    for (field, value) in writes {
        match warrior_alias::locate(body, field) {
            Alias::LoadedItem(_) | Alias::TheShop(_) | Alias::MapHeader(_) => {}
            Alias::EditorBackup(_) if memory.holds_table() => {}
            Alias::EditorBackup(_) => {
                return Err(unavailable(
                    "warrior-indexed NPC write into the editor backup without the session table",
                ));
            }
            Alias::Record { .. } => return Err("NPC body inside the party records".into()),
            Alias::Definition { .. } => {
                return Err(unavailable(
                    "warrior-indexed NPC write into the item definition cache",
                ));
            }
            Alias::AboveA5 => {
                return Err(unavailable("warrior-indexed NPC alias above the A5 world"));
            }
            Alias::Unrepresented => {
                return Err(unavailable(
                    "warrior-indexed NPC alias outside the represented owners",
                ));
            }
        }
        result.alias_writes.push(AliasWrite { body, field, value });
    }
    Ok(())
}
