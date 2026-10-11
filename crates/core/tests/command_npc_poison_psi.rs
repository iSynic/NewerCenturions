use centurions_content::{Content, InventoryRecord, combat::MonsterRecord, word};
use centurions_core::{Command, Event, RuleError, Session, apply, load, save};
use serde_json::Value;

fn content() -> Content {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap()
}

fn fixture() -> Value {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixture: Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("evidence/fixtures/qualified-npc-poison-caller.json"))
            .unwrap(),
    )
    .unwrap();
    // Native WORD439594=1 is an explicit effects-only control outside Session;
    // it is not inferred from the saved audio or channel state.
    assert_eq!(fixture["foundation"]["controlled_music_disabled_word"], 1);
    let case = &fixture["cases"][0];
    for snapshot in ["before", "after"] {
        assert_eq!(case[snapshot]["music_context"]["disabled_word"], 1);
    }
    let phases = case["native_music_phases"].as_array().unwrap();
    assert_eq!(phases.len(), 2);
    for phase in phases {
        assert_eq!(phase["address"], "133d46");
        for snapshot in ["before", "after"] {
            assert_eq!(phase[snapshot]["music_context"]["disabled_word"], 1);
        }
        for key in ["seed", "draws", "music_context"] {
            assert_eq!(phase["before"][key], phase["after"][key]);
        }
        let submissions = phase["native_submission"].as_array().unwrap();
        assert_eq!(submissions.len(), 1);
        assert_eq!(submissions[0]["disabled_word"], 1);
        let services = submissions[0]["services"].as_array().unwrap();
        assert_eq!(services.len(), 2);
        for (service, command) in services
            .iter()
            .zip(["0004000000000000", "0003000000000000"])
        {
            assert_eq!(service["service"], "SndDoImmediate");
            assert_eq!(service["command8"], command);
            assert_eq!(service["result_word"], 0);
        }
    }
    fixture
}

fn bytes(fixture: &Value, value: &Value) -> Vec<u8> {
    let hex = if let Some(hex) = value.as_str() {
        hex
    } else {
        fixture["shared_rule_buffers"][value["shared_buffer_sha256"].as_str().unwrap()]
            .as_str()
            .unwrap()
    };
    let result: Vec<_> = hex
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect();
    if value.is_object() {
        assert_eq!(result.len() as u64, value["byte_length"].as_u64().unwrap());
    }
    result
}

fn words(f: &Value, value: &Value) -> Vec<i16> {
    bytes(f, value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| i16::from_be_bytes(*pair))
        .collect()
}

fn normalized_positions(f: &Value, snapshot: &Value, key: &str) -> Vec<(i16, i16)> {
    let before = &f["cases"][0]["before"];
    let offset = (
        snapshot["shift_x"].as_i64().unwrap() as i16 - before["shift_x"].as_i64().unwrap() as i16,
        snapshot["shift_y"].as_i64().unwrap() as i16 - before["shift_y"].as_i64().unwrap() as i16,
    );
    words(f, &snapshot[key])
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            (
                pair[0].wrapping_add(offset.0),
                pair[1].wrapping_add(offset.1),
            )
        })
        .collect()
}

