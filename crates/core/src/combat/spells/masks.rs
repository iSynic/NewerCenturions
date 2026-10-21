//! Masks for combat/spells.
use super::{Combat, RandomState, RuleError, SpellContext, SpellMask, SpellTarget, position};
use centurions_content::Warrior;
use centurions_content::combat::MonsterRecord;
use centurions_content::records::{
    AttackPacket, AttackPacketMut, Condition, SpellField, SpellFieldMut,
};

pub struct GlobalSpellTargets {
    pub caster: MonsterRecord,
    pub mask: [i16; 110],
}

pub fn global_targets(
    combat: &Combat,
    warriors: &[Warrior],
    caster_index: usize,
    packet: &[u8],
) -> Result<GlobalSpellTargets, RuleError> {
    if packet.len() != 58
        || !matches!(AttackPacket(packet).targeting_mode(), 9 | 10 | 12)
        || warriors.len() > 6
        || combat.enemies.records.len() > 100
    {
        return Err(RuleError::Invalid("global spell targets".into()));
    }
    let caster = combat
        .enemies
        .records
        .get(caster_index)
        .ok_or_else(|| RuleError::Invalid("global spell caster".into()))?;
    let side = caster.record().faction()?;
    let mut result = GlobalSpellTargets {
        caster: caster.clone(),
        mask: [0; 110],
    };
    for (member, warrior) in warriors.iter().enumerate() {
        if warrior.record().active_flag() != 0
            && (AttackPacket(packet).targeting_mode() == 12
                || (warrior.record().faction()? == side)
                    == (AttackPacket(packet).targeting_mode() == 9))
        {
            result.mask[member] = 1;
        }
    }
    for (index, enemy) in combat.enemies.records.iter().enumerate() {
        if enemy.record().stamina()? > 0
            && (AttackPacket(packet).targeting_mode() == 12
                || (enemy.record().faction()? == side)
                    == (AttackPacket(packet).targeting_mode() == 9))
        {
            result.mask[index + 10] = 1;
        }
    }
    if AttackPacket(packet).targeting_mode() != 12 {
        result.caster.record_mut().set_reaction_ready(false);
    }
    Ok(result)
}

pub fn target_mask(
    combat: &Combat,
    warriors: &[Warrior],
    areas: &[u8],
    target: &SpellTarget,
    context: &SpellContext<'_>,
    fields: &[u8],
    random: &mut RandomState,
) -> Result<SpellMask, RuleError> {
    target_mask_with_origin(
        combat,
        warriors,
        areas,
        (target, target),
        context,
        fields,
        random,
    )
}

