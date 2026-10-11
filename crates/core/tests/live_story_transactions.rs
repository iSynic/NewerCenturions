use centurions_content::Content;
use centurions_core::{Command, Presentation, ScriptState, Session, apply, load, save};
use std::path::{Path, PathBuf};

fn fixture() -> (Content, Session) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let mut state = Session::new(&content);
    apply(Command::ImportStarters, &mut state, &content).unwrap();
    apply(
        Command::EnterMission {
            name: "The Thunder of Guns".into(),
        },
        &mut state,
        &content,
    )
    .unwrap();
    (content, state)
}

fn path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("centurions-{name}-{}.json", std::process::id()))
}

fn carry_script_item(state: &mut Session) -> usize {
    let warrior = &mut state.warriors[0];
    let slot = warrior.inventory().unwrap().len();
    let offset = 0x132 + slot * 22;
    warrior.bytes[offset..offset + 22].fill(0);
    warrior.bytes[offset..offset + 2].copy_from_slice(&832_i16.to_be_bytes());
    warrior.bytes[offset + 2..offset + 4].copy_from_slice(&2_i16.to_be_bytes());
    warrior.bytes[offset + 10] = 23;
    warrior.bytes[0x1a..0x1c].copy_from_slice(&((slot + 1) as i16).to_be_bytes());
    slot
}

fn manual_encounter_fixture() -> (Content, Session) {
    let (content, mut state) = fixture();
    let position = state.exploration.as_mut().unwrap();
    position.restore_map(
        7,
        centurions_core::MapState::authored(content.map(7).unwrap()),
    );
    position.x = 87;
    position.y = 87;
    position.last_move = [1, 1];
    position.script = None;
    position.presentation = None;
    state.mission_stage = 2;
    (content, state)
}

fn hive_schedule_fixture() -> (Content, Session) {
    let (content, mut state) = fixture();
    let position = state.exploration.as_mut().unwrap();
    let map = content.map(2).unwrap();
    position.restore_map(2, centurions_core::MapState::authored(map));
    position.x = 17;
    position.y = 4;
    position.day = 1;
    position.script = Some(ScriptState {
        record: map.script(168).unwrap().to_vec(),
        cursor: 3,
        map_slot: 0,
        scheduled: true,
        menu: None,
        persist: false,
        origin: None,
    });
    position.presentation = Some(Presentation::Dialogue {
        id: 618,
        text: map.dialogue(618).unwrap(),
    });
    state.mission_stage = 1;
    (content, state)
}

#[test]
fn authored_hive_schedule_edit_commits_and_survives_restart() {
    let (content, mut state) = hive_schedule_fixture();
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/schedule-edit.json"
    ))
    .unwrap();
    let case = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["controlled"] == false && case["extra_id"] == 330 && case["day"] == 1)
        .unwrap();
    let decode = |value: &serde_json::Value| {
        let hex = value.as_str().unwrap();
        (0..hex.len())
            .step_by(2)
            .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).unwrap())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        state.exploration.as_ref().unwrap().schedule,
        decode(&case["before"])
    );
    let before_random = state.random.clone();
    let checkpoint = path("hive-schedule");
    save(&checkpoint, &state, &content).unwrap();
    let mut resumed = load(&checkpoint, &content).unwrap();
    let events = apply(Command::Continue, &mut state, &content).unwrap();
    assert_eq!(
        apply(Command::Continue, &mut resumed, &content).unwrap(),
        events
    );
    assert_eq!(resumed, state);
    assert_eq!(
        state.exploration.as_ref().unwrap().schedule,
        decode(&case["after"])
    );
    assert_eq!(state.random, before_random);
    assert!(state.exploration.as_ref().unwrap().script.is_none());
    save(&checkpoint, &state, &content).unwrap();
    assert_eq!(load(&checkpoint, &content).unwrap(), state);
    let committed = state.clone();
    assert!(apply(Command::Continue, &mut state, &content).is_err());
    assert_eq!(state, committed);
    std::fs::remove_file(checkpoint).unwrap();
}

#[test]
fn invalid_schedule_slot_and_later_story_failure_preserve_the_entire_session() {
    for later_failure in [false, true] {
        let (mut content, mut state) = hive_schedule_fixture();
        if later_failure {
            state
                .exploration
                .as_mut()
                .unwrap()
                .script
                .as_mut()
                .unwrap()
                .record[16..18]
                .copy_from_slice(&200_i16.to_be_bytes());
        } else {
            let table =
                std::sync::Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[2].shared);
            table.combat.as_mut().unwrap().encounters[330 * 10..330 * 10 + 2]
                .copy_from_slice(&(-1_i16).to_be_bytes());
        }
        let before = state.clone();
        assert!(apply(Command::Continue, &mut state, &content).is_err());
        assert_eq!(state, before);
    }
}

