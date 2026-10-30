//! Values of the saved Information block that scripts and effects reach by
//! address.
use crate::Session;

/// The ten party durations.
const PARTY_DURATIONS: std::ops::Range<i64> = 0x442562..0x442576;
/// The difficulty.
const DIFFICULTY: i64 = 0x442576;
/// The 128 story flags.
const FLAGS: std::ops::Range<i64> = 0x44258c..0x44260c;

/// Reads an Information value from the session field that holds it.
pub(crate) fn read_word(
    party_condition: &[u8],
    difficulty: i16,
    flags: &[u8],
    address: i64,
) -> Option<i16> {
    let bytes = |owner: &[u8], base: i64| {
        let offset = (address - base) as usize;
        Some(i16::from_be_bytes([owner[offset], owner[offset + 1]]))
    };
    match address {
        _ if address % 2 != 0 => None,
        _ if PARTY_DURATIONS.contains(&address) => bytes(party_condition, PARTY_DURATIONS.start),
        DIFFICULTY => Some(difficulty),
        _ if FLAGS.contains(&address) => bytes(flags, FLAGS.start),
        _ => None,
    }
}

/// Writes an Information value; false leaves the session unchanged.
pub(crate) fn write_word(
    party_condition: &mut [u8],
    difficulty: &mut i16,
    flags: &mut [u8],
    address: i64,
    value: i16,
) -> bool {
    let put = |owner: &mut [u8], base: i64| {
        let offset = (address - base) as usize;
        owner[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        true
    };
    match address {
        _ if address % 2 != 0 => false,
        _ if PARTY_DURATIONS.contains(&address) => put(party_condition, PARTY_DURATIONS.start),
        // Only difficulties -2..2 are kept; other values have no owner.
        DIFFICULTY if (-2..=2).contains(&value) => {
            *difficulty = value;
            true
        }
        _ if FLAGS.contains(&address) => put(flags, FLAGS.start),
        _ => false,
    }
}

impl Session {
    /// The value at an even Information address, if the session holds it.
    pub(crate) fn information_word(&self, address: i64) -> Option<i16> {
        read_word(&self.party_condition, self.difficulty, &self.flags, address)
    }

    /// Writes an Information value; false leaves the session unchanged.
    pub(crate) fn set_information_word(&mut self, address: i64, value: i16) -> bool {
        let written = write_word(
            &mut self.party_condition,
            &mut self.difficulty,
            &mut self.flags,
            address,
            value,
        );
        if written
            && address == DIFFICULTY
            && let Some(combat) = self.combat.as_mut()
        {
            combat.difficulty = value;
        }
        written
    }
}
