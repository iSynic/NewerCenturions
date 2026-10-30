use crate::{prelude::*, support};

use centurions_core::combat::lifecycle;
use centurions_core::combat::natural_conditions::enemy_drain_tick;

fn decrement_conditions(record: &mut [u8]) {
    for value in &mut record[0xb2..0xda] {
        if (*value as i8) > 0 {
            *value -= 1;
        }
    }
}

#[test]
fn enemy_health_drain_and_death_cleanup_match_complete_native_monster_slices() {
    let fixture = support::fixture("enemy-health-round.json");
    let foundation: Combat =
        serde_json::from_value(fixture["foundation"]["combat"].clone()).unwrap();
    let mut verified = 0;
    for case in fixture.array("cases").iter() {
        if case.n::<bool>("full") {
            let raw = decode(&case["before"]["records"]);
            let mut record = raw[MONSTER_BYTES..576].to_vec();
            decrement_conditions(&mut record);
            let tick = enemy_drain_tick(&record).unwrap();
            let native_tick = case.array("results").iter().find(|result| result[1] == 29);
            assert_eq!(tick.is_some(), native_tick.is_some());
            if let Some(tick) = tick {
                assert_eq!(
                    tick.health,
                    word(
                        &decode(&case["after"]["records"])[MONSTER_BYTES..576],
                        0x110
                    )
                    .unwrap()
                );
                assert_eq!(tick.killed, !case.array("kills").is_empty());
            }
            continue;
        }
        let before = &case["before"];
        let after = &case["after"];
        let mut combat = foundation.clone();
        combat.field.tiles = before.words("map");
        combat.battle_order = before.words("order");
        combat.spell_queue = decode(&before["fields"]);
        combat.collide_check = decode(&before["used"]);
        combat.roster.records = support::monsters(&decode(&before["records"]));
        let mut warriors: Vec<_> = support::warriors(&decode(&before["party"]));
        let random = RandomState::new(before.n::<u32>("seed"));
        let saved_combat = combat.clone();
        decrement_conditions(&mut combat.roster.records[1].bytes);
        combat.roster.records[1].bytes[0x105] = 0;
        let mut sounds = vec![];
        if let Some(tick) = enemy_drain_tick(&combat.roster.records[1].bytes).unwrap() {
            combat.roster.records[1].bytes[0x110..0x112]
                .copy_from_slice(&tick.health.to_be_bytes());
            sounds.push(107);
            if tick.killed {
                sounds.push(word(&combat.roster.records[1].bytes, 0x6a).unwrap());
                lifecycle::kill_body(&mut combat, &mut warriors, 11).unwrap();
            }
        }
        combat.collide_check = spells::reset_used(&combat.collide_check).unwrap();
        assert!(combat.spell_queue.iter().all(|value| *value == 0));
        combat.roster.records[1].bytes[0x105] = 0;
        support::native_expect! {
            bytes &(support::record_bytes(&warriors)) => &after["party"], "binary_enemy_health_round";
            bytes &(support::record_bytes(&combat
                        .roster.records)) => &after["records"], "binary_enemy_health_round";
            words combat.field.tiles => &after["map"], "map, case {index}";
            words combat.battle_order => &after["order"], "order, case {index}";
            bytes &(combat.spell_queue) => &after["fields"], "binary_enemy_health_round";
            bytes &(combat.collide_check) => &after["used"], "binary_enemy_health_round";
        }
        assert_eq!(combat.todo_queue, saved_combat.todo_queue);
        assert_eq!(combat.roster.mon_pos, saved_combat.roster.mon_pos);
        assert_eq!(combat.squad, saved_combat.squad);
        assert_eq!(combat.kill_party, saved_combat.kill_party);
        assert_eq!(combat.spell_info, saved_combat.spell_info);
        assert_eq!(combat.cast_identity, saved_combat.cast_identity);
        assert_eq!(combat.round_macros, saved_combat.round_macros);
        support::native_expect! {
            scalar (sounds) => &case["sounds"], "binary_enemy_health_round";
            rng random => (&after["seed"], &after["draws"]), "binary_enemy_health_round";
        }

        verified += 1;
    }
    assert_eq!(verified, 208);
    let content = content();
    for case in support::fixture("enemy-psi-round.json").array("cases") {
        let mut combat = support::battle();
        combat.roster.records = vec![MonsterRecord {
            bytes: case.bytes("before"),
        }];
        combat.roster.mon_pos = vec![(20, 20)];
        let mut warriors = support::mission_session(&content).warriors;
        combat.squad.pos.fill((20, 21));
        let mut random = RandomState::new(12);
        runtime::reduce_enemy_round_with_fields(
            &mut combat,
            &mut warriors,
            &mut random,
            None,
            &mut 0,
            &mut [0; 6],
            (0, &mut 0),
        )
        .unwrap();
        assert_eq!(combat.roster.records[0].bytes, case.bytes("after"));
        assert_eq!(random.seed, case.n::<u32>("seed"));
        assert_eq!(random.draws, case.n::<u64>("draws"));
    }
}
