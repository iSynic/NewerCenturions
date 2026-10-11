use centurions_content::{Warrior, combat::MonsterRecord, word};
use centurions_core::{
    RandomState,
    combat::{Combat, lifecycle, natural_conditions::enemy_drain_tick, spells},
};
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../evidence/fixtures/enemy-health-round.json"
    ))
    .unwrap()
}

fn decode(value: &Value) -> Vec<u8> {
    let hex = value.as_str().unwrap();
    (0..hex.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&hex[offset..offset + 2], 16).unwrap())
        .collect()
}

fn words(value: &Value) -> Vec<i16> {
    decode(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|bytes| i16::from_be_bytes(*bytes))
        .collect()
}

fn decrement_conditions(record: &mut [u8]) {
    for value in &mut record[0xb2..0xda] {
        if (*value as i8) > 0 {
            *value -= 1;
        }
    }
}

#[test]
fn enemy_health_drain_matches_all_native_admission_and_signed_amount_cases() {
    let fixture = fixture();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 209);
    for (index, case) in cases.iter().enumerate() {
        let records = decode(&case["before"]["records"]);
        let mut record = records[288..576].to_vec();
        decrement_conditions(&mut record);
        let before = record.clone();
        let tick = enemy_drain_tick(&record).unwrap();
        assert_eq!(record, before, "pure helper, case {index}");
        let native_tick = case["results"]
            .as_array()
            .unwrap()
            .iter()
            .find(|result| result[1] == 29);
        assert_eq!(tick.is_some(), native_tick.is_some(), "case {index}");
        if let Some(tick) = tick {
            let after = decode(&case["after"]["records"]);
            assert_eq!(tick.health, word(&after[288..576], 0x110).unwrap());
            assert_eq!(tick.killed, !case["kills"].as_array().unwrap().is_empty());
            assert_eq!(
                i64::from(word(&record, 0x110).unwrap().wrapping_sub(tick.health)),
                native_tick.unwrap()[2].as_i64().unwrap(),
                "native amount, case {index}"
            );
            assert_eq!(case["sounds"][0], 107);
        }
        assert_eq!(case["draws"], serde_json::json!([]));
        assert_eq!(case["after"]["seed"], case["before"]["seed"]);
        assert_eq!(case["after"]["draws"], case["before"]["draws"]);
    }
    assert!(enemy_drain_tick(&[0; 287]).is_err());
}

#[test]
fn enemy_health_drain_and_death_cleanup_match_complete_native_monster_slices() {
    let fixture = fixture();
    let foundation: Combat =
        serde_json::from_value(fixture["foundation"]["combat"].clone()).unwrap();
    let mut verified = 0;
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        if case["full"].as_bool().unwrap() {
            continue;
        }
        let before = &case["before"];
        let after = &case["after"];
        let mut combat = foundation.clone();
        combat.map.tiles = words(&before["map"]);
        combat.order = words(&before["order"]);
        combat.spell_fields = decode(&before["fields"]);
        combat.field_used = decode(&before["used"]);
        combat.enemies.records = decode(&before["records"])
            .as_chunks::<288>()
            .0
            .iter()
            .map(|bytes| MonsterRecord {
                bytes: bytes.to_vec(),
            })
            .collect();
        let mut warriors: Vec<_> = decode(&before["party"])
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|bytes| Warrior::read(bytes).unwrap())
            .collect();
        let random = RandomState::new(before["seed"].as_u64().unwrap() as u32);
        let saved_combat = combat.clone();
        decrement_conditions(&mut combat.enemies.records[1].bytes);
        combat.enemies.records[1].bytes[0x105] = 0;
        let mut sounds = vec![];
        if let Some(tick) = enemy_drain_tick(&combat.enemies.records[1].bytes).unwrap() {
            combat.enemies.records[1].bytes[0x110..0x112]
                .copy_from_slice(&tick.health.to_be_bytes());
            sounds.push(107);
            if tick.killed {
                sounds.push(word(&combat.enemies.records[1].bytes, 0x6a).unwrap());
                lifecycle::kill_body(&mut combat, &mut warriors, 11).unwrap();
            }
        }
        combat.field_used = spells::reset_used(&combat.field_used).unwrap();
        assert!(combat.spell_fields.iter().all(|value| *value == 0));
        combat.enemies.records[1].bytes[0x105] = 0;
        assert_eq!(
            warriors
                .iter()
                .flat_map(|w| w.bytes.clone())
                .collect::<Vec<_>>(),
            decode(&after["party"]),
            "party, case {index}"
        );
        assert_eq!(
            combat
                .enemies
                .records
                .iter()
                .flat_map(|m| m.bytes.clone())
                .collect::<Vec<_>>(),
            decode(&after["records"]),
            "records, case {index}"
        );
        assert_eq!(combat.map.tiles, words(&after["map"]), "map, case {index}");
        assert_eq!(combat.order, words(&after["order"]), "order, case {index}");
        assert_eq!(combat.spell_fields, decode(&after["fields"]));
        assert_eq!(combat.field_used, decode(&after["used"]));
        assert_eq!(combat.death_queue, saved_combat.death_queue);
        assert_eq!(combat.enemies.positions, saved_combat.enemies.positions);
        assert_eq!(combat.squad, saved_combat.squad);
        assert_eq!(combat.unavailable, saved_combat.unavailable);
        assert_eq!(combat.attack_packet, saved_combat.attack_packet);
        assert_eq!(combat.attack_identity, saved_combat.attack_identity);
        assert_eq!(combat.round_scripts, saved_combat.round_scripts);
        assert_eq!(
            sounds,
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|id| id.as_i64().unwrap() as i16)
                .collect::<Vec<_>>(),
            "sounds, case {index}"
        );
        assert_eq!(random.seed, after["seed"].as_u64().unwrap() as u32);
        assert_eq!(random.draws, after["draws"].as_u64().unwrap());
        for key in [
            "positions",
            "party_positions",
            "references",
            "party_buffs",
            "cursor",
            "round",
            "death_count",
            "unavailable",
            "active_member",
        ] {
            assert_eq!(before[key], after[key], "{key}, case {index}");
        }
        verified += 1;
    }
    assert_eq!(verified, 208);
}
