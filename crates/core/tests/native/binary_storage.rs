use crate::prelude::*;

#[test]
fn storage_matches_independent_68k_execution() {
    let fixture = crate::support::fixture("storage.json");
    let content = crate::support::fixture_content("fixture");
    let mut item = InventoryRecord([0; 22]);
    for (index, byte) in item.0.iter_mut().enumerate().skip(2) {
        *byte = index as u8;
    }
    item.0[1] = 7;
    for case in fixture.array("cases") {
        let mut state = Session::new(&content);
        let mut occupied = item.clone();
        occupied.0[1] = 9;
        let records = if case.n::<bool>("crate") {
            &mut state.crate_items
        } else {
            &mut state.booty_items
        };
        records.fill(occupied);
        if case.n::<i32>("hole") >= 0 {
            records[case.n::<usize>("hole")] = InventoryRecord([0; 22]);
        }
        let command = if case.n::<bool>("crate") {
            Command::AddToCrate { item: item.clone() }
        } else {
            Command::AddToBooty { item: item.clone() }
        };
        let events = apply(command, &mut state, &content).unwrap();
        assert_eq!(
            i16::from(events.iter().any(|e| matches!(e, Event::ItemAdded { .. }))),
            case.n::<i16>("result")
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
        assert_eq!(sound, case.n::<i16>("sound"));
        let records = if case.n::<bool>("crate") {
            &state.crate_items
        } else {
            &state.booty_items
        };
        let bytes: Vec<u8> = records.iter().flat_map(|record| record.0).collect();
        assert_eq!(digest(&bytes), case.text("sha256"));
        assert_eq!(state.random.draws, 0);
    }
}
