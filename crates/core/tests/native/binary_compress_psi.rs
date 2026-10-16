use crate::prelude::*;

use centurions_core::combat::damage::DamageFlags;
use centurions_core::combat::damage::DamagePacket;
use centurions_core::combat::damage::resolve_damage;

use crate::support::warriors as party;

use crate::support::monsters as enemies;

use crate::native_common::random_state as random;

use crate::native_common::body_spell_sounds as audio;

fn verify_receipts(case: &Value, actual: &RandomState) {
    let mut expected = random(&case["before"]);
    crate::support::replay_rng(&mut expected, case.array("rng"));
    assert_eq!(&expected, actual);
    assert_eq!(expected, random(&case["after"]));
}

#[test]
fn three_complete_compress_resolutions_match_party_npc_records_rng_and_audio() {
    let fixture = crate::support::fixture("compress-psi.json");
    let cases = fixture.array("cast_cases");

    for case in cases.iter().skip(1) {
        let before = &case["before"];
        let after = &case["after"];
        let packet = before.bytes("packet");
        let mask = words(before.text("mask"));
        let warriors = party(before.text("party"));
        let monsters = enemies(before.text("records"));
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
        crate::native_common::assert_body_records(
            &result.warriors,
            &result.enemies,
            &after["party"],
            &after["records"],
        );
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
            assert_eq!(result.targets[0].1.damage, 0);
        }
    }
}

#[test]
fn all_eight_mass_helper_controls_match_native_signed_products_and_complete_records() {
    let fixture = crate::support::fixture("compress-psi.json");
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

        verify_receipts(case, &actual);
    }
}

#[test]
fn all_seven_reached_resolutions_match_native_records_cleanup_and_rng() {
    let fixture = crate::support::fixture("compress-psi.json");
    let case = &fixture["cast_cases"][0];
    let (combat, impacts, party_deaths) =
        crate::native_common::spell_sequence(&fixture, case, Some(6));
    assert_eq!(impacts, [503, 503, 503, 402, 503, 108, 108]);
    assert_eq!(party_deaths, 1);
    crate::support::native_expect! {
    scalar combat.unavailable => &case["after"]["unavailable"], "compress unavailable";
    words combat.order => &case["after"]["order"], "compress order";
    words combat.map.tiles => &case["after"]["map"], "compress map";
    }
}
