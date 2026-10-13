use crate::prelude::*;

use crate::support::bytes as hex;

use centurions_core::items;

#[derive(Deserialize)]
struct Fixture {
    item_id: i16,
    definition: String,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Case {
    seed: u32,
    after_seed: u32,
    draws: Vec<Draw>,
    record: String,
}
#[derive(Deserialize)]
struct Draw {
    bound: i16,
    raw: i16,
    result: i16,
}

#[test]
fn plain_loot_matches_original_item_execution() {
    verify(include_str!(
        "../../../../evidence/fixtures/plain-loot.json"
    ));
}

#[test]
fn starter_ammunition_matches_original_initialization() {
    verify(include_str!(
        "../../../../evidence/fixtures/ammunition-init.json"
    ));
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
        tutorials: vec![],
        identity: "fixture".into(),
        warriors: vec![],
        items: definitions,
        resources: vec![],
        original_services: None,
        scenario: None,
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

fn verify(input: &str) {
    let fixture: Fixture = serde_json::from_str(input).unwrap();
    let mut definitions = vec![vec![0; 150]; 1000];
    definitions[fixture.item_id as usize] = hex(&fixture.definition);
    let content = Content {
        tutorials: vec![],
        identity: "fixture".into(),
        warriors: vec![],
        items: definitions,
        resources: vec![],
        original_services: None,
        scenario: None,
    };
    for case in fixture.cases {
        let mut random = RandomState::new(case.seed);
        let actual = items::initialize_plain(fixture.item_id, &content, &mut random).unwrap();
        assert_eq!(actual, InventoryRecord::read(&hex(&case.record)).unwrap());
        assert_eq!(random.seed, case.after_seed);
        assert_eq!(random.draws, case.draws.len() as u64);
        let mut draws = RandomState::new(case.seed);
        for draw in case.draws {
            let raw = draws.quickdraw();
            assert_eq!(raw, draw.raw);
            assert_eq!(
                centurions_core::rules::random_bound(draw.bound, raw),
                draw.result
            );
        }
    }
}
