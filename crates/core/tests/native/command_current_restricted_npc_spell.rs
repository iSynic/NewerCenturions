use crate::prelude::*;

use crate::support::bytes as hex;

use centurions_core::combat::spells::RestrictionMemoryView;
fn fixture() -> Value {
    crate::support::fixture("current-terrain-restricted-npc-spell.json")
}

fn bytes(fixture: &Value, value: &Value) -> Vec<u8> {
    if value.is_string() {
        return hex(value);
    }
    let hash = value.text("shared_buffer_sha256");
    let result = hex(&fixture["shared_rule_buffers"][hash]);
    crate::support::expect(result.len(), &value["byte_length"]);
    assert_eq!(digest(&result), hash);
    result
}

fn words(fixture: &Value, value: &Value) -> Vec<i16> {
    crate::support::words(&bytes(fixture, value))
}
use crate::native_common::snapshot_word as number;

use crate::native_common::foundation_session as initial;

fn projected(before: &Session, fixture: &Value, snapshot: &Value, baseline: &Value) -> Session {
    let mut expected = before.clone();
    for (warrior, native) in expected
        .warriors
        .iter_mut()
        .zip(bytes(fixture, &snapshot["party"]).as_chunks::<3138>().0)
    {
        warrior.bytes = Warrior::read(native).unwrap().bytes;
    }
    for (records, key) in [
        (&mut expected.crate_items, "crate"),
        (&mut expected.booty_items, "loot"),
    ] {
        *records = crate::support::inventory(&bytes(fixture, &snapshot[key]));
    }
    expected.random.seed = snapshot.n::<u32>("seed");
    expected.random.draws = snapshot.n::<u64>("draws");
    expected.selected_warrior = number(snapshot, "selected_member") as usize;
    expected.loaded_item_field32 = Some(number(snapshot, "item_context"));
    if snapshot["draws"] != baseline["draws"] {
        expected.ammunition_work.item = Some(bytes(fixture, &snapshot["item_work"]));
    }
    expected.attack_count = number(snapshot, "count");
    expected.bandage_ready = Some(number(snapshot, "aid_gate"));
    expected.party_currency = snapshot.n::<i32>("currency");
    expected.flags = bytes(fixture, &snapshot["flags"]);
    expected.ammunition = crate::support::longs(&bytes(fixture, &snapshot["ammunition"]));
    expected.psi_memory = Some(bytes(fixture, &snapshot["memory"]));
    expected.party_buffs = bytes(fixture, &snapshot["party_buffs"]);
    expected.psi_disabled = number(snapshot, "psi_disabled") as u8;
    expected.enemy_spell_disabled = number(snapshot, "enemy_spell_disabled") as u8;
    expected
        .party_mask
        .copy_from_slice(&words(fixture, &snapshot["mask"])[..6]);
    let combat = expected.combat.as_mut().unwrap();
    assert_eq!(
        combat.enemies.records.len() * 288,
        bytes(fixture, &snapshot["records"]).len()
    );
    crate::snapshots::records(combat, &|key| bytes(fixture, &snapshot[key]));
    // Preserve the command's viewport while comparing original absolute rule coordinates.
    let delta = (
        number(snapshot, "shift_x").wrapping_sub(number(baseline, "shift_x")),
        number(snapshot, "shift_y").wrapping_sub(number(baseline, "shift_y")),
    );
    crate::snapshots::geometry(combat, &|key| bytes(fixture, &snapshot[key]), delta);
    combat.cursor = number(snapshot, "cursor") as usize;
    combat.round = before.combat.as_ref().unwrap().round.wrapping_add(
        number(snapshot, "round_script_counter")
            .wrapping_sub(number(baseline, "round_script_counter")),
    );
    combat.round_scripts.counter = number(snapshot, "round_script_counter") as i8;
    combat.round_scripts.slots = words(fixture, &snapshot["round_slots"]).try_into().unwrap();
    crate::snapshots::spell_work(combat, &|key| bytes(fixture, &snapshot[key]));
    combat.unavailable = number(snapshot, "unavailable") as i8;

    let weapon = bytes(fixture, &snapshot["weapon_work"]);
    let ammo = bytes(fixture, &snapshot["ammo_work"]);
    combat.weapon_work = WeaponWork {
        sound: Some(word(&weapon, 14).unwrap()),
        weapon_armor_filter: Some(word(&weapon, 136).unwrap()),
        ammunition_armor_filter: Some(word(&ammo, 136).unwrap()),
    };
    expected.attack_packet = combat.attack_packet.clone();
    expected.attack_identity = combat.attack_identity;
    expected.weapon_work = combat.weapon_work;
    let health = expected.health_context.as_mut().unwrap();
    health.selected_warrior = expected.selected_warrior;
    health.unavailable = combat.unavailable;
    health.effect = number(snapshot, "health_effect") as i8;
    health.last_body = number(snapshot, "health_last_body");
    health.marks_affected = number(snapshot, "heal_marks_affected") != 0;
    health
        .affected
        .copy_from_slice(&words(fixture, &snapshot["markers"])[..6]);
    expected
}
use crate::support::checkpoint;
use crate::support::same_session as same;

