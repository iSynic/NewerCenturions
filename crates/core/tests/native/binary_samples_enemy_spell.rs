use crate::prelude::*;

use crate::native_common::snapshot_party as party;

use crate::native_common::snapshot_enemies as enemies;

use crate::native_common::random_state as random;

use crate::support::assert_bytes;

#[test]
fn samples_party_race_exclusion_matches_complete_native_records_rng_and_ordered_audio() {
    let fixture = crate::support::fixture("samples-enemy-spell.json");
    let mut tested = 0;
    for (index, case) in fixture.array("cases").iter().enumerate() {
        if case.n::<bool>("complete_turn") {
            continue;
        }
        let before = &case["before"];
        let after = &case["after"];
        let mask = words(before.text("mask"));
        if mask[10..].iter().any(|flag| *flag > 0) {
            continue;
        }
        tested += 1;
        let warriors = party(before);
        let monsters = enemies(before);
        assert_eq!(warriors.len(), 6);
        assert_eq!(monsters.len(), 14);
        let packet = before.bytes("packet");
        assert_eq!(word(&packet, 50).unwrap(), 0x2000);
        assert_eq!(word(&warriors[3].bytes, 0x32).unwrap(), 3);
        let work = before.bytes("weapon_work");
        let ammunition = before.bytes("ammo_work");
        let mut actual_random = random(before);
        let result = resolve_bodies(
            &warriors,
            &monsters,
            &mask,
            &DirectSpell {
                packet: &packet,
                level: 1,
                count: before.n::<i16>("count"),
                armor_filter: word(&work, 0x88)
                    .unwrap()
                    .wrapping_add(word(&ammunition, 0x88).unwrap())
                    != 0,
                weapon_sound: word(&work, 0x0e).unwrap(),
            },
            &mut actual_random,
        )
        .unwrap();
        crate::support::native_expect! {
        bytes crate::support::record_bytes(&result.warriors) => &after["party"], &format!("case {index} complete party");
        bytes crate::support::record_bytes(&result.enemies) => &after["records"], &format!("case {index} complete monsters");
        }
        assert_eq!(result.race_excluded, [3]);
        let applied = words(case["resolutions"][0]["applied_mask"].as_str().unwrap());
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
        assert_bytes(
            &result.warriors[3].bytes,
            &warriors[3].bytes,
            "excluded warrior",
        );
        assert_eq!(actual_random, random(after));
        let mut receipts = random(before);
        crate::support::replay_rng(&mut receipts, case.array("rng"));
        assert_eq!(receipts, actual_random);

        let mut ordered = result
            .targets
            .iter()
            .map(|(body, outcome)| (*body, outcome.sounds.clone()))
            .collect::<Vec<_>>();
        ordered.extend(result.race_excluded.iter().map(|body| (*body, vec![651])));
        ordered.sort_by_key(|(body, _)| *body);
        let actual_sounds = result
            .sounds
            .into_iter()
            .chain(ordered.into_iter().flat_map(|(_, sounds)| sounds))
            .collect::<Vec<_>>();
        crate::support::expect_array(&(actual_sounds), &case["sounds"]);
        assert_eq!(actual_sounds, [204, 204, 204, 651, 204, 204]);
    }
    assert_eq!(tested, 2);
}

