//! Target masks: the bodies and cells an area spell covers.
use super::{Combat, RandomState, RuleError, SpellContext, SpellMask, SpellTarget};
use crate::combat::map_alias::MapAliases;
use crate::combat::spells::spell_level;
use centurions_content::Warrior;
use centurions_content::combat::MonsterRecord;
use centurions_content::records::{Condition, SpellField, SpellFieldMut, SpellInfoRecord};

/// The bodies a faction-wide spell reaches, with the casting monster as it stands afterwards.
pub struct GlobalSpellTargets {
    pub caster: MonsterRecord,
    pub mask: [i16; 110],
}

/// The bodies a faction-wide spell cast by a monster reaches: the caster's own side (mode 9), the
/// other side (mode 10) or everyone (mode 12). Unless the spell reaches all, the caster loses its
/// reaction.
pub fn global_targets(
    combat: &Combat,
    warriors: &[Warrior],
    caster_index: usize,
    packet: &[u8],
) -> Result<GlobalSpellTargets, RuleError> {
    if packet.len() != 58
        || !SpellInfoRecord(packet).is_global()
        || warriors.len() > 6
        || combat.roster.records.len() > 100
    {
        return Err(RuleError::Invalid("global spell targets".into()));
    }
    let caster = combat
        .roster
        .records
        .get(caster_index)
        .ok_or_else(|| RuleError::Invalid("global spell caster".into()))?;
    let side = caster.record().faction()?;
    let mut result = GlobalSpellTargets {
        caster: caster.clone(),
        mask: global_target_mask(
            warriors,
            &combat.roster.records,
            side,
            SpellInfoRecord(packet).target_type(),
        )?,
    };
    if !SpellInfoRecord(packet).is_global_all() {
        result.caster.record_mut().set_reaction_ready(false);
    }
    Ok(result)
}

/// The active monster prefix and the selected caster's fixed slot are independent.
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
        if warrior.record().in_battle() != 0
            && (mode == 12 || (warrior.record().faction()? == side) == (mode == 9))
        {
            mask[member] = 1;
        }
    }
    for (index, monster) in enemies.iter().enumerate() {
        if monster.record().stamina()? > 0
            && (mode == 12 || (monster.record().faction()? == side) == (mode == 9))
        {
            mask[index + 10] = 1;
        }
    }
    Ok(mask)
}

/// The bodies an area spell reaches, in the pattern around its target, with the field the spell
/// lays.
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

/// The rotation selects one of four adjacent authored patterns.
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

/// As `target_mask_rotated`, with the field laid at a separate origin and the memory beside the map
/// readable for cells off its edge.
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

/// The bodies an area spell reaches, with the field laid at a separate origin from the cells hit.
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

/// The bodies an area spell reaches, with the memory beside the map readable for cells off its
/// edge.
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

