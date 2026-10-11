use centurions_content::Content;
use centurions_core::{Command, Event, RuleError, Session, apply, load, save};
use serde_json::Value;

fn fixture(name: &str) -> Value {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    serde_json::from_str(
        &std::fs::read_to_string(root.join("evidence/fixtures").join(name)).unwrap(),
    )
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
    value
        .as_str()
        .unwrap()
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn words(value: &Value) -> Vec<i16> {
    bytes(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| i16::from_be_bytes(*p))
        .collect()
}
fn positions(value: &Value) -> Vec<(i16, i16)> {
    words(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| (p[0], p[1]))
        .collect()
}
fn native_work(s: &Value) -> centurions_core::combat::weapon_work::WeaponWork {
    let weapon = bytes(&s["weapon_work"]);
    let ammo = bytes(&s["ammo_work"]);
    centurions_core::combat::weapon_work::WeaponWork {
        sound: Some(centurions_content::word(&weapon, 0x0e).unwrap()),
        weapon_armor_filter: Some(centurions_content::word(&weapon, 0x88).unwrap()),
        ammunition_armor_filter: Some(centurions_content::word(&ammo, 0x88).unwrap()),
    }
}
fn copy_snapshot(state: &mut Session, s: &Value) {
    for (w, record) in state
        .warriors
        .iter_mut()
        .zip(bytes(&s["party"]).as_chunks::<3138>().0)
    {
        w.bytes = record.to_vec();
    }
    state.allies = bytes(&s["references"]);
    state.ally_count = s["ally_count"].as_u64().unwrap() as usize;
    state.ammunition = bytes(&s["ammunition"])
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| i32::from_be_bytes(*p))
        .collect();
    state.exploration.as_mut().unwrap().fatigue = s["fatigue"].as_i64().unwrap() as i16;
    state.psi_memory = Some(bytes(&s["memory"]));
    state.party_buffs = bytes(&s["party_buffs"]);
    state.loaded_item_field32 = Some(s["item_context"].as_i64().unwrap() as i16);
    state.attack_count = s["count"].as_i64().unwrap() as i16;
    state.random.seed = s["seed"].as_u64().unwrap() as u32;
    state.random.draws = s["draws"].as_u64().unwrap();
    let c = state.combat.as_mut().unwrap();
    c.map.tiles = words(&s["map"]);
    c.squad.positions = positions(&s["party_positions"]);
    c.enemies.records = bytes(&s["records"])
        .as_chunks::<288>()
        .0
        .iter()
        .map(|p| centurions_content::combat::MonsterRecord { bytes: p.to_vec() })
        .collect();
    c.enemies.positions = positions(&s["monster_positions"]);
    c.order = words(&s["order"]);
    c.round_scripts.slots = words(&s["round_slots"]).try_into().unwrap();
    c.cursor = s["cursor"].as_u64().unwrap() as usize;
    c.round = s["round"].as_i64().unwrap() as i16;
    c.unavailable = s["unavailable"].as_i64().unwrap() as i8;
    c.spell_fields = bytes(&s["fields"]);
    c.field_used = bytes(&s["used"]);
    c.attack_packet = Some(bytes(&s["packet"]));
    c.attack_identity = Some(words(&s["identity"]).try_into().unwrap());
    c.weapon_work = native_work(s);
    state.weapon_work = c.weapon_work;
    state.attack_packet = c.attack_packet.clone();
    state.attack_identity = c.attack_identity;
    let mut health = centurions_core::story_health::context(
        &state.warriors,
        state.selected_warrior,
        state.health_context.as_ref(),
    );
    health.unavailable = c.unavailable;
    state.health_context = Some(health);
}

