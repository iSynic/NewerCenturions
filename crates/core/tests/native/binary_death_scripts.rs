use crate::{prelude::*, support};

use centurions_core::combat::death_scripts::DeathEntry;
use centurions_core::combat::death_scripts::DeathStoryFrame;
use centurions_core::combat::death_scripts::EncounterBattleFrame;
use centurions_core::combat::death_scripts::cleanup_enemy;

fn input(case: &serde_json::Value) -> (Combat, Vec<Warrior>) {
    let mut combat = support::battle();
    combat.map.tiles = case.words("map_before");
    combat.squad.positions = (0..6).map(|i| (40 + i, 40)).collect();
    combat.enemies.records = vec![MonsterRecord {
        bytes: case.bytes("record_before"),
    }];
    combat.enemies.positions = vec![(20, 20)];
    combat.order[1] = 10;
    (combat, support::warriors(&case["warriors_before"]))
}

#[test]
fn native_large_death_cleanup_matches_record_map_order_and_queue() {
    let fixture = support::fixture("death-scripts.json");
    for case in fixture.array("cases") {
        let (mut combat, mut warriors) = input(case);
        let result = cleanup_enemy(&mut combat, &mut warriors, 10).unwrap();
        assert_eq!(
            combat.enemies.records[0].bytes,
            case.bytes("record_cleanup")
        );
        assert_eq!(combat.map.tiles, words(&case.bytes("map_cleanup")));
        assert_eq!(combat.order, words(&case.bytes("order_after")));
        assert_eq!(i64::from(result.increment_kills), case["kills_cleanup"]);
        if case.n::<i64>("queued") != 0 {
            let entry = result.entry.unwrap();
            assert_eq!(entry.bytes(), case.bytes("queue_cleanup")[..14]);
        }
    }
}

fn story(
    fixture: &serde_json::Value,
    case: &serde_json::Value,
    header: &mut [u8],
    queue: &mut DeathQueue,
    input: (i16, i16),
    flags: &mut [u8],
    warriors: &mut [Warrior],
) {
    let (native_body, script) = input;
    let death_frame = DeathStoryFrame::capture(header, native_body).unwrap();
    let authored = fixture.bytes(if script == 51 {
        "script51"
    } else {
        "script229"
    });
    header[8..42].copy_from_slice(&authored[8..42]);
    if script == 229 {
        flags[31] = 1;
    } else {
        let battle_frame = EncounterBattleFrame::capture(header, 6).unwrap();
        if case["child_mutation"] == 1 {
            queue.reset_for_battle();
            header.fill(0x6a);
        }
        if case["child_abort"] == 0 {
            for warrior in warriors {
                warrior.bytes[0x9df] = 1;
            }
            assert_eq!(battle_frame.restore(header).unwrap(), 6);
        }
    }
    death_frame.restore(header).unwrap();
}

#[test]
fn native_queue_child_reset_and_parent_restore_match_composite_cases() {
    let fixture = support::fixture("death-scripts.json");
    for case in fixture.array("cases") {
        let (mut combat, mut warriors) = input(case);
        let result = cleanup_enemy(&mut combat, &mut warriors, 10).unwrap();
        let entry = result.entry.unwrap();
        let mut header = case.bytes("parent_body");
        let mut flags = vec![0; 128];
        let mut queue = DeathQueue {
            pointer: 0x12345678,
            ..Default::default()
        };
        if case["queued"] == 0 {
            story(
                &fixture,
                case,
                &mut header,
                &mut queue,
                (10, entry.script),
                &mut flags,
                &mut warriors,
            );
        } else {
            queue.enqueue(entry).unwrap();
        }
        assert_eq!(header, case.bytes("body_cleanup"));
        if case["queued"] != 0 {
            if case["second_entry"] == true {
                queue
                    .enqueue(DeathEntry {
                        monster: 60,
                        faction: 1,
                        script: 229,
                        x: 25,
                        y: 25,
                    })
                    .unwrap();
            }
            queue.begin();
            while let Some((index, entry)) = queue.current() {
                let saved: DeathQueue =
                    serde_json::from_str(&serde_json::to_string(&queue).unwrap()).unwrap();
                assert_eq!(saved, queue);
                story(
                    &fixture,
                    case,
                    &mut header,
                    &mut queue,
                    (-(index as i16), entry.script),
                    &mut flags,
                    &mut warriors,
                );
                queue.acknowledge().unwrap();
            }
            queue.finish();
        }
        assert_eq!(header, case.bytes("body_after"));
        assert_eq!(flags, case.bytes("flags_after"));
        support::native_expect! {
            scalar queue.count => &case["queue_count_after"], "binary_death_scripts";
            scalar i64::from(queue.pointer) => &case["queue_pointer_after"], "binary_death_scripts";
        }
        assert_eq!(
            queue
                .entries
                .iter()
                .flat_map(|e| e.bytes())
                .collect::<Vec<_>>(),
            case.bytes("queue_after")
        );
        assert_eq!(
            support::record_bytes(&warriors),
            case.bytes("warriors_after")
        );
    }
}
