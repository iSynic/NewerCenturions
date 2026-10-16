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
    let mut events = vec![];
    for target in [15, 4] {
        events.extend([
            Event::Sound { resource_id: 431 },
            Event::Sound { resource_id: 430 },
            // Source151a88: successful condition result, before the next cast.
            Event::Feedback(centurions_core::feedback::NativeFeedback::CombatResult {
                body: target,
                code: 4,
                amount: 0,
                audible: false,
            }),
            Event::SpellResolved {
                caster: Some(21),
                spell: 2206,
                target,
                damage: 0,
                defeated: false,
            },
        ]);
    }
    events.extend([
        Event::Sound { resource_id: 138 },
        Event::TurnStarted { body: 3, round: 1 },
    ]);
    events
}

#[test]
fn actual_npc_poison_actor_two_casts_and_getup_match_complete_native_state_and_resume() {
    let content = content();
    let f = crate::support::fixture("qualified-npc-poison-caller.json");

    let case = &f["cases"][0];
    let before = initial(&f, &content);
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
    let mut expected = before.clone();
    project_native(&mut expected, &f, native_after);
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
