use centurions_content::{Content, Warrior, combat::MonsterRecord, word};
use centurions_core::{
    RandomState,
    combat::{
        spell_conditions,
        spells::{self, BodySpellResult, DirectSpell, FieldUsage, RestrictionMemoryView},
    },
};
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../evidence/fixtures/npc-poison-psi.json"
    ))
    .unwrap()
}

fn bytes(fixture: &Value, value: &Value) -> Vec<u8> {
    let hex = if let Some(hex) = value.as_str() {
        hex
    } else {
        fixture["shared_rule_buffers"][value["shared_buffer_sha256"].as_str().unwrap()]
            .as_str()
            .unwrap()
    };
    let result: Vec<_> = hex
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect();
    if value.is_object() {
        assert_eq!(result.len() as u64, value["byte_length"].as_u64().unwrap());
    }
    result
}

fn words(fixture: &Value, value: &Value) -> Vec<i16> {
    bytes(fixture, value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| i16::from_be_bytes(*pair))
        .collect()
}

fn warriors(f: &Value, snapshot: &Value) -> Vec<Warrior> {
    bytes(f, &snapshot["party"])
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|record| Warrior::read(record).unwrap())
        .collect()
}

fn enemies(f: &Value, snapshot: &Value) -> Vec<MonsterRecord> {
    bytes(f, &snapshot["npcs"])
        .as_chunks::<288>()
        .0
        .iter()
        .map(|record| MonsterRecord {
            bytes: record.to_vec(),
        })
        .collect()
}

fn random(snapshot: &Value) -> RandomState {
    RandomState {
        seed: snapshot["seed"].as_u64().unwrap() as u32,
        draws: snapshot["draws"].as_u64().unwrap(),
    }
}

fn ordered_sounds(result: &BodySpellResult) -> Vec<i16> {
    let mut sounds: Vec<_> = result
        .targets
        .iter()
        .map(|(body, result)| (*body, result.sounds.clone()))
        .collect();
    sounds.extend(result.race_excluded.iter().map(|body| (*body, vec![651])));
    sounds.sort_by_key(|(body, _)| *body);
    result
        .sounds
        .iter()
        .copied()
        .chain(sounds.into_iter().flat_map(|(_, sounds)| sounds))
        .collect()
}

fn source_alias(body: usize, backup: &[u8], items: &[u8]) -> i16 {
    let address = 0x448aa0 + body * 3138 + 0x32;
    if (0x44ecb0..=0x45363a).contains(&address) {
        word(backup, address - 0x44ecb0).unwrap()
    } else {
        word(items, address - 0x4537fe).unwrap()
    }
}

fn source_excluded(packet: &[u8], alias: i16) -> bool {
    let bit = alias.wrapping_sub(1);
    let displacement = 50 + (i32::from(bit) >> 3);
    packet[displacement as usize] & (0x80 >> (bit as u16 & 7)) != 0
}

