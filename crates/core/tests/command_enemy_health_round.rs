use centurions_content::{Content, word};
use centurions_core::{Command, Event, Session, apply, load, save};
use serde_json::Value;

fn bytes(value: &Value) -> Vec<u8> {
    value
        .as_str()
        .unwrap()
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
fn words(value: &Value) -> Vec<i16> {
    bytes(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| i16::from_be_bytes(*pair))
        .collect()
}
fn assert_bytes(actual: &[u8], expected: &[u8], name: &str) {
    assert_eq!(actual.len(), expected.len(), "{name} length");
    if let Some(offset) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "{name} byte {offset:#x}: {} != {}",
            actual[offset], expected[offset]
        );
    }
}
fn fixture() -> (Content, Session, Value) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/enemy-health-round.json"
    ))
    .unwrap();
    let session = serde_json::from_value(fixture["foundation"].clone()).unwrap();
    (content, session, fixture)
}

#[test]
fn actual_end_turn_matches_complete_original_round_and_resumes_after_reload() {
    let (content, mut state, fixture) = fixture();
    let before = state.clone();
    let case = &fixture["cases"][0];
    let expected = &case["after"];
    let checkpoint = std::env::temp_dir().join(format!(
        "centurions-enemy-health-round-{}.json",
        std::process::id()
    ));
    save(&checkpoint, &state, &content).unwrap();
    let mut resumed = load(&checkpoint, &content).unwrap();
    let events = apply(Command::EndTurn, &mut state, &content).unwrap();
    assert_eq!(
        apply(Command::EndTurn, &mut resumed, &content).unwrap(),
        events
    );
    assert_eq!(state, resumed);
    let combat = state.combat.as_ref().unwrap();
    assert_bytes(
        &state
            .warriors
            .iter()
            .flat_map(|w| w.bytes.iter().copied())
            .collect::<Vec<_>>(),
        &bytes(&expected["party"]),
        "party",
    );
    assert_bytes(
        &combat
            .enemies
            .records
            .iter()
            .flat_map(|w| w.bytes.iter().copied())
            .collect::<Vec<_>>(),
        &bytes(&expected["records"]),
        "monsters",
    );
    assert_eq!(combat.map.tiles, words(&expected["map"]));
    assert_eq!(combat.order, words(&expected["order"]));
    assert_eq!(
        combat.enemies.positions,
        words(&expected["positions"])
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| (p[0], p[1]))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        combat.squad.positions,
        words(&expected["party_positions"])
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| (p[0], p[1]))
            .collect::<Vec<_>>()
    );
    assert_eq!(combat.spell_fields, bytes(&expected["fields"]));
    assert_eq!(combat.field_used, bytes(&expected["used"]));
    assert_eq!(state.party_buffs, bytes(&expected["party_buffs"]));
    assert_eq!(combat.cursor, expected["cursor"].as_u64().unwrap() as usize);
    assert_eq!(combat.round, expected["round"].as_i64().unwrap() as i16);
    assert_eq!(combat.active_body(), Some(5));
    assert_eq!(
        combat.death_queue,
        before.combat.as_ref().unwrap().death_queue
    );
    assert_eq!(
        state.attack_count,
        expected["count"].as_i64().unwrap() as i16
    );
    assert_eq!(
        combat.unavailable,
        expected["unavailable"].as_i64().unwrap() as i8
    );
    assert_eq!(state.random, before.random);
    let mut retained_packet = before.attack_packet.clone();
    retained_packet.as_mut().unwrap()[19] = 0;
    assert_eq!(state.attack_packet, retained_packet);
    assert_eq!(
        state.attack_packet.as_ref().unwrap(),
        &bytes(&expected["packet"])
    );
    assert_eq!(
        state.attack_identity.unwrap().as_slice(),
        words(&expected["identity"])
    );
    assert_eq!(
        state.loaded_item_field32,
        Some(expected["item_context"].as_i64().unwrap() as i16)
    );
    assert_eq!(state.attack_identity, before.attack_identity);
    assert_eq!(state.psi_memory, before.psi_memory);
    let sounds: Vec<_> = events
        .iter()
        .filter_map(|e| {
            if let Event::Sound { resource_id } = e {
                Some(*resource_id)
            } else {
                None
            }
        })
        .collect();
    let mut expected_sounds: Vec<_> = case["sounds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap() as i16)
        .collect();
    // The older probe intercepted FlashMessage; its round cue is recovered at 123dcc.
    expected_sounds.insert(expected_sounds.len() - 1, -139);
    assert_eq!(sounds, expected_sounds);
    assert_eq!(word(&combat.enemies.records[1].bytes, 0x110).unwrap(), -1);
    assert_eq!(word(&state.warriors[4].bytes, 0x3c).unwrap(), 0);
    assert_eq!(
        word(&state.warriors[4].bytes, 0x40).unwrap(),
        word(&before.warriors[4].bytes, 0x40).unwrap()
    );
    let mut expected_session = before.clone();
    let effect = bytes(&expected["packet"])[18] as i8;
    expected_session.health_context.as_mut().unwrap().effect = effect;
    // The older fixture records Killbody arguments, while its state projection
    // omits WORD408574. Native Killbody(body, 0) stores that body before cleanup.
    let last_kill = case["kills"].as_array().unwrap().last().unwrap();
    assert_eq!(last_kill[1], 0);
    expected_session.health_context.as_mut().unwrap().last_body =
        last_kill[0].as_i64().unwrap() as i16;
    assert_eq!(state.health_context.as_ref().unwrap().effect, effect);
    assert_eq!(state.attack_packet.as_ref().unwrap()[18] as i8, effect);
    assert_eq!(combat.attack_packet.as_ref().unwrap()[18] as i8, effect);
    expected_session.bandage_ready = Some(1);
    expected_session.selected_warrior = 5;
    expected_session
        .health_context
        .as_mut()
        .unwrap()
        .selected_warrior = 5;
    expected_session.warriors = state.warriors.clone();
    expected_session.attack_packet = state.attack_packet.clone();
    let expected_combat = expected_session.combat.as_mut().unwrap();
    expected_combat.enemies.records = combat.enemies.records.clone();
    expected_combat.map.tiles = combat.map.tiles.clone();
    expected_combat.order = combat.order.clone();
    expected_combat.round = combat.round;
    expected_combat.cursor = combat.cursor;
    expected_combat.round_scripts.counter = expected_combat.round_scripts.counter.wrapping_add(1);
    expected_combat.spell_fields = combat.spell_fields.clone();
    expected_combat.field_used = combat.field_used.clone();
    expected_combat.attack_packet = combat.attack_packet.clone();
    let actual_json = serde_json::to_value(&state).unwrap();
    let expected_json = serde_json::to_value(&expected_session).unwrap();
    for (key, value) in expected_json["combat"].as_object().unwrap() {
        if actual_json["combat"][key] != *value {
            panic!(
                "unexpected combat change in {key}: {} != {}",
                actual_json["combat"][key], value
            );
        }
    }
    for (key, value) in serde_json::to_value(&expected_session)
        .unwrap()
        .as_object()
        .unwrap()
    {
        assert!(
            actual_json[key] == *value,
            "unexpected Session change in {key}"
        );
    }
    save(&checkpoint, &state, &content).unwrap();
    assert_eq!(load(&checkpoint, &content).unwrap(), state);
    std::fs::remove_file(checkpoint).unwrap();
}