/// Original rotation selects one of four adjacent authored patterns.
#[allow(clippy::too_many_arguments)]
pub fn target_mask_rotated(
    combat: &Combat,
    warriors: &[Warrior],
    areas: &[u8],
    target: &SpellTarget,
    context: &SpellContext<'_>,
    fields: &[u8],
    random: &mut RandomState,
    rotation: u8,
) -> Result<SpellMask, RuleError> {
    target_mask_rotated_with_origin(
        combat,
        warriors,
        areas,
        (target, target),
        context,
        fields,
        random,
        rotation,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn target_mask_rotated_with_origin(
    combat: &Combat,
    warriors: &[Warrior],
    areas: &[u8],
    targets: (&SpellTarget, &SpellTarget),
    context: &SpellContext<'_>,
    fields: &[u8],
    random: &mut RandomState,
    rotation: u8,
) -> Result<SpellMask, RuleError> {
    if rotation > 3 {
        return Err(RuleError::Invalid("spell rotation".into()));
    }
    let mut packet = context.packet.to_vec();
    if packet.len() != 58 {
        return Err(RuleError::Invalid("spell packet size".into()));
    }
    let rotated = AttackPacket(&packet).has_randomized_animation();
    if rotated {
        AttackPacketMut(&mut packet)
            .rotate_area(rotation)
            .ok_or("spell pattern overflow")?;
    }
    let adjusted = SpellContext {
        packet: &packet,
        ..*context
    };
    let mut result =
        target_mask_with_origin(combat, warriors, areas, targets, &adjusted, fields, random)?;
    if rotated
        && AttackPacket(&packet).field_kind() != 0
        && !context.collision
        && let Some(slot) = fields
            .as_chunks::<20>()
            .0
            .iter()
            .position(|f| SpellField(f).duration() == 0)
    {
        SpellFieldMut(&mut result.fields[slot * 20..slot * 20 + 20]).set_rotation(rotation);
    }
    Ok(result)
}

pub fn target_mask_with_origin(
    combat: &Combat,
    warriors: &[Warrior],
    areas: &[u8],
    targets: (&SpellTarget, &SpellTarget),
    context: &SpellContext<'_>,
    fields: &[u8],
    random: &mut RandomState,
) -> Result<SpellMask, RuleError> {
    let (target, origin) = targets;
    let p = context.packet;
    if p.len() != 58 || fields.len() != 1200 || !(0..110).contains(&context.caster) {
        return Err(RuleError::Invalid("spell mask inputs".into()));
    }
    if AttackPacket(p).effect()? == 58 {
        return Err(RuleError::Unsupported {
            action: "combat.spell.mask",
            dependency: "rotated shapes or summon targeting",
        });
    }
    let shape = match AttackPacket(p).targeting_mode() {
        3 => usize::from(AttackPacket(p).area_shape()),
        4 => context.count as usize,
        _ => 1,
    };
    if shape == 0 {
        return Err(RuleError::Unsupported {
            action: "combat.spell.mask",
            dependency: "zero-area spell pattern",
        });
    }
    let pattern = centurions_content::span(areas, (shape - 1) * 49, 49)?;
    let mut next_random = random.clone();
    let mut result = SpellMask {
        mask: [0; 110],
        fields: fields.to_vec(),
    };
    if AttackPacket(p).field_kind() != 0
        && !context.collision
        && let Some(slot) = fields
            .as_chunks::<20>()
            .0
            .iter()
            .position(|f| SpellField(f).duration() == 0)
    {
        let mut field = SpellFieldMut(&mut result.fields[slot * 20..slot * 20 + 20]);
        field.set_initiative(context.initiative);
        field.set_power(context.count);
        field.set_level(context.id % 1000 / 100 - 1);
        field.set_rotation(0);
        field.set_shape(shape);
        field.set_center((origin.first, origin.second));
        field.set_spell(context.id);
        let mut duration = next_random.range(
            AttackPacket(p).duration_bounds().0,
            AttackPacket(p).duration_bounds().1,
        ) as u8;
        for _ in 0..context.count {
            duration = duration.wrapping_add(next_random.range(
                AttackPacket(p).additional_duration_bounds().0,
                AttackPacket(p).additional_duration_bounds().1,
            ) as u8);
        }
        field.set_duration(duration);
        field.set_kind(AttackPacket(p).field_kind());
    }
    let center = if AttackPacket(p).targeting_mode() >= 2 && target.kind == 3 {
        (target.first, target.second)
    } else {
        let body = usize::try_from(target.first)
            .map_err(|_| RuleError::Invalid("negative spell target".into()))?;
        let point = position(combat, body)
            .ok_or_else(|| RuleError::Invalid("spell mask target body".into()))?;
        (
            point.0 + combat.map.shift.0 + 5,
            point.1 + combat.map.shift.1 + 5,
        )
    };
    for y in 0..7 {
        for x in 0..7 {
            if pattern[y * 7 + x] == 0 {
                continue;
            }
            let ax = center.0 - 3 + x as i16;
            let ay = center.1 - 3 + y as i16;
            if !(0..90).contains(&ax) || !(0..90).contains(&ay) {
                return Err(RuleError::Unsupported {
                    action: "combat.spell.mask",
                    dependency: "spell area outside tactical map",
                });
            }
            let body = combat.map.tiles[ax as usize * 90 + ay as usize].wrapping_abs();
            if !(0..110).contains(&body) {
                continue;
            }
            let body = body as usize;
            if context.collision && body != context.caster as usize {
                continue;
            }
            let reflected = if AttackPacket(p).defense_category() != 9 {
                let reflective = if body < 9 {
                    warriors
                        .get(body)
                        .ok_or_else(|| RuleError::Invalid("spell reflection warrior".into()))?
                        .record()
                        .condition(Condition::ReflectingScans)?
                        != 0
                } else {
                    combat
                        .enemies
                        .records
                        .get(body.wrapping_sub(10))
                        .ok_or_else(|| RuleError::Invalid("spell reflection monster".into()))?
                        .record()
                        .condition(Condition::ReflectingScans)
                        != 0
                };
                reflective && next_random.bound(100) < 34 && !context.collision
            } else {
                false
            };
            if reflected {
                result.mask[context.caster as usize] = 1;
            } else if body < 10
                || combat
                    .enemies
                    .records
                    .get(body - 10)
                    .ok_or_else(|| RuleError::Invalid("spell mask monster".into()))?
                    .record()
                    .magic_immunity()?
                    < 101
            {
                result.mask[body] = 1;
            } else {
                result.mask[body] = 0;
            }
        }
    }
    *random = next_random;
    Ok(result)
}
