//! Packets for combat/spells.
use super::{RandomState, RuleError};
use centurions_content::combat::MonsterRecord;
use centurions_content::word;

pub fn load_packet(attacks: &[u8], id: i16) -> Result<Vec<u8>, RuleError> {
    if id < 1101 {
        return Err(RuleError::Invalid("spell descriptor".into()));
    }
    let school = id / 1000 - 1;
    let level = id % 1000 / 100 - 1;
    let index = id % 100 - 1;
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

pub fn select_packet(
    monster: &MonsterRecord,
    attacks: &[u8],
    random: &mut RandomState,
) -> Result<Option<(i16, Vec<u8>)>, RuleError> {
    let mut next_random = random.clone();
    for _ in 0..30 {
        let index = next_random.range(0, 9) as usize;
        let id = word(&monster.bytes, 0x12 + index * 2)?;
        if id > 1100 {
            let packet = load_packet(attacks, id)?;
            *random = next_random;
            return Ok(Some((id, packet)));
        }
    }
    *random = next_random;
    Ok(None)
}
