use crate::prelude::*;

fn fixture() -> Value {
    crate::support::fixture("learned-portrait-heal.json")
}
use crate::support::expect_records as assert_party;
use crate::support::warriors as party;

use crate::support::event_sounds as sounds;

fn state(fixture: &Value, case: &Value, snapshot: &str) -> Session {
    let foundation = case.n::<usize>("foundation");
    let mut state: Session =
        serde_json::from_value(fixture["foundations"][foundation]["session"].clone()).unwrap();
    let snapshot = &case[snapshot];
    crate::snapshots::medical_input(&mut state, snapshot);
    state
}
use crate::native_common::medical_continuation as pending;

use crate::native_common::same_unmodified_combat as assert_world_unchanged;

#[test]
fn nine_accepted_preparations_match_native_party_packet_work_and_charges() {
    let fixture = fixture();
    let content = content();
    for case in fixture
        .array("cases")
        .iter()
        .filter(|case| case["prepared"] == true)
    {
        let before = state(&fixture, case, "before");
        let original = before.clone();
        let power = case.n::<i16>("power");
        let result = medical::begin_learned_heal(&before, &content, power).unwrap();
        assert_eq!(before, original);
        let mut actual = before.warriors.clone();
        actual[5] = result.warrior;
        assert_party(&actual, &case["after_preparation"]["party"]);
        assert_eq!(result.packet, case["after_preparation"].bytes("packet"));
        assert_eq!(result.identity, [0, 0, 0]);
        assert_eq!(result.count, power);
        assert_eq!(
            result.weapon_work,
            before.combat.as_ref().unwrap().weapon_work
        );
        assert_eq!(
            result.weapon_work.sound().unwrap(),
            if case["foundation"] == 0 { 350 } else { 363 }
        );
        assert_eq!(
            result.loaded_item_field32,
            before.loaded_item_field32.unwrap()
        );
        assert_eq!(
            sounds(&result.events),
            if power == 1 {
                vec![141, 20002, 141]
            } else {
                vec![141, 20002, 141, 137]
            }
        );
        for (a, b) in actual.iter().zip(&before.warriors) {
            assert_eq!(a.inventory().unwrap(), b.inventory().unwrap());
        }
        assert_eq!(word(&actual[5].bytes, 0x40).unwrap(), 3);
        assert_eq!(word(&actual[5].bytes, 0x3c).unwrap(), 8);
        assert_eq!(
            word(&actual[5].bytes, 0x58).unwrap(),
            word(&before.warriors[5].bytes, 0x58).unwrap() - 6 * power
        );
        assert_eq!(
            word(&actual[5].bytes, 0xac8).unwrap(),
            word(&before.warriors[5].bytes, 0xac8).unwrap() + 1
        );
        assert_eq!(
            i32::from_be_bytes(actual[5].bytes[0xa66..0xa6a].try_into().unwrap()),
            i32::from_be_bytes(before.warriors[5].bytes[0xa66..0xa6a].try_into().unwrap()) + 1
        );
    }
}

#[test]
fn four_organic_portraits_and_race_immunity_match_native_resolver_and_restoration() {
    let fixture = fixture();
    let content = content();
    for case in fixture
        .array("cases")
        .iter()
        .filter(|case| case["kind"] == "actual" || case["kind"] == "race_three")
    {
        let mut actual = state(&fixture, case, "after_preparation");
        let before = actual.clone();
        let pending = pending(case);
        let target = case.n::<usize>("target");
        let mission = content
            .map(actual.exploration.as_ref().unwrap().map)
            .unwrap();
        let mut direct_random = before.random.clone();
        let mut mask = [0; 110];
        mask[target] = 1;
        let direct = spells::resolve_player_support(
            &before.warriors,
            &before.combat.as_ref().unwrap().enemies.records,
            &mask,
            before
                .combat
                .as_ref()
                .unwrap()
                .attack_packet
                .as_ref()
                .unwrap(),
            1101,
            pending.power,
            &mut direct_random,
        )
        .unwrap();
        assert_party(&direct.warriors, &case["resolutions"][0]["party_after"]);
        let result = medical::resolve(
            actual.combat.as_mut().unwrap(),
            &mut actual.warriors,
            mission,
            &mut actual.random,
            &pending,
            &MedicalTarget::Portrait { warrior: target },
        )
        .unwrap();
        assert_eq!(result.follow_up, medical::FollowUp::EndTurn);
        assert_eq!(
            sounds(&result.events),
            if case["kind"] == "actual" {
                vec![-141, 624, 631]
            } else {
                vec![-141, 651]
            }
        );
        assert_party(&actual.warriors, &case["after"]["party"]);
        assert_eq!(actual.random, direct_random);
        let mut receipt_random = before.random.clone();
        for draw in case.array("rng").iter().take(if case["kind"] == "actual" {
            1 + pending.power as usize
        } else {
            1
        }) {
            crate::support::expect(
                receipt_random.bound(draw.n::<i16>("bound")),
                &draw["result"],
            );
        }
        assert_eq!(actual.random, receipt_random);
        assert_world_unchanged(&before, &actual);
        let combat = actual.combat.as_ref().unwrap();
        assert_eq!(
            combat.squad.positions,
            case["after"]
                .words("party_positions")
                .as_chunks::<2>()
                .0
                .iter()
                .map(|p| (p[0], p[1]))
                .collect::<Vec<_>>()
        );
        crate::support::native_expect! {
        words combat.squad.under_tiles => &case["after"]["under_tiles"], "binary_learned_portrait_heal";
        words combat.map.tiles => &case["after"]["map"], "binary_learned_portrait_heal";
        words combat.order => &case["after"]["order"], "binary_learned_portrait_heal";
        scalar combat.unavailable => &case["after"]["unavailable"], "binary_learned_portrait_heal";
        }
        let mut packet = case["after_preparation"].bytes("packet");
        if case["kind"] == "actual" {
            packet[18] = 4;
        }
        assert_eq!(combat.attack_packet, Some(packet));
        assert_eq!(word(&actual.warriors[5].bytes, 0x40).unwrap(), 3);
        assert_eq!(word(&actual.warriors[5].bytes, 0x3c).unwrap(), 8);
        assert_eq!(
            word(&actual.warriors[5].bytes, 0x58).unwrap(),
            word(&before.warriors[5].bytes, 0x58).unwrap()
        );
    }
}

