//! A warrior's career statistics: signed long counters kept by attacks,
//! captures and deaths and shown on the character screen.
use crate::ContentError;
use crate::bytes::{add_long_wrapping, long, put_long};

/// The career statistics a warrior keeps.
#[derive(Clone, Copy)]
pub enum Statistic {
    DamageReceived,
    DamageDealt,
    Hits,
    HitsReceived,
    Misses,
    AttacksAvoided,
    Kills,
    Deaths,
    Knockouts,
    ScansCast,
    VehiclesDisabled,
    VehiclesCaptured,
    /// Initial higher-level creation penalty plus later retreats.
    ScorePenalty,
}

impl Statistic {
    /// Locates the counter in the warrior record.
    fn offset(self) -> usize {
        match self {
            Self::DamageReceived => 0xa42,
            Self::DamageDealt => 0xa46,
            Self::Hits => 0xa4a,
            Self::HitsReceived => 0xa4e,
            Self::Misses => 0xa52,
            Self::AttacksAvoided => 0xa56,
            Self::Kills => 0xa5a,
            Self::Deaths => 0xa5e,
            Self::Knockouts => 0xa62,
            Self::ScansCast => 0xa66,
            Self::VehiclesDisabled => 0xa6a,
            Self::VehiclesCaptured => 0xa6e,
            Self::ScorePenalty => 0xa72,
        }
    }
}

/// A warrior's career statistics as a read-only view.
pub struct CharacterStatistics<'a>(pub &'a [u8]);
/// A warrior's career statistics as a view that can change them.
pub struct CharacterStatisticsMut<'a>(pub &'a mut [u8]);

impl CharacterStatistics<'_> {
    /// The statistic's current count.
    pub fn value(&self, statistic: Statistic) -> Result<i32, ContentError> {
        long(self.0, statistic.offset())
    }
}

impl CharacterStatisticsMut<'_> {
    /// Sets the statistic.
    pub fn set(&mut self, statistic: Statistic, value: i32) {
        let offset = statistic.offset();
        put_long(self.0, offset, value);
    }
    /// Adds `amount` to the statistic, wrapping.
    pub fn add(&mut self, statistic: Statistic, amount: i32) {
        let offset = statistic.offset();
        add_long_wrapping(self.0, offset, amount);
    }
}
