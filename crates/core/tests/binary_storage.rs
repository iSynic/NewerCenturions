use centurions_content::{Content, InventoryRecord, digest};
use centurions_core::{Command, Event, Session, apply};
use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Case {
    #[serde(rename = "crate")]
    crate_storage: bool,
    hole: i32,
    result: i16,
    sound: i16,
    sha256: String,
}

#[test]
fn storage_matches_independent_68k_execution() {
    let fixture: Fixture =
        serde_json::from_str(include_str!("../../../evidence/fixtures/storage.json")).unwrap();
    let content = Content {
        identity: "fixture".into(),
        warriors: vec![],
        items: vec![],
        resources: vec![],
        original_services: None,
        scenario: None,
    };
    let mut item = InventoryRecord([0; 22]);
    for (index, byte) in item.0.iter_mut().enumerate().skip(2) {
        *byte = index as u8;
    }
    item.0[1] = 7;
    for case in fixture.cases {
        let mut state = Session::new(&content);
        let mut occupied = item.clone();
        occupied.0[1] = 9;
        let records = if case.crate_storage {
            &mut state.crate_items
        } else {
            &mut state.booty_items
        };
        records.fill(occupied);
        if case.hole >= 0 {
            records[case.hole as usize] = InventoryRecord([0; 22]);
        }
        let command = if case.crate_storage {
            Command::AddToCrate { item: item.clone() }
        } else {
            Command::AddToBooty { item: item.clone() }
        };
        let events = apply(command, &mut state, &content).unwrap();
        assert_eq!(
            i16::from(events.iter().any(|e| matches!(e, Event::ItemAdded { .. }))),
            case.result
        );
        let sound = events
            .iter()
            .find_map(|event| {
                if let Event::Sound { resource_id } = event {
                    Some(*resource_id)
                } else {
                    None
                }
            })
            .unwrap_or(0);
        assert_eq!(sound, case.sound);
        let records = if case.crate_storage {
            &state.crate_items
        } else {
            &state.booty_items
        };
        let bytes: Vec<u8> = records.iter().flat_map(|record| record.0).collect();
        assert_eq!(digest(&bytes), case.sha256);
        assert_eq!(state.random.draws, 0);
    }
}
