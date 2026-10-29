use crate::{native_common, prelude::*, snapshots, support};

use support::buffer as bytes;

use native_common::buffer_words as words;

fn positions(f: &Value, snapshot: &Value, key: &str) -> Vec<(i16, i16)> {
    snapshots::shifted_positions(
        &|key| bytes(f, &snapshot[key]),
        key,
        snapshots::viewport_delta(snapshot, &f["earned_native_caller"]["before"]),
    )
}

fn project(state: &mut Session, f: &Value, snapshot: &Value) {
    assert_eq!(bytes(f, &snapshot["party"]).len(), 18_828);
    snapshots::npc_projection(state, f, snapshot, &f["earned_native_caller"]["before"]);
    state.combat.as_mut().unwrap().round_scripts.counter = snapshot.n("round_script_counter");
    let exploration = state.exploration.as_mut().unwrap();
    exploration.event_headers = bytes(f, &snapshot["event_headers"]);
    exploration.region_state = bytes(f, &snapshot["active_region"]);
}

fn initial(f: &Value, case: &Value, content: &Content) -> Session {
    let mut state: Session = serde_json::from_value(f["foundation"]["session"].clone()).unwrap();
    project(&mut state, f, &f["earned_native_caller"]["after"]);
    project(&mut state, f, &case["before"]);
    state.validate(content).unwrap();
    assert_eq!(current_combat(&state).active_body(), Some(3));
    assert_eq!(state.selected_warrior, 3);
    assert_eq!(current_combat(&state).map.shift, (0, -36));
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
            let health = word(&npcs[(body as usize - 10) * MONSTER_BYTES..], 0x110).unwrap();
            // Typed outcomes are synthesized at resolution; the slice captures
            // native damage, target records and ordered audio rather than Events.
            events.push(Event::SpellResolved {
                caster: None,
                spell,
                target: body,
                damage: damage.n::<i16>("d0_return_word"),
                defeated: health < 1,
                icon: Default::default(),
                flight: Default::default(),
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
        &mut state.native_work.attack_count,
        &mut state.native_work.party_mask,
        (state.selected_warrior, &mut state.bandage_ready),
    )?;
    // Project the typed slice outcome through the same Session bookkeeping
    // represented by the independently captured native globals.
    if std::mem::take(&mut combat.monster_work_written) {
        state.native_work.monster_work = Default::default();
    }
    state.native_work.attack_packet = combat.attack_packet.clone();
    combat.pending_last_defeated = None;
    // The capture does not record Ready; a collision leaves spelltargets' 0.
    combat.ready = 0;
    state.native_work.attack_identity = combat.attack_identity;
    let health = &mut state.native_work.health_context;
    health.effect = combat.attack_packet[18] as i8;
    if let Some(body) = outcome.last_defeated_body {
        health.last_body = body;
    }
    if outcome
        .events
        .iter()
        .any(|event| matches!(event, Event::SpellResolved { damage, .. } if *damage > 0))
    {
        state.bandage_ready = 0;
    }
    Ok(outcome.events)
}

#[test]
fn npc_poison_slice_matches_complete_native_records_fields_work_and_rng_for_21_cases() {
    let content = support::content();
    let f: Value = support::fixture("npc-poison-consumer.json");
    let cases = f.array("cases");

    for (index, case) in cases.iter().enumerate() {
        let before = initial(&f, case, &content);
        let mut expected = before.clone();
        project(&mut expected, &f, &case["after"]);
        // The recorded killbody/showresults path clears the newly represented WORD.
        expected.combat.as_mut().unwrap().neighbor_work.departure = 0;
        // The capture omits target WORDs 6..109; collide leaves its own marks there.
        let tail = |s: &Session| s.combat.as_ref().unwrap().target_tail.clone();
        let mut actual = before.clone();
        assert_eq!(
            consume(&mut actual, &content).unwrap(),
            expected_events(&f, case),
            "events case {index}"
        );
        expected.combat.as_mut().unwrap().target_tail = tail(&actual);
        // Nor target 0, where collide stores the field centre (1142e0..1142fe).
        expected.combat.as_mut().unwrap().target_work = actual.combat.as_ref().unwrap().target_work;
        // The capture omits the victory WORDs; killbody's count is not compared.
        expected.combat.as_mut().unwrap().victory_work = actual.combat.as_ref().unwrap().victory_work;
        assert_eq!(actual, expected, "complete Session projection case {index}");
        let mut receipt = before.random.clone();
        support::replay_rng(&mut receipt, case.array("rng"));
        assert_eq!(receipt, expected.random, "RNG case {index}");
        let combat = current_combat(&actual);
        assert!(word(&combat.enemies.records[10].bytes, 0x110).unwrap() < 1);
        assert!(!combat.order.contains(&20));
        assert_eq!(
            actual
                .native_work
                .health_context
                .last_body,
            20
        );

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
