use crate::RuleError;
use centurions_content::word;

pub(crate) fn validate(buffs: &[u8]) -> Result<(), RuleError> {
    if buffs.len() != 20 {
        return Err(RuleError::Invalid("party buff size".into()));
    }
    Ok(())
}

pub fn extend_inertial(buffs: &mut [u8], duration: i16) -> Result<(), RuleError> {
    validate(buffs)?;
    let duration = word(buffs, 4)?.max(duration);
    buffs[4..6].copy_from_slice(&duration.to_be_bytes());
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
