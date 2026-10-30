//! Source-composed callback interruption; original records are only starting inputs.
use crate::{prelude::*, support};
use centurions_content::records::{Condition, SpellFieldMut};

/// Warrior 0 acting at cursor 1 against one scripted NPC whose story sets flags 2 and 3.
fn scripted_battle(child: bool) -> (Content, Session, MonsterRecord) {
    let mut content = content();
    let tables = Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
    let mut actions = vec![(47, 2), (1, 1)];
    actions.extend(child.then_some((2, 104)));
    actions.push((47, 3));
    tables.door_scripts[42..84].copy_from_slice(&support::script_record(&actions));
    let case = &support::fixture("natural-reflection.json")["cases"][3];
    let mut state = support::mission_session(&content);
    state.phase = Phase::Combat;
    state.warriors[0] = Warrior::read(&case.bytes("input")).unwrap();
    let mut combat = support::battle();
    combat.field.tiles.fill(1155);
    let target = MonsterRecord {
        bytes: case.bytes("target"),
    };
    combat.roster.records = vec![target.clone()];
    combat.roster.records[0].record_mut().set_todo_on_death(1);
    combat.roster.mon_pos = vec![(19, 20)];
    combat.squad.pos = (0..6).map(|i| (40 + i, 40)).collect();
    combat.battle_order[1] = 0;
    combat.in_spell = 0;
    combat.kill_counts.num_enemy = 2;
    combat.kill_counts.kill_mon = 0;
    state.combat = Some(combat);
    state.random = RandomState::new(case.n("initial_seed"));
    let position = state.exploration.as_mut().unwrap();
    position.script = None;
    position.presentation = None;
    (content, state, target)
}

#[test]
fn reflected_reaction_resumes_the_saved_walk_without_repeating_the_attack() {
    let mut ordinary_points = 0;
    for (child, mode) in [(false, 0_i16), (true, 0), (true, 1), (true, 5)] {
        let (content, mut state, _) = scripted_battle(child);
        state.warriors[0].record_mut().set_movement(6);
        let combat = state.combat.as_mut().unwrap();
        let monster = &mut combat.roster.records[0];
        monster.bytes[0x110..0x112].copy_from_slice(&1_i16.to_be_bytes());
        monster.record_mut().set_reaction_ready(!child);
        combat.squad.pos[0] = (20, 20);
        combat.field.tiles[24 * 90 + 25] = 10;
        combat.field.tiles[25 * 90 + 25] = 0;
        let before = current_combat(&state).squad.pos.clone();
        support::saved_command(Command::Move { dx: 1, dy: 0 }, &mut state, &content);
        assert_eq!(current_combat(&state).squad.pos, before);
        assert_eq!((state.flags[2], state.flags[3]), (1, 0));
        let mut child_cursor = 0;
        if child {
            let parent_map = current_combat(&state).field.tiles.clone();
            support::saved_command(Command::Continue, &mut state, &content);
            state.globals.extra_record[8..10].copy_from_slice(&mode.to_be_bytes());
            let mut treasure = vec![0; 88];
            treasure[..4].copy_from_slice(&support::word_bytes(&[200, -3]));
            treasure[80..].copy_from_slice(&support::long_bytes(&[6, 17]));
            state.globals.treasure = treasure.clone();
            let arena = state.combat.as_mut().unwrap();
            arena.spell_queue.fill(73);
            arena.collide_check[0] = 1;
            arena.kill_counts.kill_mon = arena.kill_counts.num_enemy;
            for monster in &mut arena.roster.records {
                monster.record_mut().set_stamina(0);
            }
            support::saved_command(Command::EndTurn, &mut state, &content);
            if mode == 5 {
                continue;
            }
            let points = state.warriors[0].record().upgrade_points();
            if mode == 0 {
                ordinary_points = points;
            }
            assert_eq!(points, ordinary_points + i32::from(mode == 1));
            if mode == 1 {
                treasure[..2].fill(0);
                assert_eq!(state.booty_list[0].id(), 200);
            }
            assert_eq!(state.globals.treasure, treasure);
            state.booty_list.fill(InventoryRecord([0; 22]));
            child_cursor = state.globals.battle_order.index;
            let battle = &mut state.globals.battlefield;
            assert_eq!(battle.field.tiles, parent_map);
            let fields = battle.spell_queue.as_chunks::<20>().0;
            let reset = |f: &[u8; 20]| f[15] == 0 && f[18] == 0 && f[16] == 73;
            assert!(fields.iter().all(reset));
            battle.spell_queue[15] = 2;
            battle.spell_queue[18] = (child_cursor + 1) as u8;
        }
        let random = state.random.clone();
        let events = support::saved_command(Command::Continue, &mut state, &content);
        assert!(state.combat_story.is_none());
        assert_eq!((state.flags[2], state.flags[3]), (1, 1));
        assert_eq!(state.random, random);
        if child {
            assert!(state.combat.is_none());
            let globals = &state.globals;
            let getup_cursor = globals.battle_order.index;
            assert_eq!(getup_cursor, child_cursor + 1);
            let battle = &globals.battlefield;
            assert_eq!((battle.spell_queue[15], battle.collide_check[0]), (1, 1));
        } else {
            assert_eq!(current_combat(&state).squad.pos[0], (21, 20));
        }
        let moved = events
            .iter()
            .filter(|e| matches!(e, Event::TacticalMoved { .. }));
        assert_eq!(moved.count(), usize::from(!child));
    }
}

