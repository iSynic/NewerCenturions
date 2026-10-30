//! Showing spell results.
use crate::combat::attack::AttackResult;
use crate::combat::spells::DirectSpell;
use crate::combat::spells::RestrictionMemoryView;
use crate::combat::spells::conditions;
use crate::combat::spells::spell_level;
use crate::combat::{Combat, lifecycle};
use crate::{Event, RandomState, RuleError};
use centurions_content::{
    Warrior,
    records::{EnemyRecord, SpellInfoRecord, SpellInfoRecordMut},
};

/// Reports what a spell did to each target in order: the sound of any race-excluded body before
/// it, the target's sounds, the result sound for a party member who survives a damaging hit of a
/// category 8 spell with no effect, the kill of a fallen body, the result message of a spell that
/// succeeds without damage, and the resolved spell. Race-excluded bodies after the last target
/// sound at the end.
pub(crate) fn append_spell_outcomes(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    targets: Vec<(usize, AttackResult)>,
    race_excluded: Vec<usize>,
    identity: (Option<i16>, i16, &[u8], &[i16]),
    enemy_defeat_feedback: bool,
    events: &mut Vec<Event>,
) -> Result<(), RuleError> {
    let mut excluded = race_excluded.into_iter().peekable();
    for (target, outcome) in targets {
        combat.neighbors.departure = 0;
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
        // A party member who survives the hit plays the result sound.
        if SpellInfoRecord(identity.2).defense_category() == 8
            && SpellInfoRecord(identity.2).effect()? == 0
            && target < warriors.len()
            && identity.3[target] >= 0
            && !outcome.killed
            && outcome.damage != 0
        {
            let caster = identity
                .0
                .unwrap_or(combat.battle_order[combat.battle_order_index]);
            let sound = if caster >= 10 {
                let monster = &combat.roster.records[(caster - 10) as usize].bytes;
                if EnemyRecord::from_bytes(monster).weapon()? != 0 {
                    combat.weapon_traits.sound()
                } else {
                    combat.weapon_traits.natural_result_sound(monster)?
                }
            } else {
                SpellInfoRecord(identity.2).impact_sound()?
            };
            events.push(Event::Sound { resource_id: sound });
        }
        if outcome.killed {
            let feedback = if target >= 10 && enemy_defeat_feedback {
                super::death::enemy_death_feedback(
                    &combat.roster.records[target - 10],
                    target as i16,
                )?
            } else {
                vec![]
            };
            lifecycle::kill_body(combat, warriors, target as i16)?;
            events.extend(feedback);
        }
        // A spell that succeeds without damage still shows its result, and showing
        // a result clears the interaction gate even when it has no label.
        let effect = SpellInfoRecord(identity.2).effect()?.wrapping_abs();
        if matches!(effect, 1..=41 | 51 | 64) && outcome.damage == 0 {
            events.push(Event::Feedback(
                crate::feedback::GameFeedback::CombatResult {
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
            icon: crate::event::PresentationIcon(outcome.icon),
            flight: Default::default(),
        });
    }
    events.extend(excluded.map(|_| Event::Sound { resource_id: 651 }));
    Ok(())
}

/// Resolves a spell on the bodies the mask marks and returns its events.
///
/// Original: resolvespell
pub(crate) fn resolve_spell_mask(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    random: &mut RandomState,
    spell: (i16, &[u8], i16),
    mask: &[i16; 110],
    caster: i16,
    memory: RestrictionMemoryView<'_>,
) -> Result<Vec<Event>, RuleError> {
    let (id, packet, count) = spell;
    combat.mark_monster_spell_pick(packet, mask);
    if SpellInfoRecord(packet).is_party_duration() {
        combat.in_spell = 0;
    }
    combat.blank_round = 0;
    if id == 1703 && count > 0 && conditions::is_1703_packet(packet)? {
        let result = conditions::resolve_1703(
            warriors,
            &combat.roster.records,
            packet,
            count,
            mask,
            random,
        )?;
        warriors.clone_from_slice(&result.warriors);
        combat.roster.records = result.enemies;
        if mask.iter().any(|flag| *flag > 0) {
            combat.collide_check[0] = 1;
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
                    icon: crate::event::PresentationIcon(0),
                    flight: Default::default(),
                }),
        );
        return Ok(events);
    }
    let previous_hp = warriors
        .iter()
        .map(Warrior::stamina)
        .collect::<Result<Vec<_>, _>>()?;
    // Monster casts enter the same ordered condition/healing resolver as party casts.
    if crate::combat::spells::is_support(packet)? {
        let pending = std::mem::take(&mut combat.alias_writes);
        let result = crate::combat::spells::resolve_support_for_caster(
            warriors,
            &combat.roster.records,
            mask,
            id,
            &DirectSpell {
                packet,
                level: spell_level(id),
                count,
                armor_filter: combat.weapon_traits.armor_filter(),
                weapon_sound: combat.weapon_traits.sound(),
            },
            (Some(caster), Some(memory.with_pending(&pending))),
            random,
        )?;
        combat.alias_writes = pending;
        combat.alias_writes.extend(result.alias_writes);
        warriors.clone_from_slice(&result.warriors);
        combat.fatigue_reset |= result.fatigue_reset;
        if let Some(shape) = result.area_shape {
            SpellInfoRecordMut(&mut combat.spell_info).set_area_size(shape);
        }
        combat.kill_counts.add_num_enemy(result.total_delta);
        combat.roster.records = result.enemies;
        combat.kill_party = combat.kill_party.wrapping_sub(result.recovered as i8);
        if result.healed {
            SpellInfoRecordMut(&mut combat.spell_info).set_health_effect(4);
        }
        if !result.targets.is_empty() {
            combat.collide_check[0] = 1;
        }
        let mut events = result
            .sounds
            .into_iter()
            .map(|resource_id| Event::Sound { resource_id })
            .collect();
        append_spell_outcomes(
            combat,
            warriors,
            result.targets,
            result.race_excluded,
            (Some(caster), id, packet, &previous_hp),
            false,
            &mut events,
        )?;
        return Ok(events);
    }
    let direct = DirectSpell {
        packet,
        level: spell_level(id),
        count,
        armor_filter: combat.weapon_traits.armor_filter(),
        weapon_sound: combat.weapon_traits.sound(),
    };
    // Any category can drain; a category-5 drain uses the party's drain rules.
    let result = if matches!(SpellInfoRecord(packet).effect()?, 60 | 61)
        && SpellInfoRecord(packet).defense_category() == 5
    {
        crate::combat::spells::resolve_psi_drain(
            warriors,
            &combat.roster.records,
            mask,
            &direct,
            caster,
            (memory, None),
            random,
        )?
    } else {
        crate::combat::spells::resolve_bodies_with_memory(
            warriors,
            &combat.roster.records,
            mask,
            &direct,
            memory,
            random,
        )?
    };
    warriors.clone_from_slice(&result.warriors);
    combat.kill_counts.add_num_enemy(result.total_delta);
    combat.roster.records = result.enemies;
    if !result.targets.is_empty() {
        combat.collide_check[0] = 1;
    }
    let mut events: Vec<_> = result
        .sounds
        .into_iter()
        .map(|resource_id| Event::Sound { resource_id })
        .collect();
    append_spell_outcomes(
        combat,
        warriors,
        result.targets,
        result.race_excluded,
        (Some(caster), id, packet, &previous_hp),
        false,
        &mut events,
    )?;
    Ok(events)
}
