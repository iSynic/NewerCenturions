use centurions_content::{Content, Warrior, combat::MonsterRecord, word};
use centurions_core::{
    Command, Event, Phase, RandomState, Session, apply,
    combat::{
        Combat, EnemySetup, SquadPlacement, TacticalMap,
        player_psi::{self, CastRequest},
        runtime, spells,
        weapon_work::WeaponWork,
    },
};
use serde_json::Value;

fn bytes(value: &Value) -> Vec<u8> {
    value
        .as_str()
        .unwrap()
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}

fn words(value: &Value) -> Vec<i16> {
    bytes(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| i16::from_be_bytes(*p))
        .collect()
}

fn positions(value: &Value) -> Vec<(i16, i16)> {
    words(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| (p[0], p[1]))
        .collect()
}

fn fixture() -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../evidence/fixtures/warm-psi-work.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn content() -> Content {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap()
}

fn native_work(s: &Value) -> WeaponWork {
    let weapon = bytes(&s["weapon_work"]);
    let ammo = bytes(&s["ammo_work"]);
    let work = WeaponWork {
        sound: Some(word(&weapon, 0x0e).unwrap()),
        weapon_armor_filter: Some(word(&weapon, 0x88).unwrap()),
        ammunition_armor_filter: Some(word(&ammo, 0x88).unwrap()),
    };
    let projection = &s["work_projection"];
    assert_eq!(
        work.sound.unwrap(),
        projection["sound"].as_i64().unwrap() as i16
    );
    assert_eq!(
        work.weapon_armor_filter.unwrap(),
        projection["weapon_armor_filter"].as_i64().unwrap() as i16
    );
    assert_eq!(
        work.ammunition_armor_filter.unwrap(),
        projection["ammunition_armor_filter"].as_i64().unwrap() as i16
    );
    assert_eq!(work.armor_filter().unwrap(), projection["armor_filter"]);
    work
}

fn state(content: &Content, s: &Value) -> Session {
    let mut state = Session::new(content);
    apply(Command::ImportStarters, &mut state, content).unwrap();
    apply(
        Command::EnterMission {
            name: "The Thunder of Guns".into(),
        },
        &mut state,
        content,
    )
    .unwrap();
    state.warriors = bytes(&s["party"])
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|b| Warrior::read(b).unwrap())
        .collect();
    state.psi_memory = Some(bytes(&s["memory"]));
    state.party_buffs = bytes(&s["party_buffs"]);
    state.loaded_item_field32 = Some(s["item_context"].as_i64().unwrap() as i16);
    state.attack_count = s["count"].as_i64().unwrap() as i16;
    state.weapon_work = native_work(s);
    state.attack_packet = Some(bytes(&s["packet"]));
    state.attack_identity = Some(words(&s["identity"]).try_into().unwrap());
    state.random = RandomState {
        seed: s["seed"].as_u64().unwrap() as u32,
        draws: s["draws"].as_u64().unwrap(),
    };
    state.combat = Some(Combat {
        battle: 85,
        map: TacticalMap {
            tiles: words(&s["map"]),
            shift: (0, 0),
            edge_word: 0,
        },
        squad: SquadPlacement {
            positions: positions(&s["party_positions"]),
            under_tiles: vec![1155; 6],
        },
        enemies: EnemySetup {
            records: bytes(&s["records"])
                .as_chunks::<288>()
                .0
                .iter()
                .map(|b| MonsterRecord { bytes: b.to_vec() })
                .collect(),
            positions: positions(&s["monster_positions"]),
        },
        order: words(&s["order"]),
        cursor: s["cursor"].as_u64().unwrap() as usize,
        round: s["round"].as_i64().unwrap() as i16,
        round_scripts: Default::default(),
        death_queue: Default::default(),
        spell_fields: bytes(&s["fields"]),
        field_used: bytes(&s["used"]),
        attack_packet: Some(bytes(&s["packet"])),
        attack_identity: Some(words(&s["identity"]).try_into().unwrap()),
        weapon_work: state.weapon_work,
        unavailable: s["unavailable"].as_i64().unwrap() as i8,
    });
    state.phase = Phase::Combat;
    state
}

fn assert_bytes(actual: &[u8], expected: &[u8], label: &str) {
    assert_eq!(actual.len(), expected.len(), "{label} length");
    if let Some(i) = actual.iter().zip(expected).position(|(a, e)| a != e) {
        panic!("{label} byte{i:#x}: {} != {}", actual[i], expected[i]);
    }
}

