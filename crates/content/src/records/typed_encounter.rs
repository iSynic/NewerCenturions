use crate::{ContentError, span, word};

/// Size of one typed encounter.
pub const TYPED_ENCOUNTER_BYTES: usize = 516;

/// A 516-byte typed encounter (`encount2`): a 156-byte header of authored
/// alternatives followed by 40-byte Pascal labels. Selectors are signed bytes;
/// callers make their own fallbacks and bounds checks.
pub struct TypedEncounterRecord<'a>(pub &'a [u8]);

impl<'a> TypedEncounterRecord<'a> {
    /// Size of the encounter's header, which the labels follow.
    pub const HEADER_BYTES: usize = 156;
    /// How many scans the encounter can match.
    pub const SPELL_ALTERNATIVES: usize = 10;
    /// How many items the encounter can match.
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
    /// Scan `index` the encounter matches; a value below 7 matches any scan whose defense
    /// category it is.
    pub fn spell_alternative(&self, index: usize) -> Result<i16, ContentError> {
        word(self.0, 106 + index * 2)
    }
    /// The branch taken when scan `index` is cast.
    pub fn spell_selector(&self, index: usize) -> i8 {
        self.0[126 + index] as i8
    }
    /// The selected item is matched against these alternatives.
    pub fn item_alternative(&self, index: usize) -> Result<i16, ContentError> {
        word(self.0, 136 + index * 2)
    }
    /// The branch taken when item `index` is used.
    pub fn item_selector(&self, index: usize) -> i8 {
        self.0[146 + index] as i8
    }
    /// How many tries the party gets.
    pub fn attempts(&self) -> i8 {
        self.0[152] as i8
    }
    /// The dialogue shown with the encounter.
    pub fn dialogue(&self) -> Result<i16, ContentError> {
        word(self.0, 154)
    }
    /// One 40-byte Pascal label after the header; the first eight name the actions.
    pub fn action_label(&self, index: usize) -> &'a [u8] {
        let start = Self::HEADER_BYTES + index * 40;
        &self.0[start..start + 40]
    }
    /// The ninth 40-byte label after the header, still in its Pascal form.
    pub fn password_label(&self) -> Result<&'a [u8], ContentError> {
        span(self.0, Self::HEADER_BYTES + 8 * 40, 40)
    }
}

/// Ten-byte EXTRA record for a yes/no question presented from a story script.
pub struct QuestionExtra<'a>(pub &'a [u8]);

impl QuestionExtra<'_> {
    /// Size of the record.
    pub const BYTES: usize = 10;

    /// Zero applies the outcome to a yes answer; any other value to a no answer.
    pub fn answer(&self) -> Result<i16, ContentError> {
        word(self.0, 0)
    }
    /// What the chosen answer does: 0 steps the party back, 1 replaces the script, 2 or 3 takes a
    /// menu or typed encounter branch, 4 ends the script; other values carry on.
    pub fn outcome(&self) -> Result<i16, ContentError> {
        word(self.0, 2)
    }
    /// Replacement script or branch index, depending on the outcome.
    pub fn outcome_parameter(&self) -> Result<i16, ContentError> {
        word(self.0, 4)
    }
    /// The two words that name the labels of the answers.
    pub fn labels(&self) -> Result<(i16, i16), ContentError> {
        Ok((word(self.0, 6)?, word(self.0, 8)?))
    }
}