/// The bodies the area pattern covers around the target. A summon cannot be masked. Unless the
/// spell is a field collision, it lays its field first. Each covered body is reached, except a
/// monster immune to psi (resistance above 100); a body that reflects scans turns the spell back on
/// the caster 34 percent of the time, unless the spell is defense category 9 or a collision. In a
/// collision only the caster is reached.
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
    let packet = context.packet;
    if packet.len() != crate::combat::ATTACK_PACKET_BYTES
        || fields.len() != crate::combat::SPELL_FIELD_BYTES
        || !(0..110).contains(&context.caster)
    {
        return Err(RuleError::Invalid("spell mask inputs".into()));
    }
    if SpellInfoRecord(packet).effect()? == super::effects::SUMMON {
        return Err(RuleError::Unsupported {
            action: "combat.spell.mask",
            dependency: "rotated shapes or summon targeting",
        });
    }
    if rotation > 3 {
        return Err(RuleError::Invalid("spell rotation".into()));
    }
    let shape = shape(packet, context.count, rotation)?;
    // A pattern index below the table reads outside memory and is treated as an
    // empty pattern (evidence/hive-memory-verification.md).
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
    let center = if SpellInfoRecord(packet).target_type() >= 2 && target.kind == 3 {
        (target.first, target.second)
    } else {
        let body = usize::try_from(target.first)
            .map_err(|_| RuleError::Invalid("negative spell target".into()))?;
        let point = combat
            .get_body_position(body)
            .ok_or_else(|| RuleError::Invalid("spell mask target body".into()))?;
        (
            point.0 + combat.field.map_shift.0 + 5,
            point.1 + combat.field.map_shift.1 + 5,
        )
    };
    for y in 0..7 {
        for x in 0..7 {
            if pattern[y * 7 + x] == 0 {
                continue;
            }
            let ax = center.0.wrapping_sub(3).wrapping_add(x as i16);
            let ay = center.1.wrapping_sub(3).wrapping_add(y as i16);
            // The column is unsigned and the row signed, and neither is clipped, so
            // a negative row can still address the memory beside the map.
            let index = i32::from(ax as u16) * 90 + i32::from(ay);
            let body = aliases
                .read(&combat.field, index)?
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
            let reflected = if SpellInfoRecord(packet).defense_category() != 9 {
                let reflective = if body < 9 {
                    warriors
                        .get(body)
                        .ok_or_else(|| RuleError::Invalid("spell reflection warrior".into()))?
                        .record()
                        .condition(Condition::ReflectingScans)?
                        != 0
                } else {
                    combat
                        .roster
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
                    .roster
                    .records
                    .get(body - 10)
                    .ok_or_else(|| RuleError::Invalid("spell mask monster".into()))?
                    .record()
                    .psi_resistance()?
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

/// The area shape: the area byte for mode 3, the count for mode 4 and 1 otherwise,
/// plus the rotation for a byte-7 packet.
///
/// Original: spelltargets
fn shape(packet: &[u8], count: i16, rotation: u8) -> Result<usize, RuleError> {
    let mut shape = match SpellInfoRecord(packet).target_type() {
        3 => usize::from(SpellInfoRecord(packet).area_size()),
        4 => count as usize,
        _ => 1,
    };
    if SpellInfoRecord(packet).can_rotate() {
        shape += usize::from(rotation);
    }
    Ok(shape)
}

/// The first expired record becomes a field at the first target's centre with the
/// drawn duration; the rotation is stored only for a byte-7 packet.
///
/// Original: spelltargets
fn create_field(
    fields: &mut [u8],
    context: &SpellContext<'_>,
    (shape, rotation): (usize, u8),
    origin: &SpellTarget,
    random: &mut RandomState,
) -> Result<(), RuleError> {
    let packet = context.packet;
    let Some(slot) = fields
        .as_chunks::<20>()
        .0
        .iter()
        .position(|f| SpellField(f).duration() == 0)
    else {
        return Ok(());
    };
    if SpellInfoRecord(packet).field_kind() == 0 {
        return Ok(());
    }
    let mut field = SpellFieldMut(&mut fields[slot * 20..slot * 20 + 20]);
    field.set_initiative(context.initiative);
    field.set_power(context.count);
    field.set_level(spell_level(context.id));
    field.set_rotation(if SpellInfoRecord(packet).can_rotate() {
        rotation
    } else {
        0
    });
    field.set_shape(shape);
    field.set_center((origin.first, origin.second));
    field.set_spell(context.id);
    let mut duration = random.range(
        SpellInfoRecord(packet).duration_range().0,
        SpellInfoRecord(packet).duration_range().1,
    ) as u8;
    for _ in 0..context.count {
        duration = duration.wrapping_add(random.range(
            SpellInfoRecord(packet).power_duration_range().0,
            SpellInfoRecord(packet).power_duration_range().1,
        ) as u8);
    }
    field.set_duration(duration);
    field.set_kind(SpellInfoRecord(packet).field_kind());
    Ok(())
}

/// A summoning's field: its shape is offset past the pattern table, so the field
/// keeps only the shape's low byte and an empty pattern, which the summon arm ignores.
///
/// Original: spelltargets
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
    let mut shape = usize::from(SpellInfoRecord(context.packet).area_size()) + 0x2007;
    if SpellInfoRecord(context.packet).can_rotate() {
        shape += usize::from(rotation);
    }
    let mut fields = fields.to_vec();
    create_field(&mut fields, context, (shape, rotation), origin, random)?;
    Ok(fields)
}
