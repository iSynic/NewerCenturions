use crate::prelude::*;

use crate::support::items;

fn fixture() -> serde_json::Value {
    crate::support::fixture("endgame-weapons.json")
}
use crate::support::assert_bytes;
#[test]
fn required_endgame_enemy_preparation_matches_native_item_context_and_packet() {
    let fixture = fixture();

    crate::binary_endgame_weapons::compare_required_endgame_enemy_preparation_matches_native_item_context_and_packet(fixture);
}

pub(super) fn compare_required_endgame_enemy_preparation_matches_native_item_context_and_packet(
    fixture: Value,
) {
    let attacks = attacks();
    let cases = fixture.array("preparation_cases");

    for case in cases.iter() {
        let mut enemy = MonsterRecord {
            bytes: case.bytes("enemy_before"),
        };
        let prepared = enemy_weapons::prepare(
            &mut enemy,
            &case.bytes("weapon"),
            &case.bytes("ammo"),
            &attacks,
        )
        .unwrap();
        crate::support::native_expect! {
        bytes prepared.packet => &case["packet"], "case{index} packet";
        bytes enemy.bytes => &case["enemy_after"], "case{index} enemy";
        }
    }
}
#[test]
fn required_endgame_enemy_firearms_match_whole_party_and_native_randomness() {
    let fixture = fixture();

    crate::binary_endgame_weapons::compare_required_endgame_enemy_firearms_match_whole_party_and_native_randomness(fixture);
}

pub(super) fn compare_required_endgame_enemy_firearms_match_whole_party_and_native_randomness(
    fixture: Value,
) {
    let attacks = attacks();
    let cases = fixture.array("firearm_cases");

    for (index, case) in cases.iter().enumerate() {
        let mut party = case.bytes("warriors_before");
        let member = case.n::<usize>("member");
        let mut warrior = Warrior::read(&party[member * 3138..(member + 1) * 3138]).unwrap();
        let mut enemy = MonsterRecord {
            bytes: case.bytes("enemy_before"),
        };
        let prepared = enemy_weapons::prepare(
            &mut enemy,
            &case.bytes("weapon"),
            &case.bytes("ammo"),
            &attacks,
        )
        .unwrap();
        assert_eq!(prepared.packet, case.bytes("packet"), "case{index} packet");
        let mut random = RandomState::new(case.n::<u32>("seed"));
        let result =
            enemy_weapons::resolve(&mut warrior, &mut enemy, &prepared, &mut random, 0).unwrap();
        party[member * 3138..(member + 1) * 3138].copy_from_slice(&warrior.bytes);
        crate::support::native_expect! {
        bytes party => &case["warriors_after"], &format!("case{index} full party");
        bytes enemy.bytes => &case["enemy_after"], "case{index} enemy";
        scalar random.seed => &case["seed_after"], "binary_endgame_weapons";
        scalar random.draws => &case["rng_draws"], "binary_endgame_weapons";
        scalar (result.sounds) => &case["sounds"], "binary_endgame_weapons";
        }
    }
}
#[test]
fn required_endgame_default_melee_selection_matches_native_records_saves_and_both_attack_indices() {
    let fixture = fixture();

    crate::binary_endgame_weapons::compare_required_endgame_default_melee_selection_matches_native_records_saves_and_both_attack_indices(fixture);
}

