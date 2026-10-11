use centurions_content::{Content, InventoryRecord, Warrior, combat::MonsterRecord, digest, word};
use centurions_core::{
    Command, Event, RandomState, Session,
    ammunition::WorkBuffers,
    apply,
    combat::{death_scripts::DeathQueue, round_stories::RoundScripts, weapon_work::WeaponWork},
    load, save,
    story_health::HealthContext,
};
use serde_json::Value;

fn fixture() -> Value {
    let text = include_str!("../../../evidence/fixtures/reaction-work-refresh.json");
    assert_eq!(
        digest(text.as_bytes()),
        "92df3bf32ef08f19c3b6732c5918837b0b406e5f53fe522b0ad26faa6a75034d"
    );
    serde_json::from_str(text).unwrap()
}

fn content() -> Content {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap()
}

fn bytes(fixture: &Value, value: &Value) -> Vec<u8> {
    let text = value.as_str().unwrap_or_else(|| {
        fixture["shared_rule_buffers"][value["shared_buffer_sha256"].as_str().unwrap()]
            .as_str()
            .unwrap()
    });
    let bytes: Vec<_> = (0..text.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).unwrap())
        .collect();
    if value.is_object() {
        assert_eq!(bytes.len(), value["byte_length"].as_u64().unwrap() as usize);
        assert_eq!(
            digest(&bytes),
            value["shared_buffer_sha256"].as_str().unwrap()
        );
    }
    bytes
}

fn words(fixture: &Value, value: &Value) -> Vec<i16> {
    bytes(fixture, value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|word| i16::from_be_bytes(*word))
        .collect()
}

fn number(snapshot: &Value, key: &str) -> i16 {
    snapshot[key].as_i64().unwrap() as i16
}

fn random(snapshot: &Value) -> RandomState {
    RandomState {
        seed: snapshot["seed"].as_u64().unwrap() as u32,
        draws: snapshot["draws"].as_u64().unwrap(),
    }
}

fn project_party(state: &mut Session, fixture: &Value, snapshot: &Value) {
    let party = bytes(fixture, &snapshot["party"]);
    assert_eq!(party.len(), 6 * 3138);
    for (warrior, record) in state.warriors.iter_mut().zip(party.as_chunks::<3138>().0) {
        let captured = Warrior::read(record).unwrap();
        assert_eq!(warrior.name, captured.name);
        warrior.bytes = captured.bytes;
    }
}

