use crate::prelude::*;

use crate::support::assert_bytes;
use centurions_core::combat::ally_turns;

fn verify(target_kind: i64) {
    let fixture: serde_json::Value = crate::support::fixture("guardian-melee.json");

    let content = crate::support::content();
    let attacks = &content.map(0).unwrap().combat.as_ref().unwrap().attacks;
    let weapon = &content.items[187];
    for (case_index, case) in fixture.array("cases").iter().enumerate() {
        if case.n::<i64>("target_kind") != target_kind {
            continue;
        }
        let context = format!("guardian case{case_index}");
        let source = case.n::<i16>("source");
        let target = case.n::<i16>("target");
        let actor = case.n::<i16>("actor");
        let index = case.n::<usize>("index");
        let mut warriors: Vec<_> = crate::support::warriors(&case["warriors_before"]);
        let mut order = vec![-1; 110];
        order[1] = actor;
        let mut combat = Combat {
            battle: 85,
            map: TacticalMap {
                tiles: case.words("map_before"),
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: vec![(21, 20); 6],
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: crate::support::monsters(&case["records_before"]),
                positions: case
                    .bytes("positions_before")
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|position| {
                        (
                            i16::from_be_bytes(position[..2].try_into().unwrap()),
                            i16::from_be_bytes(position[2..].try_into().unwrap()),
                        )
                    })
                    .collect(),
            },
            order,
            attack_identity: Some([2, 4, 6]),
            ..crate::support::battle()
        };
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let (result, packet) = if target_kind == 0 {
            enemy_weapons::melee_warrior_with_packet(
                &mut combat,
                &mut warriors,
                &mut random,
                (source, target, index, actor),
                weapon,
                0,
                attacks,
                &[0; 58],
            )
        } else {
            enemy_weapons::melee_enemy_with_packet(
                &mut combat,
                &mut random,
                (source, target, index, actor),
                weapon,
                0,
                attacks,
                &[0; 58],
            )
        }
        .unwrap_or_else(|error| panic!("{context}: {error:?}"));
        crate::support::native_expect! {
        bytes crate::support::record_bytes(&combat
                        .enemies.records) => &case["records_after"], &format!("{context} records");
        bytes crate::support::record_bytes(&warriors) => &case["warriors_after"], &format!("{context} warriors");
        bytes packet => &case["packet_after"], &format!("{context} packet");
        scalar random.seed => &case["seed_after"], "binary_guardian_melee";
        scalar random.draws => &case["rng_draws"], "binary_guardian_melee";
        }
        let mut sounds = result.sounds;
        sounds.push(
            ally_turns::result_audio(
                &combat.enemies.records[(source - 10) as usize],
                index,
                result.damage,
                word(weapon, 14).unwrap(),
            )
            .unwrap(),
        );
        assert_eq!(serde_json::json!(sounds), case["sounds"], "{context}");
        assert!(!result.killed, "{context}");

        assert_eq!(combat.map.tiles, case.words("map_after"), "{context} map");
        let positions: Vec<_> = combat
            .enemies
            .positions
            .iter()
            .flat_map(|(x, y)| x.to_be_bytes().into_iter().chain(y.to_be_bytes()))
            .collect();
        assert_bytes(
            &positions,
            &case.bytes("positions_after"),
            &format!("{context} positions"),
        );
    }
}

#[test]
fn guardian_melee_matches_native_warrior_poison_packet_audio_and_retained_actor() {
    verify(0);
}

#[test]
fn guardian_melee_matches_native_monster_poison_packet_audio_and_retained_actor() {
    verify(1);
}

fn caller_combat(case: &serde_json::Value) -> Combat {
    let mut order = vec![-1; 110];
    order[1] = 22;
    Combat {
        battle: 85,
        map: TacticalMap {
            tiles: case.words("map_before"),
            shift: (0, 0),
            edge_word: 0,
        },
        squad: SquadPlacement {
            positions: vec![(60, 60); 6],
            under_tiles: vec![1155; 6],
        },
        enemies: EnemySetup {
            records: crate::support::monsters(&case["records_before"]),
            positions: case
                .words("positions_before")
                .as_chunks::<2>()
                .0
                .iter()
                .map(|position| (position[0], position[1]))
                .collect(),
        },
        order,
        attack_identity: Some([2, 4, 6]),
        ..crate::support::battle()
    }
}

#[test]
fn guardian_reaction_caller_matches_native_retained_actor_reload_and_natural_attacks() {
    let fixture: serde_json::Value = crate::support::fixture("guardian-melee.json");

    let content = crate::support::content();
    let attacks = &content.map(0).unwrap().combat.as_ref().unwrap().attacks;
    let cases = fixture.array("caller_cases");

    for (case_index, case) in cases.iter().enumerate() {
        let context = format!("guardian caller{case_index}");
        let mut combat = caller_combat(case);
        let mut warriors: Vec<_> = crate::support::warriors(&case["warriors_before"]);
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let events = centurions_core::combat::runtime::check_neighbors_with_context(
            &mut combat,
            &mut warriors,
            &mut random,
            22,
            &content.items,
            attacks,
        )
        .unwrap_or_else(|error| panic!("{context}: {error:?}"));
        let actual: Vec<_> = crate::support::record_bytes(&combat.enemies.records);
        assert_bytes(
            &actual,
            &case.bytes("records_after"),
            &format!("{context} records"),
        );
        let actual: Vec<_> = crate::support::record_bytes(&warriors);
        crate::support::native_expect! {
        bytes actual => &case["warriors_after"], &format!("{context} warriors");
        bytes combat.attack_packet.as_ref().unwrap() => &case["packet_after"], &context;
        }
        assert_eq!(combat.map.tiles, case.words("map_after"), "{context}");
        assert_eq!(combat.order, case.words("order_after"), "{context}");
        let actual: Vec<_> = combat
            .enemies
            .positions
            .iter()
            .flat_map(|(x, y)| x.to_be_bytes().into_iter().chain(y.to_be_bytes()))
            .collect();
        crate::support::native_expect! {
        bytes actual => &case["positions_after"], &context;
        scalar random.seed => &case["seed_after"], "binary_guardian_melee";
        scalar random.draws => &case["rng_draws"], "binary_guardian_melee";
        }
        let sounds: Vec<_> = crate::support::event_sounds(&events);
        assert_eq!(serde_json::json!(sounds), case["sounds"], "{context}");
        let calls: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                centurions_core::Event::AttackResolved {
                    attacker, target, ..
                } => Some((*attacker, *target)),
                _ => None,
            })
            .collect();
        let expected: Vec<_> = case
            .array("calls")
            .iter()
            .map(|call| {
                (
                    call[0].as_i64().unwrap() as i16,
                    call[1].as_i64().unwrap() as i16,
                )
            })
            .collect();
        assert_eq!(calls, expected, "{context}");
        let steps = case.array("attack_steps");
        assert_eq!(steps[0]["actor_current_before"], 0);
        assert_eq!(steps[0]["actor_current_after"], 187);
        assert_eq!(steps[1]["actor_current_before"], 187);
        assert_eq!(steps[1]["actor_current_after"], 0);
    }
}
