use crate::{prelude::*, support};

use centurions_core::story_inventory::*;

use support::record_bytes as records;
use support::warriors;

#[test]
fn quest_consumption_matches_original_complete_warriors_and_drop_effects() {
    let content = content();
    for name in [
        "story-inventory.json",
        "story-charges.json",
        "story-replacements.json",
        "story-equipped-items.json",
        "story-equipped-depletion.json",
        "story-retained-equipment.json",
        "story-retained-replacement.json",
        "retained-caller-joins.json",
        "retained-refit-consumer-transactions.json",
        "retained-refit-right-callers.json",
        "retained-refit-battery-loaded-return.json",
        "retained-refit-battery-empty-return.json",
    ] {
        let fixture = support::fixture(name);
        let race_movements = fixture.n::<Vec<i8>>("race_movements");
        for case in fixture.array("consume_cases") {
            let before = warriors(case.text("before"));
            let retained = case
                .get("selected_before")
                .map(|_| InventoryRecord(case.bytes("selected_before").try_into().unwrap()));
            let result = consume_party_items_retained(
                &before,
                &content,
                &race_movements,
                &case.bytes("extra"),
                retained.as_ref(),
            )
            .unwrap();
            support::native_expect! {
                bytes records(&result.warriors) => &case["after"], "consumption";
                scalar result.consumed => &case["consumed"], "binary_story_inventory";
                scalar support::event_sounds(&result.events) => &case["sounds"], "binary_story_inventory";
            }
            if name != "story-inventory.json" {
                let mut content = content.clone();
                let shared = std::sync::Arc::make_mut(
                    &mut content.scenario.as_mut().unwrap().maps[0].shared,
                );
                shared.combat.as_mut().unwrap().encounters[2470..2480]
                    .copy_from_slice(&case.bytes("extra"));
                let mut state = support::mission_session(&content);
                if let Some(record) = retained {
                    if case.get("writer").is_none() {
                        apply(Command::RefitParty, &mut state, &content).unwrap();
                    }
                    state.warriors = case
                        .get("before_prefix")
                        .map_or_else(|| before.clone(), support::warriors);
                    // Assisted row selection establishes the real screen writer before
                    // restoring the independent native consumer's party prerequisites.
                    let stock = case["writer"] == "stock";
                    if stock {
                        state.shops.insert(0, content.map(0).unwrap().shops.clone());
                        let shop = state.shops.get_mut(&0).unwrap();
                        shop[..2].copy_from_slice(&1_i16.to_be_bytes());
                        shop[2000] = 1;
                        state.shop_access = Some(centurions_core::shop::Access {
                            level: 0,
                            id: 0,
                            enhancement: 0,
                            remaining: 0,
                        });
                        let position = state.exploration.as_mut().unwrap();
                        position.presentation = Some(Presentation::Shop);
                        position.script = None;
                    }
                    let side = case["side"]
                        .as_u64()
                        .map_or(usize::from(stock), |n| n as usize);
                    let selected = (record.id() != 0).then_some((side, 0));
                    if selected.is_some() && case.get("writer").is_none() {
                        state.warriors[0].bytes[0x132..0x148].copy_from_slice(&record.0);
                    }
                    apply(
                        Command::InspectInventory {
                            members: [0, 0],
                            scroll: [0, 0],
                            selected,
                            shop_family: 0,
                        },
                        &mut state,
                        &content,
                    )
                    .unwrap();
                    assert_eq!(state.native_work.inventory_record.as_ref(), Some(&record));
                    if stock {
                        apply(Command::CloseShop, &mut state, &content).unwrap();
                    }
                }
                if case["writer"] == "refit" {
                    assert_eq!(state.warriors, support::warriors(&case["before_prefix"]));
                    support::assert_bytes(
                        state.native_work.ammunition_work.item.as_ref().unwrap(),
                        &case.bytes("item_before_prefix"),
                        "inspection before native refit",
                    );
                    state.ammunition = support::longs(&case.bytes("reserves_before_prefix")[..800]);
                    state.native_work.ammunition_work.ammo = Some(case.bytes("ammo_before_prefix"));
                    state.native_work.ammunition_work.weapon =
                        Some(case.bytes("weapon_before_prefix"));
                    let random = state.random.clone();
                    let mut prefix_events = vec![];
                    for _ in 0..case["refit_rounds"].as_u64().unwrap_or(1) {
                        prefix_events.extend(support::saved_command(
                            Command::RefitParty,
                            &mut state,
                            &content,
                        ));
                    }
                    if let Some(sounds) = case.get("prefix_sounds") {
                        assert_eq!(
                            serde_json::to_value(support::event_sounds(&prefix_events)).unwrap(),
                            *sounds
                        );
                    }
                    assert_eq!(state.random, random);
                    assert_eq!(
                        state.ammunition,
                        support::longs(&case.bytes("reserves_after_prefix")[..800])
                    );
                    support::native_expect! {
                        bytes state.native_work.ammunition_work.ammo.as_ref().unwrap() => &case["prefix_ammo"], "refit ammo before consumer";
                        bytes state.native_work.ammunition_work.item.as_ref().unwrap() => &case["prefix_item"], "refit item before consumer";
                        bytes state.native_work.ammunition_work.weapon.as_ref().unwrap() => &case["prefix_weapon"], "refit weapon before consumer";
                    }
                    support::assert_bytes(
                        &records(&state.warriors),
                        &case.bytes("before"),
                        "native refit party before consumer",
                    );
                } else if case.get("writer").is_none() {
                    state.warriors = before;
                }
                let position = state.exploration.as_mut().unwrap();
                position.presentation = Some(Presentation::Dialogue {
                    id: 469,
                    text: content.map(0).unwrap().dialogue(469).unwrap(),
                });
                position.script = Some(centurions_core::ScriptState::scheduled(
                    content.map(0).unwrap().script(128).unwrap().to_vec(),
                    2,
                ));
                let mut resumed = support::checkpoint(&state, &content, "story-charge-before");
                let random = state.random.clone();
                let events =
                    support::replay_command(Command::Continue, &mut state, &mut resumed, &content);
                support::native_expect! {
                    bytes &records(&state.warriors) => &case.bytes("after"), "shared story charge caller";
                };
                assert_eq!(
                    support::event_sounds(&events),
                    support::event_sounds(&result.events)
                );
                if let Some(expected) = case.get("events") {
                    assert_eq!(serde_json::to_value(&events).unwrap(), *expected);
                }
                assert_eq!(state.random, random);
                if case.get("item_after").is_some() {
                    assert_eq!(
                        state.native_work.ammunition_work.item,
                        Some(case.bytes("item_after"))
                    );
                }
                if case.get("selected_after").is_some() {
                    support::native_expect! {
                        bytes &state.native_work.inventory_record.as_ref().unwrap().0 => &case.bytes("selected_after"), "retained inventory work";
                    };
                }
                support::checkpoint(&state, &content, "story-charge-after");
                if case["case"] == "repeated_drop_refusal" {
                    let before = state.warriors.clone();
                    let slot = state.warriors[0].inventory().unwrap().len() - 2;
                    let events = apply(
                        Command::UnequipItem { warrior: 0, slot },
                        &mut state,
                        &content,
                    )
                    .unwrap();
                    assert_eq!(state.warriors, before);
                    assert_eq!(events, vec![Event::CollectionRejected { warning: 145 }]);
                }
            }
        }
    }
}

