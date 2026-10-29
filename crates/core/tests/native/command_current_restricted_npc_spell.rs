use crate::{native_common, prelude::*, snapshots, support};

use support::buffer as bytes;

use centurions_core::combat::spells::RestrictionMemoryView;

use native_common::buffer_words as words;
use native_common::snapshot_word as number;

use native_common::foundation_session as initial;

fn projected(before: &Session, fixture: &Value, snapshot: &Value, baseline: &Value) -> Session {
    let mut expected = before.clone();
    let read = |key: &str| bytes(fixture, &snapshot[key]);
    snapshots::session(
        &mut expected,
        snapshot,
        &read,
        number(snapshot, "selected_member") as usize,
    );
    snapshots::carried_items(&mut expected, &read);
    if snapshot["draws"] != baseline["draws"] {
        expected.native_work.ammunition_work.item = Some(bytes(fixture, &snapshot["item_work"]));
    }
    expected.psi_disabled = number(snapshot, "psi_disabled") as u8;
    expected.enemy_spell_disabled = number(snapshot, "enemy_spell_disabled") as u8;
    let combat = expected.combat.as_mut().unwrap();
    let previous = (combat.cursor, combat.round);
    snapshots::records(combat, &|key| bytes(fixture, &snapshot[key]));
    // Preserve the command's viewport while comparing original absolute rule coordinates.
    let delta = snapshots::viewport_delta(snapshot, baseline);
    snapshots::geometry(combat, &|key| bytes(fixture, &snapshot[key]), delta);
    combat.cursor = number(snapshot, "cursor") as usize;
    combat.round = current_combat(before).round.wrapping_add(
        number(snapshot, "round_script_counter")
            .wrapping_sub(number(baseline, "round_script_counter")),
    );
    snapshots::turn_origin(combat, previous);
    combat.round_scripts.counter = number(snapshot, "round_script_counter") as i8;
    combat.round_scripts.slots = words(fixture, &snapshot["round_slots"]).try_into().unwrap();
    snapshots::spell_work(combat, &|key| bytes(fixture, &snapshot[key]));
    combat.unavailable = number(snapshot, "unavailable") as i8;

    combat.weapon_work = snapshots::weapon_work(&read);
    expected.native_work.attack_packet = combat.attack_packet.clone();
    expected.native_work.attack_identity = combat.attack_identity;
    expected.native_work.weapon_work = combat.weapon_work;
    snapshots::health(&mut expected, snapshot, &read);
    snapshots::actor_work(&mut expected, snapshot);
    expected
}
use support::checkpoint;
use support::same_session as same;

fn full_native_slots(state: &Session, fixture: &Value, snapshot: &Value, baseline: &Value) {
    let combat = current_combat(state);
    let mut records = bytes(fixture, &baseline["all_records"]);
    for (index, record) in combat.enemies.records.iter().enumerate() {
        records[index * MONSTER_BYTES..(index + 1) * MONSTER_BYTES].copy_from_slice(&record.bytes);
    }
    assert_eq!(records, bytes(fixture, &snapshot["all_records"]));
    let mut positions = words(fixture, &baseline["all_monster_positions"]);
    for (index, point) in combat.enemies.positions.iter().enumerate() {
        positions[index * 2..index * 2 + 2].copy_from_slice(&[point.0, point.1]);
    }
    let delta = (
        number(snapshot, "shift_x").wrapping_sub(number(baseline, "shift_x")),
        number(snapshot, "shift_y").wrapping_sub(number(baseline, "shift_y")),
    );
    let normalized: Vec<_> = words(fixture, &snapshot["all_monster_positions"])
        .iter()
        .enumerate()
        .map(|(index, v)| v.wrapping_add(if index % 2 == 0 { delta.0 } else { delta.1 }))
        .collect();
    assert_eq!(positions, normalized);
    assert_eq!(
        combat
            .death_queue
            .entries
            .iter()
            .flat_map(|entry| entry.bytes())
            .collect::<Vec<_>>(),
        bytes(fixture, &snapshot["death_queue"])
    );
    assert_eq!(
        combat.death_queue.count,
        number(snapshot, "death_queue_count") as usize
    );
}

