use crate::prelude::*;

// Protects selector ownership, complete stat writes, and source-width arithmetic.
#[test]
fn editor_upgrade_and_statistics_match_complete_original_records() {
    let fixture = crate::support::fixture("editor-upgrades.json");
    for case in fixture.array("cases") {
        let mut warrior = Warrior::read(&case.bytes("before")).unwrap();
        centurions_core::character::apply_upgrade(&mut warrior, case.n("selector"), 1).unwrap();
        assert_eq!(warrior.bytes, case.bytes("applied"));
        centurions_core::character::calculate_statistics(&mut warrior).unwrap();
        assert_eq!(warrior.bytes, case.bytes("after"));
    }
}

// Protects per-iteration SINGLE rounding, MOS ownership and the original cost cap.
#[test]
fn upgrade_costs_match_original_numeric_dialog_order() {
    for case in crate::support::fixture("upgrade-costs.json").array("cases") {
        let warrior = Warrior::read(&case.bytes("warrior")).unwrap();
        let (cost, percent) = centurions_core::character::upgrade_cost(
            &warrior,
            case.n("selector"),
            &case.bytes("profile"),
        )
        .unwrap();
        assert_eq!(
            serde_json::json!([{ "item": 44, "value": cost }, { "item": 46, "value": percent }]),
            case["numbers"]
        );
    }
}

// Protects editor promotion without experience/scan changes and its separate C rand stream.
#[test]
fn editor_levels_match_original_records_sounds_and_library_random() {
    let content = crate::support::content();
    for case in crate::support::fixture("editor-levels.json").array("cases") {
        let mut state = crate::support::mission_session(&content);
        state.warriors[0] = Warrior::read(&case.bytes("before")).unwrap();
        state.library_random_seed = case.n("seed_before");
        let events = apply(
            Command::EditWarriorLevel { warrior: 0 },
            &mut state,
            &content,
        )
        .unwrap();
        crate::support::assert_bytes(
            &state.warriors[0].bytes,
            &case.bytes("after"),
            case["case"].as_str().unwrap(),
        );
        assert_eq!(
            (state.library_random_seed, state.library_random_draws),
            (case.n("seed_after"), case.n("draws"))
        );
        assert_eq!(
            events
                .iter()
                .filter_map(|e| if let Event::Sound { resource_id } = e {
                    Some(*resource_id)
                } else {
                    None
                })
                .collect::<Vec<_>>(),
            serde_json::from_value::<Vec<i16>>(case["sounds"].clone()).unwrap()
        );
    }
}

// Protects native affordability, selection counters, shortcut cleanup and atomic failure.
#[test]
fn editor_scan_choices_match_original_records_and_ordered_sounds() {
    let content = crate::support::content();
    let fixture = crate::support::fixture("editor-scans.json");
    for case in fixture.array("cases") {
        let mut state = crate::support::mission_session(&content);
        state.warriors[0] = Warrior::read(&case.bytes("before")).unwrap();
        let packet = &fixture["attack_packets"][case["case"].as_str().unwrap()];
        state.attack_packet = Some(packet.bytes("before"));
        state.psi_memory = Some(packet.bytes("memory_before"));
        let before = state.clone();
        let controls = serde_json::from_value::<Vec<u16>>(case["controls"].clone()).unwrap();
        let events = apply(
            Command::EditWarriorScans {
                warrior: 0,
                controls,
            },
            &mut state,
            &content,
        )
        .unwrap();
        crate::support::assert_bytes(
            &state.warriors[0].bytes,
            &case.bytes("after"),
            case["case"].as_str().unwrap(),
        );
        assert_eq!(state.attack_packet, Some(packet.bytes("after")));
        assert_eq!(state.psi_memory, Some(packet.bytes("memory_after")));
        assert_eq!(state.random, before.random);
        assert_eq!(
            (state.library_random_seed, state.library_random_draws),
            (before.library_random_seed, before.library_random_draws)
        );
        assert_eq!(
            events
                .into_iter()
                .filter_map(|e| if let Event::Sound { resource_id } = e {
                    Some(resource_id)
                } else {
                    None
                })
                .collect::<Vec<_>>(),
            serde_json::from_value::<Vec<i16>>(case["sounds"].clone()).unwrap()
        );
    }
    let mut state = crate::support::mission_session(&content);
    state.warriors[0] = Warrior::read(&fixture.array("cases")[0].bytes("before")).unwrap();
    let before = state.clone();
    assert!(
        apply(
            Command::EditWarriorScans {
                warrior: 0,
                controls: vec![2, 10, 99, 1]
            },
            &mut state,
            &content
        )
        .is_err()
    );
    assert_eq!(state, before);
}
