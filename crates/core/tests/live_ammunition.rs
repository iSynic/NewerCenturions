use centurions_content::{Content, InventoryRecord, Warrior};
use centurions_core::{Command, Presentation, RandomState, Session, apply};

fn decode(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn authored_pack_collection_matches_live_records_reserve_and_rng() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/live-ammunition.json"
    ))
    .unwrap();
    let case = &fixture["cases"][0];
    let before = &case["before"];
    let after = &case["after"];
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
    state.warriors = before["warriors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| Warrior::read(&decode(w["hex"].as_str().unwrap())).unwrap())
        .collect();
    state.random = RandomState::new(before["random_seed"].as_u64().unwrap() as u32);
    for (reserve, raw) in state.ammunition.iter_mut().zip(
        decode(before["ammunition"]["hex"].as_str().unwrap())
            .as_chunks::<4>()
            .0,
    ) {
        *reserve = i32::from_be_bytes(*raw);
    }
    for (item, raw) in state.booty_items.iter_mut().zip(
        decode(before["loot"]["hex"].as_str().unwrap())
            .as_chunks::<22>()
            .0,
    ) {
        *item = InventoryRecord(*raw);
    }
    state.exploration.as_mut().unwrap().presentation = Some(Presentation::Loot);
    state.exploration.as_mut().unwrap().script = Some(centurions_core::ScriptState {
        record: vec![0; 42],
        cursor: 0,
        map_slot: 0,
        scheduled: false,
        menu: None,
        persist: false,
        origin: None,
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
    assert_eq!(
        state
            .ammunition
            .iter()
            .flat_map(|v| v.to_be_bytes())
            .collect::<Vec<_>>(),
        decode(after["ammunition"]["hex"].as_str().unwrap())
    );
    assert_eq!(
        state
            .booty_items
            .iter()
            .take(20)
            .flat_map(|item| item.0)
            .collect::<Vec<_>>(),
        decode(after["loot"]["hex"].as_str().unwrap())
    );
    assert_eq!(
        state.random.seed,
        after["random_seed"].as_u64().unwrap() as u32
    );
    assert_eq!(state.random.draws, 1);
}
