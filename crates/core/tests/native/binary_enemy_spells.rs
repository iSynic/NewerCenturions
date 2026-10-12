use crate::prelude::*;

use centurions_core::combat::spells::resolve_warriors;

#[test]
fn enemy_spell_damage_matches_original_complete_party_and_random_consumption() {
    let fixture: serde_json::Value = crate::support::fixture("enemy-spell-damage.json");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let warriors = crate::support::warriors(&case.bytes("before"));
        let packet = case.bytes("packet");
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result = resolve_warriors(
            &warriors,
            &packet,
            (case.n::<i64>("spell_id") % 1000 / 100 - 1) as i16,
            case.n::<i16>("count"),
            &[1; 6],
            false,
            &mut random,
        )
        .unwrap();
        let actual = crate::support::record_bytes(&result.warriors);
        crate::support::native_expect! {
        bytes actual => &case["after"], &format!("case {index}");
        scalar random.seed => &case["seed_after"], "binary_enemy_spells";
        scalar random.draws => &case["rng_draws"], "binary_enemy_spells";
        scalar (result
                        .targets
                        .iter()
                        .flat_map(|(_, target)| target.sounds.iter().copied())
                        .collect::<Vec<_>>()) => &case["sounds"], "binary_enemy_spells";
        }
    }
}

#[test]
fn enemy_spell_selection_matches_original_targets_caster_and_random_consumption() {
    use centurions_content::{combat::MonsterRecord, mission::OpeningMission};
    use centurions_core::combat::{
        Combat, EnemySetup, SquadPlacement, TacticalMap, spells::select_targets,
    };
    let mut additional: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../evidence/fixtures/additional-spells.json"),
        )
        .unwrap(),
    )
    .unwrap();
    additional["cases"] = additional["selection_cases"].take();
    let fixtures: [serde_json::Value; 4] = [
        crate::support::fixture("enemy-spell-selection.json"),
        serde_json::from_str(
            &std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../evidence/fixtures/firearm-targets.json"),
            )
            .unwrap(),
        )
        .unwrap(),
        additional,
        serde_json::from_str(
            &std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../evidence/fixtures/condition-targets.json"),
            )
            .unwrap(),
        )
        .unwrap(),
    ];
    for fixture in fixtures {
        let properties = fixture.bytes("properties");
        let areas = fixture.bytes("areas");
        let mission = OpeningMission {
            tile_properties: properties[64..12064].to_vec(),
            shared: std::sync::Arc::new(Default::default()),
            ..crate::support::mission()
        };
        for (index, case) in fixture.array("cases").iter().enumerate() {
            let warriors = crate::support::warriors(&case.bytes("party"));
            let distance = case.n::<i16>("distance");
            let tiles = crate::support::words(&case.bytes("map"));
            let combat = Combat {
                battle: 28,
                map: TacticalMap {
                    tiles,
                    shift: (0, 0),
                    edge_word: 0,
                },
                squad: SquadPlacement {
                    positions: (0..6)
                        .map(|i| {
                            (
                                20 + distance + i * if case["weapon_id"].is_null() { 1 } else { 4 },
                                20,
                            )
                        })
                        .collect(),
                    under_tiles: vec![1155; 6],
                },
                enemies: EnemySetup {
                    records: vec![MonsterRecord {
                        bytes: case.bytes("caster_before"),
                    }],
                    positions: vec![(20, 20)],
                },
                order: vec![-1, 10],
                ..crate::support::battle()
            };
            let mut random = RandomState::new(case.n::<u32>("seed_before"));
            let packet = case.bytes("packet");
            let result = if case["weapon_id"].is_null() {
                select_targets(
                    &combat,
                    &warriors,
                    &mission,
                    &areas,
                    0,
                    &packet,
                    &mut random,
                )
            } else {
                centurions_core::combat::spells::select_firearm_targets(
                    &combat,
                    &warriors,
                    &mission,
                    &areas,
                    0,
                    &centurions_core::combat::firearm::PreparedFirearm {
                        packet,
                        slot: 0,
                        shots: case.n::<i16>("shots"),
                        rounds: 0,
                        weight: 0,
                        armor_filter: false,
                        sound: 0,
                    },
                    &mut random,
                )
            };
            if case["invalid_shape"].as_bool().unwrap_or(false) {
                assert!(
                    matches!(result, Err(centurions_core::RuleError::Unsupported { .. })),
                    "invalid shape case {index}"
                );
                crate::support::expect(random.seed, &case["seed_before"]);
                assert_eq!(random.draws, 0);
                continue;
            }
            let result = result.unwrap();
            crate::support::native_expect! {
            bytes result.caster.bytes => &case["caster_after"], &format!("caster case {index}");
            scalar result.range => &case["range"], "binary_enemy_spells";
            scalar result.nearest => &case["nearest"], "binary_enemy_spells";
            scalar result.count => &case["count"], "binary_enemy_spells";
            bytes crate::support::word_bytes(&result.distances) => &case["distances"], "distances case {index}";
            }
            let mut targets = [255; 42];
            for (slot, target) in result.targets.iter().enumerate() {
                targets[slot * 6..slot * 6 + 6].copy_from_slice(
                    &[target.first, target.second, target.kind]
                        .iter()
                        .flat_map(|v| v.to_be_bytes())
                        .collect::<Vec<_>>(),
                );
            }
            crate::support::native_expect! {
            bytes targets => &case["targets"], &format!("targets case {index}");
            scalar !result.targets.is_empty() => &case["cast"], "binary_enemy_spells";
            scalar random.seed => &case["seed_after"], "binary_enemy_spells";
            scalar random.draws => &case["rng_draws"], "binary_enemy_spells";
            }
        }
    }
}

