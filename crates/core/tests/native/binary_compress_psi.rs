use crate::prelude::*;

use centurions_core::combat::damage::DamageFlags;
use centurions_core::combat::damage::DamagePacket;
use centurions_core::combat::damage::resolve_damage;
use centurions_core::combat::lifecycle;
fn fixture() -> Value {
    crate::support::fixture("compress-psi.json")
}

use crate::support::warriors as party;

use crate::support::monsters as enemies;

use crate::native_common::random_state as random;

use crate::support::assert_bytes;

fn assert_records(result: &BodySpellResult, party_hex: &str, monster_hex: &str) {
    assert_bytes(
        &crate::support::record_bytes(&result.warriors),
        &bytes(party_hex),
        "complete native party",
    );
    assert_bytes(
        &crate::support::record_bytes(&result.enemies),
        &bytes(monster_hex),
        "complete native monsters",
    );
}
use crate::native_common::body_spell_sounds as audio;

fn verify_receipts(case: &Value, actual: &RandomState) {
    let mut expected = random(&case["before"]);
    for receipt in case.array("rng") {
        crate::support::expect(
            expected.bound(receipt.n::<i16>("bound")),
            &receipt["result"],
        );
    }
    assert_eq!(&expected, actual);
    assert_eq!(expected, random(&case["after"]));
}

#[test]
fn three_complete_compress_resolutions_match_party_npc_records_rng_and_audio() {
    let fixture = fixture();
    let cases = fixture.array("cast_cases");

    for case in cases.iter().skip(1) {
        let before = &case["before"];
        let after = &case["after"];
        let packet = before.bytes("packet");
        let mask = words(before.text("mask"));
        let warriors = party(before.text("party"));
        let monsters = enemies(before.text("records"));
        assert_eq!(packet[19], 8);
        assert_eq!(word(&packet, 40).unwrap(), 0);
        let work = before.bytes("weapon_work");
        let ammunition = before.bytes("ammo_work");
        let mut actual = random(before);
        let result = resolve_bodies(
            &warriors,
            &monsters,
            &mask,
            &DirectSpell {
                packet: &packet,
                level: 2,
                count: 2,
                armor_filter: word(&work, 0x88)
                    .unwrap()
                    .wrapping_add(word(&ammunition, 0x88).unwrap())
                    != 0,
                weapon_sound: word(&work, 0x0e).unwrap(),
            },
            &mut actual,
        )
        .unwrap();
        assert_records(&result, after.text("party"), after.text("records"));
        verify_receipts(case, &actual);
        // The outer caller also owns native showresults audio, captured separately.
        assert_eq!(
            audio(&result),
            case.array("native_calls")
                .iter()
                .filter(|call| call["address"] == "133a18")
                .map(|call| call["arguments"][0].as_i64().unwrap() as i16)
                .collect::<Vec<_>>()
        );
        assert!(result.race_excluded.is_empty());
        assert_eq!(result.targets.len(), 1);
        crate::support::expect(result.targets[0].0, &case["target"]);

        if case["target"] == 18 {
            assert_eq!(word(&monsters[8].bytes, 0x10).unwrap(), 0);
            assert_eq!(monsters[8].bytes[0x90], 101);
            assert_eq!(result.targets[0].1.damage, 0);
            assert_eq!(actual.draws - random(before).draws, 2);
        } else {
            assert_eq!(actual.draws - random(before).draws, 6);
        }
    }
}

#[test]
fn all_eight_mass_helper_controls_match_native_signed_products_and_complete_records() {
    let fixture = fixture();
    let cases = fixture.array("mass_cases");

    for case in cases {
        let before = &case["before"];
        let after = &case["after"];
        let mut monsters = enemies(before.text("records"));
        let packet = before.bytes("packet");
        let mut actual = random(before);
        let result = resolve_damage(
            &mut monsters[8].bytes,
            true,
            &DamagePacket {
                bytes: packet,
                attack_count: 2,
            },
            DamageFlags {
                shield: true,
                armor: true,
                saves: true,
                armor_filter: false,
            },
            &mut actual,
        )
        .unwrap();
        crate::support::native_expect! {
        scalar result.total => &case["damage"], "binary_compress_psi";
        scalar result.total => &after["last_amount"], "binary_compress_psi";
        bytes crate::support::record_bytes(&monsters) => &after["records"], "complete native helper records";
        }
        assert_eq!(before["party"], after["party"]);
        assert_eq!(word(&monsters[8].bytes, 0x110).unwrap(), 9);

        verify_receipts(case, &actual);
    }
}

