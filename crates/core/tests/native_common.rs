use crate::prelude::*;

pub fn assert_body_records(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    party: &Value,
    monsters: &Value,
) {
    crate::support::assert_bytes(
        &crate::support::record_bytes(warriors),
        &bytes(party),
        "party",
    );
    crate::support::assert_bytes(
        &crate::support::record_bytes(enemies),
        &bytes(monsters),
        "monsters",
    );
}

pub fn spell_sequence(
    fixture: &Value,
    case: &Value,
    caster: Option<usize>,
) -> (Combat, Vec<i16>, usize) {
    let before = &case["before"];
    let mut state: Session =
        serde_json::from_value(fixture["foundation"]["session"].clone()).unwrap();
    let mut warriors = snapshot_party(before);
    let mut combat = state.combat.take().unwrap();
    combat.enemies.records = crate::support::monsters(&before["records"]);
    combat.unavailable = before.n::<i8>("unavailable");
    let cache = before.bytes("first_item_cache");
    let mut random = random_state(before);
    let mut receipts = case.array("rng").iter();
    let mut sounds = vec![];
    let mut party_deaths = 0;
    for resolution in case.array("resolutions") {
        crate::support::advance_rng(&mut random, &mut receipts, resolution.n("draws_before"));
        let mut actual = random.clone();
        // The boundary supplies either one charged actor or the caller's retained roster.
        let captured = crate::support::monsters(&resolution["records_after"]);
        if let Some(caster) = caster {
            combat.enemies.records[caster] = captured[caster].clone();
        } else {
            combat.enemies.records = captured;
        }
        let result = resolve_bodies_with_item_cache(
            &warriors,
            &combat.enemies.records,
            &resolution.words("mask"),
            &DirectSpell {
                packet: &resolution.bytes("packet"),
                level: resolution.n::<i16>("spell_id") % 1000 / 100 - 1,
                count: resolution.n("count"),
                armor_filter: false,
                weapon_sound: 402,
            },
            &cache,
            &mut actual,
        )
        .unwrap();
        crate::support::advance_rng(&mut random, &mut receipts, resolution.n("draws_after"));
        assert_eq!(actual, random);
        assert!(result.race_excluded.is_empty());
        assert_eq!(
            result
                .targets
                .iter()
                .map(|(body, _)| *body)
                .collect::<Vec<_>>(),
            resolution
                .words("applied_mask")
                .iter()
                .enumerate()
                .filter_map(|(body, flag)| (*flag > 0).then_some(body))
                .collect::<Vec<_>>()
        );
        sounds.extend(body_spell_sounds(&result));
        let defeated: Vec<_> = result
            .targets
            .iter()
            .filter_map(|(body, outcome)| outcome.killed.then_some(*body as i16))
            .collect();
        warriors = result.warriors;
        combat.enemies.records = result.enemies;
        for body in defeated {
            party_deaths += usize::from(body < 10);
            centurions_core::combat::lifecycle::kill_body(&mut combat, &mut warriors, body)
                .unwrap();
        }
        assert_body_records(
            &warriors,
            &combat.enemies.records,
            &resolution["party_after"],
            &resolution["records_after"],
        );
    }
    assert_eq!(random, random_state(&case["after"]));
    crate::support::assert_bytes(
        &crate::support::record_bytes(&warriors),
        &case["after"].bytes("party"),
        "final party",
    );
    (combat, sounds, party_deaths)
}

pub fn script_mission(fixture: &Value) -> OpeningMission {
    OpeningMission {
        name: "native fixture".into(),
        start: (0, 0),
        tile_properties: fixture
            .get("properties")
            .map(crate::support::bytes)
            .unwrap_or_default(),
        default_tile: 1155,
        shared: Arc::new(StoryTables {
            door_scripts: fixture.bytes("scripts"),
            ..Default::default()
        }),
        ..crate::support::mission()
    }
}

