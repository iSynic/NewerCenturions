use crate::RuleError;

/// The ticks a script delay waits: the parameter plus one, counting a parameter of zero as zero.
pub fn elapsed_ticks(parameter: i16) -> Result<u32, RuleError> {
    elapsed_ticks_with_default(parameter, 0)
}

/// Waits until the elapsed ticks, unsigned, exceed the signed parameter.
///
/// Original: delay
pub fn elapsed_ticks_with_default(parameter: i16, default: i16) -> Result<u32, RuleError> {
    let duration = if parameter == 0 { default } else { parameter };
    (i32::from(duration) as u32)
        .checked_add(1)
        .ok_or(RuleError::Unsupported {
            action: "story.delay",
            dependency: "nonterminating unsigned delay",
        })
}
