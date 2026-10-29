//! Masks for combat/spells.
use super::{Combat, RandomState, RuleError, SpellContext, SpellMask, SpellTarget, position};
use crate::combat::map_alias::MapAliases;
use centurions_content::Warrior;
use centurions_content::combat::MonsterRecord;
use centurions_content::records::{AttackPacket, Condition, SpellField, SpellFieldMut};

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
        mask: global_target_mask(
            warriors,
            &combat.enemies.records,
            side,
            AttackPacket(packet).targeting_mode(),
        )?,
    };
    if AttackPacket(packet).targeting_mode() != 12 {
        result.caster.record_mut().set_reaction_ready(false);
    }
    Ok(result)
}

/// The active NPC prefix and the selected caster's fixed slot are independent.
pub(crate) fn global_target_mask(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    side: i16,
    mode: u8,
) -> Result<[i16; 110], RuleError> {
    if !matches!(mode, 9 | 10 | 12) || warriors.len() > 6 || enemies.len() > 100 {
        return Err(RuleError::Invalid("global spell targets".into()));
    }
    let mut mask = [0; 110];
    for (member, warrior) in warriors.iter().enumerate() {
        if warrior.record().active_flag() != 0
            && (mode == 12 || (warrior.record().faction()? == side) == (mode == 9))
        {
            mask[member] = 1;
        }
    }
    for (index, enemy) in enemies.iter().enumerate() {
        if enemy.record().stamina()? > 0
            && (mode == 12 || (enemy.record().faction()? == side) == (mode == 9))
        {
            mask[index + 10] = 1;
        }
    }
    Ok(mask)
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
        (rotation, MapAliases::default()),
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
    (rotation, aliases): (u8, MapAliases<'_>),
) -> Result<SpellMask, RuleError> {
    if rotation > 3 {
        return Err(RuleError::Invalid("spell rotation".into()));
    }
    target_mask_inner(
        combat, warriors, areas, targets, context, fields, random, aliases, rotation,
    )
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
    target_mask_inner(
        combat,
        warriors,
        areas,
        targets,
        context,
        fields,
        random,
        MapAliases::default(),
        0,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn target_mask_with_map_aliases(
    combat: &Combat,
    warriors: &[Warrior],
    areas: &[u8],
    targets: (&SpellTarget, &SpellTarget),
    context: &SpellContext<'_>,
    fields: &[u8],
    random: &mut RandomState,
    aliases: MapAliases<'_>,
) -> Result<SpellMask, RuleError> {
    target_mask_inner(
        combat, warriors, areas, targets, context, fields, random, aliases, 0,
    )
}

#[allow(clippy::too_many_arguments)]
fn target_mask_inner(
    combat: &Combat,
    warriors: &[Warrior],
    areas: &[u8],
    targets: (&SpellTarget, &SpellTarget),
    context: &SpellContext<'_>,
    fields: &[u8],
    random: &mut RandomState,
    aliases: MapAliases<'_>,
    rotation: u8,
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
    if rotation > 3 {
        return Err(RuleError::Invalid("spell rotation".into()));
    }
    let shape = shape(p, context.count, rotation)?;
    // Compatibility policy shared with spell selection: the original WORD
    // underflow reads outside Mac RAM, which reads as an empty pattern. See
    // evidence/hive-memory-verification.md.
    let empty = [0; 49];
    let pattern = if shape == 0 {
        &empty[..]
    } else {
        centurions_content::span(areas, (shape - 1) * 49, 49)?
    };
    let mut next_random = random.clone();
    let mut result = SpellMask {
        mask: [0; 110],
        fields: fields.to_vec(),
    };
    if !context.collision {
        create_field(
            &mut result.fields,
            context,
            (shape, rotation),
            origin,
            &mut next_random,
        )?;
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
            let ax = center.0.wrapping_sub(3).wrapping_add(x as i16);
            let ay = center.1.wrapping_sub(3).wrapping_add(y as i16);
            // spelltargets uses MULU.W for x and a signed row, without clipping
            // either coordinate. Negative rows can still address the map owner.
            let index = i32::from(ax as u16) * 90 + i32::from(ay);
            let body = aliases
                .read(&combat.map, index)?
                .ok_or(RuleError::Unsupported {
                    action: "combat.spell.mask",
                    dependency: "spell area outside the map and the represented memory beside it",
                })?
                .wrapping_abs();
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

/// spelltargets15cb0e's shape WORD: the area byte for mode 3, the count for mode 4 and 1
/// otherwise, plus the rotation for a byte-7 packet.
fn shape(packet: &[u8], count: i16, rotation: u8) -> Result<usize, RuleError> {
    let mut shape = match AttackPacket(packet).targeting_mode() {
        3 => usize::from(AttackPacket(packet).area_shape()),
        4 => count as usize,
        _ => 1,
    };
    if AttackPacket(packet).has_randomized_animation() {
        shape += usize::from(rotation);
    }
    Ok(shape)
}

/// spelltargets15cb3a..15cc8e: the first expired record becomes a field at the target-0
/// centre with the drawn duration; the rotation byte is stored only for a byte-7 packet.
fn create_field(
    fields: &mut [u8],
    context: &SpellContext<'_>,
    (shape, rotation): (usize, u8),
    origin: &SpellTarget,
    random: &mut RandomState,
) -> Result<(), RuleError> {
    let p = context.packet;
    let Some(slot) = fields
        .as_chunks::<20>()
        .0
        .iter()
        .position(|f| SpellField(f).duration() == 0)
    else {
        return Ok(());
    };
    if AttackPacket(p).field_kind() == 0 {
        return Ok(());
    }
    let mut field = SpellFieldMut(&mut fields[slot * 20..slot * 20 + 20]);
    field.set_initiative(context.initiative);
    field.set_power(context.count);
    field.set_level(context.id % 1000 / 100 - 1);
    field.set_rotation(if AttackPacket(p).has_randomized_animation() {
        rotation
    } else {
        0
    });
    field.set_shape(shape);
    field.set_center((origin.first, origin.second));
    field.set_spell(context.id);
    let mut duration = random.range(
        AttackPacket(p).duration_bounds().0,
        AttackPacket(p).duration_bounds().1,
    ) as u8;
    for _ in 0..context.count {
        duration = duration.wrapping_add(random.range(
            AttackPacket(p).additional_duration_bounds().0,
            AttackPacket(p).additional_duration_bounds().1,
        ) as u8);
    }
    field.set_duration(duration);
    field.set_kind(AttackPacket(p).field_kind());
    Ok(())
}

/// Effect58's spelltargets pass: the shape adds 0x2007, so a field stores that WORD's low
/// BYTE, and the pattern read past the supplied table is empty (the summon arm ignores it).
pub(crate) fn summon_field(
    fields: &[u8],
    context: &SpellContext<'_>,
    origin: &SpellTarget,
    random: &mut RandomState,
    rotation: u8,
) -> Result<Vec<u8>, RuleError> {
    if rotation > 3 || fields.len() != 1200 {
        return Err(RuleError::Invalid("summon field inputs".into()));
    }
    let mut shape = usize::from(AttackPacket(context.packet).area_shape()) + 0x2007;
    if AttackPacket(context.packet).has_randomized_animation() {
        shape += usize::from(rotation);
    }
    let mut fields = fields.to_vec();
    create_field(&mut fields, context, (shape, rotation), origin, random)?;
    Ok(fields)
}
