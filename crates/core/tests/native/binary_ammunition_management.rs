use crate::{native_common, prelude::*, snapshots, support};

use centurions_core::ammunition;

use ammunition::{AmmunitionResult, FitRequest, RefitFlags, WorkBuffers};

use support::buffer as bytes;

fn work(fixture: &Value, snapshot: &Value) -> WorkBuffers {
    WorkBuffers {
        item: Some(bytes(fixture, &snapshot["item_work"])),
        ammo: Some(bytes(fixture, &snapshot["ammo_work"])),
        weapon: Some(bytes(fixture, &snapshot["weapon_work"])),
    }
}

fn reserves(fixture: &Value, snapshot: &Value) -> Vec<i32> {
    support::longs(&bytes(fixture, &snapshot["reserves"])[..800])
}
use native_common::random_state as random;

fn compare(fixture: &Value, case: &Value, actual: AmmunitionResult, actual_random: &RandomState) {
    let after = &case["after"];
    assert_eq!(
        actual.warrior.bytes,
        bytes(fixture, &after["member_record"])
    );
    let mut party = bytes(fixture, &case["before"]["party"]);
    let member = case.n::<usize>("member");
    party[member * 3138..(member + 1) * 3138].copy_from_slice(&actual.warrior.bytes);
    assert_eq!(party, bytes(fixture, &after["party"]));
    assert_eq!(actual.reserves, reserves(fixture, after));
    assert_eq!(actual.work, work(fixture, after));
    support::native_expect! {
        scalar actual.native_return => &case["native_helper_return_word"], "binary_ammunition_management";
        scalar (actual.sounds) => &case["sounds"], "binary_ammunition_management";
        scalar (actual.warnings) => &case["warnings"], "binary_ammunition_management";
    }
    assert_eq!(actual_random, &random(after));
    let mut expected_random = random(&case["before"]);
    support::replay_rng(&mut expected_random, case.array("rng"));
    assert_eq!(actual_random, &expected_random);
    assert_eq!(actual.changed_ammo, after.n::<i64>("changed_ammo") != 0);
}

#[test]
fn refit_matches_original_manual_and_automatic_records_caches_returns_and_audio() {
    let content = content();
    let fixture = support::fixture("ammunition-management.json");
    for (index, case) in fixture
        .array("cases")
        .iter()
        .enumerate()
        .filter(|(_, case)| matches!(case.text("kind"), "manual_refit" | "automatic_refit"))
    {
        let before = &case["before"];
        let warrior = Warrior::read(&bytes(&fixture, &before["member_record"])).unwrap();
        let mut rng = random(before);
        let automatic = case["kind"] == "automatic_refit";
        let actual = ammunition::refit(
            &warrior,
            &reserves(&fixture, before),
            &content,
            RefitFlags {
                in_combat: case.n::<i64>("combat") != 0,
                force: automatic,
                silent: automatic,
                warnings: !automatic,
            },
            &work(&fixture, before),
            &mut rng,
        )
        .unwrap_or_else(|error| panic!("case {index}: {error}"));
        compare(&fixture, case, actual, &rng);
    }
    let reload = support::fixture("occupied-reload-valid-callers.json");
    for fixture in [&fixture, &reload] {
        for case in fixture
            .array("cases")
            .iter()
            .filter(|case| case["kind"] == "move_reload")
        {
            let before = &case["before_command"];
            let after = &case["after"];
            let read = |key: &str| bytes(fixture, &before[key]);
            let mut state = snapshots::helper_frame(fixture, before, case.n("member"));
            state.ammunition_work = work(fixture, before);
            let combat = state.combat.as_mut().unwrap();
            combat.enemies.records = support::monsters(&read("npcs"));
            combat.enemies.positions = support::positions(&read("npc_positions"));
            // Native control uses an unbuilt MD13 (zero HP); assist both input and unchanged NPC expectation.
            combat.enemies.records[0].bytes[0x110..0x112].copy_from_slice(&100_i16.to_be_bytes());
            let mut expected_npcs = bytes(fixture, &after["npcs"]);
            expected_npcs[0x110..0x112].copy_from_slice(&100_i16.to_be_bytes());
            combat.weapon_work = snapshots::weapon_work(&read);
            let events =
                support::saved_command(Command::Move { dx: 1, dy: 1 }, &mut state, &content);
            assert_eq!(
                support::record_bytes(&state.warriors),
                bytes(fixture, &after["party"])
            );
            assert_eq!(state.ammunition, reserves(fixture, after));
            assert_eq!(state.ammunition_work, work(fixture, after));
            assert_eq!(state.random, random(after));
            assert_eq!(
                state.inventory_record.as_ref().unwrap().0.as_slice(),
                bytes(fixture, &after["selected_record"])
            );
            let combat = current_combat(&state);
            assert_eq!(
                support::record_bytes(&combat.enemies.records),
                expected_npcs
            );
            assert_eq!(
                combat.map.tiles,
                support::words(&bytes(fixture, &after["map"]))
            );
            let warnings: Vec<_> = case
                .array("warning_phases")
                .iter()
                .map(|phase| phase.n::<i16>("argument"))
                .collect();
            let expected: Vec<_> = warnings
                .into_iter()
                .flat_map(|warning| {
                    [
                        Event::Sound { resource_id: 6000 },
                        Event::CollectionRejected { warning },
                    ]
                })
                .collect();
            assert_eq!(events, expected);
            if case["expanded_caller"] == true {
                assert_eq!(
                    state.attack_packet.as_ref().unwrap(),
                    &bytes(fixture, &after["packet"])
                );
                assert_eq!(state.attack_count, after.n::<i16>("count"));
            }
        }
    }
}

#[test]
fn accepted_fit_matches_original_whole_warrior_reserves_work_and_rng() {
    let content = content();
    let fixture = support::fixture("ammunition-management.json");
    for (index, case) in fixture
        .array("cases")
        .iter()
        .enumerate()
        .filter(|(_, case)| case["kind"] == "fit" && case["variant"] != "cancel_no_candidate")
    {
        let before = &case["before"];
        let warrior = Warrior::read(&bytes(&fixture, &before["member_record"])).unwrap();
        let mut rng = random(before);
        let adjacent = bytes(&fixture, &before["reserves"])[800..804]
            .try_into()
            .unwrap();
        let accepted = case["popup_inputs"][0]["accepted_ammo"].as_i64().unwrap() as usize;
        let actual = ammunition::fit(
            &warrior,
            &reserves(&fixture, before),
            &content,
            FitRequest {
                slot: case.n::<usize>("slot"),
                reserve_index: accepted - 200,
                in_combat: case.n::<i64>("combat") != 0,
                loot_quality: Some(adjacent),
            },
            &work(&fixture, before),
            &mut rng,
        )
        .unwrap_or_else(|error| panic!("case {index}: {error}"));
        compare(&fixture, case, actual, &rng);
    }
}
