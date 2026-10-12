use crate::prelude::*;

fn fixture() -> (Content, Value) {
    (
        crate::support::content(),
        crate::support::fixture("earned-combat-medical.json"),
    )
}
use crate::support::compare_record_bytes as assert_party;
use crate::support::warriors as party;

use crate::support::same_session as assert_session;

fn install(state: &mut Session, snapshot: &Value) {
    for (warrior, native) in state.warriors.iter_mut().zip(party(&snapshot["party"])) {
        warrior.bytes = native.bytes;
    }
    state.random = RandomState {
        seed: snapshot.n::<u32>("seed"),
        draws: snapshot.n::<u64>("draws"),
    };
    let combat = state.combat.as_mut().unwrap();
    crate::snapshots::records(combat, &|key| snapshot.bytes(key));
    crate::snapshots::geometry(combat, &|key| snapshot.bytes(key), (0, 0));
    combat.cursor = snapshot.n::<usize>("cursor");
    combat.round = snapshot.n::<i16>("round");
    combat.round_scripts.slots = snapshot.words("round_slots").try_into().unwrap();
    combat.unavailable = snapshot.n::<i8>("unavailable");
    crate::snapshots::spell_work(combat, &|key| snapshot.bytes(key));

    combat
        .weapon_work
        .load_pair(&snapshot.bytes("weapon_work"), &snapshot.bytes("ammo_work"))
        .unwrap();
}

fn state(fixture: &Value, case: &Value, pane: &str) -> Session {
    let mut state = serde_json::from_value(
        fixture["foundations"][case.n::<usize>("foundation")]["session"].clone(),
    )
    .unwrap();
    install(&mut state, &case[pane]);
    state
}
use crate::support::event_sounds as sounds;

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
    for (index, case) in fixture.array("cases")[..6].iter().enumerate() {
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
        assert_eq!(result.packet, case["prepared"].bytes("packet"));
        assert_eq!(result.identity, case["prepared"].words("identity")[..3]);
        crate::support::expect(result.count, &case["prepared"]["count"]);
        assert_eq!(
            result.loaded_item_field32,
            word(&case["used"].bytes("item_work"), 32).unwrap()
        );
        let mut expected_work = state.combat.as_ref().unwrap().weapon_work;
        expected_work
            .load_pair(
                &case["prepared"].bytes("weapon_work"),
                &case["prepared"].bytes("ammo_work"),
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
    for (index, case) in fixture.array("cases")[..5].iter().enumerate() {
        let mut state = state(&fixture, case, "prepared");
        let before = state.clone();
        let target = if index == 4 {
            MedicalTarget::ConfirmedCancel
        } else {
            let member = case.n::<usize>("target");
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
        for draw in case.array("rng") {
            crate::support::expect(native_random.bound(draw.n::<i16>("bound")), &draw["result"]);
        }
        assert_eq!(state.random, native_random);
        let native_sounds: Vec<i16> = case
            .array("use_sounds")
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
                vec![(case.n::<i16>("target"), [-11, -9, -12, -9][index])]
            );
        } else {
            assert!(effects.is_empty());
        }
    }
}

#[test]
fn native_range_and_admission_failures_are_atomic() {
    let (content, fixture) = fixture();
    for case in &fixture.array("cases")[6..] {
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
    }
    let case = &fixture["cases"][5];
    let mut state = state(&fixture, case, "prepared");
    let before = state.clone();
    let position = state.combat.as_ref().unwrap().squad.positions[case.n::<usize>("target")];
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
