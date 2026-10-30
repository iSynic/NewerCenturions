//! Party-wide equipment WORDs apply in order; an unknown prior value never becomes zero.
use crate::{prelude::*, support};

#[test]
fn equipment_globals_apply_known_words_and_reject_unknown_priors_atomically() {
    for (selector, removing) in [(16_i16, false), (30, true), (40, false), (57, false)] {
        let mut content = content();
        let mut state = crate::native_common::loaded_shot_frame(0);
        let command = if removing {
            let select = Command::SelectCombatWeapon { slot: 3 };
            support::saved_command(select, &mut state, &content);
            let items = state.warriors[4].inventory().unwrap();
            let slot = items.iter().position(|i| i.id() == 1).unwrap();
            Command::UnequipItem { warrior: 4, slot }
        } else {
            Command::SelectCombatWeapon { slot: 3 }
        };
        let amount = if removing { 0_i16 } else { 1 };
        content.items[1][46..48].copy_from_slice(&selector.to_be_bytes());
        content.items[1][50..52].copy_from_slice(&amount.to_be_bytes());
        let before = state.clone();
        let result = apply(command, &mut state, &content);
        let flag = |s: &Session| i16::from_be_bytes([s.flags[12], s.flags[13]]);
        let restored = |s: &Session| support::checkpoint(s, &content, "equipment-globals");
        match selector {
            40 => assert_eq!(state.difficulty, before.difficulty.wrapping_sub(1)),
            // Selector 30 is party duration 0; removal clears its owner directly.
            30 => assert_eq!(restored(&state).party_condition[..2], [0, 0]),
            // WORD442598 is story flag BYTEs 12..13.
            57 => assert_eq!(flag(&state), flag(&before).wrapping_sub(1)),
            _ => assert!(result.is_err() && state == before),
        }
    }
}