#[test]
fn manual_encounter_commits_a_resumable_typed_choice_without_advancing_time() {
    let (content, mut state) = manual_encounter_fixture();
    let before = state.clone();
    let events = apply(Command::Encounter, &mut state, &content).unwrap();
    assert!(matches!(
        state.exploration.as_ref().unwrap().presentation,
        Some(Presentation::Typed { id: 8, .. })
    ));
    assert_eq!(state.random.draws, before.random.draws + 3);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, centurions_core::Event::TimeAdvanced { .. }))
    );
    let position = state.exploration.as_ref().unwrap();
    let original = before.exploration.as_ref().unwrap();
    assert_eq!(
        (position.day, position.hour, position.minute),
        (original.day, original.hour, original.minute)
    );
    assert_eq!(position.last_move, [0; 2]);
    let pending = state.clone();
    assert!(apply(Command::Encounter, &mut state, &content).is_err());
    assert_eq!(state, pending);
    let destination = path("manual-encounter");
    save(&destination, &state, &content).unwrap();
    let mut resumed = load(&destination, &content).unwrap();
    assert_eq!(
        apply(Command::CancelMenu, &mut state, &content).unwrap(),
        apply(Command::CancelMenu, &mut resumed, &content).unwrap()
    );
    assert_eq!(resumed, state);
    std::fs::remove_file(destination).unwrap();
}

#[test]
fn unsupported_manual_script_rolls_back_selection_and_randomness() {
    let (mut content, mut state) = manual_encounter_fixture();
    let table = std::sync::Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[7].shared);
    table.door_scripts[250 * 42 + 8..250 * 42 + 10].copy_from_slice(&200_i16.to_be_bytes());
    let before = state.clone();
    assert!(apply(Command::Encounter, &mut state, &content).is_err());
    assert_eq!(state, before);
}

#[test]
fn script_item_cost_commits_once_and_resumes_through_the_command_boundary() {
    let (content, mut state) = fixture();
    let slot = carry_script_item(&mut state);
    apply(Command::UseItem { warrior: 0, slot }, &mut state, &content).unwrap();
    assert_eq!(
        centurions_content::word(&state.warriors[0].bytes, 0x132 + slot * 22 + 2).unwrap(),
        1
    );
    let committed = state.clone();
    assert!(apply(Command::UseItem { warrior: 0, slot }, &mut state, &content).is_err());
    assert_eq!(state, committed);
    let destination = path("script-item");
    save(&destination, &state, &content).unwrap();
    let mut resumed = load(&destination, &content).unwrap();
    for _ in 0..8 {
        if state.exploration.as_ref().unwrap().presentation.is_none() {
            break;
        }
        let events = apply(Command::Continue, &mut state, &content).unwrap();
        assert_eq!(
            apply(Command::Continue, &mut resumed, &content).unwrap(),
            events
        );
        assert_eq!(resumed, state);
    }
    assert!(state.exploration.as_ref().unwrap().script.is_none());
    assert!(
        centurions_core::story_inventory::query_party_item(&state.warriors, 832)
            .unwrap()
            .is_none()
    );
    std::fs::remove_file(destination).unwrap();
}

#[test]
fn unsupported_item_script_rolls_back_consumption_sound_and_work_context() {
    let (mut content, mut state) = fixture();
    let slot = carry_script_item(&mut state);
    let tables = std::sync::Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
    let offset = 216 * 42;
    tables.door_scripts[offset + 8..offset + 10].copy_from_slice(&200_i16.to_be_bytes());
    let before = state.clone();
    assert!(apply(Command::UseItem { warrior: 0, slot }, &mut state, &content).is_err());
    assert_eq!(state, before);
}

fn stage_choice(state: &mut Session, parameter: i16) {
    let position = state.exploration.as_mut().unwrap();
    position.script = Some(ScriptState {
        record: vec![0; 42],
        cursor: 1,
        map_slot: 0,
        scheduled: true,
        menu: None,
        persist: false,
        origin: None,
    });
    position.presentation = Some(Presentation::StageChange { parameter });
}

