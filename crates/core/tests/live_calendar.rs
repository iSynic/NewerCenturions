use centurions_content::{Content, Warrior, word};
use centurions_core::{Command, RandomState, Session, apply};

fn decode(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn long(b: &[u8], offset: usize) -> i32 {
    i32::from_be_bytes(b[offset..offset + 4].try_into().unwrap())
}
#[test]
fn noon_and_midnight_moves_match_live_original_complete_warriors_and_rng() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/live-calendar.json"
    ))
    .unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let before = &case["before"];
        let after = &case["after"];
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
        state.warriors = before["warriors"]
            .as_array()
            .unwrap()
            .iter()
            .map(|w| Warrior::read(&decode(w["hex"].as_str().unwrap())).unwrap())
            .collect();
        state.random = RandomState::new(before["random_seed"].as_u64().unwrap() as u32);
        let globals = decode(before["globals"]["hex"].as_str().unwrap());
        let p = state.exploration.as_mut().unwrap();
        p.x = long(&globals, 0x44) + long(&globals, 0x58);
        p.y = long(&globals, 0x48) + long(&globals, 0x5c);
        p.day = word(&globals, 0xb4).unwrap();
        p.hour = word(&globals, 0xaa).unwrap();
        p.minute = word(&globals, 0xa8).unwrap();
        p.fatigue = word(&decode(before["fatigue"]["hex"].as_str().unwrap()), 0).unwrap();
        apply(Command::Move { dx: 1, dy: 0 }, &mut state, &content).unwrap();
        for (index, w) in state.warriors.iter().enumerate() {
            let expected = decode(after["warriors"][index]["hex"].as_str().unwrap());
            for (offset, (actual, expected)) in w.bytes.iter().zip(expected.iter()).enumerate() {
                assert_eq!(
                    actual, expected,
                    "{} warrior {index} offset {offset:#x}",
                    case["label"]
                );
            }
        }
        let globals = decode(after["globals"]["hex"].as_str().unwrap());
        let p = state.exploration.as_ref().unwrap();
        assert_eq!(
            (p.x, p.y),
            (
                long(&globals, 0x44) + long(&globals, 0x58),
                long(&globals, 0x48) + long(&globals, 0x5c)
            )
        );
        assert_eq!(
            (p.day, p.hour, p.minute),
            (
                word(&globals, 0xb4).unwrap(),
                word(&globals, 0xaa).unwrap(),
                word(&globals, 0xa8).unwrap()
            )
        );
        assert_eq!(
            p.fatigue,
            word(&decode(after["fatigue"]["hex"].as_str().unwrap()), 0).unwrap()
        );
        assert_eq!(
            state.random.seed,
            after["random_seed"].as_u64().unwrap() as u32
        );
        assert_eq!(state.random.draws, 2);
    }
}
