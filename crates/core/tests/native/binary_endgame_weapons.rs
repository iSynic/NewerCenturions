use crate::{native_common, prelude::*, support};
use native_common::raw_word as word;
use support::items;

#[test]
fn endgame_weapon_corpus_matches_original() {
    corpus("endgame-weapons.json", true);
}

pub(super) fn corpus(name: &str, native_melee_audio: bool) {
    let fixture = support::fixture(name);
    if fixture.get("preparation_cases").is_some() {
        preparation(&fixture);
        firearms(&fixture);
    }
    if let Some(cases) = fixture
        .get("melee_cases")
        .or_else(|| fixture.get("attack_cases"))
    {
        melee(cases.as_array().unwrap(), native_melee_audio);
    }
    if fixture.get("caller_cases").is_some() {
        neighbors(&fixture, native_melee_audio);
    }
    if fixture.get("body_cases").is_some() {
        friendly(&fixture);
    }
}

pub(super) fn preparation(fixture: &Value) {
    let attacks = data("Data S");
    let cases = fixture.array("preparation_cases");

    for case in cases.iter() {
        let mut monster = MonsterRecord {
            bytes: case.bytes("enemy_before"),
        };
        let prepared = enemy_weapons::prepare(
            &mut monster,
            &case.bytes("weapon"),
            &case.bytes("ammo"),
            &attacks,
        )
        .unwrap();
        support::native_expect! {
            bytes prepared.packet => &case["packet"], "case{index} packet";
            bytes monster.bytes => &case["enemy_after"], "case{index} enemy";
        }
    }
}

pub(super) fn firearms(fixture: &Value) {
    let attacks = data("Data S");
    let cases = fixture.array("firearm_cases");

    for (index, case) in cases.iter().enumerate() {
        let mut party = case.bytes("warriors_before");
        let member = case.n::<usize>("member");
        let mut warrior = crate::support::warrior_at(&party, member);
        let mut monster = MonsterRecord {
            bytes: case.bytes("enemy_before"),
        };
        let prepared = enemy_weapons::prepare(
            &mut monster,
            &case.bytes("weapon"),
            &case.bytes("ammo"),
            &attacks,
        )
        .unwrap();
        assert_eq!(prepared.packet, case.bytes("packet"), "case{index} packet");
        let mut random = RandomState::new(case.n::<u32>("seed"));
        let result =
            enemy_weapons::resolve(&mut warrior, &mut monster, &prepared, &mut random, 0).unwrap();
        crate::support::put_warrior(&mut party, member, &warrior);
        support::native_expect! {
            bytes party => &case["warriors_after"], &format!("case{index} full party");
            bytes monster.bytes => &case["enemy_after"], "case{index} enemy";
            rng random => (&case["seed_after"], &case["rng_draws"]), "binary_endgame_weapons";
            scalar (result.sounds) => &case["sounds"], "binary_endgame_weapons";
        }
    }
}

pub(super) fn melee(cases: &[Value], native_audio: bool) {
    let items = items();

    for (index, case) in cases.iter().enumerate() {
        let mut party = case.bytes("warriors_before");
        let member = case.n::<usize>("member");
        let mut warrior = crate::support::warrior_at(&party, member);
        let mut monster = MonsterRecord {
            bytes: case.bytes("enemy_before"),
        };
        let mut random = RandomState::new(
            case.get("seed_before")
                .unwrap_or(&case["seed"])
                .as_u64()
                .unwrap() as u32,
        );
        let weapon = i16::from_be_bytes(monster.bytes[0x28..0x2a].try_into().unwrap()) as usize;
        let (result, packet) = enemy_weapons::melee_with_packet(
            &mut warrior,
            &mut monster,
            &items[weapon],
            &mut random,
            0,
            0,
            40 + member as i16,
            &data("Data S"),
            &[0; 58],
        )
        .unwrap();
        support::native_expect! {
            bytes &packet => &case.bytes("packet"), &format!("case{index} packet");
        };
        crate::support::put_warrior(&mut party, member, &warrior);
        support::native_expect! {
            bytes party => &case["warriors_after"], &format!("case{index} party");
            bytes monster.bytes => &case["enemy_after"], &format!("case{index} attacker");
            rng random => (&case["seed_after"], &case["rng_draws"]), "binary_endgame_weapons";
        }
        if native_audio {
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
        } else {
            assert_eq!(result.sounds, case.n::<Vec<i16>>("sounds"));
        }
    }
}