#[test]
fn public_round_ticks_party_protection_and_rolls_back_an_unsupported_later_effect() {
    let (content, mut state, _) = fixture();
    let mut baseline = state.clone();
    state.party_buffs[4..6].copy_from_slice(&20_i16.to_be_bytes());
    let path =
        std::env::temp_dir().join(format!("centurions-round-buff-{}.json", std::process::id()));
    save(&path, &state, &content).unwrap();
    let mut resumed = load(&path, &content).unwrap();
    let events = apply(Command::EndTurn, &mut state, &content).unwrap();
    assert_eq!(
        apply(Command::EndTurn, &mut resumed, &content).unwrap(),
        events
    );
    assert_eq!(state, resumed);
    assert_eq!(
        apply(Command::EndTurn, &mut baseline, &content).unwrap(),
        events
    );
    assert_eq!(word(&state.party_buffs, 4).unwrap(), 19);
    baseline.party_buffs = state.party_buffs.clone();
    assert_eq!(state, baseline);
    std::fs::remove_file(path).unwrap();

    let (content, mut state, _) = fixture();
    state.party_buffs[4..6].copy_from_slice(&20_i16.to_be_bytes());
    state.combat.as_mut().unwrap().enemies.records[0].bytes[0xcc] = 1;
    let before = state.clone();
    assert!(apply(Command::EndTurn, &mut state, &content).is_err());
    assert_eq!(state, before);
}

#[test]
fn unrecovered_round_effect_after_health_drain_rolls_back_entire_command() {
    let (content, original, _) = fixture();
    for variant in 0..3 {
        let mut state = original.clone();
        let arena = state.combat.as_mut().unwrap();
        match variant {
            0 => arena.enemies.records[2].bytes[0xcc] = 2,
            1 => arena.enemies.records[1].bytes[0x0e..0x10].copy_from_slice(&1_i16.to_be_bytes()),
            _ => arena.enemies.records[1].bytes[0x60..0x62].copy_from_slice(&51_i16.to_be_bytes()),
        }
        let before = state.clone();
        assert!(apply(Command::EndTurn, &mut state, &content).is_err());
        assert_eq!(state, before);
    }
}