pub fn random_state(snapshot: &Value) -> RandomState {
    RandomState {
        seed: snapshot.n::<u32>("seed"),
        draws: snapshot.n::<u64>("draws"),
    }
}
pub fn original_file(name: &str) -> Vec<u8> {
    std::fs::read(format!(
        "{}/../../NewCenturions/TheNewCenturions/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}
pub fn body_spell_sounds(result: &BodySpellResult) -> Vec<i16> {
    let mut ordered = result
        .targets
        .iter()
        .map(|(body, outcome)| (*body, outcome.sounds.clone()))
        .collect::<Vec<_>>();
    ordered.extend(result.race_excluded.iter().map(|body| (*body, vec![651])));
    ordered.sort_by_key(|(body, _)| *body);
    result
        .sounds
        .iter()
        .copied()
        .chain(ordered.into_iter().flat_map(|(_, sounds)| sounds))
        .collect()
}
pub fn compare_health_round(input: &str) {
    let fixture: serde_json::Value = serde_json::from_str(input).unwrap();
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let detailed = case.get("before_map").is_some();
        let mut warriors = crate::support::warriors(&case["warriors"]);
        let mut enemies = vec![MonsterRecord {
            bytes: case.bytes("enemy"),
        }];
        let mut random = RandomState::new(1);
        let mut combat = centurions_core::combat::Combat {
            battle: 2,
            map: centurions_core::combat::TacticalMap {
                tiles: case
                    .get("before_map")
                    .map(crate::support::words)
                    .unwrap_or_else(|| vec![1155; 8100]),
                shift: (0, 0),
                edge_word: 0,
            },
            squad: centurions_core::combat::SquadPlacement {
                positions: (0..6)
                    .map(|i| (20 + if detailed { i } else { 0 }, 20))
                    .collect(),
                under_tiles: vec![1155; 6],
            },
            enemies: centurions_core::combat::EnemySetup {
                records: enemies.clone(),
                positions: vec![if detailed { (30, 30) } else { (20, 20) }],
            },
            order: case
                .get("before_order")
                .map(crate::support::words)
                .unwrap_or_else(|| vec![-1; 110]),
            cursor: 0,
            round: 1,
            ..crate::support::battle()
        };
        let events =
            centurions_core::combat::runtime::reduce_round(&mut combat, &mut warriors, &mut random)
                .unwrap();
        if let Some(sounds) = case
            .get("tick_sounds")
            .or_else(|| case.get("bleeding_sounds"))
        {
            assert_eq!(
                events,
                sounds
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|id| centurions_core::Event::Sound {
                        resource_id: id.as_i64().unwrap() as i16
                    })
                    .collect::<Vec<_>>()
            );
        }
        for (actual, key) in [(&combat.map.tiles, "map"), (&combat.order, "order")] {
            if let Some(expected) = case.get(key) {
                assert_eq!(
                    crate::support::word_bytes(actual),
                    bytes(expected),
                    "{key} case{index}"
                );
            }
        }
        enemies = combat.enemies.records;
        initialize_turn(
            case.n::<i16>("body"),
            &mut warriors,
            &mut enemies,
            &[12, 14, 12, 14, 10, 12, 10, 10, 14, 16, 0, 12],
            &mut random,
        )
        .unwrap();
        assert_eq!(
            crate::support::record_bytes(&warriors),
            case.bytes("after_warriors"),
            "body {}",
            case["body"]
        );
        assert_eq!(enemies[0].bytes, case.bytes("after_enemy"));
        crate::support::expect(random.seed, &case["seed"]);
        crate::support::expect(random.draws, &case["draws"]);
    }
}
pub fn snapshot_party(snapshot: &Value) -> Vec<Warrior> {
    crate::support::warriors(&snapshot["party"])
}
pub fn scalar_words(value: &Value) -> Vec<i16> {
    crate::support::NativeScalar::from_native(value)
}
pub fn buffer_words(fixture: &Value, value: &Value) -> Vec<i16> {
    crate::support::words(&crate::support::buffer(fixture, value))
}
pub fn melee_combat(records: &[u8]) -> Combat {
    Combat {
        battle: 117,
        map: TacticalMap {
            tiles: vec![1155; 8100],
            shift: (0, 0),
            edge_word: 0,
        },
        squad: SquadPlacement {
            positions: (0..6).map(|i| (40 + i, 40)).collect(),
            under_tiles: vec![1155; 6],
        },
        enemies: EnemySetup {
            records: crate::support::monsters(records),
            positions: vec![(20, 20), (21, 20), (40, 40)],
        },
        attack_identity: Some([2, 4, 6]),
        ..crate::support::battle()
    }
}
pub fn raw_word(raw: &[u8], offset: usize) -> i16 {
    i16::from_be_bytes(raw[offset..offset + 2].try_into().unwrap())
}
pub fn patch_bytes(raw: &mut [u8], values: &serde_json::Value) {
    for change in values.as_array().unwrap() {
        raw[change[0].as_u64().unwrap() as usize] = change[1].as_u64().unwrap() as u8;
    }
}
pub fn patched(mut raw: Vec<u8>, values: &serde_json::Value) -> Vec<u8> {
    patch_bytes(&mut raw, values);
    raw
}
pub fn health_event(event: &HealthEvent) -> serde_json::Value {
    match event {
        HealthEvent::Animation { warrior, effect } => {
            serde_json::json!(["animation", warrior, effect])
        }
        HealthEvent::Sound(sound) => serde_json::json!(["sound", sound, 0]),
        HealthEvent::Defeated { warrior, permanent } => {
            serde_json::json!(["defeat", warrior, i16::from(*permanent)])
        }
        HealthEvent::PartyLost => serde_json::json!(["party_loss", 0, 0]),
        HealthEvent::Dialogue(dialogue) => serde_json::json!(["dialogue", dialogue, 0]),
        HealthEvent::Flash { address, sound } => serde_json::json!(["flash", address, sound]),
    }
}
pub fn snapshot_ammunition(snapshot: &Value) -> Vec<i32> {
    crate::support::longs(&snapshot["ammunition"])
}
pub fn snapshot_rng(snapshot: &Value) -> RandomState {
    RandomState {
        seed: snapshot.n::<u32>("rng_seed"),
        draws: snapshot.n::<u64>("rng_draws"),
    }
}
pub fn snapshot_enemies(snapshot: &Value) -> Vec<MonsterRecord> {
    crate::support::monsters(&snapshot["records"])
}
pub use crate::support::positions as decoded_positions;
pub fn snapshot_word(snapshot: &Value, key: &str) -> i16 {
    snapshot[key].as_i64().unwrap() as i16
}
pub fn foundation_session(fixture: &Value, case: &Value) -> Session {
    let mut state = serde_json::from_value(
        fixture["foundations"][case.n::<usize>("foundation")]["session"].clone(),
    )
    .unwrap();
    if case["before"].get("current_npc_index").is_some() {
        crate::snapshots::actor_work(&mut state, &case["before"]);
    }
    state
}
pub fn restore_party(state: &mut Session, snapshot: &Value) {
    for (warrior, record) in state
        .warriors
        .iter_mut()
        .zip(snapshot.bytes("party").as_chunks::<3138>().0)
    {
        warrior.bytes = Warrior::read(record).unwrap().bytes;
    }
}
pub fn first_difference(actual: &Value, expected: &Value, path: &str) -> Option<String> {
    if actual == expected {
        return None;
    }
    match (actual, expected) {
        (Value::Object(a), Value::Object(b)) if a.len() == b.len() => {
            for (key, value) in a {
                if let Some(other) = b.get(key) {
                    if let Some(error) = first_difference(value, other, &format!("{path}.{key}")) {
                        return Some(error);
                    }
                } else {
                    return Some(format!("{path}.{key}: unexpected field"));
                }
            }
        }
        (Value::Array(a), Value::Array(b)) if a.len() == b.len() => {
            for (index, (v, w)) in a.iter().zip(b).enumerate() {
                if let Some(error) = first_difference(v, w, &format!("{path}[{index}]")) {
                    return Some(error);
                }
            }
        }
        (Value::Array(a), Value::Array(b)) => {
            return Some(format!("{path}: lengths {} != {}", a.len(), b.len()));
        }
        _ => return Some(format!("{path}: {actual} != {expected}")),
    }
    Some(format!("{path}: fields differ"))
}
pub fn refresh_item_work(state: &mut Session, content: &Content) {
    // Compose the omitted combatupdate2 writer after the native Getup/prefix:
    // NPC WORD4e or party ca/e4 (BYTE9dd) selects the complete loaditem record.
    let body = state.combat.as_ref().unwrap().active_body().unwrap();
    let id = if body >= 10 {
        word(
            &state.combat.as_ref().unwrap().enemies.records[(body - 10) as usize].bytes,
            0x4e,
        )
        .unwrap()
    } else {
        let warrior = &state.warriors[body as usize].bytes;
        word(warrior, if warrior[0x9dd] == 0 { 0xca } else { 0xe4 }).unwrap()
    };
    if body >= 10 && id == 0 {
        return;
    }
    let mut record = content.items[usize::from(id.unsigned_abs())].clone();
    let class = word(&record, 26).unwrap().wrapping_abs();
    record[26..28].copy_from_slice(&class.to_be_bytes());
    state.loaded_item_field32 = Some(word(&record, 32).unwrap());
    state.ammunition_work.item = Some(record);
}
