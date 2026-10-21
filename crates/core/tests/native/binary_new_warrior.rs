use crate::{prelude::*, support};

// Protects complete creation records, original stat rounding and both random-stream boundaries.
#[test]
fn creation_rolls_match_native_records_and_random_consumption() {
    let content = support::content();
    for case in support::fixture("new-warrior.json").array("cases") {
        let mut state = Session::new(&content);
        state.random = RandomState::new(case.n("seed_before"));
        apply(
            Command::GenerateWarrior {
                request: centurions_core::creation::Request {
                    race: 1,
                    mos: 1,
                    gender: case.n("gender"),
                    level: 1,
                    name: "Reference".into(),
                    portrait: 257,
                    icon: 9257,
                },
            },
            &mut state,
            &content,
        )
        .unwrap();
        support::native_expect! {
            bytes &state.warriors[0].bytes => &case.bytes("after"), case.text("case");
        };
        support::assert_rng(&state.random, &case["seed_after"], &case["draws"]);
        assert_eq!(state.library_random_draws, 0);
        let events = apply(Command::CreationEquipment, &mut state, &content).unwrap();
        assert_eq!(
            events,
            case.array("item_sounds")
                .iter()
                .map(|v| Event::Sound {
                    resource_id: v.as_i64().unwrap() as i16
                })
                .collect::<Vec<_>>()
        );
        support::native_expect! {
            bytes &state.warriors[0].bytes => &case.bytes("after_initial_items"), case.text("case");
        };
        support::assert_rng(
            &state.random,
            &case["items_seed_after"],
            &case["items_draws"],
        );
    }
}
