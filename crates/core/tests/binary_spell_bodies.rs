use centurions_content::{Warrior, combat::MonsterRecord};
use centurions_core::{
    RandomState,
    combat::spells::{DirectSpell, resolve_bodies},
};

fn decode(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
fn assert_bytes(actual: &[u8], expected: &[u8], context: &str) {
    assert_eq!(actual.len(), expected.len(), "{context} length");
    if let Some(index) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "{context} byte {index}: {} != {}",
            actual[index], expected[index]
        );
    }
}

#[test]
fn spell_resolution_matches_original_ordered_party_and_monster_records() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/spell-bodies.json")).unwrap();
    assert_resolution(&fixture);
}

#[test]
fn additional_spells_match_original_ordered_party_and_monster_records() {
    let fixture: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../evidence/fixtures/additional-spells.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_resolution(&fixture);
}

fn assert_resolution(fixture: &serde_json::Value) {
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let warriors = decode(case["warriors_before"].as_str().unwrap())
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|w| Warrior::read(w).unwrap())
            .collect::<Vec<_>>();
        let enemies = decode(case["enemies_before"].as_str().unwrap())
            .as_chunks::<288>()
            .0
            .iter()
            .map(|m| MonsterRecord { bytes: m.to_vec() })
            .collect::<Vec<_>>();
        let mask = decode(case["mask"].as_str().unwrap())
            .as_chunks::<2>()
            .0
            .iter()
            .map(|w| i16::from_be_bytes(*w))
            .collect::<Vec<_>>();
        let packet = decode(case["packet"].as_str().unwrap());
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let result = resolve_bodies(
            &warriors,
            &enemies,
            &mask,
            &DirectSpell {
                packet: &packet,
                level: (case["spell_id"].as_i64().unwrap() % 1000 / 100 - 1) as i16,
                count: case["count"].as_i64().unwrap() as i16,
                armor_filter: false,
                weapon_sound: 0,
            },
            &mut random,
        )
        .unwrap();
        let actual = result
            .warriors
            .iter()
            .flat_map(|w| w.bytes.iter().copied())
            .collect::<Vec<_>>();
        assert_bytes(
            &actual,
            &decode(case["warriors_after"].as_str().unwrap()),
            &format!("case {index} party"),
        );
        let actual = result
            .enemies
            .iter()
            .flat_map(|m| m.bytes.iter().copied())
            .collect::<Vec<_>>();
        assert_bytes(
            &actual,
            &decode(case["enemies_after"].as_str().unwrap()),
            &format!("case {index} enemies"),
        );
        assert_eq!(
            random.seed,
            case["seed_after"].as_u64().unwrap() as u32,
            "seed case {index}"
        );
        assert_eq!(
            random.draws,
            case["rng_draws"].as_u64().unwrap(),
            "draws case {index}"
        );
        assert_eq!(
            result
                .sounds
                .iter()
                .copied()
                .chain(
                    result
                        .targets
                        .iter()
                        .flat_map(|(_, r)| r.sounds.iter().copied())
                )
                .collect::<Vec<_>>(),
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i16)
                .collect::<Vec<_>>(),
            "sounds case {index}"
        );
    }
}

#[test]
fn enemy_firearms_match_original_scientist_records_sounds_and_randomness() {
    use centurions_core::combat::{firearm::PreparedFirearm, spells::resolve_enemy_firearm_body};
    let fixture: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../evidence/fixtures/enemy-firearm-bodies.json"),
        )
        .unwrap(),
    )
    .unwrap();
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let caster = MonsterRecord {
            bytes: decode(case["caster_before"].as_str().unwrap()),
        };
        let mut target = MonsterRecord {
            bytes: decode(case["target_before"].as_str().unwrap()),
        };
        let prepared = PreparedFirearm {
            packet: decode(case["packet"].as_str().unwrap()),
            shots: case["shots"].as_i64().unwrap() as i16,
            sound: case["weapon_sound"].as_i64().unwrap() as i16,
            slot: 0,
            rounds: 0,
            weight: 0,
            armor_filter: false,
        };
        assert_eq!(
            case["preparation_draws"].as_u64().unwrap(),
            0,
            "native preparation {index}"
        );
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let result =
            resolve_enemy_firearm_body(&mut target, &caster, &prepared, 0, &mut random).unwrap();
        assert_bytes(
            &target.bytes,
            &decode(case["target_after"].as_str().unwrap()),
            &format!("firearm case {index}"),
        );
        assert_bytes(
            &caster.bytes,
            &decode(case["caster_after"].as_str().unwrap()),
            &format!("firearm caster {index}"),
        );
        assert_eq!(
            random.seed,
            case["seed_after"].as_u64().unwrap() as u32,
            "firearm seed {index}"
        );
        assert_eq!(
            random.draws,
            case["rng_draws"].as_u64().unwrap(),
            "firearm draws {index}"
        );
        assert_eq!(
            result.sounds,
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i16)
                .collect::<Vec<_>>(),
            "firearm sounds {index}"
        );
    }
}

#[test]
fn global_spell_masks_match_original_faction_and_living_body_rules() {
    use centurions_core::combat::{
        Combat, EnemySetup, SquadPlacement, TacticalMap, spells::global_targets,
    };
    let fixture: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../evidence/fixtures/additional-spells.json"),
        )
        .unwrap(),
    )
    .unwrap();
    for (index, case) in fixture["mask_cases"].as_array().unwrap().iter().enumerate() {
        let warriors = decode(case["warriors"].as_str().unwrap())
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|w| Warrior::read(w).unwrap())
            .collect::<Vec<_>>();
        let enemies = decode(case["enemies_before"].as_str().unwrap())
            .as_chunks::<288>()
            .0
            .iter()
            .map(|m| MonsterRecord { bytes: m.to_vec() })
            .collect::<Vec<_>>();
        let combat = Combat {
            battle: 0,
            map: TacticalMap {
                tiles: vec![],
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: vec![],
                under_tiles: vec![],
            },
            enemies: EnemySetup {
                records: enemies,
                positions: vec![],
            },
            order: vec![],
            cursor: 0,
            round: 0,
            death_queue: Default::default(),
            spell_fields: vec![0; 1200],
            field_used: vec![0; 128],
            attack_packet: Some(vec![0; 58]),
            attack_identity: Some([0; 3]),
            weapon_work: centurions_core::combat::weapon_work::WeaponWork::zero(),
            unavailable: 0,
            round_scripts: Default::default(),
        };
        let packet = decode(case["packet"].as_str().unwrap());
        let result = global_targets(&combat, &warriors, 0, &packet).unwrap();
        let actual = result
            .mask
            .iter()
            .flat_map(|v| v.to_be_bytes())
            .collect::<Vec<_>>();
        assert_bytes(
            &actual,
            &decode(case["mask"].as_str().unwrap()),
            &format!("global mask {index}"),
        );
        let mut actual = combat.enemies.records.clone();
        actual[0] = result.caster;
        let actual = actual
            .iter()
            .flat_map(|m| m.bytes.iter().copied())
            .collect::<Vec<_>>();
        assert_bytes(
            &actual,
            &decode(case["enemies_after"].as_str().unwrap()),
            &format!("global records {index}"),
        );
        assert_eq!(
            case["count_before"], case["count_after"],
            "global preserves count {index}"
        );
        assert_eq!(
            case["rng_draws"].as_u64().unwrap(),
            0,
            "global randomness {index}"
        );
    }
}
