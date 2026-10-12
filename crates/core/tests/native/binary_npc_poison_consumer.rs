use crate::prelude::*;

fn fixture() -> (Content, Value) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = crate::support::content();
    let fixture = serde_json::from_str(
        &std::fs::read_to_string(root.join("evidence/fixtures/npc-poison-consumer.json")).unwrap(),
    )
    .unwrap();
    (content, fixture)
}

fn bytes(f: &Value, value: &Value) -> Vec<u8> {
    let hex = value.as_str().unwrap_or_else(|| {
        f["shared_rule_buffers"][value.text("shared_buffer_sha256")]
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
        crate::support::expect(decoded.len() as u64, &value["byte_length"]);
    }
    decoded
}

fn words(f: &Value, value: &Value) -> Vec<i16> {
    crate::support::words(&bytes(f, value))
}

fn positions(f: &Value, snapshot: &Value, key: &str) -> Vec<(i16, i16)> {
    let origin = &f["earned_native_caller"]["before"];
    let dx = snapshot.n::<i16>("shift_x") - origin.n::<i16>("shift_x");
    let dy = snapshot.n::<i16>("shift_y") - origin.n::<i16>("shift_y");
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
    let count = snapshot.n::<usize>("body_count");
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
    combat.cursor = snapshot.n::<usize>("cursor");
    combat.round_scripts.counter = snapshot.n::<i8>("round_script_counter");
    combat.spell_fields = bytes(f, &snapshot["fields"]);
    combat.field_used = bytes(f, &snapshot["field_used"]);
    combat.attack_packet = Some(bytes(f, &snapshot["packet"]));
    combat.attack_identity = Some(words(f, &snapshot["identity"]).try_into().unwrap());
    state.attack_packet = combat.attack_packet.clone();
    state.attack_identity = combat.attack_identity;
    state.attack_count = snapshot.n::<i16>("count");
    state
        .party_mask
        .copy_from_slice(&words(f, &snapshot["mask"])[..6]);
    state.selected_warrior = snapshot.n::<usize>("active_member");
    state.bandage_ready = Some(snapshot.n::<i16>("aid_gate"));
    state.enemy_spell_disabled = snapshot.n::<u8>("enemy_spell_disabled");
    state.random.seed = snapshot.n::<u32>("seed");
    state.random.draws = snapshot.n::<u64>("draws");
    state.ammunition_work.item = Some(bytes(f, &snapshot["item_work"]));
    state.ammunition_work.weapon = Some(bytes(f, &snapshot["weapon_work"]));
    state.ammunition_work.ammo = Some(bytes(f, &snapshot["ammo_work"]));
    state.loaded_item_field32 =
        Some(word(state.ammunition_work.item.as_ref().unwrap(), 32).unwrap());
    state.flags = bytes(f, &snapshot["flags"]);
    state.allies = bytes(f, &snapshot["references"]);
    state.psi_memory = Some(bytes(f, &snapshot["memory"]));
    state.party_buffs = bytes(f, &snapshot["party_buffs"]);
    state.crate_items = crate::snapshots::inventory(&|key| bytes(f, &snapshot[key]), "crate");
    state.booty_items = crate::snapshots::inventory(&|key| bytes(f, &snapshot[key]), "loot");
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
    health.effect = snapshot.n::<i8>("health_effect");
    health.last_body = snapshot.n::<i16>("health_last_body");
    health.marks_affected = snapshot.n::<i64>("heal_marks_affected") != 0;
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
    let sounds = case.array("sounds");
    let phases = case.array("native_phases");
    let mut events = vec![];
    let mut cursor = 0;
    for collision in phases.iter().filter(|phase| phase["address"] == "1140ae") {
        let body = collision["arguments"][0].as_i64().unwrap() as i16;
        let stop = collision["after"].n::<usize>("sound_count");
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
                damage: damage.n::<i16>("d0_return_word"),
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
    let cases = f.array("cases");

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
        for draw in case.array("rng") {
            crate::support::expect(receipt.bound(draw.n::<i16>("bound")), &draw["result"]);
        }
        assert_eq!(receipt, expected.random, "RNG case {index}");
        let combat = actual.combat.as_ref().unwrap();
        assert!(word(&combat.enemies.records[10].bytes, 0x110).unwrap() < 1);
        assert!(!combat.order.contains(&20));
        assert_eq!(actual.health_context.as_ref().unwrap().last_body, 20);

        for key in ["party_positions", "npc_positions"] {
            assert_eq!(
                positions(&f, &case["before"], key),
                positions(&f, &case["after"], key),
                "inverse centering {key} case {index}"
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
