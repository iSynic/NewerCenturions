use centurions_content::{Content, InventoryRecord};
use centurions_core::{RandomState, items};
use serde::Deserialize;

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
fn hex(value: &str) -> Vec<u8> {
    (0..value.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&value[offset..offset + 2], 16).unwrap())
        .collect()
}

#[test]
fn plain_loot_matches_original_item_execution() {
    verify(include_str!("../../../evidence/fixtures/plain-loot.json"));
}

#[test]
fn starter_ammunition_matches_original_initialization() {
    verify(include_str!(
        "../../../evidence/fixtures/ammunition-init.json"
    ));
}

#[test]
fn authored_supply_items_match_original_initialization() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/supply-init.json")).unwrap();
    let mut definitions = vec![vec![0; 150]; 1000];
    for (index, definition) in hex(fixture["ammunition_table"].as_str().unwrap())
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
        identity: "fixture".into(),
        warriors: vec![],
        items: definitions,
        resources: vec![],
        original_services: None,
        scenario: None,
    };
    for case in fixture["cases"].as_array().unwrap() {
        let id = case["item_id"].as_i64().unwrap() as i16;
        let seed = case["seed"].as_u64().unwrap() as u32;
        let mut random = RandomState::new(seed);
        let actual = items::initialize_plain(id, &content, &mut random).unwrap();
        assert_eq!(
            actual.0.to_vec(),
            hex(case["record"].as_str().unwrap()),
            "item {id} seed {seed}"
        );
        assert_eq!(
            random.seed,
            case["after_seed"].as_u64().unwrap() as u32,
            "item {id} seed {seed}"
        );
        assert_eq!(
            random.draws,
            case["draws"].as_array().unwrap().len() as u64,
            "item {id} seed {seed}"
        );
    }
}

fn verify(input: &str) {
    let fixture: Fixture = serde_json::from_str(input).unwrap();
    let mut definitions = vec![vec![0; 150]; 1000];
    definitions[fixture.item_id as usize] = hex(&fixture.definition);
    let content = Content {
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
