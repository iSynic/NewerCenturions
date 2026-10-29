use crate::{native_common, prelude::*, snapshots, support};
use centurions_core::combat::target_work::TargetWork;

use support::buffer as bytes;

#[test]
fn authored_healer_turn_returns_through_command_and_saved_suffix() {
    let content = content();
    let receipt = support::fixture("npc-healing-caller.json");
    let original: Session = serde_json::from_value(receipt["input"].clone()).unwrap();
    let mut random = RandomState::new(12);
    let mut mask = vec![0; 110];
    mask[11] = 1;
    let packet = centurions_core::combat::spells::load_packet(
        &content.map(0).unwrap().combat.as_ref().unwrap().attacks,
        1101,
    )
    .unwrap();
    let result = centurions_core::combat::spells::resolve_support_for_caster(
        &original.warriors,
        &current_combat(&original).enemies.records,
        &mask,
        1101,
        &centurions_core::combat::spells::DirectSpell {
            packet: &packet,
            level: 0,
            count: 2,
            armor_filter: false,
            weapon_sound: 0,
        },
        (
            current_combat(&original).active_body(),
            Some(
                centurions_core::combat::spells::RestrictionMemoryView::ordinary_startup(
                    &content.map(0).unwrap().item_cache,
                    None,
                ),
            ),
        ),
        &mut random,
    )
    .unwrap();
    assert_eq!(
        support::record_bytes(&result.warriors),
        receipt.bytes("party_after")
    );
    assert_eq!(
        support::record_bytes(&result.enemies),
        receipt.bytes("npcs_after")
    );
    support::assert_rng(&random, &receipt["seed_after"], &receipt["rng_draws"]);
    let native_sounds = receipt.n::<Vec<i16>>("sounds_before_showresults");
    let sounds: Vec<_> = result
        .targets
        .iter()
        .flat_map(|(_, target)| &target.sounds)
        .copied()
        .collect();
    assert_eq!(&sounds[..native_sounds.len()], native_sounds);
    // The receipt bounds only healing resolution; this full AI caller is source-composed.
    let mut state = original;
    // Source-composed full commands select NPC0 after the Getup zero writers.
    native_common::npc_command_input(&mut state, 0, [0, 0]);
    assert_eq!(current_combat(&state).battle, 141);
    let mut resumed = support::checkpoint(&state, &content, "healer-before");
    let events = support::replay_command(Command::RunEnemyTurn, &mut state, &mut resumed, &content);
    assert!(
        events.iter().any(
            |event| matches!(event, Event::SpellResolved { spell: 1101, damage, .. } if *damage < 0)
        ),
        "{events:?}"
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::TurnStarted { .. })),
        "{events:?}"
    );
    support::checkpoint(&state, &content, "healer-after");
    let receipt = support::fixture("npc-natural-spell-audio.json");
    for case in receipt.array("cases") {
        let mut state: Session = serde_json::from_value(receipt["input"].clone()).unwrap();
        state.native_work.weapon_work.natural_attack_index = case.n("index");
        state
            .combat
            .as_mut()
            .unwrap()
            .weapon_work
            .natural_attack_index = case.n("index");
        native_common::npc_command_input(&mut state, 0, [0, 0]);
        let events = support::saved_command(Command::RunEnemyTurn, &mut state, &content);
        let expected = [
            Event::Sound { resource_id: 108 },
            Event::Sound {
                resource_id: case["sounds"][0].as_i64().unwrap() as i16,
            },
        ];
        assert!(
            events.windows(2).any(|pair| pair == expected),
            "native index{}: {events:?}",
            case["index"]
        );
        support::checkpoint(&state, &content, "natural-audio-after");
    }
    let mut state: Session = serde_json::from_value(receipt["input"].clone()).unwrap();
    state.native_work.weapon_work.natural_attack_index = 5;
    let combat = state.combat.as_mut().unwrap();
    combat.map = centurions_core::combat::footprint::restore_body(combat, 10).unwrap();
    let point = (combat.squad.positions[0].0 - 1, combat.squad.positions[0].1);
    let placed = centurions_core::combat::footprint::place(
        &combat.map,
        &combat.enemies.records[0],
        point,
        10,
    )
    .unwrap();
    combat.map = placed.map;
    combat.enemies.positions[0] = point;
    combat.enemies.records[0] = placed.monster;
    // The controlled loop uses this relocated slot as its selected NPC.
    combat.selected_enemy = 0;
    combat.weapon_work.natural_attack_index = 5;
    let actor = &mut combat.enemies.records[0].bytes;
    actor[0x109] = 6;
    actor[0x106] = 30;
    actor[0x52..0x54].fill(0);
    actor[0x10e..0x110].fill(0);
    native_common::npc_command_input(&mut state, 0, [0, 0]);
    support::saved_command(Command::RunEnemyTurn, &mut state, &content);
    let native = &receipt["loop_cases"][0];
    let combat = current_combat(&state);
    assert_eq!(
        combat.weapon_work.natural_attack_index,
        native.n::<i16>("index_after")
    );
    assert_eq!(
        combat.enemies.records[0].record().action_points(),
        native.n::<i8>("ap_after")
    );
    assert_eq!(
        combat.enemies.records[0].record().movement_points(),
        native.n::<i8>("movement_after")
    );
    support::checkpoint(&state, &content, "natural-writer-after");
}

