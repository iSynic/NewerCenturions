use crate::{RuleError, story_inventory::query_party_item};
use centurions_content::{
    Warrior,
    records::{PartyItemConditionExtra, VariablePredicateExtra, VariableUpdateExtra},
};

/// Where a script goes after a variable test: carry on, to another script, an encounter, a typed
/// encounter, or its end.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StoryContinuation {
    Continue,
    ReplaceScript(i16),
    Encounter(i16),
    TypedEncounter(i16),
    EndScript,
}

/// The continuation a branch mode and target name.
pub(crate) fn branch(mode: i16, target: i16) -> StoryContinuation {
    match mode {
        0 => StoryContinuation::ReplaceScript(target),
        1 => StoryContinuation::Encounter(target),
        2 => StoryContinuation::TypedEncounter(target),
        _ => StoryContinuation::Continue,
    }
}

/// The place of a variable among the mission flags.
fn index(flags: &[u8], variable: i16) -> Result<usize, RuleError> {
    let index = usize::try_from(variable)
        .map_err(|_| RuleError::Invalid("story variable index is negative".into()))?;
    if index >= flags.len() {
        return Err(RuleError::Invalid("story variable is unavailable".into()));
    }
    Ok(index)
}

/// The mission flags after a variable update and where the script goes next.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VariableUpdate {
    pub flags: Vec<u8>,
    pub continuation: StoryContinuation,
}

/// Update Variable adds a signed word, then clamps and stores a signed byte.
pub fn update_variable(flags: &[u8], extra: &[u8]) -> Result<VariableUpdate, RuleError> {
    let extra = VariableUpdateExtra::new(extra)?;
    let index = index(flags, extra.variable()?)?;
    let mut flags = flags.to_vec();
    if index < 100 {
        let value = i16::from(flags[index] as i8).wrapping_add(extra.amount()?);
        flags[index] = value.clamp(-127, 127) as i8 as u8;
    }
    let threshold = extra.threshold()?;
    let continuation = if threshold != 0 && i16::from(flags[index] as i8) >= threshold {
        branch(extra.branch()?.wrapping_sub(1), extra.target()?)
    } else {
        StoryContinuation::Continue
    };
    Ok(VariableUpdate {
        flags,
        continuation,
    })
}

/// Whether a variable test matched and where the script goes next.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VariablePredicate {
    pub matched: bool,
    pub continuation: StoryContinuation,
}

/// Tests a mission variable against its threshold and picks the target for a match or a miss.
pub fn variable_predicate(flags: &[u8], extra: &[u8]) -> Result<VariablePredicate, RuleError> {
    let extra = VariablePredicateExtra::new(extra)?;
    let index = index(flags, extra.variable()?)?;
    let matched = i16::from(flags[index] as i8) >= extra.threshold()?;
    let target = extra.target(matched)?;
    let continuation = if target == 0 {
        StoryContinuation::Continue
    } else {
        branch(extra.branch()?, target)
    };
    Ok(VariablePredicate {
        matched,
        continuation,
    })
}

/// Branches on whether the party carries an item: a found item takes its branch and target; a
/// missing one replaces the script, ends it or continues.
pub fn party_item_condition(
    warriors: &[Warrior],
    extra: &[u8],
) -> Result<StoryContinuation, RuleError> {
    let extra = PartyItemConditionExtra::new(extra)?;
    if query_party_item(warriors, extra.item()?)?.is_some() {
        return Ok(branch(extra.found_branch()?, extra.found_target()?));
    }
    Ok(match extra.missing_action()? {
        0 => StoryContinuation::ReplaceScript(extra.missing_script()?),
        3 => StoryContinuation::EndScript,
        _ => StoryContinuation::Continue,
    })
}
