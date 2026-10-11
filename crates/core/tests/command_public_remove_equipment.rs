use centurions_content::{Content, InventoryRecord, Warrior, digest, word};
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
    let text = include_str!("../../../evidence/fixtures/public-remove-equipment.json");
    assert_eq!(
        digest(text.as_bytes()),
        "474811d8ce0acb3a7c4ee5883e96ef70439b03cac1e3a35a97f38956f2cf38e0"
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
    let member = case["member"].as_u64().unwrap() as usize;
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
    state.ammunition_work = WorkBuffers {
        item: Some(bytes(fixture, &snapshot["item_work"])),
        weapon: Some(bytes(fixture, &snapshot["weapon_work"])),
        ammo: Some(bytes(fixture, &snapshot["ammo_work"])),
    };
    let weapon = state.ammunition_work.weapon.as_ref().unwrap();
    let ammo = state.ammunition_work.ammo.as_ref().unwrap();
    state.weapon_work = WeaponWork {
        sound: Some(word(weapon, 14).unwrap()),
        weapon_armor_filter: Some(word(weapon, 136).unwrap()),
        ammunition_armor_filter: Some(word(ammo, 136).unwrap()),
    };
    state.loaded_item_field32 =
        Some(word(state.ammunition_work.item.as_ref().unwrap(), 32).unwrap());
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
    // No active NPC is established by these zeroed native helper inputs.
    assert!(
        bytes(fixture, &snapshot["npcs"])
            .iter()
            .all(|byte| *byte == 0)
    );
    assert!(
        bytes(fixture, &snapshot["npc_positions"])
            .iter()
            .all(|byte| *byte == 0)
    );
    combat.enemies.records.clear();
    combat.enemies.positions.clear();
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

fn expected(state: &Session, fixture: &Value, case: &Value, content: &Content) -> Session {
    let mut expected = state.clone();
    let after = &case["after"];
    project_party(&mut expected, fixture, after);
    let id = case["id"].as_u64().unwrap() as usize;
    let mut definition = content.items[id].clone();
    let class = word(&definition, 26).unwrap().wrapping_abs();
    definition[26..28].copy_from_slice(&class.to_be_bytes());
    assert_eq!(definition, bytes(fixture, &after["item_work"]));
    assert_eq!(word(&definition, 46).unwrap(), 0);
    assert_eq!(word(&definition, 48).unwrap(), 0);
    expected.loaded_item_field32 = Some(word(&definition, 32).unwrap());
    expected.ammunition_work.item = Some(definition);
    let member = case["member"].as_u64().unwrap() as usize;
    expected.health_context.as_mut().unwrap().affected[member] = 1;
    assert_eq!(
        expected.health_context.as_ref().unwrap().affected,
        words(fixture, &after["markers"])[..6]
    );
    expected
}

fn retained_native_buffers(fixture: &Value, case: &Value) {
    for key in [
        "reserves",
        "item_work",
        "weapon_work",
        "ammo_work",
        "selected_record",
        "selected_warrior_work",
        "spell_cache",
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
        "auto_mask",
    ] {
        assert_eq!(
            bytes(fixture, &case["before"][key]),
            bytes(fixture, &case["after"][key]),
            "native retained {key}"
        );
    }
    for key in [
        "selection_control",
        "mode",
        "involuntary_movement",
        "changed_ammo",
        "spell_alias_word",
        "count",
        "cursor",
        "round_script_counter",
        "inactivity_counter",
        "killed_count",
        "override_word",
        "health_effect",
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
    assert!(case["rng"].as_array().unwrap().is_empty());
    assert_eq!(random(&case["before"]), random(&case["after"]));
}

fn compaction_is_unchanged(state: &Session, member: usize) {
    let warrior = &state.warriors[member];
    let count = warrior.inventory().unwrap().len();
    let records = warrior.bytes[0x132..0x132 + 2200].as_chunks::<22>().0;
    assert!(records[..count].iter().all(|record| record[9] == 0));
    assert!(
        records[count..]
            .iter()
            .all(|record| record[..4] == [0; 4] && record[9] == 0)
    );
    // At original compaction12006e no equipped row can move, and the unused
    // record fields it clears are already zero in these four native results.
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
        "centurions-remove-{}-{label}.json",
        std::process::id()
    ));
    save(&path, state, content).unwrap();
    let resumed = load(&path, content).unwrap();
    same(&resumed, state);
    std::fs::remove_file(path).unwrap();
    resumed
}

