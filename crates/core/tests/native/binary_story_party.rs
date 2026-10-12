use crate::prelude::*;

use centurions_core::story_party::PartySaveContext;
use centurions_core::story_party::apply_group_mask;
use centurions_core::story_party::group_encounter;
use centurions_core::story_party::group_return;
use centurions_core::story_party::select_party;
use centurions_core::story_variables::StoryContinuation;

fn fixture() -> Value {
    crate::support::fixture("story-party.json")
}
use crate::support::warriors;

fn positions(value: &Value) -> Vec<(i16, i16)> {
    words(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|record| (record[0], record[1]))
        .collect()
}
use crate::support::assert_bytes;

use crate::support::record_bytes as flatten;

#[test]
fn party_selection_matches_original_masks_complete_records_and_randomness() {
    let fixture = fixture();
    for case in fixture.array("selection_cases") {
        let warriors = warriors(&case["before"]);
        let context = &case["context"];
        let context = PartySaveContext {
            casting: context.n::<bool>("casting"),
            packet_type: context.n::<i8>("packet_type"),
            bonus: context.n::<i8>("bonus"),
            bonus_per_attack: context.n::<i8>("bonus_per_attack"),
            attack_count: context.n::<i16>("attack_count"),
            mind_shield: context.n::<bool>("mind_shield"),
        };
        let extra = case.bytes("extra");
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let mask = select_party(
            &warriors,
            case.n::<usize>("selected"),
            &extra,
            context,
            &mut random,
        )
        .unwrap();
        crate::support::native_expect! {
        words mask.as_slice() => &case["mask"], "{extra:?} {context:?}";
        bytes flatten(&warriors) => &case["after"], "selection warriors";
        scalar u64::from(random.seed) => &case["seed_after"], "binary_story_party";
        scalar random.draws => &case["draws"], "binary_story_party";
        }
    }
}

#[test]
fn late_group_filter_matches_original_tiles_formation_order_and_complete_warriors() {
    let fixture = fixture();
    for case in fixture.array("group_cases") {
        let map = TacticalMap {
            tiles: case.words("tiles_before"),
            edge_word: 0,
            shift: (
                case["shift"][0].as_i64().unwrap() as i16,
                case["shift"][1].as_i64().unwrap() as i16,
            ),
        };
        let squad = SquadPlacement {
            positions: positions(&case["squad_before"]),
            under_tiles: case.words("under_before"),
        };
        let mask = std::array::from_fn(|index| case["mask"][index].as_i64().unwrap() as i16);
        let result = apply_group_mask(
            &warriors(&case["warriors_before"]),
            &map,
            &squad,
            &case.words("order_before"),
            &mask,
            case.n::<bool>("group"),
        )
        .unwrap();
        crate::support::native_expect! {
        bytes flatten(&result.warriors) => &case["warriors_after"], "group warriors";
        words result.map.tiles => &case["tiles_after"], format!("group {} variant {}",case["group"],case["variant"]);
        }
        assert_eq!(result.squad.positions, positions(&case["squad_after"]));
        crate::support::native_expect! {
        words result.squad.under_tiles => &case["under_after"], "binary_story_party";
        words result.order => &case["order_after"], "binary_story_party";
        }
    }
}

#[test]
fn group_encounters_match_original_selection_and_post_combat_continuations() {
    let fixture = fixture();
    for case in fixture.array("return_cases") {
        let extra = case.bytes("extra");
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let plan = group_encounter(&extra, &mut random).unwrap();
        crate::support::expect(plan.battle, &case["battle"]);
        assert_eq!(plan.sound, word(&extra, 4).unwrap());
        assert_eq!(plan.message, word(&extra, 6).unwrap());
        assert_eq!(plan.failure_script, word(&extra, 8).unwrap());
        crate::support::native_expect! {
        scalar u64::from(random.seed) => &case["seed_after"], "binary_story_party";
        scalar random.draws => &case["draws"], "binary_story_party";
        }
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
            assert_bytes(
                &flatten(&before),
                &case.bytes("after"),
                "aborted group warriors",
            );
        } else {
            let retreated = case.n::<bool>("retreated");
            let result = group_return(&before, retreated, plan.failure_script).unwrap();
            assert_bytes(
                &flatten(&result.warriors),
                &case.bytes("after"),
                "returned group warriors",
            );
            events.push(serde_json::json!(["land", 8]));
            if retreated {
                assert_eq!(
                    result.continuation,
                    StoryContinuation::ReplaceScript(plan.failure_script)
                );

                events.push(serde_json::json!(["script", plan.failure_script]));
            } else {
                assert_eq!(result.continuation, StoryContinuation::Continue);

                events.push(serde_json::json!(["booty", 0]));
            }
            events.push(serde_json::json!(["update", 0]));
        }
        assert_eq!(case["events"], Value::Array(events));
    }
}
