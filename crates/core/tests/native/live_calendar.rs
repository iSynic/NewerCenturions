use crate::{prelude::*, support};

fn long(b: &[u8], offset: usize) -> i32 {
    i32::from_be_bytes(b[offset..offset + 4].try_into().unwrap())
}
#[test]
fn noon_and_midnight_moves_match_live_original_complete_warriors_and_rng() {
    let content = support::content();
    let fixture: serde_json::Value = support::fixture("live-calendar.json");
    for case in fixture.array("cases") {
        let before = &case["before"];
        let after = &case["after"];
        let mut state = support::mission_session(&content);
        state.warriors = before
            .array("warriors")
            .iter()
            .map(|w| Warrior::read(&w.bytes("hex")).unwrap())
            .collect();
        state.random = RandomState::new(before.n::<u32>("random_seed"));
        let globals = before["globals"].bytes("hex");
        let p = state.exploration.as_mut().unwrap();
        p.x = long(&globals, 0x44) + long(&globals, 0x58);
        p.y = long(&globals, 0x48) + long(&globals, 0x5c);
        p.day = word(&globals, 0xb4).unwrap();
        p.hour = word(&globals, 0xaa).unwrap();
        p.minute = word(&globals, 0xa8).unwrap();
        p.fatigue = word(&before["fatigue"].bytes("hex"), 0).unwrap();
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
        let globals = after["globals"].bytes("hex");
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
        assert_eq!(p.fatigue, word(&after["fatigue"].bytes("hex"), 0).unwrap());
        support::expect(state.random.seed, &after["random_seed"]);
        assert_eq!(state.random.draws, 2);
    }
}
