use centurions_content::{Content, Warrior, combat::MonsterRecord};
use centurions_core::{
    Command, Event, RandomState, RuleError, Session, apply,
    combat::runtime::{EnemyWork, enemy_action_with_work},
    load, save,
};
use serde_json::Value;
fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../evidence/fixtures/reached-guardian-spell.json"
    ))
    .unwrap()
}
fn content() -> Content {
    let r = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Content::load(&r.join("NewCenturions"), &r.join("analysis/extracted")).unwrap()
}
fn bytes(s: &str) -> Vec<u8> {
    s.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|v| u8::from_str_radix(std::str::from_utf8(v).unwrap(), 16).unwrap())
        .collect()
}
fn raw(c: &Value, k: &str) -> Vec<u8> {
    bytes(c[k].as_str().unwrap())
}
fn words(v: &[u8]) -> Vec<i16> {
    v.as_chunks::<2>()
        .0
        .iter()
        .map(|v| i16::from_be_bytes(*v))
        .collect()
}
fn assert_bytes(a: &[u8], e: &[u8], name: &str) {
    assert_eq!(a.len(), e.len(), "{name} length");
    if let Some(i) = a.iter().zip(e).position(|(a, e)| a != e) {
        panic!("{name} offset {i:x}: {} != {}", a[i], e[i]);
    }
}
fn state(f: &Value, c: &Value) -> Session {
    let mut s: Session = serde_json::from_value(f["foundation"].clone()).unwrap();
    s.warriors = raw(c, "party_before")
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|v| Warrior::read(v).unwrap())
        .collect();
    let combat = s.combat.as_mut().unwrap();
    combat.enemies.records = raw(c, "records_before")
        .as_chunks::<288>()
        .0
        .iter()
        .map(|v| MonsterRecord { bytes: v.to_vec() })
        .collect();
    s.enemy_spell_disabled = u8::from(c["variant"] == 6);
    s
}
#[test]
fn blocked_first_spell_chance_matches_six_complete_native_enemy_turns() {
    let f = fixture();
    let content = content();
    let mission = content.map(2).unwrap();
    let mut checked = 0;
    for case in f["cases"].as_array().unwrap() {
        if case["variant"] == 1 {
            continue;
        }
        let mut s = state(&f, case);
        let combat = s.combat.as_mut().unwrap();
        let mut rng = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let mut work = EnemyWork {
            item_context: s.loaded_item_field32,
            attack_count: s.attack_count,
            packet: combat.attack_packet.clone(),
            identity: combat.attack_identity,
            spell_disabled: s.enemy_spell_disabled,
            ..EnemyWork::default()
        };
        let events = enemy_action_with_work(
            combat,
            &mut s.warriors,
            mission,
            &mut rng,
            &content.items,
            &mut work,
        )
        .unwrap();
        let context = format!("variant{}", case["variant"]);
        assert_bytes(
            &s.warriors
                .iter()
                .flat_map(|w| w.bytes.iter().copied())
                .collect::<Vec<_>>(),
            &raw(case, "party_after"),
            &format!("{context} party"),
        );
        assert_bytes(
            &combat
                .enemies
                .records
                .iter()
                .flat_map(|m| m.bytes.iter().copied())
                .collect::<Vec<_>>(),
            &raw(case, "records_after"),
            &format!("{context} enemies"),
        );
        assert_eq!(
            combat.map.tiles,
            words(&raw(case, "map_after")),
            "{context} map"
        );
        assert_eq!(
            combat.order,
            words(&raw(case, "order_after")),
            "{context} order"
        );
        assert_eq!(
            combat.enemies.positions,
            words(&raw(case, "positions_after"))
                .as_chunks::<2>()
                .0
                .iter()
                .map(|p| (p[0], p[1]))
                .collect::<Vec<_>>(),
            "{context} positions"
        );
        assert_eq!(combat.spell_fields, raw(case, "fields_after"));
        assert_eq!(combat.field_used, raw(case, "used_after"));
        assert_eq!(
            combat.attack_packet.as_ref().unwrap(),
            &raw(case, "packet_after"),
            "{context} packet"
        );
        assert_eq!(
            work.item_context,
            Some(case["context_after"].as_i64().unwrap() as i16)
        );
        assert_eq!(
            work.attack_count,
            case["attack_count_after"].as_i64().unwrap() as i16
        );
        assert_eq!(
            rng.seed,
            case["seed_after"].as_u64().unwrap() as u32,
            "{context} seed"
        );
        assert_eq!(
            rng.draws,
            case["rng_draws"].as_u64().unwrap(),
            "{context} draws"
        );
        let sounds: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let Event::Sound { resource_id } = e {
                    Some(*resource_id)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(serde_json::json!(sounds), case["sounds"], "{context} audio");
        checked += 1;
    }
    assert_eq!(checked, 6);
}
#[test]
fn reached_disabled_spell_turn_resumes_and_uncovered_spell_is_atomic() {
    let f = fixture();
    let content = content();
    let mut s = state(&f, &f["cases"][0]);
    let path = std::env::temp_dir().join(format!(
        "centurions-enemy-guard-{}.json",
        std::process::id()
    ));
    save(&path, &s, &content).unwrap();
    let mut resumed = load(&path, &content).unwrap();
    let events = apply(Command::RunEnemyTurn, &mut s, &content).unwrap();
    assert_eq!(
        apply(Command::RunEnemyTurn, &mut resumed, &content).unwrap(),
        events
    );
    assert_eq!(resumed, s);
    save(&path, &s, &content).unwrap();
    assert_eq!(load(&path, &content).unwrap(), s);
    let mut unsupported = state(&f, &f["cases"][1]);
    let before = unsupported.clone();
    assert!(matches!(
        apply(Command::RunEnemyTurn, &mut unsupported, &content),
        Err(RuleError::Unsupported { .. })
    ));
    assert_eq!(unsupported, before);
    std::fs::remove_file(path).unwrap();
}
