use crate::prelude::*;

use centurions_core::combat::build_tactical_map;

#[derive(Deserialize)]
struct Fixture {
    land: String,
    properties: String,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Case {
    x: i32,
    y: i32,
    seed: u32,
    after_seed: u32,
    draws: u64,
    edge_after: i16,
    shift_x: i16,
    shift_y: i16,
    tiles: String,
}
#[test]
fn tactical_map_matches_all_cells_and_random_consumption() {
    let fixture: Fixture =
        serde_json::from_value(crate::support::fixture("combat-map.json")).unwrap();
    let land = decode(&fixture.land);
    let properties = decode(&fixture.properties);
    let mission = OpeningMission {
        floor: land[..16200]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|v| i16::from_be_bytes(*v))
            .collect(),
        events: land[32400..64800]
            .as_chunks::<4>()
            .0
            .iter()
            .map(|v| i32::from_be_bytes(*v))
            .collect(),
        tile_properties: properties[64..12064].to_vec(),
        default_tile: word(&properties, 0).unwrap(),
        minutes_per_tick: if word(&properties, 2).unwrap() == 0 {
            5
        } else {
            1
        },
        shared: std::sync::Arc::new(centurions_content::mission::StoryTables {
            ..crate::support::stories()
        }),
        ..crate::support::mission()
    };
    for case in fixture.cases {
        let mut random = RandomState::new(case.seed);
        let map = build_tactical_map(&mission, (case.x, case.y), 0, &mut random).unwrap();
        let expected: Vec<_> = crate::support::words(&decode(&case.tiles));
        assert_eq!(
            map.tiles, expected,
            "position {},{} seed {}",
            case.x, case.y, case.seed
        );
        assert_eq!(map.shift, (case.shift_x, case.shift_y));
        assert_eq!(map.edge_word, case.edge_after);
        assert_eq!((random.seed, random.draws), (case.after_seed, case.draws));
    }
}
