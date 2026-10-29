use super::*;
use crate::test_support::decode;
use std::fs;
#[path = "../native/items_reference.rs"]
mod native_items;

#[test]
fn pack_command_is_atomic_and_resumes_with_the_same_events() {
    let content = crate::test_support::content();
    let ammo = crate::test_support::fixture("ammunition-init.json");
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
    state.exploration.as_mut().unwrap().presentation = None;
    state.exploration.as_mut().unwrap().script = None;
    state.movement_continuation = None;
    let pack = decode(ammo["cases"][0]["record"].as_str().unwrap());
    state.warriors[0].bytes[0x132 + 66..0x132 + 88].copy_from_slice(&pack);
    state.warriors[0].bytes[0x1a..0x1c].copy_from_slice(&4i16.to_be_bytes());
    let path = std::env::temp_dir().join(format!("centurions-pack-{}.json", std::process::id()));
    save(&path, &state, &content).unwrap();
    let mut resumed = load(&path, &content).unwrap();
    let command = Command::UseAmmunitionPack {
        warrior: 0,
        slot: 3,
    };
    let expected = apply(command.clone(), &mut state, &content).unwrap();
    assert_eq!(
        apply(command.clone(), &mut resumed, &content).unwrap(),
        expected
    );
    assert_eq!(state, resumed);
    assert_eq!(state.ammunition[0], 32);
    assert_eq!(state.warriors[0].inventory().unwrap().len(), 3);
    for (id, quantity, equipped) in [(200, 0, 0), (200, 32, 1), (1, 32, 0)] {
        let mut rejected = resumed.clone();
        let w = &mut rejected.warriors[0];
        w.bytes[0x132 + 66..0x132 + 88].copy_from_slice(&pack);
        w.bytes[0x132 + 66..0x134 + 66].copy_from_slice(&(id as i16).to_be_bytes());
        w.bytes[0x134 + 66..0x136 + 66].copy_from_slice(&(quantity as i16).to_be_bytes());
        w.bytes[0x132 + 66 + 9] = equipped;
        w.bytes[0x1a..0x1c].copy_from_slice(&4i16.to_be_bytes());
        let before = rejected.clone();
        assert!(apply(command.clone(), &mut rejected, &content).is_err());
        assert_eq!(rejected, before);
    }
    fs::remove_file(path).unwrap();
}
