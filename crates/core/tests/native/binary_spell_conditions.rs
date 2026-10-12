use crate::prelude::*;

fn patches(bytes: &mut [u8], values: &serde_json::Value) {
    for change in values.as_array().unwrap() {
        bytes[change[0].as_u64().unwrap() as usize] = change[1].as_u64().unwrap() as u8;
    }
}

fn assert_bytes(actual: &[u8], expected: &[u8], case: usize, name: &str) {
    assert_eq!(actual.len(), expected.len());
    if let Some(index) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "case {case} {name} byte {index}: {} != {}",
            actual[index], expected[index]
        );
    }
}

#[test]
fn effect_36_matches_original_all_body_slots_and_ordered_random_consumption() {
    let fixture: serde_json::Value = crate::support::fixture("spell-conditions.json");
    let party = fixture.bytes("party");
    let monsters = fixture.bytes("monsters");
    let packet = fixture.bytes("packet");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let mut before = party.clone();
        let mut records = monsters.clone();
        patches(&mut before, &case["party_input"]);
        patches(&mut records, &case["monster_input"]);
        let warriors = crate::support::warriors(&before);
        let enemies = crate::support::monsters(&records);
        let mask = case.n::<Vec<i16>>("mask");
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result = spell_conditions::resolve_1703(
            &warriors,
            &enemies,
            &packet,
            case.n::<i16>("count"),
            &mask,
            &mut random,
        )
        .unwrap();
        let mut expected = before;
        patches(&mut expected, &case["party_changes"]);
        assert_bytes(
            &crate::support::record_bytes(&result.warriors),
            &expected,
            index,
            "party",
        );
        let mut expected = records;
        patches(&mut expected, &case["monster_changes"]);
        assert_bytes(
            &crate::support::record_bytes(&result.enemies),
            &expected,
            index,
            "roster",
        );
        crate::support::native_expect! {
        scalar result.duration => &case["duration"], "binary_spell_conditions";
        scalar random.seed => &case["seed_after"], "binary_spell_conditions";
        scalar random.draws => &case["rng_draws"], "binary_spell_conditions";
        scalar (result.sounds) => &case["sounds"], "binary_spell_conditions";
        }
    }
}
