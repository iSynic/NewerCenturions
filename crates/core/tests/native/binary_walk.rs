use crate::{prelude::*, support};

use centurions_core::combat::move_warrior;

#[test]
fn walking_matches_instruction_state_and_effects() {
    let fixture: serde_json::Value = support::fixture("walk.json");
    let content = Content {
        scenario: Some(centurions_content::mission::ScenarioContent::single(
            OpeningMission {
                default_tile: 0,
                ..support::terrain_mission(&fixture)
            },
        )),
        ..support::fixture_content("walking")
    };
    for case in fixture.array("cases") {
        let dx = case.n::<i8>("dx");
        let dy = case.n::<i8>("dy");
        let mut state = Session::new(&content);
        state.warriors = vec![Warrior::read(&case.bytes("input")).unwrap(); 6];
        let mut tiles = vec![1155; 8100];
        tiles[25 * 90 + 25] = 0;
        tiles[(25 + i32::from(dx)) as usize * 90 + (25 + i32::from(dy)) as usize] =
            case.n::<i16>("tile");
        let mut combat = support::battle();
        combat.battle = 2;
        combat.map.tiles = tiles;
        combat.squad.positions = vec![(20, 20); 6];
        combat.order = vec![-1, 0];
        state.combat = Some(combat);
        let events = move_warrior(&mut state, &content, dx, dy).unwrap();
        assert_eq!(state.warriors[0].bytes, case.bytes("after"));
        let combat = state.combat.unwrap();
        assert_eq!(support::word_bytes(&combat.map.tiles), case.bytes("tiles"));
        assert_eq!(
            combat.squad.positions[0],
            (case.n::<i16>("x"), case.n::<i16>("y"))
        );
        support::expect(combat.squad.under_tiles[0], &case["under"]);
        let sounds = support::event_sounds(&events);
        let mut expected_sounds = case.n::<Vec<i16>>("sounds");
        // This probe stops at the failure label before the caller's final sound.
        // The complete caller is covered by binary_warrior_opportunity.
        if !events
            .iter()
            .any(|event| matches!(event, Event::TacticalMoved { .. }))
        {
            expected_sounds.push(-130);
        }
        assert_eq!(sounds, expected_sounds);
        assert_eq!(state.random.draws, 0);
    }
}
