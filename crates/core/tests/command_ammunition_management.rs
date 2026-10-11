use centurions_content::{Content, InventoryRecord, Warrior, combat::MonsterRecord, digest, word};
use centurions_core::{
    Command, Event, Phase, RandomState, Session,
    ammunition::WorkBuffers,
    apply,
    combat::{death_scripts::DeathQueue, round_stories::RoundScripts, weapon_work::WeaponWork},
    load, save,
    story_health::HealthContext,
};
use serde_json::Value;

fn fixture() -> Value {
    let text = include_str!("../../../evidence/fixtures/ammunition-management.json");
    assert_eq!(
        digest(text.as_bytes()),
        "4f2b282b7e6735dd3428d39bdaaabb160f10e02442ad81052032a604742bf9bb"
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
    let result: Vec<_> = (0..text.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).unwrap())
        .collect();
    if value.is_object() {
        assert_eq!(
            result.len(),
            value["byte_length"].as_u64().unwrap() as usize
        );
        assert_eq!(
            digest(&result),
            value["shared_buffer_sha256"].as_str().unwrap()
        );
    }
    result
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

fn projected_work(fixture: &Value, snapshot: &Value) -> WeaponWork {
    let weapon = bytes(fixture, &snapshot["weapon_work"]);
    let ammo = bytes(fixture, &snapshot["ammo_work"]);
    WeaponWork {
        sound: Some(word(&weapon, 14).unwrap()),
        weapon_armor_filter: Some(word(&weapon, 136).unwrap()),
        ammunition_armor_filter: Some(word(&ammo, 136).unwrap()),
    }
}

fn project_operation(state: &mut Session, fixture: &Value, snapshot: &Value) {
    for (warrior, record) in state
        .warriors
        .iter_mut()
        .zip(bytes(fixture, &snapshot["party"]).as_chunks::<3138>().0)
    {
        let native = Warrior::read(record).unwrap();
        assert_eq!(warrior.name, native.name);
        warrior.bytes = native.bytes;
    }
    let reserves = bytes(fixture, &snapshot["reserves"]);
    assert_eq!(reserves.len(), 804);
    state.ammunition = reserves[..800]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|long| i32::from_be_bytes(*long))
        .collect();
    state.ammunition_work = WorkBuffers {
        item: Some(bytes(fixture, &snapshot["item_work"])),
        ammo: Some(bytes(fixture, &snapshot["ammo_work"])),
        weapon: Some(bytes(fixture, &snapshot["weapon_work"])),
    };
    state.loot_quality = Some(bytes(fixture, &snapshot["loot_quality"]));
    assert_eq!(
        &reserves[800..804],
        &state.loot_quality.as_ref().unwrap()[..4]
    );
    state.loaded_item_field32 =
        Some(word(state.ammunition_work.item.as_ref().unwrap(), 32).unwrap());
    state.weapon_work = projected_work(fixture, snapshot);
    state.random = random(snapshot);
    if let Some(combat) = state.combat.as_mut() {
        combat.weapon_work = state.weapon_work;
    }
}

fn project_captured_globals(state: &mut Session, fixture: &Value, snapshot: &Value, member: usize) {
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
    if let Some(combat) = state.combat.as_mut() {
        combat.attack_packet = state.attack_packet.clone();
        combat.attack_identity = state.attack_identity;
        combat.unavailable = 0;
    }
}

fn background() -> Session {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/player-psi.json")).unwrap();
    let mut state: Session = serde_json::from_value(fixture["route_foundation"].clone()).unwrap();
    // This valid mission is scaffolding for controlled operation inputs. Its
    // unrecorded background stays in the expected clone; this is no earned route.
    let exploration = state.exploration.as_mut().unwrap();
    exploration.presentation = None;
    exploration.script = None;
    state.movement_continuation = None;
    state.medical_continuation = None;
    state.combat_story = None;
    state.handled_deaths.clear();
    state
}

