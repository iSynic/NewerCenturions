//! Source-derived clinic entry and strict save continuation coverage.
use crate::{prelude::*, support};

#[test]
fn clinic_intro_defers_pooling_and_resumes_the_script_once() {
    let mut content = support::content();
    let tables = std::sync::Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
    tables.combat.as_mut().unwrap().encounters[2974..2976].copy_from_slice(&1_i16.to_be_bytes());
    let mut state = support::mission_session(&content);
    state.warriors[0].record_mut().set_carried_credits(123);
    let carried: i32 = state
        .warriors
        .iter()
        .map(|w| i32::from(w.record().carried_credits().unwrap() as u16))
        .sum();
    let pool = state.party_currency;
    let warriors = state.warriors.clone();
    let random = state.random.clone();
    let record = support::script_record(&[(-32, 297), (9, 611)]);
    support::pause_script(&mut state, record, 0);
    let events = support::saved_command(Command::Continue, &mut state, &content);
    assert_eq!(events[0], Event::Sound { resource_id: 655 });
    assert!(matches!(events[1], Event::Message { id: 1, .. }));
    assert_eq!((state.party_currency, &state.warriors), (pool, &warriors));
    assert_eq!(
        support::saved_command(Command::Continue, &mut state, &content),
        vec![Event::Sound { resource_id: 430 }, Event::StoryPresented]
    );
    assert_eq!(state.party_currency, pool.wrapping_add(carried));
    let empty = |w: &Warrior| w.record().carried_credits().unwrap() == 0;
    assert!(state.warriors.iter().all(empty));
    let events = support::saved_command(Command::ClinicControl { item: 11 }, &mut state, &content);
    assert!(events.starts_with(&[
        Event::Sound { resource_id: -141 },
        Event::Sound { resource_id: 611 }
    ]));
    assert!(state.exploration.as_ref().unwrap().script.is_none());
    assert_eq!(state.random, random);
}
