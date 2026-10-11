use crate::RuleError;
use centurions_content::{span, word};

pub const RECORD_BYTES: usize = 516;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PasswordSelection {
    Retry,
    Selected(i8),
}

pub fn attempt_count(record: &[u8]) -> Result<i16, RuleError> {
    Ok(i16::from(span(record, 0, 156)?[152] as i8))
}

pub fn dialogue(record: &[u8]) -> Result<i16, RuleError> {
    Ok(word(span(record, 0, 156)?, 154)?)
}

pub fn password_label(record: &[u8]) -> Result<&[u8], RuleError> {
    let label = span(record, 156 + 8 * 40, 40)?;
    if label[0] > 39 {
        return Err(RuleError::Invalid(
            "typed encounter label is too long".into(),
        ));
    }
    Ok(&label[1..1 + usize::from(label[0])])
}

pub fn select_password(record: &[u8], input: &[u8]) -> Result<PasswordSelection, RuleError> {
    let record = span(record, 0, RECORD_BYTES)?;
    if record[97] == 0 {
        return Err(RuleError::Invalid("typed input is unavailable".into()));
    }
    if input.is_empty() || input[0] == 0 {
        return Ok(PasswordSelection::Retry);
    }
    if input.iter().any(|value| *value > 127) {
        return Err(RuleError::Unsupported {
            action: "story.typed_input",
            dependency: "non-ASCII original character conversion",
        });
    }
    let normalized: Vec<u8> = input.iter().map(u8::to_ascii_lowercase).collect();
    if [b"map".as_slice(), b"boat", b"mission"].contains(&normalized.as_slice()) {
        return Err(RuleError::Unsupported {
            action: "story.typed_input",
            dependency: "original developer command",
        });
    }
    let expected = password_label(record)?;
    if expected.iter().any(|value| *value > 127) {
        return Err(RuleError::Unsupported {
            action: "story.typed_input",
            dependency: "non-ASCII original character conversion",
        });
    }
    // encounter2 stops at the first space in the expected text, not the submitted text.
    let matched = expected
        .iter()
        .take(40)
        .take_while(|value| **value != 0 && **value != b' ')
        .enumerate()
        .all(|(index, value)| normalized.get(index) == Some(value));
    Ok(PasswordSelection::Selected(if matched {
        record[97] as i8
    } else {
        4
    }))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedContinuation {
    Abort,
    Cancel,
    Branch { selector: i8, body: [u8; 32] },
}

pub fn continuation(
    record: &[u8],
    selected: i8,
    remaining: i16,
) -> Result<TypedContinuation, RuleError> {
    let record = span(record, 0, 156)?;
    match selected {
        -1 => Ok(TypedContinuation::Abort),
        0 => Ok(TypedContinuation::Cancel),
        1..=4 => {
            let selector = if selected == 4 && remaining == 1 && (record[152] as i8) > 1 {
                3
            } else {
                selected
            };
            Ok(TypedContinuation::Branch {
                selector,
                body: crate::story::dispatch_menu_branch(record, selector)?,
            })
        }
        _ => Err(RuleError::Unsupported {
            action: "story.typed_input",
            dependency: "typed encounter selector outside the four authored branches",
        }),
    }
}

pub fn finish_attempt(remaining: i16) -> (i16, bool) {
    (remaining.wrapping_sub(1), remaining > 1)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QuestionPlan {
    Continue,
    MoveAway,
    ReplaceScript(i16),
    MenuBranch(u8),
    TypedBranch(u8),
    EndScript,
}

pub fn question_labels(extra: &[u8]) -> Result<(i16, i16), RuleError> {
    Ok((word(extra, 6)?, word(extra, 8)?))
}

pub fn question_plan(extra: &[u8], answer: bool) -> Result<QuestionPlan, RuleError> {
    let extra = span(extra, 0, 10)?;
    if answer != (word(extra, 0)? == 0) {
        return Ok(QuestionPlan::Continue);
    }
    Ok(match word(extra, 2)? {
        0 => QuestionPlan::MoveAway,
        1 => QuestionPlan::ReplaceScript(word(extra, 4)?),
        2 | 3 => {
            let branch = word(extra, 4)?;
            if !(0..4).contains(&branch) {
                return Err(RuleError::Invalid("question branch is unavailable".into()));
            }
            if word(extra, 2)? == 2 {
                QuestionPlan::MenuBranch(branch as u8)
            } else {
                QuestionPlan::TypedBranch(branch as u8)
            }
        }
        4 => QuestionPlan::EndScript,
        _ => QuestionPlan::Continue,
    })
}