fn initial(fixture: &Value, case: &Value) -> Session {
    let background: Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/player-psi.json")).unwrap();
    // The mission supplies valid unobserved background for controlled native
    // inputs. These original helper controls do not describe an earned route.
    let mut state: Session =
        serde_json::from_value(background["route_foundation"].clone()).unwrap();
    let exploration = state.exploration.as_mut().unwrap();
    exploration.presentation = None;
    exploration.script = None;
    state.movement_continuation = None;
    state.medical_continuation = None;
    state.combat_story = None;
    state.handled_deaths.clear();
    let snapshot = &case["before"];
    let member = snapshot["active_member"].as_u64().unwrap() as usize;
    project_party(&mut state, fixture, snapshot);
    let reserves = bytes(fixture, &snapshot["reserves"]);
    assert_eq!(reserves.len(), 804);
    state.ammunition = reserves[..800]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|long| i32::from_be_bytes(*long))
        .collect();
    state.loot_quality = Some(bytes(fixture, &snapshot["loot_quality"]));
    assert_eq!(&reserves[800..], &state.loot_quality.as_ref().unwrap()[..4]);
    // Captured neutral input buffers establish scalar projections only.
    // They are not authored definition0; raw records remain unknown.
    for key in ["item_work", "weapon_work", "ammo_work"] {
        assert_eq!(bytes(fixture, &snapshot[key]), vec![0; 150]);
    }
    state.ammunition_work = WorkBuffers::default();
    state.weapon_work = WeaponWork {
        sound: Some(0),
        weapon_armor_filter: Some(0),
        ammunition_armor_filter: Some(0),
    };
    state.loaded_item_field32 =
        Some(fixture["foundation"]["item_context"].as_i64().unwrap() as i16);
    state.random = random(snapshot);
    state.selected_warrior = member;
    state.flags = bytes(fixture, &snapshot["flags"]);
    state.party_buffs = bytes(fixture, &snapshot["party_buffs"]);
    state.psi_memory = Some(bytes(fixture, &snapshot["memory"]));
    state.crate_items = bytes(fixture, &snapshot["crate"])
        .as_chunks::<22>()
        .0
        .iter()
        .map(|record| InventoryRecord(*record))
        .collect();
    state.booty_items = bytes(fixture, &snapshot["loot"])
        .as_chunks::<22>()
        .0
        .iter()
        .map(|record| InventoryRecord(*record))
        .collect();
    state.attack_packet = Some(bytes(fixture, &snapshot["packet"]));
    state.attack_identity = Some(words(fixture, &snapshot["identity"]).try_into().unwrap());
    state.attack_count = number(snapshot, "count");
    state
        .party_mask
        .copy_from_slice(&words(fixture, &snapshot["mask"])[..6]);
    state.health_context = Some(HealthContext {
        unavailable: 0,
        selected_warrior: member,
        effect: number(snapshot, "health_effect") as i8,
        last_body: number(snapshot, "health_last_body"),
        marks_affected: number(snapshot, "heal_marks_affected") != 0,
        affected: words(fixture, &snapshot["markers"])[..6]
            .try_into()
            .unwrap(),
    });
    let combat = state.combat.as_mut().unwrap();
    combat.map.tiles = words(fixture, &snapshot["map"]);
    combat.map.shift = (
        number(snapshot, "shift_x") - 5,
        number(snapshot, "shift_y") - 5,
    );
    combat.squad.positions = words(fixture, &snapshot["party_positions"])
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| (pair[0], pair[1]))
        .collect();
    let npcs = bytes(fixture, &snapshot["npcs"]);
    assert_eq!(npcs.len(), 100 * 288);
    assert!(npcs[288..].iter().all(|byte| *byte == 0));
    let positions = words(fixture, &snapshot["npc_positions"]);
    assert!(positions[2..].iter().all(|word| *word == 0));
    combat.enemies.records = vec![MonsterRecord {
        bytes: npcs[..288].to_vec(),
    }];
    combat.enemies.positions = vec![(positions[0], positions[1])];
    combat.squad.under_tiles = combat
        .squad
        .positions
        .iter()
        .map(|(x, y)| {
            let index = (*x as usize + 5 + combat.map.shift.0 as usize) * 90
                + *y as usize
                + 5
                + combat.map.shift.1 as usize;
            combat.map.tiles[index]
        })
        .collect();
    combat.order = words(fixture, &snapshot["order"]);
    combat.cursor = snapshot["cursor"].as_u64().unwrap() as usize;
    combat.round = number(snapshot, "round_script_counter") - 1;
    combat.round_scripts = RoundScripts::default();
    combat.death_queue = DeathQueue::default();
    combat.spell_fields = bytes(fixture, &snapshot["fields"]);
    combat.field_used = bytes(fixture, &snapshot["field_used"]);
    combat.attack_packet = state.attack_packet.clone();
    combat.attack_identity = state.attack_identity;
    combat.weapon_work = state.weapon_work;
    combat.unavailable = 0;
    state
}

fn normalized(content: &Content, id: usize) -> Vec<u8> {
    let mut record = content.items[id].clone();
    let class = word(&record, 26).unwrap().wrapping_abs();
    record[26..28].copy_from_slice(&class.to_be_bytes());
    record
}

fn full_party(state: &Session) -> Vec<u8> {
    state
        .warriors
        .iter()
        .flat_map(|warrior| warrior.bytes.clone())
        .collect()
}