fn initial_operation(fixture: &Value, case: &Value) -> Session {
    let mut state = background();
    let member = case["member"].as_u64().unwrap() as usize;
    if number(case, "combat") == 0 {
        state.phase = Phase::Exploration;
        state.combat = None;
    } else {
        // Native helper receipts do not execute the combat caller. Retain the
        // valid hostile background so its finish check has ordinary semantics.
        let combat = state.combat.as_mut().unwrap();
        combat.order[combat.cursor] = member as i16;
        combat.death_queue = DeathQueue::default();
    }
    project_operation(&mut state, fixture, &case["before"]);
    project_captured_globals(&mut state, fixture, &case["before"], member);
    state
}

fn pairs(fixture: &Value, value: &Value) -> Vec<(i16, i16)> {
    words(fixture, value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| (pair[0], pair[1]))
        .collect()
}

fn project_move(state: &mut Session, fixture: &Value, snapshot: &Value) {
    project_operation(state, fixture, snapshot);
    project_captured_globals(state, fixture, snapshot, 4);
    // The frozen native centerfield setup establishes count1. The remaining
    // ninety-nine captured slots are zero retained memory, not active NPCs.
    let npcs = bytes(fixture, &snapshot["npcs"]);
    assert_eq!(npcs.len(), 100 * 288);
    assert!(npcs[288..].iter().all(|byte| *byte == 0));
    let combat = state.combat.as_mut().unwrap();
    combat.map.tiles = words(fixture, &snapshot["map"]);
    // Native camera LONGs already include the five-cell origin. TacticalMap
    // stores the displacement and adds five when indexing the captured map.
    combat.map.shift = (
        number(snapshot, "shift_x") - 5,
        number(snapshot, "shift_y") - 5,
    );
    combat.map.edge_word = 0;
    combat.squad.positions = pairs(fixture, &snapshot["party_positions"]);
    combat.squad.under_tiles = vec![1155; 6];
    combat.enemies.records = vec![MonsterRecord {
        bytes: npcs[..288].to_vec(),
    }];
    combat.enemies.positions = pairs(fixture, &snapshot["npc_positions"])[..1].to_vec();
    combat.order = words(fixture, &snapshot["order"]);
    combat.cursor = snapshot["cursor"].as_u64().unwrap() as usize;
    combat.round = number(snapshot, "round_script_counter") - 1;
    combat.round_scripts = RoundScripts::default();
    combat.death_queue = DeathQueue::default();
    combat.spell_fields = bytes(fixture, &snapshot["fields"]);
    combat.field_used = bytes(fixture, &snapshot["field_used"]);
    state.party_mask = state.health_context.as_ref().unwrap().affected;
}

fn difference(actual: &Value, expected: &Value, path: &str) -> Option<String> {
    if actual == expected {
        return None;
    }
    match (actual, expected) {
        (Value::Object(a), Value::Object(b)) if a.len() == b.len() => {
            for (key, value) in a {
                if let Some(other) = b.get(key) {
                    if let Some(error) = difference(value, other, &format!("{path}.{key}")) {
                        return Some(error);
                    }
                } else {
                    return Some(format!("{path}.{key}: unexpected field"));
                }
            }
        }
        (Value::Array(a), Value::Array(b)) if a.len() == b.len() => {
            for (index, (value, other)) in a.iter().zip(b).enumerate() {
                if let Some(error) = difference(value, other, &format!("{path}[{index}]")) {
                    return Some(error);
                }
            }
        }
        _ => return Some(format!("{path}: {actual} != {expected}")),
    }
    Some(format!("{path}: fields differ"))
}

fn same(actual: &Session, expected: &Session) {
    if let Some(error) = difference(
        &serde_json::to_value(actual).unwrap(),
        &serde_json::to_value(expected).unwrap(),
        "session",
    ) {
        panic!("{error}");
    }
}

