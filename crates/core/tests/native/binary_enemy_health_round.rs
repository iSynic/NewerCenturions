use crate::prelude::*;

use centurions_core::combat::lifecycle;
use centurions_core::combat::natural_conditions::enemy_drain_tick;

fn fixture() -> Value {
    crate::support::fixture("enemy-health-round.json")
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
    let cases = fixture.array("cases");

    for (index, case) in cases.iter().enumerate() {
        let records = decode(&case["before"]["records"]);
        let mut record = records[288..576].to_vec();
        decrement_conditions(&mut record);
        let before = record.clone();
        let tick = enemy_drain_tick(&record).unwrap();
        assert_eq!(record, before, "pure helper, case {index}");
        let native_tick = case.array("results").iter().find(|result| result[1] == 29);
        assert_eq!(tick.is_some(), native_tick.is_some(), "case {index}");
        if let Some(tick) = tick {
            let after = decode(&case["after"]["records"]);
            assert_eq!(tick.health, word(&after[288..576], 0x110).unwrap());
            assert_eq!(tick.killed, !case.array("kills").is_empty());
            assert_eq!(
                i64::from(word(&record, 0x110).unwrap().wrapping_sub(tick.health)),
                native_tick.unwrap()[2].as_i64().unwrap(),
                "native amount, case {index}"
            );
        }
        assert_eq!(case["draws"], serde_json::json!([]));
    }
    assert!(enemy_drain_tick(&[0; 287]).is_err());
}

#[test]
fn enemy_health_drain_and_death_cleanup_match_complete_native_monster_slices() {
    let fixture = fixture();
    let foundation: Combat =
        serde_json::from_value(fixture["foundation"]["combat"].clone()).unwrap();
    let mut verified = 0;
    for case in fixture.array("cases").iter() {
        if case.n::<bool>("full") {
            continue;
        }
        let before = &case["before"];
        let after = &case["after"];
        let mut combat = foundation.clone();
        combat.map.tiles = before.words("map");
        combat.order = before.words("order");
        combat.spell_fields = decode(&before["fields"]);
        combat.field_used = decode(&before["used"]);
        combat.enemies.records = crate::support::monsters(&decode(&before["records"]));
        let mut warriors: Vec<_> = crate::support::warriors(&decode(&before["party"]));
        let random = RandomState::new(before.n::<u32>("seed"));
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
        crate::support::native_expect! {
        bytes &(crate::support::record_bytes(&warriors)) => &after["party"], "binary_enemy_health_round";
        bytes &(crate::support::record_bytes(&combat
                        .enemies.records)) => &after["records"], "binary_enemy_health_round";
        words combat.map.tiles => &after["map"], "map, case {index}";
        words combat.order => &after["order"], "order, case {index}";
        bytes &(combat.spell_fields) => &after["fields"], "binary_enemy_health_round";
        bytes &(combat.field_used) => &after["used"], "binary_enemy_health_round";
        }
        assert_eq!(combat.death_queue, saved_combat.death_queue);
        assert_eq!(combat.enemies.positions, saved_combat.enemies.positions);
        assert_eq!(combat.squad, saved_combat.squad);
        assert_eq!(combat.unavailable, saved_combat.unavailable);
        assert_eq!(combat.attack_packet, saved_combat.attack_packet);
        assert_eq!(combat.attack_identity, saved_combat.attack_identity);
        assert_eq!(combat.round_scripts, saved_combat.round_scripts);
        crate::support::native_expect! {
        scalar (sounds) => &case["sounds"], "binary_enemy_health_round";
        scalar random.seed => &after["seed"], "binary_enemy_health_round";
        scalar random.draws => &after["draws"], "binary_enemy_health_round";
        }

        verified += 1;
    }
    assert_eq!(verified, 208);
}
