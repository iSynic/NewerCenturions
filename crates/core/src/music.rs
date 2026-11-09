//! The music title drawn on every track change.
use crate::random::RandomState;

/// How many titles the music playlist holds.
pub const TITLES: usize = 13;

/// Draws the next title from the game's random numbers: a number from 1 to 13 is drawn until it
/// lands on a title chosen on the playlist, and after a thousand and one draws without one the
/// search gives up. The title is the index into the playlist.
///
/// Original: submitmusic
pub fn choose_title(random: &mut RandomState, playlist: &[bool; TITLES]) -> Option<usize> {
    for _ in 0..=1000 {
        let title = random.bound(TITLES as i16) as usize - 1;
        if playlist[title] {
            return Some(title);
        }
    }
    None
}
