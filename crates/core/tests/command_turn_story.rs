//! Source-derived general turn dispatch; fixture records are only initial inputs.
use crate::{prelude::*, support};

#[test]
fn zero_id_turn_story_resumes_after_saved_pages_without_repeating_chance_or_effects() {
    let mut content = content();
    let tables = Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
    let record = &mut tables.door_scripts[..42];
    record.fill(0);
    record[8..20].copy_from_slice(&support::word_bytes(&[1, 9, 47, 1, 47, 9]));
    record[24..36].copy_from_slice(&support::word_bytes(&[1, 611, 2, 1, 3, 612]));
    let fixture = support::fixture("final-turn-stories.json");
    let (mut arena, warriors, random) =
        crate::binary_final_turn_stories::input(&fixture["cases"][0]);
    arena.battle_order_index = 0;
    arena.roster.records[0].record_mut().set_faction(1);
    arena.roster.records[0]
        .record_mut()
        .set_turn_macro_chance(100);
    arena.roster.records[0].bytes[0x5e..0x60].fill(0);
    let before = arena.roster.records.clone();
    let mut state = support::mission_session(&content);
    state.warriors = warriors;
    state.random = random;
    state.combat = Some(arena);
    state.phase = Phase::Combat;
    crate::native_common::source_victory_context(&mut state);
    let mut expected = state.random.clone();
    expected.bound(100);
    let events = support::saved_command(Command::EndTurn, &mut state, &content);
    assert_eq!(support::event_sounds(&events), [611]);
    support::assert_no_turn_started(&events);
    assert!(state.turn_dispatch.is_some());
    assert_eq!((state.flags[2], state.flags[3]), (1, 0));
    assert_eq!(current_combat(&state).roster.records, before);
    assert_eq!(state.random, expected);
    // First acknowledgement closes the prefix message, then the delegated page.
    support::saved_command(Command::Continue, &mut state, &content);
    let events = support::saved_command(Command::Continue, &mut state, &content);
    assert_eq!(support::event_sounds(&events), [612]);
    assert!(matches!(
        events.last(),
        Some(Event::TurnStarted { body: 10, .. })
    ));
    assert_eq!((state.flags[2], state.flags[3]), (1, 1));
    assert_eq!(state.random, expected);
    assert!(state.turn_dispatch.is_none());
}
