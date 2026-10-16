use crate::prelude::*;

use centurions_core::combat::move_warrior;

#[test]
fn walking_matches_instruction_state_and_effects() {
    let fixture: serde_json::Value = crate::support::fixture("walk.json");
    let properties = fixture.bytes("properties");
    let content = Content {
        tutorials: vec![],
        identity: "walking".into(),
        warriors: vec![],
        items: vec![],
        resources: vec![],
        original_services: None,
        scenario: Some(centurions_content::mission::ScenarioContent::single(
            OpeningMission {
                tile_properties: properties[64..12064].to_vec(),
                default_tile: 0,
                shared: std::sync::Arc::new(centurions_content::mission::StoryTables {
                    ..crate::support::stories()
                }),
                ..crate::support::mission()
            },
        )),
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
        state.combat = Some(Combat {
            battle: 2,
            map: TacticalMap {
                tiles,
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: vec![(20, 20); 6],
                under_tiles: vec![1155; 6],
            },
            order: vec![-1, 0],
            ..crate::support::battle()
        });
        let events = move_warrior(&mut state, &content, dx, dy).unwrap();
        assert_eq!(state.warriors[0].bytes, case.bytes("after"));
        let combat = state.combat.unwrap();
        assert_eq!(
            crate::support::word_bytes(&combat.map.tiles),
            case.bytes("tiles")
        );
        assert_eq!(
            combat.squad.positions[0],
            (case.n::<i16>("x"), case.n::<i16>("y"))
        );
        crate::support::expect(combat.squad.under_tiles[0], &case["under"]);
        let sounds: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let Event::Sound { resource_id } = e {
                    Some(i64::from(*resource_id))
                } else {
                    None
                }
            })
            .collect();
        let mut expected_sounds = case
            .array("sounds")
            .iter()
            .map(|v| v.as_i64().unwrap())
            .collect::<Vec<_>>();
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
