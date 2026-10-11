use centurions_content::{Content, Warrior, word};
use centurions_core::combat::medical;
use centurions_core::{
    Event, MedicalContinuation, MedicalOrigin, MedicalTarget, RandomState, Session,
};
use serde_json::Value;

fn fixture() -> (Content, Value) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    (
        Content::load(
            &root.join("NewCenturions"),
            &root.join("analysis/extracted"),
        )
        .unwrap(),
        serde_json::from_str(include_str!(
            "../../../evidence/fixtures/earned-combat-medical.json"
        ))
        .unwrap(),
    )
}

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
        .map(|word| i16::from_be_bytes(*word))
        .collect()
}

fn party(value: &Value) -> Vec<Warrior> {
    bytes(value)
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|record| Warrior::read(record).unwrap())
        .collect()
}

fn assert_party(actual: &[Warrior], expected: &[Warrior]) {
    assert_eq!(actual.len(), expected.len());
    for (member, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert!(
            actual.bytes == expected.bytes,
            "member {member}: {:?}",
            actual
                .bytes
                .iter()
                .zip(&expected.bytes)
                .enumerate()
                .filter(|(_, (actual, expected))| actual != expected)
                .take(24)
                .collect::<Vec<_>>()
        );
    }
}

fn assert_session(actual: &Session, expected: &Session) {
    assert_party(&actual.warriors, &expected.warriors);
    let actual = serde_json::to_value(actual).unwrap();
    let expected = serde_json::to_value(expected).unwrap();
    for (key, value) in actual.as_object().unwrap() {
        assert!(value == &expected[key], "session field {key} differs");
    }
}

fn install(state: &mut Session, snapshot: &Value) {
    for (warrior, native) in state.warriors.iter_mut().zip(party(&snapshot["party"])) {
        warrior.bytes = native.bytes;
    }
    state.random = RandomState {
        seed: snapshot["seed"].as_u64().unwrap() as u32,
        draws: snapshot["draws"].as_u64().unwrap(),
    };
    let combat = state.combat.as_mut().unwrap();
    for (record, native) in combat
        .enemies
        .records
        .iter_mut()
        .zip(bytes(&snapshot["records"]).as_chunks::<288>().0)
    {
        record.bytes = native.to_vec();
    }
    for (destination, key) in [
        (&mut combat.squad.positions, "party_positions"),
        (&mut combat.enemies.positions, "monster_positions"),
    ] {
        *destination = words(&snapshot[key])
            .as_chunks::<2>()
            .0
            .iter()
            .map(|position| (position[0], position[1]))
            .collect();
    }
    combat.squad.under_tiles = words(&snapshot["under_tiles"]);
    combat.map.tiles = words(&snapshot["map"]);
    combat.order = words(&snapshot["order"]);
    combat.cursor = snapshot["cursor"].as_u64().unwrap() as usize;
    combat.round = snapshot["round"].as_i64().unwrap() as i16;
    combat.round_scripts.slots = words(&snapshot["round_slots"]).try_into().unwrap();
    combat.unavailable = snapshot["unavailable"].as_i64().unwrap() as i8;
    combat.spell_fields = bytes(&snapshot["fields"]);
    combat.field_used = bytes(&snapshot["used"]);
    combat.attack_packet = Some(bytes(&snapshot["packet"]));
    combat.attack_identity = Some(words(&snapshot["identity"]).try_into().unwrap());
    combat
        .weapon_work
        .load_pair(
            &bytes(&snapshot["weapon_work"]),
            &bytes(&snapshot["ammo_work"]),
        )
        .unwrap();
}

fn state(fixture: &Value, case: &Value, pane: &str) -> Session {
    let mut state = serde_json::from_value(
        fixture["foundations"][case["foundation"].as_u64().unwrap() as usize]["session"].clone(),
    )
    .unwrap();
    install(&mut state, &case[pane]);
    state
}

fn sounds(events: &[Event]) -> Vec<i16> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::Sound { resource_id } => Some(*resource_id),
            _ => None,
        })
        .collect()
}

fn pending() -> MedicalContinuation {
    MedicalContinuation {
        origin: MedicalOrigin::OwnedNeedle,
        actor: 4,
        item: 700,
        spell: 1401,
        power: 1,
    }
}

#[test]
fn six_native_earned_needle_preparations_match_complete_party_records() {
    let (content, fixture) = fixture();
    for (index, case) in fixture["cases"].as_array().unwrap()[..6].iter().enumerate() {
        let state = state(&fixture, case, "pane_opened");
        let before = state.clone();
        let result = medical::begin(
            &state,
            &content,
            medical::BeginRequest {
                warrior: 4,
                slot: 3,
            },
        )
        .unwrap();
        assert_session(&state, &before);
        let mut warriors = state.warriors.clone();
        warriors[4] = result.warrior;
        assert_party(&warriors, &party(&case["prepared"]["party"]));
        assert_eq!(result.packet, bytes(&case["prepared"]["packet"]));
        assert_eq!(result.identity, words(&case["prepared"]["identity"])[..3]);
        assert_eq!(
            result.count,
            case["prepared"]["count"].as_i64().unwrap() as i16
        );
        assert_eq!(
            result.loaded_item_field32,
            word(&bytes(&case["used"]["item_work"]), 32).unwrap()
        );
        let mut expected_work = state.combat.as_ref().unwrap().weapon_work;
        expected_work
            .load_pair(
                &bytes(&case["prepared"]["weapon_work"]),
                &bytes(&case["prepared"]["ammo_work"]),
            )
            .unwrap();
        assert_eq!(result.weapon_work, expected_work);
        assert_eq!(
            sounds(&result.events),
            if index == 0 { vec![] } else { vec![655] }
        );
        assert_eq!(
            warriors[4].inventory().unwrap().len(),
            if index == 0 { 4 } else { 3 }
        );
        assert_eq!(
            word(&warriors[4].bytes, 0x9d6).unwrap(),
            if index == 0 { 170 } else { 165 }
        );
        if index == 0 {
            let remaining = &warriors[4].inventory().unwrap()[3].0;
            assert_eq!(word(remaining, 2).unwrap(), 1);
            assert_eq!(remaining[10], 24);
        }
    }
}