fn project_native(state: &mut Session, f: &Value, snapshot: &Value) {
    for (warrior, record) in state
        .warriors
        .iter_mut()
        .zip(bytes(f, &snapshot["party"]).as_chunks::<3138>().0)
    {
        warrior.bytes = record.to_vec();
    }
    state.flags = bytes(f, &snapshot["flags"]);
    state.allies = bytes(f, &snapshot["references"]);
    state.party_buffs = bytes(f, &snapshot["party_buffs"]);
    state.psi_memory = Some(bytes(f, &snapshot["memory"]));
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
    state.ammunition_work.item = Some(bytes(f, &snapshot["item_work"]));
    state.ammunition_work.weapon = Some(bytes(f, &snapshot["weapon_work"]));
    state.ammunition_work.ammo = Some(bytes(f, &snapshot["ammo_work"]));
    state.loaded_item_field32 =
        Some(word(state.ammunition_work.item.as_ref().unwrap(), 32).unwrap());
    state.attack_count = snapshot["count"].as_i64().unwrap() as i16;
    state.selected_warrior = snapshot["active_member"].as_u64().unwrap() as usize;
    state.bandage_ready = Some(snapshot["aid_gate"].as_i64().unwrap() as i16);
    state.enemy_spell_disabled = snapshot["enemy_spell_disabled"].as_i64().unwrap() as u8;
    state.random.seed = snapshot["seed"].as_u64().unwrap() as u32;
    state.random.draws = snapshot["draws"].as_u64().unwrap();
    let count = snapshot["body_count"].as_u64().unwrap() as usize;
    let all_npcs = bytes(f, &snapshot["npcs"]);
    assert_eq!(all_npcs.len(), 28_800);
    assert!(all_npcs[count * 288..].iter().all(|byte| *byte == 0));
    let positions = normalized_positions(f, snapshot, "npc_positions");
    assert_eq!(positions.len(), 100);
    assert!(positions[count..].iter().all(|point| *point == (0, 0)));
    let combat = state.combat.as_mut().unwrap();
    combat.enemies.records = all_npcs[..count * 288]
        .as_chunks::<288>()
        .0
        .iter()
        .map(|record| MonsterRecord {
            bytes: record.to_vec(),
        })
        .collect();
    combat.enemies.positions = positions[..count].to_vec();
    combat.squad.positions = normalized_positions(f, snapshot, "party_positions");
    combat.map.tiles = words(f, &snapshot["map"]);
    combat.order = words(f, &snapshot["order"]);
    combat.cursor = snapshot["cursor"].as_u64().unwrap() as usize;
    combat.spell_fields = bytes(f, &snapshot["fields"]);
    combat.field_used = bytes(f, &snapshot["field_used"]);
    combat.attack_packet = Some(bytes(f, &snapshot["packet"]));
    combat.attack_identity = Some(words(f, &snapshot["identity"]).try_into().unwrap());
    let weapon = state.ammunition_work.weapon.as_ref().unwrap();
    let ammo = state.ammunition_work.ammo.as_ref().unwrap();
    combat.weapon_work = centurions_core::combat::weapon_work::WeaponWork {
        sound: Some(word(weapon, 14).unwrap()),
        weapon_armor_filter: Some(word(weapon, 136).unwrap()),
        ammunition_armor_filter: Some(word(ammo, 136).unwrap()),
    };
    state.weapon_work = combat.weapon_work;
    state.attack_packet = combat.attack_packet.clone();
    state.attack_identity = combat.attack_identity;
    let health = state.health_context.as_mut().unwrap();
    health.selected_warrior = state.selected_warrior;
    health.effect = snapshot["health_effect"].as_i64().unwrap() as i8;
    health.last_body = snapshot["health_last_body"].as_i64().unwrap() as i16;
    health.marks_affected = snapshot["heal_marks_affected"].as_i64().unwrap() != 0;
    state
        .party_mask
        .copy_from_slice(&words(f, &snapshot["mask"])[..6]);
}

fn initial(f: &Value, content: &Content) -> Session {
    let state: Session = serde_json::from_value(f["foundation"]["session"].clone()).unwrap();
    state.validate(content).unwrap();
    let mut projected = state.clone();
    project_native(&mut projected, f, &f["cases"][0]["before"]);
    assert_eq!(projected, state, "native input projection");
    assert_eq!(state.combat.as_ref().unwrap().active_body(), Some(21));
    assert_eq!(state.combat.as_ref().unwrap().enemies.records.len(), 14);
    state
}

fn expected_events() -> Vec<Event> {
    vec![
        Event::Sound { resource_id: 431 },
        Event::Sound { resource_id: 430 },
        Event::SpellResolved {
            caster: Some(21),
            spell: 2206,
            target: 15,
            damage: 0,
            defeated: false,
        },
        Event::Sound { resource_id: 431 },
        Event::Sound { resource_id: 430 },
        Event::SpellResolved {
            caster: Some(21),
            spell: 2206,
            target: 4,
            damage: 0,
            defeated: false,
        },
        Event::Sound { resource_id: 138 },
        Event::TurnStarted { body: 3, round: 1 },
    ]
}

