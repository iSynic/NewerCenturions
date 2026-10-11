use centurions_content::{Content, word};
use centurions_core::{
    Command, Event, Presentation, RandomState, ScriptState, Session, apply, load, save,
    story_health::HealthContext,
};
use serde_json::Value;
use std::path::Path;

fn content() -> Content {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap()
}

fn decode(value: &Value) -> Vec<u8> {
    let hex = value.as_str().unwrap();
    (0..hex.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&hex[offset..offset + 2], 16).unwrap())
        .collect()
}

fn put(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

fn definition(content: &Content, id: usize) -> Vec<u8> {
    let mut record = content.items[id].clone();
    assert_eq!(record.len(), 150);
    // loaditem130afe changes only decimal WORD26 after copying all 150 bytes.
    let class = word(&record, 26).unwrap().wrapping_abs();
    put(&mut record, 26, class);
    record
}

fn project_party(state: &mut Session, native: &Value) {
    let records = decode(native);
    assert_eq!(records.len(), 6 * 3138);
    for (warrior, record) in state.warriors.iter_mut().zip(records.as_chunks::<3138>().0) {
        warrior.bytes = record.to_vec();
    }
}

fn initial(content: &Content, native: &Value) -> Session {
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
    project_party(&mut state, native);
    let position = state.exploration.as_mut().unwrap();
    position.script = None;
    position.presentation = None;
    state.movement_continuation = None;
    state.random = RandomState::new(12);
    state.ammunition_work.item = Some(definition(content, 165));
    state.loaded_item_field32 =
        Some(word(state.ammunition_work.item.as_ref().unwrap(), 32).unwrap());
    state.ammunition_work.weapon = Some(definition(content, 2));
    state.ammunition_work.ammo = Some(definition(content, 201));
    state.health_context = Some(HealthContext {
        unavailable: 0,
        selected_warrior: 0,
        effect: 0,
        last_body: 0,
        marks_affected: false,
        affected: [0; 6],
    });
    state.validate(content).unwrap();
    state
}

fn expect_item(state: &mut Session, content: &Content, id: usize) {
    let item = definition(content, id);
    state.loaded_item_field32 = Some(word(&item, 32).unwrap());
    state.ammunition_work.item = Some(item);
}

fn check_command(
    label: &str,
    content: &Content,
    input: &Session,
    expected: &Session,
    command: Command,
    events: &[Event],
) {
    input.validate(content).unwrap();
    expected.validate(content).unwrap();
    assert_eq!(
        expected.ammunition_work.weapon,
        input.ammunition_work.weapon
    );
    assert_eq!(expected.ammunition_work.ammo, input.ammunition_work.ammo);
    let destination = std::env::temp_dir().join(format!(
        "centurions-noncombat-item-{label}-{}.json",
        std::process::id()
    ));
    save(&destination, input, content).unwrap();
    let mut resumed = load(&destination, content).unwrap();
    assert_eq!(resumed, *input);
    let mut actual = input.clone();
    assert_eq!(
        apply(command.clone(), &mut actual, content).unwrap(),
        events
    );
    assert_eq!(actual, *expected, "{label} complete state");
    assert_eq!(apply(command, &mut resumed, content).unwrap(), events);
    assert_eq!(resumed, *expected);
    save(&destination, expected, content).unwrap();
    assert_eq!(load(&destination, content).unwrap(), *expected);
    std::fs::remove_file(destination).unwrap();
}

fn story_input(content: &mut Content, case: &Value, maximum: Option<i16>) -> Session {
    let mut extra = decode(&case["extra"]);
    if let Some(maximum) = maximum {
        put(&mut extra, 2, maximum);
    }
    let tables = std::sync::Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
    tables.combat.as_mut().unwrap().encounters[..10].copy_from_slice(&extra);
    let mut state = initial(content, &case["before"]);
    let mut record = vec![0; 42];
    put(&mut record, 8, 22);
    state.exploration.as_mut().unwrap().script = Some(ScriptState {
        record,
        cursor: 0,
        map_slot: 0,
        scheduled: true,
        menu: None,
        persist: false,
        origin: None,
    });
    state.exploration.as_mut().unwrap().presentation = Some(Presentation::Dialogue {
        id: 1,
        text: content.map(0).unwrap().dialogue(1).unwrap(),
    });
    state
}

#[test]
fn opcode22_loads_only_the_definition_of_an_actual_native_removal() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/story-inventory.json"
    ))
    .unwrap();
    let original_content = content();
    // The frozen operation supplies party bytes; the scheduled wrapper has no later writer.
    for (index, maximum) in [(0, None), (4, None), (3, None), (0, Some(0)), (0, Some(-1))] {
        let case = &fixture["consume_cases"][index];
        assert_eq!(case["rng_draws"], 0);
        let mut content = original_content.clone();
        let input = story_input(&mut content, case, maximum);
        let mut expected = input.clone();
        expected.exploration.as_mut().unwrap().script = None;
        expected.exploration.as_mut().unwrap().presentation = None;
        let events = if maximum.is_some() {
            vec![]
        } else {
            project_party(&mut expected, &case["after"]);
            if case["consumed"].as_u64().unwrap() != 0 {
                expect_item(
                    &mut expected,
                    &content,
                    word(&decode(&case["extra"]), 0).unwrap().unsigned_abs() as usize,
                );
            }
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|sound| Event::Sound {
                    resource_id: sound.as_i64().unwrap() as i16,
                })
                .collect()
        };
        check_command(
            &format!("consume-{index}-{maximum:?}"),
            &content,
            &input,
            &expected,
            Command::Continue,
            &events,
        );
    }
}

