use crate::{prelude::*, support};

use centurions_core::story_npc_battle::{NpcBattleReturn, NpcBattleStart, finish};

#[test]
fn npc_battle_caller_matches_original_order_and_all_party_bytes() {
    let fixture: Value = support::fixture("npc-battle-caller.json");
    let scripts = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../NewCenturions/TheNewCenturions/Scenarios/The Thunder of Guns/0/Data ED3"
    ))
    .unwrap();
    for case in fixture.array("cases") {
        let mut header = case.bytes("header_before");
        let mut random = RandomState::new(case.n::<u32>("seed"));
        let mut start = NpcBattleStart::capture(&case.bytes("extra"), &header, 3).unwrap();
        let mut warriors: Vec<_> = support::warriors(&case["warriors_before"]);
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
        let outcome =
            start.return_kind(case.n::<u64>("outcome") == 1, case.n::<u64>("outcome") == 2);
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
                support::native_expect! {
                    bytes header => &case["header_at_booty"], "binary_npc_battle";
                    records &warriors => &case["warriors_at_booty"], "binary_npc_battle";
                }
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
        support::native_expect! {
            scalar battle => &case["battle"], "binary_npc_battle";
            bytes header => &case["header_after"], "binary_npc_battle";
            scalar cursor => &case["cursor_after"], "binary_npc_battle";
            bytes movement.as_slice() => &case["move_after"], "binary_npc_battle";
            scalar terrain_context => &case["terrain_context_after"], "binary_npc_battle";
            scalar map_dirty => &case["map_dirty_after"], "binary_npc_battle";
            records &warriors => &case["warriors_after"], "binary_npc_battle";
        }
        assert_eq!(events, case.array("events"));
        support::assert_rng(&random, &case["seed_after"], &case["rng_draws"]);
    }
}
