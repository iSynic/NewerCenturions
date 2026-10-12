use crate::prelude::*;

use centurions_core::combat::damage::*;

#[test]
fn damage_matches_complete_original_routine() {
    let fixture: serde_json::Value = crate::support::fixture("damage.json");
    for (index, case) in fixture.array("cases").iter().enumerate() {
        let mut target = case.bytes("input");
        let flags = case.n::<u64>("flags");
        let mut random = RandomState::new(case.n::<u32>("initial_seed"));
        let result = resolve_damage(
            &mut target,
            case.n::<bool>("monster"),
            &DamagePacket {
                bytes: case.bytes("packet"),
                attack_count: 2,
            },
            DamageFlags {
                shield: flags & 1 != 0,
                armor: flags & 2 != 0,
                saves: flags & 4 != 0,
                armor_filter: flags & 8 != 0,
            },
            &mut random,
        )
        .unwrap();
        assert_eq!(target, case.bytes("after"), "target case {index}");
        assert_eq!(
            result.total,
            i16::try_from(case.n::<i64>("total")).unwrap(),
            "damage case {index}"
        );
        crate::support::native_expect! {
        scalar random.seed => &case["seed"], "binary_damage";
        scalar random.draws => &case["draws"], "binary_damage";
        }
        if result.total >= 0 {
            crate::support::expect(result.sound, &case["sound"]);
        }
        let shield_sounds = if result.shield_sound {
            vec![206]
        } else {
            vec![]
        };
        let expected_sounds: Vec<i16> = case.n::<Vec<i16>>("sounds");
        assert_eq!(shield_sounds, expected_sounds, "shield sound case {index}");
    }
}
