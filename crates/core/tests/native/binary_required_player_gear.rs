use crate::{prelude::*, support};

use centurions_core::items;
use support::warriors as party;

use support::inventory as records;

use support::assert_bytes;

use support::compare_party_named as assert_party;

use support::event_sounds;

#[test]
fn equipment_admission_matches_all_12_native_race_and_mos_cases() {
    let fixture = support::fixture("required-player-gear.json");
    let content = content();
    let mission = content.map(0).unwrap();
    let cases = fixture.array("gate_cases");

    for case in cases {
        let warriors = party(case["before"].text("party"));
        let member = case.n::<usize>("member");
        let id = case.n::<usize>("item_id");
        assert_eq!(
            equipment::can_use(&warriors[member], &content.items[id], mission).unwrap(),
            case.n::<i64>("allowed") != 0
        );
        assert_party(&warriors, &case["after"], "admission unchanged");

        assert!(case["after"].n::<Vec<i16>>("sounds").is_empty());
    }
}

#[test]
fn crate_transfer_matches_18_native_whole_party_and_storage_cases() {
    let fixture = support::fixture("required-player-gear.json");
    let content = content();
    let cases = fixture.array("transfer_cases");

    for (index, case) in cases.iter().enumerate() {
        let source = case.n::<usize>("source");
        let slot = case.n::<usize>("crate_slot");
        let mut state: Session =
            serde_json::from_value(fixture["foundations"][source]["session"].clone()).unwrap();
        state.warriors = party(case["before"].text("party"));
        state.crate_items = records(case["before"].text("crate"));
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
        assert_bytes(&actual, &case["picked"].bytes("loot"), "picked loot");
        assert_eq!(result.collected, case.n::<i64>("collected") != 0);
        assert_eq!(result.sounds, case["picked"].n::<Vec<i16>>("sounds"));
        assert_eq!(result.warnings, case["picked"].n::<Vec<i16>>("warnings"));
        let events = equipment::take_crate(&mut state, &content, slot, 0).unwrap();
        assert_party(
            &state.warriors,
            &case["after"],
            &format!("transfer{index} closed"),
        );
        let actual: Vec<_> = state.crate_items.iter().flat_map(|r| r.0).collect();
        assert_bytes(&actual, &case["after"].bytes("crate"), "closed crate");
        assert_eq!(event_sounds(&events), case["after"].n::<Vec<i16>>("sounds"));
        assert_eq!(state.random.draws, 0);
        assert_eq!(state.random.seed, 12);
    }
}

#[test]
fn equipment_wear_compact_remove_and_rewear_match_48_native_cases() {
    let fixture = support::fixture("required-player-gear.json");
    let content = content();
    let cases = fixture.array("equipment_cases");

    for (index, case) in cases.iter().enumerate() {
        let source = case.n::<usize>("source");
        let member = case.n::<usize>("member");
        let slot = case.n::<usize>("inventory_slot");
        let mission = content
            .map(
                fixture["foundations"][source]["session"]["exploration"]["map"]
                    .as_i64()
                    .unwrap() as i32,
            )
            .unwrap();
        let mut warriors = party(case["before"].text("party"));
        let mut audio = case["before"].n::<Vec<i16>>("sounds");
        let result = equipment::wear(
            &warriors[member],
            &content,
            mission,
            slot,
            case.n::<i64>("combat") != 0,
        )
        .unwrap();
        assert_eq!(
            result.changed,
            case.n::<i64>("equipped") != 0,
            "wear{index} admitted"
        );
        warriors[member] = result.warrior;
        audio.extend(result.sounds);
        let mut expected_audio = case["worn"].n::<Vec<i16>>("sounds");
        // The historical probe returns at warn16474a before Sound6000.
        // Enabled wear still reaches that shared original warning wrapper.
        for warning in &result.warnings {
            assert_eq!(*warning, 29, "wear{index} CanUse rejection");
            expected_audio.push(6000);
        }
        assert_eq!(audio, expected_audio);
        assert_eq!(result.warnings, case["worn"].n::<Vec<i16>>("warnings"));
        assert_party(&warriors, &case["worn"], &format!("wear{index}"));
        if !result.changed {
            continue;
        }
        items::compact(&mut warriors[member]).unwrap();
        assert_party(&warriors, &case["compacted"], &format!("compact{index}"));
        let id = case.n::<i16>("item_id");
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
        assert_eq!(audio, case["removed_state"].n::<Vec<i16>>("sounds"));
        let reworn = equipment::wear(
            &warriors[member],
            &content,
            mission,
            slot,
            case.n::<i64>("combat") != 0,
        )
        .unwrap();
        assert!(reworn.changed);
        warriors[member] = reworn.warrior;
        audio.extend(reworn.sounds);
        assert_party(&warriors, &case["reworn_state"], &format!("rewear{index}"));
        assert_eq!(audio, case["reworn_state"].n::<Vec<i16>>("sounds"));
    }
}

#[test]
fn replacement_firearm_preparation_shot_and_refill_match_four_native_callers() {
    let fixture = support::fixture("required-player-gear.json");
    let content = content();
    for (index, case) in fixture.array("equipment_cases").iter().enumerate() {
        if case["fired"].is_null() {
            continue;
        }
        let member = case.n::<usize>("member");
        let source = case.n::<usize>("source");
        let mut state: Session =
            serde_json::from_value(fixture["foundations"][source]["session"].clone()).unwrap();
        state.warriors = party(case["reworn_state"].text("party"));
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
        support::native_expect! {
            bytes &prepared.packet => &case["prepared"].bytes("packet"), "prepared packet";
        };
        assert_eq!(prepared.rounds, 1);
        assert_eq!(prepared.shots, 1);
        assert_eq!(word(&case["prepared"].bytes("weapon"), 44).unwrap(), 4101);
        let mut tiles = vec![1155; 8100];
        tiles[25 * 90 + 25] = 10;
        let mut order = vec![-1; 110];
        order[1] = member as i16;
        state.phase = Phase::Combat;
        let mut combat = support::battle();
        combat.battle = 85;
        combat.field.tiles = tiles;
        combat.squad.pos.fill((20, 10));
        combat.roster.records = vec![MonsterRecord {
            bytes: case["prepared"].bytes("target"),
        }];
        combat.roster.mon_pos = vec![(20, 20)];
        combat.battle_order = order;
        state.combat = Some(combat);
        state.random = RandomState::new(12);
        let events = runtime::player_shoot_action(&mut state, &content, 10).unwrap();
        assert_party(&state.warriors, &case["fired"], &format!("shot{index}"));
        support::native_expect! {
            bytes &current_combat(&state).roster.records[0].bytes => &case["fired"].bytes("target"), "shot target";
        };
        assert_eq!(state.random.draws, 54);
        support::expect(state.random.seed, &case["fired"]["rng_seed"]);
        let expected_audio = &case["fired"].n::<Vec<i16>>("sounds")
            [case["targeted"].n::<Vec<i16>>("sounds").len()..];
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
        support::expect(state.random.seed, &case["reloaded"]["rng_seed"]);
    }
}
