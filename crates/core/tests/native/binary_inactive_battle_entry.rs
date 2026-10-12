use crate::prelude::*;
use crate::support::assert_bytes;

#[test]
fn inactive_battle_entry_matches_original_complete_records_formation_order_and_rng() {
    let content = crate::support::content();
    let fixture: serde_json::Value = crate::support::fixture("inactive-battle-entry.json");
    let cases = fixture.array("cases");

    for case in cases {
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
        state.warriors = crate::support::warriors(&case["warriors_before"]);
        if case["seed_before"] == 12 {
            state.health_context = Some(centurions_core::story_health::context(
                &state.warriors,
                0,
                None,
            ));
            crate::support::expect(
                state.health_context.as_ref().unwrap().unavailable,
                &case["unavailable"],
            );
        }
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
        combat.validate(&state.warriors).unwrap();
        let context = format!(
            "variant{} battle{} seed{}",
            case["variant"], case["battle"], case["seed_before"]
        );
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
        scalar combat.map.edge_word => &case["edge"], "binary_inactive_battle_entry";
        bytes crate::support::record_bytes(&state.warriors) => &case["warriors"], &format!("{context} warriors");
        bytes crate::support::record_bytes(&combat
                        .enemies.records) => &case["enemies"], &format!("{context} enemies");
        scalar combat.enemies.records.len() => &case["count"], "binary_inactive_battle_entry";
        }
        let positions = |p: &[(i16, i16)]| p.iter().flat_map(|(x, y)| [*x, *y]).collect::<Vec<_>>();
        crate::support::native_expect! {
        bytes encode(&positions(&combat.squad.positions)) => &case["squad"], &format!("{context} squad");
        words &(combat.squad.under_tiles) => &case["under"], "binary_inactive_battle_entry";
        bytes encode(&positions(&combat.enemies.positions)) => &case["positions"], &format!("{context} enemy positions");
        words &(combat.order) => &case["order"], "binary_inactive_battle_entry";
        scalar combat.cursor => &case["cursor"], "binary_inactive_battle_entry";
        }
        assert_eq!(
            (state.random.seed, state.random.draws),
            (case.n::<u32>("seed_after"), case.n::<u64>("draws")),
            "{context} RNG"
        );
    }
}
#[test]
fn ordinary_battle_return_restores_all_participation_after_reward_boundary() {
    let fixture: serde_json::Value = crate::support::fixture("inactive-battle-return.json");
    let cases = fixture.array("cases");

    for case in cases.iter().filter(|case| case["aborted"] == false) {
        let before: Vec<_> = crate::support::warriors(&case["before"]);
        let returned = centurions_core::story_party::group_return(&before, false, 0).unwrap();
        let actual: Vec<_> = crate::support::record_bytes(&returned.warriors);
        assert_bytes(&actual, &case.bytes("after"), "ordinary return warriors");
    }
}
