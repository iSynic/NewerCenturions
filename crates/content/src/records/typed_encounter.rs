use crate::{ContentError, span, word};

pub const TYPED_ENCOUNTER_BYTES: usize = 516;

/// Original 516-byte typed encounter record read by encounter2. The 156-byte
/// header holds the authored alternatives; 40-byte Pascal labels follow it.
/// Selectors stay signed bytes; callers keep their own fallbacks and bounds checks.
pub struct TypedEncounterRecord<'a>(pub &'a [u8]);

impl<'a> TypedEncounterRecord<'a> {
    pub const HEADER_BYTES: usize = 156;
    pub const SPELL_ALTERNATIVES: usize = 10;
    pub const ITEM_ALTERNATIVES: usize = 5;

    /// Selector for a matched physical action set; zero disables physical actions.
    pub fn action_selector(&self) -> i8 {
        self.0[96] as i8
    }
    /// Selector for a matched password; zero disables typed input.
    pub fn password_selector(&self) -> i8 {
        self.0[97] as i8
    }
    /// One byte per physical action; nonzero marks an action the match requires.
    pub fn required_actions(&self) -> &'a [u8] {
        &self.0[98..106]
    }
    pub fn spell_alternative(&self, index: usize) -> Result<i16, ContentError> {
        word(self.0, 106 + index * 2)
    }
    pub fn spell_selector(&self, index: usize) -> i8 {
        self.0[126 + index] as i8
    }
    /// encounter2 122a1a matches the selected item against these alternatives.
    pub fn item_alternative(&self, index: usize) -> Result<i16, ContentError> {
        word(self.0, 136 + index * 2)
    }
    pub fn item_selector(&self, index: usize) -> i8 {
        self.0[146 + index] as i8
    }
    pub fn attempts(&self) -> i8 {
        self.0[152] as i8
    }
    pub fn dialogue(&self) -> Result<i16, ContentError> {
        word(self.0, 154)
    }
    /// The ninth 40-byte label after the header, still in its Pascal form.
    pub fn password_label(&self) -> Result<&'a [u8], ContentError> {
        span(self.0, Self::HEADER_BYTES + 8 * 40, 40)
    }
}

/// Ten-byte EXTRA record for a yes/no question presented from a story script.
pub struct QuestionExtra<'a>(pub &'a [u8]);

impl QuestionExtra<'_> {
    pub const BYTES: usize = 10;

    /// Zero applies the outcome to a yes answer; any other value to a no answer.
    pub fn answer(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    pub fn outcome(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// Replacement script or branch index, depending on the outcome.
    pub fn outcome_parameter(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    pub fn labels(&self) -> Result<(i16, i16), ContentError> {
        Ok((word(self.0, 6)?, word(self.0, 8)?))
    }
}
