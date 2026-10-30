//! Spell fields: their duration ticks, usage reset and collisions.
use super::{Combat, FieldUsage, RuleError, SpellTarget, load_packet};
use centurions_content::Warrior;
use centurions_content::records::{SpellField, SpellFieldMut, SpellInfoRecordMut};

/// Ages the spell fields: a field whose turn in the initiative order has come loses one round of
/// duration.
pub fn tick_fields(fields: &[u8], cursor: i16) -> Result<Vec<u8>, RuleError> {
    if fields.len() != 1200 {
        return Err(RuleError::Invalid("spell field table size".into()));
    }
    let mut result = fields.to_vec();
    for field in result.as_chunks_mut::<20>().0 {
        if SpellField(field).duration() != 0 && SpellField(field).initiative() == cursor {
            let duration = SpellField(field).duration().wrapping_sub(1);
            SpellFieldMut(field).set_duration(duration);
        }
    }
    Ok(result)
}

/// Clears the usage marks of the 60 spell fields for a new turn.
pub fn reset_used(used: &[u8]) -> Result<Vec<u8>, RuleError> {
    if used.len() != 128 {
        return Err(RuleError::Invalid("spell field usage size".into()));
    }
    let mut result = used.to_vec();
    result[..60].fill(0);
    Ok(result)
}

/// A spell field a body stands in: the spell, its power and level, its rotation, and the cell it is
/// centered on.
pub struct FieldCollision {
    pub rotation: u8,
    pub packet: Vec<u8>,
    pub id: i16,
    pub count: i16,
    pub level: i16,
    pub target: SpellTarget,
}

/// The spell field a body is struck by, if any: the field must still last and not yet have struck
/// this turn, and the body (counting the cells a large monster covers) must stand inside it. The
/// field's spell is cast on the body.
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
            .record()
            .in_battle()
            == 0
    {
        return Ok(None);
    }
    let field = SpellField(&fields[usage.slot * 20..usage.slot * 20 + 20]);
    if field.duration() == 0 || usage.used[usage.slot] != 0 {
        return Ok(None);
    }
    let point = combat
        .get_body_position(body)
        .ok_or_else(|| RuleError::Invalid("spell collision body position".into()))?;
    let point = crate::combat::records::wrapping_grid_point(point, combat.field.map_shift);
    let size = if body < 10 {
        0
    } else {
        combat
            .roster
            .records
            .get(body - 10)
            .ok_or_else(|| RuleError::Invalid("spell collision monster".into()))?
            .record()
            .size()?
            .wrapping_abs()
    };
    if !(0..=3).contains(&size) {
        return Err(RuleError::Unsupported {
            action: "combat.spell.collision",
            dependency: "other monster footprint",
        });
    }
    let inside = |p: (i16, i16)| -> Result<bool, RuleError> { Ok(field.contains(p)?) };
    if !inside(point)?
        && !((size == 1 || size == 3) && inside((point.0, point.1.wrapping_sub(1)))?)
        && !((size == 2 || size == 3) && inside((point.0.wrapping_sub(1), point.1))?)
        && !(size == 3 && inside((point.0.wrapping_sub(1), point.1.wrapping_sub(1)))?)
    {
        return Ok(None);
    }
    let id = field.spell()?;
    let mut packet = load_packet(attacks, id)?;
    SpellInfoRecordMut(&mut packet).prepare_field_collision();
    Ok(Some(FieldCollision {
        rotation: field.rotation(),
        packet,
        id,
        count: field.power(),
        level: field.level(),
        target: SpellTarget::cell(field.center().0, field.center().1),
    }))
}
