use crate::{prelude::*, support};

fn state(f: &Value, c: &Value) -> Session {
    let mut s: Session = serde_json::from_value(f["foundation"].clone()).unwrap();
    s.warriors = support::warriors(&c["party_before"]);
    let combat = s.combat.as_mut().unwrap();
    combat.roster.records = support::monsters(&c["records_before"]);
    s.enemy_spell_disabled = u8::from(c["variant"] == 6);
    // The probe writes447196=5; this full command also supplies Getup's zero WORDs.
    crate::native_common::monster_command_input(&mut s, c.n::<usize>("actor") - 10, [0, 0]);
    s
}
#[test]
fn blocked_first_spell_chance_matches_six_complete_native_enemy_turns() {
    let f = support::fixture("reached-guardian-spell.json");
    let content = content();
    let mission = content.map(2).unwrap();
    for case in f.array("cases") {
        if case["variant"] == 1 {
            continue;
        }
        let mut s = state(&f, case);
        let combat = s.combat.as_mut().unwrap();
        let mut rng = RandomState::new(case.n::<u32>("seed_before"));
        let mut work = EnemyGlobals {
            item_context: s.globals.loaded_item_field32,
            attack_count: s.globals.attack_count,
            packet: combat.spell_info.clone(),
            identity: combat.cast_identity,
            spell_disabled: s.enemy_spell_disabled,
            ..EnemyGlobals::default()
        };
        let events = enemy_action_with_globals(
            combat,
            &mut s.warriors,
            mission,
            &mut rng,
            &content.items,
            &mut work,
        )
        .unwrap();
        let context = format!("variant{}", case["variant"]);
        support::native_expect! {
            bytes &support::record_bytes(&s.warriors) => &case.bytes("party_after"), &format!("{context} party");
            bytes &support::record_bytes(&combat.roster.records) => &case.bytes("records_after"), &format!("{context} enemies");
        };
        assert_eq!(
            combat.field.tiles,
            words(&case.bytes("map_after")),
            "{context} map"
        );
        assert_eq!(
            combat.battle_order,
            words(&case.bytes("order_after")),
            "{context} order"
        );
        support::native_expect! {
            positions combat.roster.mon_pos => &case.bytes("positions_after"), "{context} positions";
        };
        assert_eq!(combat.spell_queue, case.bytes("fields_after"));
        assert_eq!(combat.collide_check, case.bytes("used_after"));
        assert_eq!(
            combat.spell_info,
            case.bytes("packet_after"),
            "{context} packet"
        );
        assert_eq!(work.item_context, case.n::<i16>("context_after"));
        support::native_expect! {
            scalar work.attack_count => &case["attack_count_after"], "binary_enemy_spell_guards";
            rng rng => (&case["seed_after"], &case["rng_draws"]), "binary_enemy_spell_guards";
        }
        let sounds: Vec<_> = support::event_sounds(&events);
        assert_eq!(serde_json::json!(sounds), case["sounds"], "{context} audio");
    }
}
#[test]
fn reached_disabled_spell_turn_resumes() {
    let f = support::fixture("reached-guardian-spell.json");
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
    std::fs::remove_file(path).unwrap();
}
