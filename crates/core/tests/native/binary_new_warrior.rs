use crate::prelude::*;

// Protects complete creation records, original stat rounding and both random-stream boundaries.
#[test]
fn creation_rolls_match_native_records_and_random_consumption() {
    let content = crate::support::content();
    for case in crate::support::fixture("new-warrior.json").array("cases") {
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
        crate::support::assert_bytes(
            &state.warriors[0].bytes,
            &case.bytes("after"),
            case.text("case"),
        );
        assert_eq!(
            (state.random.seed, state.random.draws),
            (case.n("seed_after"), case.n("draws"))
        );
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
        crate::support::assert_bytes(
            &state.warriors[0].bytes,
            &case.bytes("after_initial_items"),
            case.text("case"),
        );
        assert_eq!(
            (state.random.seed, state.random.draws),
            (case.n("items_seed_after"), case.n("items_draws"))
        );
    }
}
