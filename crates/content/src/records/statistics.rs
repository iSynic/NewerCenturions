//! LONG counters used by attack, capture, killbody and viewcharacter.
use crate::{ContentError, mission::long};

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

pub struct CharacterStatistics<'a>(pub &'a [u8]);
pub struct CharacterStatisticsMut<'a>(pub &'a mut [u8]);

impl CharacterStatistics<'_> {
    pub fn value(&self, statistic: Statistic) -> Result<i32, ContentError> {
        long(self.0, statistic.offset())
    }
}

impl CharacterStatisticsMut<'_> {
    pub fn set(&mut self, statistic: Statistic, value: i32) {
        let offset = statistic.offset();
        self.0[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
    }
    pub fn add(&mut self, statistic: Statistic, amount: i32) {
        let offset = statistic.offset();
        let previous = i32::from_be_bytes(self.0[offset..offset + 4].try_into().unwrap());
        self.0[offset..offset + 4].copy_from_slice(&previous.wrapping_add(amount).to_be_bytes());
    }
}
