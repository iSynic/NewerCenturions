use crate::RuleError;
use centurions_content::word;

pub(crate) fn validate(buffs: &[u8]) -> Result<(), RuleError> {
    if buffs.len() != 20 {
        return Err(RuleError::Invalid("party buff size".into()));
    }
    Ok(())
}

pub(crate) fn mind_shield(buffs: &[u8]) -> Result<bool, RuleError> {
    Ok(word(buffs, 14)? != 0)
}

pub fn extend_inertial(buffs: &mut [u8], duration: i16) -> Result<(), RuleError> {
    extend(buffs, 2, duration)
}

/// resolvespell15130e writes beyond the directory NUL for short scenario names.
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

/// The tail WORD44263e, or None when it overlaps the scenario-directory text.
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
            .native_import
            .as_ref()
            .map(|records| records.scenario_duration_tail())
            .transpose()?
            .unwrap_or(0),
    }))
}

/// resolvespell15133c retains the greater signed duration in the selected party slot.
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

/// Effect50 compares the scaled WORD before subtracting one at151236.
pub(crate) fn extend_from_count(buffs: &mut [u8], count: i16) -> Result<(), RuleError> {
    validate(buffs)?;
    let scaled = count.wrapping_mul(30);
    if word(buffs, 0)? < scaled {
        buffs[..2].copy_from_slice(&scaled.wrapping_sub(1).to_be_bytes());
    }
    Ok(())
}

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

pub fn attack2_damage(buffs: &[u8], target: i16, amount: i16) -> Result<Option<i16>, RuleError> {
    validate(buffs)?;
    Ok(
        (word(buffs, 4)? != 0 && (0..9).contains(&target) && amount > 1)
            .then(|| amount.wrapping_sub(5).max(1)),
    )
}

/// resolvespell15133c's max-write of WORD 442562+2*effect beyond the party durations,
/// through the represented Information owners; false when that WORD is unowned.
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

/// `extend_indexed` over an NPC work copy's owners; the live combat holds difficulty.
pub(crate) fn extend_indexed_owners(
    party_buffs: &mut [u8],
    difficulty: &mut i16,
    flags: &mut [u8],
    effect: i16,
    duration: i16,
) -> bool {
    use crate::state::information::{read_word, write_word};
    let address = indexed_address(effect);
    read_word(party_buffs, *difficulty, flags, address).is_some_and(|current| {
        write_word(
            party_buffs,
            difficulty,
            flags,
            address,
            current.max(duration),
        )
    })
}

fn indexed_address(effect: i16) -> i64 {
    0x442562 + 2 * i64::from(effect)
}
