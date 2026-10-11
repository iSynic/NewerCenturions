use centurions_content::{Content, Warrior};
use centurions_core::{
    Command, Event, RuleError, Session, apply,
    combat::runtime::{self, EnemyWork},
    load, save,
};
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../evidence/fixtures/restricted-npc-spell.json"
    ))
    .unwrap()
}
fn content() -> Content {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap()
}
fn bytes(value: &Value) -> Vec<u8> {
    let hex = value.as_str().unwrap();
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
fn words(value: &Value) -> Vec<i16> {
    bytes(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| i16::from_be_bytes(*pair))
        .collect()
}
fn assert_snapshot(state: &Session, snapshot: &Value, item_context: Option<i16>) {
    let combat = state.combat.as_ref().unwrap();
    for (name, actual) in [
        (
            "party",
            state
                .warriors
                .iter()
                .flat_map(|w| w.bytes.iter().copied())
                .collect::<Vec<_>>(),
        ),
        (
            "records",
            combat
                .enemies
                .records
                .iter()
                .flat_map(|m| m.bytes.iter().copied())
                .collect(),
        ),
        ("fields", combat.spell_fields.clone()),
        ("used", combat.field_used.clone()),
        ("packet", combat.attack_packet.clone().unwrap()),
        ("memory", state.psi_memory.clone().unwrap()),
        ("party_buffs", state.party_buffs.clone()),
    ] {
        let expected = bytes(&snapshot[name]);
        assert_eq!(actual.len(), expected.len(), "{name} length");
        if let Some(i) = actual.iter().zip(&expected).position(|(a, b)| a != b) {
            panic!("{name} byte {i:x}: {} != {}", actual[i], expected[i]);
        }
    }
    assert_eq!(combat.map.tiles, words(&snapshot["map"]));
    for (actual, name) in [
        (&combat.squad.positions, "party_positions"),
        (&combat.enemies.positions, "monster_positions"),
    ] {
        assert_eq!(
            *actual,
            words(&snapshot[name])
                .as_chunks::<2>()
                .0
                .iter()
                .map(|p| (p[0], p[1]))
                .collect::<Vec<_>>()
        );
    }
    assert_eq!(combat.order, words(&snapshot["order"]));
    assert_eq!(combat.cursor, snapshot["cursor"].as_u64().unwrap() as usize);
    assert_eq!(combat.round, snapshot["round"].as_i64().unwrap() as i16);
    assert_eq!(
        combat.unavailable,
        snapshot["unavailable"].as_i64().unwrap() as i8
    );
    assert_eq!(
        combat.attack_identity.unwrap().to_vec(),
        words(&snapshot["identity"])
    );
    assert_eq!(
        state.attack_count,
        snapshot["count"].as_i64().unwrap() as i16
    );
    assert_eq!(state.loaded_item_field32, item_context);
    assert_eq!(state.random.seed, snapshot["seed"].as_u64().unwrap() as u32);
    assert_eq!(state.random.draws, snapshot["draws"].as_u64().unwrap());
}
fn decision(state: &mut Session, content: &Content) -> Vec<Event> {
    let combat = state.combat.as_mut().unwrap();
    let mut work = EnemyWork {
        item_context: state.loaded_item_field32,
        attack_count: state.attack_count,
        packet: combat.attack_packet.clone(),
        identity: combat.attack_identity,
        spell_disabled: state.enemy_spell_disabled,
        ..Default::default()
    };
    let events = runtime::enemy_action_with_work_and_buffs(
        combat,
        &mut state.warriors,
        content.map(2).unwrap(),
        &mut state.random,
        &content.items,
        &mut work,
        &state.party_buffs,
    )
    .unwrap();
    state.loaded_item_field32 = work.item_context;
    state.attack_count = work.attack_count;
    events
}

fn source_item_refresh(state: &mut Session, content: &Content, body: i16) {
    // combatupdate2 selects WORD4e for NPCs, or ca/e4 by party BYTE9dd;
    // loaditem copies all150 bytes and absolute-normalizes only WORD26.
    let id = if body >= 10 {
        centurions_content::word(
            &state.combat.as_ref().unwrap().enemies.records[(body - 10) as usize].bytes,
            0x4e,
        )
        .unwrap()
    } else {
        let warrior = &state.warriors[body as usize].bytes;
        centurions_content::word(warrior, if warrior[0x9dd] == 0 { 0xca } else { 0xe4 }).unwrap()
    };
    if body >= 10 && id == 0 {
        return;
    }
    let mut record = content.items[usize::from(id.unsigned_abs())].clone();
    let class = centurions_content::word(&record, 26)
        .unwrap()
        .wrapping_abs();
    record[26..28].copy_from_slice(&class.to_be_bytes());
    state.loaded_item_field32 = Some(centurions_content::word(&record, 32).unwrap());
    state.ammunition_work.item = Some(record);
}

fn source_health_writes(state: &mut Session, case: &Value) {
    // Independent native resolution snapshots supply each positive Health write.
    let mut party = bytes(&case["before"]["party"]);
    let mut records = bytes(&case["before"]["records"]);
    let health = state.health_context.as_mut().unwrap();
    for resolution in case["resolutions"].as_array().unwrap() {
        let after_party = bytes(&resolution["party_after"]);
        let after_records = bytes(&resolution["records_after"]);
        for (body, flag) in words(&resolution["applied_mask"]).iter().enumerate() {
            if *flag <= 0 {
                continue;
            }
            let (before, after, offset) = if body < 6 {
                (&party, &after_party, body * 3138 + 0x50)
            } else {
                (&records, &after_records, (body - 10) * 288 + 0x110)
            };
            let previous = centurions_content::word(before, offset).unwrap();
            let current = centurions_content::word(after, offset).unwrap();
            let damage = previous.wrapping_sub(current);
            if body < 6 && damage > 0 && previous >= 0 {
                health.affected[body] = -1;
            }
            if damage > 0 && current < 1 {
                health.last_body = body as i16;
            }
        }
        party = after_party;
        records = after_records;
    }
}
#[test]
fn reached_npc_restriction_matches_complete_original_decision_and_public_resume() {
    let content = content();
    let fixture = fixture();
    let case = &fixture["cases"][0];
    let mission = content.map(2).unwrap();
    assert_eq!(
        mission.item_cache,
        bytes(&case["before"]["first_item_cache"])
    );
    assert_eq!(
        mission.item_cache,
        bytes(&case["after"]["first_item_cache"])
    );
    let mut initial: Session =
        serde_json::from_value(fixture["foundation"]["session"].clone()).unwrap();
    initial.validate(&content).unwrap();
    assert_snapshot(&initial, &case["before"], Some(1));
    let actor = initial.combat.as_ref().unwrap().active_body().unwrap();
    let mut expected = initial.clone();
    let mut events = decision(&mut expected, &content);
    // Historical combatupdate2 was skipped. NPC21's actual selected item
    // WORD4e=187 reaches loaditem11fd68; its decimal WORD32 is zero.
    assert_eq!(
        centurions_content::word(&bytes(&case["before"]["records"]), 11 * 288 + 0x4e).unwrap(),
        187
    );
    let refreshed = centurions_content::word(&content.items[187], 32).unwrap();
    assert_eq!(refreshed, 0);
    assert_snapshot(&expected, &case["after"], Some(refreshed));
    let sounds = events
        .iter()
        .filter_map(|e| match e {
            Event::Sound { resource_id } => Some(*resource_id),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(serde_json::to_value(sounds).unwrap(), case["sounds"]);
    let mut receipt = initial.random.clone();
    for draw in case["rng"].as_array().unwrap() {
        assert_eq!(
            receipt.bound(draw["bound"].as_i64().unwrap() as i16),
            draw["result"].as_i64().unwrap() as i16
        );
    }
    assert_eq!(receipt, expected.random);
    let mut unchanged = initial.clone();
    for (w, native) in unchanged
        .warriors
        .iter_mut()
        .zip(bytes(&case["after"]["party"]).as_chunks::<3138>().0)
    {
        w.bytes = Warrior::read(native).unwrap().bytes;
    }
    unchanged.random = expected.random.clone();
    unchanged.attack_count = expected.attack_count;
    unchanged.loaded_item_field32 = expected.loaded_item_field32;
    unchanged.combat = expected.combat.clone();
    source_item_refresh(&mut unchanged, &content, actor);
    source_item_refresh(&mut expected, &content, actor);
    assert_eq!(unchanged, expected);
    // The independent decision ends before Getup; compose its separately recovered suffix.
    events.extend(runtime::end_turn(&mut expected, &content).unwrap());
    let combat = expected.combat.as_ref().unwrap();
    expected.attack_packet = combat.attack_packet.clone();
    expected.attack_identity = combat.attack_identity;
    expected.weapon_work = combat.weapon_work;
    let mut health = centurions_core::story_health::context(
        &expected.warriors,
        expected.selected_warrior,
        expected.health_context.as_ref(),
    );
    health.unavailable = combat.unavailable;
    health.effect = bytes(&case["after"]["packet"])[18] as i8;
    assert_eq!(
        combat.attack_packet.as_ref().unwrap()[18] as i8,
        health.effect
    );
    expected.health_context = Some(health);
    source_health_writes(&mut expected, case);
    let successor = expected.combat.as_ref().unwrap().active_body().unwrap();
    source_item_refresh(&mut expected, &content, successor);
    let path = std::env::temp_dir().join(format!(
        "centurions-restricted-npc-caller-{}.json",
        std::process::id()
    ));
    save(&path, &initial, &content).unwrap();
    let mut resumed = load(&path, &content).unwrap();
    assert_eq!(
        apply(Command::RunEnemyTurn, &mut initial, &content).unwrap(),
        events
    );
    assert_eq!(initial, expected);
    assert_eq!(
        initial.health_context.as_ref().unwrap().effect,
        bytes(&case["after"]["packet"])[18] as i8
    );
    assert_eq!(
        initial.attack_packet.as_ref().unwrap()[18] as i8,
        initial.health_context.as_ref().unwrap().effect
    );
    assert_eq!(
        apply(Command::RunEnemyTurn, &mut resumed, &content).unwrap(),
        events
    );
    assert_eq!(resumed, expected);
    save(&path, &initial, &content).unwrap();
    assert_eq!(load(&path, &content).unwrap(), initial);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn later_unsupported_initiative_rolls_back_restricted_casts_and_rng() {
    let content = content();
    let fixture = fixture();
    let mut state: Session =
        serde_json::from_value(fixture["foundation"]["session"].clone()).unwrap();
    let combat = state.combat.as_mut().unwrap();
    combat.order.fill(-1);
    combat.order[109] = 21;
    combat.cursor = 109;
    state.warriors[0].bytes[0xa2..0xa4].copy_from_slice(&1_i16.to_be_bytes());
    let before = state.clone();
    let mut partial = before.clone();
    let events = decision(&mut partial, &content);
    assert!(partial.random.draws > before.random.draws);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::SpellResolved { spell: 2207, .. })),
        "{events:?}"
    );
    assert!(matches!(
        apply(Command::RunEnemyTurn, &mut state, &content),
        Err(RuleError::Unsupported {
            action: "combat.round",
            dependency: "ongoing regeneration, damage or bleeding",
        })
    ));
    assert_eq!(state, before);
}

#[test]
fn missing_item_cache_rolls_back_the_complete_public_enemy_turn() {
    let mut content = content();
    let fixture = fixture();
    let mut state: Session =
        serde_json::from_value(fixture["foundation"]["session"].clone()).unwrap();
    let before = state.clone();
    let map = state.exploration.as_ref().unwrap().map as usize;
    std::sync::Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[map].shared)
        .item_cache
        .clear();
    assert!(matches!(
        apply(Command::RunEnemyTurn, &mut state, &content),
        Err(RuleError::Unsupported {
            action: "combat.spell",
            ..
        })
    ));
    assert_eq!(state, before);
}
