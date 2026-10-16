use crate::prelude::*;

use crate::support::compare_record_bytes as assert_party;
use crate::support::warriors as party;

use crate::support::event_sounds as sounds;

fn state(fixture: &Value, index: usize) -> Session {
    serde_json::from_value(fixture["foundations"][index]["session"].clone()).unwrap()
}
use crate::snapshots::medical_input as install;

#[test]
fn actual_owned_needle_consumption_and_preparation_match_native_complete_records() {
    let fixture = crate::support::fixture("combat-aid.json");
    let content = content();
    for (index, case) in fixture.array("stim_cases").iter().enumerate() {
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
        assert_eq!(result.packet, case["prepared"].bytes("packet"));
        assert_eq!(result.identity, [0, 3, 0]);
        crate::support::expect(result.count, &case["prepared"]["count"]);
        assert_eq!(
            result.loaded_item_field32,
            // The caller refreshes the display's independent item cache after preparation.
            word(&case["used"].bytes("item_work"), 32).unwrap()
        );
        assert_eq!(
            result.weapon_work.sound().unwrap(),
            word(&case["prepared"].bytes("weapon_work"), 14).unwrap()
        );
        assert_eq!(
            result.weapon_work.weapon_armor_filter,
            Some(word(&case["prepared"].bytes("weapon_work"), 136).unwrap())
        );
        assert_eq!(
            result.weapon_work.ammunition_armor_filter,
            Some(word(&case["prepared"].bytes("ammo_work"), 136).unwrap())
        );
        assert_eq!(sounds(&result.events), vec![655]);
        assert_eq!(actual[4].inventory().unwrap().len(), 3);
        assert_eq!(word(&actual[4].bytes, 0x9d6).unwrap(), 165);
        assert_eq!(word(&actual[4].bytes, 0x58).unwrap(), 0);
    }
}

#[test]
fn four_native_target_continuations_match_at_the_medical_boundary() {
    let fixture = crate::support::fixture("combat-aid.json");
    let content = content();
    let pending = MedicalContinuation {
        origin: centurions_core::MedicalOrigin::OwnedNeedle,
        actor: 4,
        item: 700,
        spell: 1401,
        power: 1,
    };
    for (index, case) in fixture.array("stim_cases").iter().enumerate() {
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
        crate::support::replay_rng(&mut native_random, case.array("rng"));
        assert_eq!(state.random, native_random);
        let actual = state.combat.as_ref().unwrap();
        let before = original.combat.as_ref().unwrap();
        assert_eq!(actual.enemies, before.enemies);
        assert_eq!(actual.spell_fields, before.spell_fields);
        assert_eq!(actual.weapon_work, before.weapon_work);
        assert_eq!(actual.death_queue, before.death_queue);
        assert_eq!(actual.attack_identity, Some([0, 3, 0]));
        let mut packet = case["prepared"].bytes("packet");
        if index == 1 || index == 2 {
            packet[18] = 4;
        }
        assert_eq!(actual.attack_packet, Some(packet));
        if index == 1 {
            assert_eq!(
                actual.squad.positions,
                case["after"]
                    .words("party_positions")
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|position| (position[0], position[1]))
                    .collect::<Vec<_>>()
            );
            crate::support::native_expect! {
            words actual.squad.under_tiles => &case["after"]["under_tiles"], "binary_combat_aid";
            words actual.map.tiles => &case["after"]["map"], "binary_combat_aid";
            words actual.order => &case["after"]["order"], "binary_combat_aid";
            }
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
fn native_item_admission_and_invalid_targeting_are_atomic() {
    let fixture = crate::support::fixture("combat-aid.json");
    let content = content();
    for case in fixture.array("admission_cases") {
        let mut state = state(&fixture, 1);
        install(&mut state, &case["before"]);
        let before = state.clone();
        assert!(
            medical::begin(
                &state,
                &content,
                medical::BeginRequest {
                    warrior: case.n::<usize>("member"),
                    slot: case.n::<usize>("slot")
                }
            )
            .is_err()
        );
        assert_eq!(state, before);
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
    let target = MedicalTarget::Portrait { warrior: 6 };
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
