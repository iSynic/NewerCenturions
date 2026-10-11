use centurions_content::{Warrior, digest};
use centurions_core::{RandomState, combat::entry_party::initiative};

fn bytes(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).unwrap())
        .collect()
}
fn changes(raw: &mut [u8], changes: &serde_json::Value) {
    for change in changes.as_array().unwrap() {
        raw[change[0].as_u64().unwrap() as usize] = change[1].as_u64().unwrap() as u8;
    }
}
#[test]
fn inactive_party_initiative_matches_original_records_order_and_random_consumption() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/inactive-initiative.json"
    ))
    .unwrap();
    let original = bytes(fixture["warriors"].as_str().unwrap());
    let enemies: Vec<_> = fixture["enemy_speeds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap() as i16)
        .collect();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 140);
    for (index, case) in cases.iter().enumerate() {
        let mut before = original.clone();
        changes(&mut before, &case["before_changes"]);
        assert_eq!(digest(&before), case["before_sha256"], "case{index} input");
        let warriors: Vec<_> = before
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|raw| Warrior::read(raw).unwrap())
            .collect();
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let outcome = initiative(
            &warriors,
            &enemies,
            case["mode"].as_i64().unwrap() as i16,
            case["unavailable"].as_i64().unwrap() as i8,
            &mut random,
        )
        .unwrap();
        let actual: Vec<_> = outcome
            .warriors
            .iter()
            .flat_map(|w| w.bytes.iter().copied())
            .collect();
        let mut expected = before;
        changes(&mut expected, &case["after_changes"]);
        assert_eq!(actual, expected, "case{index} complete warrior records");
        assert_eq!(digest(&actual), case["after_sha256"], "case{index} output");
        let order: Vec<_> = bytes(case["order"].as_str().unwrap())
            .as_chunks::<2>()
            .0
            .iter()
            .map(|w| i16::from_be_bytes(*w))
            .collect();
        assert_eq!(outcome.order, order, "case{index} order");
        assert_eq!(
            random.seed,
            case["seed_after"].as_u64().unwrap() as u32,
            "case{index} seed"
        );
        assert_eq!(
            random.draws,
            case["draws"].as_u64().unwrap(),
            "case{index} draws"
        );
    }
}

#[test]
fn unavailable_counter_capacity_diagnostics_preserve_party_and_randomness() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/inactive-initiative.json"
    ))
    .unwrap();
    let original = bytes(fixture["warriors"].as_str().unwrap());
    let warriors: Vec<_> = original
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|raw| Warrior::read(raw).unwrap())
        .collect();
    for (counter, mode) in [(-128, 0), (-1, 0), (127, 0), (1, 1), (127, 1)] {
        let mut random = RandomState::new(12);
        let before = random.clone();
        let party = warriors.clone();
        assert!(initiative(&party, &[8; 9], mode, counter, &mut random).is_err());
        assert_eq!(party, warriors);
        assert_eq!(random, before);
    }
}
