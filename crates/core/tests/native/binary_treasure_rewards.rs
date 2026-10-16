use crate::prelude::*;

use centurions_core::story_treasure;

#[test]
fn original_treasure_eight_preserves_participation_and_awards_experience() {
    let fixture: Value = crate::support::fixture("treasure-rewards.json");

    let content = crate::support::content();
    let cases = fixture.array("cases");

    for case in cases {
        let before = case.bytes("before");
        let warriors: Vec<_> = crate::support::warriors(&before);
        let mut random = RandomState {
            seed: case.n::<u32>("seed_before"),
            draws: 0,
        };
        let result = story_treasure::prepare(
            &warriors,
            &case.bytes("table_before"),
            &content,
            &mut random,
            0,
        )
        .unwrap();
        assert_eq!(
            crate::support::record_bytes(&result.warriors),
            case.bytes("after")
        );
        crate::support::expect(result.share, &case["share"]);
        crate::support::expect(result.credits, &case["currency"]);
        let mut crate_bytes = vec![0; 22000];
        for (slot, item) in result.items.iter().enumerate() {
            crate_bytes[slot * 22..(slot + 1) * 22].copy_from_slice(&item.0);
        }
        assert_eq!(crate_bytes, case.bytes("crate_after"));

        crate::support::native_expect! {
        scalar random.seed => &case["seed_after"], "binary_treasure_rewards";
        scalar random.draws => &case["rng_draws"], "binary_treasure_rewards";
        }
    }
}

#[test]
fn authored_credit_chest_awards_once_across_saved_script_return() {
    let content = content();
    let mut state = crate::support::mission_session(&content);
    let position = state.exploration.as_mut().unwrap();
    position.restore_map(
        5,
        centurions_core::MapState::authored(content.map(5).unwrap()),
    );
    (position.x, position.y) = (84, 64);
    apply(Command::Move { dx: 1, dy: 0 }, &mut state, &content).unwrap();
    assert!(matches!(
        state.exploration.as_ref().unwrap().presentation,
        Some(Presentation::Dialogue { id: 494, .. })
    ));
    let currency = state.party_currency;
    let mut resumed = crate::support::checkpoint(&state, &content, "credit-chest-before");
    let events = apply(Command::Continue, &mut state, &content).unwrap();
    assert_eq!(
        apply(Command::Continue, &mut resumed, &content).unwrap(),
        events
    );
    assert_eq!(state, resumed);
    assert_eq!(state.party_currency, currency.wrapping_add(500));
    let mut resumed = crate::support::checkpoint(&state, &content, "credit-chest-after");
    for candidate in [&mut state, &mut resumed] {
        for slot in 0..candidate.booty_items.len() {
            if candidate.booty_items[slot].id() != 0 {
                apply(Command::StoreLoot { slot }, candidate, &content).unwrap();
            }
        }
    }
    assert_eq!(
        apply(Command::Continue, &mut state, &content).unwrap(),
        apply(Command::Continue, &mut resumed, &content).unwrap()
    );
    assert_eq!(state, resumed);
    assert_eq!(state.party_currency, currency.wrapping_add(500));
}