#[test]
fn failed_story_transaction_discards_mutations_and_ordered_effects() {
    let (content, mut state) = fixture();
    let position = state.exploration.as_mut().unwrap();
    let mut record = vec![0; 42];
    for (offset, value) in [(8, 47_i16), (10, 9), (12, 200), (24, 2), (26, 6000)] {
        record[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    }
    position.script = Some(ScriptState {
        record,
        cursor: 0,
        map_slot: 0,
        scheduled: true,
        menu: None,
        persist: false,
        origin: None,
    });
    position.presentation = Some(Presentation::Dialogue {
        id: 0,
        text: String::new(),
    });
    let before = state.clone();
    assert!(apply(Command::Continue, &mut state, &content).is_err());
    assert_eq!(state, before);
    assert!(apply(Command::SelectMenuEntry { index: 4 }, &mut state, &content).is_err());
    assert_eq!(state, before);
}

#[test]
fn visited_map_consequences_and_pending_choices_survive_v3_reload() {
    let (content, mut state) = fixture();
    let position = state.exploration.as_mut().unwrap();
    position.floor[81] = 123;
    position.objects[82] = -205;
    position.events[83] = 47;
    position.visibility[84] = 1;
    position.event_headers[42] = 19;
    position.region_state[37] = 9;
    let original = position.map_state();
    state.flags[17] = 1;
    state.mission_score = 23;
    stage_choice(&mut state, 2);
    apply(Command::Choose { answer: true }, &mut state, &content).unwrap();
    assert_eq!(state.visited_maps.get(&0), Some(&original));
    let destination = path("visited-choice");
    save(&destination, &state, &content).unwrap();
    assert_eq!(load(&destination, &content).unwrap(), state);
    stage_choice(&mut state, 1);
    apply(Command::Choose { answer: true }, &mut state, &content).unwrap();
    assert_eq!(state.exploration.as_ref().unwrap().map_state(), original);
    assert_eq!(state.flags[17], 1);
    assert_eq!(state.mission_score, 23);
    std::fs::remove_file(destination).unwrap();
}

#[test]
fn v2_migration_preserves_recovered_history_and_rejects_unknown_maps() {
    let (content, mut state) = fixture();
    state.exploration.as_mut().unwrap().event_headers[42] = 17;
    let destination = path("legacy-story");
    save(&destination, &state, &content).unwrap();
    let mut document: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&destination).unwrap()).unwrap();
    document["schema_version"] = 2.into();
    let session = document["session"].as_object_mut().unwrap();
    for field in [
        "menus",
        "flags",
        "mission_score",
        "mission_stage",
        "visited_maps",
        "allies",
        "ally_count",
        "party_mask",
        "selected_warrior",
    ] {
        session.remove(field);
    }
    let position = session["exploration"].as_object_mut().unwrap();
    for field in ["floor", "objects", "events", "visibility"] {
        position.remove(field);
    }
    std::fs::write(&destination, serde_json::to_vec(&document).unwrap()).unwrap();
    let migrated = load(&destination, &content).unwrap();
    assert_eq!(migrated.exploration.as_ref().unwrap().event_headers[42], 17);
    assert_eq!(
        migrated.exploration.as_ref().unwrap().floor,
        content.map(0).unwrap().floor
    );
    assert_eq!(migrated.flags[0], 255);
    document["session"]["exploration"]["map"] = 1.into();
    std::fs::write(&destination, serde_json::to_vec(&document).unwrap()).unwrap();
    assert!(load(&destination, &content).is_err());
    std::fs::remove_file(destination).unwrap();
}

fn registration_script(state: &mut Session) {
    let mut record = vec![0; 42];
    for (index, (opcode, parameter)) in [(47_i16, 31_i16), (9, 505), (98, 0)]
        .into_iter()
        .enumerate()
    {
        record[8 + index * 2..10 + index * 2].copy_from_slice(&opcode.to_be_bytes());
        record[24 + index * 2..26 + index * 2].copy_from_slice(&parameter.to_be_bytes());
    }
    let position = state.exploration.as_mut().unwrap();
    position.x = 5;
    position.y = 50;
    position.last_move = [0, 1];
    position.script = Some(ScriptState {
        record,
        cursor: 0,
        map_slot: 0,
        scheduled: true,
        menu: None,
        persist: false,
        origin: None,
    });
    position.presentation = Some(Presentation::Dialogue {
        id: 0,
        text: String::new(),
    });
}