#[test]
fn four_public_removals_at_zero_ap_match_complete_native_state_and_saved_replay() {
    let fixture = fixture();
    let content = content();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 5);
    for (index, case) in fixture["cases"].as_array().unwrap()[..4].iter().enumerate() {
        assert_eq!(case["id"], [1, 2, 38, 165][index]);
        assert_eq!(case["variant"], "active_ap_zero");
        assert_eq!(case["native_helper_return_word"], 1);
        assert!(case["warnings"].as_array().unwrap().is_empty());
        let state = initial(&fixture, case);
        let member = case["member"].as_u64().unwrap() as usize;
        let expected = expected(&state, &fixture, case, &content);
        state.validate(&content).unwrap();
        expected.validate(&content).unwrap();
        assert_eq!(word(&state.warriors[member].bytes, 0x40).unwrap(), 0);
        assert_eq!(word(&expected.warriors[member].bytes, 0x40).unwrap(), 0);
        assert_eq!(
            word(&state.warriors[member].bytes, 0x42).unwrap(),
            word(&expected.warriors[member].bytes, 0x42).unwrap()
        );
        assert_eq!(expected.combat, state.combat);
        compaction_is_unchanged(&expected, member);
        retained_native_buffers(&fixture, case);
        let events: Vec<_> = case["sounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|sound| Event::Sound {
                resource_id: sound.as_i64().unwrap() as i16,
            })
            .collect();
        let definition = &content.items[case["id"].as_u64().unwrap() as usize];
        let sound = word(definition, 12).unwrap();
        let sound = if sound == 0 {
            word(definition, 14).unwrap()
        } else {
            sound
        };
        assert_eq!(events, vec![Event::Sound { resource_id: sound }]);
        let command = Command::UnequipItem {
            warrior: member,
            slot: case["slot"].as_u64().unwrap() as usize,
        };
        let mut uninterrupted = state.clone();
        let mut resumed = checkpoint(&state, &content, &format!("{index}-before"));
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
        if index == 0 {
            assert_eq!(state.health_context.as_ref().unwrap().affected[0], 0);
            assert_eq!(expected.health_context.as_ref().unwrap().affected[0], 1);
        }
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
fn inactive_and_unverified_removals_preserve_the_entire_session() {
    let fixture = fixture();
    let content = content();
    let inactive = &fixture["cases"][4];
    assert_eq!(inactive["native_helper_return_word"], 0);
    assert_eq!(inactive["warnings"], serde_json::json!([10]));
    assert_eq!(inactive["sounds"], serde_json::json!([6000]));
    retained_native_buffers(&fixture, inactive);
    assert_eq!(
        bytes(&fixture, &inactive["before"]["party"]),
        bytes(&fixture, &inactive["after"]["party"])
    );
    assert_eq!(inactive["before"]["markers"], inactive["after"]["markers"]);
    let state = initial(&fixture, inactive);
    assert_eq!(state.combat.as_ref().unwrap().active_body(), Some(1));
    // The public diagnostic discards the native warning/audio candidate.
    atomic(
        &state,
        &content,
        Command::UnequipItem {
            warrior: 0,
            slot: 0,
        },
    );
    let resumed = checkpoint(&state, &content, "inactive-before");
    atomic(
        &resumed,
        &content,
        Command::UnequipItem {
            warrior: 0,
            slot: 0,
        },
    );
    let active = initial(&fixture, &fixture["cases"][2]);
    atomic(
        &active,
        &content,
        Command::UnequipItem {
            warrior: 1,
            slot: 0,
        },
    );
    atomic(
        &active,
        &content,
        Command::UnequipItem {
            warrior: 0,
            slot: 100,
        },
    );
    let mut unverified = active.clone();
    unverified.warriors[0].bytes[0x132 + 14] = 1;
    atomic(
        &unverified,
        &content,
        Command::UnequipItem {
            warrior: 0,
            slot: 0,
        },
    );
    let mut missing = content;
    missing.items[38].clear();
    atomic(
        &active,
        &missing,
        Command::UnequipItem {
            warrior: 0,
            slot: 0,
        },
    );
}
