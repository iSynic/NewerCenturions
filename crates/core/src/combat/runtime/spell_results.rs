//! Spell results in the original combat execution order.
use crate::combat::{Combat, lifecycle};
use crate::{Event, RandomState, RuleError};
use centurions_content::combat::MonsterRecord;
use centurions_content::{Warrior, word};

pub(super) fn increment_stat(bytes: &mut [u8], offset: usize, amount: i32) {
    let value =
        i32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap()).wrapping_add(amount);
    bytes[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
}

pub(super) fn enemy_death_sound(monster: &MonsterRecord) -> Result<i16, RuleError> {
    centurions_content::span(&monster.bytes, 0, 288)?;
    if monster.bytes[0xab] != 0 {
        return Err(RuleError::Unsupported {
            action: "combat.death",
            dependency: "alternate enemy defeat feedback",
        });
    }
    Ok(word(&monster.bytes, 0x6a)?)
}

pub(crate) fn append_spell_outcomes(
    c: &mut Combat,
    w: &mut [Warrior],
    targets: Vec<(usize, crate::combat::attack::AttackResult)>,
    race_excluded: Vec<usize>,
    identity: (Option<i16>, i16, &[u8], &[i16]),
    enemy_defeat_feedback: bool,
    events: &mut Vec<Event>,
) -> Result<(), RuleError> {
    let mut excluded = race_excluded.into_iter().peekable();
    for (target, outcome) in targets {
        while excluded.peek().is_some_and(|body| *body < target) {
            excluded.next();
            events.push(Event::Sound { resource_id: 651 });
        }
        events.extend(
            outcome
                .sounds
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id }),
        );
        // Native result audio follows surviving party hits at 151bca/1587b8.
        if identity.2[19] == 8
            && word(identity.2, 40)? == 0
            && target < w.len()
            && identity.3[target] >= 0
            && !outcome.killed
            && outcome.damage != 0
        {
            let caster = identity.0.unwrap_or(c.order[c.cursor]);
            let sound = if caster >= 10 {
                let enemy = &c.enemies.records[(caster - 10) as usize].bytes;
                if word(enemy, 0x4e)? != 0 {
                    c.weapon_work.sound()?
                } else {
                    return Err(RuleError::Unsupported {
                        action: "combat.spell",
                        dependency: "noncaster category-eight result audio",
                    });
                }
            } else {
                word(identity.2, 38)?
            };
            events.push(Event::Sound { resource_id: sound });
        }
        if outcome.killed {
            let sound = if target >= 10 && enemy_defeat_feedback {
                Some(enemy_death_sound(&c.enemies.records[target - 10])?)
            } else {
                None
            };
            lifecycle::kill_body(c, w, target as i16)?;
            if let Some(resource_id) = sound {
                events.push(Event::Sound { resource_id });
            }
        }
        // resolvespell151a88 presents successful zero-damage spelllist results.
        // showresults clears the interaction gate even when its code has no label.
        let effect = word(identity.2, 40)?;
        if matches!(effect, 1..=41 | 51 | 64) && outcome.damage == 0 {
            events.push(Event::Feedback(
                crate::feedback::NativeFeedback::CombatResult {
                    body: target as i16,
                    code: effect,
                    amount: 0,
                    audible: false,
                },
            ));
        }
        events.push(Event::SpellResolved {
            caster: identity.0,
            spell: identity.1,
            target: target as i16,
            damage: outcome.damage,
            defeated: outcome.killed,
        });
    }
    events.extend(excluded.map(|_| Event::Sound { resource_id: 651 }));
    Ok(())
}

pub(crate) fn resolve_spell_mask(
    c: &mut Combat,
    w: &mut [Warrior],
    random: &mut RandomState,
    spell: (i16, &[u8], i16),
    mask: &[i16; 110],
    caster: i16,
    memory: crate::combat::spells::RestrictionMemoryView<'_>,
) -> Result<Vec<Event>, RuleError> {
    let (id, packet, count) = spell;
    c.inactive_rounds = 0;
    if id == 1703 {
        let result = crate::combat::spell_conditions::resolve_1703(
            w,
            &c.enemies.records,
            packet,
            count,
            mask,
            random,
        )?;
        w.clone_from_slice(&result.warriors);
        c.enemies.records = result.enemies;
        if mask.iter().any(|flag| *flag > 0) {
            c.field_used[0] = 1;
        }
        let mut events: Vec<_> = result
            .sounds
            .into_iter()
            .map(|resource_id| Event::Sound { resource_id })
            .collect();
        events.extend(
            mask.iter()
                .enumerate()
                .filter(|(_, flag)| **flag > 0)
                .map(|(target, _)| Event::SpellResolved {
                    caster: Some(caster),
                    spell: id,
                    target: target as i16,
                    damage: 0,
                    defeated: false,
                }),
        );
        return Ok(events);
    }
    let previous_hp = w
        .iter()
        .map(Warrior::stamina)
        .collect::<Result<Vec<_>, _>>()?;
    let result = crate::combat::spells::resolve_bodies_with_memory(
        w,
        &c.enemies.records,
        mask,
        &crate::combat::spells::DirectSpell {
            packet,
            level: id % 1000 / 100 - 1,
            count,
            armor_filter: c.weapon_work.armor_filter()?,
            weapon_sound: c.weapon_work.sound()?,
        },
        memory,
        random,
    )?;
    w.clone_from_slice(&result.warriors);
    c.enemies.records = result.enemies;
    if !result.targets.is_empty() {
        c.field_used[0] = 1;
    }
    let mut events: Vec<_> = result
        .sounds
        .into_iter()
        .map(|resource_id| Event::Sound { resource_id })
        .collect();
    append_spell_outcomes(
        c,
        w,
        result.targets,
        result.race_excluded,
        (Some(caster), id, packet, &previous_hp),
        false,
        &mut events,
    )?;
    Ok(events)
}
