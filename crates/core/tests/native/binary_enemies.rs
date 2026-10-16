use crate::prelude::*;

use centurions_content::combat::BattleRecord;

use centurions_core::combat::place_enemies_at_difficulty;

#[test]
fn enemy_setup_matches_whole_records_positions_map_and_randomness() {
    for name in [
        "enemies.json",
        "enemies-opening.json",
        "difficulty-enemies.json",
    ] {
        let fixture = crate::support::fixture(name);
        let map_fixture: serde_json::Value = crate::support::fixture("combat-map.json");
        let mission = OpeningMission {
            default_tile: 0,
            ..crate::support::terrain_mission(&map_fixture)
        };
        let battle = BattleRecord::read(&fixture.bytes("battle"), 0).unwrap();
        for case in fixture.array("cases") {
            let mut map = TacticalMap {
                tiles: fixture.words("map"),
                shift: (-39, -15),
                edge_word: 0,
            };
            let mut random = RandomState::new(case.n("seed"));
            let base = case.n::<i16>("base");
            let result = place_enemies_at_difficulty(
                &mut map,
                &mission,
                &battle,
                &fixture.bytes("definitions"),
                (base, base),
                &mut random,
                case.get("difficulty")
                    .map_or(0, |value| value.as_i64().unwrap() as i16),
            )
            .unwrap();
            crate::support::native_expect! {
            scalar result.records.len() => &case["count"], "enemy setup count";
            bytes crate::support::record_bytes(&result.records) => &case["records"], "enemy setup records";
            words map.tiles => &case["tiles"], "enemy setup map";
            scalar random.seed => &case["after_seed"], "enemy setup seed";
            scalar random.draws => &case["draws"], "enemy setup draws";
            }
            assert_eq!(
                result.positions,
                crate::support::positions(&case.bytes("positions"))
            );
        }
    }
}
