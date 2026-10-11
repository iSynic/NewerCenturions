use centurions_content::Warrior;
use centurions_core::RandomState;
use serde_json::{Value, json};

use centurions_core::story_npc_battle::{NpcBattleReturn, NpcBattleStart, finish};

fn bytes(value: &Value) -> Vec<u8> {
    let text = value.as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&text[index..index + 2], 16).unwrap())
        .collect()
}

#[test]
fn npc_battle_caller_matches_original_order_and_all_party_bytes() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/npc-battle-caller.json"
    ))
    .unwrap();
    let scripts = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data ED3"
    ))
    .unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let mut header = bytes(&case["header_before"]);
        let mut random = RandomState::new(case["seed"].as_u64().unwrap() as u32);
        let mut start = NpcBattleStart::capture(&bytes(&case["extra"]), &header, 3).unwrap();
        let mut warriors: Vec<_> = bytes(&case["warriors_before"])
            .as_chunks::<3138>()
            .0
            .iter()
            .map(|record| Warrior::read(record).unwrap())
            .collect();
        let mut events = vec![json!(["extra", 391])];
        if start.sound != 0 {
            events.push(json!(["sound", start.sound]));
        }
        if start.text != 0 {
            events.push(json!(["text", start.text]));
        }
        let battle = start.select_battle(&mut random);
        events.push(json!(["combat", battle, 0]));
        header.fill(0x6a);
        let mut cursor = 87;
        let mut movement = [9, 10];
        let mut terrain_context = 2;
        let mut map_dirty = 1;
        let outcome = start.return_kind(
            case["outcome"].as_u64().unwrap() == 1,
            case["outcome"].as_u64().unwrap() == 2,
        );
        if outcome != NpcBattleReturn::Abort {
            terrain_context = 255;
            events.push(json!(["load_map", 4]));
            map_dirty = 0;
        }
        if let Some(restored) = start
            .restore(&mut header, outcome == NpcBattleReturn::Abort)
            .unwrap()
        {
            cursor = restored;
        }
        match outcome {
            NpcBattleReturn::Abort => {
                finish(outcome, &mut warriors, [254, 3]).unwrap();
            }
            NpcBattleReturn::Reward => {
                assert_eq!(header, bytes(&case["header_at_booty"]));
                assert_eq!(
                    warriors
                        .iter()
                        .flat_map(|w| w.bytes.iter().copied())
                        .collect::<Vec<_>>(),
                    bytes(&case["warriors_at_booty"])
                );
                events.push(json!(["booty", 0, 0]));
                finish(outcome, &mut warriors, [254, 3]).unwrap();
                events.push(json!(["update_main", 0]));
            }
            NpcBattleReturn::EscapeScript(id) => {
                events.push(json!(["load_script", id]));
                let offset = usize::from(id.unsigned_abs()) * 42;
                header[8..].copy_from_slice(&scripts[offset + 8..offset + 42]);
                cursor = -1;
                finish(outcome, &mut warriors, [254, 3]).unwrap();
                events.push(json!(["update_main", 0]));
            }
            NpcBattleReturn::EscapeMove => {
                events.push(json!(["update_main", 0]));
                events.push(json!(["warning", 118]));
                events.push(json!(["sound", 26260]));
                events.push(json!(["warning", 124]));
                movement = finish(outcome, &mut warriors, [254, 3]).unwrap().unwrap();
                events.push(json!(["move_party", -1]));
            }
        }
        assert_eq!(battle, case["battle"].as_i64().unwrap() as i16);
        assert_eq!(header, bytes(&case["header_after"]));
        assert_eq!(cursor, case["cursor_after"].as_i64().unwrap() as i16);
        assert_eq!(movement.as_slice(), bytes(&case["move_after"]));
        assert_eq!(
            terrain_context,
            case["terrain_context_after"].as_u64().unwrap()
        );
        assert_eq!(map_dirty, case["map_dirty_after"].as_u64().unwrap());
        assert_eq!(
            warriors
                .iter()
                .flat_map(|w| w.bytes.iter().copied())
                .collect::<Vec<_>>(),
            bytes(&case["warriors_after"])
        );
        assert_eq!(events, case["events"].as_array().unwrap().clone());
        assert_eq!(random.seed, case["seed_after"].as_u64().unwrap() as u32);
        assert_eq!(random.draws, case["rng_draws"].as_u64().unwrap());
    }
}

#[test]
fn invalid_npc_battle_input_does_not_change_state() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../evidence/fixtures/npc-battle-caller.json"
    ))
    .unwrap();
    let before_party = bytes(&fixture["cases"][0]["warriors_before"]);
    let mut warriors = vec![Warrior::read(&before_party[..3138]).unwrap()];
    warriors[0].bytes.truncate(10);
    let before = warriors.clone();
    assert!(finish(NpcBattleReturn::EscapeMove, &mut warriors, [254, 3]).is_err());
    assert_eq!(warriors, before);
    assert!(NpcBattleStart::capture(&[0; 9], &[0; 42], 0).is_err());
    assert!(NpcBattleStart::capture(&[0; 10], &[0; 41], 0).is_err());
}