#[test]
fn restricted_monster_targets_remain_diagnostic_without_native_alias_memory() {
    let fixture = crate::support::fixture("samples-enemy-spell.json");
    let mut tested = 0;
    for case in fixture.array("cases") {
        if case.n::<bool>("complete_turn") {
            continue;
        }
        let before = &case["before"];
        let mask = words(before.text("mask"));
        if !mask[10..].iter().any(|flag| *flag > 0) {
            continue;
        }
        tested += 1;
        let warriors = party(before);
        let monsters = enemies(before);
        let packet = before.bytes("packet");
        let mut actual_random = random(before);
        let result = resolve_bodies(
            &warriors,
            &monsters,
            &mask,
            &DirectSpell {
                packet: &packet,
                level: 1,
                count: 1,
                armor_filter: false,
                weapon_sound: 0,
            },
            &mut actual_random,
        );
        assert!(matches!(result, Err(RuleError::Unsupported { .. })));
        assert_eq!(actual_random, random(before));
        let mut usage = FieldUsage {
            slot: 7,
            used: vec![0; 128],
        };
        usage.used[8] = 1;
        let previous_usage = usage.used.clone();
        let collision = resolve_collision(
            &warriors,
            &monsters,
            &mask,
            &DirectSpell {
                packet: &packet,
                level: 1,
                count: 1,
                armor_filter: false,
                weapon_sound: 0,
            },
            &mut usage,
            &mut actual_random,
        );
        assert!(matches!(collision, Err(RuleError::Unsupported { .. })));
        assert_eq!(actual_random, random(before));
        assert_eq!(usage.used, previous_usage);
        assert_eq!(usage.slot, 7);
        crate::support::native_expect! {
        bytes crate::support::record_bytes(&warriors) => &before["party"], "unchanged party inputs";
        bytes crate::support::record_bytes(&monsters) => &before["records"], "unchanged monster inputs";
        }
    }
    assert_eq!(tested, 2);
}

#[test]
fn collision_race_gate_precedes_usage_and_unsupported_restrictions_remain_atomic() {
    let fixture = crate::support::fixture("samples-enemy-spell.json");
    let before = &fixture["cases"][1]["before"];
    let warriors = party(before);
    let monsters = enemies(before);
    let packet = before.bytes("packet");
    let mut mask = vec![0; 110];
    mask[3] = 1;
    let context = DirectSpell {
        packet: &packet,
        level: 1,
        count: 1,
        armor_filter: false,
        weapon_sound: 0,
    };
    let mut usage = FieldUsage {
        slot: 7,
        used: vec![0; 128],
    };
    usage.used[8] = 1;
    let previous_usage = usage.used.clone();
    let mut actual_random = random(before);
    let result = resolve_collision(
        &warriors,
        &monsters,
        &mask,
        &context,
        &mut usage,
        &mut actual_random,
    )
    .unwrap();
    assert!(result.targets.is_empty());
    assert_eq!(result.race_excluded, [3]);
    assert_eq!(usage.used, previous_usage);
    assert_eq!(usage.slot, 7);
    for (actual, expected) in result.warriors.iter().zip(&warriors) {
        assert_bytes(&actual.bytes, &expected.bytes, "excluded collision party");
    }
    for (actual, expected) in result.enemies.iter().zip(&monsters) {
        assert_bytes(
            &actual.bytes,
            &expected.bytes,
            "excluded collision monsters",
        );
    }
    let mut prefix = random(before);
    let receipt = &fixture["cases"][4]["rng"][0];
    assert_eq!(prefix.bound(receipt.n::<i16>("bound")), 1);
    assert_eq!(actual_random, prefix);

    for restriction in [0x4000_i16, i16::MIN, -1] {
        let mut unsupported = packet.clone();
        unsupported[50..52].copy_from_slice(&restriction.to_be_bytes());
        let mut actual_random = random(before);
        let result = resolve_bodies(
            &warriors,
            &monsters,
            &mask,
            &DirectSpell {
                packet: &unsupported,
                ..context
            },
            &mut actual_random,
        );
        assert!(matches!(result, Err(RuleError::Unsupported { .. })));
        assert_eq!(actual_random, random(before));
    }
    for race in [0i16, 17, -1] {
        let mut unsupported = warriors.clone();
        unsupported[3].bytes[0x32..0x34].copy_from_slice(&race.to_be_bytes());
        let mut actual_random = random(before);
        let result = resolve_collision(
            &unsupported,
            &monsters,
            &mask,
            &context,
            &mut usage,
            &mut actual_random,
        );
        assert!(matches!(result, Err(RuleError::Unsupported { .. })));
        assert_eq!(actual_random, random(before));
        assert_eq!(usage.used, previous_usage);
    }
    let mut unrepresented = vec![0; 110];
    unrepresented[109] = 1;
    let mut actual_random = random(before);
    let result = resolve_bodies(
        &warriors,
        &monsters,
        &unrepresented,
        &context,
        &mut actual_random,
    );
    assert!(matches!(result, Err(RuleError::Unsupported { .. })));
    assert_eq!(actual_random, random(before));
}
