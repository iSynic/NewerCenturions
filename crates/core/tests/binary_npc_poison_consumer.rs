use centurions_content::{Content, InventoryRecord, combat::MonsterRecord, word};
use centurions_core::{Event, RuleError, Session, combat::runtime};
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

fn expected_events(f: &Value, case: &Value) -> Vec<Event> {
    let sounds = case["sounds"].as_array().unwrap();
    let phases = case["native_phases"].as_array().unwrap();
    let mut events = vec![];
    let mut cursor = 0;
    for collision in phases.iter().filter(|phase| phase["address"] == "1140ae") {
        let body = collision["arguments"][0].as_i64().unwrap() as i16;
        let stop = collision["after"]["sound_count"].as_u64().unwrap() as usize;
        events.extend(sounds[cursor..stop].iter().map(|sound| Event::Sound {
            resource_id: sound.as_i64().unwrap() as i16,
        }));
        cursor = stop;
        let damage: Vec<_> = phases
            .iter()
            .filter(|phase| phase["address"] == "15075a" && phase["arguments"][0] == body)
            .collect();
        assert!(damage.len() <= 1);
        if let Some(damage) = damage.first() {
            let identity = words(f, &collision["after"]["identity"]);
            let spell = (identity[0] + 1) * 1000 + (identity[1] + 1) * 100 + identity[2] + 1;
            let npcs = bytes(f, &collision["after"]["npcs"]);
            let health = word(&npcs[(body as usize - 10) * 288..], 0x110).unwrap();
            // Typed outcomes are synthesized at resolution; the slice captures
            // native damage, target records and ordered audio rather than Events.
            events.push(Event::SpellResolved {
                caster: None,
                spell,
                target: body,
                damage: damage["d0_return_word"].as_i64().unwrap() as i16,
                defeated: health < 1,
            });
        }
    }
    events.extend(sounds[cursor..].iter().map(|sound| Event::Sound {
        resource_id: sound.as_i64().unwrap() as i16,
    }));
    events
}

fn consume(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let combat = state.combat.as_mut().unwrap();
    let outcome = runtime::reduce_enemy_round_with_fields(
        combat,
        &mut state.warriors,
        &mut state.random,
        content.map(2),
        &mut state.attack_count,
        &mut state.party_mask,
        state.selected_warrior,
    )?;
    // Project the typed slice outcome through the same Session bookkeeping
    // represented by the independently captured native globals.
    state.attack_packet = combat.attack_packet.clone();
    state.attack_identity = combat.attack_identity;
    let health = state.health_context.as_mut().unwrap();
    health.effect = combat.attack_packet.as_ref().unwrap()[18] as i8;
    if let Some(body) = outcome.last_defeated_body {
        health.last_body = body;
    }
    if outcome
        .events
        .iter()
        .any(|event| matches!(event, Event::SpellResolved { damage, .. } if *damage > 0))
    {
        state.bandage_ready = Some(0);
    }
    Ok(outcome.events)
}

#[test]
fn npc_poison_slice_matches_complete_native_records_fields_work_and_rng_for_21_cases() {
    let (content, f) = fixture();
    let cases = f["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 21);
    for (index, case) in cases.iter().enumerate() {
        let before = initial(&f, case, &content);
        let mut expected = before.clone();
        project(&mut expected, &f, &case["after"]);
        let mut actual = before.clone();
        assert_eq!(
            consume(&mut actual, &content).unwrap(),
            expected_events(&f, case),
            "events case {index}"
        );
        assert_eq!(actual, expected, "complete Session projection case {index}");
        let mut receipt = before.random.clone();
        for draw in case["rng"].as_array().unwrap() {
            assert_eq!(
                receipt.bound(draw["bound"].as_i64().unwrap() as i16),
                draw["result"].as_i64().unwrap() as i16
            );
        }
        assert_eq!(receipt, expected.random, "RNG case {index}");
        let combat = actual.combat.as_ref().unwrap();
        assert!(word(&combat.enemies.records[10].bytes, 0x110).unwrap() < 1);
        assert!(!combat.order.contains(&20));
        assert_eq!(actual.health_context.as_ref().unwrap().last_body, 20);
        assert_eq!(
            case["after"]["killed_count"],
            case["before"]["killed_count"].as_i64().unwrap() + 1
        );
        for key in ["party_positions", "npc_positions"] {
            assert_eq!(
                positions(&f, &case["before"], key),
                positions(&f, &case["after"], key),
                "inverse centering {key} case {index}"
            );
        }
        for key in [
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
            "attack_cache",
            "item_work",
            "weapon_work",
            "ammo_work",
            "item_cache0",
            "item_cache1",
            "item_cache2",
            "item_cache3",
            "scenario_cache",
            "active_region",
            "event_headers",
            "terrain_property_work",
            "selected_warrior_work",
            "remove_warrior_work",
            "music_context",
        ] {
            assert_eq!(
                case["before"][key], case["after"][key],
                "retained native {key} case {index}"
            );
        }
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
        assert!(combat.field_used.iter().all(|value| *value == 0));
        assert_eq!(
            content.map(2).unwrap().item_cache,
            bytes(&f, &case["before"]["item_cache0"])
        );
        assert_eq!(
            content.map(2).unwrap().combat.as_ref().unwrap().attacks,
            bytes(&f, &case["before"]["attack_cache"])
        );
    }
}

#[test]
fn unsupported_later_npc_round_work_rolls_back_complete_slice_input() {
    let (content, f) = fixture();
    let original = initial(&f, &f["cases"][3], &content);
    for variant in 0..3 {
        let mut state = original.clone();
        match variant {
            0 => state.combat.as_mut().unwrap().enemies.records[13].bytes[0xcc] = 2,
            1 => state.combat.as_mut().unwrap().enemies.records[10].bytes[0x10..0x12]
                .copy_from_slice(&4_i16.to_be_bytes()),
            _ => state.combat.as_mut().unwrap().spell_fields[37] = 1,
        }
        let before = state.clone();
        assert!(matches!(
            consume(&mut state, &content),
            Err(RuleError::Unsupported { .. })
        ));
        assert_eq!(state, before, "atomic slice variant {variant}");
    }
}

#[test]
fn malformed_slice_tables_reject_without_panicking_or_committing() {
    let (content, f) = fixture();
    let original = initial(&f, &f["cases"][0], &content);
    for variant in 0..3 {
        let mut state = original.clone();
        match variant {
            0 => state.combat.as_mut().unwrap().map.tiles.clear(),
            1 => state.combat.as_mut().unwrap().order.clear(),
            _ => state.selected_warrior = 6,
        }
        let before = state.clone();
        assert!(matches!(
            consume(&mut state, &content),
            Err(RuleError::Invalid(_))
        ));
        assert_eq!(state, before, "invalid slice variant {variant}");
    }
}