fn normalized_positions(f: &Value, snapshot: &Value, key: &str) -> Vec<(i16, i16)> {
    snapshots::shifted_positions(
        &|key| bytes(f, &snapshot[key]),
        key,
        snapshots::viewport_delta(snapshot, &f["cases"][0]["before"]),
    )
}

// The receipt retains all110 target WORDs, including the separately modeled104-word tail.
fn project_mask(state: &mut Session, f: &Value, snapshot: &Value) {
    let mask = native_common::buffer_words(f, &snapshot["mask"]);
    state.native_work.party_mask.copy_from_slice(&mask[..6]);
    match state.combat.as_mut() {
        Some(combat) => combat.target_tail = mask[6..].to_vec(),
        None => state.native_work.spell_target_tail = mask[6..].to_vec(),
    }
}

fn project_native(state: &mut Session, f: &Value, snapshot: &Value) {
    snapshots::npc_projection(state, f, snapshot, &f["cases"][0]["before"]);
    project_mask(state, f, snapshot);
    state.native_work.weapon_work = snapshots::weapon_work(&|key| bytes(f, &snapshot[key]));
    state.combat.as_mut().unwrap().weapon_work = state.native_work.weapon_work;
}

fn initial(f: &Value, content: &Content) -> Session {
    let mut state: Session = serde_json::from_value(f["foundation"]["session"].clone()).unwrap();
    // The qualified before receipt records447196=11; the declared A5 inputs are zero.
    native_common::npc_command_input(&mut state, f["cases"][0]["before"].n("npc_index"), [0, 0]);
    project_mask(&mut state, f, &f["cases"][0]["before"]);
    state.validate(content).unwrap();
    let mut projected = state.clone();
    project_native(&mut projected, f, &f["cases"][0]["before"]);
    assert_eq!(projected, state, "native input projection");
    assert_eq!(current_combat(&state).active_body(), Some(21));
    assert_eq!(current_combat(&state).enemies.records.len(), 14);
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
                icon: Default::default(),
                flight: Default::default(),
            },
        ]);
    }
    events.extend([
        native_common::guard_result(21),
        Event::Sound { resource_id: 138 },
        Event::TurnStarted { body: 3, round: 1 },
    ]);
    events
}

#[test]
fn actual_npc_poison_actor_two_casts_and_getup_match_complete_native_state_and_resume() {
    let content = content();
    let f = support::fixture("qualified-npc-poison-caller.json");

    let case = &f["cases"][0];
    let before = initial(&f, &content);
    let native_after = &case["after"];
    assert_eq!(native_after["shift_x"], 45);
    assert_eq!(native_after["shift_y"], 7);
    // Original centering changes every position and its origin together. The
    // command model retains its logical coordinate frame for the same map cells.
    assert_eq!(
        normalized_positions(&f, native_after, "party_positions"),
        current_combat(&before).squad.positions
    );
    assert_eq!(
        normalized_positions(&f, native_after, "npc_positions")[..14],
        current_combat(&before).enemies.positions
    );
    let mut expected = before.clone();
    project_native(&mut expected, &f, native_after);
    let events = expected_events();
    let sounds: Vec<_> = support::event_sounds(&events);
    assert_eq!(serde_json::to_value(sounds).unwrap(), case["sounds"]);
    let mut receipt = before.random.clone();

    support::replay_rng(&mut receipt, case.array("rng"));
    assert_eq!(receipt, expected.random);
    // The captured suffix executes getup125bda; Ready itself is not in the receipt.
    // Compose only these newly represented fields from that explicit original writer.
    expected.native_work.ready = 0;
    // Getup125be6 also writes the selector WORD, absent from this older receipt.
    expected.native_work.walking_confusion = 0;
    expected.combat.as_mut().unwrap().ready = 0;
    // The same captured Getup calls emptyque even with an empty queue; BYTE584 is omitted.
    expected.native_work.death_processing.queue = 0;
    // The captured result paths clear the new WORD; the command retains it.
    expected.combat.as_mut().unwrap().neighbor_work.departure = 0;
    expected.native_work.neighbor_work.departure = 0;
    // The receipt omits the target WORDs: one decision triple, which cast's loop index
    // passes, then getup125a50's cleartarget.
    expected.combat.as_mut().unwrap().target_work = TargetWork {
        cell: (-1, -1),
        count: 1,
        index: 1,
    };
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
    support::same_session(&actual, &expected);
    assert_eq!(
        apply(Command::RunEnemyTurn, &mut resumed, &content).unwrap(),
        events
    );
    assert_eq!(resumed, expected);
    save(&path, &actual, &content).unwrap();
    assert_eq!(load(&path, &content).unwrap(), expected);
    std::fs::remove_file(path).unwrap();
}
