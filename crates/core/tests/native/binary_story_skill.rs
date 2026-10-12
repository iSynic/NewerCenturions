use crate::prelude::*;

use centurions_core::story_skill::{SkillPlan, predicate};

#[test]
fn chosen_warrior_checks_match_original_signed_comparisons_and_dispatch() {
    let fixture: serde_json::Value = crate::support::fixture("story-skill.json");
    for case in fixture.array("cases") {
        let extra = case.bytes("extra");
        let raw = case.bytes("warrior");
        let warriors = vec![
            Warrior {
                name: "Coriand".into(),
                original_sha256: String::new(),
                bytes: raw,
            };
            6
        ];
        let mask_raw = case.bytes("selected_mask");
        let selected = std::array::from_fn(|index| word(&mask_raw, index * 2).unwrap());
        let before = warriors.clone();
        let plan = predicate(&extra, &warriors, &selected).unwrap();
        let branch = case.n::<i64>("branch");
        if branch == -1 {
            assert_eq!(plan, SkillPlan::Continue);
        } else {
            assert_eq!(
                plan,
                SkillPlan::ReplaceScript {
                    warrior: case.n::<usize>("selected_index"),
                    success: branch == 299,
                    script: branch as i16,
                }
            );
        }
        assert_eq!(warriors, before);

        assert!(predicate(&extra, &warriors, &[0; 6]).is_err());
    }
}