fn assert_snapshot(state: &Session, expected: &Value, events: &[Event], label: &str) {
    let c = state.combat.as_ref().unwrap();
    assert_bytes(
        &state
            .warriors
            .iter()
            .flat_map(|w| w.bytes.iter().copied())
            .collect::<Vec<_>>(),
        &bytes(&expected["party"]),
        &format!("{label} full party"),
    );
    assert_bytes(
        &c.enemies
            .records
            .iter()
            .flat_map(|m| m.bytes.iter().copied())
            .collect::<Vec<_>>(),
        &bytes(&expected["records"]),
        &format!("{label} full enemies"),
    );
    assert_eq!(c.map.tiles, words(&expected["map"]), "{label} map");
    assert_eq!(c.squad.positions, positions(&expected["party_positions"]));
    assert_eq!(
        c.enemies.positions,
        positions(&expected["monster_positions"])
    );
    assert_eq!(c.spell_fields, bytes(&expected["fields"]), "{label} fields");
    assert_eq!(c.field_used, bytes(&expected["used"]), "{label} usage");
    assert_eq!(
        c.attack_packet.as_ref().unwrap(),
        &bytes(&expected["packet"])
    );
    assert_eq!(
        c.attack_identity.unwrap().to_vec(),
        words(&expected["identity"])
    );
    assert_eq!(
        state.psi_memory.as_ref().unwrap(),
        &bytes(&expected["memory"])
    );
    assert_eq!(state.party_buffs, bytes(&expected["party_buffs"]));
    assert_eq!(state.loaded_item_field32.unwrap(), expected["item_context"]);
    assert_eq!(state.attack_count, expected["count"]);
    assert_eq!(c.unavailable, expected["unavailable"]);
    assert_eq!(c.order, words(&expected["order"]));
    assert_eq!(c.cursor, expected["cursor"]);
    assert_eq!(c.round, expected["round"]);
    assert_eq!(c.weapon_work, native_work(expected));
    assert_eq!(state.weapon_work, c.weapon_work);
    assert_eq!(state.random.seed, expected["seed"]);
    assert_eq!(state.random.draws, expected["draws"]);
    let sounds: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            Event::Sound { resource_id } => Some(*resource_id),
            _ => None,
        })
        .collect();
    assert_eq!(
        serde_json::json!(sounds),
        expected["sounds"],
        "{label} audio"
    );
    let mut receipt = RandomState::new(12);
    for draw in expected["rng_records"].as_array().unwrap() {
        assert_eq!(
            receipt.bound(draw["bound"].as_i64().unwrap() as i16),
            draw["result"].as_i64().unwrap() as i16,
            "{label} native RNG receipt"
        );
    }
    assert_eq!(receipt, state.random, "{label} RNG receipt state");
}

#[test]
fn native_loading_and_preparation_preserve_independent_weapon_and_ammunition_fields() {
    let content = content();
    let fixture = fixture();
    for case in fixture["cases"].as_array().unwrap() {
        let mut work = native_work(&case["unwarmed"]);
        let weapon = &content.items[case["warm_weapon"].as_u64().unwrap() as usize];
        let ammunition = &content.items[case["warm_ammo"].as_u64().unwrap() as usize];
        work.load_pair(weapon, ammunition).unwrap();
        assert_eq!(work, native_work(&case["loaded"]));
        work.prepare(weapon, ammunition).unwrap();
        assert_eq!(work, native_work(&case["prepared"]));
        for phase in ["before", "charged", "targeted", "cast", "after", "collided"] {
            assert_eq!(work, native_work(&case[phase]));
            for buffer in ["weapon_work", "ammo_work"] {
                assert_eq!(case[phase][buffer], case["prepared"][buffer]);
            }
        }
    }
}

#[test]
fn warmed_cast_automatic_initiative_and_field_collision_match_complete_native_cases() {
    let content = content();
    let mission = content.map(0).unwrap();
    let fixture = fixture();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 4);
    for case in fixture["cases"].as_array().unwrap() {
        let label = format!(
            "warm weapon{} ammo{}",
            case["warm_weapon"], case["warm_ammo"]
        );
        let mut state = state(&content, &case["before"]);
        let original = state.combat.as_ref().unwrap().clone();
        let point = words(&case["casts"][0]["targets"]);
        let mut mask_rng = state.random.clone();
        let c = state.combat.as_ref().unwrap();
        let packet = c.attack_packet.as_ref().unwrap();
        let mask = spells::target_mask(
            c,
            &state.warriors,
            &mission.areas,
            &spells::SpellTarget {
                first: point[0],
                second: point[1],
                kind: 3,
            },
            &spells::SpellContext {
                packet,
                id: 2101,
                count: 1,
                caster: 1,
                initiative: c.cursor as i16,
                collision: false,
            },
            &c.spell_fields,
            &mut mask_rng,
        )
        .unwrap();
        assert_eq!(mask.mask.to_vec(), words(&case["resolutions"][0]["mask"]));
        assert_eq!(mask.fields, bytes(&case["field_events"][1]["fields"]));
        assert_eq!(mask_rng.draws, case["resolutions"][0]["draws_before"]);
        let mut events = player_psi::cast(
            state.combat.as_mut().unwrap(),
            &mut state.warriors,
            mission,
            &mut state.random,
            state.psi_memory.as_mut().unwrap(),
            0,
            CastRequest {
                spell: 2101,
                power: 1,
                target: Some((point[0] - 5, point[1] - 5)),
            },
        )
        .unwrap_or_else(|error| panic!("{label}: {error:?}"));
        assert_snapshot(&state, &case["cast"], &events, &format!("{label} cast"));
        events.extend(runtime::end_turn(&mut state, &content).unwrap());
        assert_eq!(state.combat.as_ref().unwrap().active_body(), Some(10));
        assert_snapshot(&state, &case["after"], &events, &format!("{label} advance"));
        events.extend(
            runtime::collide_fields(
                state.combat.as_mut().unwrap(),
                &mut state.warriors,
                mission,
                &mut state.attack_count,
                &mut state.random,
                10,
            )
            .unwrap(),
        );
        assert_snapshot(
            &state,
            &case["collided"],
            &events,
            &format!("{label} collision"),
        );
        let c = state.combat.as_ref().unwrap();
        assert_eq!(c.death_queue, original.death_queue);
        assert_eq!(c.round_scripts, original.round_scripts);
        let expected_filter = i16::from(c.weapon_work.armor_filter().unwrap());
        for call in case["damage_arguments"].as_array().unwrap() {
            assert_eq!(call["arguments"][4], expected_filter);
        }
    }
}
