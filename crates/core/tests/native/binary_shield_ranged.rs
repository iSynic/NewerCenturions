use crate::prelude::*;

use centurions_core::combat::damage::DamageFlags;
use centurions_core::combat::damage::DamagePacket;
use centurions_core::combat::damage::resolve_damage;

use crate::native_common::snapshot_party as party;

use crate::native_common::snapshot_rng as random;

use crate::native_common::snapshot_ammunition as stock;

use crate::support::assert_bytes;

#[test]
fn two_native_charged_shots_match_complete_damage_records_rng_and_shield_audio() {
    let f = crate::support::fixture("shield-ranged.json");

    for case in f.array("shot_cases") {
        let before = &case["targeted"];
        let after = &case["fired"];
        let source = case.n::<i64>("source");
        let mut actual = random(before);
        let draws = &after.array("draws")[before.array("draws").len()..];
        let sites = &after.array("rng_sites")[before.array("rng_sites").len()..];
        assert_eq!(
            sites[..3],
            [
                Value::from("1384bc"),
                Value::from("1502ca"),
                Value::from("1503f0")
            ]
        );
        // The native duration and hit prefix precedes the recovered damage boundary.
        crate::support::replay_rng(&mut actual, &draws[..3]);
        let entry = after
            .array("native_calls")
            .iter()
            .find(|call| call["address"] == "15075a")
            .unwrap();
        assert_eq!(entry["arguments"], serde_json::json!([3, 1, 1, 1, 0]));
        crate::support::expect(actual.draws, &entry["draws"]);
        let mut warriors = party(before);
        let packet = before.bytes("packet");
        assert_eq!(packet, bytes(&case["resolutions"][0]["packet"]));
        let result = resolve_damage(
            &mut warriors[3].bytes,
            false,
            &DamagePacket {
                bytes: packet,
                attack_count: 1,
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
        let expected = party(after);
        assert_eq!(
            result.total,
            word(&party(before)[3].bytes, 0x50).unwrap() - word(&expected[3].bytes, 0x50).unwrap()
        );
        assert_eq!(result.total, if source == 12 { 11 } else { 21 });
        assert_eq!(result.components[0], result.total);
        assert!(result.components[1..].iter().all(|amount| *amount == 0));
        assert!(result.shield_sound);
        assert_eq!(word(&warriors[3].bytes, 0x26).unwrap(), 238);
        assert_eq!(word(&warriors[3].bytes, 0x160).unwrap(), 238);
        // HP and the actor's AP/MOV belong to the cast caller, outside resolvedamage.
        let mut target = expected[3].bytes.clone();
        target[0x50..0x52].copy_from_slice(&party(before)[3].bytes[0x50..0x52]);
        assert_bytes(&warriors[3].bytes, &target, "complete damage target");
        for member in [1, 2, 4, 5] {
            assert_bytes(
                &warriors[member].bytes,
                &expected[member].bytes,
                "uninvolved warrior",
            );
        }
        let mut actor = expected[0].bytes.clone();
        for offset in [0x3c, 0x40] {
            actor[offset..offset + 2].copy_from_slice(&party(before)[0].bytes[offset..offset + 2]);
        }
        assert_bytes(&warriors[0].bytes, &actor, "actor outside caller AP/MOV");
        assert_eq!(actual, random(after));
        let mut receipt = random(before);
        crate::support::replay_rng(&mut receipt, draws);
        assert_eq!(actual, receipt);
        let calls = after.array("native_calls");
        let shield: Vec<_> = calls
            .iter()
            .filter(|call| call["name"] == "Sound" && call["return"] == "150c22")
            .map(|call| call["arguments"][0].as_i64().unwrap())
            .collect();
        assert_eq!(shield, vec![206]);
        let events = after.array("ranged_events");
        let guard = events
            .iter()
            .find(|event| event["address"] == "150a26")
            .unwrap();
        let physical = if source == 12 { 12 } else { 24 };
        assert_eq!(
            guard["damage_components"],
            serde_json::json!([physical, 0, 0, 0, 0, 0, 0, 0, 0])
        );
        let write = events
            .iter()
            .find(|event| event["address"] == "150b30")
            .unwrap();
        assert_eq!(write["d6"], 238);
        assert_eq!(
            write["damage_components"],
            serde_json::json!([physical - 1, 0, 0, 0, 0, 0, 0, 0, 0])
        );
        assert_eq!(
            write.n::<u64>("draws") - guard.n::<u64>("draws"),
            physical as u64
        );
        let inventory_write = events
            .iter()
            .find(|event| event["address"] == "150b44")
            .unwrap();
        assert_eq!(inventory_write["charge"], 238);
        assert_eq!(inventory_write["draws"], write["draws"]);
    }
}

#[test]
fn native_pre_compaction_pistol_fit_restores_battery_cache_without_an_admission_guard() {
    let f = crate::support::fixture("shield-ranged.json");

    let case = &f["restore_cases"][0];
    let before = &case["before"];
    let after = &case["after"];
    let content = content();
    let mut warriors = party(before);
    let slot = warriors[3]
        .inventory()
        .unwrap()
        .iter()
        .position(|item| item.id() == 2 && item.0[9] == 1)
        .unwrap();
    let item = warriors[3].inventory().unwrap()[slot].clone();
    assert_eq!(word(&warriors[3].bytes, 0x126).unwrap(), 307);
    assert_eq!(word(&item.0, 2).unwrap(), 8);
    assert_eq!(word(&item.0, 4).unwrap(), 201);
    assert_eq!(word(&item.0, 6).unwrap(), 12);
    let initial_stock = stock(before);
    assert_eq!(initial_stock[1], 132);
    let mut actual = random(before);
    let result = firearm::fit_ammunition(
        &warriors[3],
        slot,
        &content,
        &initial_stock,
        false,
        &mut actual,
    )
    .unwrap();
    assert!(result.loaded);
    assert_eq!(result.loaded_item_field32, 2);
    assert_eq!(result.sounds, vec![404]);
    let native_sounds: Vec<i16> = after.array("sounds")[before.array("sounds").len()..]
        .iter()
        .map(|sound| sound.as_i64().unwrap() as i16)
        .collect();
    assert_eq!(result.sounds, native_sounds);
    warriors[3] = result.warrior;
    let expected = party(after);
    for (actual, expected) in warriors.iter().zip(expected) {
        assert_bytes(
            &actual.bytes,
            &expected.bytes,
            "complete firearm cache restoration party",
        );
    }
    assert_eq!(word(&warriors[3].bytes, 0x126).unwrap(), 201);
    let item = warriors[3].inventory().unwrap()[slot].clone();
    assert_eq!(word(&item.0, 2).unwrap(), 12);
    assert_eq!(word(&item.0, 4).unwrap(), 201);
    assert_eq!(result.ammunition, stock(after));
    assert_eq!(result.ammunition[1], 128);
    assert_eq!(actual, random(after));
    let mut receipt = random(before);
    crate::support::replay_rng(
        &mut receipt,
        &after.array("draws")[before.array("draws").len()..],
    );
    assert_eq!(receipt, actual);
    assert_eq!(actual.draws - random(before).draws, 1);
}
