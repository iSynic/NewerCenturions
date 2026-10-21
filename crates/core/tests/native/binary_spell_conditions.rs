use crate::{prelude::*, support};

use support::patch_bytes as patches;

#[test]
fn effect_36_matches_original_all_body_slots_and_ordered_random_consumption() {
    let fixture: serde_json::Value = support::fixture("spell-conditions.json");
    let party = fixture.bytes("party");
    let monsters = fixture.bytes("monsters");
    let packet = fixture.bytes("packet");
    for case in fixture.array("cases") {
        let mut before = party.clone();
        let mut records = monsters.clone();
        patches(&mut before, &case["party_input"]);
        patches(&mut records, &case["monster_input"]);
        let warriors = support::warriors(&before);
        let enemies = support::monsters(&records);
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result = spell_conditions::resolve_1703(
            &warriors,
            &enemies,
            &packet,
            case.n::<i16>("count"),
            &case.n::<Vec<i16>>("mask"),
            &mut random,
        )
        .unwrap();
        let mut expected = before;
        patches(&mut expected, &case["party_changes"]);
        support::native_expect! {
            bytes &support::record_bytes(&result.warriors) => &expected, "party";
        };
        let mut expected = records;
        patches(&mut expected, &case["monster_changes"]);
        support::native_expect! {
            bytes &support::record_bytes(&result.enemies) => &expected, "roster";
            scalar result.duration => &case["duration"], "binary_spell_conditions";
            rng random => (&case["seed_after"], &case["rng_draws"]), "binary_spell_conditions";
            scalar (result.sounds) => &case["sounds"], "binary_spell_conditions";
        }
    }
}