#[test]
fn confirmed_cancel_and_qualified_rejections_preserve_native_cost_lifecycles() {
    let fixture = fixture();
    let content = content();
    for case in fixture.array("cases") {
        let kind = case.text("kind");
        if kind == "confirmed_cancel" {
            let mut actual = state(&fixture, case, "after_preparation");
            let before = actual.clone();
            let pending = pending(case);
            let mission = content
                .map(actual.exploration.as_ref().unwrap().map)
                .unwrap();
            let result = medical::resolve(
                actual.combat.as_mut().unwrap(),
                &mut actual.warriors,
                mission,
                &mut actual.random,
                &pending,
                &MedicalTarget::ConfirmedCancel,
            )
            .unwrap();
            assert_party(&actual.warriors, &case["after"]["party"]);
            assert_eq!(actual.random, before.random);
            assert_eq!(result.follow_up, medical::FollowUp::None);
            assert_eq!(sounds(&result.events), vec![6000, 20005]);
            assert_world_unchanged(&before, &actual);
            assert_eq!(actual.combat, before.combat);
            assert_eq!(
                word(&actual.warriors[5].bytes, 0xac8).unwrap(),
                word(&party(&case["before"]["party"])[5].bytes, 0xac8).unwrap()
            );
            assert_eq!(
                actual.warriors[5].bytes[0xa66..0xa6a],
                before.warriors[5].bytes[0xa66..0xa6a]
            );
        } else if matches!(kind, "living_portrait" | "terminal_portrait") {
            let mut actual = state(&fixture, case, "after_preparation");
            let before = actual.clone();
            let pending = pending(case);
            let mission = content
                .map(actual.exploration.as_ref().unwrap().map)
                .unwrap();
            assert!(
                medical::resolve(
                    actual.combat.as_mut().unwrap(),
                    &mut actual.warriors,
                    mission,
                    &mut actual.random,
                    &pending,
                    &MedicalTarget::Portrait {
                        warrior: case.n::<usize>("target")
                    }
                )
                .is_err()
            );
            assert_eq!(actual, before);
        } else if matches!(kind, "insufficient_psi" | "cannot_cast" | "not_learned") {
            let actual = state(&fixture, case, "before");
            let before = actual.clone();
            assert!(medical::begin_learned_heal(&actual, &content, 1).is_err());
            assert_eq!(actual, before);
        }
    }
    let case = &fixture["cases"][0];
    let mut actual = state(&fixture, case, "after_preparation");
    let before = actual.clone();
    let pending = pending(case);
    let mission = content
        .map(actual.exploration.as_ref().unwrap().map)
        .unwrap();
    let p = actual.combat.as_ref().unwrap().squad.positions[5];
    assert!(
        medical::resolve(
            actual.combat.as_mut().unwrap(),
            &mut actual.warriors,
            mission,
            &mut actual.random,
            &pending,
            &MedicalTarget::LivingPoint { x: p.0, y: p.1 }
        )
        .is_err()
    );
    assert_eq!(actual, before);
    let original = state(&fixture, case, "before");
    for power in [-1, 0, 5, 7, 8] {
        assert!(medical::begin_learned_heal(&original, &content, power).is_err());
    }
}
