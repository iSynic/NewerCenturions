use centurions_content::Content;
use centurions_core::{Command, Event, Phase, RandomState, Session, apply, load, save};
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
    state.psi_memory = Some(bytes(&s["memory"]));
    state.party_buffs = bytes(&s["party_buffs"]);
    state.loaded_item_field32 = Some(s["item_context"].as_i64().unwrap() as i16);
    state.attack_count = s["count"].as_i64().unwrap() as i16;
    state.random.seed = s["seed"].as_u64().unwrap() as u32;
    state.random.draws = s["draws"].as_u64().unwrap();
    let c = state.combat.as_mut().unwrap();
    c.map.shift = (0, 0);
    c.map.tiles = words(&s["map"]);
    c.squad.positions = positions(&s["party_positions"]);
    c.squad.under_tiles = vec![1155; 6];
    c.enemies.records = bytes(&s["records"])
        .as_chunks::<288>()
        .0
        .iter()
        .map(|p| centurions_content::combat::MonsterRecord { bytes: p.to_vec() })
        .collect();
    c.enemies.positions = positions(&s["monster_positions"]);
    c.order = words(&s["order"]);
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
fn state(content: &Content, case: &Value) -> Session {
    let mut state = Session::new(content);
    apply(Command::ImportStarters, &mut state, content).unwrap();
    apply(
        Command::EnterMission {
            name: "The Thunder of Guns".into(),
        },
        &mut state,
        content,
    )
    .unwrap();
    let p = fixture("player-psi.json");
    state.combat = Some(serde_json::from_value(p["route_foundation"]["combat"].clone()).unwrap());
    state.phase = Phase::Combat;
    state.random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
    copy_snapshot(&mut state, &case["before"]);
    state
}
fn command(case: &Value) -> Command {
    Command::CastPsi {
        spell: 2101,
        power: case["power"].as_i64().unwrap() as i16,
        target: Some((20, 20)),
    }
}
fn assert_session(actual: &Session, expected: &Session) {
    let a = serde_json::to_value(actual).unwrap();
    let e = serde_json::to_value(expected).unwrap();
    for (key, value) in e.as_object().unwrap() {
        assert!(a[key] == *value, "Session field {key} differs");
    }
}

#[test]
fn retained_work_cast_and_automatic_initiative_match_complete_native_session_and_resume() {
    let content = content();
    let f = fixture("warm-psi-work.json");
    for (index, case) in f["cases"].as_array().unwrap().iter().enumerate() {
        let mut state = state(&content, case);
        let before = state.clone();
        let path = std::env::temp_dir().join(format!(
            "centurions-warm-{}-{index}.json",
            std::process::id()
        ));
        save(&path, &state, &content).unwrap();
        let mut resumed = load(&path, &content).unwrap();
        let events = apply(command(case), &mut state, &content).unwrap();
        assert_eq!(
            apply(command(case), &mut resumed, &content).unwrap(),
            events
        );
        assert_session(&state, &resumed);
        let mut expected = before;
        copy_snapshot(&mut expected, &case["after"]);
        let effect = bytes(&case["after"]["packet"])[18] as i8;
        expected.health_context.as_mut().unwrap().effect = effect;
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
        let identity = words(&case["after"]["identity"]);
        expected.psi_menu_choices[1] = Some(centurions_core::PsiMenuChoice {
            row: identity[1] as u8,
            column: identity[2] as u8,
            power: case["after"]["count"].as_i64().unwrap() as i16,
        });
        expected.bandage_ready = Some(1);
        assert_session(&state, &expected);
        let mut expected_events: Vec<_> = case["after"]["sounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| Event::Sound {
                resource_id: v.as_i64().unwrap() as i16,
            })
            .collect();
        expected_events.push(Event::SpellResolved {
            caster: Some(1),
            spell: 2101,
            target: 10,
            damage: centurions_content::word(&bytes(&case["before"]["records"]), 0x110)
                .unwrap()
                .wrapping_sub(
                    centurions_content::word(&bytes(&case["after"]["records"]), 0x110).unwrap(),
                ),
            defeated: false,
        });
        expected_events.push(Event::TurnStarted { body: 10, round: 1 });
        assert_eq!(events, expected_events);
        save(&path, &state, &content).unwrap();
        assert_session(&state, &load(&path, &content).unwrap());
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn unsupported_next_actor_rolls_back_created_field_costs_damage_and_rng() {
    let content = content();
    let f = fixture("warm-psi-work.json");
    for case in f["cases"].as_array().unwrap() {
        let mut state = state(&content, case);
        let monster = &mut state.combat.as_mut().unwrap().enemies.records[0].bytes;
        monster[0x5e..0x60].copy_from_slice(&32767_i16.to_be_bytes());
        monster[0x5c..0x5e].copy_from_slice(&100_i16.to_be_bytes());
        let before = state.clone();
        assert!(apply(command(case), &mut state, &content).is_err());
        assert_session(&state, &before);
    }
}

#[test]
fn invalid_target_power_and_exhausted_psi_preserve_complete_session() {
    let content = content();
    let f = fixture("warm-psi-work.json");
    for (power, target, mp) in [
        (0, Some((20, 20)), 100),
        (3, Some((20, 20)), 100),
        (1, None, 100),
        (1, Some((40, 40)), 100),
        (1, Some((20, 20)), 7),
        (2, Some((20, 20)), 15),
    ] {
        let mut state = state(&content, &f["cases"][0]);
        state.warriors[1].bytes[0x58..0x5a].copy_from_slice(&i16::to_be_bytes(mp));
        let before = state.clone();
        assert!(
            apply(
                Command::CastPsi {
                    spell: 2101,
                    power,
                    target
                },
                &mut state,
                &content
            )
            .is_err()
        );
        assert_session(&state, &before);
    }
}

#[test]
fn legacy_or_partially_unknown_work_is_preserved_and_required_reads_are_atomic() {
    use centurions_core::combat::weapon_work::WeaponWork;
    let content = content();
    let f = fixture("warm-psi-work.json");
    let source = state(&content, &f["cases"][1]);
    let path = std::env::temp_dir().join(format!(
        "centurions-legacy-weapon-work-{}.json",
        std::process::id()
    ));
    for version in [1, 2, 3] {
        let mut document = serde_json::json!({"schema_version": version, "session": source});
        document["session"]
            .as_object_mut()
            .unwrap()
            .remove("weapon_work");
        document["session"]["combat"]
            .as_object_mut()
            .unwrap()
            .remove("weapon_work");
        std::fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
        let mut legacy = load(&path, &content).unwrap();
        assert_eq!(legacy.weapon_work, WeaponWork::default());
        assert_eq!(
            legacy.combat.as_ref().unwrap().weapon_work,
            WeaponWork::default()
        );
        let before = legacy.clone();
        assert!(apply(command(&f["cases"][1]), &mut legacy, &content).is_err());
        assert_session(&legacy, &before);
    }
    for missing in 0..3 {
        let mut partial = source.clone();
        let work = &mut partial.combat.as_mut().unwrap().weapon_work;
        match missing {
            0 => work.sound = None,
            1 => work.weapon_armor_filter = None,
            _ => work.ammunition_armor_filter = None,
        }
        partial.weapon_work = *work;
        save(&path, &partial, &content).unwrap();
        let mut resumed = load(&path, &content).unwrap();
        assert_session(&resumed, &partial);
        let before = resumed.clone();
        assert!(apply(command(&f["cases"][1]), &mut resumed, &content).is_err());
        assert_session(&resumed, &before);
    }
    std::fs::remove_file(path).unwrap();
}
