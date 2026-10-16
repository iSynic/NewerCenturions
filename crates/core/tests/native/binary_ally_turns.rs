use crate::prelude::*;

use centurions_core::combat::ally_turns::item_context_after_load;
use centurions_core::combat::ally_turns::item_context_after_weight_calc;
use centurions_core::combat::ally_turns::natural_attack_enemy;
use centurions_core::combat::ally_turns::result_audio;
use centurions_core::combat::ally_turns::scientist_turn_with_item_context;

use crate::support::monsters as records;
fn mission(f: &serde_json::Value, content: &centurions_content::Content) -> OpeningMission {
    let p = f.bytes("properties");
    let mut m = content.map(0).unwrap().clone();
    m.tile_properties = p[64..12064].to_vec();
    m.terrain = 2;
    m.default_tile = 155;
    m
}
fn combat(case: &serde_json::Value, distance: i16) -> Combat {
    let mut combat = crate::support::battle();
    combat.battle = 8;
    combat.map.tiles = if case["map_before"].is_string() {
        case.words("map_before")
    } else {
        vec![1155; 8100]
    };
    combat.squad.positions = (0..6).map(|i| (40 + i, 40)).collect();
    combat.enemies.records = records(&case.bytes("records_before"));
    combat.enemies.positions = vec![(20, 20), (20 + distance, 20)];
    combat.order = vec![-1, 10];
    combat
}
use crate::support::event_sounds as sounds;

#[test]
fn scientist_turn_matches_original_records_map_and_rng() {
    let f = crate::support::fixture("ally-turns.json");
    let content = content();
    let mission = mission(&f, &content);
    for (index, case) in f.array("turn_cases").iter().enumerate() {
        let mut c = combat(case, case.n::<i16>("distance"));
        let mut w: Vec<_> = crate::support::warriors(&case.bytes("warriors_before"));
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let events = scientist_turn_with_item_context(
            &mut c,
            &mut w,
            &mission,
            &mut random,
            &content.items,
            case["item_context"].as_i64().unwrap_or(0) as i16,
        )
        .unwrap();
        crate::support::native_expect! {
        bytes crate::support::record_bytes(&c.enemies.records) => &case["records_after"], format!("ally{index}");
        bytes crate::support::record_bytes(&w) => &case["warriors_after"], format!("party{index}");
        words c.map.tiles => &case["map_after"], format!("map{index}");
        scalar random.seed => &case["seed_after"], format!("seed{index}");
        scalar random.draws => &case["rng_draws"], format!("draws{index}");
        scalar sounds(&events) => &case["sounds"], format!("sounds{index}");
        }
        assert_eq!(
            c.enemies.positions,
            crate::support::positions(&case.bytes("positions")),
            "positions{index}"
        );
    }
    // The same command must fire the original200-ammo rifles, including a converted hostile.
    for id in [64, 98, 35] {
        let case = &f["turn_cases"][1];
        let mut state = crate::support::mission_session(&content);
        let mut c = combat(case, 4);
        c.order.resize(110, -1);
        c.enemies.records[0] = centurions_core::combat::monster::summon(
            MonsterRecord {
                bytes: mission.combat.as_ref().unwrap().monsters[id * 288..(id + 1) * 288].to_vec(),
            },
            0,
            0,
            &mut state.random,
        )
        .unwrap();
        c.enemies.records[0].bytes[0x109] = 2;
        state.combat = Some(c);
        state.phase = centurions_core::Phase::Combat;
        state.loaded_item_field32 = Some(1);
        state.random = RandomState::new(12);
        let mut resumed = crate::support::checkpoint(&state, &content, "rifle-allies");
        let events = crate::support::replay_command(
            Command::RunEnemyTurn,
            &mut state,
            &mut resumed,
            &content,
        );
        assert!(
            sounds(&events).contains(&(case["sounds"][0].as_i64().unwrap() as i16)),
            "rifle{id}"
        );
    }
}
#[test]
fn monster_natural_attack_matches_original_scientist_damage() {
    let f = crate::support::fixture("ally-turns.json");
    for (index, case) in f.array("attack_cases").iter().enumerate() {
        let mut c = combat(case, 0);
        c.enemies.positions = vec![(0, 0); 2];
        let mut w: Vec<Warrior> = crate::support::warriors(&case.bytes("warriors_before"));
        let mut random = RandomState::new(case.n::<u32>("seed_before"));
        let result = natural_attack_enemy(
            &mut c,
            &mut w,
            &mut random,
            10,
            11,
            case.n::<usize>("attack"),
        )
        .unwrap();
        crate::support::native_expect! {
        bytes crate::support::record_bytes(&c.enemies.records) => &case["records_after"], format!("records{index}");
        bytes crate::support::record_bytes(&w) => &case["warriors_after"], format!("party{index}");
        words c.map.tiles => &case["map_after"], format!("map{index}");
        scalar i16::from(result.killed) => &case["result"], "binary_ally_turns";
        scalar random.draws => &case["rng_draws"], "binary_ally_turns";
        scalar random.seed => &case["seed_after"], "binary_ally_turns";
        }
        assert_eq!(
            c.enemies.positions,
            crate::support::positions(&case.bytes("positions")),
            "positions{index}"
        );
        let mut observed_sounds = result.sounds;
        // The direct native probe leaves the result-audio index word at zero.
        observed_sounds.push(result_audio(&c.enemies.records[0], 0, result.damage, 0).unwrap());
        crate::support::expect_array(&(observed_sounds), &case["sounds"]);
    }
}

#[test]
fn item_work_context_matches_original_load_and_weight_calculation() {
    let f = crate::support::fixture("ally-turns.json");
    let content = content();
    for case in f["context_cases"].as_array().expect("context cases") {
        let result = if case["kind"] == "loaditem" {
            item_context_after_load(&content.items, case.n::<i16>("item_id")).unwrap()
        } else {
            let party = case.bytes("warriors_before");
            let member = case.n::<usize>("member");
            let w = Warrior::read(&party[member * 3138..(member + 1) * 3138]).unwrap();
            item_context_after_weight_calc(&w, &content.items, 0x5a5a).unwrap()
        };
        crate::support::expect(result, &case["field32"]);
    }
}