#[test]
fn all_seven_reached_resolutions_match_native_records_cleanup_and_rng() {
    let fixture = fixture();
    let case = &fixture["cast_cases"][0];
    assert!(case.n::<bool>("complete_turn"));
    let mut state: Session =
        serde_json::from_value(fixture["foundation"]["session"].clone()).unwrap();
    let before = &case["before"];
    let cache = before.bytes("first_item_cache");
    let mut warriors = party(before.text("party"));
    let mut combat = state.combat.take().unwrap();
    combat.enemies.records = enemies(before.text("records"));
    combat.unavailable = before.n::<i8>("unavailable");
    let mut receipts_random = random(before);
    let receipts = case.array("rng");
    let mut cursor = 0;
    let mut impacts = vec![];
    let mut party_deaths = 0;
    let resolutions = case.array("resolutions");
    assert_eq!(resolutions.len(), 7);
    for resolution in resolutions {
        while receipts_random.draws < resolution.n::<u64>("draws_before") {
            let receipt = &receipts[cursor];
            cursor += 1;
            crate::support::expect(
                receipts_random.bound(receipt.n::<i16>("bound")),
                &receipt["result"],
            );
        }
        // The caller's selection and charges supply the actor at this resolution boundary.
        combat.enemies.records[6] = enemies(resolution.text("records_after"))[6].clone();
        let mut actual = receipts_random.clone();
        let packet = resolution.bytes("packet");
        let mask = words(resolution.text("mask"));
        let result = resolve_bodies_with_item_cache(
            &warriors,
            &combat.enemies.records,
            &mask,
            &DirectSpell {
                packet: &packet,
                level: resolution.n::<i16>("spell_id") % 1000 / 100 - 1,
                count: resolution.n::<i16>("count"),
                armor_filter: false,
                weapon_sound: 402,
            },
            &cache,
            &mut actual,
        )
        .unwrap();
        while receipts_random.draws < resolution.n::<u64>("draws_after") {
            let receipt = &receipts[cursor];
            cursor += 1;
            crate::support::expect(
                receipts_random.bound(receipt.n::<i16>("bound")),
                &receipt["result"],
            );
        }
        assert_eq!(actual, receipts_random);
        impacts.extend(audio(&result));
        let applied = words(resolution.text("applied_mask"));
        assert_eq!(
            result
                .targets
                .iter()
                .map(|(body, _)| *body)
                .collect::<Vec<_>>(),
            applied
                .iter()
                .enumerate()
                .filter_map(|(body, flag)| (*flag > 0).then_some(body))
                .collect::<Vec<_>>()
        );
        let defeated = result
            .targets
            .iter()
            .filter_map(|(body, result)| result.killed.then_some(*body as i16))
            .collect::<Vec<_>>();
        warriors = result.warriors;
        combat.enemies.records = result.enemies;
        for body in defeated {
            if body < 10 {
                party_deaths += 1;
            }
            lifecycle::kill_body(&mut combat, &mut warriors, body).unwrap();
        }
        let completed = BodySpellResult {
            warriors: warriors.clone(),
            enemies: combat.enemies.records.clone(),
            sounds: vec![],
            targets: vec![],
            race_excluded: vec![],
        };
        assert_records(
            &completed,
            resolution.text("party_after"),
            resolution.text("records_after"),
        );
    }
    assert_eq!(cursor, receipts.len());
    assert_eq!(receipts_random, random(&case["after"]));
    assert_eq!(receipts_random.draws - random(before).draws, 115);
    assert_eq!(impacts, [503, 503, 503, 402, 503, 108, 108]);
    assert_eq!(party_deaths, 1);
    crate::support::native_expect! {
    scalar combat.unavailable => &case["after"]["unavailable"], "binary_compress_psi";
    words &(combat.order) => &case["after"]["order"], "binary_compress_psi";
    words &(combat.map.tiles) => &case["after"]["map"], "binary_compress_psi";
    }
}
