use centurions_content::{Content, Warrior, word};
use centurions_core::story_item_use::use_scenario_item;
use serde_json::Value;

fn bytes(value: &Value) -> Vec<u8> {
    let text = value.as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).unwrap())
        .collect()
}

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../evidence/fixtures/item-use.json")).unwrap()
}

fn warriors(value: &Value) -> Vec<Warrior> {
    bytes(value)
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|record| Warrior::read(record).unwrap())
        .collect()
}

fn content(definition: Vec<u8>) -> Content {
    let mut items = vec![vec![0; 150]; 833];
    items[832] = definition;
    Content {
        identity: String::from("item-use fixture"),
        warriors: vec![],
        items,
        resources: vec![],
        original_services: None,
        scenario: None,
    }
}

fn assert_bytes(actual: &[u8], expected: &[u8], label: &str) {
    assert_eq!(actual.len(), expected.len());
    if let Some(offset) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "{label} byte {offset:#x}: actual {}, original {}",
            actual[offset], expected[offset]
        );
    }
}

#[test]
fn scenario_item_use_matches_original_complete_party_ordered_costs_and_returns() {
    let fixture = fixture();
    let movement = fixture["race_movements"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_i64().unwrap() as i8)
        .collect::<Vec<_>>();
    for case in fixture["cases"].as_array().unwrap() {
        let before = warriors(&case["before"]);
        let content = content(bytes(&case["definition"]));
        let result = use_scenario_item(
            &before,
            case["member"].as_u64().unwrap() as usize,
            case["slot"].as_u64().unwrap() as usize,
            &content,
            &movement,
        )
        .unwrap();
        let actual = result
            .warriors
            .iter()
            .flat_map(|warrior| warrior.bytes.iter().copied())
            .collect::<Vec<_>>();
        assert_bytes(
            &actual,
            &bytes(&case["after"]),
            &format!("member {} variant {}", case["member"], case["variant"]),
        );
        assert_eq!(
            result.return_code,
            case["return_code"].as_i64().unwrap() as i16
        );
        assert_eq!(
            result.script,
            if case["script"] == -1 {
                None
            } else {
                Some(case["script"].as_i64().unwrap() as i16)
            }
        );
        assert_eq!(
            result.sounds,
            case["sounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_i64().unwrap() as i16)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            result.warnings,
            case["warnings"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_i64().unwrap() as i16)
                .collect::<Vec<_>>()
        );
        let mut effects = result
            .warnings
            .iter()
            .map(|warning| serde_json::json!(["warning", warning]))
            .chain(
                result
                    .sounds
                    .iter()
                    .map(|sound| serde_json::json!(["sound", sound])),
            )
            .collect::<Vec<_>>();
        if let Some(script) = result.script {
            effects.push(serde_json::json!(["script", script]));
            let args = bytes(&case["script_args"]);
            assert_eq!(&args[..8], &[0; 8]);
            assert_eq!(word(&args, 8).unwrap(), 1);
            assert_eq!(word(&args, 10).unwrap(), script);
            assert_eq!(word(&args, 12).unwrap(), 0);
        } else {
            assert!(case["script_args"].as_str().unwrap().is_empty());
        }
        assert_eq!(case["effects"], Value::Array(effects));
        assert_eq!(case["draws"], 0);
        assert_eq!(case["seed_before"], case["seed_after"]);
        let original = before
            .iter()
            .flat_map(|warrior| warrior.bytes.iter().copied())
            .collect::<Vec<_>>();
        assert_bytes(&original, &bytes(&case["before"]), "caller inputs");
    }
}

#[test]
fn unsupported_item_use_returns_a_diagnostic_without_changing_inputs() {
    let fixture = fixture();
    let case = &fixture["cases"][0];
    let mut before = warriors(&case["before"]);
    let member = case["member"].as_u64().unwrap() as usize;
    let slot = case["slot"].as_u64().unwrap() as usize;
    let definition = bytes(&case["definition"]);
    before[member].bytes[0x13b + slot * 22] = 1;
    let original = before.clone();
    let content = content(definition);
    assert!(use_scenario_item(&before, member, slot, &content, &[12; 10]).is_err());
    assert_eq!(before, original);
}
