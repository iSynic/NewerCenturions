//! The icon in the cast rectangle while a target is chosen, and the facing it turns to.
use crate::SpellInfo;

/// The first cicn of a look's eight cycling frames.
const CYCLING_ICONS: i16 = 11984;
/// The first cicn of a look's eight facings.
const FACING_ICONS: i16 = 11992;

/// Which of eight ways the vector (`dx`, `dy`) points: 0 is up and the numbers run clockwise to
/// 7, as the screen's y grows downward. A zero vector gives `incoming`, the value the caller left
/// in the register that returns the facing.
///
/// Original: direction
pub fn direction(dx: i16, dy: i16, incoming: i8) -> i16 {
    match (dx.signum(), dy.signum()) {
        (1, -1) => 1,
        (1, 0) => 2,
        (1, 1) => 3,
        (0, 1) => 4,
        (-1, 1) => 5,
        (-1, 0) => 6,
        (-1, -1) => 7,
        (0, -1) => 0,
        _ => i16::from(incoming),
    }
}

/// The cicn of a look turned toward the cell `offset` from the acting warrior.
///
/// Original: missileset
pub fn facing_icon(look: i16, offset: (i16, i16)) -> i16 {
    look.wrapping_mul(8)
        .wrapping_add(FACING_ICONS)
        .wrapping_add(direction(offset.0, offset.1, offset.1 as i8))
}

/// The cicn plotted in the cast rectangle while the player chooses targets. A firearm keeps its
/// weapon icon; a spell, psi power or item in effect cycles through its look's eight frames; a
/// weapon turns `facing_look`'s icon toward the cell `offset` from the acting warrior.
///
/// Original: combatchoice
pub fn aim_icon(
    info: &SpellInfo,
    in_spell: i16,
    frame: i16,
    facing_look: i16,
    offset: (i16, i16),
) -> Option<i16> {
    if info.firearm {
        info.weapon_icon
    } else if in_spell != 0 {
        Some(
            info.look
                .wrapping_mul(8)
                .wrapping_add(CYCLING_ICONS)
                .wrapping_add(frame),
        )
    } else {
        Some(facing_icon(facing_look, offset))
    }
}

#[cfg(test)]
mod tests {
    use super::direction;

    #[test]
    fn eight_facings_run_clockwise_from_up_and_a_zero_vector_keeps_the_incoming_value() {
        let (dx, dy) = ([0, 9, 9, 9, 0, -9, -9, -9], [-9, -9, 0, 9, 9, 9, 0, -9]);
        assert!((0..8).all(|i| direction(dx[i], dy[i], 5) == i as i16));
        assert_eq!(direction(0, 0, 5), 5);
    }
}