fn checkpoint(state: &Session, content: &Content, label: &str) -> Session {
    let path = std::env::temp_dir().join(format!(
        "centurions-ammunition-command-{}-{label}.json",
        std::process::id()
    ));
    save(&path, state, content).unwrap();
    let restored = load(&path, content).unwrap();
    same(&restored, state);
    std::fs::remove_file(path).unwrap();
    restored
}

fn expected_events(case: &Value, caller_sound: bool) -> Vec<Event> {
    let mut events = vec![];
    if caller_sound {
        events.push(Event::Sound { resource_id: 141 });
    }
    events.extend(
        case["sounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|sound| Event::Sound {
                resource_id: sound.as_i64().unwrap() as i16,
            }),
    );
    events.extend(case["warnings"].as_array().unwrap().iter().map(|warning| {
        Event::CollectionRejected {
            warning: warning.as_i64().unwrap() as i16,
        }
    }));
    events
}

fn exercise(
    state: Session,
    expected: Session,
    content: &Content,
    command: Command,
    events: Vec<Event>,
    label: &str,
) {
    state.validate(content).unwrap();
    expected.validate(content).unwrap();
    let mut uninterrupted = state.clone();
    let mut resumed = checkpoint(&state, content, &format!("{label}-before"));
    assert_eq!(
        apply(command.clone(), &mut uninterrupted, content).unwrap(),
        events,
        "{label} events"
    );
    same(&uninterrupted, &expected);
    assert_eq!(
        apply(command, &mut resumed, content).unwrap(),
        events,
        "{label} resumed events"
    );
    same(&resumed, &expected);
    same(
        &checkpoint(&uninterrupted, content, &format!("{label}-after")),
        &expected,
    );
}

fn check_rng(case: &Value) {
    let mut rng = random(&case["before"]);
    for draw in case["rng"].as_array().unwrap() {
        assert_eq!(rng.bound(number(draw, "bound")), number(draw, "result"));
    }
    assert_eq!(rng, random(&case["after"]));
}

fn retained_native_buffers(fixture: &Value, case: &Value) {
    // These original whole buffers are outside the operation's writes. Keep
    // their independent native equality alongside the full Session comparison.
    for key in [
        "npcs",
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
        "markers",
        "auto_mask",
        "backup_warriors",
        "item_cache0",
        "item_cache1",
        "item_cache2",
        "item_cache3",
        "scenario_cache",
        "packet",
        "identity",
        "mask",
        "targets",
    ] {
        assert_eq!(
            bytes(fixture, &case["before"][key]),
            bytes(fixture, &case["after"][key]),
            "retained native {key}"
        );
    }
}

#[test]
fn outdoor_reload_matches_native_records_caches_ordered_effects_and_saved_replay() {
    let fixture = fixture();
    let content = content();
    let mut checked = 0;
    for (index, case) in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(index, case)| case["kind"] == "manual_refit" && *index != 38)
    {
        let state = initial_operation(&fixture, case);
        let mut expected = state.clone();
        project_operation(&mut expected, &fixture, &case["after"]);
        check_rng(case);
        retained_native_buffers(&fixture, case);
        exercise(
            state,
            expected,
            &content,
            Command::Reload { warrior: 0 },
            expected_events(case, false),
            &format!("reload-{index}"),
        );
        checked += 1;
    }
    assert_eq!(checked, 14);
}