#[test]
fn registration_opcode_continues_without_external_state() {
    let (mut content, mut state) = fixture();
    content.original_services = None;
    registration_script(&mut state);
    let random = state.random.clone();
    apply(Command::Continue, &mut state, &content).unwrap();
    let position = state.exploration.as_ref().unwrap();
    assert_eq!((position.x, position.y), (5, 50));
    assert!(position.script.is_none());
    assert!(position.presentation.is_none());
    assert_eq!(state.flags[31], 1);
    assert_eq!(state.random, random);
}

#[test]
fn registration_opcode_ignores_legacy_acceptance_receipts() {
    let (mut content, original) = fixture();
    let code_hash = content
        .resources
        .iter()
        .find(|resource| {
            resource.r#type == "CODE"
                && resource.id == 1
                && resource.original.ends_with("/New Centurions")
        })
        .unwrap()
        .sha256
        .clone();
    for accepted in [false, true] {
        content.original_services = Some(centurions_content::OriginalServices {
            schema_version: 1,
            content_identity: content.identity.clone(),
            executable_code_sha256: code_hash.clone(),
            registration_accepted: accepted,
            authority: "Controlled external-service contract test".into(),
        });
        let mut state = original.clone();
        registration_script(&mut state);
        let random = state.random.clone();
        apply(Command::Continue, &mut state, &content).unwrap();
        let position = state.exploration.as_ref().unwrap();
        assert_eq!((position.x, position.y), (5, 50));
        assert!(position.script.is_none());
        assert!(position.presentation.is_none());
        assert_eq!(state.flags[31], 1);
        assert_eq!(state.random, random);
    }
}

#[test]
fn queued_death_child_battle_resumes_parent_without_restoring_roster_or_duplicate_rewards() {
    use centurions_core::{
        Event, MapState,
        combat::{self, death_scripts::DeathEntry, runtime::DeathStory},
    };
    let (content, mut state) = fixture();
    let position = state.exploration.as_mut().unwrap();
    position.restore_map(1, MapState::authored(content.map(1).unwrap()));
    position.x = 77;
    position.y = 9;
    position.presentation = None;
    position.script = Some(ScriptState {
        record: content.map(1).unwrap().script(55).unwrap().to_vec(),
        cursor: 1,
        map_slot: 8,
        scheduled: true,
        menu: None,
        persist: false,
        origin: None,
    });
    combat::begin_battle(&mut state, &content, 14, 0).unwrap();
    // Controlled defeat is the same boundary used by the native nested-caller fixture.
    let arena = state.combat.as_mut().unwrap();
    arena.enemies.records[0].bytes[0x110..0x112].fill(0);
    combat::lifecycle::kill_body(arena, &mut state.warriors, 10).unwrap();
    arena.death_queue.begin();
    let (_, entry) = arena.death_queue.current().unwrap();
    assert_eq!(entry.script, 51);
    state.combat_story = Some(DeathStory {
        body: 0,
        script: 51,
        actor: 0,
        entry: DeathEntry { ..entry },
        ..Default::default()
    });
    let checkpoint = path("nested-death");
    save(&checkpoint, &state, &content).unwrap();
    let mut resumed = load(&checkpoint, &content).unwrap();
    let mut started = vec![14];
    for _ in 0..20 {
        let events = apply(Command::Continue, &mut state, &content).unwrap();
        assert_eq!(
            apply(Command::Continue, &mut resumed, &content).unwrap(),
            events
        );
        assert_eq!(state, resumed);
        for event in events {
            if let Event::CombatStarted { battle, .. } = event {
                started.push(battle);
            }
        }
        if state
            .combat_story
            .as_ref()
            .is_some_and(|story| story.waiting_child)
        {
            break;
        }
    }
    assert_eq!(started, [14, 15]);
    assert_eq!(state.combat.as_ref().unwrap().battle, 15);
    assert_eq!(state.combat.as_ref().unwrap().enemies.records.len(), 1);
    assert_eq!(
        state.combat.as_ref().unwrap().enemies.records[0].bytes[0x103],
        67
    );
    for candidate in [&mut state, &mut resumed] {
        candidate.combat.as_mut().unwrap().enemies.records[0].bytes[0x110..0x112].fill(0);
    }
    let child_events = combat::runtime::confirm_survivors(&mut state, &content).unwrap();
    assert_eq!(
        combat::runtime::confirm_survivors(&mut resumed, &content).unwrap(),
        child_events
    );
    assert_eq!(state, resumed);
    assert!(state.combat.is_none());
    let rewarded = state.warriors.clone();
    let rewarded_random = state.random.clone();
    let events = apply(Command::Continue, &mut state, &content).unwrap();
    assert_eq!(
        apply(Command::Continue, &mut resumed, &content).unwrap(),
        events
    );
    assert_eq!(state, resumed);
    assert!(state.combat_story.is_none());
    assert_eq!(state.warriors, rewarded);
    assert_eq!(state.random, rewarded_random);
    assert_eq!(
        state
            .exploration
            .as_ref()
            .unwrap()
            .script
            .as_ref()
            .unwrap()
            .cursor,
        1
    );
    for _ in 0..20 {
        let events = apply(Command::Continue, &mut state, &content).unwrap();
        assert_eq!(
            apply(Command::Continue, &mut resumed, &content).unwrap(),
            events
        );
        assert_eq!(state, resumed);
        for event in events {
            if let Event::CombatStarted { battle, .. } = event {
                started.push(battle);
            }
        }
        if state.combat.is_some() {
            break;
        }
    }
    assert_eq!(started, [14, 15, 15]);
    assert!(state.combat_story.is_none());
    std::fs::remove_file(checkpoint).unwrap();
}

