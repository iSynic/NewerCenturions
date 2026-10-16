use crate::prelude::*;

use crate::support::bytes as hex;

use centurions_core::items;

#[test]
fn plain_loot_matches_original_item_execution() {
    verify("plain-loot.json");
}

#[test]
fn starter_ammunition_matches_original_initialization() {
    verify("ammunition-init.json");
}

#[test]
fn authored_supply_items_match_original_initialization() {
    let fixture: serde_json::Value = crate::support::fixture("supply-init.json");
    let mut definitions = vec![vec![0; 150]; 1000];
    for (index, definition) in hex(fixture.text("ammunition_table"))
        .as_chunks::<150>()
        .0
        .iter()
        .enumerate()
    {
        definitions[200 + index] = definition.to_vec();
    }
    for (id, bytes) in fixture["definitions"].as_object().unwrap() {
        definitions[id.parse::<usize>().unwrap()] = hex(bytes.as_str().unwrap());
    }
    let content = Content {
        items: definitions,
        ..crate::support::fixture_content("fixture")
    };
    for case in fixture.array("cases") {
        let id = case.n::<i16>("item_id");
        let seed = case.n::<u32>("seed");
        let mut random = RandomState::new(seed);
        let actual = items::initialize_plain(id, &content, &mut random).unwrap();
        crate::support::native_expect! {
        bytes &(actual.0.to_vec()) => &case["record"], "binary_loot";
        scalar random.seed => &case["after_seed"], "binary_loot";
        }
        assert_eq!(
            random.draws,
            case.array("draws").len() as u64,
            "item {id} seed {seed}"
        );
    }
}

fn verify(name: &str) {
    let fixture = crate::support::fixture(name);
    let mut definitions = vec![vec![0; 150]; 1000];
    definitions[fixture.n::<usize>("item_id")] = fixture.bytes("definition");
    let content = Content {
        items: definitions,
        ..crate::support::fixture_content("fixture")
    };
    for case in fixture.array("cases") {
        let mut random = RandomState::new(case.n("seed"));
        let actual = items::initialize_plain(fixture.n("item_id"), &content, &mut random).unwrap();
        assert_eq!(
            actual,
            InventoryRecord::read(&case.bytes("record")).unwrap()
        );
        assert_eq!(random.seed, case.n::<u32>("after_seed"));
        assert_eq!(random.draws, case.array("draws").len() as u64);
        let mut draws = RandomState::new(case.n("seed"));
        for draw in case.array("draws") {
            let raw = draws.quickdraw();
            assert_eq!(raw, draw.n::<i16>("raw"));
            assert_eq!(
                centurions_core::rules::random_bound(draw.n("bound"), raw),
                draw.n::<i16>("result")
            );
        }
    }
}
