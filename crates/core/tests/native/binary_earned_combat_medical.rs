use crate::prelude::*;

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
    if let Some(counter) = snapshot.get("round_counter") {
        combat.round_scripts.counter = counter.as_i64().unwrap() as i8;
        combat.inactive_rounds = snapshot.n("inactive_rounds");
    }
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

fn pending(case: &Value) -> MedicalContinuation {
    let item = case
        .get("item")
        .map_or(700, |id| id.as_i64().unwrap() as i16);
    MedicalContinuation {
        origin: MedicalOrigin::OwnedNeedle,
        actor: 4,
        item,
        spell: 1401,
        power: 1,
    }
}

#[test]
fn native_owned_needle_preparations_match_complete_party_records() {
    let content = crate::support::content();
    let fixture: Value = crate::support::fixture("earned-combat-medical.json");
    let powers = crate::support::fixture("medical-item-powers.json");
    let low_ap = crate::support::fixture("medical-low-ap.json");
    for (index, case) in fixture.array("cases")[..6]
        .iter()
        .chain(powers.array("cases"))
        .chain(low_ap.array("cases"))
        .enumerate()
    {
        let mut state = state(&fixture, case, "pane_opened");
        let events = apply(
            Command::UseItem {
                warrior: 4,
                slot: 3,
            },
            &mut state,
            &content,
        )
        .unwrap();
        let warriors = &state.warriors;
        let combat = state.combat.as_ref().unwrap();
        assert_party(warriors, &party(&case["prepared"]["party"]));
        assert_eq!(
            combat.attack_packet.as_ref().unwrap(),
            &case["prepared"].bytes("packet")
        );
        assert_eq!(
            combat.attack_identity.unwrap(),
            case["prepared"].words("identity")[..3]
        );
        crate::support::expect(state.attack_count, &case["prepared"]["count"]);
        assert_eq!(
            state.loaded_item_field32.unwrap(),
            word(&case["prepared"].bytes("item_work"), 32).unwrap()
        );
        let mut expected_work = state.combat.as_ref().unwrap().weapon_work;
        expected_work
            .load_pair(
                &case["prepared"].bytes("weapon_work"),
                &case["prepared"].bytes("ammo_work"),
            )
            .unwrap();
        assert_eq!(combat.weapon_work, expected_work);
        assert_eq!(sounds(&events), if index == 0 { vec![] } else { vec![655] });
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
        if case["item"] == 701 {
            let state = crate::support::checkpoint(&state, &content, "higher-medical");
            let path =
                std::env::temp_dir().join(format!("medical-legacy-{}.json", std::process::id()));
            save(&path, &state, &content).unwrap();
            let mut legacy: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            legacy["schema_version"] = 5.into();
            std::fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();
            assert!(load(&path, &content).is_err());
            std::fs::remove_file(path).unwrap();
        }
        if case["low_ap"] == true {
            let mut resumed = crate::support::checkpoint(&state, &content, "one-ap-medical");
            let point = state.combat.as_ref().unwrap().squad.positions[0];
            let events = apply(
                Command::SelectMedicalTarget {
                    selection: MedicalTarget::LivingPoint {
                        x: point.0,
                        y: point.1,
                    },
                },
                &mut resumed,
                &content,
            )
            .unwrap();
            let mut expected = state.clone();
            crate::command_learned_portrait_heal::project(&mut expected, &case["after"]);
            expected.medical_continuation = None;
            expected.ammunition_work = centurions_core::ammunition::WorkBuffers {
                item: Some(case["after"].bytes("item_work")),
                weapon: Some(case["after"].bytes("weapon_work")),
                ammo: Some(case["after"].bytes("ammo_work")),
            };
            assert_session(&resumed, &expected);
            assert_eq!(sounds(&events), case.n::<Vec<i16>>("use_sounds")[1..]);
        }
    }
}

#[test]
fn native_living_heals_and_cancel_match_complete_medical_rule_state() {
    let content = crate::support::content();
    let fixture: Value = crate::support::fixture("earned-combat-medical.json");
    let powers = crate::support::fixture("medical-item-powers.json");
    for (index, case) in fixture.array("cases")[..5]
        .iter()
        .chain(powers.array("cases"))
        .enumerate()
    {
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
            &pending(case),
            &target,
        )
        .unwrap();
        let mut expected = before.clone();
        install(&mut expected, &case["after"]);
        assert_session(&state, &expected);
        let mut native_random = before.random;
        crate::support::replay_rng(&mut native_random, case.array("rng"));
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
    }
}

#[test]
fn native_range_and_admission_failures_are_atomic() {
    let content = crate::support::content();
    let fixture: Value = crate::support::fixture("earned-combat-medical.json");
    for case in &fixture.array("cases")[6..] {
        let state = state(&fixture, case, "pane_opened");
        let before = state.clone();
        assert!(
            matches!(medical::begin(&state, &content, medical::BeginRequest { warrior: 4, slot: 3 }),
            Err(RuleError::Refused { feedback }) if feedback.is_empty())
        );
        assert_session(&state, &before);
    }
    let case = &fixture["cases"][5];
    let mut state = state(&fixture, case, "prepared");
    let before = state.clone();
    let position = state.combat.as_ref().unwrap().squad.positions[case.n::<usize>("target")];
    assert!(matches!(
        medical::resolve(
            state.combat.as_mut().unwrap(),
            &mut state.warriors,
            content
                .map(state.exploration.as_ref().unwrap().map)
                .unwrap(),
            &mut state.random,
            &pending(case),
            &MedicalTarget::LivingPoint {
                x: position.0,
                y: position.1
            }
        ),
        Err(RuleError::Refused { feedback }) if feedback == vec![centurions_core::feedback::NativeFeedback::Combat {
            kind: centurions_core::feedback::CombatRefusal::OutOfRange,
        }]
    ));
    assert_session(&state, &before);
}
