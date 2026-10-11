use centurions_content::{Content, InventoryRecord, Warrior, combat::MonsterRecord, word};
use centurions_core::{
    Event, Phase, RandomState, Session,
    combat::{Combat, EnemySetup, SquadPlacement, TacticalMap, firearm, runtime},
    equipment, items,
};
use serde_json::Value;

fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).unwrap())
        .collect()
}
fn party(hex: &str) -> Vec<Warrior> {
    bytes(hex)
        .as_chunks::<3138>()
        .0
        .iter()
        .map(|b| Warrior::read(b).unwrap())
        .collect()
}
fn records(hex: &str) -> Vec<InventoryRecord> {
    bytes(hex)
        .as_chunks::<22>()
        .0
        .iter()
        .map(|b| InventoryRecord::read(b).unwrap())
        .collect()
}
fn data() -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../evidence/fixtures/required-player-gear.json"),
        )
        .unwrap(),
    )
    .unwrap()
}
fn content() -> Content {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Content::load(
        &root.join("NewCenturions"),
        &root.join("analysis/extracted"),
    )
    .unwrap()
}
fn assert_bytes(actual: &[u8], expected: &[u8], context: &str) {
    assert_eq!(actual.len(), expected.len(), "{context} size");
    if let Some(at) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!("{context} byte {at:x}: {} != {}", actual[at], expected[at]);
    }
}
fn assert_party(warriors: &[Warrior], snapshot: &Value, context: &str) {
    let actual: Vec<_> = warriors
        .iter()
        .flat_map(|w| w.bytes.iter().copied())
        .collect();
    assert_bytes(
        &actual,
        &bytes(snapshot["party"].as_str().unwrap()),
        context,
    );
}
fn sounds(snapshot: &Value) -> Vec<i16> {
    snapshot["sounds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_i64().unwrap() as i16)
        .collect()
}
fn warnings(snapshot: &Value) -> Vec<i16> {
    snapshot["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_i64().unwrap() as i16)
        .collect()
}
fn event_sounds(events: &[Event]) -> Vec<i16> {
    events
        .iter()
        .filter_map(|e| {
            if let Event::Sound { resource_id } = e {
                Some(*resource_id)
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn equipment_admission_matches_all_12_native_race_and_mos_cases() {
    let fixture = data();
    let content = content();
    let mission = content.map(0).unwrap();
    let cases = fixture["gate_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 12);
    for case in cases {
        let warriors = party(case["before"]["party"].as_str().unwrap());
        let member = case["member"].as_u64().unwrap() as usize;
        let id = case["item_id"].as_u64().unwrap() as usize;
        assert_eq!(
            equipment::can_use(&warriors[member], &content.items[id], mission).unwrap(),
            case["allowed"].as_i64().unwrap() != 0
        );
        assert_party(&warriors, &case["after"], "admission unchanged");
        assert_eq!(case["after"]["rng_draws"].as_u64().unwrap(), 0);
        assert!(sounds(&case["after"]).is_empty());
    }
}

#[test]
fn crate_transfer_matches_18_native_whole_party_and_storage_cases() {
    let fixture = data();
    let content = content();
    let cases = fixture["transfer_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 18);
    for (index, case) in cases.iter().enumerate() {
        let source = case["source"].as_u64().unwrap() as usize;
        let slot = case["crate_slot"].as_u64().unwrap() as usize;
        let mut state: Session =
            serde_json::from_value(fixture["foundations"][source]["session"].clone()).unwrap();
        state.warriors = party(case["before"]["party"].as_str().unwrap());
        state.crate_items = records(case["before"]["crate"].as_str().unwrap());
        state.random = RandomState::new(12);
        let mission = content
            .map(state.exploration.as_ref().unwrap().map)
            .unwrap();
        let result = equipment::collect_item(
            &state.warriors[0],
            &state.crate_items[slot],
            &content,
            mission,
        )
        .unwrap();
        let mut picked_party = state.warriors.clone();
        picked_party[0] = result.warrior;
        assert_party(
            &picked_party,
            &case["picked"],
            &format!("transfer{index} picked"),
        );
        let mut loot = state.crate_items.clone();
        loot[slot] = result.item;
        let actual: Vec<_> = loot.iter().flat_map(|r| r.0).collect();
        assert_bytes(
            &actual,
            &bytes(case["picked"]["loot"].as_str().unwrap()),
            "picked loot",
        );
        assert_eq!(result.collected, case["collected"].as_i64().unwrap() != 0);
        assert_eq!(result.sounds, sounds(&case["picked"]));
        assert_eq!(result.warnings, warnings(&case["picked"]));
        let events = equipment::take_crate(&mut state, &content, slot, 0).unwrap();
        assert_party(
            &state.warriors,
            &case["after"],
            &format!("transfer{index} closed"),
        );
        let actual: Vec<_> = state.crate_items.iter().flat_map(|r| r.0).collect();
        assert_bytes(
            &actual,
            &bytes(case["after"]["crate"].as_str().unwrap()),
            "closed crate",
        );
        assert_eq!(event_sounds(&events), sounds(&case["after"]));
        assert_eq!(state.random.draws, 0);
        assert_eq!(state.random.seed, 12);
    }
}

#[test]
fn equipment_wear_compact_remove_and_rewear_match_48_native_cases() {
    let fixture = data();
    let content = content();
    let cases = fixture["equipment_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 48);
    for (index, case) in cases.iter().enumerate() {
        let source = case["source"].as_u64().unwrap() as usize;
        let member = case["member"].as_u64().unwrap() as usize;
        let slot = case["inventory_slot"].as_u64().unwrap() as usize;
        let mission = content
            .map(
                fixture["foundations"][source]["session"]["exploration"]["map"]
                    .as_i64()
                    .unwrap() as i32,
            )
            .unwrap();
        let mut warriors = party(case["before"]["party"].as_str().unwrap());
        let mut audio = sounds(&case["before"]);
        let result = equipment::wear(
            &warriors[member],
            &content,
            mission,
            slot,
            case["combat"].as_i64().unwrap() != 0,
        )
        .unwrap();
        assert_eq!(
            result.changed,
            case["equipped"].as_i64().unwrap() != 0,
            "wear{index} admitted"
        );
        warriors[member] = result.warrior;
        audio.extend(result.sounds);
        let mut expected_audio = sounds(&case["worn"]);
        // The historical probe returns at warn16474a before Sound6000.
        // Enabled wear still reaches that shared original warning wrapper.
        for warning in &result.warnings {
            assert_eq!(*warning, 29, "wear{index} CanUse rejection");
            expected_audio.push(6000);
        }
        assert_eq!(audio, expected_audio);
        assert_eq!(result.warnings, warnings(&case["worn"]));
        assert_party(&warriors, &case["worn"], &format!("wear{index}"));
        if !result.changed {
            continue;
        }
        items::compact(&mut warriors[member]).unwrap();
        assert_party(&warriors, &case["compacted"], &format!("compact{index}"));
        let id = case["item_id"].as_i64().unwrap() as i16;
        let slot = warriors[member]
            .inventory()
            .unwrap()
            .iter()
            .position(|i| i.id() == id)
            .unwrap();
        let removed = equipment::remove(&warriors[member], &content, mission, slot).unwrap();
        assert!(removed.changed);
        warriors[member] = removed.warrior;
        audio.extend(removed.sounds);
        assert_party(&warriors, &case["removed_state"], &format!("remove{index}"));
        assert_eq!(audio, sounds(&case["removed_state"]));
        let reworn = equipment::wear(
            &warriors[member],
            &content,
            mission,
            slot,
            case["combat"].as_i64().unwrap() != 0,
        )
        .unwrap();
        assert!(reworn.changed);
        warriors[member] = reworn.warrior;
        audio.extend(reworn.sounds);
        assert_party(&warriors, &case["reworn_state"], &format!("rewear{index}"));
        assert_eq!(audio, sounds(&case["reworn_state"]));
        assert_eq!(case["reworn_state"]["rng_draws"].as_u64().unwrap(), 0);
    }
}

#[test]
fn replacement_firearm_preparation_shot_and_refill_match_four_native_callers() {
    let fixture = data();
    let content = content();
    let mut checked = 0;
    for (index, case) in fixture["equipment_cases"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        if case["fired"].is_null() {
            continue;
        }
        checked += 1;
        let member = case["member"].as_u64().unwrap() as usize;
        let source = case["source"].as_u64().unwrap() as usize;
        let mut state: Session =
            serde_json::from_value(fixture["foundations"][source]["session"].clone()).unwrap();
        state.warriors = party(case["reworn_state"]["party"].as_str().unwrap());
        items::compact(&mut state.warriors[member]).unwrap();
        assert_party(&state.warriors, &case["prepared"], "prepared party");
        let mission = content
            .map(state.exploration.as_ref().unwrap().map)
            .unwrap();
        let prepared = firearm::prepare(
            &state.warriors[member],
            &content.items[38],
            &content.items[203],
            &mission.combat.as_ref().unwrap().attacks,
        )
        .unwrap();
        assert_bytes(
            &prepared.packet,
            &bytes(case["prepared"]["packet"].as_str().unwrap()),
            "prepared packet",
        );
        assert_eq!(prepared.rounds, 1);
        assert_eq!(prepared.shots, 1);
        assert_eq!(
            word(&bytes(case["prepared"]["weapon"].as_str().unwrap()), 44).unwrap(),
            4101
        );
        let mut tiles = vec![1155; 8100];
        tiles[25 * 90 + 25] = 10;
        let mut order = vec![-1; 110];
        order[1] = member as i16;
        state.phase = Phase::Combat;
        state.combat = Some(Combat {
            battle: 85,
            map: TacticalMap {
                tiles,
                shift: (0, 0),
                edge_word: 0,
            },
            squad: SquadPlacement {
                positions: vec![(20, 10); 6],
                under_tiles: vec![1155; 6],
            },
            enemies: EnemySetup {
                records: vec![MonsterRecord {
                    bytes: bytes(case["prepared"]["target"].as_str().unwrap()),
                }],
                positions: vec![(20, 20)],
            },
            order,
            cursor: 1,
            round: 0,
            round_scripts: Default::default(),
            death_queue: Default::default(),
            spell_fields: vec![0; 1200],
            field_used: vec![0; 128],
            attack_packet: None,
            attack_identity: None,
            weapon_work: centurions_core::combat::weapon_work::WeaponWork::zero(),
            unavailable: 0,
        });
        state.random = RandomState::new(12);
        let events = runtime::player_shoot_action(&mut state, &content, 10).unwrap();
        assert_party(&state.warriors, &case["fired"], &format!("shot{index}"));
        assert_bytes(
            &state.combat.as_ref().unwrap().enemies.records[0].bytes,
            &bytes(case["fired"]["target"].as_str().unwrap()),
            "shot target",
        );
        assert_eq!(state.random.draws, 54);
        assert_eq!(
            state.random.seed,
            case["fired"]["rng_seed"].as_u64().unwrap() as u32
        );
        let expected_audio = &sounds(&case["fired"])[sounds(&case["targeted"]).len()..];
        assert_eq!(event_sounds(&events), expected_audio);
        state.ammunition[3] = 12;
        let events = firearm::reload_command(&mut state, &content, member).unwrap();
        assert_party(
            &state.warriors,
            &case["reloaded"],
            &format!("reload{index}"),
        );
        assert_eq!(state.ammunition[3], 9);
        assert_eq!(event_sounds(&events), vec![405]);
        assert_eq!(state.random.draws, 55);
        assert_eq!(
            state.random.seed,
            case["reloaded"]["rng_seed"].as_u64().unwrap() as u32
        );
    }
    assert_eq!(checked, 4);
}
