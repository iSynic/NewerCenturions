//! Looking after a step marks the squares in view of the chosen view, and walls stop sight unless the party sees through them.
use crate::{prelude::*, support};

#[test]
fn looking_after_a_step_marks_the_squares_seen_and_a_wall_stops_sight() {
    let mut content = content();
    let land = &mut content.scenario.as_mut().unwrap().maps[0];
    (land.map_stats, land.map_stats[7 * 40 + 13]) = (vec![0; 12000], 1);
    let mut state = support::mission_session(&content);
    let position = state.exploration.as_mut().unwrap();
    (position.x, position.y) = (40, 40);
    (position.floor, position.objects) = (vec![1; 8100], vec![0; 8100]);
    (position.floor[44 * 90 + 40], position.objects[43 * 90 + 40]) = (7, 7);
    let mut eye = state.clone();
    eye.party_condition[8] = 1;
    let seen = [(state, false), (eye.clone(), false), (eye, true)].map(|(mut state, small)| {
        apply(Command::Move { dx: 1, dy: 0 }, &mut state, &content).unwrap();
        let p = json!({"auto_refit": false, "small_combat_field": small});
        let look = json!({"WithPreferences": {"command": "RevealSight", "preferences": p}});
        apply(serde_json::from_value(look).unwrap(), &mut state, &content).unwrap();
        [44, 45, 48, 49].map(|x| state.exploration.as_ref().unwrap().visibility[x * 90 + 40])
    });
    assert_eq!(seen, [[1, 0, 0, 0], [1, 1, 1, 0], [1, 1, 0, 0]]);
}
