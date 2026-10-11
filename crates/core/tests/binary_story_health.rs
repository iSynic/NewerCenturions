use centurions_content::{Warrior, digest, word};
use centurions_core::{
    RandomState,
    story_health::{HealthContext, HealthEvent, heal_outdoors, party_health},
};

fn bytes(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).unwrap())
        .collect()
}

fn changes(raw: &mut [u8], values: &serde_json::Value) {
    for change in values.as_array().unwrap() {
        raw[change[0].as_u64().unwrap() as usize] = change[1].as_u64().unwrap() as u8;
    }
}

fn event(event: &HealthEvent) -> serde_json::Value {
    match event {
        HealthEvent::Animation { warrior, effect } => {
            serde_json::json!(["animation", warrior, effect])
        }
        HealthEvent::Sound(sound) => serde_json::json!(["sound", sound, 0]),
        HealthEvent::Defeated { warrior, permanent } => {
            serde_json::json!(["defeat", warrior, i16::from(*permanent)])
        }
        HealthEvent::PartyLost => serde_json::json!(["party_loss", 0, 0]),
        HealthEvent::Dialogue(dialogue) => serde_json::json!(["dialogue", dialogue, 0]),
    }
}

#[test]
fn outdoor_heal_and_party_callers_match_original_records_effect_order_and_rng() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/story-health.json")).unwrap();
    let original = bytes(fixture["warriors"].as_str().unwrap());
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let mut before = original.clone();
        changes(&mut before, &case["before_changes"]);
        assert_eq!(digest(&before), case["before_sha256"], "input {index}");
        let warriors: Vec<_> = before
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|raw| Warrior {
                name: "fixture".into(),
                original_sha256: String::new(),
                bytes: raw.to_vec(),
            })
            .collect();
        let affected = bytes(case["affected_before"].as_str().unwrap());
        let context = HealthContext {
            unavailable: case["unavailable_before"].as_i64().unwrap() as i8,
            selected_warrior: case["member"].as_u64().unwrap() as usize,
            effect: 77,
            last_body: -7,
            marks_affected: case["marks_affected"].as_bool().unwrap(),
            affected: std::array::from_fn(|member| word(&affected, member * 2).unwrap()),
        };
        let mask_bytes = bytes(case["mask"].as_str().unwrap());
        let mask = std::array::from_fn(|member| word(&mask_bytes, member * 2).unwrap());
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let outcome = if case["kind"] == "heal" {
            heal_outdoors(
                &warriors,
                context.selected_warrior,
                case["amount"].as_i64().unwrap() as i16,
                case["animate"].as_bool().unwrap(),
                &context,
                &mut random,
            )
        } else {
            party_health(
                &warriors,
                &bytes(case["extra"].as_str().unwrap()),
                case["kind"] == "all",
                &mask,
                &context,
                &mut random,
            )
        }
        .unwrap();
        let actual: Vec<_> = outcome
            .warriors
            .iter()
            .flat_map(|w| w.bytes.clone())
            .collect();
        let mut expected = before.clone();
        changes(&mut expected, &case["after_changes"]);
        assert_eq!(actual, expected, "full warrior state {index}");
        assert_eq!(digest(&actual), case["after_sha256"], "output {index}");
        assert_eq!(
            outcome.context.unavailable,
            case["unavailable_after"].as_i64().unwrap() as i8,
            "counter {index}"
        );
        assert_eq!(
            outcome.context.effect,
            case["effect_after"].as_i64().unwrap() as i8,
            "effect {index}"
        );
        assert_eq!(
            outcome.context.last_body,
            case["last_body_after"].as_i64().unwrap() as i16,
            "body {index}"
        );
        let actual_affected: Vec<_> = outcome
            .context
            .affected
            .iter()
            .flat_map(|v| v.to_be_bytes())
            .collect();
        assert_eq!(
            actual_affected,
            bytes(case["affected_after"].as_str().unwrap()),
            "affected {index}"
        );
        assert_eq!(
            outcome.events.iter().map(event).collect::<Vec<_>>(),
            case["events"].as_array().unwrap().clone(),
            "events {index}"
        );
        assert_eq!(
            serde_json::to_value(outcome.amounts).unwrap(),
            case["amounts"],
            "amounts {index}"
        );
        assert_eq!(
            random.seed,
            case["seed_after"].as_u64().unwrap() as u32,
            "seed {index}"
        );
        assert_eq!(
            random.draws,
            case["draws"].as_u64().unwrap(),
            "draws {index}"
        );
        assert_eq!(case["mask"], case["mask_after"]);
    }
}

#[test]
fn unsupported_health_input_preserves_party_and_randomness() {
    let warrior = Warrior {
        name: "fixture".into(),
        original_sha256: String::new(),
        bytes: vec![0; 10],
    };
    let warriors = vec![warrior];
    let context = HealthContext {
        unavailable: 0,
        selected_warrior: 0,
        effect: 0,
        last_body: 0,
        marks_affected: false,
        affected: [0; 6],
    };
    let mut random = RandomState::new(12);
    let before_random = random.clone();
    let before_warriors = warriors.clone();
    assert!(party_health(&warriors, &[0; 10], true, &[1; 6], &context, &mut random).is_err());
    assert_eq!(warriors, before_warriors);
    assert_eq!(random, before_random);
}
