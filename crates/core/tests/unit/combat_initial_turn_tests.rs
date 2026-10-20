use super::battle_entry::initial_cursor;
use super::*;
use crate::RandomState;
use crate::test_support::{bytes, fixture};
use centurions_content::combat::MonsterRecord;
#[test]
fn sparse_initial_order_matches_original_getup_execution() {
    let fixture = fixture("sparse-turn.json");
    for case in fixture["cases"].as_array().unwrap() {
        let body = case["body"].as_i64().unwrap() as i16;
        let first = case["first_slot"].as_u64().unwrap() as usize;
        let mut order = vec![-1; 110];
        order[first] = body;
        let cursor = initial_cursor(&order).unwrap();
        assert_eq!(cursor, case["cursor"].as_u64().unwrap() as usize);
        let mut warriors = bytes(&case["warriors"])
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|w| Warrior::read(w).unwrap())
            .collect::<Vec<_>>();
        let mut enemies = vec![MonsterRecord {
            bytes: bytes(&case["enemy"]),
        }];
        let mut random = RandomState::new(1);
        initialize_turn(
            order[cursor],
            &mut warriors,
            &mut enemies,
            &[12, 14, 12, 14, 10, 12, 10, 10, 14, 16, 0, 12],
            &mut random,
        )
        .unwrap();
        assert_eq!(
            warriors
                .iter()
                .flat_map(|w| w.bytes.clone())
                .collect::<Vec<_>>(),
            bytes(&case["after_warriors"])
        );
        assert_eq!(enemies[0].bytes, bytes(&case["after_enemy"]));
        assert_eq!(random.seed, case["seed"].as_u64().unwrap() as u32);
        assert_eq!(random.draws, case["draws"].as_u64().unwrap());
    }
}
