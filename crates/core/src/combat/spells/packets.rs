//! Spell descriptors and the attack packets they load.
use super::{RandomState, RuleError};
use centurions_content::combat::MonsterRecord;

/// The lowest spell descriptor: school 0, level 0, index 0. Lower item and
/// weapon descriptors are not spells.
pub(crate) const FIRST_SPELL_DESCRIPTOR: i16 = 1101;

/// Decodes a spell descriptor into its zero-based school, level and index:
/// thousands, hundreds and units each hold the field plus one.
pub(crate) fn spell_identity(id: i16) -> [i16; 3] {
    [id / 1000 - 1, spell_level(id), id % 100 - 1]
}

/// The zero-based level held in a spell descriptor's hundreds digit.
pub(crate) fn spell_level(id: i16) -> i16 {
    id % 1000 / 100 - 1
}

/// Encodes a school, level and index identity as a spell descriptor.
pub(crate) fn spell_descriptor(identity: [i16; 3]) -> i16 {
    (identity[0] + 1) * 1000 + (identity[1] + 1) * 100 + identity[2] + 1
}

/// `spell_descriptor` with the 16-bit wrapping arithmetic of callers that use it.
pub(crate) fn wrapping_spell_descriptor(identity: [i16; 3]) -> i16 {
    identity[0]
        .wrapping_mul(1000)
        .wrapping_add(identity[1].wrapping_mul(100))
        .wrapping_add(identity[2])
        .wrapping_add(FIRST_SPELL_DESCRIPTOR)
}

/// The attack packet for a spell number: found by its school, level and index in the attack table.
pub fn load_packet(attacks: &[u8], id: i16) -> Result<Vec<u8>, RuleError> {
    if id < FIRST_SPELL_DESCRIPTOR {
        return Err(RuleError::Invalid("spell descriptor".into()));
    }
    let [school, level, index] = spell_identity(id);
    if school < 0 || level < 0 || index < 0 {
        return Err(RuleError::Invalid("spell descriptor fields".into()));
    }
    Ok(centurions_content::span(
        attacks,
        school as usize * 2842 + level as usize * 406 + index as usize * 58,
        58,
    )?
    .to_vec())
}

/// A monster chooses a spell to cast: up to 30 times it picks a random one of its ten spell slots
/// and takes the first pick that holds a spell. None when no pick does.
pub fn select_packet(
    monster: &MonsterRecord,
    attacks: &[u8],
    random: &mut RandomState,
) -> Result<Option<(i16, Vec<u8>)>, RuleError> {
    let mut next_random = random.clone();
    for _ in 0..30 {
        let index = next_random.range(0, 9) as usize;
        let id = monster.record().spell_slot(index)?;
        if id > 1100 {
            let packet = load_packet(attacks, id)?;
            *random = next_random;
            return Ok(Some((id, packet)));
        }
    }
    *random = next_random;
    Ok(None)
}
