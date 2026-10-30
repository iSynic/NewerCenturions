use crate::{prelude::*, support};

#[test]
fn field_timers_and_usage_reset_match_original_complete_buffers() {
    for (index, case) in support::fixture("spell-fields.json")["ticks"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let fields = case.bytes("fields_before");
        let used = case.bytes("used_before");
        let cursor = case.n::<i16>("cursor_before");
        support::expect(cursor.wrapping_add(1), &case["cursor_after"]);
        let actual = spells::tick_fields(&fields, cursor.wrapping_add(1)).unwrap();
        support::native_expect! {
            bytes &actual => &case.bytes("fields_after"), &format!("field tick {index}");
        };
        let actual = spells::reset_used(&used).unwrap();
        support::native_expect! {
            bytes &actual => &case.bytes("used_after"), &format!("used reset {index}");
        };
    }
}

#[test]
fn complete_collisions_match_original_all_bodies_fields_usage_and_randomness() {
    let fixture = support::fixture("spell-fields.json");
    let mission = OpeningMission {
        shared: std::sync::Arc::new(centurions_content::mission::StoryTables {
            areas: fixture.bytes("areas"),
            combat: Some(centurions_content::combat::CombatContent {
                attacks: data("Data S"),
                encounters: vec![],
                battles: vec![],
                monsters: vec![],
                race_moves: vec![],
            }),
            ..Default::default()
        }),
        ..support::mission()
    };
    let rotated = support::fixture("rotated-fields.json");
    let mut cases: Vec<_> = fixture
        .array("cases")
        .iter()
        .chain(rotated.array("cases"))
        .cloned()
        .collect();
    // The new native editor prefix proves the registered backup remains zero.
    // Restrict an otherwise unchanged body10 collision to exercise that owner.
    let mut registered = fixture["cases"][1].clone();
    registered["registered_backup_control"] = true.into();
    cases.push(registered);
    for case in &cases {
        let mut mission = mission.clone();
        if case["registered_backup_control"] == true {
            let backup = support::fixture("editor-backup-boundary.json");
            assert!(
                backup["cases"][1]
                    .bytes("after")
                    .iter()
                    .all(|byte| *byte == 0)
            );
            let tables = std::sync::Arc::make_mut(&mut mission.shared);
            let attacks = &mut tables.combat.as_mut().unwrap().attacks;
            assert_eq!(attacks[2842 + 49], 0);
            attacks[2842 + 50..2842 + 52].copy_from_slice(&0x2000_i16.to_be_bytes());
        }
        let mut warriors = support::warriors(&case["warriors_before"]);
        let mut combat = support::battle();
        combat.battle = 28;
        combat.field.tiles = fixture.words("map");
        combat.squad.pos = (0..6).map(|i| (20 + i, 20)).collect();
        combat.roster.records = support::monsters(&case["enemies_before"]);
        combat.roster.mon_pos = vec![(20, 22), (22, 22)];
        combat.battle_order = vec![-1, 10];
        combat.spell_queue = case.bytes("fields_before");
        combat.collide_check = case.bytes("used_before");
        if case.get("armor").and_then(Value::as_i64) == Some(1) {
            combat
                .weapon_traits
                .load_weapon(&content().items[130])
                .unwrap();
        }
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let mut count = 6;
        let events = runtime::collide_fields(
            &mut combat,
            &mut warriors,
            &mission,
            &mut count,
            &mut random,
            (case.n::<i16>("body"), 0, &mut 0),
        )
        .unwrap();
        support::native_expect! {
            records &warriors => &case["warriors_after"], "collision party";
            records &combat.roster.records => &case["enemies_after"], "collision enemies";
            bytes combat.spell_queue => &case["fields_after"], "collision fields";
            bytes combat.collide_check => &case["used_after"], "collision usage";
            scalar count => &case["count_after"], "collision count";
            scalar combat.cast_identity[1] => &case["level_after"], "collision level";
            rng random => (&case["seed_after"], &case["rng_draws"]), "collision seed";
            scalar support::event_sounds(&events) => &case["sounds"], "collision ordered sounds";
        }
    }
}
