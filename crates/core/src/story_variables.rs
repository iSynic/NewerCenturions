use crate::{RuleError, story_inventory::query_party_item};
use centurions_content::{Warrior, span, word};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StoryContinuation {
    Continue,
    ReplaceScript(i16),
    Encounter(i16),
    TypedEncounter(i16),
    EndScript,
}

pub(crate) fn branch(mode: i16, target: i16) -> StoryContinuation {
    match mode {
        0 => StoryContinuation::ReplaceScript(target),
        1 => StoryContinuation::Encounter(target),
        2 => StoryContinuation::TypedEncounter(target),
        _ => StoryContinuation::Continue,
    }
}

fn index(flags: &[u8], extra: &[u8]) -> Result<usize, RuleError> {
    let index = usize::try_from(word(extra, 0)?)
        .map_err(|_| RuleError::Invalid("story variable index is negative".into()))?;
    if index >= flags.len() {
        return Err(RuleError::Invalid("story variable is unavailable".into()));
    }
    Ok(index)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VariableUpdate {
    pub flags: Vec<u8>,
    pub continuation: StoryContinuation,
}

/// newland 76 adds a signed word before clamping and storing a signed byte.
pub fn update_variable(flags: &[u8], extra: &[u8]) -> Result<VariableUpdate, RuleError> {
    let extra = span(extra, 0, 10)?;
    let index = index(flags, extra)?;
    let mut flags = flags.to_vec();
    if index < 100 {
        let value = i16::from(flags[index] as i8).wrapping_add(word(extra, 2)?);
        flags[index] = value.clamp(-127, 127) as i8 as u8;
    }
    let threshold = word(extra, 6)?;
    let continuation = if threshold != 0 && i16::from(flags[index] as i8) >= threshold {
        branch(word(extra, 4)?.wrapping_sub(1), word(extra, 8)?)
    } else {
        StoryContinuation::Continue
    };
    Ok(VariableUpdate {
        flags,
        continuation,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VariablePredicate {
    pub matched: bool,
    pub continuation: StoryContinuation,
}

pub fn variable_predicate(flags: &[u8], extra: &[u8]) -> Result<VariablePredicate, RuleError> {
    let extra = span(extra, 0, 10)?;
    let index = index(flags, extra)?;
    let matched = i16::from(flags[index] as i8) >= word(extra, 2)?;
    let target = word(extra, if matched { 8 } else { 6 })?;
    let continuation = if target == 0 {
        StoryContinuation::Continue
    } else {
        branch(word(extra, 4)?, target)
    };
    Ok(VariablePredicate {
        matched,
        continuation,
    })
}

pub fn party_item_condition(
    warriors: &[Warrior],
    extra: &[u8],
) -> Result<StoryContinuation, RuleError> {
    let extra = span(extra, 0, 10)?;
    if query_party_item(warriors, word(extra, 0)?)?.is_some() {
        return Ok(branch(word(extra, 2)?, word(extra, 6)?));
    }
    Ok(match word(extra, 4)? {
        0 => StoryContinuation::ReplaceScript(word(extra, 8)?),
        3 => StoryContinuation::EndScript,
        _ => StoryContinuation::Continue,
    })
}