fn full_native_slots(state: &Session, fixture: &Value, snapshot: &Value, baseline: &Value) {
    let combat = state.combat.as_ref().unwrap();
    let live = combat.enemies.records.len();
    let mut records = bytes(fixture, &baseline["all_records"]);
    assert_eq!(records.len(), 100 * 288);
    for (index, record) in combat.enemies.records.iter().enumerate() {
        records[index * 288..(index + 1) * 288].copy_from_slice(&record.bytes);
    }
    assert_eq!(records, bytes(fixture, &snapshot["all_records"]));
    assert!(records[live * 288..].iter().all(|byte| *byte == 0));
    let mut positions = words(fixture, &baseline["all_monster_positions"]);
    assert_eq!(positions.len(), 200);
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
        assert_eq!(phases.len(), targets.len());
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
                (&previous_party, &party, body * 3138 + 0x50)
            } else {
                (&previous_npcs, &npcs, (body - 10) * 288 + 0x110)
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
    result
}

fn current_decision(index: usize) {
    let content = content();
    let fixture = fixture();

    for case in &fixture.array("cases")[index..index + 1] {
        let mut state = initial(&fixture, case);
        state.validate(&content).unwrap();
        let before_projection = projected(&state, &fixture, &case["before"], &case["before"]);
        same(&state, &before_projection);
        full_native_slots(&state, &fixture, &case["before"], &case["before"]);
        let before = state.clone();
        let expected = projected(&before, &fixture, &case["after"], &case["before"]);
        let expected_events = native_events(&fixture, case);
        let mut resumed = checkpoint(&state, &content, &format!("before-{index}"));
        let events = apply(Command::RunEnemyTurn, &mut state, &content).unwrap();
        assert_eq!(events, expected_events, "case {index}");
        same(&state, &expected);
        full_native_slots(&state, &fixture, &case["after"], &case["before"]);
        assert_eq!(
            apply(Command::RunEnemyTurn, &mut resumed, &content).unwrap(),
            events
        );
        same(&resumed, &expected);
        let mut random = before.random.clone();
        for draw in case.array("rng") {
            assert_eq!(random.bound(number(draw, "bound")), number(draw, "result"));
        }
        assert_eq!(random, state.random);
        assert_eq!(
            state.random.draws - before.random.draws,
            case.array("rng").len() as u64
        );
        assert_eq!(
            state.health_context.as_ref().unwrap().effect,
            state.attack_packet.as_ref().unwrap()[18] as i8
        );
        same(
            &checkpoint(&state, &content, &format!("after-{index}")),
            &expected,
        );
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
    let fixture = fixture();
    let case = &fixture["cases"][0];
    let state = initial(&fixture, case);
    let combat = state.combat.as_ref().unwrap();
    let mission = content
        .map(state.exploration.as_ref().unwrap().map)
        .unwrap();
    let mut random = state.random.clone();
    for draw in &case.array("rng")[..3] {
        assert_eq!(random.bound(number(draw, "bound")), number(draw, "result"));
    }
    assert_eq!(random.draws, 53215);
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
}

#[test]
fn each_captured_resolution_matches_native_targets_damage_and_rng_interval() {
    let fixture = fixture();
    for (case_index, case) in fixture.array("cases").iter().enumerate() {
        let foundation = initial(&fixture, case);
        let mut before_party = bytes(&fixture, &case["before"]["party"]);
        let mut before_npcs = bytes(&fixture, &case["before"]["records"]);
        for (index, resolution) in case.array("resolutions").iter().enumerate() {
            let warriors: Vec<_> = crate::support::warriors(&before_party);
            let enemies: Vec<_> = before_npcs
                .as_chunks::<288>()
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
                        (&before_party, &expected_party, body * 3138 + 0x50)
                    } else {
                        (&before_npcs, &expected_npcs, (body - 10) * 288 + 0x110)
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