#[test]
fn unobstructed_battle_return_resumes_authored_outer_script_atomically() {
    use centurions_core::{Event, MapState, combat};
    for seed in [12, 1] {
        let (content, mut state) = fixture();
        state.random = centurions_core::RandomState::new(seed);
        let position = state.exploration.as_mut().unwrap();
        position.restore_map(1, MapState::authored(content.map(1).unwrap()));
        position.x = 77;
        position.y = 9;
        position.presentation = None;
        position.script = Some(ScriptState {
            record: content.map(1).unwrap().script(55).unwrap().to_vec(),
            cursor: 7,
            map_slot: 8,
            scheduled: true,
            menu: None,
            persist: false,
            origin: None,
        });
        combat::begin_battle(&mut state, &content, 15, 0).unwrap();
        for warrior in &mut state.warriors {
            warrior.bytes[0x9ce..0x9d2].copy_from_slice(&1_000_000_i32.to_be_bytes());
        }
        for monster in &mut state.combat.as_mut().unwrap().enemies.records {
            monster.bytes[0x110..0x112].fill(0);
        }
        state.exploration.as_mut().unwrap().presentation = Some(Presentation::Survivors);
        let mut unsupported = state.clone();
        unsupported
            .exploration
            .as_mut()
            .unwrap()
            .script
            .as_mut()
            .unwrap()
            .record[22..24]
            .copy_from_slice(&300_i16.to_be_bytes());
        let rejected = unsupported.clone();
        assert!(apply(Command::Continue, &mut unsupported, &content).is_err());
        assert_eq!(unsupported, rejected);
        let destination = path(&format!("unobstructed-battle-{seed}"));
        save(&destination, &state, &content).unwrap();
        let mut resumed = load(&destination, &content).unwrap();
        let events = apply(Command::Continue, &mut state, &content).unwrap();
        assert_eq!(
            apply(Command::Continue, &mut resumed, &content).unwrap(),
            events
        );
        assert_eq!(state, resumed);
        assert!(
            events
                .iter()
                .any(|event| matches!(event, Event::BattleEnded { victory: true }))
        );
        assert!(state.combat.is_none());
        assert_ne!(
            state
                .exploration
                .as_ref()
                .unwrap()
                .script
                .as_ref()
                .unwrap()
                .record,
            content.map(1).unwrap().script(55).unwrap()
        );
        assert!(state.exploration.as_ref().unwrap().presentation.is_some());
        save(&destination, &state, &content).unwrap();
        assert_eq!(load(&destination, &content).unwrap(), state);
        std::fs::remove_file(destination).unwrap();
    }
}