fn native_events(fixture: &Value, case: &Value) -> Vec<Event> {
    let sounds = case.array("sounds");
    let mut audio_cursor = 0;
    let mut result = Vec::new();
    let actor = number(case, "actor");
    let mut previous_party = bytes(fixture, &case["before"]["party"]);
    let mut previous_npcs = bytes(fixture, &case["before"]["records"]);
    for resolution in case.array("resolutions") {
        let targets: Vec<_> = words(fixture, &resolution["applied_mask"])
            .iter()
            .enumerate()
            .filter(|(_, value)| **value > 0)
            .map(|(body, _)| body)
            .collect();
        let phases: Vec<_> = case
            .array("phase_receipts")
            .iter()
            .filter(|phase| {
                phase["address"] == "15075a"
                    && phase["spell_id"] == resolution["spell_id"]
                    && phase.n::<u64>("draws_before") >= resolution.n::<u64>("draws_before")
                    && phase.n::<u64>("draws_after") <= resolution.n::<u64>("draws_after")
            })
            .collect();
        let party = bytes(fixture, &resolution["party_after"]);
        let npcs = bytes(fixture, &resolution["records_after"]);
        for (index, (body, phase)) in targets.iter().zip(&phases).enumerate() {
            let end = phases
                .get(index + 1)
                .map_or(resolution.n::<usize>("sound_count_after"), |next| {
                    next.n::<usize>("sound_count_before")
                });
            result.extend(sounds[audio_cursor..end].iter().map(|sound| Event::Sound {
                resource_id: sound.as_i64().unwrap() as i16,
            }));
            audio_cursor = end;
            let (before, after, offset) = if *body < 6 {
                (&previous_party, &party, body * WARRIOR_BYTES + 0x50)
            } else {
                (&previous_npcs, &npcs, (body - 10) * MONSTER_BYTES + 0x110)
            };
            let damage = number(phase, "d0_return_word");
            assert_eq!(
                word(before, offset)
                    .unwrap()
                    .wrapping_sub(word(after, offset).unwrap()),
                damage
            );
            result.push(Event::SpellResolved {
                caster: Some(actor),
                spell: number(resolution, "spell_id"),
                target: *body as i16,
                damage,
                defeated: word(after, offset).unwrap() <= 0 && damage != 0,
                icon: Default::default(),
                flight: Default::default(),
            });
        }
        previous_party = party;
        previous_npcs = npcs;
    }
    result.extend(sounds[audio_cursor..].iter().map(|sound| Event::Sound {
        resource_id: sound.as_i64().unwrap() as i16,
    }));
    let after = &case["after"];
    let round = initial(fixture, case)
        .combat
        .as_ref()
        .unwrap()
        .round
        .wrapping_add(
            number(after, "round_script_counter")
                .wrapping_sub(number(&case["before"], "round_script_counter")),
        );
    result.push(Event::TurnStarted {
        body: words(fixture, &after["order"])[number(after, "cursor") as usize],
        round,
    });
    if bytes(fixture, &case["after"]["records"])[(actor - 10) as usize * MONSTER_BYTES + 0x104] != 0
    {
        result = native_common::turn_guard_events(result, actor);
    }
    result
}

fn current_decision(index: usize) {
    let content = content();
    let fixture = support::fixture("current-terrain-restricted-npc-spell.json");

    for case in &fixture.array("cases")[index..index + 1] {
        let mut state = initial(&fixture, case);
        let before_projection = projected(&state, &fixture, &case["before"], &case["before"]);
        same(&state, &before_projection);
        full_native_slots(&state, &fixture, &case["before"], &case["before"]);
        let before = state.clone();
        let expected = projected(&before, &fixture, &case["after"], &case["before"]);
        let expected_events = native_events(&fixture, case);
        let mut resumed = checkpoint(&state, &content, &format!("before-{index}"));
        let events = apply(Command::RunEnemyTurn, &mut state, &content).unwrap();
        assert_eq!(events, expected_events, "case {index}");
        support::same_captured_session(&state, &expected);
        full_native_slots(&state, &fixture, &case["after"], &case["before"]);
        assert_eq!(
            apply(Command::RunEnemyTurn, &mut resumed, &content).unwrap(),
            events
        );
        support::same_captured_session(&resumed, &expected);
        let mut random = before.random.clone();
        for draw in case.array("rng") {
            assert_eq!(random.bound(number(draw, "bound")), number(draw, "result"));
        }
        assert_eq!(random, state.random);
        assert_eq!(
            state.random.draws - before.random.draws,
            case.array("rng").len() as u64
        );
        checkpoint(&state, &content, &format!("after-{index}"));
    }
}

