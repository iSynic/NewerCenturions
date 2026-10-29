use crate::RuleError;

pub fn elapsed_ticks(parameter: i16) -> Result<u32, RuleError> {
    elapsed_ticks_with_default(parameter, 0)
}

/// delay1205f8 compares unsigned elapsed ticks strictly above the signed WORD.
pub fn elapsed_ticks_with_default(parameter: i16, default: i16) -> Result<u32, RuleError> {
    let duration = if parameter == 0 { default } else { parameter };
    (i32::from(duration) as u32)
        .checked_add(1)
        .ok_or(RuleError::Unsupported {
            action: "story.delay",
            dependency: "nonterminating unsigned delay",
        })
}
