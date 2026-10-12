use crate::prelude::*;

use centurions_core::combat::ally_turns::item_context_after_load;
use centurions_core::combat::ally_turns::item_context_after_weight_calc;
use centurions_core::combat::ally_turns::natural_attack_enemy;
use centurions_core::combat::ally_turns::result_audio;
use centurions_core::combat::ally_turns::scientist_turn_with_item_context;

fn bytes(c: &serde_json::Value, k: &str) -> Vec<u8> {
    decode(c[k].as_str().unwrap())
}
fn records(b: &[u8]) -> Vec<MonsterRecord> {
    crate::support::monsters(b)
}
fn fixture() -> serde_json::Value {
    crate::support::fixture("ally-turns.json")
}
fn mission(f: &serde_json::Value, content: &centurions_content::Content) -> OpeningMission {
    let p = bytes(f, "properties");
    let mut m = content.map(0).unwrap().clone();
    m.tile_properties = p[64..12064].to_vec();
    m.terrain = 2;
    m.default_tile = 155;
    m
}
fn combat(case: &serde_json::Value, distance: i16) -> Combat {
    Combat {
        battle: 8,
        map: TacticalMap {
            tiles: if case["map_before"].is_string() {
                words(&bytes(case, "map_before"))
            } else {
                vec![1155; 8100]
            },
            shift: (0, 0),
            edge_word: 0,
        },
        squad: SquadPlacement {
            positions: (0..6).map(|i| (40 + i, 40)).collect(),
            under_tiles: vec![1155; 6],
        },
        enemies: EnemySetup {
            records: records(&bytes(case, "records_before")),
            positions: vec![(20, 20), (20 + distance, 20)],
        },
        order: vec![-1, 10],
        ..crate::support::battle()
    }
}
fn record_bytes(c: &Combat) -> Vec<u8> {
    crate::support::record_bytes(&c.enemies.records)
}
use crate::support::event_sounds as sounds;

#[test]
fn scientist_turn_matches_original_records_map_and_rng() {
    let f = fixture();
    let content = content();
    let mission = mission(&f, &content);
    for (index, case) in f.array("turn_cases").iter().enumerate() {
        let mut c = combat(case, case.n::<i16>("distance"));
        let mut w: Vec<_> = crate::support::warriors(&bytes(case, "warriors_before"));
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
        let expected = bytes(case, "records_after");
        let actual = record_bytes(&c);
        let diff: Vec<_> = actual
            .iter()
            .zip(&expected)
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(i, (a, b))| (i, a, b))
            .collect();
        assert!(
            diff.is_empty(),
            "case{index} differences{diff:?} draws{} seed{} sounds{:?}",
            random.draws,
            random.seed,
            sounds(&events)
        );
        assert_eq!(
            crate::support::record_bytes(&w),
            bytes(case, "warriors_after"),
            "party{index}"
        );
        assert_eq!(c.map.tiles, words(&bytes(case, "map_after")), "map{index}");
        let p = words(&bytes(case, "positions"));
        assert_eq!(
            c.enemies.positions,
            vec![(p[0], p[1]), (p[2], p[3])],
            "positions{index}"
        );
        crate::support::expect(random.draws, &case["rng_draws"]);
        assert_eq!(
            random.seed,
            u32::try_from(case.n::<u64>("seed_after")).unwrap(),
            "seed{index}"
        );
        crate::support::expect_array(&(sounds(&events)), &case["sounds"]);
    }
}
#[test]
fn monster_natural_attack_matches_original_scientist_damage() {
    let f = fixture();
    for (index, case) in f.array("attack_cases").iter().enumerate() {
        let mut c = combat(case, 0);
        c.enemies.positions = vec![(0, 0); 2];
        let mut w: Vec<Warrior> = crate::support::warriors(&bytes(case, "warriors_before"));
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
        assert_eq!(
            record_bytes(&c),
            bytes(case, "records_after"),
            "records{index} draws{}",
            random.draws
        );
        crate::support::native_expect! {
        scalar i16::from(result.killed) => &case["result"], "binary_ally_turns";
        scalar random.draws => &case["rng_draws"], "binary_ally_turns";
        scalar random.seed => &case["seed_after"], "binary_ally_turns";
        }
        assert_eq!(c.map.tiles, words(&bytes(case, "map_after")), "map{index}");
        assert_eq!(
            crate::support::record_bytes(&w),
            bytes(case, "warriors_after"),
            "party{index}"
        );
        let p = words(&bytes(case, "positions"));
        assert_eq!(
            c.enemies.positions,
            vec![(p[0], p[1]), (p[2], p[3])],
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
    let f = fixture();
    let content = content();
    for case in f["context_cases"].as_array().expect("context cases") {
        let result = if case["kind"] == "loaditem" {
            item_context_after_load(&content.items, case.n::<i16>("item_id")).unwrap()
        } else {
            let party = bytes(case, "warriors_before");
            let member = case.n::<usize>("member");
            let w = Warrior::read(&party[member * 3138..(member + 1) * 3138]).unwrap();
            item_context_after_weight_calc(&w, &content.items, 0x5a5a).unwrap()
        };
        crate::support::expect(result, &case["field32"]);
    }
}
