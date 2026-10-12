use crate::prelude::*;

use centurions_core::combat::spells::resolve_collision_with_item_cache;
fn fixture() -> Value {
    crate::support::fixture("restricted-npc-spell.json")
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

#[test]
fn npc18_restriction_and_native_immunity_match_three_complete_resolver_cases() {
    let fixture = fixture();
    let source = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../NewCenturions/TheNewCenturions/Data Files/Data ID"),
    )
    .unwrap();
    let cases = fixture.array("cases");

    for (index, case) in cases.iter().enumerate().skip(1) {
        assert!(!case.n::<bool>("complete_turn"));
        let before = &case["before"];
        let after = &case["after"];
        let warriors = party(before.text("party"));
        let monsters = enemies(before.text("records"));
        let packet = before.bytes("packet");
        let cache = before.bytes("first_item_cache");
        assert_bytes(&cache, &source[..30_000], "authentic first item cache");
        assert_eq!(
            centurions_content::digest(&cache),
            before["first_item_cache_digest"]
        );
        assert_eq!(word(&cache, 81 * 150 + 2).unwrap(), 0);
        assert_eq!(monsters[8].bytes[0x90], 101);
        let mask = words(before.text("mask"));
        let context = DirectSpell {
            packet: &packet,
            level: 1,
            count: 3,
            armor_filter: false,
            weapon_sound: 0,
        };
        let mut actual = random(before);
        let result = resolve_bodies_with_item_cache(
            &warriors,
            &monsters,
            &mask,
            &context,
            &cache,
            &mut actual,
        )
        .unwrap();
        assert_records(&result, after.text("party"), after.text("records"));
        assert!(result.targets.is_empty());
        assert_eq!(
            result.race_excluded,
            if index == 3 { vec![18] } else { vec![] }
        );
        crate::support::expect_array(&(audio(&result)), &case["sounds"]);
        assert_eq!(actual, random(after));
        let mut receipts = random(before);
        for receipt in case.array("rng") {
            crate::support::expect(
                receipts.bound(receipt.n::<i16>("bound")),
                &receipt["result"],
            );
        }
        assert_eq!(receipts, actual);
        assert_eq!(actual.draws - random(before).draws, 1);
        for (key, value) in before.as_object().unwrap() {
            if !matches!(key.as_str(), "seed" | "draws") {
                assert_eq!(value, &after[key], "native unchanged {key} case{index}");
            }
        }
        let mut usage = FieldUsage {
            slot: 7,
            used: before.bytes("used"),
        };
        usage.used[8] = 1;
        let original_usage = usage.used.clone();
        let mut collision_random = random(before);
        let collision = resolve_collision_with_item_cache(
            &warriors,
            &monsters,
            &mask,
            &context,
            &mut usage,
            &cache,
            &mut collision_random,
        )
        .unwrap();
        assert_records(&collision, after.text("party"), after.text("records"));
        assert_eq!(collision.race_excluded, result.race_excluded);
        assert_eq!(audio(&collision), audio(&result));
        assert_eq!(collision_random, actual);
        assert_eq!(usage.used, original_usage);
    }
}

#[test]
fn reached_caller_all_four_resolutions_match_native_party_monsters_and_rng() {
    let fixture = fixture();
    let case = &fixture["cases"][0];
    assert!(case.n::<bool>("complete_turn"));
    let before = &case["before"];
    let cache = before.bytes("first_item_cache");
    let mut warriors = party(before.text("party"));
    let mut receipt_random = random(before);
    let receipts = case.array("rng");
    let mut receipt_index = 0;
    let mut impacts = vec![];
    let resolutions = case.array("resolutions");
    assert_eq!(resolutions.len(), 4);
    for resolution in resolutions {
        while receipt_random.draws < resolution.n::<u64>("draws_before") {
            let receipt = &receipts[receipt_index];
            receipt_index += 1;
            crate::support::expect(
                receipt_random.bound(receipt.n::<i16>("bound")),
                &receipt["result"],
            );
        }
        let mut actual_random = receipt_random.clone();
        let packet = resolution.bytes("packet");
        let mask = words(resolution.text("mask"));
        // Caller charging has already supplied these actor records at this boundary.
        let monsters = enemies(resolution.text("records_after"));
        let result = resolve_bodies_with_item_cache(
            &warriors,
            &monsters,
            &mask,
            &DirectSpell {
                packet: &packet,
                level: resolution.n::<i16>("spell_id") % 1000 / 100 - 1,
                count: resolution.n::<i16>("count"),
                armor_filter: false,
                weapon_sound: 402,
            },
            &cache,
            &mut actual_random,
        )
        .unwrap();
        while receipt_random.draws < resolution.n::<u64>("draws_after") {
            let receipt = &receipts[receipt_index];
            receipt_index += 1;
            crate::support::expect(
                receipt_random.bound(receipt.n::<i16>("bound")),
                &receipt["result"],
            );
        }
        assert_eq!(actual_random, receipt_random);
        assert_records(
            &result,
            resolution.text("party_after"),
            resolution.text("records_after"),
        );
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
        assert!(result.race_excluded.is_empty());
        impacts.extend(audio(&result));
        warriors = result.warriors;
    }
    assert_eq!(impacts, [204, 204, 206]);
    assert_eq!(receipt_index, receipts.len());
    assert_eq!(receipt_random, random(&case["after"]));
    assert_bytes(
        &crate::support::record_bytes(&warriors),
        &case["after"].bytes("party"),
        "final native resolver party",
    );
}
