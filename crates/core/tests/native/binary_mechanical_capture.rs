use crate::{prelude::*, support};
use centurions_core::{Event, feedback::NativeFeedback};

#[test]
fn mechanical_transactions_match_original_records_random_and_ordered_feedback() {
    let fixture = support::fixture("mechanical-capture.json");
    for case in fixture.array("cases") {
        let mut warrior = Warrior {
            name: "Native reference".into(),
            original_sha256: String::new(),
            bytes: case.bytes("warrior_before"),
        };
        let mut enemy = centurions_content::combat::MonsterRecord {
            bytes: case.bytes("enemy_before"),
        };
        let mut random = RandomState::new(case.n("seed_before"));
        let effects =
            centurions_core::combat::capture::resolve(&mut warrior, &mut enemy, &mut random)
                .unwrap();
        assert_eq!(
            warrior.bytes,
            case.bytes("warrior_after"),
            "{}",
            case.text("id")
        );
        assert_eq!(
            enemy.bytes,
            case.bytes("enemy_after"),
            "{}",
            case.text("id")
        );
        assert_eq!(
            random,
            crate::native_common::random_at(case, "seed_after", "draws")
        );
        let actual: Vec<String> = effects
            .iter()
            .map(|effect| match effect {
                Event::Sound { resource_id } => format!("sound:{resource_id}"),
                Event::Feedback(NativeFeedback::Warning { index }) => format!("warning:{index}"),
                Event::Feedback(NativeFeedback::Flash {
                    address: 0x40250d, ..
                }) => "exit".into(),
                Event::Feedback(NativeFeedback::Flash { address, .. }) => {
                    format!("flash:{address:x}")
                }
                _ => panic!("unexpected capture effect"),
            })
            .collect();
        assert_eq!(serde_json::to_value(actual).unwrap(), case["effects"]);
    }
}
