use crate::{native_common, prelude::*, support};

#[test]
fn spell_resolution_matches_original_ordered_party_and_monster_records() {
    let fixture: serde_json::Value = support::fixture("spell-bodies.json");
    assert_resolution(&fixture);
}

#[test]
fn additional_spells_match_original_ordered_party_and_monster_records() {
    let fixture: serde_json::Value = support::fixture("additional-spells.json");
    assert_resolution(&fixture);
}

fn assert_resolution(fixture: &serde_json::Value) {
    for case in fixture.array("cases") {
        let warriors = support::warriors(&case.bytes("warriors_before"));
        let enemies = support::monsters(&case.bytes("enemies_before"));
        let mask = support::words(&case.bytes("mask"));
        let packet = case.bytes("packet");
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result = resolve_bodies(
            &warriors,
            &enemies,
            &mask,
            &DirectSpell {
                packet: &packet,
                level: (case.n::<i64>("spell_id") % 1000 / 100 - 1) as i16,
                count: case.n::<i16>("count"),
                armor_filter: false,
                weapon_sound: 0,
            },
            &mut random,
        )
        .unwrap();
        native_common::assert_body_records(
            &result.warriors,
            &result.enemies,
            &case["warriors_after"],
            &case["enemies_after"],
        );
        support::native_expect! {
            rng random => (&case["seed_after"], &case["rng_draws"]), "binary_spell_bodies";
            scalar native_common::body_spell_sounds(&result) => &case["sounds"], "binary_spell_bodies";
        }
    }
}

#[test]
fn enemy_firearms_match_original_scientist_records_sounds_and_randomness() {
    use centurions_core::combat::{firearm::PreparedFirearm, spells::resolve_enemy_firearm_body};
    let fixture: serde_json::Value = support::fixture("enemy-firearm-bodies.json");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let caster = MonsterRecord {
            bytes: case.bytes("caster_before"),
        };
        let mut target = MonsterRecord {
            bytes: case.bytes("target_before"),
        };
        let prepared = PreparedFirearm {
            packet: case.bytes("packet"),
            shots: case.n::<i16>("shots"),
            sound: case.n::<i16>("weapon_sound"),
            slot: 0,
            rounds: 0,
            weight: 0,
            armor_filter: false,
        };

        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result =
            resolve_enemy_firearm_body(&mut target, &caster, &prepared, 0, &mut random).unwrap();
        support::native_expect! {
            bytes target.bytes => &case["target_after"], &format!("firearm case {index}");
            bytes caster.bytes => &case["caster_after"], &format!("firearm caster {index}");
            rng random => (&case["seed_after"], &case["rng_draws"]), "binary_spell_bodies";
            scalar (result.sounds) => &case["sounds"], "binary_spell_bodies";
        }
    }
}

#[test]
fn global_spell_masks_match_original_faction_and_living_body_rules() {
    use centurions_core::combat::spells::global_targets;
    let fixture: serde_json::Value = support::fixture("additional-spells.json");
    for (index, case) in fixture.array("mask_cases").iter().enumerate() {
        let warriors = support::warriors(&case.bytes("warriors"));
        let enemies = support::monsters(&case.bytes("enemies_before"));
        let mut combat = support::battle();
        combat.map.tiles.clear();
        combat.squad.positions.clear();
        combat.squad.under_tiles.clear();
        combat.enemies.records = enemies;
        combat.order.clear();
        combat.cursor = 0;
        let packet = case.bytes("packet");
        let result = global_targets(&combat, &warriors, 0, &packet).unwrap();
        let actual = support::word_bytes(&result.mask);
        support::native_expect! {
            bytes &actual => &case.bytes("mask"), &format!("global mask {index}");
        };
        let mut actual = combat.enemies.records.clone();
        actual[0] = result.caster;
        let actual = support::record_bytes(&actual);
        support::native_expect! {
            bytes &actual => &case.bytes("enemies_after"), &format!("global records {index}");
        };
    }
}
