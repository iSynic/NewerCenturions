//! Original initializer components; the placement caller's file and rectangle services are explicit.
use crate::support::{self, NativeCase};
use centurions_content::combat::{BattleRecord, MonsterRecord};
use centurions_core::combat::{TacticalMap, monster, place_enemies_at_difficulty};
use centurions_core::{RandomState, RuleError};
use serde_json::Value;

#[test]
fn signed_npc_levels_preserve_original_records_and_draws() {
    let fixture: Value = serde_json::from_slice(
        &std::fs::read(support::root().join("analysis/backlog-46/i17/native-initialization.json"))
            .unwrap(),
    )
    .unwrap();
    for case in fixture.array("cases") {
        let name = case.text("name");
        let before = &case["before"];
        let definition = MonsterRecord {
            bytes: before.bytes("definition"),
        };
        let mut random = RandomState::new(before.n("seed"));
        let record = if before.text("kind") == "summon" {
            monster::summon(definition, -1, before.n("difficulty"), &mut random).unwrap()
        } else {
            let mut mission = support::mission();
            let properties = std::fs::read(support::root().join(before.text("properties"))).unwrap();
            mission.tile_properties = properties[64..12064].to_vec();
            let battle = BattleRecord::read(&before.bytes("battle"), 0).unwrap();
            let initial_map = TacticalMap {
                tiles: before.words("map"),
                shift: (-5, -5),
                edge_word: 0,
            };
            let mut map = initial_map.clone();
            let mut definitions = vec![0; 288];
            definitions.extend_from_slice(&definition.bytes);
            let mut result = place_enemies_at_difficulty(
                &mut map,
                &mission,
                &battle,
                &definitions,
                (0, 0),
                &mut random,
                before.n("difficulty"),
            )
            .unwrap();
            assert_eq!(result.records.len(), case.n::<usize>("after_count"), "{name}");
            assert_eq!(map.tiles, case.words("after_map"), "{name} complete map");
            support::native_expect! {
                positions result.positions => &case.bytes("after_positions"), "signed placement";
            };
            // The native BYTE cursor wraps to -128 and can never reach level128.
            definitions[288..290].copy_from_slice(&128_i16.to_be_bytes());
            let mut blocked_map = initial_map.clone();
            let mut blocked_random = RandomState::new(before.n("seed"));
            let blocked = place_enemies_at_difficulty(
                &mut blocked_map,
                &mission,
                &battle,
                &definitions,
                (0, 0),
                &mut blocked_random,
                before.n("difficulty"),
            );
            assert!(matches!(blocked, Err(RuleError::Unsupported { .. })));
            assert_eq!(blocked_map, initial_map);
            assert_eq!(blocked_random, RandomState::new(before.n("seed")));
            result.records.remove(0)
        };
        assert_eq!(record.bytes, case.bytes("after_record"), "{name} complete NPC");
        assert_eq!(random.seed, case.n::<u32>("after_seed"), "{name} seed");
        assert_eq!(random.draws, case.n::<u64>("draws"), "{name} draws");
    }
}
