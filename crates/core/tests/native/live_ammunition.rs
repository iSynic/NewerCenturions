use crate::{prelude::*, support};

#[test]
fn authored_pack_collection_matches_live_records_reserve_and_rng() {
    let content = support::content();
    let fixture: serde_json::Value = support::fixture("live-ammunition.json");
    let case = &fixture["cases"][0];
    let before = &case["before"];
    let after = &case["after"];
    let mut state = support::mission_session(&content);
    state.warriors = before
        .array("warriors")
        .iter()
        .map(|w| Warrior::read(&w.bytes("hex")).unwrap())
        .collect();
    state.random = RandomState::new(before.n::<u32>("random_seed"));
    for (reserve, raw) in state
        .ammunition
        .iter_mut()
        .zip(before["ammunition"].bytes("hex").as_chunks::<4>().0)
    {
        *reserve = i32::from_be_bytes(*raw);
    }
    for (item, raw) in state
        .booty_items
        .iter_mut()
        .zip(before["loot"].bytes("hex").as_chunks::<22>().0)
    {
        *item = InventoryRecord(*raw);
    }
    state.exploration.as_mut().unwrap().presentation = Some(Presentation::Loot);
    state.exploration.as_mut().unwrap().script = Some(centurions_core::ScriptState {
        scheduled: false,
        ..centurions_core::ScriptState::scheduled(vec![0; 42], 0)
    });
    apply(
        Command::TakeLoot {
            slot: 4,
            warrior: 0,
        },
        &mut state,
        &content,
    )
    .unwrap();
    for (index, w) in state.warriors.iter().enumerate() {
        assert_eq!(
            w.bytes,
            decode(after["warriors"][index]["hex"].as_str().unwrap()),
            "warrior {index}"
        );
    }
    support::native_expect! {
        bytes support::long_bytes(&state.ammunition) => &after["ammunition"]["hex"], "live_ammunition";
        bytes state
                .booty_items
                .iter()
                .take(20)
                .flat_map(|item| item.0)
                .collect::<Vec<_>>() => &after["loot"]["hex"], "live_ammunition";
        scalar state.random.seed => &after["random_seed"], "live_ammunition";
    }
    assert_eq!(state.random.draws, 1);
}
