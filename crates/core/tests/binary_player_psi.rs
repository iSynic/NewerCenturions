use centurions_content::{Content, Warrior, combat::MonsterRecord, word};
use centurions_core::{
    Event, RandomState,
    combat::{
        Combat, EnemySetup, SquadPlacement, TacticalMap,
        player_psi::{self, CastRequest},
        spells,
    },
};
use serde_json::Value;

fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).unwrap())
        .collect()
}
fn words(hex: &str) -> Vec<i16> {
    bytes(hex)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| i16::from_be_bytes(*b))
        .collect()
}
fn party(hex: &str) -> Vec<Warrior> {
    bytes(hex)
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|b| Warrior::read(b).unwrap())
        .collect()
}
fn positions(hex: &str) -> Vec<(i16, i16)> {
    words(hex)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| (p[0], p[1]))
        .collect()
}
fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../evidence/fixtures/player-psi.json")).unwrap()
}
fn content() -> Content {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap()
}
fn assert_bytes(actual: &[u8], expected: &[u8], context: &str) {
    assert_eq!(actual.len(), expected.len(), "{context} size");
    if let Some(at) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!("{context} byte {at:x}: {} != {}", actual[at], expected[at]);
    }
}
fn controlled(case: &Value) -> Combat {
    let mut order = vec![-1; 110];
    order[1] = 1;
    Combat {
        battle: 85,
        map: TacticalMap {
            tiles: words(case["map_before"].as_str().unwrap()),
            shift: (0, 0),
            edge_word: 0,
        },
        squad: SquadPlacement {
            positions: positions(case["warrior_positions"].as_str().unwrap()),
            under_tiles: vec![1155; 6],
        },
        enemies: EnemySetup {
            records: bytes(case["records_before"].as_str().unwrap())
                .as_chunks::<288>()
                .0
                .iter()
                .map(|b| MonsterRecord { bytes: b.to_vec() })
                .collect(),
            positions: positions(case["monster_positions"].as_str().unwrap()),
        },
        order,
        cursor: 1,
        round: 0,
        round_scripts: Default::default(),
        death_queue: Default::default(),
        spell_fields: bytes(case["fields_before"].as_str().unwrap()),
        field_used: vec![0; 128],
        attack_packet: None,
        attack_identity: None,
        weapon_work: centurions_core::combat::weapon_work::WeaponWork::zero(),
        unavailable: 0,
    }
}
fn verify(
    case: &Value,
    mut combat: Combat,
    mission: &centurions_content::mission::OpeningMission,
    context: &str,
) {
    let mut warriors = party(case["warriors_before"].as_str().unwrap());
    let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
    let id = case["spell_id"].as_i64().unwrap() as i16;
    let power = case["power"].as_i64().unwrap() as i16;
    let target = if id == 2204 {
        let raw = words(case["casts"][0]["targets"].as_str().unwrap());
        Some((
            raw[0] - combat.map.shift.0 - 5,
            raw[1] - combat.map.shift.1 - 5,
        ))
    } else {
        None
    };
    let packet = spells::load_packet(&mission.combat.as_ref().unwrap().attacks, id).unwrap();
    let mut mask_rng = random.clone();
    let mask = if let Some(point) = target {
        let point = spells::SpellTarget {
            first: point.0 + combat.map.shift.0 + 5,
            second: point.1 + combat.map.shift.1 + 5,
            kind: 3,
        };
        spells::target_mask(
            &combat,
            &warriors,
            &mission.areas,
            &point,
            &spells::SpellContext {
                packet: &packet,
                id,
                count: power,
                caster: 1,
                initiative: combat.cursor as i16,
                collision: false,
            },
            &combat.spell_fields,
            &mut mask_rng,
        )
        .unwrap()
        .mask
        .to_vec()
    } else {
        let mut mask = vec![0; 110];
        for (i, w) in warriors.iter().enumerate() {
            if w.bytes[0x9df] != 0 && word(&w.bytes, 0x10).unwrap() != 0 {
                mask[i] = 1;
            }
        }
        for (i, m) in combat.enemies.records.iter().enumerate() {
            if word(&m.bytes, 0x110).unwrap() > 0 && word(&m.bytes, 0xe).unwrap() != 0 {
                mask[i + 10] = 1;
            }
        }
        mask
    };
    assert_eq!(
        mask,
        words(case["resolutions"][0]["mask"].as_str().unwrap()),
        "{context} selection mask"
    );
    assert_eq!(
        mask_rng.draws,
        case["resolutions"][0]["draws_before"].as_u64().unwrap(),
        "{context} target draw order"
    );
    let mut memory = vec![0; 480];
    let events = player_psi::cast(
        &mut combat,
        &mut warriors,
        mission,
        &mut random,
        &mut memory,
        0,
        CastRequest {
            spell: id,
            power,
            target,
        },
    )
    .unwrap();
    let actual_party: Vec<_> = warriors
        .iter()
        .flat_map(|w| w.bytes.iter().copied())
        .collect();
    let actual_records: Vec<_> = combat
        .enemies
        .records
        .iter()
        .flat_map(|m| m.bytes.iter().copied())
        .collect();
    assert_bytes(
        &actual_party,
        &bytes(case["warriors_after"].as_str().unwrap()),
        &format!("{context} party"),
    );
    assert_bytes(
        &actual_records,
        &bytes(case["records_after"].as_str().unwrap()),
        &format!("{context} enemies"),
    );
    assert_eq!(
        combat.map.tiles,
        words(case["map_after"].as_str().unwrap()),
        "{context} map"
    );
    assert_eq!(
        combat.squad.positions,
        positions(case["warrior_positions"].as_str().unwrap()),
        "{context} warrior positions"
    );
    assert_eq!(
        combat.enemies.positions,
        positions(case["monster_positions"].as_str().unwrap()),
        "{context} monster positions"
    );
    assert_bytes(
        &combat.spell_fields,
        &bytes(case["fields_after"].as_str().unwrap()),
        &format!("{context} fields"),
    );
    assert_bytes(
        &combat.field_used,
        &bytes(case["used_after"].as_str().unwrap()),
        &format!("{context} used"),
    );
    assert_bytes(
        &memory,
        &bytes(case["memory_after"].as_str().unwrap()),
        &format!("{context} memory"),
    );
    assert_eq!(combat.attack_packet, Some(packet), "{context} packet");
    assert_eq!(
        combat.attack_identity,
        Some([id / 1000 - 1, id % 1000 / 100 - 1, id % 100 - 1]),
        "{context} descriptor"
    );
    assert_eq!(
        random.seed,
        case["seed_after"].as_u64().unwrap() as u32,
        "{context} seed"
    );
    assert_eq!(
        random.draws,
        case["rng_draws"].as_u64().unwrap(),
        "{context} draws"
    );
    let sounds: Vec<_> = events
        .iter()
        .filter_map(|e| {
            if let Event::Sound { resource_id } = e {
                Some(*resource_id)
            } else {
                None
            }
        })
        .collect();
    let expected: Vec<_> = case["sounds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_i64().unwrap() as i16)
        .collect();
    assert_eq!(sounds, expected, "{context} sounds");
    let mut applied = vec![0; 110];
    for event in events {
        if let Event::SpellResolved { target, .. } = event {
            applied[target as usize] = 1;
        }
    }
    assert_eq!(
        applied,
        words(case["resolutions"][0]["applied_mask"].as_str().unwrap()),
        "{context} applied mask"
    );
}

#[test]
fn learned_player_psi_matches_96_complete_native_casts() {
    let data = fixture();
    let content = content();
    let mission = content.map(0).unwrap();
    let cases = data["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 96);
    for (i, case) in cases.iter().enumerate() {
        verify(
            case,
            controlled(case),
            mission,
            &format!("player psi case {i}"),
        );
    }
}
#[test]
fn learned_player_psi_matches_two_native_campaign_inputs() {
    let data = fixture();
    let content = content();
    let foundation = &data["route_foundation"];
    let combat: Combat = serde_json::from_value(foundation["combat"].clone()).unwrap();
    let mission = content
        .map(foundation["exploration"]["map"].as_i64().unwrap() as i32)
        .unwrap();
    let cases = data["route_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 2);
    for (i, case) in cases.iter().enumerate() {
        let mut combat = combat.clone();
        combat.weapon_work = centurions_core::combat::weapon_work::WeaponWork::zero();
        combat.spell_fields.fill(0);
        combat.field_used.fill(0);
        verify(case, combat, mission, &format!("campaign psi case {i}"));
    }
}
#[test]
fn player_psi_admission_matches_16_native_gates_without_mutation() {
    let data = fixture();
    let cases = data["gate_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 16);
    for case in cases {
        let warriors = party(case["before"].as_str().unwrap());
        let admission = player_psi::admission(
            &warriors,
            case["member"].as_i64().unwrap() as i16,
            u8::from(case["disabled"].as_bool().unwrap()),
        )
        .unwrap();
        assert_eq!(admission.allowed, case["allowed"].as_i64().unwrap() != 0);
        let sounds: Vec<_> = case["sounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_i64().unwrap() as i16)
            .collect();
        assert_eq!(admission.sounds, sounds);
        let actual: Vec<_> = warriors
            .iter()
            .flat_map(|w| w.bytes.iter().copied())
            .collect();
        assert_bytes(
            &actual,
            &bytes(case["after"].as_str().unwrap()),
            "admission unchanged",
        );
        assert_eq!(case["rng_draws"].as_u64().unwrap(), 0);
    }
}
#[test]
fn player_psi_affordability_and_range_match_32_native_boundaries() {
    let data = fixture();
    let content = content();
    let packet = spells::load_packet(
        &content.map(0).unwrap().combat.as_ref().unwrap().attacks,
        2204,
    )
    .unwrap();
    let cases = data["prerequisite_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 32);
    for case in cases {
        let power = case["power"].as_i64().unwrap() as i16;
        let mp = case["mp"].as_i64().unwrap() as i16;
        let distance = case["distance"].as_i64().unwrap() as i16;
        assert_eq!(
            mp >= player_psi::cost(&packet, power).unwrap(),
            case["affordable"].as_bool().unwrap()
        );
        let range = player_psi::range(&packet, power).unwrap();
        assert_eq!(range, case["range"].as_i64().unwrap() as i16);
        assert_eq!(
            player_psi::in_range((20, 20), (20 + distance, 20), range),
            case["in_range"].as_bool().unwrap()
        );
        assert_eq!(case["before"], case["after"]);
        assert_eq!(case["rng_draws"].as_u64().unwrap(), 0);
    }
}
