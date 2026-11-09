//! Stepping off the edge of a land onto the land that lies beside it.
use super::script_execution::enter_land;
use super::{active_map, unsupported};
use crate::{Event, RuleError, Session};
use centurions_content::{Content, layout::Layout};

/// The last column or row of a land.
const LAST_SQUARE: i32 = 89;
/// The screen column or row the party is shown in when it stands on the edge of the land: the
/// first square at the low edge, the last of the ten shown at the high edge.
const SCREEN_LOW: i32 = 0;
const SCREEN_HIGH: i32 = 9;
/// Where the view starts when the party is at the centre of the screen, and how far it can start.
const SCREEN_CENTRE: i32 = 5;
const VIEW_LIMIT: i32 = 80;

/// Where the party is shown on the screen for a column or row of the land. The view is centred on
/// the party and held inside the land.
fn screen_position(square: i32) -> i32 {
    square - (square - SCREEN_CENTRE).clamp(0, VIEW_LIMIT)
}

/// The land that lies beside `land` in the direction of the step, or `None` where none does. The
/// land is found in the Layout, which is read as land 0 being -1, and moved one cell along each
/// axis of the step whose screen position is at the edge. A cell off the Layout, an empty cell or
/// the land itself gives none; the row is changed before the column is tested, and both can apply.
///
/// Original: checklayout
fn neighbour_land(
    layout: Option<&Layout>,
    land: i32,
    step: [i8; 2],
    screen: (i32, i32),
) -> Option<i16> {
    let layout = layout?;
    let wanted = if land == 0 { -1 } else { land as i16 };
    let (mut row, mut column) = layout.find(wanted).unwrap_or((8, 16));
    if step[0] != 0 {
        if screen.0 == SCREEN_LOW {
            column -= 1;
        }
        if screen.0 == SCREEN_HIGH {
            column += 1;
        }
    }
    if step[1] != 0 {
        if screen.1 == SCREEN_LOW {
            row -= 1;
        }
        if screen.1 == SCREEN_HIGH {
            row += 1;
        }
    }
    match layout.cell(row, column)? {
        0 => None,
        cell if cell == wanted => None,
        -1 => Some(0),
        cell => Some(cell),
    }
}

/// Handles a step that would leave the land, out of combat. A column or row beyond the 90 squares
/// asks the Layout for the neighbouring land; where there is none the step is cancelled and the
/// party stays. Otherwise the land the party leaves is saved, the neighbour is entered and the
/// party appears on the opposite edge, and the step is done. `None` is returned when the step
/// stays within the land.
///
/// Original: checkkeypad
pub(super) fn cross_edge(
    state: &mut Session,
    content: &Content,
    dx: i8,
    dy: i8,
) -> Result<Option<Vec<Event>>, RuleError> {
    let position = state.exploration.as_ref().unwrap();
    let (x, y, land) = (position.x, position.y, position.map);
    let outside = |square: i32, step: i8| !(0..=LAST_SQUARE).contains(&(square + i32::from(step)));
    if !outside(x, dx) && !outside(y, dy) {
        return Ok(None);
    }
    let layout = active_map(state, content)?.layout.as_ref();
    let screen = (screen_position(x), screen_position(y));
    let mut step = [dx, dy];
    let mut sliding = false;
    let mut destination = 0;
    for axis in 0..2 {
        let square = [x, y][axis];
        if outside(square, step[axis]) {
            match neighbour_land(layout, land, step, screen) {
                None => {
                    step = [0, 0];
                    destination = -2;
                }
                Some(found) => {
                    sliding = true;
                    destination = i32::from(found);
                }
            }
        }
    }
    if !sliding {
        return Ok(Some(vec![]));
    }
    if destination < 0 {
        return Err(unsupported(
            "exploration.land_edge",
            "a corner step whose second axis has no neighbouring land",
        ));
    }
    enter_land(state, content, destination)?;
    let position = state.exploration.as_mut().unwrap();
    for (axis, square, screen) in [
        (0, &mut position.x, screen.0),
        (1, &mut position.y, screen.1),
    ] {
        if step[axis] != 0 {
            if screen == SCREEN_LOW {
                *square = LAST_SQUARE;
            } else if screen == SCREEN_HIGH {
                *square = 0;
            }
        }
    }
    Ok(Some(vec![Event::Moved {
        x: position.x,
        y: position.y,
    }]))
}
