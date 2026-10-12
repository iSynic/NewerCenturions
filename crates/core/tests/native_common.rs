use crate::prelude::*;
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
        let mut warriors: Vec<_> = case
            .bytes("warriors")
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|w| Warrior::read(w).unwrap())
            .collect();
        let mut enemies = vec![MonsterRecord {
            bytes: case.bytes("enemy"),
        }];
        let mut random = RandomState::new(1);
        let mut combat = centurions_core::combat::Combat {
            battle: 2,
            map: centurions_core::combat::TacticalMap {
                tiles: case
                    .bytes("before_map")
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|v| i16::from_be_bytes(*v))
                    .collect(),
                shift: (0, 0),
                edge_word: 0,
            },
            squad: centurions_core::combat::SquadPlacement {
                positions: (0..6).map(|i| (20 + i, 20)).collect(),
                under_tiles: vec![1155; 6],
            },
            enemies: centurions_core::combat::EnemySetup {
                records: enemies.clone(),
                positions: vec![(30, 30)],
            },
            order: case
                .bytes("before_order")
                .as_chunks::<2>()
                .0
                .iter()
                .map(|v| i16::from_be_bytes(*v))
                .collect(),
            cursor: 0,
            round: 1,
            ..crate::support::battle()
        };
        let events =
            centurions_core::combat::runtime::reduce_round(&mut combat, &mut warriors, &mut random)
                .unwrap();
        if let Some(sounds) = case.get("tick_sounds") {
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
        assert_eq!(
            crate::support::word_bytes(&combat.map.tiles),
            case.bytes("map"),
            "map case {index}"
        );
        assert_eq!(
            crate::support::word_bytes(&combat.order),
            case.bytes("order"),
            "order case {index}"
        );
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
    snapshot
        .bytes("party")
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|record| Warrior::read(record).unwrap())
        .collect()
}
pub fn scalar_words(value: &Value) -> Vec<i16> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_i64().unwrap() as i16)
        .collect()
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
pub fn medical_continuation(case: &Value) -> MedicalContinuation {
    MedicalContinuation {
        origin: MedicalOrigin::LearnedHeal,
        actor: 5,
        item: 0,
        spell: 1101,
        power: case.n::<i16>("power"),
    }
}
pub fn same_unmodified_combat(before: &Session, after: &Session) {
    assert_eq!(after.ammunition, before.ammunition);
    assert_eq!(after.crate_items, before.crate_items);
    assert_eq!(after.booty_items, before.booty_items);
    assert_eq!(after.psi_memory, before.psi_memory);
    assert_eq!(after.party_buffs, before.party_buffs);
    assert_eq!(after.health_context, before.health_context);
    let actual = after.combat.as_ref().unwrap();
    let previous = before.combat.as_ref().unwrap();
    assert_eq!(actual.enemies, previous.enemies);
    assert_eq!(actual.spell_fields, previous.spell_fields);
    assert_eq!(actual.field_used, previous.field_used);
    assert_eq!(actual.death_queue, previous.death_queue);
    assert_eq!(actual.weapon_work, previous.weapon_work);
    assert_eq!(actual.cursor, previous.cursor);
    assert_eq!(actual.round, previous.round);
    assert_eq!(actual.round_scripts, previous.round_scripts);
}
pub fn patch_bytes(raw: &mut [u8], values: &serde_json::Value) {
    for change in values.as_array().unwrap() {
        raw[change[0].as_u64().unwrap() as usize] = change[1].as_u64().unwrap() as u8;
    }
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
    }
}
pub fn snapshot_ammunition(snapshot: &Value) -> Vec<i32> {
    snapshot
        .bytes("ammunition")
        .as_chunks::<4>()
        .0
        .iter()
        .map(|record| i32::from_be_bytes(*record))
        .collect()
}
pub fn snapshot_rng(snapshot: &Value) -> RandomState {
    RandomState {
        seed: snapshot.n::<u32>("rng_seed"),
        draws: snapshot.n::<u64>("rng_draws"),
    }
}
pub fn snapshot_enemies(snapshot: &Value) -> Vec<MonsterRecord> {
    snapshot
        .bytes("records")
        .as_chunks::<288>()
        .0
        .iter()
        .map(|record| MonsterRecord {
            bytes: record.to_vec(),
        })
        .collect()
}
pub fn decoded_positions(value: &Value) -> Vec<(i16, i16)> {
    words(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| (p[0], p[1]))
        .collect()
}
pub fn snapshot_word(snapshot: &Value, key: &str) -> i16 {
    snapshot[key].as_i64().unwrap() as i16
}
pub fn foundation_session(fixture: &Value, case: &Value) -> Session {
    serde_json::from_value(fixture["foundations"][case.n::<usize>("foundation")]["session"].clone())
        .unwrap()
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
