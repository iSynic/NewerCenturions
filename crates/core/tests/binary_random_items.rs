use centurions_content::Content;
use centurions_core::{RandomState, items};

fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).unwrap())
        .collect()
}

#[test]
fn item_initialization_and_enhancement_match_original_records_and_randomness() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/random-items.json")).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let content = Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 304);
    for (index, case) in cases.iter().enumerate() {
        let id = case["id"].as_i64().unwrap() as i16;
        assert_eq!(
            content.items[id.unsigned_abs() as usize],
            bytes(case["definition"].as_str().unwrap())
        );
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let record = items::initialize(
            id,
            case["enhancement"].as_i64().unwrap() as i16,
            &content,
            &mut random,
        )
        .unwrap_or_else(|error| panic!("case{index} item{id}: {error:?}"));
        assert_eq!(
            record.0.to_vec(),
            bytes(case["after"].as_str().unwrap()),
            "case{index} item{id}"
        );
        assert_eq!(
            random.seed,
            case["seed_after"].as_u64().unwrap() as u32,
            "case{index} seed"
        );
        assert_eq!(
            random.draws,
            case["rng_draws"].as_u64().unwrap(),
            "case{index} draws"
        );
    }
}
