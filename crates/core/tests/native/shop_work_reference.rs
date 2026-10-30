//! Original loadshop, moveicon stock slices and saveshop, with source command joins.
//! Native stock slices do not qualify payment, inventory or original UI execution.
use crate::shop::{self, TheShop};
use crate::test_support::{bytes, content, fixture};
use crate::{Command, Content, Session};
use centurions_content::records::SHOP_BYTES;
use serde_json::Value;

fn source(case: &Value) -> &Value {
    &case["declarations"]["source_input"]
}

fn work_after(case: &Value) -> Vec<u8> {
    bytes(&case["memory"]["current_shop_work"]["hex"])
}

fn session(captures: &Value, content: &Content) -> Session {
    let mut state = Session::new(content);
    for command in [
        Command::ImportStarters,
        Command::EnterMission {
            name: "The Thunder of Guns".into(),
        },
    ] {
        crate::apply(command, &mut state, content).unwrap();
    }
    let table = content.map(0).unwrap().shops.clone();
    state.shops.insert(0, table);
    for case in &captures["cases"].as_array().unwrap()[..2] {
        let input = source(case);
        let id = input["shop_id"].as_u64().unwrap() as usize;
        state.shops.get_mut(&0).unwrap()[id * SHOP_BYTES..(id + 1) * SHOP_BYTES]
            .copy_from_slice(&bytes(&input["shop_record"]));
    }
    state.globals.the_shop.copy_file(&bytes(
        &source(&captures["cases"][0])["before"]["current_shop_work"],
    ));
    state.party_currency = 1_000_000;
    state.warriors[0].record_mut().set_load(0);
    state.warriors[0].record_mut().set_weight_capacity(i16::MAX);
    state
}

#[test]
fn retained_shop_matches_native_work_and_public_transaction_file_timing() {
    let captures = fixture("shop-work.json");
    let cases = captures["cases"].as_array().unwrap();
    let mut work = TheShop::default();
    for case in cases {
        assert_eq!(case["status"], "completed");
        let input = source(case);
        work.copy_file(&bytes(&input["before"]["current_shop_work"]));
        match input["parent"].as_str().unwrap() {
            "loadshop" => work.load(&bytes(&input["shop_record"])).unwrap(),
            "moveicon-buy-work-slice" => work
                .buy_one(input["slot"].as_u64().unwrap() as usize)
                .unwrap(),
            "moveicon-sale-work-slice" => work
                .accept_sale(input["item"].as_i64().unwrap() as i16)
                .unwrap(),
            "saveshop" => assert_eq!(
                work.record(),
                bytes(&case["memory"]["file_write_payload"]["hex"]),
            ),
            parent => panic!("unexpected native shop owner {parent}"),
        }
        assert_eq!(work.record(), work_after(case), "{}", case["name"]);
    }

    let content = content();
    let mut state = session(&captures, &content);
    let files = state.shops.clone();
    for case in &cases[..2] {
        shop::visit(
            &mut state,
            &content,
            source(case)["shop_id"].as_i64().unwrap() as i16,
        )
        .unwrap();
        assert_eq!(state.globals.the_shop.record(), work_after(case));
        assert_eq!(state.shops, files, "loadshop changed its input file");
    }
    crate::apply(Command::OpenShop, &mut state, &content).unwrap();
    assert!(
        shop::stock_family(&state, 0)
            .unwrap()
            .contains(&(50, 186, 1))
    );
    crate::apply(
        Command::BuyItem {
            warrior: 0,
            stock: 50,
        },
        &mut state,
        &content,
    )
    .unwrap();
    assert_eq!(state.globals.the_shop.record(), work_after(&cases[2]));
    let slot = state.warriors[0]
        .inventory()
        .unwrap()
        .iter()
        .rposition(|item| item.id() == 186)
        .unwrap();
    crate::apply(Command::SellItem { warrior: 0, slot }, &mut state, &content).unwrap();
    assert_eq!(state.globals.the_shop.record(), work_after(&cases[3]));
    assert_eq!(state.shops, files, "moveicon flushed stock before saveshop");
    crate::apply(Command::CloseShop, &mut state, &content).unwrap();
    let id = source(&cases[4])["shop_id"].as_u64().unwrap() as usize;
    assert_eq!(
        &state.shops[&0][id * SHOP_BYTES..(id + 1) * SHOP_BYTES],
        bytes(&cases[4]["memory"]["file_write_payload"]["hex"]),
    );

    // A controlled second-level copy isolates the source-composed Access transition.
    let level = 2;
    let mut level_content = content.clone();
    let mut records = level_content.scenario.as_ref().unwrap().clone();
    records.level = level;
    std::sync::Arc::make_mut(&mut level_content.scenario.as_mut().unwrap().catalog)
        .insert((records.name.clone(), level), records);
    let table = level_content
        .context(state.mission.as_deref(), level)
        .unwrap()
        .map(0)
        .unwrap()
        .shops
        .clone();
    state.shops.insert(level, table);
    state.shops.get_mut(&level).unwrap()[id * SHOP_BYTES..(id + 1) * SHOP_BYTES]
        .copy_from_slice(&bytes(&source(&cases[1])["shop_record"]));
    state.scenario_level = level;
    state.shop_access = Some(shop::Access {
        level: 0,
        id: 1,
        enhancement: 7,
        remaining: 3,
    });
    let mut extra = [0; 10];
    extra[..2].copy_from_slice(&(id as i16).to_be_bytes());
    shop::alter(&mut state, &level_content, &extra).unwrap();
    let access = state.shop_access.as_ref().unwrap();
    assert_eq!(
        (
            access.level,
            access.id,
            access.enhancement,
            access.remaining
        ),
        (level, id as u8, 7, 3)
    );
    assert_eq!(
        &state.shops[&level][id * SHOP_BYTES..(id + 1) * SHOP_BYTES],
        state.globals.the_shop.record(),
    );
}
