use centurions_content::{Content, Warrior};
use centurions_core::{Command, RandomState, Session, apply, combat::begin_battle};
use std::path::Path;

fn bytes(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn words(s: &str) -> Vec<i16> {
    bytes(s)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|v| i16::from_be_bytes(*v))
        .collect()
}
fn assert_bytes(actual: &[u8], expected: &[u8], context: &str) {
    assert_eq!(actual.len(), expected.len(), "{context} length");
    if let Some(index) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "{context} differs at byte {index}: {} != {}",
            actual[index], expected[index]
        );
    }
}

#[test]
fn required_battle_entry_matches_original_complete_records_and_random_consumption() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/required-battle-entry.json"
    ))
    .unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let mut state = Session::new(&content);
        apply(Command::ImportStarters, &mut state, &content).unwrap();
        apply(
            Command::EnterMission {
                name: "The Thunder of Guns".into(),
            },
            &mut state,
            &content,
        )
        .unwrap();
        state.warriors = bytes(fixture["warriors_before"].as_str().unwrap())
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|b| Warrior::read(b).unwrap())
            .collect();
        let map = content.map(case["map"].as_i64().unwrap() as i32).unwrap();
        let position = state.exploration.as_mut().unwrap();
        position.map = case["map"].as_i64().unwrap() as i32;
        position.x = case["x"].as_i64().unwrap() as i32;
        position.y = case["y"].as_i64().unwrap() as i32;
        position.floor.clone_from(&map.floor);
        position.events.clone_from(&map.events);
        state.random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        begin_battle(
            &mut state,
            &content,
            case["battle"].as_i64().unwrap() as i16,
            0,
        )
        .unwrap();
        let combat = state.combat.as_ref().unwrap();
        let context = format!("battle{} seed{}", case["battle"], case["seed_before"]);
        let encode = |v: &[i16]| v.iter().flat_map(|w| w.to_be_bytes()).collect::<Vec<_>>();
        assert_bytes(
            &encode(&combat.map.tiles),
            &bytes(case["tiles"].as_str().unwrap()),
            &format!("{context} map"),
        );
        assert_eq!(
            combat.map.shift,
            (
                case["shift"][0].as_i64().unwrap() as i16,
                case["shift"][1].as_i64().unwrap() as i16
            ),
            "{context} shift"
        );
        assert_eq!(
            combat.map.edge_word,
            case["edge"].as_i64().unwrap() as i16,
            "{context} edge"
        );
        assert_bytes(
            &state
                .warriors
                .iter()
                .flat_map(|w| w.bytes.iter().copied())
                .collect::<Vec<_>>(),
            &bytes(case["warriors"].as_str().unwrap()),
            &format!("{context} warriors"),
        );
        assert_bytes(
            &combat
                .enemies
                .records
                .iter()
                .flat_map(|m| m.bytes.iter().copied())
                .collect::<Vec<_>>(),
            &bytes(case["enemies"].as_str().unwrap()),
            &format!("{context} enemies"),
        );
        assert_eq!(
            combat.enemies.records.len(),
            case["count"].as_u64().unwrap() as usize,
            "{context} count"
        );
        let positions = |p: &[(i16, i16)]| p.iter().flat_map(|(x, y)| [*x, *y]).collect::<Vec<_>>();
        assert_bytes(
            &encode(&positions(&combat.squad.positions)),
            &bytes(case["squad"].as_str().unwrap()),
            &format!("{context} squad"),
        );
        assert_eq!(
            combat.squad.under_tiles,
            words(case["under"].as_str().unwrap()),
            "{context} under"
        );
        assert_bytes(
            &encode(&positions(&combat.enemies.positions)),
            &bytes(case["positions"].as_str().unwrap()),
            &format!("{context} enemy positions"),
        );
        assert_eq!(
            combat.order,
            words(case["order"].as_str().unwrap()),
            "{context} order"
        );
        assert_eq!(
            combat.cursor,
            case["cursor"].as_u64().unwrap() as usize,
            "{context} cursor"
        );
        assert_eq!(
            (state.random.seed, state.random.draws),
            (
                case["seed_after"].as_u64().unwrap() as u32,
                case["draws"].as_u64().unwrap()
            ),
            "{context} RNG"
        );
    }
}
