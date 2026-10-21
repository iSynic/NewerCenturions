use crate::{prelude::*, support};

use centurions_core::story::{
    ConditionalJump, add_mission_score, flag_condition_jump, set_mission_flag,
};

#[test]
fn mission_flag_mutations_match_complete_original_flag_table() {
    for case in support::fixture("story-flags.json")["set_cases"]
        .as_array()
        .unwrap()
    {
        let flags = case.bytes("before");
        let parameter = case.n::<i16>("parameter");
        assert_eq!(
            set_mission_flag(&flags, parameter).unwrap(),
            case.bytes("after")
        );
        assert!(set_mission_flag(&flags, i16::MIN).is_err());
    }
}

#[test]
fn mission_flag_condition_matches_original_branch_decision() {
    for case in support::fixture("story-flags.json")["condition_cases"]
        .as_array()
        .unwrap()
    {
        let mut flags = vec![0; 128];
        let index = case.n::<i16>("index");
        flags[index as usize] = case.n::<u8>("flag");
        let extra: Vec<u8> = [index, case.n::<i16>("mode"), 3, 0, 0]
            .into_iter()
            .flat_map(i16::to_be_bytes)
            .collect();
        assert_eq!(
            flag_condition_jump(&flags, &extra).unwrap(),
            case.n::<bool>("matched")
                .then_some(ConditionalJump::EndMenu)
        );
    }
}

#[test]
fn mission_score_addition_matches_original_word_arithmetic() {
    for case in support::fixture("story-flags.json")["score_cases"]
        .as_array()
        .unwrap()
    {
        support::expect(
            add_mission_score(case.n::<i16>("before"), case.n::<i16>("parameter")),
            &case["after"],
        );
    }
}
