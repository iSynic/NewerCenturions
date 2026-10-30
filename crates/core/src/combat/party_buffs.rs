use crate::RuleError;
use centurions_content::word;

/// Checks that the party's effect durations have their fixed size.
pub(crate) fn validate(buffs: &[u8]) -> Result<(), RuleError> {
    if buffs.len() != 20 {
        return Err(RuleError::Invalid("party buff size".into()));
    }
    Ok(())
}

/// Whether the party's mind shield is up.
pub(crate) fn mind_shield(buffs: &[u8]) -> Result<bool, RuleError> {
    Ok(word(buffs, 14)? != 0)
}

/// Extends the party's inertial protection to at least the duration.
pub fn extend_inertial(buffs: &mut [u8], duration: i16) -> Result<(), RuleError> {
    extend(buffs, 2, duration)
}

/// Some party durations run past the end of a short scenario name (see
/// docs/source-notes/combat.md).
///
/// Original: resolvespell
pub(crate) fn extend_scenario_tail(
    state: &mut crate::Session,
    duration: i16,
) -> Result<(), RuleError> {
    let previous = scenario_tail(state)?.ok_or(RuleError::Unsupported {
        action: "psi.party",
        dependency: "indexed duration overlaps scenario-directory text",
    })?;
    state.scenario_duration_tail = Some(previous.max(duration));
    Ok(())
}

/// The last party duration, or None when the scenario name still covers it.
pub(crate) fn scenario_tail(state: &crate::Session) -> Result<Option<i16>, RuleError> {
    if state
        .mission
        .as_ref()
        .is_none_or(|name| name.len() + b":Scenarios:".len() >= 50)
    {
        return Ok(None);
    }
    Ok(Some(match state.scenario_duration_tail {
        Some(value) => value,
        None => state
            .imported
            .as_ref()
            .map(|records| records.scenario_duration_tail())
            .transpose()?
            .unwrap_or(0),
    }))
}

/// A party spell keeps the longer of its current and new duration.
///
/// Original: resolvespell
pub fn extend(buffs: &mut [u8], effect: i16, duration: i16) -> Result<(), RuleError> {
    validate(buffs)?;
    let offset = usize::try_from(effect)
        .ok()
        .filter(|slot| *slot < 10)
        .ok_or_else(|| RuleError::Invalid("party effect outside duration table".into()))?
        * 2;
    let duration = word(buffs, offset)?.max(duration);
    buffs[offset..offset + 2].copy_from_slice(&duration.to_be_bytes());
    Ok(())
}

/// Effect 50 compares the scaled count before taking one off.
///
/// Original: resolvespell
pub(crate) fn extend_from_count(buffs: &mut [u8], count: i16) -> Result<(), RuleError> {
    validate(buffs)?;
    let scaled = count.wrapping_mul(30);
    if word(buffs, 0)? < scaled {
        buffs[..2].copy_from_slice(&scaled.wrapping_sub(1).to_be_bytes());
    }
    Ok(())
}

/// Counts every running party duration down by one each round; the first counts down a second time.
pub fn reduce(buffs: &mut [u8]) -> Result<(), RuleError> {
    validate(buffs)?;
    for offset in (0..20).step_by(2) {
        let duration = word(buffs, offset)?;
        if duration > 0 {
            buffs[offset..offset + 2].copy_from_slice(&(duration - 1).to_be_bytes());
        }
    }
    let first = word(buffs, 0)?;
    if first != 0 {
        buffs[..2].copy_from_slice(&first.wrapping_sub(1).to_be_bytes());
    }
    Ok(())
}

/// While the party's inertial protection lasts, a hit of more than one point on a warrior is
/// reduced by five, to at least one.
pub fn attack2_damage(buffs: &[u8], target: i16, amount: i16) -> Result<Option<i16>, RuleError> {
    validate(buffs)?;
    Ok(
        (word(buffs, 4)? != 0 && (0..9).contains(&target) && amount > 1)
            .then(|| amount.wrapping_sub(5).max(1)),
    )
}

/// A party effect number past the duration table extends whatever value lies
/// there; false where the session does not hold it.
///
/// Original: resolvespell
pub(crate) fn extend_indexed(
    state: &mut crate::Session,
    effect: i16,
    duration: i16,
) -> Result<bool, RuleError> {
    let address = indexed_address(effect);
    let Some(current) = state.information_word(address) else {
        return Ok(false);
    };
    Ok(state.set_information_word(address, current.max(duration)))
}

/// `extend_indexed` for a monster's copy of the battle; the live battle holds the
/// difficulty.
pub(crate) fn extend_indexed_owners(
    party_condition: &mut [u8],
    difficulty: &mut i16,
    flags: &mut [u8],
    effect: i16,
    duration: i16,
) -> bool {
    use crate::state::information::{read_word, write_word};
    let address = indexed_address(effect);
    read_word(party_condition, *difficulty, flags, address).is_some_and(|current| {
        write_word(
            party_condition,
            difficulty,
            flags,
            address,
            current.max(duration),
        )
    })
}

/// Where a party effect number past the duration table lands.
fn indexed_address(effect: i16) -> i64 {
    0x442562 + 2 * i64::from(effect)
}