#[test]
fn accepted_fit_matches_native_helper_and_combat_caller_in_complete_saved_sessions() {
    let fixture = fixture();
    let content = content();
    let mut checked = 0;
    for (index, case) in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, case)| case["kind"] == "fit" && case["variant"] != "cancel_no_candidate")
    {
        let state = initial_operation(&fixture, case);
        let mut expected = state.clone();
        project_operation(&mut expected, &fixture, &case["after"]);
        let in_combat = number(case, "combat") != 0;
        if in_combat {
            assert_eq!(word(&expected.warriors[0].bytes, 0x40).unwrap(), 7);
            assert_eq!(expected.combat.as_ref().unwrap().active_body(), Some(0));
        } else {
            assert_eq!(word(&expected.warriors[0].bytes, 0x40).unwrap(), 8);
        }
        check_rng(case);
        retained_native_buffers(&fixture, case);
        let ammunition = number(&case["popup_inputs"][0], "accepted_ammo");
        exercise(
            state,
            expected,
            &content,
            Command::FitAmmunition {
                warrior: 0,
                slot: case["slot"].as_u64().unwrap() as usize,
                ammunition,
            },
            expected_events(case, in_combat),
            &format!("fit-{index}"),
        );
        checked += 1;
    }
    assert_eq!(checked, 12);
}

#[test]
fn four_controlled_occupied_move_commands_match_the_complete_native_rule_overlay() {
    let fixture = fixture();
    let content = content();
    for index in 50..54 {
        let case = &fixture["cases"][index];
        assert_eq!(case["kind"], "move_reload");
        let mut state = background();
        state.exploration.as_mut().unwrap().map = 0;
        state.visited_maps.remove(&0);
        project_move(&mut state, &fixture, &case["before"]);
        assert_eq!(
            state.combat.as_ref().unwrap().enemies.records[0]
                .stamina()
                .unwrap(),
            0
        );
        assert_eq!(state.combat.as_ref().unwrap().squad.positions[4], (5, 5));
        assert_eq!(state.combat.as_ref().unwrap().enemies.positions[0], (6, 6));
        let mut expected = state.clone();
        project_move(&mut expected, &fixture, &case["after"]);
        check_rng(case);
        retained_native_buffers(&fixture, case);
        exercise(
            state,
            expected,
            &content,
            Command::Move { dx: 1, dy: 1 },
            expected_events(case, false),
            &format!("move-{index}"),
        );
    }
}

fn atomic(state: &Session, content: &Content, command: Command) {
    if state.ammunition.iter().all(|stock| *stock >= 0) {
        state.validate(content).unwrap();
    }
    let mut actual = state.clone();
    assert!(apply(command, &mut actual, content).is_err());
    same(&actual, state);
}

#[test]
fn unrepresented_or_invalid_ammunition_inputs_preserve_every_session_field_and_rng() {
    let fixture = fixture();
    let content = content();
    let negative = initial_operation(&fixture, &fixture["cases"][38]);
    assert_eq!(negative.ammunition[3], -1);
    atomic(&negative, &content, Command::Reload { warrior: 0 });
    let fit = initial_operation(&fixture, &fixture["cases"][13]);
    atomic(
        &fit,
        &content,
        Command::FitAmmunition {
            warrior: 0,
            slot: 0,
            ammunition: 200,
        },
    );
    atomic(
        &fit,
        &content,
        Command::FitAmmunition {
            warrior: 1,
            slot: 0,
            ammunition: 203,
        },
    );
    let mut missing = content.clone();
    missing.items[38].clear();
    atomic(
        &fit,
        &missing,
        Command::FitAmmunition {
            warrior: 0,
            slot: 0,
            ammunition: 203,
        },
    );
    let mut move_state = background();
    move_state.exploration.as_mut().unwrap().map = 0;
    move_state.visited_maps.remove(&0);
    project_move(&mut move_state, &fixture, &fixture["cases"][50]["before"]);
    let mut unknown = move_state.clone();
    unknown.ammunition_work.item = None;
    unknown.loaded_item_field32 = None;
    atomic(&unknown, &content, Command::Move { dx: 1, dy: 1 });
    let mut wrong_turn = move_state.clone();
    wrong_turn.combat.as_mut().unwrap().order[0] = 10;
    atomic(&wrong_turn, &content, Command::Move { dx: 1, dy: 1 });
    atomic(&move_state, &missing, Command::Move { dx: 1, dy: 1 });
}
