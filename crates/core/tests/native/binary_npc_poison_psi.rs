use crate::prelude::*;
use centurions_core::combat::spells::RestrictionMemoryView;

#[test]
fn all_164_native_npc_poison_resolutions_match_complete_records_rng_and_audio() {
    let fixture = crate::support::fixture("npc-poison-psi.json");

    let mut families = [0; 4];
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let before = &case["before"];
        let after = &case["after"];
        let read = |snapshot: &Value, key: &str| crate::support::buffer(&fixture, &snapshot[key]);
        let party = crate::support::warriors(&read(before, "party"));
        let npcs = crate::support::monsters(&read(before, "npcs"));
        let items = read(before, "item_cache0");
        let backup = read(before, "backup_warriors");
        let memory = if case["editor_control"] == true {
            RestrictionMemoryView::with_editor_backup(&items, None, &backup).unwrap()
        } else {
            RestrictionMemoryView::ordinary_startup(&items, None)
        };
        let packet = read(before, "packet");
        let mut random = crate::native_common::random_state(before);
        let result = spells::resolve_bodies_with_memory(
            &party,
            &npcs,
            &crate::support::words(&read(before, "mask")),
            &DirectSpell {
                packet: &packet,
                level: crate::support::words(&read(before, "identity"))[1],
                count: case.n::<i16>("count"),
                armor_filter: false,
                weapon_sound: word(&read(before, "weapon_work"), 14).unwrap(),
            },
            memory,
            &mut random,
        )
        .unwrap();
        assert_eq!(
            crate::support::record_bytes(&result.warriors),
            read(after, "party"),
            "case {index} party"
        );
        assert_eq!(
            crate::support::record_bytes(&result.enemies),
            read(after, "npcs"),
            "case {index} NPCs"
        );
        assert_eq!(random, crate::native_common::random_state(after));
        assert_eq!(
            crate::native_common::body_spell_sounds(&result),
            case.n::<Vec<i16>>("sounds"),
            "case {index} ordered effects"
        );
        families[match case.text("family") {
            "ordinary" => 0,
            "exact_limit" => 1,
            "resistance" => 2,
            "editor_owner" => 3,
            _ => panic!("unexpected native family"),
        }] += 1;
    }
    assert_eq!(families, [96, 24, 40, 4]);
}
