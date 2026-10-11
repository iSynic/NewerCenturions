use centurions_content::{Warrior, word};
use centurions_core::story_skill::{SkillPlan, predicate};

fn bytes(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).unwrap())
        .collect()
}

#[test]
fn chosen_warrior_checks_match_original_signed_comparisons_and_dispatch() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/story-skill.json")).unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let extra = bytes(case["extra"].as_str().unwrap());
        let raw = bytes(case["warrior"].as_str().unwrap());
        let warriors = vec![
            Warrior {
                name: "Coriand".into(),
                original_sha256: String::new(),
                bytes: raw,
            };
            6
        ];
        let mask_raw = bytes(case["selected_mask"].as_str().unwrap());
        let selected = std::array::from_fn(|index| word(&mask_raw, index * 2).unwrap());
        let before = warriors.clone();
        let plan = predicate(&extra, &warriors, &selected).unwrap();
        let branch = case["branch"].as_i64().unwrap();
        if branch == -1 {
            assert_eq!(plan, SkillPlan::Continue);
            assert_eq!(case["cursor_after"], 3);
        } else {
            assert_eq!(
                plan,
                SkillPlan::ReplaceScript {
                    warrior: case["selected_index"].as_u64().unwrap() as usize,
                    success: branch == 299,
                    script: branch as i16,
                }
            );
            assert_eq!(case["cursor_after"], -1);
        }
        assert_eq!(warriors, before);
        assert_eq!(case["mask_after"], case["selected_mask"]);
        assert_eq!(case["party_unchanged"], true);
        assert_eq!(case["choice_calls"], 1);
        assert_eq!(case["choice_zero"], 0);
        assert_eq!(case["choice_mode"], 0);
        assert!(predicate(&extra, &warriors, &[0; 6]).is_err());
    }
    assert_eq!(fixture["rng_draws"], 0);
}
