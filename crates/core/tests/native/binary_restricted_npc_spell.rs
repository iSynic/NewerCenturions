use crate::{native_common, prelude::*, support};

use centurions_core::combat::spells::resolve_collision_with_item_cache;

use support::warriors as party;

use support::monsters as enemies;

use native_common::random_state as random;

use support::assert_bytes;

use native_common::body_spell_sounds as audio;

#[test]
fn npc18_restriction_and_native_immunity_match_three_complete_resolver_cases() {
    let fixture = support::fixture("restricted-npc-spell.json");
    let source = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../NewCenturions/TheNewCenturions/Data Files/Data ID"),
    )
    .unwrap();
    let cases = fixture.array("cases");

    for (index, case) in cases.iter().enumerate().skip(1) {
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
        native_common::assert_body_records(
            &result.warriors,
            &result.enemies,
            &after["party"],
            &after["records"],
        );
        assert!(result.targets.is_empty());
        assert_eq!(
            result.race_excluded,
            if index == 3 { vec![18] } else { vec![] }
        );
        support::expect_array(&(audio(&result)), &case["sounds"]);
        assert_eq!(actual, random(after));
        let mut receipts = random(before);
        support::replay_rng(&mut receipts, case.array("rng"));
        assert_eq!(receipts, actual);
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
        native_common::assert_body_records(
            &collision.warriors,
            &collision.enemies,
            &after["party"],
            &after["records"],
        );
        assert_eq!(collision.race_excluded, result.race_excluded);
        assert_eq!(audio(&collision), audio(&result));
        assert_eq!(collision_random, actual);
        assert_eq!(usage.used, original_usage);
    }
}

#[test]
fn reached_caller_all_four_resolutions_match_native_party_monsters_and_rng() {
    let fixture = support::fixture("restricted-npc-spell.json");
    let case = &fixture["cases"][0];
    let (_, impacts, deaths) = native_common::spell_sequence(&fixture, case, None);
    assert_eq!(impacts, [204, 204, 206]);
    assert_eq!(deaths, 0);
}
