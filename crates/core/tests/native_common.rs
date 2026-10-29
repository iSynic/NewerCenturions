use crate::prelude::*;

pub fn earned_receipt(name: &str, index: usize) -> Value {
    crate::support::fixture(&format!("{name}-receipt-{}.json", [12, 1][index]))
}

/// The earned receipts were recorded by an engine that let an exhausted squad
/// move; their starting sessions sit at fatigue 135, where buttonchoice refuses
/// every move. Starting rested keeps the recorded prefixes reachable without
/// changing the frozen receipt bytes; no recorded event carries fatigue.
pub fn earned_start(receipt: &Value) -> Session {
    let mut state: Session = serde_json::from_value(receipt["starting_session"].clone()).unwrap();
    assert!(state.native_import.is_none());
    state.exploration.as_mut().unwrap().fatigue = 4;
    state
}

/// The recording held fatigue at its 135 clamp for the whole prefix, so the
/// native-compared actions that follow start from the receipt's own value.
pub fn earned_rejoin(state: &mut Session, receipt: &Value) {
    state.exploration.as_mut().unwrap().fatigue =
        receipt["starting_session"]["exploration"]["fatigue"]
            .as_i64()
            .unwrap() as i16;
}

/// Historical command recordings omitted showresults151a88 for the Shield packet.
/// Restore that source-owned event without changing the frozen recording bytes.
pub fn shield_result_events(events: Vec<Event>) -> Vec<Event> {
    let mut result = vec![];
    for event in events {
        if let Event::SpellResolved {
            spell: 1305,
            target,
            damage: 0,
            ..
        } = &event
        {
            result.push(Event::Feedback(
                centurions_core::feedback::NativeFeedback::CombatResult {
                    body: *target,
                    code: 13,
                    amount: 0,
                    audible: false,
                },
            ));
        }
        result.push(event);
    }
    result
}

/// Reconstructed prefix recordings predate the native death-feedback correction and
/// the target key's firstt(76, 77). Preserve those bytes; showresults158ee0 supplies
/// the omitted non165 NPC cue, and combatchoice 11b026 the tutorial.
pub fn prefix_result_events(
    state: &Session,
    after: &Session,
    command: &Command,
    events: Vec<Event>,
) -> Vec<Event> {
    let mut result = vec![];
    if matches!(
        command,
        Command::ShootEnemy { .. } | Command::ShootAt { .. }
    ) {
        result.push(Event::Feedback(
            centurions_core::feedback::NativeFeedback::Tutorial {
                first: 76,
                last: 77,
            },
        ));
    }
    for event in shield_result_events(events) {
        if matches!(command, Command::ShootEnemy { .. })
            && let Event::AttackResolved {
                attacker,
                target,
                defeated: true,
                ..
            } = &event
            && *attacker < 6
            && *target >= 10
            && state.warriors[*attacker as usize]
                .record()
                .equipped_weapon()
                .unwrap()
                != 165
        {
            let sound = state.combat.as_ref().unwrap().enemies.records[(*target - 10) as usize]
                .record()
                .death_sound()
                .unwrap();
            if sound != 0 {
                result.push(Event::Sound { resource_id: sound });
            }
        }
        result.push(event);
    }
    if let Some(body) = state.combat.as_ref().and_then(Combat::active_body) {
        let guarding = if body < 6 {
            after.warriors[body as usize].record().guarding().unwrap() != 0
        } else {
            after.combat.as_ref().is_some_and(|combat| {
                combat.enemies.records[(body - 10) as usize]
                    .record()
                    .guarding()
                    != 0
            })
        };
        if guarding {
            return turn_guard_events(result, body);
        }
    }
    result
}

pub fn guard_result(body: i16) -> Event {
    Event::Feedback(centurions_core::feedback::NativeFeedback::CombatResult {
        body,
        code: -5,
        amount: 0,
        audible: false,
    })
}

