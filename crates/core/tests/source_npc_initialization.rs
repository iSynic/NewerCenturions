//! Source-derived caller comparisons; native fixtures provide only starting inputs.
use crate::{prelude::*, support};
use centurions_content::combat::BattleRecord;
use centurions_core::combat::{monster, place_enemies_at_difficulty};
use centurions_core::story_allies::add_ally;

#[test]
fn random_weapon_callers_preserve_prefix_draws_and_their_overlapping_range_order() {
    let fixture = support::fixture("enemies.json");
    let mission = support::terrain_mission(&support::fixture("combat-map.json"));
    let mut battle = BattleRecord::read(&fixture.bytes("battle"), 0).unwrap();
    let cell = battle.cells.iter().position(|id| *id != 0).unwrap();
    let mut definition =
        MonsterRecord::read(&fixture.bytes("definitions"), battle.cells[cell]).unwrap();
    definition.record_mut().set_selected_item(150);
    definition.record_mut().set_primary_weapon(150);
    definition.bytes[0x28..0x2a].copy_from_slice(&50_i16.to_be_bytes());
    battle.cells.fill(0);
    battle.cells[cell] = 1;
    let menu = vec![0; 20 * 288];
    for (caller, selector, primary, item) in [
        (0, -2, 125, 125),
        (1, -2, 125, 125),
        (2, -2, 137, 137),
        (2, -1, 75, 50),
    ] {
        let initialize = |definition: &MonsterRecord, seed| {
            let mut random = RandomState::new(seed);
            let record = match caller {
                0 => {
                    let added = add_ally(&menu, 0, &definition.bytes, 0, &mut random).unwrap();
                    MonsterRecord {
                        bytes: added.records[..288].to_vec(),
                    }
                }
                1 => monster::summon(definition.clone(), -1, 0, &mut random).unwrap(),
                _ => {
                    let mut map = TacticalMap {
                        tiles: fixture.words("map"),
                        shift: (-39, -15),
                        edge_word: 0,
                    };
                    let mut definitions = vec![0; 576];
                    definitions[288..].copy_from_slice(&definition.bytes);
                    let base = fixture["cases"][0].n::<i16>("base");
                    place_enemies_at_difficulty(
                        &mut map,
                        &mission,
                        &battle,
                        &definitions,
                        (base, base),
                        &mut random,
                        0,
                    )
                    .unwrap()
                    .records
                    .remove(0)
                }
            };
            (record, random)
        };
        // Table two's inclusive ranges overlap at85.
        let (seed, mut expected_record, expected_random) = (1..10000)
            .find_map(|seed| {
                let (record, mut after) = initialize(&definition, seed);
                (after.bound(100) == 85).then_some((seed, record, after))
            })
            .unwrap();
        let mut selected = definition.clone();
        selected.record_mut().set_selected_item(selector);
        let (actual, random) = initialize(&selected, seed);
        expected_record.record_mut().set_primary_weapon(primary);
        expected_record.record_mut().set_selected_item(item);
        assert_eq!(actual, expected_record, "caller{caller}");
        assert_eq!(random, expected_random, "caller{caller}");
    }
    definition.record_mut().set_selected_item(-10);
    let mut random = RandomState::new(1);
    let before = random.clone();
    let refused = monster::summon(definition, -1, 0, &mut random);
    assert!(matches!(refused, Err(RuleError::Unsupported { .. })));
    assert_eq!(random, before);
}

#[test]
fn signed_removal_dispatch_precedes_unneeded_origin_reads_and_survives_save_resume() {
    let foundation = support::fixture("round-stories.json");
    for (count, seed, removed) in [(i16::MIN, 1, true), (i16::MIN, 22039, false), (1, 1, false)] {
        let mut content = content();
        let tables = Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
        tables.combat.as_mut().unwrap().encounters.resize(9010, 0);
        tables.combat.as_mut().unwrap().encounters[9000..9010]
            .copy_from_slice(&support::word_bytes(&[i16::MAX, count, 0, 0, 0]));
        let (arena, warriors, _) =
            crate::binary_round_stories::input(&foundation, &foundation["cases"][0]);
        let mut state = support::mission_session(&content);
        state.warriors = warriors;
        state.phase = Phase::Combat;
        state.combat = Some(arena);
        state.random = RandomState::new(seed);
        let record = support::script_record(&[(125, 900)]);
        support::pause_script(&mut state, record, 0);
        let mut expected = state.random.clone();
        if count < 0 {
            expected.bound(count);
        }
        let events = support::saved_command(Command::Continue, &mut state, &content);
        let bodies: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::CombatantRemoved { body } => Some(*body),
                _ => None,
            })
            .collect();
        assert_eq!(bodies, if removed { vec![10] } else { vec![] });
        assert_eq!(
            current_combat(&state).enemies.records[0].stamina().unwrap() == 0,
            removed
        );
        assert_eq!(state.random, expected);
        support::saved_command(Command::EndMission, &mut state, &content);
    }
}

#[test]
fn initial_npc_story_resumes_its_saved_entry_without_an_advance_announcement() {
    let mut content = content();
    let tables = Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
    let record = &mut tables.door_scripts[..42];
    record.copy_from_slice(&support::script_record(&[(47, 2), (1, 1), (47, 3)]));
    let definitions = &mut tables.combat.as_mut().unwrap().monsters;
    for enemy in definitions.as_chunks_mut::<288>().0 {
        enemy[0x5c..0x60].copy_from_slice(&support::word_bytes(&[100, 0]));
    }
    let mut state = support::mission_session(&content);
    let position = state.exploration.as_mut().unwrap();
    position.script = None;
    position.presentation = None;
    let events = centurions_core::combat::begin_battle(&mut state, &content, -1, 0).unwrap();
    assert_eq!(
        state.turn_dispatch.as_ref().unwrap().caller,
        centurions_core::combat::runtime::TurnCaller::Initial
    );
    support::assert_no_turn_started(&events);
    assert_eq!((state.flags[2], state.flags[3]), (1, 0));
    let events = support::saved_command(Command::Continue, &mut state, &content);
    support::assert_no_turn_started(&events);
    assert_eq!((state.flags[2], state.flags[3]), (1, 1));
    assert!(state.turn_dispatch.is_none());
}
