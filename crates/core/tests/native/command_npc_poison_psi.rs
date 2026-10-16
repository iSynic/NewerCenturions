use crate::prelude::*;

use crate::support::buffer as bytes;

fn normalized_positions(f: &Value, snapshot: &Value, key: &str) -> Vec<(i16, i16)> {
    crate::snapshots::shifted_positions(
        &|key| bytes(f, &snapshot[key]),
        key,
        crate::snapshots::viewport_delta(snapshot, &f["cases"][0]["before"]),
    )
}

fn project_native(state: &mut Session, f: &Value, snapshot: &Value) {
    crate::snapshots::npc_projection(state, f, snapshot, &f["cases"][0]["before"]);
    state.weapon_work = crate::snapshots::weapon_work(&|key| bytes(f, &snapshot[key]));
    state.combat.as_mut().unwrap().weapon_work = state.weapon_work;
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

#[test]
fn actual_npc_poison_actor_two_casts_and_getup_match_complete_native_state_and_resume() {
    let content = content();
    let f = crate::support::fixture("qualified-npc-poison-caller.json");

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
    let sounds: Vec<_> = crate::support::event_sounds(&events);
    assert_eq!(serde_json::to_value(sounds).unwrap(), case["sounds"]);
    let mut receipt = before.random.clone();

    crate::support::replay_rng(&mut receipt, case.array("rng"));
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
