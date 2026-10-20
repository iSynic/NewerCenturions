//! Fields for combat/spells.
use super::{Combat, FieldUsage, RuleError, SpellTarget, load_packet, position};
use centurions_content::{Warrior, word};

pub fn tick_fields(fields: &[u8], cursor: i16) -> Result<Vec<u8>, RuleError> {
    if fields.len() != 1200 {
        return Err(RuleError::Invalid("spell field table size".into()));
    }
    let mut result = fields.to_vec();
    for field in result.as_chunks_mut::<20>().0 {
        if field[15] != 0 && i16::from(field[18] as i8) == cursor {
            field[15] = field[15].wrapping_sub(1);
        }
    }
    Ok(result)
}

pub fn reset_used(used: &[u8]) -> Result<Vec<u8>, RuleError> {
    if used.len() != 128 {
        return Err(RuleError::Invalid("spell field usage size".into()));
    }
    let mut result = used.to_vec();
    result[..60].fill(0);
    Ok(result)
}

pub struct FieldCollision {
    pub rotation: u8,
    pub packet: Vec<u8>,
    pub id: i16,
    pub count: i16,
    pub level: i16,
    pub target: SpellTarget,
}

pub fn collision_field(
    combat: &Combat,
    warriors: &[Warrior],
    fields: &[u8],
    usage: &FieldUsage,
    body: usize,
    attacks: &[u8],
) -> Result<Option<FieldCollision>, RuleError> {
    if fields.len() != 1200 || usage.used.len() != 128 || usage.slot >= 60 || body >= 110 {
        return Err(RuleError::Invalid("spell collision inputs".into()));
    }
    if body < 10
        && warriors
            .get(body)
            .ok_or_else(|| RuleError::Invalid("spell collision warrior".into()))?
            .bytes[0x9df]
            == 0
    {
        return Ok(None);
    }
    let field = &fields[usage.slot * 20..usage.slot * 20 + 20];
    if field[15] == 0 || usage.used[usage.slot] != 0 {
        return Ok(None);
    }
    let point = position(combat, body)
        .ok_or_else(|| RuleError::Invalid("spell collision body position".into()))?;
    let point = (
        point.0.wrapping_add(combat.map.shift.0).wrapping_add(5),
        point.1.wrapping_add(combat.map.shift.1).wrapping_add(5),
    );
    let size = if body < 10 {
        0
    } else {
        word(
            &combat
                .enemies
                .records
                .get(body - 10)
                .ok_or_else(|| RuleError::Invalid("spell collision monster".into()))?
                .bytes,
            0x10,
        )?
        .wrapping_abs()
    };
    if !(0..=3).contains(&size) {
        return Err(RuleError::Unsupported {
            action: "combat.spell.collision",
            dependency: "other monster footprint",
        });
    }
    let inside = |p: (i16, i16)| -> Result<bool, RuleError> {
        Ok(p.1 >= word(field, 0)?
            && p.1 < word(field, 4)?
            && p.0 >= word(field, 2)?
            && p.0 < word(field, 6)?)
    };
    if !inside(point)?
        && !((size == 1 || size == 3) && inside((point.0, point.1.wrapping_sub(1)))?)
        && !((size == 2 || size == 3) && inside((point.0.wrapping_sub(1), point.1))?)
        && !(size == 3 && inside((point.0.wrapping_sub(1), point.1.wrapping_sub(1)))?)
    {
        return Ok(None);
    }
    let id = word(field, 8)?;
    let mut packet = load_packet(attacks, id)?;
    if packet[19] == 9 {
        packet[19] = 1;
        packet[20] = 1;
    }
    packet[4] = 0;
    Ok(Some(FieldCollision {
        rotation: field[17],
        packet,
        id,
        count: i16::from(field[16] as i8),
        level: i16::from(field[19] as i8),
        target: SpellTarget {
            first: i16::from(field[12] as i8).wrapping_add(3),
            second: i16::from(field[13] as i8).wrapping_add(3),
            kind: 3,
        },
    }))
}