#[test]
fn explicit_promotion_choice_resumes_and_applies_library_random_cost_once() {
    let (content, mut state) = fixture();
    let member = state
        .warriors
        .iter()
        .position(|w| centurions_content::word(&w.bytes, 0x36).unwrap() != 0)
        .unwrap();
    state.warriors[member].bytes[0x9ce..0x9d2].copy_from_slice(&(-1_i32).to_be_bytes());
    let position = state.exploration.as_mut().unwrap();
    position.script = Some(ScriptState {
        record: vec![0; 42],
        cursor: 8,
        map_slot: 0,
        scheduled: true,
        menu: None,
        persist: false,
        origin: None,
    });
    position.presentation = Some(Presentation::Loot);
    let before = state.clone();
    assert!(apply(Command::Continue, &mut state, &content).is_err());
    assert_eq!(state, before);
    let checkpoint = path("promotion-choice");
    save(&checkpoint, &state, &content).unwrap();
    let mut resumed = load(&checkpoint, &content).unwrap();
    let events = apply(Command::KeepCurrentSpells, &mut state, &content).unwrap();
    assert_eq!(
        apply(Command::KeepCurrentSpells, &mut resumed, &content).unwrap(),
        events
    );
    assert_eq!(resumed, state);
    assert_eq!(state.random, before.random);
    assert_eq!(state.library_random_draws, before.library_random_draws + 1);
    assert_eq!(
        centurions_content::word(&state.warriors[member].bytes, 0x3a).unwrap(),
        centurions_content::word(&before.warriors[member].bytes, 0x3a).unwrap() + 1
    );
    assert!(events.iter().any(|event| matches!(event, centurions_core::Event::LevelAdvanced { warrior, .. } if *warrior == member)));
    let closed = state.clone();
    assert!(apply(Command::KeepCurrentSpells, &mut state, &content).is_err());
    assert_eq!(state, closed);
    std::fs::remove_file(checkpoint).unwrap();
}

#[test]
fn promotion_passes_the_former_registration_level_limit_without_a_receipt() {
    let (mut content, mut state) = fixture();
    content.original_services = None;
    state.warriors[0].bytes[0x3a..0x3c].copy_from_slice(&7_i16.to_be_bytes());
    state.warriors[0].bytes[0x9ce..0x9d2].copy_from_slice(&(-1_i32).to_be_bytes());
    let position = state.exploration.as_mut().unwrap();
    position.script = None;
    position.presentation = Some(Presentation::Loot);
    let draws = state.library_random_draws;
    let events = apply(Command::Continue, &mut state, &content).unwrap();
    assert_eq!(
        centurions_content::word(&state.warriors[0].bytes, 0x3a).unwrap(),
        8
    );
    assert_eq!(state.library_random_draws, draws + 1);
    assert!(events.iter().any(|event| matches!(
        event,
        centurions_core::Event::LevelAdvanced {
            warrior: 0,
            level: 8
        }
    )));
    assert!(!events.iter().any(|event| matches!(
        event,
        centurions_core::Event::CollectionRejected { warning: 40 | 452 }
    )));
}

#[test]
fn authored_npc_battle_entry_serializes_the_original_parent_frame() {
    for seed in [12, 1] {
        let (content, mut state) = fixture();
        state.random = centurions_core::RandomState::new(seed);
        state.mission_stage = 3;
        let position = state.exploration.as_mut().unwrap();
        let map = content.map(4).unwrap();
        position.restore_map(4, centurions_core::MapState::authored(map));
        position.x = 45;
        position.y = 3;
        let header = map.header[16 * 42..17 * 42].to_vec();
        position.script = Some(ScriptState {
            record: header.clone(),
            cursor: 1,
            map_slot: 16,
            scheduled: false,
            menu: None,
            persist: true,
            origin: Some(centurions_core::ScriptOrigin {
                map: 4,
                x: 45,
                y: 3,
            }),
        });
        position.presentation = Some(Presentation::Dialogue {
            id: -831,
            text: map.dialogue(-831).unwrap(),
        });
        apply(Command::Continue, &mut state, &content).unwrap();
        assert!(matches!(
            state.exploration.as_ref().unwrap().presentation,
            Some(Presentation::Question { extra: 0, .. })
        ));
        let destination = path(&format!("npc-entry-{seed}"));
        save(&destination, &state, &content).unwrap();
        let mut resumed = load(&destination, &content).unwrap();
        let events = apply(Command::Choose { answer: true }, &mut state, &content).unwrap();
        assert_eq!(
            apply(Command::Choose { answer: true }, &mut resumed, &content).unwrap(),
            events
        );
        assert_eq!(state, resumed);
        assert_eq!(state.combat.as_ref().unwrap().battle, 117);
        assert_eq!(state.combat.as_ref().unwrap().enemies.records.len(), 18);
        let frame = state.battle_npc.as_ref().unwrap();
        assert_eq!(frame.frame.actions.as_slice(), &header[8..40]);
        assert_eq!(frame.frame.action_index, 3);
        assert_eq!(frame.escape_script, -1);
        save(&destination, &state, &content).unwrap();
        assert_eq!(load(&destination, &content).unwrap(), state);
        std::fs::remove_file(destination).unwrap();
    }
}
