//! The shop window tidies and weighs the shopper, and wearing refuses in the original order.
use crate::{prelude::*, support};

#[test]
fn shop_tidies_and_weighs_the_shopper_and_wear_refuses_in_the_original_order() {
    let mut content = support::content();
    let mut state = support::mission_session(&content);
    let access = centurions_core::shop::Access {
        level: 0,
        id: 0,
        enhancement: 0,
        remaining: 0,
    };
    state.shop_access = Some(access);
    // The worn firearm sits behind the unworn items and the stored load has drifted.
    let load = state.warriors[0].record().load().unwrap();
    let (worn, last) = (
        state.warriors[0].record().inventory_slot(0),
        state.warriors[0].record().inventory_slot(2),
    );
    let mut pack = state.warriors[0].record_mut();
    pack.set_inventory_slot(0, &last);
    pack.set_inventory_slot(2, &worn);
    pack.set_load(0);
    apply(Command::OpenShop, &mut state, &content).unwrap();
    assert_eq!(state.warriors[0].inventory().unwrap()[0], worn);
    assert_eq!(state.warriors[0].record().load().unwrap(), load);
    // A class above 23 is refused before the level; below that the level refusal sounds.
    let mut item = state.warriors[0].record().inventory_slot(2);
    item.0[13] = 99;
    state.warriors[0].record_mut().set_inventory_slot(2, &item);
    let equip = Command::EquipItem {
        warrior: 0,
        slot: 2,
    };
    let events = apply(equip.clone(), &mut state, &content).unwrap();
    assert_eq!(events, [Event::CollectionRejected { warning: 48 }]);
    content.items[item.id() as usize][26..28].copy_from_slice(&2_i16.to_be_bytes());
    let events = apply(equip, &mut state, &content).unwrap();
    let level =
        Event::Feedback(centurions_core::feedback::GameFeedback::EquipmentLevel { level: 99 });
    assert_eq!(events, [level]);
}