#[test]
fn item_prerequisites_match_original_first_party_holder_and_slot() {
    for case in support::fixture("story-inventory.json").array("query_cases") {
        let before = warriors(case.text("before"));
        let extra = case.bytes("extra");
        let found = query_party_item(&before, word(&extra, 0).unwrap()).unwrap();
        support::native_expect! {
            scalar item_prerequisite_matches(&before, &extra).unwrap() => &case["matched"], "binary_story_inventory";
            scalar found.as_ref().map_or(0, |item| item.warrior + 1) => &case["holder"], "binary_story_inventory";
            scalar found.as_ref().map_or(0, |item| item.slot) => &case["slot"], "binary_story_inventory";
            bytes records(&before) => &case["after"], "binary_story_inventory";
        }
    }
}

#[test]
fn story_experience_awards_match_original_complete_normalized_warriors() {
    for case in support::fixture("story-inventory.json").array("award_cases") {
        let result =
            award_story_experience(&warriors(case.text("before")), case.n::<i32>("amount"))
                .unwrap();
        support::native_expect! {
            bytes records(&result.warriors) => &case["after"], "experience award";
            scalar result.share => &case["share"], "binary_story_inventory";
        }
    }
}

#[test]
fn authored_treasure_matches_original_full_loot_and_rng_for_both_seeds() {
    let content = content();
    for case in support::fixture("story-inventory.json").array("treasure_cases") {
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let items =
            grant_story_treasure(&case.bytes("table_before"), &content, &mut random, 0).unwrap();
        let mut loot = vec![InventoryRecord([0; 22]); 1000];
        loot[..items.len()].clone_from_slice(&items);
        support::native_expect! {
            bytes loot.iter().flat_map(|item| item.0).collect::<Vec<_>>() => &case["crate_after"], &format!("treasure {} seed {}", case["treasure"], case["seed_before"]);
            rng random => (&case["seed_after"], &case["rng_draws"]), "binary_story_inventory";
            bytes records(&normalize_story_loot(&warriors(case.text("before"))).unwrap()) => &case["after"], "binary_story_inventory";
        }
    }
}