#[test]
fn all_164_native_npc_poison_resolutions_match_complete_records_rng_and_audio() {
    let f = fixture();
    let cases = f["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 164);
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let mut families = [0; 4];
    for (index, case) in cases.iter().enumerate() {
        let before = &case["before"];
        let after = &case["after"];
        let party = warriors(&f, before);
        let npcs = enemies(&f, before);
        assert_eq!(party.len(), 6);
        assert_eq!(npcs.len(), 100);
        for (slot, npc) in npcs.iter().enumerate() {
            if [0, 1, 2, 8].contains(&slot) {
                assert_eq!(word(&npc.bytes, 0x110).unwrap(), 100);
                assert_eq!(word(&npc.bytes, 0x112).unwrap(), 100);
                assert_eq!(word(&npc.bytes, 2).unwrap(), 0);
                assert_eq!(word(&npc.bytes, 4).unwrap(), 15);
            } else {
                assert_eq!(npc.bytes, vec![0; 288]);
            }
        }
        let packet = bytes(&f, &before["packet"]);
        assert_eq!(
            packet,
            spells::load_packet(
                &content.map(2).unwrap().combat.as_ref().unwrap().attacks,
                2206,
            )
            .unwrap()
        );
        let mask = words(&f, &before["mask"]);
        let count = case["count"].as_i64().unwrap() as i16;
        let backup = bytes(&f, &before["backup_warriors"]);
        let items = bytes(&f, &before["item_cache0"]);
        let editor = case["editor_control"].as_bool().unwrap();
        if editor {
            assert_eq!(
                &backup[..5 * 3138],
                &bytes(&f, &before["party"])[..5 * 3138]
            );
            assert!(backup[5 * 3138..].iter().all(|byte| *byte == 0));
        } else {
            assert_eq!(backup, vec![0; 18_828]);
        }
        assert_eq!(content.map(2).unwrap().item_cache, items);
        for (key, id) in [("item_work", 187), ("weapon_work", 0), ("ammo_work", 0)] {
            let mut original = content.items[id].clone();
            let class = word(&original, 26).unwrap().wrapping_abs();
            original[26..28].copy_from_slice(&class.to_be_bytes());
            assert_eq!(
                bytes(&f, &before[key]),
                original,
                "work {key}, case {index}"
            );
        }
        let mut source_records = bytes(&f, &before["npcs"]);
        let mut source_random = random(before);
        assert_eq!(source_random.bound(1), 1);
        let mut duration = 0_i16;
        for _ in 0..count {
            duration = duration.wrapping_add(source_random.bound(3));
        }
        assert_eq!(i64::from(duration), after["duration"].as_i64().unwrap());
        let mut expected_sounds = vec![];
        let mut expected_targets = vec![];
        let mut excluded = vec![];
        for (slot, npc) in npcs.iter().enumerate() {
            let body = slot + 10;
            if mask[body] <= 0 {
                continue;
            }
            let alias = source_alias(body, &backup, &items);
            assert_eq!(
                i64::from(alias),
                before[format!("alias{body}")].as_i64().unwrap()
            );
            if source_excluded(&packet, alias) {
                expected_sounds.push(651);
                excluded.push(body);
                continue;
            }
            if (npc.bytes[0x8c] as i8) >= 101 || word(&npc.bytes, 0xa).unwrap() >= 101 {
                continue;
            }
            let saved = source_random.bound(100) <= word(&npc.bytes, 0xa).unwrap();
            let immune = npc.bytes[0xc3..=0xc8].iter().any(|byte| *byte != 0);
            if saved || immune {
                continue;
            }
            let previous = i16::from(npc.bytes[0xb5] as i8);
            let total = previous.wrapping_add(duration);
            if previous >= 0 && total.wrapping_abs() < 125 {
                source_records[slot * 288 + 0xb5] = npc.bytes[0xb5].wrapping_add(duration as u8);
            }
            expected_targets.push(body);
            expected_sounds.push(430);
        }
        assert_eq!(
            source_records,
            bytes(&f, &after["npcs"]),
            "source whole records, case {index}"
        );
        assert_eq!(source_random, random(after), "source RNG, case {index}");
        assert_eq!(
            serde_json::to_value(&expected_sounds).unwrap(),
            case["sounds"]
        );
        let memory = if editor {
            RestrictionMemoryView::with_editor_backup(&items, None, &backup).unwrap()
        } else {
            RestrictionMemoryView::ordinary_startup(&items, None)
        };
        let weapon = bytes(&f, &before["weapon_work"]);
        let ammo = bytes(&f, &before["ammo_work"]);
        assert_eq!(word(&weapon, 136).unwrap(), 0);
        assert_eq!(word(&ammo, 136).unwrap(), 0);
        let mut actual_random = random(before);
        let result = spells::resolve_bodies_with_memory(
            &party,
            &npcs,
            &mask,
            &DirectSpell {
                packet: &packet,
                level: words(&f, &before["identity"])[1],
                count,
                armor_filter: false,
                weapon_sound: word(&weapon, 14).unwrap(),
            },
            memory,
            &mut actual_random,
        )
        .unwrap();
        assert_eq!(result.warriors, party, "whole party, case {index}");
        assert_eq!(
            result
                .enemies
                .iter()
                .flat_map(|npc| npc.bytes.iter().copied())
                .collect::<Vec<_>>(),
            source_records,
            "runtime whole records, case {index}"
        );
        assert_eq!(actual_random, random(after));
        assert_eq!(ordered_sounds(&result), expected_sounds);
        assert_eq!(result.race_excluded, excluded);
        assert_eq!(
            result
                .targets
                .iter()
                .map(|(body, _)| *body)
                .collect::<Vec<_>>(),
            expected_targets
        );
        assert!(
            result
                .targets
                .iter()
                .all(|(_, target)| target.damage == 0 && !target.killed)
        );
        let mut receipt = random(before);
        for draw in case["rng"].as_array().unwrap() {
            assert_eq!(
                receipt.bound(draw["bound"].as_i64().unwrap() as i16),
                draw["result"].as_i64().unwrap() as i16
            );
        }
        assert_eq!(receipt, actual_random);
        for (body, _) in &result.targets {
            let mut standalone = npcs[*body - 10].bytes.clone();
            let mut unchanged_random = random(before);
            spell_conditions::apply(&mut standalone, true, 4, duration, &mut unchanged_random)
                .unwrap();
            assert_eq!(standalone, result.enemies[*body - 10].bytes);
            assert_eq!(unchanged_random, random(before));
        }
        for (key, value) in before.as_object().unwrap() {
            if ![
                "markers",
                "seed",
                "draws",
                "selected_npc",
                "duration",
                "sound_count",
                "field_used",
                "npcs",
                "resistance_flag",
            ]
            .contains(&key.as_str())
            {
                assert_eq!(value, &after[key], "native retained {key}, case {index}");
            }
        }
        assert_eq!(case["native_return_word"], 0);
        families[match case["family"].as_str().unwrap() {
            "ordinary" => 0,
            "exact_limit" => 1,
            "resistance" => 2,
            "editor_owner" => 3,
            _ => panic!("unexpected native family"),
        }] += 1;
    }
    assert_eq!(families, [96, 24, 40, 4]);
}

#[test]
fn unsupported_npc_poison_contexts_preserve_full_input_records_rng_and_usage() {
    let f = fixture();
    let before = &f["cases"][0]["before"];
    let party = warriors(&f, before);
    let npcs = enemies(&f, before);
    let packet = bytes(&f, &before["packet"]);
    let mask = words(&f, &before["mask"]);
    let cache = bytes(&f, &before["item_cache0"]);
    let initial = random(before);
    for variant in 0..8 {
        let mut modified_packet = packet.clone();
        let mut modified_mask = mask.clone();
        let mut defended_party = party.clone();
        match variant {
            4 => modified_packet[19] = 7,
            5 => modified_mask[6] = 1,
            6 => {
                modified_mask[0] = 1;
                defended_party[0].bytes[0x9a..0x9c].copy_from_slice(&1_i16.to_be_bytes());
            }
            _ => {}
        }
        let context = DirectSpell {
            packet: &modified_packet,
            level: if variant == 0 { 2 } else { 1 },
            count: match variant {
                1 => 0,
                2 => 8,
                _ => 1,
            },
            armor_filter: variant == 3,
            weapon_sound: 0,
        };
        let mut actual = initial.clone();
        let memory = if variant == 7 {
            RestrictionMemoryView::item_cache_only(&cache)
        } else {
            RestrictionMemoryView::ordinary_startup(&cache, None)
        };
        assert!(
            spells::resolve_bodies_with_memory(
                &defended_party,
                &npcs,
                &modified_mask,
                &context,
                memory,
                &mut actual
            )
            .is_err(),
            "variant {variant}"
        );
        assert_eq!(actual, initial);
    }
    let mut usage = FieldUsage {
        slot: 0,
        used: bytes(&f, &before["field_used"]),
    };
    let saved_usage = usage.used.clone();
    let mut actual = initial.clone();
    assert!(
        spells::resolve_collision_with_item_cache(
            &party,
            &npcs,
            &mask,
            &DirectSpell {
                packet: &packet,
                level: 1,
                count: 1,
                armor_filter: false,
                weapon_sound: 0
            },
            &mut usage,
            &cache,
            &mut actual,
        )
        .is_err()
    );
    assert_eq!(usage.used, saved_usage);
    assert_eq!(actual, initial);
    assert_eq!(party, warriors(&f, before));
    assert_eq!(npcs, enemies(&f, before));
    assert!(RestrictionMemoryView::with_editor_backup(&cache, None, &[0; 18_827]).is_err());
}
