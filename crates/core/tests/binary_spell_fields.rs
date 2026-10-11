use centurions_content::{Warrior, combat::MonsterRecord};
use centurions_core::{
    RandomState,
    combat::{Combat, EnemySetup, SquadPlacement, TacticalMap, spells},
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
fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!("../../../evidence/fixtures/spell-fields.json")).unwrap()
}

#[test]
fn field_timers_and_usage_reset_match_original_complete_buffers() {
    for (index, case) in fixture()["ticks"].as_array().unwrap().iter().enumerate() {
        let fields = decode(case["fields_before"].as_str().unwrap());
        let used = decode(case["used_before"].as_str().unwrap());
        let cursor = case["cursor_before"].as_i64().unwrap() as i16;
        assert_eq!(
            cursor.wrapping_add(1),
            case["cursor_after"].as_i64().unwrap() as i16
        );
        let actual = spells::tick_fields(&fields, cursor.wrapping_add(1)).unwrap();
        assert_bytes(
            &actual,
            &decode(case["fields_after"].as_str().unwrap()),
            &format!("field tick {index}"),
        );
        let actual = spells::reset_used(&used).unwrap();
        assert_bytes(
            &actual,
            &decode(case["used_after"].as_str().unwrap()),
            &format!("used reset {index}"),
        );
        assert_eq!(case["rng_draws"].as_u64().unwrap(), 0);
    }
}

