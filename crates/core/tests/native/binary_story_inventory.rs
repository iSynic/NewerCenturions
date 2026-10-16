use crate::prelude::*;

use centurions_core::story_inventory::*;

use crate::support::record_bytes as records;
use crate::support::warriors;

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
    ] {
        let fixture = crate::support::fixture(name);
        let race_movements: Vec<i8> = fixture
            .array("race_movements")
            .iter()
            .map(|v| v.as_i64().unwrap() as i8)
            .collect();
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
            crate::support::native_expect! {
            bytes records(&result.warriors) => &case["after"], "consumption";
            scalar result.consumed => &case["consumed"], "binary_story_inventory";
            scalar crate::support::event_sounds(&result.events) => &case["sounds"], "binary_story_inventory";
            }
            if name != "story-inventory.json" {
                let mut content = content.clone();
                let shared = std::sync::Arc::make_mut(
                    &mut content.scenario.as_mut().unwrap().maps[0].shared,
                );
                shared.combat.as_mut().unwrap().encounters[2470..2480]
                    .copy_from_slice(&case.bytes("extra"));
                let mut state = crate::support::mission_session(&content);
                state.inventory_record = retained;
                if state.inventory_record.is_some() {
                    apply(Command::RefitParty, &mut state, &content).unwrap();
                }
                state.warriors = before;
                let position = state.exploration.as_mut().unwrap();
                position.presentation = Some(Presentation::Dialogue {
                    id: 469,
                    text: content.map(0).unwrap().dialogue(469).unwrap(),
                });
                position.script = Some(crate::support::script_state(
                    content.map(0).unwrap().script(128).unwrap().to_vec(),
                    2,
                ));
                let mut resumed =
                    crate::support::checkpoint(&state, &content, "story-charge-before");
                let random = state.random.clone();
                let events = crate::support::replay_command(
                    Command::Continue,
                    &mut state,
                    &mut resumed,
                    &content,
                );
                crate::support::assert_bytes(
                    &records(&state.warriors),
                    &case.bytes("after"),
                    "shared story charge caller",
                );
                assert_eq!(
                    crate::support::event_sounds(&events),
                    crate::support::event_sounds(&result.events)
                );
                if let Some(expected) = case.get("events") {
                    assert_eq!(serde_json::to_value(&events).unwrap(), *expected);
                }
                assert_eq!(state.random, random);
                if case.get("item_after").is_some() {
                    assert_eq!(state.ammunition_work.item, Some(case.bytes("item_after")));
                }
                if case.get("selected_after").is_some() {
                    crate::support::assert_bytes(
                        &state.inventory_record.as_ref().unwrap().0,
                        &case.bytes("selected_after"),
                        "retained inventory work",
                    );
                }
                crate::support::checkpoint(&state, &content, "story-charge-after");
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
    for case in crate::support::fixture("story-inventory.json")["query_cases"]
        .as_array()
        .unwrap()
    {
        let before = warriors(case.text("before"));
        let extra = case.bytes("extra");
        let found = query_party_item(&before, word(&extra, 0).unwrap()).unwrap();
        crate::support::native_expect! {
        scalar item_prerequisite_matches(&before, &extra).unwrap() => &case["matched"], "binary_story_inventory";
        scalar found.as_ref().map_or(0, |item| item.warrior + 1) => &case["holder"], "binary_story_inventory";
        scalar found.as_ref().map_or(0, |item| item.slot) => &case["slot"], "binary_story_inventory";
        bytes records(&before) => &case["after"], "binary_story_inventory";
        }
    }
}

#[test]
fn story_experience_awards_match_original_complete_normalized_warriors() {
    for case in crate::support::fixture("story-inventory.json")["award_cases"]
        .as_array()
        .unwrap()
    {
        let result =
            award_story_experience(&warriors(case.text("before")), case.n::<i32>("amount"))
                .unwrap();
        crate::support::native_expect! {
        bytes records(&result.warriors) => &case["after"], "experience award";
        scalar result.share => &case["share"], "binary_story_inventory";
        }
    }
}

#[test]
fn authored_treasure_matches_original_full_loot_and_rng_for_both_seeds() {
    let content = content();
    for case in crate::support::fixture("story-inventory.json")["treasure_cases"]
        .as_array()
        .unwrap()
    {
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let items =
            grant_story_treasure(&case.bytes("table_before"), &content, &mut random, 0).unwrap();
        let mut loot = vec![InventoryRecord([0; 22]); 1000];
        loot[..items.len()].clone_from_slice(&items);
        crate::support::native_expect! {
        bytes loot.iter().flat_map(|item| item.0).collect::<Vec<_>>() => &case["crate_after"], &format!("treasure {} seed {}", case["treasure"], case["seed_before"]);
        scalar random.seed => &case["seed_after"], "binary_story_inventory";
        scalar random.draws => &case["rng_draws"], "binary_story_inventory";
        bytes records(&normalize_story_loot(&warriors(case.text("before"))).unwrap()) => &case["after"], "binary_story_inventory";
        }
    }
}
