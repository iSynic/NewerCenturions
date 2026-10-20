//! Masks for combat/spells.
use super::{
    Combat, RandomState, RuleError, SpellContext, SpellMask, SpellTarget, position, put_word,
};
use centurions_content::combat::MonsterRecord;
use centurions_content::{Warrior, word};

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
        || !matches!(packet[1], 9 | 10 | 12)
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
    let side = word(&caster.bytes, 0xe)?;
    let mut result = GlobalSpellTargets {
        caster: caster.clone(),
        mask: [0; 110],
    };
    for (member, warrior) in warriors.iter().enumerate() {
        if warrior.record().active_flag() != 0
            && (packet[1] == 12 || (warrior.record().faction()? == side) == (packet[1] == 9))
        {
            result.mask[member] = 1;
        }
    }
    for (index, enemy) in combat.enemies.records.iter().enumerate() {
        if word(&enemy.bytes, 0x110)? > 0
            && (packet[1] == 12 || (word(&enemy.bytes, 0xe)? == side) == (packet[1] == 9))
        {
            result.mask[index + 10] = 1;
        }
    }
    if packet[1] != 12 {
        result.caster.bytes[0x104] = 0;
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
    let rotated = packet[7] != 0;
    if rotated {
        packet[31] = packet[31]
            .checked_add(rotation)
            .ok_or("spell pattern overflow")?;
    }
    let adjusted = SpellContext {
        packet: &packet,
        ..*context
    };
    let mut result =
        target_mask_with_origin(combat, warriors, areas, targets, &adjusted, fields, random)?;
    if rotated
        && packet[4] != 0
        && !context.collision
        && let Some(slot) = fields.as_chunks::<20>().0.iter().position(|f| f[15] == 0)
    {
        result.fields[slot * 20 + 17] = rotation;
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
    if word(p, 40)? == 58 {
        return Err(RuleError::Unsupported {
            action: "combat.spell.mask",
            dependency: "rotated shapes or summon targeting",
        });
    }
    let shape = match p[1] {
        3 => usize::from(p[31]),
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
    if p[4] != 0
        && !context.collision
        && let Some(slot) = fields.as_chunks::<20>().0.iter().position(|f| f[15] == 0)
    {
        let f = &mut result.fields[slot * 20..slot * 20 + 20];
        f[18] = context.initiative as u8;
        f[16] = context.count as u8;
        f[19] = (context.id % 1000 / 100 - 1) as u8;
        f[17] = 0;
        f[14] = shape as u8;
        f[12] = origin.first.wrapping_sub(3) as u8;
        f[13] = origin.second.wrapping_sub(3) as u8;
        let left = i16::from(f[12] as i8);
        let top = i16::from(f[13] as i8);
        put_word(f, 0, top);
        put_word(f, 2, left);
        put_word(f, 4, top.wrapping_add(7));
        put_word(f, 6, left.wrapping_add(7));
        put_word(f, 8, context.id);
        let mut duration = next_random.range(i16::from(p[13] as i8), i16::from(p[14] as i8)) as u8;
        for _ in 0..context.count {
            duration = duration.wrapping_add(
                next_random.range(i16::from(p[15] as i8), i16::from(p[16] as i8)) as u8,
            );
        }
        f[15] = duration;
        put_word(f, 10, i16::from(p[4] as i8));
    }
    let center = if p[1] >= 2 && target.kind == 3 {
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
            let reflected = if p[19] != 9 {
                let reflective = if body < 9 {
                    word(
                        &warriors
                            .get(body)
                            .ok_or_else(|| RuleError::Invalid("spell reflection warrior".into()))?
                            .bytes,
                        0x9a,
                    )? != 0
                } else {
                    combat
                        .enemies
                        .records
                        .get(body.wrapping_sub(10))
                        .ok_or_else(|| RuleError::Invalid("spell reflection monster".into()))?
                        .bytes[0xd0]
                        != 0
                };
                reflective && next_random.bound(100) < 34 && !context.collision
            } else {
                false
            };
            if reflected {
                result.mask[context.caster as usize] = 1;
            } else if body < 10
                || word(
                    &combat
                        .enemies
                        .records
                        .get(body - 10)
                        .ok_or_else(|| RuleError::Invalid("spell mask monster".into()))?
                        .bytes,
                    0xa,
                )? < 101
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
