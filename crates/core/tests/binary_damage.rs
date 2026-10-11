use centurions_core::{RandomState, combat::damage::*};
fn decode(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn damage_matches_complete_original_routine() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/damage.json")).unwrap();
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let mut target = decode(case["input"].as_str().unwrap());
        let flags = case["flags"].as_u64().unwrap();
        let mut random = RandomState::new(case["initial_seed"].as_u64().unwrap() as u32);
        let result = resolve_damage(
            &mut target,
            case["monster"].as_bool().unwrap(),
            &DamagePacket {
                bytes: decode(case["packet"].as_str().unwrap()),
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
        assert_eq!(
            target,
            decode(case["after"].as_str().unwrap()),
            "target case {index}"
        );
        assert_eq!(
            result.total,
            i16::try_from(case["total"].as_i64().unwrap()).unwrap(),
            "damage case {index}"
        );
        assert_eq!(
            random.seed,
            case["seed"].as_u64().unwrap() as u32,
            "seed case {index}"
        );
        assert_eq!(
            random.draws,
            case["draws"].as_u64().unwrap(),
            "draws case {index}"
        );
        if result.total >= 0 {
            assert_eq!(
                result.sound,
                case["sound"].as_i64().unwrap() as i16,
                "sound case {index}"
            );
        }
        let shield_sounds = if result.shield_sound {
            vec![206]
        } else {
            vec![]
        };
        let expected_sounds: Vec<i16> = case["sounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|sound| sound.as_i64().unwrap() as i16)
            .collect();
        assert_eq!(shield_sounds, expected_sounds, "shield sound case {index}");
    }
}