#[test]
fn seed12_current_decision_and_getup_match_native_state_events_and_saved_resume() {
    current_decision(0);
}

#[test]
fn seed1_current_decision_and_getup_match_native_state_events_and_saved_resume() {
    current_decision(1);
}

#[test]
fn current_seed12_target_selection_matches_the_original_count_and_ordered_targets() {
    let content = content();
    let fixture = support::fixture("current-terrain-restricted-npc-spell.json");
    let case = &fixture["cases"][0];
    let state = initial(&fixture, case);
    let combat = current_combat(&state);
    let mission = content
        .map(state.exploration.as_ref().unwrap().map)
        .unwrap();
    let mut random = state.random.clone();
    for draw in &case.array("rng")[..3] {
        assert_eq!(random.bound(number(draw, "bound")), number(draw, "result"));
    }
    let packet = bytes(&fixture, &case["casts"][0]["packet"]);
    let selected = spells::select_targets(
        combat,
        &state.warriors,
        mission,
        &mission.areas,
        6,
        &packet,
        &mut random,
    )
    .unwrap();
    let native_targets = words(&fixture, &case["casts"][0]["targets"]);
    assert_eq!(selected.count, number(&case["casts"][0], "count"));
    assert_eq!(
        selected
            .targets
            .iter()
            .map(|target| [target.first, target.second, target.kind])
            .collect::<Vec<_>>(),
        native_targets.as_chunks::<3>().0
    );
    assert_eq!(
        random.draws,
        case["casts"][0]["draws_before"].as_u64().unwrap()
    );

    // Controlled command linkage, not an earned route or a full native-turn receipt.
    for spell in [2205_i16, 3602] {
        let mut rotated = state.clone();
        let caster = &mut rotated.combat.as_mut().unwrap().enemies.records[6].bytes;
        for slot in caster[0x12..0x26].as_chunks_mut::<2>().0 {
            *slot = spell.to_be_bytes();
        }
        caster[0x10e..0x110].copy_from_slice(&350_i16.to_be_bytes());
        caster[0x7e] = 1;
        let mut resumed = checkpoint(&rotated, &content, "rotated-npc-before");
        let events = apply(Command::RunEnemyTurn, &mut rotated, &content).unwrap();
        assert!(events.iter().any(
            |event| matches!(event, Event::SpellResolved { spell: actual, .. } if *actual == spell)
        ));
        assert_eq!(
            apply(Command::RunEnemyTurn, &mut resumed, &content).unwrap(),
            events
        );
        same(&rotated, &resumed);
        if spell == 2205 {
            assert!(
                current_combat(&rotated)
                    .spell_fields
                    .as_chunks::<20>()
                    .0
                    .iter()
                    .any(|field| word(field, 8).unwrap() == 2205)
            );
        }
        checkpoint(&rotated, &content, "rotated-npc-after");
    }
}