fn native_loaders(fixture: &Value, case: &Value, content: &Content) {
    let mut checked = 0;
    for phase in case["native_phases"].as_array().unwrap() {
        let key = match phase["address"].as_str().unwrap() {
            "130d38" => "weapon_work",
            "130e54" => "ammo_work",
            "130afe" => "item_work",
            _ => continue,
        };
        let id = phase["arguments"][0].as_u64().unwrap() as usize;
        assert_eq!(
            bytes(fixture, &phase["after"][key]),
            normalized(content, id)
        );
        for retained in ["item_work", "weapon_work", "ammo_work"] {
            if retained != key {
                assert_eq!(
                    bytes(fixture, &phase["before"][retained]),
                    bytes(fixture, &phase["after"][retained])
                );
            }
        }
        checked += 1;
    }
    assert_eq!(checked, 5);
}

fn compose_reaction(state: &Session, fixture: &Value, case: &Value, content: &Content) -> Session {
    let mut expected = state.clone();
    let mode = number(case, "mode");
    assert_eq!(expected.combat.as_ref().unwrap().active_body(), Some(0));
    assert_eq!(expected.warriors[0].bytes[0x9dd], 1);
    assert_eq!(word(&expected.warriors[0].bytes, 0xe4).unwrap(), 1);
    assert_eq!(word(&expected.warriors[0].bytes, 0xca).unwrap(), 0);
    let record = &mut expected.warriors[0].bytes;
    let dodged = u32::from_be_bytes(record[0xa56..0xa5a].try_into().unwrap());
    record[0xa56..0xa5a].copy_from_slice(&dodged.wrapping_add(1).to_be_bytes());
    assert_eq!(
        full_party(&expected),
        bytes(fixture, &case["after"]["party"])
    );
    let combat = expected.combat.as_mut().unwrap();
    assert_eq!(word(&combat.enemies.records[0].bytes, 0x4e).unwrap(), 0);
    assert_eq!(
        combat.enemies.records[0].bytes[0x104],
        if mode == 0 { 1 } else { 0 }
    );
    combat.enemies.records[0].bytes[0x104] = 0;
    let mut npcs = vec![0; 100 * 288];
    npcs[..288].copy_from_slice(&combat.enemies.records[0].bytes);
    assert_eq!(npcs, bytes(fixture, &case["after"]["npcs"]));
    // Natural helper136398 establishes all three definitions before the miss;
    // active-party refresh1115a8 subsequently replaces only ITEM with ID1.
    let natural = normalized(content, 0);
    assert_eq!(word(&natural, 100).unwrap(), 0);
    let item = normalized(content, 1);
    assert_eq!(natural, bytes(fixture, &case["after"]["weapon_work"]));
    assert_eq!(natural, bytes(fixture, &case["after"]["ammo_work"]));
    assert_eq!(item, bytes(fixture, &case["after"]["item_work"]));
    expected.ammunition_work = WorkBuffers {
        item: Some(item.clone()),
        weapon: Some(natural.clone()),
        ammo: Some(natural.clone()),
    };
    expected.loaded_item_field32 = Some(word(&item, 32).unwrap());
    expected.weapon_work = WeaponWork {
        sound: Some(word(&natural, 14).unwrap()),
        weapon_armor_filter: Some(word(&natural, 136).unwrap()),
        ammunition_armor_filter: Some(word(&natural, 136).unwrap()),
    };
    combat.weapon_work = expected.weapon_work;
    let attacks = &content
        .map(state.exploration.as_ref().unwrap().map)
        .unwrap()
        .combat
        .as_ref()
        .unwrap()
        .attacks;
    let packet = attacks[0x21c2..0x21c2 + 58].to_vec();
    assert_eq!(packet, bytes(fixture, &case["after"]["packet"]));
    combat.attack_packet = Some(packet.clone());
    expected.attack_packet = Some(packet.clone());
    expected.health_context.as_mut().unwrap().effect = packet[18] as i8;
    assert_eq!(
        expected.health_context.as_ref().unwrap().effect,
        number(&case["after"], "health_effect") as i8
    );
    assert_eq!(case["rng"].as_array().unwrap().len(), 1);
    assert_eq!(case["rng"][0]["bound"], 100);
    assert_eq!(expected.random.bound(100), 48);
    assert_eq!(case["rng"][0]["result"], 48);
    assert_eq!(expected.random, random(&case["after"]));
    assert_eq!(case["sounds"], serde_json::json!([650]));
    native_loaders(fixture, case, content);
    expected
}