#[test]
fn complete_collisions_match_original_all_bodies_fields_usage_and_randomness() {
    let fixture = fixture();
    let areas = decode(fixture["areas"].as_str().unwrap());
    let map = decode(fixture["map"].as_str().unwrap())
        .as_chunks::<2>()
        .0
        .iter()
        .map(|v| i16::from_be_bytes(*v))
        .collect::<Vec<_>>();
    let attacks = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../NewCenturions/TheNewCenturions/Data Files/Data S"),
    )
    .unwrap();
    let mission = centurions_content::mission::OpeningMission {
        name: "fixture".into(),
        start: (2, 2),
        header: vec![],
        regions: vec![],
        floor: vec![],
        objects: vec![],
        events: vec![],
        tile_properties: vec![],
        terrain: 2,
        default_tile: 155,
        minutes_per_tick: 5,
        shared: std::sync::Arc::new(centurions_content::mission::StoryTables {
            areas: areas.clone(),
            combat: Some(centurions_content::combat::CombatContent {
                encounters: vec![],
                battles: vec![],
                monsters: vec![],
                race_moves: vec![],
                attacks: attacks.clone(),
            }),
            ..Default::default()
        }),
    };
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let mut warriors = decode(case["warriors_before"].as_str().unwrap())
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
        let mut combat = Combat {
            battle: 28,
            map: TacticalMap {
                tiles: map.clone(),
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: (0..6).map(|i| (20 + i, 20)).collect(),
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: enemies,
                positions: vec![(20, 22), (22, 22)],
            },
            order: vec![-1, 10],
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
        };
        let fields = decode(case["fields_before"].as_str().unwrap());
        let mut usage = spells::FieldUsage {
            slot: 0,
            used: decode(case["used_before"].as_str().unwrap()),
        };
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let mut runtime_warriors = warriors.clone();
        let mut runtime_combat = combat.clone();
        runtime_combat.spell_fields = fields.clone();
        runtime_combat.field_used = usage.used.clone();
        let mut runtime_random = random.clone();
        if index == 0 {
            let mut blocked_combat = runtime_combat.clone();
            let field = blocked_combat.spell_fields[..20].to_vec();
            blocked_combat.spell_fields[40..60].copy_from_slice(&field);
            blocked_combat.spell_fields[57] = 1;
            let before_combat = blocked_combat.clone();
            let mut blocked_warriors = runtime_warriors.clone();
            let before_warriors = blocked_warriors.clone();
            let mut blocked_random = random.clone();
            let before_random = blocked_random.clone();
            let mut blocked_count = 6;
            assert!(
                centurions_core::combat::runtime::collide_fields(
                    &mut blocked_combat,
                    &mut blocked_warriors,
                    &mission,
                    &mut blocked_count,
                    &mut blocked_random,
                    case["body"].as_i64().unwrap() as i16,
                )
                .is_err()
            );
            assert_eq!(blocked_combat, before_combat);
            assert_eq!(blocked_warriors, before_warriors);
            assert_eq!(blocked_count, 6);
            assert_eq!(blocked_random, before_random);
        }
        let mut sounds = vec![];
        let mut count = 6;
        let mut level = 0;
        for slot in 0..60 {
            usage.slot = slot;
            let Some(collision) = spells::collision_field(
                &combat,
                &warriors,
                &fields,
                &usage,
                case["body"].as_u64().unwrap() as usize,
                &attacks,
            )
            .unwrap() else {
                continue;
            };
            count = collision.count;
            level = collision.level;
            let context = spells::SpellContext {
                packet: &collision.packet,
                id: collision.id,
                count,
                caster: case["body"].as_i64().unwrap() as i16,
                initiative: 1,
                collision: true,
            };
            let mask = spells::target_mask(
                &combat,
                &warriors,
                &areas,
                &collision.target,
                &context,
                &fields,
                &mut random,
            )
            .unwrap();
            assert_eq!(mask.fields, fields, "collision preserves fields {index}");
            let result = spells::resolve_collision(
                &warriors,
                &combat.enemies.records,
                &mask.mask,
                &spells::DirectSpell {
                    packet: &collision.packet,
                    level,
                    count,
                    armor_filter: false,
                    weapon_sound: 0,
                },
                &mut usage,
                &mut random,
            )
            .unwrap();
            sounds.extend(result.sounds);
            for (_, target) in result.targets {
                assert!(!target.killed);
                sounds.extend(target.sounds);
            }
            warriors = result.warriors;
            combat.enemies.records = result.enemies;
        }
        let actual = warriors
            .iter()
            .flat_map(|w| w.bytes.iter().copied())
            .collect::<Vec<_>>();
        assert_bytes(
            &actual,
            &decode(case["warriors_after"].as_str().unwrap()),
            &format!("collision party {index}"),
        );
        let actual = combat
            .enemies
            .records
            .iter()
            .flat_map(|m| m.bytes.iter().copied())
            .collect::<Vec<_>>();
        assert_bytes(
            &actual,
            &decode(case["enemies_after"].as_str().unwrap()),
            &format!("collision enemies {index}"),
        );
        assert_bytes(
            &fields,
            &decode(case["fields_after"].as_str().unwrap()),
            &format!("collision fields {index}"),
        );
        assert_bytes(
            &usage.used,
            &decode(case["used_after"].as_str().unwrap()),
            &format!("collision usage {index}"),
        );
        assert_eq!(
            count,
            case["count_after"].as_i64().unwrap() as i16,
            "collision count {index}"
        );
        assert_eq!(
            level,
            case["level_after"].as_i64().unwrap() as i16,
            "collision level {index}"
        );
        assert_eq!(
            random.seed,
            case["seed_after"].as_u64().unwrap() as u32,
            "collision seed {index}"
        );
        assert_eq!(
            random.draws,
            case["rng_draws"].as_u64().unwrap(),
            "collision draws {index}"
        );
        assert_eq!(
            sounds,
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i16)
                .collect::<Vec<_>>(),
            "collision sounds {index}"
        );
        let mut runtime_count = 6;
        let events = centurions_core::combat::runtime::collide_fields(
            &mut runtime_combat,
            &mut runtime_warriors,
            &mission,
            &mut runtime_count,
            &mut runtime_random,
            case["body"].as_i64().unwrap() as i16,
        )
        .unwrap();
        assert_bytes(
            &runtime_warriors
                .iter()
                .flat_map(|w| w.bytes.iter().copied())
                .collect::<Vec<_>>(),
            &decode(case["warriors_after"].as_str().unwrap()),
            &format!("runtime collision party {index}"),
        );
        assert_bytes(
            &runtime_combat
                .enemies
                .records
                .iter()
                .flat_map(|m| m.bytes.iter().copied())
                .collect::<Vec<_>>(),
            &decode(case["enemies_after"].as_str().unwrap()),
            &format!("runtime collision enemies {index}"),
        );
        assert_bytes(
            &runtime_combat.spell_fields,
            &decode(case["fields_after"].as_str().unwrap()),
            &format!("runtime collision fields {index}"),
        );
        assert_bytes(
            &runtime_combat.field_used,
            &decode(case["used_after"].as_str().unwrap()),
            &format!("runtime collision usage {index}"),
        );
        assert_eq!(
            runtime_count,
            case["count_after"].as_i64().unwrap() as i16,
            "runtime collision count {index}"
        );
        assert_eq!(
            runtime_random.seed,
            case["seed_after"].as_u64().unwrap() as u32,
            "runtime collision seed {index}"
        );
        assert_eq!(
            runtime_random.draws,
            case["rng_draws"].as_u64().unwrap(),
            "runtime collision draws {index}"
        );
        assert_eq!(
            events
                .into_iter()
                .filter_map(|event| match event {
                    centurions_core::Event::Sound { resource_id } => Some(resource_id),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            sounds,
            "runtime collision sounds {index}"
        );
    }
}
