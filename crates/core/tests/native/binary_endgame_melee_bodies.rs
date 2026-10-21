use crate::{native_common, prelude::*, support};
use native_common::original_file as data;

use native_common::melee_combat as combat;
#[test]
fn endgame_default194_melee_matches_native_full_records_packet_and_randomness() {
    let fixture = support::fixture("endgame-melee-bodies.json");

    crate::binary_endgame_melee_bodies::compare_endgame_default194_melee_matches_native_full_records_packet_and_randomness(fixture);
}

pub(super) fn compare_endgame_default194_melee_matches_native_full_records_packet_and_randomness(
    fixture: Value,
) {
    let items = data("Data Files/Data ID");
    let attacks = data("Data Files/Data S");
    let cases = fixture.array("cases");

    for (index, case) in cases.iter().enumerate() {
        let mut combat = combat(&case.bytes("records_before"));
        let before = combat.clone();
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let id = case.n::<usize>("weapon");
        let result = enemy_weapons::melee_enemy_with_packet(
            &mut combat,
            &mut random,
            (10, 11, case.n::<usize>("index"), case.n::<i16>("actor")),
            &items[id * 150..(id + 1) * 150],
            0,
            &attacks,
            &[0; 58],
        );
        let (result, packet) =
            result.unwrap_or_else(|error| panic!("case{index} weapon{id}: {error:?}"));
        let actual: Vec<_> = support::record_bytes(&combat.enemies.records);
        support::native_expect! {
            bytes actual => &case["records_after"], &format!("case{index} records");
            bytes packet => &case["packet_after"], &format!("case{index} packet");
            rng random => (&case["seed_after"], &case["rng_draws"]), "binary_endgame_melee_bodies";
        }
        let sound = if result.damage == 0 {
            650
        } else if case["loaded"] == 0 {
            let b = &before.enemies.records[0].bytes;
            let a = case.n::<usize>("index");
            let offset = if b[0x93 + a * 4] == 0 {
                0x95
            } else {
                0x95 + a * 4
            };
            i16::from(b[offset] as i8)
        } else {
            i16::from_be_bytes(items[id * 150 + 14..id * 150 + 16].try_into().unwrap())
        };
        assert_eq!(
            case["sounds"],
            serde_json::json!([sound]),
            "case{index} native sound"
        );
        assert!(result.sounds.is_empty());
    }
}
