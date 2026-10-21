use crate::{native_common, prelude::*, support};

use centurions_core::combat::spells::select_targets;

use native_common::snapshot_party as party;

fn monsters(snapshot: &Value, key: &str) -> Vec<MonsterRecord> {
    support::monsters(&bytes(&snapshot[key]))
}
use native_common::random_state as random;

use native_common::body_spell_sounds as audio;

fn receipts(case: &Value, initial: &RandomState, actual: &RandomState) {
    let mut expected = initial.clone();
    support::replay_rng(&mut expected, case.array("rng"));
    assert_eq!(&expected, actual);
}

#[test]
fn twelve_full_direct_poison_cases_match_signed_duration_gates_records_rng_and_audio() {
    let f = support::fixture("poison-psi.json");
    let cases = f.array("direct_cases");

    for (index, case) in cases.iter().enumerate() {
        let before = &case["before"];
        let after = &case["after"];
        let warriors = party(before);
        let enemies = monsters(before, "all_records");
        let packet = before.bytes("packet");
        let mask = before.words("mask");
        let initial = random(before);
        let mut actual = initial.clone();
        let weapon = before.bytes("weapon_work");
        let ammo = before.bytes("ammo_work");
        let result = resolve_bodies(
            &warriors,
            &enemies,
            &mask,
            &DirectSpell {
                packet: &packet,
                level: before.words("identity")[1],
                count: case.n::<i16>("count"),
                armor_filter: word(&weapon, 136)
                    .unwrap()
                    .wrapping_add(word(&ammo, 136).unwrap())
                    != 0,
                weapon_sound: word(&weapon, 14).unwrap(),
            },
            &mut actual,
        )
        .unwrap();
        native_common::assert_body_records(
            &result.warriors,
            &result.enemies,
            &after["party"],
            &after["all_records"],
        );
        assert_eq!(actual, random(after), "case {index}");
        receipts(case, &initial, &actual);
        support::expect_array(&(audio(&result)), &case["sounds"]);
        assert!(
            result
                .targets
                .iter()
                .all(|(_, result)| result.damage == 0 && !result.killed)
        );
        assert_eq!(
            result.race_excluded,
            if index == 9 { vec![4] } else { vec![] }
        );
        assert_eq!(result.targets.len(), usize::from(!matches!(index, 9 | 10)));
    }
}

#[test]
fn actual_actor_target_charge_and_resolution_match_native_complete_boundaries() {
    let f = support::fixture("poison-psi.json");
    let case = &f["cases"][0];
    let before = &case["before"];

    let content = support::content();
    let state: Session = serde_json::from_value(f["foundation"]["session"].clone()).unwrap();
    let mission = content
        .map(state.exploration.as_ref().unwrap().map)
        .unwrap();
    let mut combat = state.combat.unwrap();
    let warriors = party(before);
    combat.enemies.records = monsters(before, "records");
    let cast = &case["casts"][0];
    let packet = cast.bytes("packet");
    let mut actual = random(before);
    let rng = case.array("rng");
    support::replay_rng(&mut actual, &rng[..3]);
    let selected = select_targets(
        &combat,
        &warriors,
        mission,
        &mission.areas,
        6,
        &packet,
        &mut actual,
    )
    .unwrap();
    assert!(selected.cast);
    assert_eq!(selected.count, 4);
    assert_eq!(
        selected
            .targets
            .iter()
            .flat_map(|target| [target.first, target.second, target.kind])
            .collect::<Vec<_>>(),
        cast.words("targets")
    );
    let resolution = &case["resolutions"][0];
    support::expect(actual.draws, &resolution["draws_before"]);
    let mut expected_random = random(before);
    support::replay_rng(&mut expected_random, &rng[..23]);
    assert_eq!(actual, expected_random);
    combat.enemies.records[6] = selected.caster;
    support::native_expect! {
        bytes &support::record_bytes(&combat.enemies.records) => &resolution.bytes("records_after"), "complete charged monsters";
    };
    let result = resolve_bodies_with_item_cache(
        &warriors,
        &combat.enemies.records,
        &resolution.words("mask"),
        &DirectSpell {
            packet: &packet,
            level: 1,
            count: 4,
            armor_filter: false,
            weapon_sound: 402,
        },
        &before.bytes("first_item_cache"),
        &mut actual,
    )
    .unwrap();
    native_common::assert_body_records(
        &result.warriors,
        &result.enemies,
        &resolution["party_after"],
        &resolution["records_after"],
    );
    assert_eq!(audio(&result), vec![430]);
    support::replay_rng(&mut expected_random, &rng[23..29]);
    assert_eq!(actual, expected_random);
    support::replay_rng(&mut expected_random, &rng[29..]);
    assert_eq!(expected_random, random(&case["after_cast"]));
}