pub(super) fn compare_required_endgame_default_melee_selection_matches_native_records_saves_and_both_attack_indices(
    fixture: Value,
) {
    let items = items();
    let cases = fixture.array("melee_cases");

    for (index, case) in cases.iter().enumerate() {
        let mut party = case.bytes("warriors_before");
        let member = case.n::<usize>("member");
        let mut warrior = Warrior::read(&party[member * 3138..(member + 1) * 3138]).unwrap();
        let mut enemy = MonsterRecord {
            bytes: case.bytes("enemy_before"),
        };
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let weapon = i16::from_be_bytes(enemy.bytes[0x28..0x2a].try_into().unwrap()) as usize;
        let (result, packet) = enemy_weapons::melee_with_packet(
            &mut warrior,
            &mut enemy,
            &items[weapon],
            &mut random,
            0,
            0,
            40 + member as i16,
            &attacks(),
            &[0; 58],
        )
        .unwrap();
        assert_bytes(
            &packet,
            &case.bytes("packet"),
            &format!("case{index} packet"),
        );
        party[member * 3138..(member + 1) * 3138].copy_from_slice(&warrior.bytes);
        crate::support::native_expect! {
        bytes party => &case["warriors_after"], &format!("case{index} party");
        bytes enemy.bytes => &case["enemy_after"], &format!("case{index} attacker");
        scalar random.seed => &case["seed_after"], "binary_endgame_weapons";
        scalar random.draws => &case["rng_draws"], "binary_endgame_weapons";
        }
        assert_eq!(
            case.n::<Vec<i16>>("sounds"),
            vec![if result.damage != 0 {
                i16::from_be_bytes(items[weapon][14..16].try_into().unwrap())
            } else {
                650
            }],
            "case{index} native melee hit/miss audio"
        );
        assert!(result.sounds.is_empty());
    }
}
#[test]
fn required_endgame_adjacent_melee_caller_matches_native_action_count_and_cost() {
    let fixture = fixture();

    crate::binary_endgame_weapons::compare_required_endgame_adjacent_melee_caller_matches_native_action_count_and_cost(fixture);
}

pub(super) fn compare_required_endgame_adjacent_melee_caller_matches_native_action_count_and_cost(
    fixture: Value,
) {
    let items = items();
    let cases = fixture.array("caller_cases");

    for (index, case) in cases.iter().enumerate() {
        let raw = case.bytes("warriors_before");
        let mut warriors: Vec<_> = crate::support::warriors(&raw);
        let mut combat = Combat {
            battle: 117,
            map: TacticalMap {
                tiles: case.words("map_before"),
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: vec![(21, 20), (41, 40), (42, 40), (43, 40), (44, 40), (45, 40)],
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: vec![MonsterRecord {
                    bytes: case.bytes("enemy_before"),
                }],
                positions: vec![(20, 20)],
            },
            order: case.words("order_before"),
            ..crate::support::battle()
        };
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let events = runtime::check_neighbors_with_definitions(
            &mut combat,
            &mut warriors,
            &mut random,
            10,
            &items,
        )
        .unwrap();
        let actual: Vec<_> = crate::support::record_bytes(&warriors);
        crate::support::native_expect! {
        bytes actual => &case["warriors_after"], &format!("case{index} party");
        bytes combat.enemies.records[0].bytes => &case["enemy_after"], &format!("case{index} attacker");
        }
        assert_eq!(combat.map.tiles, case.words("map_after"), "case{index} map");
        assert_eq!(combat.order, case.words("order_after"), "case{index} order");
        crate::support::native_expect! {
        scalar random.seed => &case["seed_after"], "binary_endgame_weapons";
        scalar random.draws => &case["rng_draws"], "binary_endgame_weapons";
        scalar (crate::support::event_sounds(&events)) => &case["sounds"], "binary_endgame_weapons";
        }
        let resolved: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::AttackResolved {
                    attacker, target, ..
                } => Some((*attacker, *target)),
                _ => None,
            })
            .collect();
        assert_eq!(
            resolved,
            case.array("calls")
                .iter()
                .map(|call| (
                    call[0].as_i64().unwrap() as i16,
                    call[1].as_i64().unwrap() as i16
                ))
                .collect::<Vec<_>>(),
            "case{index} ordered attack calls"
        );
    }
}
#[test]
fn required_endgame_enemy_firearms_match_native_friendly_body_targets() {
    let fixture = fixture();

    crate::binary_endgame_weapons::compare_required_endgame_enemy_firearms_match_native_friendly_body_targets(fixture);
}

pub(super) fn compare_required_endgame_enemy_firearms_match_native_friendly_body_targets(
    fixture: Value,
) {
    let items = items();
    let attacks = attacks();
    let cases = fixture.array("body_cases");

    for (index, case) in cases.iter().enumerate() {
        let mut enemy = MonsterRecord {
            bytes: case.bytes("enemy_before"),
        };
        let mut target = MonsterRecord {
            bytes: case.bytes("target_before"),
        };
        let id = i16::from_be_bytes(enemy.bytes[0x26..0x28].try_into().unwrap());
        let ammo = enemy_weapons::ammo_id(&enemy, case.n::<i16>("context")).unwrap();
        let prepared = enemy_weapons::prepare(
            &mut enemy,
            &items[id as usize],
            &items[ammo as usize],
            &attacks,
        )
        .unwrap();
        assert_bytes(
            &prepared.packet,
            &case.bytes("packet"),
            &format!("body{index} packet"),
        );
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result =
            spells::resolve_enemy_firearm_body(&mut target, &enemy, &prepared, 0, &mut random)
                .unwrap();
        crate::support::native_expect! {
        bytes target.bytes => &case["target_after"], &format!("body{index} target");
        bytes enemy.bytes => &case["enemy_after"], &format!("body{index} attacker");
        scalar random.seed => &case["seed_after"], "binary_endgame_weapons";
        scalar random.draws => &case["rng_draws"], "binary_endgame_weapons";
        scalar (result.sounds) => &case["sounds"], "binary_endgame_weapons";
        }
    }
}
use crate::native_common::raw_word as word;