fn initial(f: &Value) -> Session {
    serde_json::from_value(f["foundation"]["session"].clone()).unwrap()
}
fn assert_session(actual: &Session, expected: &Session) {
    let actual = serde_json::to_value(actual).unwrap();
    let expected = serde_json::to_value(expected).unwrap();
    for (key, value) in expected.as_object().unwrap() {
        assert_eq!(&actual[key], value, "Session field {key}");
    }
}
fn decision(state: &mut Session, content: &Content) -> Vec<Event> {
    use centurions_core::combat::runtime::{self, EnemyWork};
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

#[test]
fn reached_poison_actor_and_actual_getup_match_native_whole_session_events_and_resume() {
    let content = content();
    let f = fixture("poison-psi.json");
    let case = &f["cases"][0];
    let before = initial(&f);
    before.validate(&content).unwrap();
    let mut projected = before.clone();
    copy_snapshot(&mut projected, &case["before"]);
    assert_session(&before, &projected);
    assert_eq!(
        content.map(2).unwrap().item_cache,
        bytes(&case["before"]["first_item_cache"])
    );
    assert_eq!(
        content.map(2).unwrap().item_cache,
        bytes(&case["after_getup"]["first_item_cache"])
    );
    let mut expected = before.clone();
    copy_snapshot(&mut expected, &case["after_getup"]);
    let effect = bytes(&case["after_getup"]["packet"])[18] as i8;
    expected.health_context.as_mut().unwrap().effect = effect;
    expected.bandage_ready = Some(1);
    expected.selected_warrior = 2; // Getup 12659c selects the incoming party member.
    expected.health_context.as_mut().unwrap().selected_warrior = 2;
    // Getup's party combatupdate2 refresh uses actual BYTE9dd/WORDca or e4;
    // the historical probe skipped the complete 150-byte loaditem writer.
    let actor = &expected.warriors[2].bytes;
    let selected =
        centurions_content::word(actor, if actor[0x9dd] == 0 { 0xca } else { 0xe4 }).unwrap();
    assert_eq!(selected, 1);
    let mut item = content.items[usize::from(selected.unsigned_abs())].clone();
    let class = centurions_content::word(&item, 26).unwrap().wrapping_abs();
    item[26..28].copy_from_slice(&class.to_be_bytes());
    assert_eq!(
        expected.loaded_item_field32,
        Some(centurions_content::word(&item, 32).unwrap())
    );
    expected.ammunition_work.item = Some(item);
    let expected_events = vec![
        Event::Sound { resource_id: 431 },
        Event::Sound { resource_id: 430 },
        Event::SpellResolved {
            caster: Some(16),
            spell: 2206,
            target: 4,
            damage: 0,
            defeated: false,
        },
        Event::Sound { resource_id: 138 },
        Event::TurnStarted { body: 2, round: 2 },
    ];
    let expected_sounds: Vec<_> = expected_events
        .iter()
        .filter_map(|e| match e {
            Event::Sound { resource_id } => Some(*resource_id),
            _ => None,
        })
        .collect();
    assert_eq!(
        serde_json::to_value(expected_sounds).unwrap(),
        case["sounds"]
    );
    let mut receipt = before.random.clone();
    for draw in case["rng"].as_array().unwrap() {
        assert_eq!(
            receipt.bound(draw["bound"].as_i64().unwrap() as i16),
            draw["result"].as_i64().unwrap() as i16
        );
    }
    assert_eq!(receipt, expected.random);
    assert_eq!(
        centurions_content::word(&expected.warriors[4].bytes, 0x64).unwrap(),
        5
    );
    assert_eq!(expected.combat.as_ref().unwrap().active_body(), Some(2));
    let path = std::env::temp_dir().join(format!(
        "centurions-poison-caller-{}.json",
        std::process::id()
    ));
    save(&path, &before, &content).unwrap();
    let mut resumed = load(&path, &content).unwrap();
    let mut state = before;
    assert_eq!(
        apply(Command::RunEnemyTurn, &mut state, &content).unwrap(),
        expected_events
    );
    assert_eq!(state.health_context.as_ref().unwrap().effect, effect);
    assert_eq!(state.attack_packet.as_ref().unwrap()[18] as i8, effect);
    assert_eq!(
        state
            .combat
            .as_ref()
            .unwrap()
            .attack_packet
            .as_ref()
            .unwrap()[18] as i8,
        effect
    );
    assert_session(&state, &expected);
    assert_eq!(
        apply(Command::RunEnemyTurn, &mut resumed, &content).unwrap(),
        expected_events
    );
    assert_session(&resumed, &expected);
    save(&path, &state, &content).unwrap();
    assert_session(&state, &load(&path, &content).unwrap());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn poison_addition_and_cast_costs_roll_back_when_later_round_is_unsupported() {
    let content = content();
    let f = fixture("poison-psi.json");
    let mut state = initial(&f);
    let combat = state.combat.as_mut().unwrap();
    combat.order.fill(-1);
    combat.order[109] = 16;
    combat.cursor = 109;
    state.warriors[2].bytes[0xa2..0xa4].copy_from_slice(&1_i16.to_be_bytes());
    let before = state.clone();
    let mut partial = before.clone();
    let events = decision(&mut partial, &content);
    assert!(partial.random.draws > before.random.draws);
    assert_eq!(
        centurions_content::word(&partial.warriors[4].bytes, 0x64).unwrap(),
        5
    );
    assert!(events.iter().any(|e| matches!(
        e,
        Event::SpellResolved {
            spell: 2206,
            target: 4,
            ..
        }
    )));
    assert!(matches!(
        apply(Command::RunEnemyTurn, &mut state, &content),
        Err(RuleError::Unsupported {
            action: "combat.round",
            dependency: "ongoing regeneration, damage or bleeding"
        })
    ));
    assert_session(&state, &before);
}

#[test]
fn unsupported_poison_category_preserves_complete_public_turn() {
    let mut content = content();
    let f = fixture("poison-psi.json");
    let mut state = initial(&f);
    let before = state.clone();
    let map = state.exploration.as_ref().unwrap().map as usize;
    let offset = 2842 + 406 + 5 * 58;
    std::sync::Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[map].shared)
        .combat
        .as_mut()
        .unwrap()
        .attacks[offset + 19] = 7;
    assert!(matches!(
        apply(Command::RunEnemyTurn, &mut state, &content),
        Err(RuleError::Unsupported {
            action: "combat.spell",
            ..
        })
    ));
    assert_session(&state, &before);
}