pub(super) fn neighbors(fixture: &Value, native_audio: bool) {
    let items = items();
    let cases = fixture.array("caller_cases");

    for (index, case) in cases.iter().enumerate() {
        let raw = case.bytes("warriors_before");
        let mut warriors: Vec<_> = support::warriors(&raw);
        // Genetic caller cases record the acting member; earlier corpora act as member 0.
        let member = if case.get("member").is_some() {
            case.n("member")
        } else {
            0
        };
        let mut combat = native_common::neighbor_combat(case, member);
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let events = runtime::check_neighbors_with_definitions(
            &mut combat,
            &mut warriors,
            &mut random,
            10,
            &items,
        )
        .unwrap();
        let actual: Vec<_> = support::record_bytes(&warriors);
        support::native_expect! {
            bytes actual => &case["warriors_after"], &format!("case{index} party");
            bytes combat.roster.records[0].bytes => &case["enemy_after"], &format!("case{index} attacker");
        }
        assert_eq!(combat.field.tiles, case.words("map_after"), "case{index} map");
        assert_eq!(combat.battle_order, case.words("order_after"), "case{index} order");
        support::assert_rng(&random, &case["seed_after"], &case["rng_draws"]);
        // Earlier probes intercept result-audio; their empty list is not a silence oracle.
        if native_audio {
            support::expect(support::event_sounds(&events), &case["sounds"]);
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
            support::attack_calls(&case["calls"]),
            "case{index} ordered attack calls"
        );
    }
}

pub(super) fn friendly(fixture: &Value) {
    let items = items();
    let attacks = data("Data S");
    let cases = fixture.array("body_cases");

    for (index, case) in cases.iter().enumerate() {
        let mut monster = MonsterRecord {
            bytes: case.bytes("enemy_before"),
        };
        let mut target = MonsterRecord {
            bytes: case.bytes("target_before"),
        };
        let id = i16::from_be_bytes(monster.bytes[0x26..0x28].try_into().unwrap());
        let ammo = enemy_weapons::ammo_id(&monster, case.n::<i16>("context")).unwrap();
        let prepared = enemy_weapons::prepare(
            &mut monster,
            &items[id as usize],
            &items[ammo as usize],
            &attacks,
        )
        .unwrap();
        support::native_expect! {
            bytes &prepared.packet => &case.bytes("packet"), &format!("body{index} packet");
        };
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result =
            spells::resolve_enemy_firearm_body(&mut target, &monster, &prepared, 0, &mut random)
                .unwrap();
        support::native_expect! {
            bytes target.bytes => &case["target_after"], &format!("body{index} target");
            bytes monster.bytes => &case["enemy_after"], &format!("body{index} attacker");
            rng random => (&case["seed_after"], &case["rng_draws"]), "binary_endgame_weapons";
            scalar (result.sounds) => &case["sounds"], "binary_endgame_weapons";
        }
    }
}