#[test]
fn required_endgame_ranged_to_natural_fallback_matches_original_full_records_and_randomness() {
    let fixture: serde_json::Value = crate::support::fixture("endgame-weapons.json");
    let cases = fixture.array("natural_cases");

    for (index, case) in cases.iter().enumerate() {
        let mut party = case.bytes("warriors_before");
        let member = case.n::<usize>("member");
        let mut warrior = Warrior::read(&party[member * 3138..(member + 1) * 3138]).unwrap();
        let mut enemy = MonsterRecord {
            bytes: case.bytes("enemy_before"),
        };
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let (result, packet) = centurions_core::combat::attack::natural_attack_with_packet(
            &mut warrior,
            &mut enemy,
            case.n::<usize>("index"),
            &mut random,
            &std::fs::read(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../NewCenturions/TheNewCenturions/Data Files/Data S"
            ))
            .unwrap(),
            &[0; 58],
        )
        .unwrap();
        assert_eq!(packet, case.bytes("packet"), "case{index} packet");
        party[member * 3138..(member + 1) * 3138].copy_from_slice(&warrior.bytes);
        crate::support::native_expect! {
        bytes party => &case["warriors_after"], "case{index} party";
        bytes enemy.bytes => &case["enemy_after"], "case{index} enemy";
        scalar random.seed => &case["seed_after"], "binary_endgame_weapons";
        scalar random.draws => &case["rng_draws"], "binary_endgame_weapons";
        scalar (result
                        .sounds
                        .into_iter()
                        .chain([centurions_core::combat::ally_turns::result_audio(
                            &enemy,
                            0,
                            result.damage,
                            0,
                        )
                        .unwrap()])
                        .collect::<Vec<_>>()) => &case["sounds"], "binary_endgame_weapons";
        }
    }
}

