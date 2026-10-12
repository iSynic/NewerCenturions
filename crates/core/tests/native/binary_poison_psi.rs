use crate::prelude::*;

use centurions_core::combat::spells::select_targets;

fn fixture() -> Value {
    crate::support::fixture("poison-psi.json")
}
use crate::native_common::snapshot_party as party;

fn monsters(snapshot: &Value, key: &str) -> Vec<MonsterRecord> {
    crate::support::monsters(&bytes(&snapshot[key]))
}
use crate::native_common::random_state as random;
use crate::support::assert_bytes;

fn assert_records(result: &BodySpellResult, expected: &Value, monster_key: &str) {
    assert_bytes(
        &crate::support::record_bytes(&result.warriors),
        &expected.bytes("party"),
        "complete party",
    );
    assert_bytes(
        &crate::support::record_bytes(&result.enemies),
        &bytes(&expected[monster_key]),
        "complete monsters",
    );
}

fn audio(result: &BodySpellResult) -> Vec<i16> {
    let mut sounds = result
        .targets
        .iter()
        .map(|(body, result)| (*body, result.sounds.clone()))
        .collect::<Vec<_>>();
    sounds.extend(result.race_excluded.iter().map(|body| (*body, vec![651])));
    sounds.sort_by_key(|(body, _)| *body);
    result
        .sounds
        .iter()
        .copied()
        .chain(sounds.into_iter().flat_map(|(_, sounds)| sounds))
        .collect()
}

fn receipts(case: &Value, initial: &RandomState, actual: &RandomState) {
    let mut expected = initial.clone();
    for draw in case.array("rng") {
        crate::support::expect(expected.bound(draw.n::<i16>("bound")), &draw["result"]);
    }
    assert_eq!(&expected, actual);
}

#[test]
fn twelve_full_direct_poison_cases_match_signed_duration_gates_records_rng_and_audio() {
    let f = fixture();
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
        assert_records(&result, after, "all_records");
        assert_eq!(actual, random(after), "case {index}");
        receipts(case, &initial, &actual);
        crate::support::expect_array(&(audio(&result)), &case["sounds"]);
        assert!(
            result
                .targets
                .iter()
                .all(|(_, result)| result.damage == 0 && !result.killed)
        );
        assert_eq!(word(&packet, 40).unwrap(), 4);
        assert_eq!(word(&packet, 50).unwrap(), 0x2000);
        assert_eq!(
            result.race_excluded,
            if index == 9 { vec![4] } else { vec![] }
        );
        assert_eq!(result.targets.len(), usize::from(!matches!(index, 9 | 10)));
        assert!(result.enemies.iter().all(|enemy| enemy.bytes[0xb5] == 0));

        crate::support::native_expect! {
        bytes crate::support::record_bytes(&warriors) => &before["party"], "immutable input party";
        bytes crate::support::record_bytes(&enemies) => &before["all_records"], "immutable input monsters";
        }
        if index < 9 {
            let duration = after.n::<i16>("duration");
            let mut target = warriors[4].bytes.clone();
            let mut condition_random = initial.clone();
            spell_conditions::apply(&mut target, false, 4, duration, &mut condition_random)
                .unwrap();
            assert_bytes(
                &target,
                &result.warriors[4].bytes,
                "standalone condition addition",
            );
            assert_eq!(condition_random, initial);
        }
    }
}

#[test]
fn actual_actor_target_charge_and_resolution_match_native_complete_boundaries() {
    let f = fixture();
    let case = &f["cases"][0];
    let before = &case["before"];

    let content = crate::support::content();
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
    for draw in &rng[..3] {
        crate::support::expect(actual.bound(draw.n::<i16>("bound")), &draw["result"]);
    }
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
    crate::support::expect(actual.draws, &resolution["draws_before"]);
    let mut expected_random = random(before);
    for draw in &rng[..23] {
        crate::support::expect(
            expected_random.bound(draw.n::<i16>("bound")),
            &draw["result"],
        );
    }
    assert_eq!(actual, expected_random);
    combat.enemies.records[6] = selected.caster;
    assert_bytes(
        &crate::support::record_bytes(&combat.enemies.records),
        &resolution.bytes("records_after"),
        "complete charged monsters",
    );
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
    let expected = serde_json::json!({ "party": resolution["party_after"], "records": resolution["records_after"] });
    assert_records(&result, &expected, "records");
    assert_eq!(audio(&result), vec![430]);
    for draw in &rng[23..29] {
        crate::support::expect(
            expected_random.bound(draw.n::<i16>("bound")),
            &draw["result"],
        );
    }
    assert_eq!(actual, expected_random);
    for draw in &rng[29..] {
        crate::support::expect(
            expected_random.bound(draw.n::<i16>("bound")),
            &draw["result"],
        );
    }
    assert_eq!(expected_random, random(&case["after_cast"]));
    assert_eq!(random(&case["after_cast"]), random(&case["after_getup"]));
}
