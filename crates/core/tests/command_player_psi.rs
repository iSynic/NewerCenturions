use centurions_content::Content;
use centurions_core::{Command, Session, apply, load, save};
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../evidence/fixtures/player-psi.json")).unwrap()
}

fn content() -> Content {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap()
}

fn bytes(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn assert_bytes(actual: &[u8], expected: &[u8], context: &str) {
    assert_eq!(actual.len(), expected.len(), "{context} size");
    if let Some((offset, (a, e))) = actual
        .iter()
        .zip(expected)
        .enumerate()
        .find(|(_, (a, e))| a != e)
    {
        panic!("{context}: first difference at {offset:#x}: actual {a:#x}, expected {e:#x}");
    }
}

fn foundation(fixture: &Value) -> Session {
    let mut state: Session = serde_json::from_value(fixture["route_foundation"].clone()).unwrap();
    let memory: Vec<_> = state
        .warriors
        .iter()
        .flat_map(|w| w.bytes[0xa76..0xac6].iter().copied())
        .collect();
    state.psi_memory = Some(memory);
    state.weapon_work = centurions_core::combat::weapon_work::WeaponWork::zero();
    state.combat.as_mut().unwrap().weapon_work = state.weapon_work;
    state
}

fn command(spell: i16) -> Command {
    Command::CastPsi {
        spell,
        power: 1,
        target: (spell == 2204).then_some((37, 59)),
    }
}

#[test]
fn production_casts_match_original_campaign_records_and_resume() {
    let fixture = fixture();
    let content = content();
    for case in fixture["route_cases"].as_array().unwrap() {
        let spell = case["spell_id"].as_i64().unwrap() as i16;
        let mut uninterrupted = foundation(&fixture);
        let mut resumed = uninterrupted.clone();
        let path = std::env::temp_dir().join(format!(
            "centurions-player-psi-{}-{spell}.json",
            std::process::id()
        ));
        save(&path, &resumed, &content).unwrap();
        resumed = load(&path, &content).unwrap();
        let events = apply(command(spell), &mut uninterrupted, &content).unwrap();
        assert_eq!(
            apply(command(spell), &mut resumed, &content).unwrap(),
            events
        );
        assert_eq!(resumed, uninterrupted);
        let actual: Vec<_> = uninterrupted
            .warriors
            .iter()
            .flat_map(|w| w.bytes.iter().copied())
            .collect();
        assert_bytes(
            &actual,
            &bytes(case["warriors_after"].as_str().unwrap()),
            &format!("spell {spell} warriors"),
        );
        let combat = uninterrupted.combat.as_ref().unwrap();
        let actual: Vec<_> = combat
            .enemies
            .records
            .iter()
            .flat_map(|m| m.bytes.iter().copied())
            .collect();
        assert_bytes(
            &actual,
            &bytes(case["records_after"].as_str().unwrap()),
            &format!("spell {spell} enemies"),
        );
        let actual: Vec<_> = combat
            .map
            .tiles
            .iter()
            .flat_map(|word| word.to_be_bytes())
            .collect();
        assert_bytes(
            &actual,
            &bytes(case["map_after"].as_str().unwrap()),
            &format!("spell {spell} map"),
        );
        assert_eq!(
            combat.spell_fields,
            bytes(case["fields_after"].as_str().unwrap())
        );
        assert_eq!(
            combat.field_used,
            bytes(case["used_after"].as_str().unwrap())
        );
        assert_eq!(
            uninterrupted.psi_memory.as_ref().unwrap(),
            &bytes(case["memory_after"].as_str().unwrap())
        );
        assert_eq!(
            uninterrupted.random.seed,
            case["seed_after"].as_u64().unwrap() as u32
        );
        let initial_draws = foundation(&fixture).random.draws;
        assert_eq!(
            uninterrupted.random.draws - initial_draws,
            case["rng_draws"].as_u64().unwrap()
        );
        assert_eq!(uninterrupted.attack_packet, combat.attack_packet);
        assert_eq!(uninterrupted.attack_identity, combat.attack_identity);
        assert_eq!(uninterrupted.attack_count, 1);
        save(&path, &uninterrupted, &content).unwrap();
        assert_eq!(load(&path, &content).unwrap(), uninterrupted);
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn unsupported_and_invalid_player_casts_leave_the_complete_session_unchanged() {
    let fixture = fixture();
    let content = content();
    for request in [
        Command::CastPsi {
            spell: 2101,
            power: 1,
            target: Some((37, 59)),
        },
        Command::CastPsi {
            spell: 2112,
            power: 1,
            target: Some((37, 59)),
        },
        Command::CastPsi {
            spell: 2204,
            power: 3,
            target: Some((37, 59)),
        },
        Command::CastPsi {
            spell: 2204,
            power: 2,
            target: Some((37, 59)),
        },
        Command::CastPsi {
            spell: 2204,
            power: 1,
            target: Some((80, 80)),
        },
        Command::CastPsi {
            spell: 2204,
            power: 1,
            target: None,
        },
        Command::CastPsi {
            spell: 2302,
            power: 1,
            target: Some((37, 59)),
        },
    ] {
        let mut state = foundation(&fixture);
        let before = state.clone();
        assert!(
            apply(request.clone(), &mut state, &content).is_err(),
            "{request:?}"
        );
        assert_eq!(state, before, "{request:?}");
    }
    for offset in [0x98, 0xac, 0x60, 0x68, 0x90, 0x14] {
        let mut state = foundation(&fixture);
        let body = state.combat.as_ref().unwrap().active_body().unwrap() as usize;
        state.warriors[body].bytes[offset..offset + 2].copy_from_slice(&1_i16.to_be_bytes());
        let before = state.clone();
        assert!(apply(command(2204), &mut state, &content).is_err());
        assert_eq!(state, before, "condition {offset:x}");
    }
    let mut state = foundation(&fixture);
    state.psi_disabled = 1;
    let before = state.clone();
    assert!(apply(command(2204), &mut state, &content).is_err());
    assert_eq!(state, before);
}

#[test]
fn old_save_initializes_spell_memory_from_party_bytes_and_rejects_bad_lengths() {
    let fixture = fixture();
    let content = content();
    let mut state = foundation(&fixture);
    state.warriors[0].bytes[0xa76..0xa7e].copy_from_slice(&[0, 1, 0, 2, 0, 3, 0, 4]);
    let path = std::env::temp_dir().join(format!(
        "centurions-player-psi-migration-{}.json",
        std::process::id()
    ));
    let mut saved = serde_json::json!({"schema_version": 3, "session": state});
    saved["session"]
        .as_object_mut()
        .unwrap()
        .remove("psi_memory");
    saved["session"]
        .as_object_mut()
        .unwrap()
        .remove("psi_disabled");
    std::fs::write(&path, serde_json::to_vec(&saved).unwrap()).unwrap();
    let migrated = load(&path, &content).unwrap();
    assert_eq!(
        &migrated.psi_memory.as_ref().unwrap()[..8],
        &[0, 1, 0, 2, 0, 3, 0, 4]
    );
    assert_eq!(migrated.psi_disabled, 0);
    saved["session"]["psi_memory"] = serde_json::json!([]);
    std::fs::write(&path, serde_json::to_vec(&saved).unwrap()).unwrap();
    assert!(load(&path, &content).is_err());
    std::fs::remove_file(path).unwrap();
}
