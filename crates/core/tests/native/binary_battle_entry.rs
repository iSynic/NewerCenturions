use crate::{prelude::*, support};
#[test]
fn underground_battle_entry_matches_original_complete_records_and_random_consumption() {
    let fixture: serde_json::Value = support::fixture("battle-entry.json");
    crate::binary_battle_entry::compare_underground_battle_entry_matches_original_complete_records_and_random_consumption(fixture);
}

pub(super) fn compare_underground_battle_entry_matches_original_complete_records_and_random_consumption(
    fixture: Value,
) {
    let content = support::content();

    for case in fixture.array("cases") {
        let state = battle_state(&content, &fixture, case, 6, 0, 0);
        assert_battle(&state, case);
    }
}

pub(super) fn assert_battle(state: &Session, case: &Value) {
    let context = format!(
        "battle{} seed{} variant{}",
        case["battle"], case["seed_before"], case["variant"]
    );
    let combat = current_combat(state);
    combat.validate(&state.warriors).unwrap();
    let encode = |v: &[i16]| support::word_bytes(v);
    support::native_expect! {
        bytes &encode(&combat.map.tiles) => &case.bytes("tiles"), &format!("{context} map");
    };
    let [x, y] = case.n::<[i16; 2]>("shift");
    assert_eq!(combat.map.shift, (x, y));
    support::native_expect! {
        scalar combat.map.edge_word => &case["edge"], "binary_battle_entry";
        records &state.warriors => &case["warriors"], &format!("{context} warriors");
        records &combat.enemies.records => &case["enemies"], &format!("{context} enemies");
        scalar combat.enemies.records.len() => &case["count"], "binary_battle_entry";
    }
    let positions = |p: &[(i16, i16)]| p.iter().flat_map(|(x, y)| [*x, *y]).collect::<Vec<_>>();
    support::native_expect! {
        bytes encode(&positions(&combat.squad.positions)) => &case["squad"], &format!("{context} squad");
        words &(combat.squad.under_tiles) => &case["under"], "binary_battle_entry";
        bytes encode(&positions(&combat.enemies.positions)) => &case["positions"], &format!("{context} enemy positions");
        words &(combat.order) => &case["order"], "binary_battle_entry";
        scalar combat.cursor => &case["cursor"], "binary_battle_entry";
    }
    support::assert_rng(&state.random, &case["seed_after"], &case["draws"]);
}

fn battle_state(
    content: &Content,
    fixture: &Value,
    case: &Value,
    count: usize,
    difficulty: i16,
    marker: i16,
) -> Session {
    let mut state = Session::new(content);
    apply(
        Command::SetDifficulty { level: difficulty },
        &mut state,
        content,
    )
    .unwrap();
    apply(
        Command::SelectStarters {
            indices: (0..count).collect(),
        },
        &mut state,
        content,
    )
    .unwrap();
    apply(
        Command::EnterMission {
            name: "The Thunder of Guns".into(),
        },
        &mut state,
        content,
    )
    .unwrap();
    state.warriors = support::warriors(&fixture["warriors_before"]);
    state.warriors.truncate(count);
    state.warriors[0].bytes[0x90..0x92].copy_from_slice(&marker.to_be_bytes());
    let map = content.map(case.n::<i32>("map")).unwrap();
    let position = state.exploration.as_mut().unwrap();
    position.map = case.n::<i32>("map");
    position.x = case.n::<i32>("x");
    position.y = case.n::<i32>("y");
    position.floor.clone_from(&map.floor);
    position.events.clone_from(&map.events);
    state.random = RandomState::new(case.n::<u32>("seed_before"));
    begin_battle(&mut state, content, case.n::<i16>("battle"), 0).unwrap();
    state
}

// Protects selected-count propagation through combat setup and saved tactical state.
#[test]
fn selected_squad_count_survives_mission_combat_entry_and_reload() {
    let content = support::content();
    let fixture = support::fixture("battle-entry.json");
    for (count, difficulty, marker) in [(1, -2, 0), (3, 0, 1), (6, 2, -1)] {
        let mut state = battle_state(
            &content,
            &fixture,
            &fixture.array("cases")[0],
            count,
            difficulty,
            marker,
        );
        assert_eq!(state.warriors.len(), count);
        state.validate(&content).unwrap();
        let path = std::env::temp_dir().join(format!(
            "centurions-squad-{}-{count}.json",
            std::process::id()
        ));
        save(&path, &state, &content).unwrap();
        assert_eq!(load(&path, &content).unwrap(), state);
        let before = state.clone();
        for command in [
            Command::SetDifficulty { level: 0 },
            Command::EditWarriorUpgrade {
                warrior: 0,
                selector: 0,
            },
        ] {
            assert!(apply(command, &mut state, &content).is_err());
            assert_eq!(state, before);
        }
        if difficulty != 0 {
            let mut value: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            value["schema_version"] = 3.into();
            support::legacy_save(&mut value);
            std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
            assert!(load(&path, &content).is_err());
        }
        std::fs::remove_file(path).unwrap();
    }
}
