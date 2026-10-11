use centurions_content::{Warrior, digest, word};
use centurions_core::{
    RandomState,
    story_health::{HealthContext, HealthEvent},
    story_reduce::reduce_outdoors,
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
fn outdoor_reduction_matches_original_complete_warriors_revival_counter_and_rng() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/story-reduce.json")).unwrap();
    let original = bytes(fixture["warriors"].as_str().unwrap());
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 384);
    for (index, case) in cases.iter().enumerate() {
        let mut before = original.clone();
        changes(&mut before, &case["before_changes"]);
        assert_eq!(digest(&before), case["before_sha256"], "input {index}");
        let warriors: Vec<_> = before
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|raw| Warrior::read(raw).unwrap())
            .collect();
        let affected = bytes(case["affected_before"].as_str().unwrap());
        let context = HealthContext {
            unavailable: case["unavailable_before"].as_i64().unwrap() as i8,
            selected_warrior: 0,
            effect: 77,
            last_body: -7,
            marks_affected: false,
            affected: std::array::from_fn(|member| word(&affected, member * 2).unwrap()),
        };
        let mut random = RandomState::new(case["seed_before"].as_u64().unwrap() as u32);
        let outcome = reduce_outdoors(&warriors, &context, &mut random).unwrap();
        let actual: Vec<_> = outcome
            .warriors
            .iter()
            .flat_map(|warrior| warrior.bytes.iter().copied())
            .collect();
        let mut expected = before.clone();
        changes(&mut expected, &case["after_changes"]);
        if let Some(offset) = actual.iter().zip(&expected).position(|(a, b)| a != b) {
            panic!(
                "case {index} full warrior byte {offset}: {} != {}",
                actual[offset], expected[offset]
            );
        }
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
        assert_eq!(outcome.context.affected, context.affected);
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
        assert_eq!(case["enemy_unchanged"], true);
        assert_eq!(case["buffs_unchanged"], true);
    }
}

#[test]
fn unrecovered_active_reduction_channels_preserve_party_and_randomness() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../evidence/fixtures/story-reduce.json")).unwrap();
    let original = bytes(fixture["warriors"].as_str().unwrap());
    let context = HealthContext {
        unavailable: 0,
        selected_warrior: 0,
        effect: 0,
        last_body: 0,
        marks_affected: false,
        affected: [0; 6],
    };
    for offset in [0x8a, 0x8c, 0x98, 0xa0, 0xa2] {
        let mut warrior = Warrior::read(&original[..3138]).unwrap();
        warrior.bytes[offset..offset + 2].copy_from_slice(&1_i16.to_be_bytes());
        let warriors = vec![warrior];
        let before = warriors.clone();
        let mut random = RandomState::new(12);
        let initial = random.clone();
        assert!(reduce_outdoors(&warriors, &context, &mut random).is_err());
        assert_eq!(warriors, before);
        assert_eq!(random, initial);
    }
}
