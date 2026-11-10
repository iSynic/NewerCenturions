//! What the party has seen of the land it explores.
use crate::state::LAND_CELLS;
use crate::{Phase, RuleError, Session, combat::party_buffs};
use centurions_content::Content;
use centurions_content::mission::OpeningMission;

/// A view of the field map: how far it looks (the number of fans in each sweep, the fan that looks
/// straight ahead and the steps each sweep takes), the cell the party sits in, and how far across
/// and down the view can scroll.
struct View {
    fans: i16,
    straight: i16,
    steps: [i16; 2],
    centre: i32,
    limits: (i32, i32),
}

/// The 15 by 13 view of the 800 by 600 screen.
const LARGE_VIEW: View = View {
    fans: 15,
    straight: 8,
    steps: [9, 8],
    centre: 8,
    limits: (75, 76),
};

/// The 10 by 10 view of the 640 by 480 screen.
const SMALL_VIEW: View = View {
    fans: 10,
    straight: 5,
    steps: [6, 6],
    centre: 5,
    limits: (80, 80),
};

/// The view the screen size preference chooses.
fn view(large: bool) -> &'static View {
    if large { &LARGE_VIEW } else { &SMALL_VIEW }
}

/// The top left square of the field map view. On the large view the party sits in the ninth
/// column and row and the view stops scrolling at 75 across and 76 down; on the small view it sits
/// in the sixth and the view stops at 80 both ways.
///
/// Original: centerpictlarge, centerpictsmall
pub fn field_origin(x: i32, y: i32, large: bool) -> (i32, i32) {
    let view = view(large);
    (
        (x - view.centre).clamp(0, view.limits.0),
        (y - view.centre).clamp(0, view.limits.1),
    )
}

/// Marks the squares the party sees from where it stands, with the sweep of the view the screen
/// size preference chooses. Seen squares stay seen, on every land whether or not the land hides
/// what is unseen.
///
/// Original: cansee3, cansee2
pub(crate) fn reveal_sight(state: &mut Session, content: &Content) -> Result<(), RuleError> {
    if state.phase != Phase::Exploration {
        return Ok(());
    }
    let Some(position) = state.exploration.as_ref() else {
        return Ok(());
    };
    if position.visibility.len() != LAND_CELLS
        || position.floor.len() != LAND_CELLS
        || position.objects.len() != LAND_CELLS
    {
        return Ok(());
    }
    let Some(land) = state.land(content) else {
        return Ok(());
    };
    let through_walls = party_buffs::wizard_eye(&state.party_condition)?;
    let large = !state.command_preferences.small_combat_field;
    let (origin_x, origin_y) = field_origin(position.x, position.y, large);
    let seen = sweep(
        view(large),
        (position.x - origin_x, position.y - origin_y),
        (origin_x, origin_y),
        through_walls,
        |index| {
            if index < LAND_CELLS {
                Some(position.floor[index])
            } else {
                // The square past the last row reads on into the object layer.
                position.objects.get(index - LAND_CELLS).copied()
            }
        },
        land,
    )?;
    let position = state.exploration.as_mut().unwrap();
    for index in seen {
        position.visibility[index] = 1;
    }
    Ok(())
}

/// Casts the rays of a view from the party's square, a horizontal and a vertical sweep, each in
/// both directions, and gives the squares they reach in order. A ray stops after the first wall
/// it meets away from the party's own row or column, unless the party sees through walls. A ray
/// that starts off the land stays where it is.
///
/// Original: cansee3
fn sweep(
    view: &View,
    center: (i32, i32),
    origin: (i32, i32),
    through_walls: bool,
    floor: impl Fn(usize) -> Option<i16>,
    land: &OpeningMission,
) -> Result<Vec<usize>, RuleError> {
    let mut seen = Vec::new();
    for (sweep, steps) in view.steps.into_iter().enumerate() {
        for side in [-1_i16, 1] {
            for fan in 0..view.fans {
                let mut row = center.0 as i16;
                let mut column = center.1 as i16;
                let mut slope = match fan.cmp(&view.straight) {
                    std::cmp::Ordering::Less => -1_i16,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                };
                for _ in 0..steps {
                    if (if sweep == 0 { row } else { column }) == fan {
                        slope = 0;
                    }
                    let map_row = origin.0 + i32::from(row);
                    let map_column = origin.1 + i32::from(column);
                    if map_row < 0 || map_column < 0 {
                        continue;
                    }
                    let index = (map_row * 90 + map_column) as usize;
                    let tile = floor(index)
                        .ok_or_else(|| RuleError::Invalid("sight ray left the land".into()))?;
                    let blocked =
                        !through_walls && 0 < tile && tile < 600 && land.sight_blocker(tile)? == 1;
                    let away = if sweep == 0 {
                        i32::from(column) != center.1
                    } else {
                        i32::from(row) != center.0
                    };
                    if map_row < 90 && map_column < 90 {
                        seen.push(index);
                    }
                    if sweep == 0 {
                        row += slope;
                        column += side;
                    } else {
                        row += side;
                        column += slope;
                    }
                    if blocked && away {
                        break;
                    }
                }
            }
        }
    }
    Ok(seen)
}
