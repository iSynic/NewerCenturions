//! Spell results in the original combat execution order.
use crate::combat::{Combat, lifecycle};
use crate::{Event, RandomState, RuleError};
use centurions_content::{
    Warrior,
    records::{AttackPacket, AttackPacketMut, EnemyRecord},
};

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
        c.neighbor_work.departure = 0;
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
        if AttackPacket(identity.2).defense_category() == 8
            && AttackPacket(identity.2).effect()? == 0
            && target < w.len()
            && identity.3[target] >= 0
            && !outcome.killed
            && outcome.damage != 0
        {
            let caster = identity.0.unwrap_or(c.order[c.cursor]);
            let sound = if caster >= 10 {
                let enemy = &c.enemies.records[(caster - 10) as usize].bytes;
                if EnemyRecord::from_bytes(enemy).selected_item()? != 0 {
                    c.weapon_work.sound()
                } else {
                    c.weapon_work.natural_result_sound(enemy)?
                }
            } else {
                AttackPacket(identity.2).impact_sound()?
            };
            events.push(Event::Sound { resource_id: sound });
        }
        if outcome.killed {
            let feedback = if target >= 10 && enemy_defeat_feedback {
                super::death::enemy_death_feedback(&c.enemies.records[target - 10], target as i16)?
            } else {
                vec![]
            };
            lifecycle::kill_body(c, w, target as i16)?;
            events.extend(feedback);
        }
        // resolvespell151a88 presents successful zero-damage spelllist results.
        // showresults clears the interaction gate even when its code has no label.
        let effect = AttackPacket(identity.2).effect()?.wrapping_abs();
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
            icon: crate::event::PresentationIcon(outcome.icon),
            flight: Default::default(),
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
    c.mark_monster_spell_work(packet, mask);
    if AttackPacket(packet).targeting_mode() == 7 {
        c.ready = 0;
    }
    c.inactive_rounds = 0;
    if id == 1703 && count > 0 && crate::combat::spell_conditions::is_1703_packet(packet)? {
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
                    icon: crate::event::PresentationIcon(0),
                    flight: Default::default(),
                }),
        );
        return Ok(events);
    }
    let previous_hp = w
        .iter()
        .map(Warrior::stamina)
        .collect::<Result<Vec<_>, _>>()?;
    // NPC casts enter the same ordered condition/healing resolver as party casts.
    if crate::combat::spells::is_support(packet)? {
        let pending = std::mem::take(&mut c.alias_writes);
        let result = crate::combat::spells::resolve_support_for_caster(
            w,
            &c.enemies.records,
            mask,
            id,
            &crate::combat::spells::DirectSpell {
                packet,
                level: id % 1000 / 100 - 1,
                count,
                armor_filter: c.weapon_work.armor_filter(),
                weapon_sound: c.weapon_work.sound(),
            },
            (Some(caster), Some(memory.with_pending(&pending))),
            random,
        )?;
        c.alias_writes = pending;
        c.alias_writes.extend(result.alias_writes);
        w.clone_from_slice(&result.warriors);
        c.fatigue_reset |= result.fatigue_reset;
        if let Some(shape) = result.area_shape {
            AttackPacketMut(&mut c.attack_packet).set_area_shape(shape);
        }
        c.victory_work.add_total(result.total_delta);
        c.enemies.records = result.enemies;
        c.unavailable = c.unavailable.wrapping_sub(result.recovered as i8);
        if result.healed {
            AttackPacketMut(&mut c.attack_packet).set_health_effect(4);
        }
        if !result.targets.is_empty() {
            c.field_used[0] = 1;
        }
        let mut events = result
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
        return Ok(events);
    }
    let direct = crate::combat::spells::DirectSpell {
        packet,
        level: id % 1000 / 100 - 1,
        count,
        armor_filter: c.weapon_work.armor_filter(),
        weapon_sound: c.weapon_work.sound(),
    };
    // resolvespell's body loop has no category test; a category-5 drain reaches
    // spelllist 15ba5a through the drain resolver the party caster uses.
    let result = if matches!(AttackPacket(packet).effect()?, 60 | 61)
        && AttackPacket(packet).defense_category() == 5
    {
        crate::combat::spells::resolve_psi_drain(
            w,
            &c.enemies.records,
            mask,
            &direct,
            caster,
            (memory, None),
            random,
        )?
    } else {
        crate::combat::spells::resolve_bodies_with_memory(
            w,
            &c.enemies.records,
            mask,
            &direct,
            memory,
            random,
        )?
    };
    w.clone_from_slice(&result.warriors);
    c.victory_work.add_total(result.total_delta);
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
