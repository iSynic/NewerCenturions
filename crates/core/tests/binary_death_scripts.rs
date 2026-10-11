use centurions_content::{Warrior, combat::MonsterRecord};
use centurions_core::combat::death_scripts::{
    DeathEntry, DeathQueue, DeathStoryFrame, EncounterBattleFrame, cleanup_enemy,
};
use centurions_core::combat::{Combat, EnemySetup, SquadPlacement, TacticalMap};

fn decode(value: &str) -> Vec<u8> {
    (0..value.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&value[i..i + 2], 16).unwrap())
        .collect()
}
fn bytes(case: &serde_json::Value, key: &str) -> Vec<u8> {
    decode(case[key].as_str().unwrap())
}
fn words(value: &[u8]) -> Vec<i16> {
    value
        .as_chunks::<2>()
        .0
        .iter()
        .map(|v| i16::from_be_bytes(*v))
        .collect()
}
fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../evidence/fixtures/death-scripts.json"
    ))
    .unwrap()
}
fn input(case: &serde_json::Value) -> (Combat, Vec<Warrior>) {
    let mut order = vec![-1; 110];
    order[1] = 10;
    (
        Combat {
            battle: 0,
            map: TacticalMap {
                tiles: words(&bytes(case, "map_before")),
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: (0..6).map(|i| (40 + i, 40)).collect(),
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: vec![MonsterRecord {
                    bytes: bytes(case, "record_before"),
                }],
                positions: vec![(20, 20)],
            },
            order,
            cursor: 1,
            round: 0,
            death_queue: Default::default(),
            spell_fields: vec![0; 1200],
            field_used: vec![0; 128],
            attack_packet: Some(vec![0; 58]),
            attack_identity: Some([0; 3]),
            weapon_work: centurions_core::combat::weapon_work::WeaponWork::zero(),
            unavailable: 0,
            round_scripts: Default::default(),
        },
        bytes(case, "warriors_before")
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|w| Warrior {
                name: "fixture".into(),
                original_sha256: String::new(),
                bytes: w.to_vec(),
            })
            .collect(),
    )
}

#[test]
fn native_large_death_cleanup_matches_record_map_order_and_queue() {
    let fixture = fixture();
    for case in fixture["cases"].as_array().unwrap() {
        let (mut combat, mut warriors) = input(case);
        let result = cleanup_enemy(&mut combat, &mut warriors, 10).unwrap();
        assert_eq!(
            combat.enemies.records[0].bytes,
            bytes(case, "record_cleanup")
        );
        assert_eq!(combat.map.tiles, words(&bytes(case, "map_cleanup")));
        assert_eq!(combat.order, words(&bytes(case, "order_after")));
        assert_eq!(i64::from(result.increment_kills), case["kills_cleanup"]);
        if case["queued"].as_i64().unwrap() != 0 {
            let entry = result.entry.unwrap();
            assert_eq!(entry.bytes(), bytes(case, "queue_cleanup")[..14]);
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
    let authored = bytes(
        fixture,
        if script == 51 {
            "script51"
        } else {
            "script229"
        },
    );
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
    let fixture = fixture();
    for case in fixture["cases"].as_array().unwrap() {
        let (mut combat, mut warriors) = input(case);
        let result = cleanup_enemy(&mut combat, &mut warriors, 10).unwrap();
        let entry = result.entry.unwrap();
        let mut header = bytes(case, "parent_body");
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
        assert_eq!(header, bytes(case, "body_cleanup"));
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
        assert_eq!(header, bytes(case, "body_after"));
        assert_eq!(flags, bytes(case, "flags_after"));
        assert_eq!(
            queue.count,
            case["queue_count_after"].as_u64().unwrap() as usize
        );
        assert_eq!(
            i64::from(queue.pointer),
            case["queue_pointer_after"].as_i64().unwrap()
        );
        assert_eq!(
            queue
                .entries
                .iter()
                .flat_map(|e| e.bytes())
                .collect::<Vec<_>>(),
            bytes(case, "queue_after")
        );
        assert_eq!(
            warriors
                .iter()
                .flat_map(|w| &w.bytes)
                .copied()
                .collect::<Vec<_>>(),
            bytes(case, "warriors_after")
        );
    }
}

#[test]
fn unsupported_death_dimensions_leave_input_unchanged() {
    let fixture = fixture();
    let (mut combat, mut warriors) = input(&fixture["cases"][0]);
    combat.enemies.records[0].bytes[0x10..0x12].copy_from_slice(&i16::MIN.to_be_bytes());
    let before = (combat.clone(), warriors.clone());
    assert!(cleanup_enemy(&mut combat, &mut warriors, 10).is_err());
    assert_eq!((combat, warriors), before);
}
