//! Command/save/replay coverage for source-derived item activation variants.
use crate::command_support_psi::outdoor;
use crate::{prelude::*, support};
use std::fs;

#[test]
fn source_item_activation_preserves_ranked_choices_owner_targets_and_unlimited_charges() {
    let case = &support::fixture("outdoor-medical.json")["cases"][0];
    for (mode, power, quantity, in_spell) in [
        (0, 0_i16, 3, 0),
        (0, 0, 3, -1),
        (0, 0, 3, 1),
        (0, 2, 3, 0),
        (0, 9, 3, 0),
        (0, i16::MIN, 3, 0),
        (5, 0, 3, 0),
        (3, 1, 3, 0),
        (255, i16::MIN, -1, 0),
        (2, 8, 3, 0),
    ] {
        let mut content = support::content();
        let item = case.n::<usize>("item");
        content.items[item][26..28].copy_from_slice(&24_i16.to_be_bytes());
        content.items[item][42..44].copy_from_slice(&power.to_be_bytes());
        if quantity < 0 {
            content.items[item][20..22].copy_from_slice(&(-1_i16).to_be_bytes());
        }
        let tables =
            std::sync::Arc::make_mut(&mut content.scenario.as_mut().unwrap().maps[0].shared);
        tables.combat.as_mut().unwrap().attacks[3 * 406 + 1] = mode;
        let mut state = outdoor(&content, case);
        state.globals.in_spell = in_spell;
        let actor = case.n::<usize>("actor");
        let slot = case.n::<usize>("slot");
        state.warriors[actor]
            .record_mut()
            .set_inventory_quantity(slot, quantity);
        for warrior in &mut state.warriors {
            warrior.record_mut().set_stamina(1);
        }
        if in_spell == 1 {
            support::saved_command(
                Command::CastPartyPsi {
                    warrior: 5,
                    spell: 1101,
                    power: 1,
                },
                &mut state,
                &content,
            );
            let paid = state.warriors.clone();
            let mut random = state.random.clone();
            random.bound(1);
            let events = support::saved_command(Command::FinishOutdoorPsi, &mut state, &content);
            assert_eq!(state.warriors, paid);
            assert_eq!(state.random, random);
            assert_eq!(support::event_sounds(&events), [-644]);
            // buttonchoice clears Ready; restore the item's independently declared input.
            assert_eq!(std::mem::replace(&mut state.globals.in_spell, 1), 0);
            let memory = &state.globals.define_spells;
            assert_eq!(&memory[472..480], &support::word_bytes(&[0, 0, 0, 1]));
        }
        let draws = state.random.draws;
        let activation = Command::UseItem {
            warrior: actor,
            slot,
        };
        let mut events = support::saved_command(activation, &mut state, &content);
        if state.outdoor_psi.is_some() {
            assert_eq!(state.random.draws - draws, u64::from(power == 8));
            if mode == 0 && power == 0 {
                for warrior in [0, 0, 1, 0] {
                    let command = Command::SelectOutdoorPsiTarget { warrior };
                    support::saved_command(command, &mut state, &content);
                }
                assert_eq!(&state.globals.party_track[..2], &[0, -1]);
                assert_eq!(state.outdoor_psi.as_ref().unwrap().remaining, -2);
                let path = std::env::temp_dir()
                    .join(format!("centurions-zero-ranks-{}.json", std::process::id()));
                save(&path, &state, &content).unwrap();
                let mut old: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                old["session"]["party_mask"][0] = (-1).into();
                fs::write(&path, serde_json::to_vec(&old).unwrap()).unwrap();
                assert!(load(&path, &content).is_err());
                fs::remove_file(path).unwrap();
                let before = state.clone();
                if in_spell == 0 {
                    assert!(apply(Command::FinishOutdoorPsi, &mut state, &content).is_err());
                    assert_eq!(state, before);
                }
                events.extend(support::saved_command(
                    if in_spell != 0 {
                        Command::FinishOutdoorPsi
                    } else {
                        Command::CancelOutdoorPsi
                    },
                    &mut state,
                    &content,
                ));
                assert_eq!(state.warriors, before.warriors);
                assert_eq!(state.random.draws, before.random.draws + 1);
            } else {
                for warrior in 0..if mode == 0 { 2 } else { 1 } {
                    let command = Command::SelectOutdoorPsiTarget { warrior };
                    events.extend(support::saved_command(command, &mut state, &content));
                }
            }
        }
        let targets: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::SpellResolved { target, .. } => Some(*target),
                _ => None,
            })
            .collect();
        let expected = match mode {
            0 if power == 0 => vec![],
            0 if power == 2 => vec![0, 1],
            0 | 3 => (0..state.warriors.len() as i16).collect(),
            5 => vec![actor as i16],
            _ => vec![0],
        };
        assert_eq!(targets, expected);
        assert!(state.outdoor_psi.is_none());
        assert_eq!(state.globals.in_spell, 0);
        let bag = state.warriors[actor].inventory().unwrap();
        let held = bag.iter().find(|row| row.id() == item as i16).unwrap();
        assert_eq!(
            held.charge().unwrap(),
            if quantity < 0 { -1 } else { quantity - 1 }
        );
    }
}
