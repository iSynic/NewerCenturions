use crate::prelude::*;

#[test]
fn first_turn_matches_whole_records_and_random_draws() {
    let fixture: serde_json::Value = crate::support::fixture("turn.json");
    for case in fixture.array("cases") {
        let mut warriors: Vec<_> = crate::support::warriors(&case["warriors"]);
        let mut enemies = vec![MonsterRecord {
            bytes: case.bytes("enemy"),
        }];
        let mut random = RandomState::new(1);
        initialize_turn(
            case.n::<i16>("body"),
            &mut warriors,
            &mut enemies,
            &[12, 14, 12, 14, 10, 12, 10, 10, 14, 16, 0, 12],
            &mut random,
        )
        .unwrap();
        crate::support::native_expect! {
        bytes crate::support::record_bytes(&warriors) => &case["after_warriors"], format!("body {}",case["body"]);
        bytes enemies[0].bytes => &case["after_enemy"], "binary_turn";
        scalar random.seed => &case["seed"], "binary_turn";
        scalar random.draws => &case["draws"], "binary_turn";
        }
    }
}
