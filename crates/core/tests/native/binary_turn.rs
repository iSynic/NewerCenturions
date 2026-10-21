use crate::{prelude::*, support};

#[test]
fn first_turn_matches_whole_records_and_random_draws() {
    let fixture: serde_json::Value = support::fixture("turn.json");
    for case in fixture.array("cases") {
        let mut warriors: Vec<_> = support::warriors(&case["warriors"]);
        let mut enemies = vec![MonsterRecord {
            bytes: case.bytes("enemy"),
        }];
        let mut random = RandomState::new(1);
        initialize_turn(
            case.n::<i16>("body"),
            &mut warriors,
            &mut enemies,
            &[12, 14, 12, 14, 10, 12, 10, 10, 14, 16, 0, 12],
            &mut random,
        )
        .unwrap();
        support::native_expect! {
            records &warriors => &case["after_warriors"], format!("body {}",case["body"]);
            bytes enemies[0].bytes => &case["after_enemy"], "binary_turn";
            rng random => (&case["seed"], &case["draws"]), "binary_turn";
        }
    }
    let content = content();
    for case in support::fixture("turn-caller-contexts.json").array("cases") {
        let mut state = support::mission_session(&content);
        state.warriors = support::warriors(&case["warriors"]);
        state.random = RandomState::new(case.n("seed_before"));
        let position = state.exploration.as_mut().unwrap();
        position.presentation = None;
        position.script = None;
        let mut c = support::battle();
        c.enemies.records = vec![MonsterRecord {
            bytes: case.bytes("enemy"),
        }];
        c.enemies.positions = vec![(20, 20)];
        c.order[1..4].copy_from_slice(&[1, 0, 2]);
        state.combat = Some(c);
        state.phase = Phase::Combat;
        let events = support::saved_command(Command::EndTurn, &mut state, &content);
        support::native_expect! {
            records &state.warriors => &case["after_warriors"], "conditioned Getup party";
            records &current_combat(&state).enemies.records => &case["after_enemy"], "conditioned Getup NPC";
            rng state.random => (&case["seed"], &case["draws"]), "conditioned Getup RNG";
            scalar current_combat(&state).cursor => &case["cursor"], "conditioned Getup skip";
            scalar current_combat(&state).unavailable => &case["unavailable"], "conditioned Getup counter";
            scalar state.selected_warrior => &case["selected_warrior"], "conditioned Getup selection";
            scalar support::event_sounds(&events) => &case["sounds"], "conditioned Getup sounds";
        }
    }
}