/// Older receipts omit Getup125bd2's result-only cue before the next turn.
/// Its guard field remains independently compared in their native records.
pub fn turn_guard_events(mut events: Vec<Event>, body: i16) -> Vec<Event> {
    if let Some(mut at) = events
        .iter()
        .position(|event| matches!(event, Event::TurnStarted { .. }))
    {
        if at > 0 && matches!(events[at - 1], Event::Sound { resource_id: 138 }) {
            at -= 1;
        }
        events.insert(at, guard_result(body));
    }
    events
}

pub fn assert_body_records(
    warriors: &[Warrior],
    enemies: &[MonsterRecord],
    party: &Value,
    monsters: &Value,
) {
    crate::support::native_expect! {
        bytes crate::support::record_bytes(warriors) => party, "party";
        bytes crate::support::record_bytes(enemies) => monsters, "monsters";
    }
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

/// The RNG state recorded under a case's seed and draw-count keys.
pub fn random_at(value: &Value, seed: &str, draws: &str) -> RandomState {
    RandomState {
        seed: value.n(seed),
        draws: value.n(draws),
    }
}
pub fn random_state(snapshot: &Value) -> RandomState {
    random_at(snapshot, "seed", "draws")
}
pub fn original_file(name: &str) -> Vec<u8> {
    std::fs::read(
        crate::support::root()
            .join("NewCenturions/TheNewCenturions")
            .join(name),
    )
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
        crate::support::assert_rng(&random, &case["seed"], &case["draws"]);
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
    let mut combat = crate::support::battle();
    combat.battle = 117;
    combat.map.tiles.fill(1155);
    combat.squad.positions = (0..6).map(|i| (40 + i, 40)).collect();
    combat.enemies.records = crate::support::monsters(records);
    combat.enemies.positions = vec![(20, 20), (21, 20), (40, 40)];
    combat.attack_identity = [2, 4, 6];
    combat
}

pub fn neighbor_combat(case: &Value, member: i16) -> Combat {
    let mut combat = crate::support::battle();
    combat.battle = 117;
    combat.map.tiles = case.words("map_before");
    combat.squad.positions = (0..6).map(|i| (40 + i, 40)).collect();
    combat.squad.positions[member as usize] = (21, 20);
    combat.enemies.records = vec![MonsterRecord {
        bytes: case.bytes("enemy_before"),
    }];
    combat.enemies.positions = vec![(20, 20)];
    combat.order = case.words("order_before");
    combat
}
pub fn raw_word(raw: &[u8], offset: usize) -> i16 {
    i16::from_be_bytes(raw[offset..offset + 2].try_into().unwrap())
}
pub fn patched(mut raw: Vec<u8>, values: &serde_json::Value) -> Vec<u8> {
    crate::support::patch_bytes(&mut raw, values);
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
    random_at(snapshot, "rng_seed", "rng_draws")
}
pub use crate::support::positions as decoded_positions;
pub fn snapshot_word(snapshot: &Value, key: &str) -> i16 {
    snapshot[key].as_i64().unwrap() as i16
}
/// Declared probe/source inputs only: selected447196, attempts438780, inhibit43877e.
/// Call before execution, never to fill unknown runtime/migration or expected-state history.
pub fn npc_command_input(state: &mut Session, selected: usize, words: [i16; 2]) {
    source_victory_context(state);
    state.combat.as_mut().unwrap().selected_enemy = selected;
    state.native_work.selected_enemy = selected;
    state.native_work.script_enemy = state.combat.as_ref().unwrap().script_enemy;
    state.native_work.npc_work.attempts = words[0];
    state.native_work.npc_work.inhibit = words[1];
}

pub fn foundation_session(fixture: &Value, case: &Value) -> Session {
    let mut state = crate::support::source_frame(
        fixture["foundations"][case.n::<usize>("foundation")]["session"].clone(),
    );
    // Only the current-terrain receipt declares this NPC entry; loaded-shot foundations do not.
    if let Some(selected) = case["before"].get("current_npc_index") {
        npc_command_input(&mut state, selected.as_u64().unwrap() as usize, [0, 0]);
    }
    state
}
pub fn restore_party(state: &mut Session, snapshot: &Value) {
    for (warrior, record) in state
        .warriors
        .iter_mut()
        .zip(snapshot.bytes("party").as_chunks::<WARRIOR_BYTES>().0)
    {
        warrior.bytes = Warrior::read(record).unwrap().bytes;
    }
}
pub fn first_difference(actual: &Value, expected: &Value, path: &str) -> Option<String> {
    if actual == expected {
        return None;
    }
    match (actual, expected) {
        (Value::Object(a), Value::Object(b)) => {
            if a.len() != b.len() {
                return Some(format!("{path}: keys {:?} != {:?}", a.keys(), b.keys()));
            }
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
pub fn earned_inventory_work(index: usize) -> centurions_content::InventoryRecord {
    centurions_content::InventoryRecord(
        crate::support::fixture("earned-crate-equipment.json")["cases"][5 + index]["picked"]
            .bytes("selected_record")
            .try_into()
            .unwrap(),
    )
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
    state.native_work.loaded_item_field32 = word(&record, 32).unwrap();
    state.native_work.ammunition_work.item = Some(record);
}

// Shared controlled geometry for independent movement and complete actor-decision receipts.
pub fn enemy_movement_case(case: &Value) -> Combat {
    let dx = case.n::<i16>("dx");
    let dy = case.n::<i16>("dy");
    let layout = case.n::<i64>("layout");
    let mut tiles = vec![1155; 8100];
    if layout == 1 || layout == 3 {
        tiles[((25 + dx) * 90 + 25 + dy) as usize] = if layout == 3 { 1000 } else { 1061 };
    }
    if layout == 2 {
        for x in 24..=26 {
            for y in 24..=26 {
                tiles[x * 90 + y] = 1061;
            }
        }
    }
    tiles[25 * 90 + 25] = 10;
    tiles[((25 + dx * 4) * 90 + 25 + dy * 4) as usize] = 0;
    Combat {
        battle: 2,
        map: TacticalMap {
            tiles,
            shift: (0, 0),
            edge_word: 0,
        },
        squad: SquadPlacement {
            positions: vec![(20 + dx * 4, 20 + dy * 4); 6],
            under_tiles: vec![1155; 6],
        },
        enemies: EnemySetup {
            records: vec![MonsterRecord {
                bytes: case.bytes("input"),
            }],
            positions: vec![(20, 20)],
        },
        order: vec![-1, 10],
        ..crate::support::battle()
    }
}

/// Shared controlled Psi caller state; captures retain their separate round ownership.
pub fn psi_work(snapshot: &Value) -> WeaponWork {
    crate::snapshots::weapon_work(&|key| snapshot.bytes(key))
}
pub fn psi_snapshot(content: &Content, snapshot: &Value) -> Session {
    let mut state = crate::support::mission_session(content);
    state.warriors = snapshot_party(snapshot);
    state.native_work.psi_memory = snapshot.bytes("memory");
    state.party_buffs = snapshot.bytes("party_buffs");
    state.native_work.loaded_item_field32 = snapshot.n("item_context");
    state.native_work.attack_count = snapshot.n("count");
    state.native_work.weapon_work = psi_work(snapshot);
    state.native_work.attack_packet = snapshot.bytes("packet");
    state.native_work.attack_identity = snapshot.words("identity").try_into().unwrap();
    state.random = random_state(snapshot);
    let mut combat = crate::support::psi_frame::combat(snapshot);
    combat.weapon_work = state.native_work.weapon_work;
    state.combat = Some(combat);
    state.phase = Phase::Combat;
    source_victory_context(&mut state);
    state
}

/// A constructed region table keeps its land's authored tile set (byte 520), which
/// selects the live tile properties.
pub fn keep_tile_set(state: &mut Session, content: &Content) {
    let position = state.exploration.as_mut().unwrap();
    position.region_state[0x208] = content.map(position.map).unwrap().regions[0x208];
}

pub use crate::support::{source_victory_context, source_victory_context_recorded};

/// Loaded-shot record foundation with the declared source counter history.
pub fn loaded_shot_frame(index: usize) -> Session {
    let fixture = crate::support::fixture("earned-loaded-shot-replay.json");
    crate::support::source_frame(fixture["foundations"][index]["session"].clone())
}
