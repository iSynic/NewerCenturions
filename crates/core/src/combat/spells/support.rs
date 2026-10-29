//! Shared condition/healing resolution in resolvespell's body order.
use super::{
    DirectSpell, FieldUsage, RandomState, RestrictionMemoryView, RuleError, SupportSpellResult,
    monster_race_is_excluded, race_is_excluded, status_resisted,
};
use centurions_content::{
    Warrior,
    combat::MonsterRecord,
    records::{AttackPacket, WarriorRecord, WarriorRecordMut},
};

mod conditions;
mod health;
pub(super) mod target;

pub fn resolve_player_support(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    packet: &[u8],
    id: i16,
    count: i16,
    random: &mut RandomState,
) -> Result<SupportSpellResult, RuleError> {
    resolve_support_with_context(
        warriors,
        enemies,
        mask,
        id,
        &DirectSpell {
            packet,
            level: id % 1000 / 100 - 1,
            count,
            armor_filter: false,
            weapon_sound: 0,
        },
        random,
    )
}

pub fn resolve_support_with_context(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    id: i16,
    context: &DirectSpell<'_>,
    random: &mut RandomState,
) -> Result<SupportSpellResult, RuleError> {
    resolve_support_for_caster(warriors, enemies, mask, id, context, (None, None), random)
}

#[allow(clippy::too_many_arguments)]
pub fn resolve_support_for_caster(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    _id: i16,
    context: &DirectSpell<'_>,
    caller: (Option<i16>, Option<RestrictionMemoryView<'_>>),
    random: &mut RandomState,
) -> Result<SupportSpellResult, RuleError> {
    resolve_support_inner(warriors, enemies, mask, context, caller, None, random)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_support_inner(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    mask: &[i16],
    context: &DirectSpell<'_>,
    caller: (Option<i16>, Option<RestrictionMemoryView<'_>>),
    mut usage: Option<&mut FieldUsage>,
    random: &mut RandomState,
) -> Result<SupportSpellResult, RuleError> {
    if !(1..=6).contains(&warriors.len())
        || enemies.len() > 100
        || mask.len() < 10 + enemies.len()
        || context.packet.len() != 58
        || !(0..7).contains(&context.level)
        || warriors.iter().any(|w| w.bytes.len() != 3138)
        || enemies.iter().any(|m| m.bytes.len() != 288)
    {
        return Err(RuleError::Invalid("support spell inputs".into()));
    }
    // resolvespell's movement (151418) and summon (1516ea) arms and spelllist's
    // category-5 drain (15ba5a) replace this body loop. Every game caller routes
    // them first; only the record-only public adapters, which no command reaches,
    // can pass one here.
    if !super::is_support(context.packet)? {
        return Err(
            "movement, summon or category-5 drain packet in a record-only body resolver".into(),
        );
    }
    if mask[warriors.len()..10].iter().any(|flag| *flag > 0)
        || mask[10 + enemies.len()..].iter().any(|flag| *flag > 0)
    {
        return Err(RuleError::Unsupported {
            action: "combat.spell.support",
            dependency: "condition target is an inactive warrior or unloaded NPC record",
        });
    }
    let attack = AttackPacket(context.packet);
    let effect = attack.effect()?.wrapping_abs();
    let mut next_random = random.clone();
    let (minimum, maximum) = attack.duration_bounds();
    let mut duration = next_random.range(minimum, maximum);
    let (minimum, maximum) = attack.additional_duration_bounds();
    if minimum != 0 {
        for _ in 0..context.count {
            duration = duration.wrapping_add(next_random.range(minimum, maximum));
        }
    }
    let mut result = SupportSpellResult {
        total_delta: 0,
        warriors: warriors.to_vec(),
        enemies: enemies.to_vec(),
        sounds: if matches!(attack.targeting_mode(), 9 | 10) {
            vec![6001]
        } else {
            vec![]
        },
        targets: vec![],
        healed: false,
        recovered: 0,
        race_excluded: vec![],
        fatigue_reset: false,
        area_shape: None,
        forced_kills: vec![],
        alias_writes: vec![],
    };
    if effect == 66 {
        train_party(&mut result, &attack, mask, &mut next_random)?;
    }
    let mut amount = 0;
    for body in (0..warriors.len()).chain(10..10 + enemies.len()) {
        if mask[body] <= 0 {
            continue;
        }
        let record = if body < 10 {
            &result.warriors[body].bytes
        } else {
            &result.enemies[body - 10].bytes
        };
        let excluded = if body < 10 {
            race_is_excluded(context.packet, record, caller.1)?
        } else {
            attack.race_restriction()? != 0
                && monster_race_is_excluded(context.packet, body, caller.1)?
        };
        if excluded {
            result.race_excluded.push(body);
            continue;
        }
        if status_resisted(record, body >= 10, context, caller.1, &mut next_random)? {
            continue;
        }
        // Resistance precedes this shared byte, including already-used fields.
        if let Some(usage) = usage.as_deref_mut() {
            if usage.used[usage.slot + 1] != 0 {
                continue;
            }
            usage.used[usage.slot + 1] = 1;
        }
        target::resolve(
            &mut result,
            body,
            context,
            effect,
            duration,
            &mut amount,
            caller,
            &mut next_random,
        )?;
    }
    *random = next_random;
    Ok(result)
}

/// resolvespell15133c trains each selected party member before the body loop,
/// whether or not combat mode is set.
pub(super) fn train_party(
    result: &mut SupportSpellResult,
    attack: &AttackPacket,
    mask: &[i16],
    next_random: &mut RandomState,
) -> Result<(), RuleError> {
    let mut selector = attack.area_shape() as i8;
    if selector == 0 {
        selector = next_random.bound(5) as i8;
        result.area_shape = Some(selector as u8);
    }
    for (warrior, selected) in result.warriors.iter_mut().zip(mask) {
        if *selected == 0 {
            continue;
        }
        WarriorRecordMut::from_bytes(&mut warrior.bytes).train_attribute(selector);
        let view = WarriorRecord::from_bytes(&warrior.bytes);
        if selector == 10 {
            let value = view.maximum_stamina()?.wrapping_add(next_random.bound(8));
            WarriorRecordMut::from_bytes(&mut warrior.bytes).set_maximum_stamina(value);
        } else if selector == 11 && view.maximum_psi()? > 0 {
            let value = view.maximum_psi()?.wrapping_add(next_random.bound(20));
            WarriorRecordMut::from_bytes(&mut warrior.bytes).set_maximum_psi(value);
        }
    }
    Ok(())
}
