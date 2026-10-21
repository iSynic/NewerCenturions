use crate::{prelude::*, support};

use centurions_core::combat::build_tactical_map;

#[test]
fn tactical_map_matches_all_cells_and_random_consumption() {
    let fixture = support::fixture("combat-map.json");
    let land = fixture.bytes("land");
    let properties = fixture.bytes("properties");
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
        ..support::mission()
    };
    for case in fixture.array("cases") {
        let mut random = RandomState::new(case.n("seed"));
        let map = build_tactical_map(&mission, (case.n("x"), case.n("y")), 0, &mut random).unwrap();
        let expected = case.words("tiles");
        assert_eq!(
            map.tiles, expected,
            "position {},{} seed {}",
            case["x"], case["y"], case["seed"]
        );
        assert_eq!(map.shift, (case.n("shift_x"), case.n("shift_y")));
        assert_eq!(map.edge_word, case.n::<i16>("edge_after"));
        support::assert_rng(&random, &case["after_seed"], &case["draws"]);
    }
}
