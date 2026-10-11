use centurions_content::{Content, InventoryRecord, combat::MonsterRecord, word};
use centurions_core::{Command, Event, RuleError, Session, apply, load, save};
use serde_json::Value;

fn fixture() -> (Content, Value) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let fixture = serde_json::from_str(
        &std::fs::read_to_string(root.join("evidence/fixtures/npc-poison-consumer.json")).unwrap(),
    )
    .unwrap();
    (content, fixture)
}

fn bytes(f: &Value, value: &Value) -> Vec<u8> {
    let hex = value.as_str().unwrap_or_else(|| {
        f["shared_rule_buffers"][value["shared_buffer_sha256"].as_str().unwrap()]
            .as_str()
            .unwrap()
    });
    let decoded: Vec<_> = hex
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect();
    if value.is_object() {
        assert_eq!(decoded.len() as u64, value["byte_length"].as_u64().unwrap());
    }
    decoded
}

fn words(f: &Value, value: &Value) -> Vec<i16> {
    bytes(f, value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| i16::from_be_bytes(*pair))
        .collect()
}

fn positions(f: &Value, snapshot: &Value, key: &str) -> Vec<(i16, i16)> {
    let origin = &f["earned_native_caller"]["before"];
    let dx =
        snapshot["shift_x"].as_i64().unwrap() as i16 - origin["shift_x"].as_i64().unwrap() as i16;
    let dy =
        snapshot["shift_y"].as_i64().unwrap() as i16 - origin["shift_y"].as_i64().unwrap() as i16;
    words(f, &snapshot[key])
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| (pair[0].wrapping_add(dx), pair[1].wrapping_add(dy)))
        .collect()
}

fn project(state: &mut Session, f: &Value, snapshot: &Value) {
    let party = bytes(f, &snapshot["party"]);
    assert_eq!(party.len(), 18_828);
    for (warrior, record) in state.warriors.iter_mut().zip(party.as_chunks::<3138>().0) {
        warrior.bytes = record.to_vec();
    }
    let count = snapshot["body_count"].as_u64().unwrap() as usize;
    let npcs = bytes(f, &snapshot["npcs"]);
    assert_eq!(npcs.len(), 28_800);
    assert!(npcs[count * 288..].iter().all(|byte| *byte == 0));
    let npc_positions = positions(f, snapshot, "npc_positions");
    assert_eq!(npc_positions.len(), 100);
    assert!(npc_positions[count..].iter().all(|point| *point == (0, 0)));
    let combat = state.combat.as_mut().unwrap();
    combat.enemies.records = npcs[..count * 288]
        .as_chunks::<288>()
        .0
        .iter()
        .map(|record| MonsterRecord {
            bytes: record.to_vec(),
        })
        .collect();
    combat.enemies.positions = npc_positions[..count].to_vec();
    combat.squad.positions = positions(f, snapshot, "party_positions");
    combat.map.tiles = words(f, &snapshot["map"]);
    combat.order = words(f, &snapshot["order"]);
    combat.cursor = snapshot["cursor"].as_u64().unwrap() as usize;
    combat.round_scripts.counter = snapshot["round_script_counter"].as_i64().unwrap() as i8;
    combat.spell_fields = bytes(f, &snapshot["fields"]);
    combat.field_used = bytes(f, &snapshot["field_used"]);
    combat.attack_packet = Some(bytes(f, &snapshot["packet"]));
    combat.attack_identity = Some(words(f, &snapshot["identity"]).try_into().unwrap());
    state.attack_packet = combat.attack_packet.clone();
    state.attack_identity = combat.attack_identity;
    state.attack_count = snapshot["count"].as_i64().unwrap() as i16;
    state
        .party_mask
        .copy_from_slice(&words(f, &snapshot["mask"])[..6]);
    state.selected_warrior = snapshot["active_member"].as_u64().unwrap() as usize;
    state.bandage_ready = Some(snapshot["aid_gate"].as_i64().unwrap() as i16);
    state.enemy_spell_disabled = snapshot["enemy_spell_disabled"].as_i64().unwrap() as u8;
    state.random.seed = snapshot["seed"].as_u64().unwrap() as u32;
    state.random.draws = snapshot["draws"].as_u64().unwrap();
    state.ammunition_work.item = Some(bytes(f, &snapshot["item_work"]));
    state.ammunition_work.weapon = Some(bytes(f, &snapshot["weapon_work"]));
    state.ammunition_work.ammo = Some(bytes(f, &snapshot["ammo_work"]));
    state.loaded_item_field32 =
        Some(word(state.ammunition_work.item.as_ref().unwrap(), 32).unwrap());
    state.flags = bytes(f, &snapshot["flags"]);
    state.allies = bytes(f, &snapshot["references"]);
    state.psi_memory = Some(bytes(f, &snapshot["memory"]));
    state.party_buffs = bytes(f, &snapshot["party_buffs"]);
    state.crate_items = bytes(f, &snapshot["crate"])
        .as_chunks::<22>()
        .0
        .iter()
        .map(|record| InventoryRecord(*record))
        .collect();
    state.booty_items = bytes(f, &snapshot["loot"])
        .as_chunks::<22>()
        .0
        .iter()
        .map(|record| InventoryRecord(*record))
        .collect();
    let reserves = bytes(f, &snapshot["reserves"]);
    assert_eq!(reserves.len(), 804);
    state.ammunition = reserves[..800]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|value| i32::from_be_bytes(*value))
        .collect();
    let health = state.health_context.as_mut().unwrap();
    health.selected_warrior = state.selected_warrior;
    health.effect = snapshot["health_effect"].as_i64().unwrap() as i8;
    health.last_body = snapshot["health_last_body"].as_i64().unwrap() as i16;
    health.marks_affected = snapshot["heal_marks_affected"].as_i64().unwrap() != 0;
    let exploration = state.exploration.as_mut().unwrap();
    exploration.event_headers = bytes(f, &snapshot["event_headers"]);
    exploration.region_state = bytes(f, &snapshot["active_region"]);
}

