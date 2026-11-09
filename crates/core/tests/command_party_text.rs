//! Names in story text, and Squad Order while the party's equipment is held.
use crate::{prelude::*, support};
use centurions_core::{confiscation::ConfiscatedEquipment, story::text::substitute_names};

#[test]
fn name_tokens_give_way_to_the_last_member_standing() {
    let content = content();
    let mut party = support::mission_session(&content).warriors[..3].to_vec();
    for (warrior, name) in party.iter_mut().zip(["Ann", "Bo", "Cy"]) {
        warrior.name = name.into();
    }
    party[1].record_mut().set_stamina(0);
    let fill = |party: &[Warrior], text| substitute_names(text, party).unwrap();
    assert_eq!(fill(&party, "^1 ^2 ^3 ^5 ^7 end^"), "Ann Cy Cy Cy Cy end^");
    // With nobody standing the token names its own place, and an unknown token the first.
    for warrior in &mut party {
        warrior.record_mut().set_stamina(0);
    }
    assert_eq!(fill(&party, "^3 ^2 ^7"), "Cy Bo Ann");
}

#[test]
fn squad_order_is_refused_while_equipment_is_held() {
    let content = content();
    let mut state = support::mission_session(&content);
    let exploration = state.exploration.as_mut().unwrap();
    (exploration.presentation, exploration.script) = (None, None);
    state.confiscated = Some(ConfiscatedEquipment {
        active: true,
        money: 0,
        items: vec![InventoryRecord([0; 22]); 600],
    });
    let names: Vec<_> = state.warriors.iter().map(|w| w.name.clone()).collect();
    let destinations = (0..names.len()).rev().collect();
    let refused = apply(Command::ReorderSquad { destinations }, &mut state, &content);
    assert!(matches!(refused, Err(RuleError::Refused { ref feedback }) if feedback.len() == 1));
    assert!(state.warriors.iter().map(|w| &w.name).eq(names.iter()));
}
