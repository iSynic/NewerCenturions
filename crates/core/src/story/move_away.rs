use serde::{Deserialize, Serialize};

/// The result of backing away: the square the party returns to, the floor value of the square it
/// backed away from, and whether the controls need updating.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveAway {
    pub x: i32,
    pub y: i32,
    pub floor: i16,
    pub mark: i8,
    pub flags: [i8; 6],
    pub saved_delta: [i8; 2],
    pub update_controls: bool,
}

/// Steps the party back one square from the direction it moved.
pub fn move_away(x: i32, y: i32, dx: i8, dy: i8, floor_ahead: i16, flags: [i8; 6]) -> MoveAway {
    MoveAway {
        x: x.wrapping_sub(i32::from(dx)),
        y: y.wrapping_sub(i32::from(dy)),
        floor: floor_ahead.wrapping_abs(),
        mark: 1,
        flags: [0; 6],
        saved_delta: [dx, dy],
        update_controls: flags[3] != 0 || flags[5] != 0,
    }
}