#[test]
fn each_captured_resolution_matches_native_targets_damage_and_rng_interval() {
    let fixture = support::fixture("current-terrain-restricted-npc-spell.json");
    for (case_index, case) in fixture.array("cases").iter().enumerate() {
        let foundation = initial(&fixture, case);
        let mut before_party = bytes(&fixture, &case["before"]["party"]);
        let mut before_npcs = bytes(&fixture, &case["before"]["records"]);
        for (index, resolution) in case.array("resolutions").iter().enumerate() {
            let warriors: Vec<_> = support::warriors(&before_party);
            let enemies: Vec<_> = before_npcs
                .as_chunks::<MONSTER_BYTES>()
                .0
                .iter()
                .map(|record| centurions_content::combat::MonsterRecord {
                    bytes: record.to_vec(),
                })
                .collect();
            let mut random = foundation.random.clone();
            for draw in case.array("rng") {
                if random.draws == resolution.n::<u64>("draws_before") {
                    break;
                }
                assert_eq!(random.bound(number(draw, "bound")), number(draw, "result"));
            }
            let native_work = case
                .array("current_native_events")
                .iter()
                .rfind(|event| event.n::<u64>("draws") <= random.draws)
                .unwrap();
            let snapshot = &native_work["snapshot"];
            let weapon = bytes(&fixture, &snapshot["weapon_work"]);
            let ammo = bytes(&fixture, &snapshot["ammo_work"]);
            let item = bytes(&fixture, &snapshot["item_work"]);
            let cache = bytes(&fixture, &snapshot["first_item_cache"]);
            let packet = bytes(&fixture, &resolution["packet"]);
            let mask = words(&fixture, &resolution["mask"]);
            let outcome = spells::resolve_bodies_with_memory(
                &warriors,
                &enemies,
                &mask,
                &DirectSpell {
                    packet: &packet,
                    level: number(resolution, "spell_id") % 1000 / 100 - 1,
                    count: number(resolution, "count"),
                    armor_filter: word(&weapon, 136).unwrap() != 0
                        || word(&ammo, 136).unwrap() != 0,
                    weapon_sound: word(&weapon, 14).unwrap(),
                },
                RestrictionMemoryView::ordinary_startup(&cache, Some(word(&item, 0x32).unwrap())),
                &mut random,
            )
            .unwrap();
            let expected_party = bytes(&fixture, &resolution["party_after"]);
            let expected_npcs = bytes(&fixture, &resolution["records_after"]);
            let expected_seed = case
                .array("rng")
                .iter()
                .take((resolution.n::<u64>("draws_after") - foundation.random.draws) as usize)
                .fold(foundation.random.clone(), |mut rng, draw| {
                    rng.bound(number(draw, "bound"));
                    rng
                });
            let expected_targets: Vec<_> = words(&fixture, &resolution["applied_mask"])
                .iter()
                .enumerate()
                .filter(|(_, flag)| **flag > 0)
                .map(|(body, _)| {
                    let (before, after, offset) = if body < 6 {
                        (&before_party, &expected_party, body * WARRIOR_BYTES + 0x50)
                    } else {
                        (
                            &before_npcs,
                            &expected_npcs,
                            (body - 10) * MONSTER_BYTES + 0x110,
                        )
                    };
                    let damage = word(before, offset)
                        .unwrap()
                        .wrapping_sub(word(after, offset).unwrap());
                    (
                        body,
                        damage,
                        word(after, offset).unwrap() < 1 && damage != 0,
                    )
                })
                .collect();
            assert_eq!(
                outcome
                    .targets
                    .iter()
                    .map(|(body, result)| (*body, result.damage, result.killed))
                    .collect::<Vec<_>>(),
                expected_targets,
                "case {case_index} resolution {index}"
            );
            assert_eq!(
                random, expected_seed,
                "case {case_index} resolution {index}"
            );
            before_party = expected_party;
            before_npcs = expected_npcs;
        }
    }
}

#[test]
fn npc_mode7_indexed_duration_writes_its_information_owner_or_commits_nothing() {
    let fixture = support::fixture("current-terrain-restricted-npc-spell.json");
    let mut state = initial(&fixture, &fixture["cases"][0]);
    let caster = &mut state.combat.as_mut().unwrap().enemies.records[6].bytes;
    for slot in caster[0x12..0x26].as_chunks_mut::<2>().0 {
        *slot = 2205_i16.to_be_bytes();
    }
    caster[0x10e..0x110].copy_from_slice(&350_i16.to_be_bytes());
    caster[0x7e] = 1;
    state.flags[0..2].copy_from_slice(&3_i16.to_be_bytes());
    let map = state.exploration.as_ref().unwrap().map as usize;
    // Controlled linkage: spell 2205 as a mode7 packet with a fixed duration of 9.
    let cast = |effect: i16| {
        let mut content = content();
        let scenario = content.scenario.as_mut().unwrap();
        let tables = std::sync::Arc::make_mut(&mut scenario.maps[map].shared);
        let attacks = &mut tables.combat.as_mut().unwrap().attacks;
        let packet = &mut attacks[2842 + 406 + 4 * 58..][..58];
        packet[1] = 7;
        packet[13..16].copy_from_slice(&[9, 9, 0]);
        packet[40..42].copy_from_slice(&effect.to_be_bytes());
        let mut candidate = state.clone();
        (apply(Command::RunEnemyTurn, &mut candidate, &content), candidate)
    };
    // resolvespell15133c: effect 21 max-writes WORD44258c, the first story flag WORD.
    let (result, owned) = cast(21);
    result.unwrap();
    assert_eq!(word(&owned.flags, 0).unwrap(), 9);
    assert_eq!(owned.flags[2..], state.flags[2..]);
    // Effect 20 reaches WORD44258a, which no session owner represents.
    let (result, unowned) = cast(20);
    assert!(matches!(result, Err(RuleError::Unsupported { .. })));
    same(&unowned, &state);
}
