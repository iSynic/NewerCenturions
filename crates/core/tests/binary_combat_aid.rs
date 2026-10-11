use centurions_content::{Content, Warrior, word};
use centurions_core::combat::{medical, spells};
use centurions_core::{Event, MedicalContinuation, MedicalTarget, RandomState, Session};
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../evidence/fixtures/combat-aid.json")).unwrap()
}

fn content() -> Content {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap()
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

fn sounds(events: &[Event]) -> Vec<i16> {
    events
        .iter()
        .filter_map(|event| {
            if let Event::Sound { resource_id } = event {
                Some(*resource_id)
            } else {
                None
            }
        })
        .collect()
}

fn state(fixture: &Value, index: usize) -> Session {
    serde_json::from_value(fixture["foundations"][index]["session"].clone()).unwrap()
}

fn install(state: &mut Session, snapshot: &Value) {
    state.warriors = party(&snapshot["party"]);
    state.random = RandomState {
        seed: snapshot["seed"].as_u64().unwrap() as u32,
        draws: snapshot["draws"].as_u64().unwrap(),
    };
    let combat = state.combat.as_mut().unwrap();
    combat.attack_packet = Some(bytes(&snapshot["packet"]));
    combat.attack_identity = Some(words(&snapshot["identity"])[..3].try_into().unwrap());
    combat.cursor = snapshot["cursor"].as_u64().unwrap() as usize;
    combat.round = snapshot["round"].as_i64().unwrap() as i16;
    combat.order = words(&snapshot["order"]);
    combat.map.tiles = words(&snapshot["map"]);
    combat.squad.positions = words(&snapshot["party_positions"])
        .as_chunks::<2>()
        .0
        .iter()
        .map(|position| (position[0], position[1]))
        .collect();
    combat.squad.under_tiles = words(&snapshot["under_tiles"]);
    combat.unavailable = snapshot["unavailable"].as_i64().unwrap() as i8;
    combat.spell_fields = bytes(&snapshot["fields"]);
    combat.field_used = bytes(&snapshot["used"]);
    combat
        .weapon_work
        .load_pair(
            &bytes(&snapshot["weapon_work"]),
            &bytes(&snapshot["ammo_work"]),
        )
        .unwrap();
}

#[test]
fn actual_owned_needle_consumption_and_preparation_match_native_complete_records() {
    let fixture = fixture();
    let content = content();
    for (index, case) in fixture["stim_cases"].as_array().unwrap().iter().enumerate() {
        let mut state = state(&fixture, usize::from(index != 0));
        install(&mut state, &case["before"]);
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
        assert_eq!(state, before);
        let mut actual = state.warriors.clone();
        actual[4] = result.warrior;
        assert_party(&actual, &party(&case["prepared"]["party"]));
        assert_eq!(result.packet, bytes(&case["prepared"]["packet"]));
        assert_eq!(result.identity, [0, 3, 0]);
        assert_eq!(
            result.count,
            case["prepared"]["count"].as_i64().unwrap() as i16
        );
        assert_eq!(
            result.loaded_item_field32,
            // The caller refreshes the display's independent item cache after preparation.
            word(&bytes(&case["used"]["item_work"]), 32).unwrap()
        );
        assert_eq!(
            result.weapon_work.sound().unwrap(),
            word(&bytes(&case["prepared"]["weapon_work"]), 14).unwrap()
        );
        assert_eq!(
            result.weapon_work.weapon_armor_filter,
            Some(word(&bytes(&case["prepared"]["weapon_work"]), 136).unwrap())
        );
        assert_eq!(
            result.weapon_work.ammunition_armor_filter,
            Some(word(&bytes(&case["prepared"]["ammo_work"]), 136).unwrap())
        );
        assert_eq!(sounds(&result.events), vec![655]);
        assert_eq!(actual[4].inventory().unwrap().len(), 3);
        assert_eq!(word(&actual[4].bytes, 0x9d6).unwrap(), 165);
        assert_eq!(word(&actual[4].bytes, 0x58).unwrap(), 0);
    }
}

#[test]
fn four_native_target_continuations_match_at_the_medical_boundary() {
    let fixture = fixture();
    let content = content();
    let pending = MedicalContinuation {
        origin: centurions_core::MedicalOrigin::OwnedNeedle,
        actor: 4,
        item: 700,
        spell: 1401,
        power: 1,
    };
    for (index, case) in fixture["stim_cases"].as_array().unwrap().iter().enumerate() {
        let mut state = state(&fixture, usize::from(index != 0));
        install(&mut state, &case["prepared"]);
        let original = state.clone();
        let target = match index {
            0 => MedicalTarget::Portrait { warrior: 3 },
            1 => MedicalTarget::Portrait { warrior: 5 },
            2 => {
                let position = state.combat.as_ref().unwrap().squad.positions[2];
                MedicalTarget::LivingPoint {
                    x: position.0,
                    y: position.1,
                }
            }
            _ => MedicalTarget::ConfirmedCancel,
        };
        let mut expected = party(if index < 3 {
            &case["resolutions"][0]["party_after"]
        } else {
            &case["after"]["party"]
        });
        if index == 1 {
            // The frozen resolver capture precedes the portrait caller's placement writes.
            expected = party(&case["after"]["party"]);
            expected[5].bytes[0x3c..0x42].copy_from_slice(
                &party(&case["resolutions"][0]["party_after"])[5].bytes[0x3c..0x42],
            );
        }
        let result = medical::resolve(
            state.combat.as_mut().unwrap(),
            &mut state.warriors,
            content
                .map(state.exploration.as_ref().unwrap().map)
                .unwrap(),
            &mut state.random,
            &pending,
            &target,
        )
        .unwrap();
        assert_party(&state.warriors, &expected);
        assert_eq!(
            result.follow_up,
            [
                medical::FollowUp::EndTurn,
                medical::FollowUp::EndTurn,
                medical::FollowUp::FinishPlayerAction,
                medical::FollowUp::None
            ][index]
        );
        assert_eq!(
            sounds(&result.events),
            [
                vec![-141, 651],
                vec![-141, 430, 631],
                vec![145, 431, 430],
                vec![6000, 20005]
            ][index]
        );
        let mut native_random = original.random.clone();
        for draw in case["rng"].as_array().unwrap() {
            assert_eq!(
                native_random.bound(draw["bound"].as_i64().unwrap() as i16),
                draw["result"].as_i64().unwrap() as i16
            );
        }
        assert_eq!(state.random, native_random);
        let actual = state.combat.as_ref().unwrap();
        let before = original.combat.as_ref().unwrap();
        assert_eq!(actual.enemies, before.enemies);
        assert_eq!(actual.spell_fields, before.spell_fields);
        assert_eq!(actual.weapon_work, before.weapon_work);
        assert_eq!(actual.death_queue, before.death_queue);
        assert_eq!(actual.attack_identity, Some([0, 3, 0]));
        let mut packet = bytes(&case["prepared"]["packet"]);
        if index == 1 || index == 2 {
            packet[18] = 4;
        }
        assert_eq!(actual.attack_packet, Some(packet));
        if index == 1 {
            assert_eq!(
                actual.squad.positions,
                words(&case["after"]["party_positions"])
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|position| (position[0], position[1]))
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                actual.squad.under_tiles,
                words(&case["after"]["under_tiles"])
            );
            assert_eq!(actual.map.tiles, words(&case["after"]["map"]));
            assert_eq!(actual.order, words(&case["after"]["order"]));
            assert_eq!(actual.squad.positions[5], (9, 23));
            assert_eq!(actual.unavailable, before.unavailable.wrapping_sub(1));
        } else {
            assert_eq!(actual.squad, before.squad);
            assert_eq!(actual.map, before.map);
            assert_eq!(actual.order, before.order);
            assert_eq!(actual.unavailable, before.unavailable);
        }
        let mut used = before.field_used.clone();
        if index == 2 {
            used[0] = 1;
        }
        assert_eq!(actual.field_used, used);
        assert_eq!(actual.cursor, before.cursor);
        assert_eq!(actual.round, before.round);
        assert_eq!(state.ammunition, original.ammunition);
        assert_eq!(state.crate_items, original.crate_items);
        assert_eq!(state.booty_items, original.booty_items);
        assert_eq!(state.psi_memory, original.psi_memory);
    }
}