#[test]
fn getup_drain_death_runs_its_script_before_the_next_monster_is_drained() {
    let (content, mut state, mut second) = scripted_battle(false);
    let combat = state.combat.as_mut().unwrap();
    for monster in [&mut combat.roster.records[0], &mut second] {
        monster.record_mut().set_stamina(2);
        monster.record_mut().set_condition(Condition::Diseased, 5);
    }
    combat.roster.records.push(second);
    combat.roster.mon_pos.push((30, 30));
    combat.kill_counts.num_enemy = 3;
    support::saved_command(Command::EndTurn, &mut state, &content);
    // getup's killbody ran newland before the loop reached the second NPC.
    assert_eq!((state.flags[2], state.flags[3]), (1, 0));
    let second = |state: &Session| current_combat(state).roster.records[1].clone();
    assert_eq!(second(&state).record().stamina().unwrap(), 2);
    support::saved_command(Command::Continue, &mut state, &content);
    assert_eq!((state.flags[2], state.flags[3]), (1, 1));
    assert!(second(&state).record().stamina().unwrap() < 1);
}

#[test]
fn round_field_moves_the_selected_warrior_to_the_field_centre() {
    let (content, mut state, defeated) = scripted_battle(true);
    state.warriors[0].record_mut().set_movement(6);
    let combat = state.combat.as_mut().unwrap();
    let monster = &mut combat.roster.records[0];
    monster.bytes[0x110..0x112].copy_from_slice(&1_i16.to_be_bytes());
    monster.record_mut().set_reaction_ready(false);
    combat.squad.pos[0] = (20, 20);
    combat.field.tiles[24 * 90 + 25] = 10;
    combat.field.tiles[25 * 90 + 25] = 0;
    support::saved_command(Command::Move { dx: 1, dy: 0 }, &mut state, &content);
    support::saved_command(Command::Continue, &mut state, &content);
    let arena = state.combat.as_mut().unwrap();
    arena.kill_counts.kill_mon = arena.kill_counts.num_enemy;
    arena.roster.records[0].record_mut().set_stamina(0);
    support::saved_command(Command::EndTurn, &mut state, &content);
    // Getup scans past the other warriors to the round's reduction, whose party
    // collide meets a movement field (Data S 1403, effect 56) covering warrior 0.
    let globals = &mut state.globals;
    globals.kill_counts.num_enemy = globals.kill_counts.kill_mon + 1;
    globals.battle_order.order[3..7].fill(-1);
    let mut dead = defeated;
    dead.record_mut().set_stamina(0);
    for slot in &mut globals.monsters.slots {
        slot.get_or_insert(centurions_core::combat::globals::MonsterSlot {
            record: dead.clone(),
            position: None,
        });
    }
    let battle = &mut globals.battlefield;
    let ((x, y), shift) = (battle.pos[0], battle.field.map_shift);
    let mut field = centurions_content::records::SpellFieldMut(&mut battle.spell_queue[..20]);
    field.set_center((x + shift.0 + 3, y + shift.1 + 5));
    field.set_spell(1403);
    field.set_duration(3);
    field.set_power(1);
    field.set_level(3);
    state.bandage_ready = 1;
    let events = support::saved_command(Command::Continue, &mut state, &content);
    let moved = Event::TacticalMoved {
        body: 0,
        x: x - 2,
        y,
    };
    assert!(events.contains(&moved));
    assert_eq!(state.globals.battlefield.pos[0], (x - 2, y));
    assert!(events.contains(&Event::TurnStarted { body: 0, round: 1 }));
}

#[test]
fn live_field_and_newland17_movement_arms_move_the_selected_warrior() {
    for scripted in [false, true] {
        let (mut content, mut state, _) = scripted_battle(false);
        let tables = Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
        let combat = state.combat.as_mut().unwrap();
        if scripted {
            // getup's killbody runs the NPC's story; newland17 casts Data S 1403
            // (effect 56) at the death entry's target.
            tables.door_scripts[42..84].copy_from_slice(&support::script_record(&[(17, 0)]));
            let extra = support::word_bytes(&[1403, 1]);
            tables.combat.as_mut().unwrap().encounters[..4].copy_from_slice(&extra);
            let monster = &mut combat.roster.records[0];
            monster.record_mut().set_stamina(2);
            monster.record_mut().set_condition(Condition::Diseased, 5);
        } else {
            // getup's party collide meets a movement field (Data S 1403, effect 56)
            // centred on (38, 40), two cells left of warrior 0, with map shift 0.
            let mut field = SpellFieldMut(&mut combat.spell_queue[..20]);
            field.set_center((43, 45));
            field.set_spell(1403);
            field.set_duration(3);
            field.set_power(1);
            field.set_level(3);
        }
        state.bandage_ready = 1;
        let events = support::saved_command(Command::EndTurn, &mut state, &content);
        let (x, y) = current_combat(&state).squad.pos[0];
        assert!(scripted || (x, y) == (38, 40));
        assert!(events.contains(&Event::TacticalMoved { body: 0, x, y }));
    }
}
