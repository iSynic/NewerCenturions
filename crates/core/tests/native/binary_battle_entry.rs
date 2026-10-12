use crate::prelude::*;
use crate::support::assert_bytes;
#[test]
fn underground_battle_entry_matches_original_complete_records_and_random_consumption() {
    let fixture: serde_json::Value = crate::support::fixture("battle-entry.json");
    crate::binary_battle_entry::compare_underground_battle_entry_matches_original_complete_records_and_random_consumption(fixture);
}

pub(super) fn compare_underground_battle_entry_matches_original_complete_records_and_random_consumption(
    fixture: Value,
) {
    let content = crate::support::content();

    for case in fixture.array("cases") {
        let mut state = Session::new(&content);
        apply(Command::ImportStarters, &mut state, &content).unwrap();
        apply(
            Command::EnterMission {
                name: "The Thunder of Guns".into(),
            },
            &mut state,
            &content,
        )
        .unwrap();
        state.warriors = crate::support::warriors(&fixture["warriors_before"]);
        let map = content.map(case.n::<i32>("map")).unwrap();
        let position = state.exploration.as_mut().unwrap();
        position.map = case.n::<i32>("map");
        position.x = case.n::<i32>("x");
        position.y = case.n::<i32>("y");
        position.floor.clone_from(&map.floor);
        position.events.clone_from(&map.events);
        state.random = RandomState::new(case.n::<u32>("seed_before"));
        begin_battle(&mut state, &content, case.n::<i16>("battle"), 0).unwrap();
        let combat = state.combat.as_ref().unwrap();
        let context = format!("battle{} seed{}", case["battle"], case["seed_before"]);
        let encode = |v: &[i16]| crate::support::word_bytes(v);
        assert_bytes(
            &encode(&combat.map.tiles),
            &case.bytes("tiles"),
            &format!("{context} map"),
        );
        assert_eq!(
            combat.map.shift,
            (
                case["shift"][0].as_i64().unwrap() as i16,
                case["shift"][1].as_i64().unwrap() as i16
            ),
            "{context} shift"
        );
        crate::support::native_expect! {
        scalar combat.map.edge_word => &case["edge"], "binary_battle_entry";
        bytes crate::support::record_bytes(&state.warriors) => &case["warriors"], &format!("{context} warriors");
        bytes crate::support::record_bytes(&combat
                        .enemies.records) => &case["enemies"], &format!("{context} enemies");
        scalar combat.enemies.records.len() => &case["count"], "binary_battle_entry";
        }
        let positions = |p: &[(i16, i16)]| p.iter().flat_map(|(x, y)| [*x, *y]).collect::<Vec<_>>();
        crate::support::native_expect! {
        bytes encode(&positions(&combat.squad.positions)) => &case["squad"], &format!("{context} squad");
        words &(combat.squad.under_tiles) => &case["under"], "binary_battle_entry";
        bytes encode(&positions(&combat.enemies.positions)) => &case["positions"], &format!("{context} enemy positions");
        words &(combat.order) => &case["order"], "binary_battle_entry";
        scalar combat.cursor => &case["cursor"], "binary_battle_entry";
        }
        assert_eq!(
            (state.random.seed, state.random.draws),
            (case.n::<u32>("seed_after"), case.n::<u64>("draws")),
            "{context} RNG"
        );
    }
}