#[test]
fn required_endgame_ranged_to_natural_fallback_matches_original_full_records_and_randomness() {
    let fixture: serde_json::Value = support::fixture("endgame-weapons.json");
    let cases = fixture.array("natural_cases");

    for (index, case) in cases.iter().enumerate() {
        let mut party = case.bytes("warriors_before");
        let member = case.n::<usize>("member");
        let mut warrior = crate::support::warrior_at(&party, member);
        let mut monster = MonsterRecord {
            bytes: case.bytes("enemy_before"),
        };
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let (result, packet) = centurions_core::combat::attack::natural_attack_with_packet(
            &mut warrior,
            &mut monster,
            case.n::<usize>("index"),
            &mut random,
            &support::data("Data S"),
            &[0; 58],
            0,
        )
        .unwrap();
        assert_eq!(packet, case.bytes("packet"), "case{index} packet");
        crate::support::put_warrior(&mut party, member, &warrior);
        support::native_expect! {
            bytes party => &case["warriors_after"], "case{index} party";
            bytes monster.bytes => &case["enemy_after"], "case{index} enemy";
            rng random => (&case["seed_after"], &case["rng_draws"]), "binary_endgame_weapons";
            scalar (result
                        .sounds
                        .into_iter()
                        .chain([centurions_core::combat::ally_turns::result_audio(
                            &monster,
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
    let f: serde_json::Value = support::fixture("endgame-weapons.json");

    let content = support::content();
    let mission = content.map(0).unwrap();
    let cases = f.array("full_caller_cases");

    for (index, case) in cases.iter().enumerate() {
        let before_records = case.bytes("records_before");
        let mut warriors: Vec<_> = support::warriors(&case["warriors_before"]);
        let mut positions = vec![(20, 20)];
        if before_records.len() > MONSTER_BYTES {
            positions.push((24, 20));
        }
        let mut combat = support::battle();
        combat.battle = 129;
        combat.field.tiles = case.words("map_before");
        combat.squad.pos = support::positions(&case["warrior_positions"]);
        combat.roster.records = support::monsters(&before_records);
        combat.roster.mon_pos = positions;
        combat.battle_order = case.words("order_before");
        combat.spell_info = case.bytes("packet_before");
        combat.cast_identity = case.n::<Vec<i16>>("identity_before").try_into().unwrap();
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let mut work = runtime::EnemyGlobals {
            item_context: case.n::<i16>("item_context_before"),
            attack_count: case.n::<i16>("attack_count_before"),
            packet: combat.spell_info.clone(),
            identity: combat.cast_identity,
            ..runtime::EnemyGlobals::default()
        };
        let events = runtime::enemy_action_with_globals(
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
        let actual_party: Vec<_> = support::record_bytes(&warriors);
        let actual_records: Vec<_> = support::record_bytes(&combat.roster.records);
        support::native_expect! {
            bytes actual_party => &case["warriors_after"], &format!("case{index} party");
            bytes actual_records => &case["records_after"], &format!("case{index} enemies");
        }
        assert_eq!(work.packet, combat.spell_info, "case{index} packet work");
        assert_eq!(
            work.identity, combat.cast_identity,
            "case{index} identity work"
        );
        assert_eq!(
            combat.spell_info,
            case.bytes("packet_after"),
            "case{index} packet"
        );
        assert_eq!(
            combat.cast_identity,
            <[i16; 3]>::try_from(case.n::<Vec<i16>>("identity_after")).unwrap(),
            "case{index} identity"
        );
        assert_eq!(combat.field.tiles, case.words("map_after"), "case{index} map");
        assert_eq!(combat.battle_order, case.words("order_after"), "case{index} order");

        support::native_expect! {
            positions combat.roster.mon_pos => &case["positions_after"], "case{index} positions";
        };
        support::expect(combat.kill_party, &case["unavailable_after"]);
        assert_eq!(
            work.item_context,
            case.n::<i16>("item_context_after"),
            "case{index} item context"
        );
        support::native_expect! {
            scalar work.attack_count => &case["attack_count_after"], "binary_endgame_weapons";
            rng random => (&case["seed_after"], &case["rng_draws"]), "binary_endgame_weapons";
            scalar (support::event_sounds(&events)) => &case["sounds"], "binary_endgame_weapons";
        }
        let mut expected_hits = vec![];
        let mut prior_party = case.bytes("warriors_before");
        let mut prior_records = before_records;
        for resolution in case.array("resolutions") {
            let no_human_warrior = prior_party
                .as_chunks::<WARRIOR_BYTES>()
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
                        &prior_party[body * WARRIOR_BYTES..(body + 1) * WARRIOR_BYTES],
                        &after_party[body * WARRIOR_BYTES..(body + 1) * WARRIOR_BYTES],
                        0x50,
                    )
                } else if body >= 10 && body < 10 + combat.roster.records.len() {
                    (
                        &prior_records[(body - 10) * MONSTER_BYTES..(body - 9) * MONSTER_BYTES],
                        &after_records[(body - 10) * MONSTER_BYTES..(body - 9) * MONSTER_BYTES],
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
                    ..
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
