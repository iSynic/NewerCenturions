use crate::{prelude::*, support};
use support::{checkpoint, replay_command as step};

use std::path::PathBuf;

fn fixture() -> (Content, Session) {
    let content = support::content();
    let state = support::mission_session(&content);
    (content, state)
}

fn path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("centurions-{name}-{}.json", std::process::id()))
}

fn stage_choice(state: &mut Session, parameter: i16) {
    let position = state.exploration.as_mut().unwrap();
    position.script = Some(centurions_core::ScriptState::scheduled(vec![0; 42], 1));
    position.presentation = Some(Presentation::StageChange { parameter });
}

#[test]
fn failed_story_transaction_discards_mutations_and_ordered_effects() {
    let (content, mut state) = fixture();
    for case in support::fixture("story-statistics.json").array("cases") {
        let warriors = support::warriors(&case.bytes("before"));
        let mask = case.n::<Vec<i16>>("mask").try_into().unwrap();
        let expected = case.bytes("after");
        assert_eq!(
            digest(&support::record_bytes(
                &centurions_core::story_party::adjust_statistics(
                    &warriors,
                    &mask,
                    &case.bytes("extra")
                )
                .unwrap()
            )),
            digest(&expected)
        );
        let parameter = case.n::<i16>("parameter");
        if parameter < 0 {
            continue;
        }
        let mut adjusted = support::mission_session(&content);
        adjusted.warriors = warriors;
        adjusted.native_work.party_mask = mask;
        let position = adjusted.exploration.as_mut().unwrap();
        let mut record = position.event_headers[..42].to_vec();
        record[8..40].fill(0);
        record[8..10].copy_from_slice(&108_i16.to_be_bytes());
        record[24..26].copy_from_slice(&parameter.to_be_bytes());
        position.script = Some(centurions_core::ScriptState::scheduled(record, 0));
        position.presentation = Some(Presentation::Dialogue {
            id: 0,
            text: String::new(),
        });
        let mut restored = checkpoint(&adjusted, &content, "story-statistics");
        replay_command(Command::Continue, &mut adjusted, &mut restored, &content);
        assert_eq!(
            digest(&support::record_bytes(&adjusted.warriors)),
            digest(&expected)
        );
    }
    // Protects original avoidance answers and saved admission from repeating the lottery.
    for case in support::fixture("regional-choice.json").array("cases") {
        let mut walking = support::mission_session(&content);
        let position = walking.exploration.as_mut().unwrap();
        position.x = 10;
        position.y = 10;
        position.floor.fill(0);
        position.events.fill(0);
        position.event_headers.fill(0);
        position.region_state = case.bytes("region");
        crate::native_common::keep_tile_set(&mut walking, &content);
        walking.random = RandomState::new(12);
        apply(Command::Move { dx: 1, dy: 0 }, &mut walking, &content).unwrap();
        assert!(matches!(
            walking.exploration.as_ref().unwrap().presentation,
            Some(Presentation::Avoidance { region: 0, .. })
        ));
        assert_eq!(
            walking.random,
            RandomState {
                seed: 844405278,
                draws: 5
            }
        );
        walking = checkpoint(&walking, &content, "regional-choice");
        let events = apply(
            Command::Choose {
                answer: case.n("answer"),
            },
            &mut walking,
            &content,
        )
        .unwrap();
        if case.n::<bool>("battle_called") {
            assert!(
                events
                    .iter()
                    .any(|event| matches!(event, Event::CombatStarted { battle: 1, .. }))
            );
            assert_eq!(current_combat(&walking).order[1..7], [0, 1, 2, 3, 4, 5]);
        } else {
            assert_eq!(walking.phase, Phase::Exploration);
            assert!(walking.exploration.as_ref().unwrap().presentation.is_none());
            assert_eq!(walking.random.seed, case.n::<u32>("seed"));
            assert_eq!(walking.random.draws, case.n::<u64>("draws"));
        }
    }
    let position = state.exploration.as_mut().unwrap();
    let mut record = vec![0; 42];
    for (offset, value) in [(8, 47_i16), (10, 9), (12, 63), (24, 2), (26, 6000)] {
        record[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    }
    position.script = Some(centurions_core::ScriptState::scheduled(record, 0));
    position.presentation = Some(Presentation::Dialogue {
        id: 0,
        text: String::new(),
    });
    let before = state.clone();
    assert!(apply(Command::Continue, &mut state, &content).is_err());
    assert_eq!(state, before);

    // Scripted effects retain selection, original record changes and RNG across reload.
    for case in support::fixture("story-spell.json").array("cases") {
        let mut casting = support::mission_session(&content);
        casting.warriors = support::warriors(&case.bytes("before"));
        casting.random = RandomState::new(12);
        casting.native_work.party_mask = case.n::<Vec<i16>>("mask").try_into().unwrap();
        let position = casting.exploration.as_mut().unwrap();
        position.script = Some(centurions_core::ScriptState::scheduled(
            content.map(0).unwrap().script(176).unwrap().to_vec(),
            1,
        ));
        position.presentation = Some(Presentation::Dialogue {
            id: 661,
            text: String::new(),
        });
        let events = support::saved_command(Command::Continue, &mut casting, &content);
        assert_eq!(
            support::record_bytes(&casting.warriors),
            case.bytes("after")
        );
        support::assert_rng(&casting.random, &case["seed"], &case["draws"]);
        assert_eq!(support::event_sounds(&events), case.n::<Vec<i16>>("sounds"));
    }
    assert!(apply(Command::SelectMenuEntry { index: 4 }, &mut state, &content).is_err());
    assert_eq!(state, before);

    // The round35 dialog must pause before script36's faction changes and reduction.
    for case in support::fixture("story-faction.json").array("cases") {
        let mut neutralizing = support::mission_session(&content);
        neutralizing.phase = Phase::Combat;
        for warrior in &mut neutralizing.warriors {
            warrior.bytes[0x9df] = 1;
        }
        let mut arena = support::battle();
        arena.enemies.records = support::monsters(&case.bytes("before"));
        arena.enemies.positions = vec![(10, 10); arena.enemies.records.len()];
        arena.order[1] = 0;
        arena.order[2] = 10;
        arena.order[109] = 1;
        arena.cursor = 109;
        arena.round_scripts.counter = 2;
        arena.round_scripts.slots[0] = 35;
        neutralizing.combat = Some(arena);
        neutralizing.party_buffs[2..4].copy_from_slice(&5_i16.to_be_bytes());
        let random = neutralizing.random.clone();
        assert_eq!(
            apply(Command::EndTurn, &mut neutralizing, &content).unwrap(),
            vec![Event::StoryPresented]
        );
        let paused = current_combat(&neutralizing);
        assert_eq!(paused.cursor, 110);
        assert_eq!(paused.round_scripts.counter, 3);
        assert_eq!(
            support::record_bytes(&paused.enemies.records),
            case.bytes("before")
        );
        assert_eq!(neutralizing.random, random);
        assert_eq!(
            centurions_content::word(&neutralizing.party_buffs, 2).unwrap(),
            5
        );
        assert!(matches!(
            neutralizing.exploration.as_ref().unwrap().presentation,
            Some(Presentation::Dialogue { id: 155, .. })
        ));
        let checkpoint = path("native-faction");
        save(&checkpoint, &neutralizing, &content).unwrap();
        let mut restored = load(&checkpoint, &content).unwrap();
        let mut legacy: Value =
            serde_json::from_slice(&std::fs::read(&checkpoint).unwrap()).unwrap();
        legacy["schema_version"] = 5.into();
        support::legacy_save(&mut legacy);
        std::fs::write(&checkpoint, serde_json::to_vec(&legacy).unwrap()).unwrap();
        assert!(load(&checkpoint, &content).is_err());
        replay_command(
            Command::Continue,
            &mut neutralizing,
            &mut restored,
            &content,
        );
        let arena = current_combat(&neutralizing);
        assert!(arena.round_scripts.pending.is_none());
        assert_eq!(arena.round_scripts.counter, 3);
        assert_eq!(arena.active_body(), Some(0));
        assert_eq!(
            centurions_content::word(&neutralizing.party_buffs, 2).unwrap(),
            4
        );
        for (actual, native) in arena
            .enemies
            .records
            .iter()
            .zip(support::monsters(&case.bytes("after")))
        {
            assert_eq!(
                centurions_content::word(&actual.bytes, 14).unwrap(),
                centurions_content::word(&native.bytes, 14).unwrap()
            );
        }
        save(&checkpoint, &neutralizing, &content).unwrap();
        assert_eq!(load(&checkpoint, &content).unwrap(), neutralizing);
        std::fs::remove_file(checkpoint).unwrap();
    }
    // Source-qualified newland opcode119 uses43800e, independently of the active PC/selected NPC.
    let mut survivor = support::mission_session(&content);
    survivor.phase = Phase::Combat;
    let mut arena = support::battle();
    arena.enemies.records =
        support::monsters(&support::fixture("death-summon.json").array("cases")[1].bytes("before"));
    arena.enemies.records.push(arena.enemies.records[0].clone());
    arena.enemies.positions = vec![(20, 20), (21, 20)];
    arena.script_enemy = 1;
    arena.order[1] = 0;
    let untouched = arena.enemies.records[0].clone();
    survivor.combat = Some(arena);
    let position = survivor.exploration.as_mut().unwrap();
    let mut record = vec![0; 42];
    record[8..10].copy_from_slice(&119_i16.to_be_bytes());
    position.script = Some(centurions_core::ScriptState::scheduled(record, 0));
    position.presentation = Some(Presentation::Dialogue {
        id: 0,
        text: String::new(),
    });
    let mut restored = checkpoint(&survivor, &content, "retained-survivor");
    replay_command(Command::Continue, &mut survivor, &mut restored, &content);
    let arena = current_combat(&survivor);
    assert_eq!(arena.enemies.records[0], untouched);
    assert_eq!(arena.enemies.records[1].stamina().unwrap(), 1);
    assert_eq!(word(&arena.enemies.records[1].bytes, 14).unwrap(), 0);
    assert_eq!(arena.enemies.records[1].bytes[0x82], 1);
    // The queued curse must use its removed body's location and preserve its lingering field.
    let case = support::fixture("tactical-curse.json");
    let mut curse = support::mission_session(&content);
    curse.phase = Phase::Combat;
    curse.warriors = support::warriors(&case.bytes("before"));
    curse.random = RandomState::new(12);
    let mut combat = support::battle();
    combat.map.tiles = case.words("map");
    combat.squad.positions = (0..6).map(|i| (20 + i, 20)).collect();
    combat.enemies.records = support::monsters(&case.bytes("enemy_before"));
    combat.enemies.positions = vec![(20, 22)];
    combat.order[1] = 0;
    let entry = centurions_core::combat::death_scripts::DeathEntry {
        script: 176,
        x: 25,
        y: 25,
        ..Default::default()
    };
    combat.death_queue.enqueue(entry).unwrap();
    combat.death_queue.begin();
    curse.combat = Some(combat);
    curse.combat_story = Some(centurions_core::combat::runtime::DeathStory {
        script: 176,
        cursor: 1,
        actor: 0,
        entry,
        ..Default::default()
    });
    apply(Command::Continue, &mut curse, &content).unwrap();
    assert_eq!(support::record_bytes(&curse.warriors), case.bytes("after"));
    assert_eq!(
        current_combat(&curse).enemies.records[0].bytes,
        case.bytes("enemy_after")
    );
    assert_eq!(current_combat(&curse).spell_fields, case.bytes("fields"));
    assert_eq!(
        curse.native_work.party_mask.as_slice(),
        &case.words("mask")[..6]
    );
    support::assert_rng(&curse.random, &case["seed"], &case["draws"]);
    // A queued original question must return through the shared dispatcher after reload.
    curse.warriors[0].bytes[0x40..0x42].copy_from_slice(&5_i16.to_be_bytes());
    let entry = centurions_core::combat::death_scripts::DeathEntry {
        script: 11,
        x: 25,
        y: 25,
        ..Default::default()
    };
    let combat = curse.combat.as_mut().unwrap();
    combat.death_queue = Default::default();
    combat.death_queue.enqueue(entry).unwrap();
    combat.death_queue.begin();
    curse.combat_story = Some(centurions_core::combat::runtime::DeathStory {
        script: 11,
        cursor: 1,
        actor: 0,
        entry,
        ..Default::default()
    });
    for _ in 0..2 {
        apply(Command::Continue, &mut curse, &content).unwrap();
    }
    assert!(matches!(
        curse.exploration.as_ref().unwrap().presentation,
        Some(Presentation::Question { extra: 50, .. })
    ));
    let mut resumed = checkpoint(&curse, &content, "queued-question");
    let random = curse.random.clone();
    assert_eq!(
        apply(Command::Choose { answer: false }, &mut curse, &content).unwrap(),
        apply(Command::Choose { answer: false }, &mut resumed, &content).unwrap()
    );
    assert_eq!(curse, resumed);
    assert!(curse.combat_story.is_none());
    assert!(curse.exploration.as_ref().unwrap().presentation.is_none());
    assert_eq!(curse.random, random);

    // The authored purchase must pay once, preserve its continuation and refuse without charging.
    for case in support::fixture("script-round.json").array("cases") {
        let mut expected = case.bytes("after");
        let mut random = RandomState::new(12);
        let mut warriors = support::warriors(&case.bytes("before"));
        if case.n::<i16>("combat") == 0 {
            let context = centurions_core::story_health::context(&warriors, 0, None);
            warriors =
                centurions_core::story_reduce::reduce_outdoors(&warriors, &context, &mut random, 0)
                    .unwrap()
                    .warriors;
        } else {
            centurions_core::combat::runtime::reduce_round(
                &mut support::battle(),
                &mut warriors,
                &mut random,
            )
            .unwrap();
            // getup clears affected markers after the independently captured reduce call.
            for bytes in expected.as_chunks_mut::<WARRIOR_BYTES>().0 {
                bytes[0x14..0x16].fill(0);
            }
        }
        assert_eq!(support::record_bytes(&warriors), expected);
        support::assert_rng(&random, &case["seed"], &case["draws"]);
    }
    // The supplied death16 recreates MD61, then updates the mission score exactly once.
    for case in support::fixture("death-summon.json").array("cases") {
        let mut summoning = support::mission_session(&content);
        summoning.phase = Phase::Combat;
        summoning.mission_score = 10;
        summoning.random = RandomState::new(12);
        let mut combat = support::battle();
        combat.map.tiles = case.words("map_before");
        combat.enemies.records = support::monsters(&case.bytes("before"));
        combat.enemies.positions = vec![(20, 20)];
        combat.order[1] = 0;
        let entry = centurions_core::combat::death_scripts::DeathEntry {
            script: 16,
            faction: case.n("faction"),
            x: 25,
            y: 25,
            ..Default::default()
        };
        combat.death_queue.enqueue(entry).unwrap();
        combat.death_queue.begin();
        summoning.combat = Some(combat);
        summoning.combat_story = Some(centurions_core::combat::runtime::DeathStory {
            script: 16,
            cursor: 2,
            actor: 0,
            entry,
            ..Default::default()
        });
        if case.n::<i16>("faction") == 1 {
            let mut round = summoning.clone();
            round.combat_story = None;
            let c = round.combat.as_mut().unwrap();
            c.death_queue = Default::default();
            c.cursor = 109;
            c.enemies.records[0].bytes[0x110..0x112].copy_from_slice(&1_i16.to_be_bytes());
            c.enemies.records[0].bytes[0xce] = 2;
            c.enemies.records[0].bytes[0xcb] = 0;
            apply(Command::EndTurn, &mut round, &content).unwrap();
            assert!(round.combat_story.as_ref().unwrap().resume_turn);
            round = checkpoint(&round, &content, "round-death");
            let events = apply(Command::Continue, &mut round, &content).unwrap();
            assert!(
                events
                    .iter()
                    .any(|e| matches!(e, Event::CombatantAdded { .. }))
            );
            assert!(matches!(
                events.last(),
                Some(Event::TurnStarted { body: 0, .. })
            ));
            assert!(round.combat_story.is_none());
        }
        support::saved_command(Command::Continue, &mut summoning, &content);
        assert_eq!(summoning.mission_score, 8);
        let combat = current_combat(&summoning);
        assert_eq!(
            support::record_bytes(&combat.enemies.records),
            case.bytes("after")
        );
        assert_eq!(combat.map.tiles, case.words("map_after"));
        assert_eq!(
            combat.enemies.positions[1],
            support::positions(&case.bytes("position"))[0]
        );
        // The native receipt stops before the caller's survivor dialog entry.
        let mut expected_random = crate::native_common::random_at(case, "seed", "draws");
        if summoning.survivors.is_some() {
            expected_random.bound(4);
        }
        assert_eq!(summoning.random, expected_random);
    }
    for credits in [30_000, 0] {
        let mut purchase = support::mission_session(&content);
        purchase.party_currency = credits;
        for warrior in &mut purchase.warriors {
            warrior.bytes[0x9da..0x9dc].fill(0);
        }
        let position = purchase.exploration.as_mut().unwrap();
        position.presentation = Some(Presentation::Dialogue {
            id: 0,
            text: String::new(),
        });
        position.script = Some(centurions_core::ScriptState::scheduled(
            content.map(0).unwrap().script(308).unwrap().to_vec(),
            1,
        ));
        let events = apply(Command::Continue, &mut purchase, &content).unwrap();
        assert_eq!(purchase.party_currency, if credits > 0 { 7_000 } else { 0 });
        assert_eq!(
            events.iter().any(|event| matches!(
                event,
                Event::Feedback(centurions_core::feedback::NativeFeedback::Warning { index: 50 })
            )),
            credits == 0
        );
        let mut resumed = checkpoint(&purchase, &content, "paid-story");
        if credits > 0 {
            for candidate in [&mut purchase, &mut resumed] {
                apply(Command::StoreLoot { slot: 0 }, candidate, &content).unwrap();
            }
        }
        replay_command(Command::Continue, &mut purchase, &mut resumed, &content);
        assert_eq!(purchase.party_currency, if credits > 0 { 7_000 } else { 0 });
    }
    // Confiscation protects native gear/modifier restoration and a pending recovery save.
    let receipt = support::fixture("confiscated-equipment.json");
    let mut captive: Session = serde_json::from_value(receipt["input"].clone()).unwrap();
    let mut record = content.map(0).unwrap().script(282).unwrap().to_vec();
    record[20..24].fill(0); // Bound this caller after taking equipment, before its battle.
    let stage = |state: &mut Session, record, cursor| {
        let position = state.exploration.as_mut().unwrap();
        position.script = Some(centurions_core::ScriptState::scheduled(record, cursor));
        position.presentation = Some(Presentation::Dialogue {
            id: 0,
            text: String::new(),
        });
    };
    stage(&mut captive, record, 4);
    let events = apply(Command::Continue, &mut captive, &content).unwrap();
    assert_eq!(
        support::record_bytes(&captive.warriors),
        bytes(receipt["taken"].text("warriors"))
    );
    let held = captive.confiscated.as_ref().unwrap();
    assert!(held.active);
    assert_eq!(
        held.items.iter().flat_map(|i| i.0).collect::<Vec<_>>(),
        bytes(receipt["taken"].text("saved_items"))
    );
    assert_eq!(
        held.money,
        receipt["taken"]["saved_money"].as_i64().unwrap() as i32
    );
    assert_eq!(
        captive.party_currency,
        receipt["taken"]["money"].as_i64().unwrap() as i32
    );
    assert_eq!(
        captive.random.seed,
        receipt["taken"]["seed"].as_u64().unwrap() as u32
    );
    assert_eq!(
        captive.random.draws,
        receipt["taken"]["draws"].as_u64().unwrap()
    );
    assert!(matches!(
        events.first(),
        Some(Event::Feedback(
            centurions_core::feedback::NativeFeedback::Flash {
                address: 0x401c02,
                ..
            }
        ))
    ));
    let mut resumed = checkpoint(&captive, &content, "confiscated");
    let mut skirmish = captive.clone();
    skirmish.native_work.party_mask.fill(1);
    stage(
        &mut skirmish,
        content.map(0).unwrap().script(282).unwrap().to_vec(),
        6,
    );
    apply(Command::Continue, &mut skirmish, &content).unwrap();
    assert_eq!(skirmish.phase, centurions_core::Phase::Combat);
    assert_eq!(skirmish.suppress_allies, 1);
    skirmish.validate(&content).unwrap();
    for candidate in [&mut captive, &mut resumed] {
        stage(
            candidate,
            content.map(0).unwrap().script(284).unwrap().to_vec(),
            0,
        );
        apply(Command::Continue, candidate, &content).unwrap();
        assert_eq!(
            support::record_bytes(&candidate.warriors),
            bytes(receipt["restored"].text("warriors"))
        );
        assert_eq!(
            candidate.exploration.as_ref().unwrap().presentation,
            Some(Presentation::Loot)
        );
        apply(Command::Continue, candidate, &content).unwrap();
        assert_eq!(candidate.suppress_allies, 0);
        assert!(matches!(
            candidate.exploration.as_ref().unwrap().presentation,
            Some(Presentation::Dialogue { id: 1068, .. })
        ));
    }
    assert_eq!(captive, resumed);
    // A supplied one-item choice must close once and resume its dialogue after reload.
    let limited = support::fixture("limited-loot.json");
    let mut choice = support::mission_session(&content);
    stage(
        &mut choice,
        content.map(0).unwrap().script(191).unwrap().to_vec(),
        1,
    );
    apply(Command::Continue, &mut choice, &content).unwrap();
    assert_eq!(
        choice.loot_limit,
        Some(limited["remaining_before"].as_i64().unwrap() as i16)
    );
    assert_eq!(
        content.map(0).unwrap().treasure[76 * 88..77 * 88],
        bytes(limited.text("treasure"))
    );
    let mut resumed = checkpoint(&choice, &content, "limited-choice");
    let slot = choice
        .booty_items
        .iter()
        .position(|item| item.id() != 0)
        .unwrap();
    for candidate in [&mut choice, &mut resumed] {
        apply(Command::TakeLoot { slot, warrior: 0 }, candidate, &content).unwrap();
        assert_eq!(
            candidate.loot_limit,
            Some(limited["remaining_after"].as_i64().unwrap() as i16)
        );
        assert!(!candidate.booty_items.iter().any(|item| item.id() != 0));
        assert!(matches!(
            candidate.exploration.as_ref().unwrap().presentation,
            Some(Presentation::Dialogue { id: 741, .. })
        ));
        let before = candidate.clone();
        assert!(apply(Command::TakeLoot { slot, warrior: 0 }, candidate, &content).is_err());
        assert_eq!(*candidate, before);
    }
    assert_eq!(choice, resumed);
    apply(Command::Continue, &mut choice, &content).unwrap();
    assert_eq!(
        choice.exploration.as_ref().unwrap().presentation,
        Some(Presentation::Shop)
    );
    apply(Command::CloseShop, &mut choice, &content).unwrap();
    assert!(matches!(
        choice.exploration.as_ref().unwrap().presentation,
        Some(Presentation::Dialogue { id: 743, .. })
    ));
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
    let abandoned = destination.with_extension("json.tmp");
    std::fs::write(&abandoned, b"interrupted save").unwrap();
    save(&destination, &state, &content).unwrap();
    assert_eq!(load(&destination, &content).unwrap(), state);
    assert_eq!(std::fs::read(&abandoned).unwrap(), b"interrupted save");
    std::fs::remove_file(abandoned).unwrap();
    stage_choice(&mut state, 1);
    apply(Command::Choose { answer: true }, &mut state, &content).unwrap();
    assert_eq!(state.exploration.as_ref().unwrap().map_state(), original);
    assert_eq!(state.flags[17], 1);
    assert_eq!(state.mission_score, 23);
    std::fs::remove_file(destination).unwrap();

    // Synthetic second level exercises isolation; native map-byte receipts remain above.
    let mut content = content;
    let mut second = content.scenario.as_ref().unwrap().clone();
    second.level = 1;
    second.catalog = Default::default();
    std::sync::Arc::make_mut(&mut content.scenario.as_mut().unwrap().catalog)
        .insert((second.name.clone(), 1), second);
    let unchanged_rng = state.random.clone();
    for level in [1, 0] {
        let position = state.exploration.as_mut().unwrap();
        let mut record = vec![0; 42];
        record[4..7].copy_from_slice(&[0, 20, 20]);
        record[40] = level;
        position.script = Some(ScriptState {
            record,
            cursor: 8,
            map_slot: 0,
            item_return: false,
            scheduled: false,
            menu: None,
            persist: false,
            origin: None,
            caller_action: None,
            chained: false,
        });
        position.presentation = Some(Presentation::Dialogue {
            id: 0,
            text: String::new(),
        });
        apply(Command::Continue, &mut state, &content).unwrap();
        assert_eq!(state.scenario_level, level);
        state = checkpoint(&state, &content, "level-travel");
    }
    assert_eq!(state.exploration.as_ref().unwrap().floor[81], 123);
    assert_eq!(state.random, unchanged_rng);
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
    support::legacy_save(&mut document);
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
        "inventory_record",
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

#[test]
fn shared_direct_death_child_preserves_original_header_and_saved_dialogue_return() {
    use centurions_core::{
        Event, MapState,
        combat::{self, runtime::DeathStory},
    };

    let (mut content, mut state) = fixture();
    let position = state.exploration.as_mut().unwrap();
    position.restore_map(1, MapState::authored(content.map(1).unwrap()));
    position.x = 77;
    position.y = 9;
    position.presentation = None;
    position.script = Some(ScriptState {
        map_slot: 8,
        ..centurions_core::ScriptState::scheduled(
            content.map(1).unwrap().script(55).unwrap().to_vec(),
            1,
        )
    });
    combat::begin_battle(&mut state, &content, 14, 0).unwrap();
    // Controlled defeat is the same boundary used by the native nested-caller fixture.
    let arena = state.combat.as_mut().unwrap();
    arena.direct_deaths = true;
    arena.enemies.records[0].bytes[0x110..0x112].fill(0);
    combat::lifecycle::kill_body(arena, &mut state.warriors, 10).unwrap();
    arena.death_queue.processing = true;
    let entry = arena.direct_death.take().unwrap().1;
    assert_eq!(entry.script, 51);

    // Source-derived delegated caller; zero score replaces one sound, not native receipt bytes.
    let scripts =
        &mut std::sync::Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[1].shared)
            .door_scripts;
    scripts[51 * 42 + 10..51 * 42 + 12].copy_from_slice(&109_i16.to_be_bytes());
    scripts[51 * 42 + 26..51 * 42 + 28].fill(0);

    state.combat_story = Some(DeathStory {
        direct: true,
        body: 0,
        script: 51,
        actor: 0,
        entry,
        ..Default::default()
    });
    let mut resumed = checkpoint(&state, &content, "nested-death");
    let mut started = vec![14];
    for _ in 0..20 {
        let events = step(Command::Continue, &mut state, &mut resumed, &content);
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
    for candidate in [&mut state, &mut resumed] {
        candidate.combat.as_mut().unwrap().enemies.records[0].bytes[0x110..0x112].fill(0);
    }
    step(Command::EndTurn, &mut state, &mut resumed, &content);
    assert!(state.combat.is_none());
    let rewarded = state.warriors.clone();
    let rewarded_random = state.random.clone();
    // Source-derived post-child prompt; the complete queued controls run in binary_nested_victory.

    let script = state.exploration.as_mut().unwrap().script.as_mut().unwrap();
    let cursor = usize::from(script.cursor);
    script.record[8 + cursor * 2..10 + cursor * 2].copy_from_slice(&1_i16.to_be_bytes());
    script.record[24 + cursor * 2..26 + cursor * 2].copy_from_slice(&851_i16.to_be_bytes());
    resumed = checkpoint(&state, &content, "delegated-child-return");
    step(Command::Continue, &mut state, &mut resumed, &content);
    assert!(state.combat_story.as_ref().unwrap().child_returned);
    state = checkpoint(&state, &content, "returned-child-dialogue");
    assert!(matches!(
        state.exploration.as_ref().unwrap().presentation,
        Some(Presentation::Dialogue { id: 851, .. })
    ));

    step(Command::Continue, &mut state, &mut resumed, &content);
    assert!(state.combat_story.is_none());
    let returned = state.exploration.as_ref().unwrap().script.as_ref().unwrap();
    assert_eq!(
        &returned.record,
        &support::fixture("nested-caller.json").array("cases")[0].bytes("body_after")
    );
    assert_eq!(state.warriors, rewarded);
    assert_eq!(state.random, rewarded_random);
    assert_eq!(returned.cursor, 1);
}