fn retained_native_buffers(fixture: &Value, case: &Value) {
    for key in [
        "reserves",
        "selected_record",
        "markers",
        "auto_mask",
        "map",
        "order",
        "party_positions",
        "npc_positions",
        "party_buffs",
        "fields",
        "field_used",
        "flags",
        "memory",
        "references",
        "crate",
        "loot",
        "loot_quality",
        "selected_warrior_work",
        "remove_warrior_work",
        "backup_warriors",
        "item_cache0",
        "item_cache1",
        "item_cache2",
        "item_cache3",
        "scenario_cache",
        "identity",
        "mask",
    ] {
        assert_eq!(
            bytes(fixture, &case["before"][key]),
            bytes(fixture, &case["after"][key]),
            "native retained {key}"
        );
    }
    for key in [
        "active_member",
        "displayed_body",
        "unavailable",
        "count",
        "cursor",
        "round_script_counter",
        "inactivity_counter",
        "killed_count",
        "override_word",
        "heal_marks_affected",
        "health_last_body",
        "shift_x",
        "shift_y",
    ] {
        assert_eq!(
            case["before"][key], case["after"][key],
            "native retained {key}"
        );
    }
    let mut targets = bytes(fixture, &case["before"]["targets"]);
    targets[44..102].copy_from_slice(&bytes(fixture, &case["after"]["packet"]));
    assert_eq!(targets, bytes(fixture, &case["after"]["targets"]));
}

fn cell(state: &Session, x: i16, y: i16) -> usize {
    let shift = state.combat.as_ref().unwrap().map.shift;
    (usize::try_from(x + shift.0 + 5).unwrap() * 90) + usize::try_from(y + shift.1 + 5).unwrap()
}

fn same(actual: &Session, expected: &Session) {
    let actual = serde_json::to_value(actual).unwrap();
    let expected = serde_json::to_value(expected).unwrap();
    for (key, value) in expected.as_object().unwrap() {
        assert!(actual[key] == *value, "Session.{key} differs");
    }
    assert_eq!(
        actual.as_object().unwrap().len(),
        expected.as_object().unwrap().len()
    );
}

fn checkpoint(state: &Session, content: &Content, label: &str) -> Session {
    let path = std::env::temp_dir().join(format!(
        "centurions-reaction-work-{}-{label}.json",
        std::process::id()
    ));
    save(&path, state, content).unwrap();
    let restored = load(&path, content).unwrap();
    same(&restored, state);
    std::fs::remove_file(path).unwrap();
    restored
}