fn initial(f: &Value, case: &Value, content: &Content) -> Session {
    assert_eq!(f["foundation"]["controlled_music_disabled_word"], 1);
    let mut state: Session = serde_json::from_value(f["foundation"]["session"].clone()).unwrap();
    project(&mut state, f, &f["earned_native_caller"]["after"]);
    project(&mut state, f, &case["before"]);
    state.validate(content).unwrap();
    assert_eq!(state.combat.as_ref().unwrap().active_body(), Some(3));
    assert_eq!(state.selected_warrior, 3);
    assert_eq!(state.combat.as_ref().unwrap().map.shift, (0, -36));
    state
}

// The oracle covers only NPC processing at 125f44..126330. Cursor109 is an
// explicit caller control; prior party reduction and the remaining Getup
// continuation are exercised as command integration, not native whole-Getup proof.
fn controlled_round(f: &Value, content: &Content) -> Session {
    let mut state = initial(f, &f["cases"][0], content);
    state.combat.as_mut().unwrap().cursor = 109;
    state.validate(content).unwrap();
    state
}

#[test]
fn earned_poison_with_real_fields_survives_public_round_save_resume_and_repeat() {
    let (content, f) = fixture();
    let before = controlled_round(&f, &content);
    let path = std::env::temp_dir().join(format!(
        "centurions-npc-poison-consumer-command-{}.json",
        std::process::id()
    ));
    save(&path, &before, &content).unwrap();
    let document: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(document["schema_version"], 3);
    let mut resumed = load(&path, &content).unwrap();
    assert_eq!(resumed, before);
    let mut actual = before.clone();
    let events = apply(Command::EndTurn, &mut actual, &content).unwrap();
    assert_eq!(
        apply(Command::EndTurn, &mut resumed, &content).unwrap(),
        events
    );
    assert_eq!(actual, resumed);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::TurnStarted { round: 2, .. }))
    );
    let combat = actual.combat.as_ref().unwrap();
    assert_eq!(combat.round, before.combat.as_ref().unwrap().round + 1);
    assert_eq!(
        combat.round_scripts.counter,
        before
            .combat
            .as_ref()
            .unwrap()
            .round_scripts
            .counter
            .wrapping_add(1)
    );
    assert_eq!(combat.enemies.records[5].bytes[0xb5], 0);
    assert!(word(&combat.enemies.records[10].bytes, 0x110).unwrap() < 1);
    assert!(!combat.order.contains(&20));
    assert_eq!(actual.health_context.as_ref().unwrap().last_body, 20);
    assert_eq!(
        actual.health_context.as_ref().unwrap().effect,
        combat.attack_packet.as_ref().unwrap()[18] as i8
    );
    assert_eq!(actual.attack_packet, combat.attack_packet);
    assert_eq!(actual.attack_identity, combat.attack_identity);
    assert!(actual.random.draws > before.random.draws);
    assert_eq!(
        combat
            .spell_fields
            .as_chunks::<20>()
            .0
            .iter()
            .filter(|record| record[15] != 0)
            .count(),
        3
    );
    for _ in 0..3 {
        save(&path, &actual, &content).unwrap();
        let mut loaded = load(&path, &content).unwrap();
        assert_eq!(loaded, actual);
        let events = apply(Command::EndTurn, &mut actual, &content).unwrap();
        assert_eq!(
            apply(Command::EndTurn, &mut resumed, &content).unwrap(),
            events
        );
        assert_eq!(
            apply(Command::EndTurn, &mut loaded, &content).unwrap(),
            events
        );
        assert_eq!(actual, resumed);
        assert_eq!(actual, loaded);
    }
    save(&path, &actual, &content).unwrap();
    assert_eq!(load(&path, &content).unwrap(), actual);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn later_unsupported_round_or_death_rejects_the_whole_public_command() {
    let (content, f) = fixture();
    let original = controlled_round(&f, &content);
    let items = content.items.clone();
    let attacks = content
        .map(2)
        .unwrap()
        .combat
        .as_ref()
        .unwrap()
        .attacks
        .clone();
    let cache = content.map(2).unwrap().item_cache.clone();
    let mut accepted = original.clone();
    apply(Command::EndTurn, &mut accepted, &content).unwrap();
    assert_eq!(
        accepted.combat.as_ref().unwrap().enemies.records[5].bytes[0xb5],
        0
    );
    assert!(
        word(
            &accepted.combat.as_ref().unwrap().enemies.records[10].bytes,
            0x110
        )
        .unwrap()
            < 1
    );
    assert!(accepted.random.draws > original.random.draws);
    for variant in 0..3 {
        let mut state = original.clone();
        match variant {
            0 => state.combat.as_mut().unwrap().enemies.records[13].bytes[0xcc] = 2,
            1 => state.combat.as_mut().unwrap().enemies.records[10].bytes[0x10..0x12]
                .copy_from_slice(&4_i16.to_be_bytes()),
            _ => state.combat.as_mut().unwrap().spell_fields[37] = 1,
        }
        state.validate(&content).unwrap();
        let before = state.clone();
        assert!(matches!(
            apply(Command::EndTurn, &mut state, &content),
            Err(RuleError::Unsupported { .. })
        ));
        assert_eq!(state, before, "atomic public variant {variant}");
        assert_eq!(content.items, items);
        assert_eq!(
            content.map(2).unwrap().combat.as_ref().unwrap().attacks,
            attacks
        );
        assert_eq!(content.map(2).unwrap().item_cache, cache);
    }
}
