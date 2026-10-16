use crate::prelude::*;

use crate::support::assert_bytes;

fn state(f: &Value, c: &Value) -> Session {
    let mut s: Session = serde_json::from_value(f["foundation"].clone()).unwrap();
    s.warriors = crate::support::warriors(&c["party_before"]);
    let combat = s.combat.as_mut().unwrap();
    combat.enemies.records = crate::support::monsters(&c["records_before"]);
    s.enemy_spell_disabled = u8::from(c["variant"] == 6);
    s
}
#[test]
fn blocked_first_spell_chance_matches_six_complete_native_enemy_turns() {
    let f = crate::support::fixture("reached-guardian-spell.json");
    let content = content();
    let mission = content.map(2).unwrap();
    for case in f.array("cases") {
        if case["variant"] == 1 {
            continue;
        }
        let mut s = state(&f, case);
        let combat = s.combat.as_mut().unwrap();
        let mut rng = RandomState::new(case.n::<u32>("seed_before"));
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
            &crate::support::record_bytes(&s.warriors),
            &case.bytes("party_after"),
            &format!("{context} party"),
        );
        assert_bytes(
            &crate::support::record_bytes(&combat.enemies.records),
            &case.bytes("records_after"),
            &format!("{context} enemies"),
        );
        assert_eq!(
            combat.map.tiles,
            words(&case.bytes("map_after")),
            "{context} map"
        );
        assert_eq!(
            combat.order,
            words(&case.bytes("order_after")),
            "{context} order"
        );
        assert_eq!(
            combat.enemies.positions,
            crate::support::positions(&case.bytes("positions_after")),
            "{context} positions"
        );
        assert_eq!(combat.spell_fields, case.bytes("fields_after"));
        assert_eq!(combat.field_used, case.bytes("used_after"));
        assert_eq!(
            combat.attack_packet.as_ref().unwrap(),
            &case.bytes("packet_after"),
            "{context} packet"
        );
        assert_eq!(work.item_context, Some(case.n::<i16>("context_after")));
        crate::support::native_expect! {
        scalar work.attack_count => &case["attack_count_after"], "binary_enemy_spell_guards";
        scalar rng.seed => &case["seed_after"], "binary_enemy_spell_guards";
        scalar rng.draws => &case["rng_draws"], "binary_enemy_spell_guards";
        }
        let sounds: Vec<_> = crate::support::event_sounds(&events);
        assert_eq!(serde_json::json!(sounds), case["sounds"], "{context} audio");
    }
}
#[test]
fn reached_disabled_spell_turn_resumes_and_uncovered_spell_is_atomic() {
    let f = crate::support::fixture("reached-guardian-spell.json");
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
