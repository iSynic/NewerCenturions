//! Public caller compositions; numeric fidelity belongs to the native suffix cases.
use crate::{prelude::*, support};
use centurions_content::records::EnemyRecord;

fn input(spell: i16) -> (Content, Session, usize, i16) {
    let mut content = support::content();
    let area = support::fixture("outdoor-area-party.json");
    let buffs = support::fixture("outdoor-psi.json");
    let fixture = if spell == 1102 { &buffs } else { &area };
    let case = fixture
        .array("cases")
        .iter()
        .find(|case| case["spell_id"] == spell)
        .unwrap();
    let mut state = super::command_support_psi::outdoor(&content, case);
    state.holdover = area.array("cases")[0]["before"].bytes("allies");
    state.held_over = 2;
    let first = state.holdover[..288].to_vec();
    state.holdover[288..576].copy_from_slice(&first);
    for record in state.holdover[..576].as_chunks_mut::<288>().0 {
        let mut record = centurions_content::records::EnemyRecordMut::from_bytes(record);
        record.set_stamina(if spell == 1706 { 5 } else { 50 });
        record.set_stamina_max(50);
    }
    if spell != 1102 {
        let offset = (spell / 1000 - 1) as usize * 2842
            + (spell % 1000 / 100 - 1) as usize * 406
            + (spell % 100 - 1) as usize * 58;
        let tables = Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
        let packet = &mut tables.combat.as_mut().unwrap().attacks[offset..offset + 58];
        packet[32..34].copy_from_slice(&12_i16.to_be_bytes());
        packet[34..36].copy_from_slice(&12_i16.to_be_bytes());
        if spell == 1706 {
            packet[40..42].copy_from_slice(&57_i16.to_be_bytes());
        }
    }
    (content, state, case.n("caster"), case.n("power"))
}

#[test]
fn public_allied_casts_publish_partial_records_and_keep_terminal_copy_order() {
    for spell in [1301_i16, 1706, 1102] {
        let (content, mut state, warrior, power) = input(spell);
        state.globals.monsters.num_mon = 17;
        let position = state.globals.monsters.slots[1]
            .as_ref()
            .unwrap()
            .position;
        let untouched = state.globals.monsters.slots[99].clone();
        state.globals.monsters.slots[0] = None;
        let before = state.holdover.clone();
        support::saved_command(
            Command::CastPartyPsi {
                warrior,
                spell,
                power,
            },
            &mut state,
            &content,
        );
        state = support::checkpoint(&state, &content, "outdoor-partial-record");
        assert_eq!(state.globals.monsters.num_mon, 17);
        assert_eq!(
            state.globals.monsters.slots[0]
                .as_ref()
                .unwrap()
                .position,
            None
        );
        assert_eq!(
            state.globals.monsters.slots[1]
                .as_ref()
                .unwrap()
                .position,
            position
        );
        assert_eq!(state.globals.monsters.slots[99], untouched);
        for index in 0..2 {
            assert_eq!(
                state.globals.monsters.slots[index]
                    .as_ref()
                    .unwrap()
                    .record
                    .bytes,
                state.holdover[index * 288..(index + 1) * 288]
            );
        }
        let tail = &state.globals.track_tail;
        assert_eq!(&tail[4..6], [1, 1]);
        assert!(tail[..4].iter().chain(&tail[6..]).all(|value| *value == 0));
        let hp = EnemyRecord::from_bytes(&state.holdover[..288])
            .stamina()
            .unwrap();
        match spell {
            1301 => assert!(hp < 50 && hp > 0),
            1706 => assert!(hp > 5 && hp <= 50),
            _ => assert_eq!(state.holdover, before),
        }
    }
    let (content, mut terminal, warrior, power) = input(1301);
    for warrior in &mut terminal.warriors {
        warrior.record_mut().set_stamina(200);
    }
    centurions_content::records::EnemyRecordMut::from_bytes(&mut terminal.holdover[..288])
        .set_stamina(1);
    terminal
        .globals
        .health_context
        .kill_party = terminal.warriors.len() as i8 - 1;
    let menu = terminal.holdover.clone();
    let second = terminal.holdover[288..576].to_vec();
    let events = support::saved_command(
        Command::CastPartyPsi {
            warrior,
            spell: 1301,
            power,
        },
        &mut terminal,
        &content,
    );
    assert!(terminal.exploration.is_none() && terminal.warriors.is_empty());
    assert_eq!(terminal.held_over, 2);
    assert_eq!(terminal.holdover, menu);
    assert_eq!(
        terminal.globals.monsters.slots[1]
            .as_ref()
            .unwrap()
            .record
            .bytes,
        second
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, Event::MissionEnded { defeated: true }))
            .count(),
        1
    );
}