#[test]
fn required_endgame_firearm_whole_caller_matches_native_cast_order_records_and_work_context() {
    let f: serde_json::Value = crate::support::fixture("endgame-weapons.json");

    let content = crate::support::content();
    let mission = content.map(0).unwrap();
    let cases = f.array("full_caller_cases");

    for (index, case) in cases.iter().enumerate() {
        let before_records = case.bytes("records_before");
        let mut warriors: Vec<_> = crate::support::warriors(&case["warriors_before"]);
        let position_words = case.words("warrior_positions");
        let mut positions = vec![(20, 20)];
        if before_records.len() > 288 {
            positions.push((24, 20));
        }
        let mut combat = Combat {
            battle: 129,
            map: TacticalMap {
                tiles: case.words("map_before"),
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: position_words
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|p| (p[0], p[1]))
                    .collect(),
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: crate::support::monsters(&before_records),
                positions,
            },
            order: case.words("order_before"),
            attack_packet: Some(case.bytes("packet_before")),
            attack_identity: Some(case.n::<Vec<i16>>("identity_before").try_into().unwrap()),
            ..crate::support::battle()
        };
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let mut work = runtime::EnemyWork {
            item_context: Some(case.n::<i16>("item_context_before")),
            attack_count: case.n::<i16>("attack_count_before"),
            packet: combat.attack_packet.clone(),
            identity: combat.attack_identity,
            ..runtime::EnemyWork::default()
        };
        let events = runtime::enemy_action_with_work(
            &mut combat,
            &mut warriors,
            mission,
            &mut random,
            &content.items,
            &mut work,
        )
        .unwrap_or_else(|error| {
            panic!(
                "case{index} MD{} variant{}: {error:?}",
                case["monster_id"], case["variant"]
            )
        });
        let actual_party: Vec<_> = crate::support::record_bytes(&warriors);
        let actual_records: Vec<_> = crate::support::record_bytes(&combat.enemies.records);
        crate::support::native_expect! {
        bytes actual_party => &case["warriors_after"], &format!("case{index} party");
        bytes actual_records => &case["records_after"], &format!("case{index} enemies");
        }
        assert_eq!(work.packet, combat.attack_packet, "case{index} packet work");
        assert_eq!(
            work.identity, combat.attack_identity,
            "case{index} identity work"
        );
        assert_eq!(
            combat.attack_packet,
            Some(case.bytes("packet_after")),
            "case{index} packet"
        );
        assert_eq!(
            combat.attack_identity,
            Some(case.n::<Vec<i16>>("identity_after").try_into().unwrap()),
            "case{index} identity"
        );
        assert_eq!(combat.map.tiles, case.words("map_after"), "case{index} map");
        assert_eq!(combat.order, case.words("order_after"), "case{index} order");
        let after_positions = case.words("positions_after");
        assert_eq!(
            combat.enemies.positions,
            after_positions
                .as_chunks::<2>()
                .0
                .iter()
                .map(|p| (p[0], p[1]))
                .collect::<Vec<_>>(),
            "case{index} positions"
        );
        crate::support::expect(combat.unavailable, &case["unavailable_after"]);
        assert_eq!(
            work.item_context,
            Some(case.n::<i16>("item_context_after")),
            "case{index} item context"
        );
        crate::support::native_expect! {
        scalar work.attack_count => &case["attack_count_after"], "binary_endgame_weapons";
        scalar random.seed => &case["seed_after"], "binary_endgame_weapons";
        scalar random.draws => &case["rng_draws"], "binary_endgame_weapons";
        scalar (crate::support::event_sounds(&events)) => &case["sounds"], "binary_endgame_weapons";
        }
        let mut expected_hits = vec![];
        let mut prior_party = case.bytes("warriors_before");
        let mut prior_records = before_records;
        for resolution in case.array("resolutions") {
            let no_human_warrior = prior_party
                .as_chunks::<3138>()
                .0
                .iter()
                .all(|warrior| warrior[0x9df] == 0 || word(warrior, 0x10) != 0);
            if resolution["kind"] == "natural" && no_human_warrior {
                assert_eq!(
                    resolution["draws_before"], resolution["draws_after"],
                    "case{index} native attack2 early return"
                );
                prior_party = resolution.bytes("party_after");
                prior_records = resolution.bytes("records_after");
                continue;
            }
            let mask = if resolution["kind"] == "natural" {
                let mut mask = vec![0; 110];
                mask[resolution.n::<usize>("target")] = 1;
                mask
            } else {
                assert_eq!(resolution["count"], 1, "case{index} native caller count");
                resolution.words("mask")
            };
            let after_party = resolution.bytes("party_after");
            let after_records = resolution.bytes("records_after");
            for (body, selected) in mask.iter().enumerate() {
                if *selected == 0 {
                    continue;
                }
                let (before, after, hp) = if body < 6 {
                    (
                        &prior_party[body * 3138..(body + 1) * 3138],
                        &after_party[body * 3138..(body + 1) * 3138],
                        0x50,
                    )
                } else if body >= 10 && body < 10 + combat.enemies.records.len() {
                    (
                        &prior_records[(body - 10) * 288..(body - 9) * 288],
                        &after_records[(body - 10) * 288..(body - 9) * 288],
                        0x110,
                    )
                } else {
                    panic!("case{index} unexpected selected body{body}")
                };
                expected_hits.push((
                    resolution["attacker"].as_i64().unwrap_or(10) as i16,
                    body as i16,
                    word(before, hp).wrapping_sub(word(after, hp)),
                    word(after, hp) < 1,
                ));
            }
            prior_party = after_party;
            prior_records = after_records;
        }
        let actual_hits = events
            .iter()
            .filter_map(|event| match event {
                Event::AttackResolved {
                    attacker,
                    target,
                    damage,
                    defeated,
                } => Some((*attacker, *target, *damage, *defeated)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(actual_hits, expected_hits, "case{index} ordered casts");
        if let Some(prelude) = case["prelude"].as_object() {
            assert_eq!(
                prelude["item_context_after"], 0,
                "case{index} native natural reset"
            );
        }
    }
}