fn decision_without_getup(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    use centurions_core::combat::runtime::{self, EnemyWork};
    let combat = state.combat.as_mut().unwrap();
    let mut work = EnemyWork {
        item_context: state.loaded_item_field32,
        attack_count: state.attack_count,
        packet: combat.attack_packet.clone(),
        identity: combat.attack_identity,
        spell_disabled: state.enemy_spell_disabled,
        buffers: state.ammunition_work.clone(),
        display_writers: true,
        ..Default::default()
    };
    runtime::enemy_action_with_work_and_buffs(
        combat,
        &mut state.warriors,
        content.map(2).unwrap(),
        &mut state.random,
        &content.items,
        &mut work,
        &state.party_buffs,
    )
}

#[test]
fn actual_npc_poison_actor_two_casts_and_getup_match_complete_native_state_and_resume() {
    let content = content();
    let f = fixture();
    assert_eq!(f["cases"].as_array().unwrap().len(), 1);
    let case = &f["cases"][0];
    let before = initial(&f, &content);
    let native_before = &case["before"];
    let native_after = &case["after"];
    assert_eq!(native_after["shift_x"], 45);
    assert_eq!(native_after["shift_y"], 7);
    // Original centering changes every position and its origin together. The
    // command model retains its logical coordinate frame for the same map cells.
    assert_eq!(
        normalized_positions(&f, native_after, "party_positions"),
        before.combat.as_ref().unwrap().squad.positions
    );
    assert_eq!(
        normalized_positions(&f, native_after, "npc_positions")[..14],
        before.combat.as_ref().unwrap().enemies.positions
    );
    for key in [
        "map",
        "fields",
        "field_used",
        "flags",
        "memory",
        "references",
        "crate",
        "loot",
        "loot_quality",
        "reserves",
        "party_buffs",
        "backup_warriors",
        "weapon_work",
        "ammo_work",
        "attack_cache",
        "item_cache0",
        "item_cache1",
        "item_cache2",
        "item_cache3",
        "scenario_cache",
        "round_script_counter",
        "inactivity_counter",
        "killed_count",
        "event_headers",
        "selected_warrior_work",
        "remove_warrior_work",
        "music_context",
    ] {
        assert_eq!(
            native_before[key], native_after[key],
            "retained native {key}"
        );
    }
    assert_eq!(
        content.map(2).unwrap().item_cache,
        bytes(&f, &native_before["item_cache0"])
    );
    let mut expected = before.clone();
    project_native(&mut expected, &f, native_after);
    let mut original_item = content.items[1].clone();
    let class = word(&original_item, 26).unwrap().wrapping_abs();
    original_item[26..28].copy_from_slice(&class.to_be_bytes());
    assert_eq!(
        expected.ammunition_work.item.as_ref().unwrap(),
        &original_item
    );
    assert_eq!(word(&expected.warriors[3].bytes, 0x3c).unwrap(), 14);
    assert_eq!(word(&expected.warriors[3].bytes, 0x40).unwrap(), 4);
    assert_eq!(word(&expected.warriors[4].bytes, 0x64).unwrap(), 12);
    let actor = &expected.combat.as_ref().unwrap().enemies.records[11].bytes;
    assert_eq!((actor[0x104], actor[0x106], actor[0x109]), (1, 16, 4));
    assert_eq!(word(actor, 0x10e).unwrap(), 236);
    assert_eq!(
        expected.combat.as_ref().unwrap().enemies.records[5].bytes[0xb5],
        1
    );
    assert_eq!(expected.combat.as_ref().unwrap().active_body(), Some(3));
    assert_eq!(
        expected.combat.as_ref().unwrap().round,
        before.combat.as_ref().unwrap().round
    );
    let events = expected_events();
    let sounds: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            Event::Sound { resource_id } => Some(*resource_id),
            _ => None,
        })
        .collect();
    assert_eq!(serde_json::to_value(sounds).unwrap(), case["sounds"]);
    let mut receipt = before.random.clone();
    assert_eq!(case["rng"].as_array().unwrap().len(), 22);
    for draw in case["rng"].as_array().unwrap() {
        assert_eq!(
            receipt.bound(draw["bound"].as_i64().unwrap() as i16),
            draw["result"].as_i64().unwrap() as i16
        );
    }
    assert_eq!(receipt, expected.random);
    let path = std::env::temp_dir().join(format!(
        "centurions-npc-poison-caller-{}.json",
        std::process::id()
    ));
    save(&path, &before, &content).unwrap();
    let mut resumed = load(&path, &content).unwrap();
    assert_eq!(resumed, before);
    let mut actual = before;
    assert_eq!(
        apply(Command::RunEnemyTurn, &mut actual, &content).unwrap(),
        events
    );
    assert_eq!(actual, expected);
    assert_eq!(
        apply(Command::RunEnemyTurn, &mut resumed, &content).unwrap(),
        events
    );
    assert_eq!(resumed, expected);
    save(&path, &actual, &content).unwrap();
    assert_eq!(load(&path, &content).unwrap(), expected);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn unsupported_poison_after_earlier_cast_or_round_writes_rolls_back_whole_public_command() {
    let content = content();
    let f = fixture();
    let original = initial(&f, &content);
    let original_items = content.items.clone();
    let original_attacks = content
        .map(2)
        .unwrap()
        .combat
        .as_ref()
        .unwrap()
        .attacks
        .clone();
    let original_cache = content.map(2).unwrap().item_cache.clone();
    for variant in 0..4 {
        let mut state = original.clone();
        match variant {
            0 => state.warriors[4].bytes[0x9a..0x9c].copy_from_slice(&1_i16.to_be_bytes()),
            1 => {
                let combat = state.combat.as_mut().unwrap();
                combat.order.fill(-1);
                combat.order[109] = 21;
                combat.cursor = 109;
                state.warriors[2].bytes[0xa2..0xa4].copy_from_slice(&1_i16.to_be_bytes());
            }
            2 => {
                let combat = state.combat.as_mut().unwrap();
                combat.order.fill(-1);
                combat.order[109] = 21;
                combat.cursor = 109;
                combat.enemies.records[13].bytes[0xcc] = 2;
            }
            _ => state.combat.as_mut().unwrap().enemies.records[5].bytes[0xd0] = 1,
        }
        state.validate(&content).unwrap();
        let before = state.clone();
        if variant < 3 {
            let mut partial = before.clone();
            let decision = decision_without_getup(&mut partial, &content);
            if variant == 0 {
                assert!(matches!(
                    decision,
                    Err(RuleError::Unsupported {
                        action: "combat.spell",
                        dependency: "poison ward reflection"
                    })
                ));
            } else {
                assert_eq!(decision.unwrap(), expected_events()[..6]);
                assert_eq!(word(&partial.warriors[4].bytes, 0x64).unwrap(), 12);
            }
            assert_eq!(
                partial.combat.as_ref().unwrap().enemies.records[5].bytes[0xb5],
                1
            );
            assert!(partial.random.draws > before.random.draws);
        }
        let error = apply(Command::RunEnemyTurn, &mut state, &content).unwrap_err();
        assert!(matches!(
            error,
            RuleError::Unsupported {
                action: "combat.spell" | "combat.round",
                ..
            }
        ));
        assert_eq!(state, before, "atomic variant {variant}");
        assert_eq!(content.items, original_items);
        assert_eq!(
            content.map(2).unwrap().combat.as_ref().unwrap().attacks,
            original_attacks
        );
        assert_eq!(content.map(2).unwrap().item_cache, original_cache);
    }
    let mut modified_content = content.clone();
    std::sync::Arc::make_mut(&mut modified_content.scenario.as_mut().unwrap().maps[2].shared)
        .combat
        .as_mut()
        .unwrap()
        .attacks[2842 + 406 + 5 * 58 + 19] = 7;
    let mut state = original.clone();
    assert!(matches!(
        apply(Command::RunEnemyTurn, &mut state, &modified_content),
        Err(RuleError::Unsupported {
            action: "combat.spell",
            ..
        })
    ));
    assert_eq!(state, original);
}
