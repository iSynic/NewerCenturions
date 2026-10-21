use crate::{prelude::*, support};

use centurions_core::travel::*;

#[test]
fn scripted_map_mutations_match_original_complete_map_layers() {
    for case in support::fixture("map-actions.json")["cases"]
        .as_array()
        .unwrap()
    {
        let extra = case.bytes("extra");
        let target = i16::from_be_bytes(extra[..2].try_into().unwrap()) as usize;
        for map in 0..2 {
            let before = &case[format!("map{map}_before")];
            let after = &case[format!("map{map}_after")];
            let mut floor = words(before.text("floor"));
            let mut objects = words(before.text("objects"));
            let mut headers = before.bytes("headers");
            if map == target {
                match case.n::<u64>("opcode") {
                    12 => {
                        let mutation = tile_mutation(&extra).unwrap();
                        match mutation.layer {
                            TileLayer::Floor => floor = apply_tile(&floor, &mutation).unwrap(),
                            TileLayer::Object => objects = apply_tile(&objects, &mutation).unwrap(),
                        }
                    }
                    13 => headers = set_event_chances(&headers, &extra).unwrap(),
                    7 => {
                        let replacement = event_replacement(&extra).unwrap();
                        headers = replace_event_script(
                            &headers,
                            replacement.slot,
                            &case.bytes("replacement"),
                        )
                        .unwrap();
                    }
                    opcode => panic!("unexpected fixture opcode {opcode}"),
                }
            }
            support::native_expect! {
                words &(floor) => &after["floor"], "binary_travel";
                words &(objects) => &after["objects"], "binary_travel";
                bytes headers => &after["headers"], "binary_travel";
            }
        }
    }
    // Opcode8 reads the mutable map header; opcode39 instead loads Data ED3.
    let captured = support::fixture("header-dispatch.json");
    let content = content();
    let mut state = support::mission_session(&content);
    let position = state.exploration.as_mut().unwrap();
    position.x = 30;
    position.y = 78;
    position.floor.fill(155);
    position.region_state.fill(0);
    position.presentation = None;
    position.event_headers = captured.bytes("headers");
    let mut restored = support::checkpoint(&state, &content, "header-alias");
    support::replay_command(
        Command::Move { dx: 0, dy: 1 },
        &mut state,
        &mut restored,
        &content,
    );
    let position = state.exploration.as_ref().unwrap();
    assert!(matches!(
        position.presentation,
        Some(Presentation::Dialogue { id: 124, .. })
    ));
    assert_eq!(
        position.script.as_ref().unwrap().record,
        captured.bytes("after")
    );
}

#[test]
fn normal_secret_search_matches_original_full_map_and_rng() {
    let content = content();
    for case in support::fixture("map-actions.json")["secret_cases"]
        .as_array()
        .unwrap()
    {
        let floor = words(case.text("before"));
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result = discover_secrets(
            &floor,
            (case.n::<i16>("x"), case.n::<i16>("y")),
            case.n::<i16>("chance"),
            &mut random,
        )
        .unwrap();
        if case["forced_skill"].as_bool() == Some(false) {
            let mut state = support::mission_session(&content);
            let records = case.bytes("warriors");
            state.warriors.truncate(records.len() / 3138);
            for (warrior, record) in state.warriors.iter_mut().zip(records.as_chunks::<3138>().0) {
                for offset in [0xfc, 0xb6a] {
                    warrior.bytes[offset..offset + 2].copy_from_slice(&record[offset..offset + 2]);
                }
            }
            assert_eq!(
                secret_search_skill(&state.warriors).unwrap(),
                case.n::<i16>("chance")
            );
            let position = state.exploration.as_mut().unwrap();
            position.floor = floor.clone();
            position.event_headers.fill(0);
            position.x = case.n("x");
            position.y = case.n("y");
            position.presentation = Some(Presentation::Messages {
                pages: vec![(0, content.map(0).unwrap().dialogue(0).unwrap())],
                next: None,
            });
            state.movement_continuation = Some(centurions_core::MovementContinuation::Secrets);
            state.random = RandomState::new(case.n("seed_before"));
            let events = apply(Command::Continue, &mut state, &content).unwrap();
            assert_eq!(state.exploration.as_ref().unwrap().floor, result.floor);
            assert_eq!(state.random, random);
            assert_eq!(
                events
                    .iter()
                    .filter(|event| matches!(
                        event,
                        Event::Feedback(centurions_core::feedback::NativeFeedback::ResourceFlash {
                            resource_id: 138,
                            sound: 5000,
                            ..
                        })
                    ))
                    .count(),
                result.discoveries.len()
            );
        }
        support::native_expect! {
            words &(result.floor) => &case["after"], "binary_travel";
            rng random => (&case["seed_after"], &case["rng_draws"]), "binary_travel";
            scalar result.center_revealed => &case["center_revealed"], "binary_travel";
            scalar result.discoveries.len() => &case["flashes"], "binary_travel";
        }
    }
}