#[test]
fn native_living_heals_and_cancel_match_complete_medical_rule_state() {
    let (content, fixture) = fixture();
    for (index, case) in fixture["cases"].as_array().unwrap()[..5].iter().enumerate() {
        let mut state = state(&fixture, case, "prepared");
        let before = state.clone();
        let target = if index == 4 {
            MedicalTarget::ConfirmedCancel
        } else {
            let member = case["target"].as_u64().unwrap() as usize;
            let position = state.combat.as_ref().unwrap().squad.positions[member];
            MedicalTarget::LivingPoint {
                x: position.0,
                y: position.1,
            }
        };
        let result = medical::resolve(
            state.combat.as_mut().unwrap(),
            &mut state.warriors,
            content
                .map(state.exploration.as_ref().unwrap().map)
                .unwrap(),
            &mut state.random,
            &pending(),
            &target,
        )
        .unwrap();
        let mut expected = before.clone();
        install(&mut expected, &case["after"]);
        assert_session(&state, &expected);
        let mut native_random = before.random;
        for draw in case["rng"].as_array().unwrap() {
            assert_eq!(
                native_random.bound(draw["bound"].as_i64().unwrap() as i16),
                draw["result"].as_i64().unwrap() as i16
            );
        }
        assert_eq!(state.random, native_random);
        let native_sounds: Vec<i16> = case["use_sounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_i64().unwrap() as i16)
            .filter(|sound| *sound != 655)
            .collect();
        assert_eq!(sounds(&result.events), native_sounds);
        assert_eq!(
            result.follow_up,
            if index == 4 {
                medical::FollowUp::None
            } else {
                medical::FollowUp::FinishPlayerAction
            }
        );
        let effects: Vec<(i16, i16)> = result
            .events
            .iter()
            .filter_map(|event| match event {
                Event::SpellResolved {
                    spell: 1401,
                    target,
                    damage,
                    ..
                } => Some((*target, *damage)),
                _ => None,
            })
            .collect();
        if index < 4 {
            assert_eq!(
                effects,
                vec![(
                    case["target"].as_i64().unwrap() as i16,
                    [-11, -9, -12, -9][index]
                )]
            );
        } else {
            assert!(effects.is_empty());
        }
    }
}

#[test]
fn native_range_and_admission_failures_are_atomic() {
    let (content, fixture) = fixture();
    for case in &fixture["cases"].as_array().unwrap()[6..] {
        let state = state(&fixture, case, "pane_opened");
        let before = state.clone();
        assert!(
            medical::begin(
                &state,
                &content,
                medical::BeginRequest {
                    warrior: 4,
                    slot: 3
                }
            )
            .is_err()
        );
        assert_session(&state, &before);
        assert!(case["rng"].as_array().unwrap().is_empty());
        assert!(case["use_sounds"].as_array().unwrap().is_empty());
    }
    let case = &fixture["cases"][5];
    let mut state = state(&fixture, case, "prepared");
    let before = state.clone();
    let position =
        state.combat.as_ref().unwrap().squad.positions[case["target"].as_u64().unwrap() as usize];
    assert!(
        medical::resolve(
            state.combat.as_mut().unwrap(),
            &mut state.warriors,
            content
                .map(state.exploration.as_ref().unwrap().map)
                .unwrap(),
            &mut state.random,
            &pending(),
            &MedicalTarget::LivingPoint {
                x: position.0,
                y: position.1
            }
        )
        .is_err()
    );
    assert_session(&state, &before);
    assert!(case["rng"].as_array().unwrap().is_empty());
}

#[test]
fn other_medical_item_shapes_remain_unsupported_without_writes() {
    let (content, fixture) = fixture();
    let original = state(&fixture, &fixture["cases"][1], "pane_opened");
    for (offset, value) in [(3, 2), (10, 29), (13, 2), (18, 1), (8, 1), (5, 1), (7, 2)] {
        let mut state = original.clone();
        state.warriors[4].bytes[0x132 + 3 * 22 + offset] = value;
        let before = state.clone();
        assert!(
            medical::begin(
                &state,
                &content,
                medical::BeginRequest {
                    warrior: 4,
                    slot: 3
                }
            )
            .is_err()
        );
        assert_session(&state, &before);
    }
    let mut state = state(&fixture, &fixture["cases"][0], "pane_opened");
    state.warriors[4].bytes[0x132 + 3 * 22 + 2..0x132 + 3 * 22 + 4]
        .copy_from_slice(&3_i16.to_be_bytes());
    let before = state.clone();
    assert!(
        medical::begin(
            &state,
            &content,
            medical::BeginRequest {
                warrior: 4,
                slot: 3
            }
        )
        .is_err()
    );
    assert_session(&state, &before);
}
