use crate::prelude::*;
use crate::support::assert_bytes;

#[test]
fn field_timers_and_usage_reset_match_original_complete_buffers() {
    for (index, case) in crate::support::fixture("spell-fields.json")["ticks"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let fields = case.bytes("fields_before");
        let used = case.bytes("used_before");
        let cursor = case.n::<i16>("cursor_before");
        crate::support::expect(cursor.wrapping_add(1), &case["cursor_after"]);
        let actual = spells::tick_fields(&fields, cursor.wrapping_add(1)).unwrap();
        assert_bytes(
            &actual,
            &case.bytes("fields_after"),
            &format!("field tick {index}"),
        );
        let actual = spells::reset_used(&used).unwrap();
        assert_bytes(
            &actual,
            &case.bytes("used_after"),
            &format!("used reset {index}"),
        );
    }
}

#[test]
fn complete_collisions_match_original_all_bodies_fields_usage_and_randomness() {
    let fixture = crate::support::fixture("spell-fields.json");
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
        ..crate::support::mission()
    };
    let rotated = crate::support::fixture("rotated-fields.json");
    for case in fixture.array("cases").iter().chain(rotated.array("cases")) {
        let mut warriors = crate::support::warriors(&case["warriors_before"]);
        let mut combat = crate::support::battle();
        combat.battle = 28;
        combat.map.tiles = fixture.words("map");
        combat.squad.positions = (0..6).map(|i| (20 + i, 20)).collect();
        combat.enemies.records = crate::support::monsters(&case["enemies_before"]);
        combat.enemies.positions = vec![(20, 22), (22, 22)];
        combat.order = vec![-1, 10];
        combat.spell_fields = case.bytes("fields_before");
        combat.field_used = case.bytes("used_before");
        if case.get("armor").and_then(Value::as_i64) == Some(1) {
            combat
                .weapon_work
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
            case.n::<i16>("body"),
        )
        .unwrap();
        crate::support::native_expect! {
            bytes crate::support::record_bytes(&warriors) => &case["warriors_after"], "collision party";
            bytes crate::support::record_bytes(&combat.enemies.records) => &case["enemies_after"], "collision enemies";
            bytes combat.spell_fields => &case["fields_after"], "collision fields";
            bytes combat.field_used => &case["used_after"], "collision usage";
            scalar count => &case["count_after"], "collision count";
            scalar combat.attack_identity.unwrap()[1] => &case["level_after"], "collision level";
            scalar random.seed => &case["seed_after"], "collision seed";
            scalar random.draws => &case["rng_draws"], "collision draws";
            scalar crate::support::event_sounds(&events) => &case["sounds"], "collision ordered sounds";
        }
    }
}