#[test]
fn class23_use_establishes_the_complete_frozen_selected_definition() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/item-use.json")).unwrap();
    // Native variant 10 controls WORD50=32767 and returns without a scenario script.
    let case = &fixture["cases"][10];
    assert_eq!(case["script"], -1);
    assert_eq!(case["draws"], 0);
    let mut content = content();
    content.items[832] = decode(&case["definition"]);
    let input = initial(&content, &case["before"]);
    let mut expected = input.clone();
    project_party(&mut expected, &case["after"]);
    expect_item(&mut expected, &content, 832);
    let events = case["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|warning| Event::CollectionRejected {
            warning: warning.as_i64().unwrap() as i16,
        })
        .chain(
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|sound| Event::Sound {
                    resource_id: sound.as_i64().unwrap() as i16,
                }),
        )
        .collect::<Vec<_>>();
    check_command(
        "use-832",
        &content,
        &input,
        &expected,
        Command::UseItem {
            warrior: 0,
            slot: 3,
        },
        &events,
    );
}

fn noon_input(content: &mut Content, case: &Value) -> Session {
    let map = &mut content.scenario.as_mut().unwrap().maps[0];
    map.minutes_per_tick = 5;
    map.tile_properties[..10].fill(0);
    put(&mut map.tile_properties, 2, 1);
    std::sync::Arc::make_mut(&mut map.shared).schedule = decode(&case["schedule_before"]);
    let mut state = initial(content, &case["input"]);
    let position = state.exploration.as_mut().unwrap();
    position.x = 2;
    position.y = 2;
    position.day = 1;
    position.hour = 11;
    position.minute = 55;
    position.fatigue = 4;
    position.floor.fill(0);
    position.objects.fill(0);
    position.events.fill(0);
    position.region_state.fill(0);
    state.random = RandomState::new(1);
    state
}

fn noon_expected(input: &Session, case: &Value) -> Session {
    let mut expected = input.clone();
    project_party(&mut expected, &case["output"]);
    let position = expected.exploration.as_mut().unwrap();
    position.x = 3;
    position.y = 2;
    position.facing = 1;
    position.last_move = [1, 0];
    position.hour = 12;
    position.minute = 0;
    position.fatigue = 5;
    position.schedule = decode(&case["schedule_after"]);
    expected
}

#[test]
fn noon_food_loads_authored800_and_nonpositive_or_absent_food_retains_work() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/calendar.json")).unwrap();
    let case = &fixture["cases"][0];
    let mut content = content();
    assert_eq!(content.items[800], decode(&fixture["food_definition"]));
    assert_eq!(case["draws"], 0);
    let original = noon_input(&mut content, case);
    let events = [
        Event::Sound { resource_id: 0 },
        Event::Moved { x: 3, y: 2 },
        Event::Sound { resource_id: 0 },
        Event::TimeAdvanced {
            day: 1,
            hour: 12,
            minute: 0,
        },
    ];
    let mut expected = noon_expected(&original, case);
    expect_item(&mut expected, &content, 800);
    check_command(
        "food",
        &content,
        &original,
        &expected,
        Command::Move { dx: 1, dy: 0 },
        &events,
    );

    // These controls change only the search inputs. Source 1126e2 is unreachable;
    // hourly native changes remain, with no food weight/quantity costs and half recovery.
    for absent in [false, true] {
        let mut input = original.clone();
        if absent {
            for warrior in &mut input.warriors {
                put(&mut warrior.bytes, 0x132 + 2 * 22, 801);
            }
        } else {
            put(&mut input.warriors[0].bytes, 0x134 + 2 * 22, 0);
        }
        let mut expected = noon_expected(&input, case);
        for (warrior, before) in expected.warriors.iter_mut().zip(&input.warriors) {
            warrior.bytes[0x132 + 2 * 22..0x132 + 3 * 22]
                .copy_from_slice(&before.bytes[0x132 + 2 * 22..0x132 + 3 * 22]);
            warrior.bytes[0x9d6..0x9d8].copy_from_slice(&before.bytes[0x9d6..0x9d8]);
        }
        put(&mut expected.warriors[3].bytes, 0x50, 25);
        check_command(
            if absent { "food-absent" } else { "food-zero" },
            &content,
            &input,
            &expected,
            Command::Move { dx: 1, dy: 0 },
            &events,
        );
    }
}

#[test]
fn later_use_failure_rolls_back_inventory_item_and_all_other_work() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/item-use.json")).unwrap();
    let case = &fixture["cases"][0];
    let mut content = content();
    assert_eq!(content.items[832], decode(&case["definition"]));
    let input = initial(&content, &case["before"]);
    let tables = std::sync::Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
    // The native item cost and ITEM writer precede this unsupported script opcode.
    put(&mut tables.door_scripts, 216 * 42 + 8, 200);
    let mut actual = input.clone();
    assert!(matches!(
        apply(
            Command::UseItem {
                warrior: 0,
                slot: 3
            },
            &mut actual,
            &content
        ),
        Err(centurions_core::RuleError::Unsupported {
            action: "story.opcode",
            ..
        })
    ));
    assert_eq!(actual, input);
}
