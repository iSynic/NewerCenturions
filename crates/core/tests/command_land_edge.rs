//! Stepping off the edge of a land, through the saved Move command.
use crate::{prelude::*, support};
use centurions_content::layout::Layout;

/// With no neighbour the step is cancelled; with one the party appears on the opposite edge of it,
/// and the land it left keeps its changes.
#[test]
fn edge_steps_slide_to_the_laid_out_neighbour_or_are_cancelled() {
    let mut content = content();
    let mut state = support::mission_session(&content);
    let walk = |state: &mut Session, content: &Content, at: (i32, i32), (dx, dy): (i8, i8)| {
        let position = state.exploration.as_mut().unwrap();
        (position.x, position.y) = at;
        support::saved_command(Command::Move { dx, dy }, state, content);
        let position = state.exploration.as_ref().unwrap();
        (position.map, position.x, position.y)
    };
    state.exploration.as_mut().unwrap().facing = 1;
    assert_eq!(walk(&mut state, &content, (0, 40), (-1, 0)), (0, 0, 40));
    assert_eq!(state.exploration.as_ref().unwrap().facing, 1);
    // A cancelled edge step neither warns of fatigue nor breaks camp.
    let mut tired = state.clone();
    tired.exploration.as_mut().unwrap().fatigue = 135;
    tired.field_controls.get_or_insert_default().camping = true;
    apply(Command::Move { dx: -1, dy: 0 }, &mut tired, &content).unwrap();
    assert!(tired.field_controls.unwrap().camping && tired.exploration.unwrap().fatigue == 135);
    // Land 0 has land 3 to its right and land 4 below it.
    let mut bytes = [0; 256];
    bytes[..2].copy_from_slice(&(-1_i16).to_be_bytes());
    (bytes[3], bytes[33]) = (3, 4);
    state.exploration.as_mut().unwrap().floor[0] = 77;
    for map in &mut content.scenario.as_mut().unwrap().maps {
        Arc::make_mut(&mut map.shared).layout = Some(Layout::read(&bytes));
    }
    assert_eq!(walk(&mut state, &content, (89, 40), (1, 0)), (3, 0, 40));
    assert_eq!(walk(&mut state, &content, (0, 40), (-1, 0)), (0, 89, 40));
    assert_eq!(state.exploration.as_ref().unwrap().floor[0], 77);
    assert_eq!(walk(&mut state, &content, (40, 89), (0, 1)), (4, 40, 0));
}