#[test]
fn spell_target_masks_reflection_and_fields_match_original_complete_buffers() {
    use centurions_core::combat::{
        Combat, EnemySetup, SquadPlacement, TacticalMap,
        spells::{SpellContext, SpellTarget, target_mask},
    };
    let fixture: serde_json::Value = crate::support::fixture("spell-targets.json");
    let areas = fixture.bytes("areas");
    let tiles: Vec<i16> = crate::support::words(&fixture.bytes("map"));
    let party = fixture.bytes("party");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let variant = case.n::<i64>("variant");
        let mut warriors = crate::support::warriors(&party);
        if variant == 1 {
            for warrior in &mut warriors {
                warrior.bytes[0x9a..0x9c].copy_from_slice(&1_i16.to_be_bytes());
            }
        }
        let monsters = crate::support::monsters(&case["records"]);
        let combat = Combat {
            battle: 28,
            map: TacticalMap {
                tiles: tiles.clone(),
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: (0..6).map(|i| (20 + i, 20)).collect(),
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: monsters,
                positions: vec![(20, 22), (22, 22)],
            },
            order: vec![-1, 10],
            ..crate::support::battle()
        };
        let mut fields = vec![0; 1200];
        if variant == 2 {
            for slot in fields.as_chunks_mut::<20>().0 {
                slot[15] = 1;
            }
        }
        let packet = case.bytes("packet");
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result = target_mask(
            &combat,
            &warriors,
            &areas,
            &SpellTarget {
                first: case.n::<i16>("first"),
                second: case.n::<i16>("second"),
                kind: case.n::<i16>("kind"),
            },
            &SpellContext {
                packet: &packet,
                id: case.n::<i16>("spell_id"),
                count: case.n::<i16>("count"),
                caster: 10,
                initiative: 1,
                collision: false,
            },
            &fields,
            &mut random,
        )
        .unwrap();
        crate::support::native_expect! {
        bytes crate::support::word_bytes(&result.mask) => &case["mask"], &format!("mask case {index}");
        bytes result.fields => &case["fields"], &format!("fields case {index}");
        scalar random.seed => &case["seed_after"], "binary_enemy_spells";
        scalar random.draws => &case["rng_draws"], "binary_enemy_spells";
        }
    }
}