#[test]
fn public_moves_match_native_reactions_and_source_composed_outer_effects() {
    let fixture = fixture();
    let content = content();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 2);
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        assert_eq!(number(case, "mode"), [0, 2][index]);
        let mut state = initial(&fixture, case);
        assert_eq!(state.combat.as_ref().unwrap().squad.positions[0], (20, 20));
        assert_eq!(
            state.combat.as_ref().unwrap().enemies.positions,
            vec![(21, 20)]
        );
        let old_cell = cell(&state, 20, 20);
        let next_cell = cell(&state, 19, 20);
        let mission = content
            .map(state.exploration.as_ref().unwrap().map)
            .unwrap();
        let tile = if index == 0 { 1061 } else { 1155 };
        assert_eq!(mission.tile_word(155, 0).unwrap(), 82);
        assert_eq!(mission.tile_word(155, 2).unwrap(), 2);
        assert_eq!(mission.tile_word(155, 4).unwrap(), 0);
        if index == 0 {
            assert_ne!(mission.tile_word(61, 4).unwrap(), 0);
            // Declared blocking destination does not change native adjacency,
            // the miss roll, or any helper's retained rule writer inputs.
            state.combat.as_mut().unwrap().map.tiles[next_cell] = tile;
        }
        state.validate(&content).unwrap();
        let mut expected = compose_reaction(&state, &fixture, case, &content);
        retained_native_buffers(&fixture, case);
        // The original helper is the native oracle boundary. Outer Move is
        // source-composed in a fixed logical frame; native camera recentering
        // and a complete original walking caller are not certified here.
        // Facing11c0cc and blocked sounds11c932/11ce50 follow Mode0.
        expected.warriors[0].bytes[0x9e0] = 1;
        expected.bandage_ready = Some(0);
        // AttackResolved uses the established synthesized neighbor convention;
        // native attack2 requests650 internally and captures no typed event.
        // The Sound subsequence alone has the native audio ordering.
        let mut events = vec![
            Event::AttackResolved {
                attacker: 10,
                target: 0,
                damage: 0,
                defeated: false,
            },
            Event::Sound { resource_id: 650 },
        ];
        if index == 0 {
            events.extend([
                Event::Sound { resource_id: -129 },
                Event::Sound { resource_id: -130 },
            ]);
        } else {
            // Getmovecost153de6 halves signed WORD2, subtracts1, clamps0,
            // then adds1 for nonzero dx.11c966 is the captured Mode2 helper;
            // 11c99a/11ca0e/11ca76/11cad4/11caf4 commit MV/map/position.
            let property = mission.tile_word(tile - 1000, 2).unwrap();
            let half = property.wrapping_add((property as u16 >> 15) as i16) >> 1;
            let cost = half.wrapping_sub(1).max(0).wrapping_add(1);
            assert_eq!(cost, 1);
            let remaining = word(&expected.warriors[0].bytes, 0x3c)
                .unwrap()
                .wrapping_sub(cost);
            expected.warriors[0].bytes[0x3c..0x3e].copy_from_slice(&remaining.to_be_bytes());
            let combat = expected.combat.as_mut().unwrap();
            combat.map.tiles[old_cell] = combat.squad.under_tiles[0];
            combat.squad.under_tiles[0] = tile;
            combat.map.tiles[next_cell] = 0;
            combat.squad.positions[0] = (19, 20);
            events.extend([
                Event::TacticalMoved {
                    body: 0,
                    x: 19,
                    y: 20,
                },
                Event::Sound {
                    resource_id: mission.tile_word(tile - 1000, 0).unwrap(),
                },
            ]);
        }
        expected.validate(&content).unwrap();
        let mut uninterrupted = state.clone();
        let mut resumed = checkpoint(&state, &content, &format!("{index}-before"));
        let command = Command::Move { dx: -1, dy: 0 };
        assert_eq!(
            apply(command.clone(), &mut uninterrupted, &content).unwrap(),
            events
        );
        same(&uninterrupted, &expected);
        assert_eq!(apply(command, &mut resumed, &content).unwrap(), events);
        same(&resumed, &expected);
        same(
            &checkpoint(&uninterrupted, &content, &format!("{index}-after")),
            &expected,
        );
    }
}

fn atomic(state: &Session, content: &Content, command: Command) {
    state.validate(content).unwrap();
    let mut candidate = state.clone();
    assert!(
        apply(command.clone(), &mut candidate, content).is_err(),
        "admitted {command:?}"
    );
    same(&candidate, state);
}

#[test]
fn unsupported_moves_or_missing_reaction_resources_preserve_state_and_rng() {
    let fixture = fixture();
    let content = content();
    let state = initial(&fixture, &fixture["cases"][0]);
    atomic(&state, &content, Command::Move { dx: 0, dy: 0 });
    atomic(&state, &content, Command::Move { dx: -2, dy: 0 });
    let mut enemy_turn = state.clone();
    enemy_turn.combat.as_mut().unwrap().cursor = 2;
    atomic(&enemy_turn, &content, Command::Move { dx: -1, dy: 0 });
    let mut marker = state.clone();
    let next_cell = cell(&marker, 19, 20);
    marker.combat.as_mut().unwrap().map.tiles[next_cell] = 111;
    atomic(&marker, &content, Command::Move { dx: -1, dy: 0 });
    for missing in [0, 1] {
        let mut content = content.clone();
        content.items[missing].clear();
        atomic(&state, &content, Command::Move { dx: -1, dy: 0 });
    }
}