#[test]
fn native_item_admission_and_unsupported_targeting_are_atomic() {
    let fixture = fixture();
    let content = content();
    for case in fixture["admission_cases"].as_array().unwrap() {
        let mut state = state(&fixture, 1);
        install(&mut state, &case["before"]);
        let before = state.clone();
        assert!(
            medical::begin(
                &state,
                &content,
                medical::BeginRequest {
                    warrior: case["member"].as_u64().unwrap() as usize,
                    slot: case["slot"].as_u64().unwrap() as usize
                }
            )
            .is_err()
        );
        assert_eq!(state, before);
        assert_eq!(
            bytes(&case["after"]["party"]),
            bytes(&case["before"]["party"])
        );
        assert!(case["rng"].as_array().unwrap().is_empty());
        assert!(case["sounds"].as_array().unwrap().is_empty());
    }
    let mut state = state(&fixture, 1);
    install(&mut state, &fixture["stim_cases"][1]["prepared"]);
    let pending = MedicalContinuation {
        origin: centurions_core::MedicalOrigin::OwnedNeedle,
        actor: 4,
        item: 700,
        spell: 1401,
        power: 1,
    };
    for target in [
        MedicalTarget::Portrait { warrior: 4 },
        MedicalTarget::Portrait { warrior: 6 },
        MedicalTarget::LivingPoint { x: -100, y: -100 },
    ] {
        let before = state.clone();
        assert!(
            medical::resolve(
                state.combat.as_mut().unwrap(),
                &mut state.warriors,
                content
                    .map(state.exploration.as_ref().unwrap().map)
                    .unwrap(),
                &mut state.random,
                &pending,
                &target
            )
            .is_err()
        );
        assert_eq!(state, before);
    }
    let original_ap = state.warriors[4].bytes[0x40..0x42].to_vec();
    state.warriors[4].bytes[0x40..0x42].copy_from_slice(&1_i16.to_be_bytes());
    let before = state.clone();
    let position = state.combat.as_ref().unwrap().squad.positions[2];
    assert!(
        medical::resolve(
            state.combat.as_mut().unwrap(),
            &mut state.warriors,
            content
                .map(state.exploration.as_ref().unwrap().map)
                .unwrap(),
            &mut state.random,
            &pending,
            &MedicalTarget::LivingPoint {
                x: position.0,
                y: position.1
            },
        )
        .is_err()
    );
    assert_eq!(state, before);
    state.warriors[4].bytes[0x40..0x42].copy_from_slice(&original_ap);
    for (index, value) in [(0, 0), (2, 5), (11, 7), (40, 1), (50, 0)] {
        let combat = state.combat.as_mut().unwrap();
        let old = combat.attack_packet.as_ref().unwrap()[index];
        combat.attack_packet.as_mut().unwrap()[index] = value;
        let before = state.clone();
        assert!(
            medical::resolve(
                state.combat.as_mut().unwrap(),
                &mut state.warriors,
                content
                    .map(state.exploration.as_ref().unwrap().map)
                    .unwrap(),
                &mut state.random,
                &pending,
                &MedicalTarget::Portrait { warrior: 5 }
            )
            .is_err()
        );
        assert_eq!(state, before);
        state
            .combat
            .as_mut()
            .unwrap()
            .attack_packet
            .as_mut()
            .unwrap()[index] = old;
    }
    let before = state.random.clone();
    let mut mask = [0; 110];
    mask[5] = 1;
    assert!(
        spells::resolve_player_support(
            &state.warriors,
            &state.combat.as_ref().unwrap().enemies.records,
            &mask,
            state
                .combat
                .as_ref()
                .unwrap()
                .attack_packet
                .as_ref()
                .unwrap(),
            1401,
            2,
            &mut state.random
        )
        .is_err()
    );
    assert_eq!(state.random, before);
}
