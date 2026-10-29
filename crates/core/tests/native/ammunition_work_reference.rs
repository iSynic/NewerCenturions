//! Independent native item/ammo/weapon work record comparisons.
use super::*;
use crate::test_support::{bytes, content, fixture};

pub(super) fn compare_npc_preparation() {
    let content = content();
    let fixture = fixture("final-enemy-weapons.json");
    let cases = fixture["preparation_cases"].as_array().unwrap();
    for (index, case) in cases.iter().enumerate() {
        let input = bytes(&case["enemy_before"]);
        let weapon = word(&input, 0x26).unwrap();
        let ammo = if case["context"].as_i64().unwrap() == 0 {
            0
        } else {
            word(&input, 0x4c).unwrap()
        };
        let retained = definition(&content, 700).unwrap();
        let mut work = WorkBuffers {
            item: Some(retained.clone()),
            ..Default::default()
        };
        work.prepare_pair(
            &content.items[weapon.unsigned_abs() as usize],
            &content.items[ammo.unsigned_abs() as usize],
        )
        .unwrap();
        assert_eq!(
            work.weapon,
            Some(bytes(&case["weapon"])),
            "weapon case {index}"
        );
        assert_eq!(work.ammo, Some(bytes(&case["ammo"])), "ammo case {index}");
        assert_eq!(work.item, Some(retained), "retained item case {index}");
    }
}

#[test]
fn complete_npc_preparation_records_match_original_instructions() {
    compare_npc_preparation();
}
