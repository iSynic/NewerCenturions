use centurions_content::{Warrior, word};
use centurions_core::{
    RandomState,
    combat::{SquadPlacement, TacticalMap},
    story_party::{
        PartySaveContext, apply_group_mask, group_encounter, group_return, select_party,
    },
    story_variables::StoryContinuation,
};
use serde_json::Value;

fn bytes(value: &Value) -> Vec<u8> {
    let text = value.as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).unwrap())
        .collect()
}

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../evidence/fixtures/story-party.json")).unwrap()
}

fn warriors(value: &Value) -> Vec<Warrior> {
    bytes(value)
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|record| Warrior::read(record).unwrap())
        .collect()
}

fn words(value: &Value) -> Vec<i16> {
    bytes(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|record| i16::from_be_bytes(*record))
        .collect()
}

fn positions(value: &Value) -> Vec<(i16, i16)> {
    words(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|record| (record[0], record[1]))
        .collect()
}

fn assert_bytes(actual: &[u8], expected: &[u8], label: &str) {
    assert_eq!(actual.len(), expected.len(), "{label} byte count");
    if let Some(offset) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "{label} byte {offset:#x}: actual {}, original {}",
            actual[offset], expected[offset]
        );
    }
}

fn flatten(warriors: &[Warrior]) -> Vec<u8> {
    warriors
        .iter()
        .flat_map(|warrior| warrior.bytes.iter().copied())
        .collect()
}

#[test]
fn party_selection_matches_original_masks_complete_records_and_randomness() {
    let fixture = fixture();
    for case in fixture["selection_cases"].as_array().unwrap() {
        let warriors = warriors(&case["before"]);
        let context = &case["context"];
        let context = PartySaveContext {
            casting: context["casting"].as_bool().unwrap(),
            packet_type: context["packet_type"].as_i64().unwrap() as i8,
            bonus: context["bonus"].as_i64().unwrap() as i8,
            bonus_per_attack: context["bonus_per_attack"].as_i64().unwrap() as i8,
            attack_count: context["attack_count"].as_i64().unwrap() as i16,
            mind_shield: context["mind_shield"].as_bool().unwrap(),
        };
        let extra = bytes(&case["extra"]);
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let mask = select_party(
            &warriors,
            case["selected"].as_u64().unwrap() as usize,
            &extra,
            context,
            &mut random,
        )
        .unwrap();
        assert_eq!(
            mask.as_slice(),
            words(&case["mask"]),
            "{extra:?} {context:?}"
        );
        assert_bytes(
            &flatten(&warriors),
            &bytes(&case["after"]),
            "selection warriors",
        );
        assert_eq!(u64::from(random.seed), case["seed_after"].as_u64().unwrap());
        assert_eq!(random.draws, case["draws"].as_u64().unwrap());
    }
}

#[test]
fn late_group_filter_matches_original_tiles_formation_order_and_complete_warriors() {
    let fixture = fixture();
    for case in fixture["group_cases"].as_array().unwrap() {
        let map = TacticalMap {
            tiles: words(&case["tiles_before"]),
            edge_word: 0,
            shift: (
                case["shift"][0].as_i64().unwrap() as i16,
                case["shift"][1].as_i64().unwrap() as i16,
            ),
        };
        let squad = SquadPlacement {
            positions: positions(&case["squad_before"]),
            under_tiles: words(&case["under_before"]),
        };
        let mask = std::array::from_fn(|index| case["mask"][index].as_i64().unwrap() as i16);
        let result = apply_group_mask(
            &warriors(&case["warriors_before"]),
            &map,
            &squad,
            &words(&case["order_before"]),
            &mask,
            case["group"].as_bool().unwrap(),
        )
        .unwrap();
        assert_bytes(
            &flatten(&result.warriors),
            &bytes(&case["warriors_after"]),
            "group warriors",
        );
        assert_eq!(
            result.map.tiles,
            words(&case["tiles_after"]),
            "group {} variant {}",
            case["group"],
            case["variant"]
        );
        assert_eq!(result.squad.positions, positions(&case["squad_after"]));
        assert_eq!(result.squad.under_tiles, words(&case["under_after"]));
        assert_eq!(result.order, words(&case["order_after"]));
        assert_eq!(case["draws"], 0);
    }
}

#[test]
fn group_encounters_match_original_selection_and_post_combat_continuations() {
    let fixture = fixture();
    for case in fixture["return_cases"].as_array().unwrap() {
        let extra = bytes(&case["extra"]);
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let plan = group_encounter(&extra, &mut random).unwrap();
        assert_eq!(plan.battle, case["battle"].as_i64().unwrap() as i16);
        assert_eq!(plan.sound, word(&extra, 4).unwrap());
        assert_eq!(plan.message, word(&extra, 6).unwrap());
        assert_eq!(plan.failure_script, word(&extra, 8).unwrap());
        assert_eq!(u64::from(random.seed), case["seed_after"].as_u64().unwrap());
        assert_eq!(random.draws, case["draws"].as_u64().unwrap());
        let mut events = vec![];
        if plan.sound != 0 {
            events.push(serde_json::json!(["sound", plan.sound]));
        }
        if plan.message != 0 {
            events.push(serde_json::json!(["message", plan.message]));
        }
        events.push(serde_json::json!(["combat", 1]));
        let before = warriors(&case["before"]);
        if case["aborted"] == true {
            assert_eq!(case["continuation"], "abort");
            assert_bytes(
                &flatten(&before),
                &bytes(&case["after"]),
                "aborted group warriors",
            );
        } else {
            let retreated = case["retreated"].as_bool().unwrap();
            let result = group_return(&before, retreated, plan.failure_script).unwrap();
            assert_bytes(
                &flatten(&result.warriors),
                &bytes(&case["after"]),
                "returned group warriors",
            );
            events.push(serde_json::json!(["land", 8]));
            if retreated {
                assert_eq!(
                    result.continuation,
                    StoryContinuation::ReplaceScript(plan.failure_script)
                );
                assert_eq!(case["cursor"], -1);
                events.push(serde_json::json!(["script", plan.failure_script]));
            } else {
                assert_eq!(result.continuation, StoryContinuation::Continue);
                assert_eq!(case["cursor"], 3);
                events.push(serde_json::json!(["booty", 0]));
            }
            events.push(serde_json::json!(["update", 0]));
            assert_eq!(case["script_before"], case["script_after"]);
        }
        assert_eq!(case["events"], Value::Array(events));
    }
}

#[test]
fn unsupported_party_save_does_not_consume_randomness() {
    let fixture = fixture();
    let warriors = warriors(&fixture["selection_cases"][0]["before"]);
    let mut random = RandomState::new(12);
    let before = random.clone();
    let mut extra = [0; 10];
    extra[0..2].copy_from_slice(&4i16.to_be_bytes());
    extra[2..4].copy_from_slice(&6i16.to_be_bytes());
    assert!(
        select_party(
            &warriors,
            0,
            &extra,
            PartySaveContext::default(),
            &mut random
        )
        .is_err()
    );
    assert_eq!(random, before);
}
