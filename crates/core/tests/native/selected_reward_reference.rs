//! newland48's shared EXTRA reread and saved bodycount/selected returns.
use super::detached_attacks::compare;
use super::outdoor_deaths::prepare;
use crate::combat::round_stories::RoundDispatch;
use crate::combat::runtime::SelectedBattleTreasure;
use crate::test_support::{bytes, fixture};
use crate::{Command, Content, Event, Phase, Presentation, ScriptState, Session, story_cleanup};
use centurions_content::InventoryRecord;
use serde_json::{Value, json};

fn scalar(value: &Value) -> i16 {
    let raw = bytes(value);
    if raw.len() == 1 {
        i16::from(raw[0] as i8)
    } else {
        i16::from_be_bytes(raw.try_into().unwrap())
    }
}

fn words(value: &Value) -> Vec<i16> {
    bytes(value)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|raw| i16::from_be_bytes(*raw))
        .collect()
}

fn checkpoint(state: &mut Session, content: &Content) {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "centurions-selected-return-{}-{}.json",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    crate::save(&path, state, content).unwrap();
    let restored = crate::load(&path, content).unwrap();
    assert!(
        *state == restored,
        "selected return checkpoint changed state"
    );
    *state = restored;
    std::fs::remove_file(path).unwrap();
}

fn retained_input(source: &Value) -> (Session, Content) {
    let before = &source["before"];
    let count = (scalar(&before["party_last"]) + 1) as usize;
    assert!((1..=6).contains(&count));
    // Native storage retains six physical records; only partyLast+1 are named
    // warriors admitted to the source roster. The inactive tail stays captured.
    let mut logical = source.clone();
    logical["before"]["warriors"] = before["warriors"].as_str().unwrap()[..count * 3138 * 2].into();
    let (mut state, content) = prepare(&logical);
    state.selected_warrior = scalar(&before["selected_warrior"]) as usize;
    state.allies = bytes(&before["allies"]);
    state.ally_count = scalar(&before["ally_count"]) as usize;
    state.suppress_allies = scalar(&before["suppress_allies"]);
    state.command_preferences.auto_refit = scalar(&before["auto_refit"]) != 0;
    state.native_work.early_booty = scalar(&before["early_booty"]) as i8;
    state.native_work.battle_rewards_suppressed =
        scalar(&before["battle_rewards_suppressed"]) as i8;
    state.native_work.walking_confusion = scalar(&before["confusion_global"]);
    state.native_work.npc_backing.active_count = scalar(&before["npc_count"]);
    state.native_work.extra_record = bytes(&before["extra_work"]);
    state.native_work.treasure_record = bytes(&before["treasure_work"]);
    state.native_work.loot_quality = Some(bytes(&before["loot_flags"]));
    state.booty_items = bytes(&before["loot_scratch"])
        .as_chunks::<22>()
        .0
        .iter()
        .map(|raw| InventoryRecord(*raw))
        .collect();
    state.flags[..100].copy_from_slice(&bytes(&before["survivor_flags"]));
    state.native_work.inventory_record =
        InventoryRecord::read(&bytes(&before["selected_inventory_record"])).unwrap();
    state.ammunition = bytes(&before["ammunition_reserves"])[..800]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|raw| i32::from_be_bytes(*raw))
        .collect();
    state.native_work.round_work.counter = Some(scalar(&before["round_counter"]) as i8);
    state.native_work.round_work.slot = Some(scalar(&before["round_slot"]));
    state.native_work.round_work.processing = Some(scalar(&before["round_flag"]) as u8);
    state.native_work.round_work.slots = Some(words(&before["rounds"]).try_into().unwrap());
    (state, content)
}

fn started_owner(source: &Value, content: &mut Content, count: usize) -> Session {
    let entry = bytes(&source["entry_extra"]);
    let treasure = bytes(&source["treasure_record"]);
    for map in &mut content.scenario.as_mut().unwrap().maps {
        let shared = std::sync::Arc::make_mut(&mut map.shared);
        shared.combat.as_mut().unwrap().encounters[5350..5360].copy_from_slice(&entry);
        // The ordinary control declares a constructed fread buffer; early controls
        // supply the installed row. This changes private content inputs, never assets.
        shared.treasure[704..792].copy_from_slice(&treasure);
    }
    let mut state = Session::new(content);
    crate::apply(Command::ImportStarters, &mut state, content).unwrap();
    crate::apply(
        Command::EnterMission {
            name: "The Thunder of Guns".into(),
        },
        &mut state,
        content,
    )
    .unwrap();
    // The public start establishes typed owner/action history. Its played setup is
    // outside the oracle, which replaces Combat with declared retained RAM below.
    let starters = state.warriors.clone();
    state.warriors = (0..count)
        .map(|index| starters[index % starters.len()].clone())
        .collect();
    let position = state.exploration.as_mut().unwrap();
    position.script = Some(ScriptState::scheduled(
        bytes(&source["before"]["story_record"]),
        0,
    ));
    position.presentation = Some(Presentation::Dialogue {
        id: 0,
        text: String::new(),
    });
    crate::apply(Command::Continue, &mut state, content).unwrap();
    assert!(state.combat.is_some() && state.selected_battle_treasure.is_some());
    assert_eq!(
        state
            .exploration
            .as_ref()
            .unwrap()
            .script
            .as_ref()
            .unwrap()
            .cursor,
        1
    );
    assert_eq!(state.native_work.extra_record, entry);
    state
}

fn input(source: &Value, getup: bool) -> (Session, Content) {
    let (mut state, mut content) = retained_input(source);
    let mut started = started_owner(source, &mut content, state.warriors.len());
    let caller = started
        .exploration
        .as_ref()
        .unwrap()
        .script
        .clone()
        .unwrap();
    state.native_work.tactical_work.saved_land =
        started.native_work.tactical_work.saved_land.clone();
    state.battle_group_script = started.battle_group_script;
    state.battle_after_message = started.battle_after_message;
    state.battle_treasure = started.battle_treasure;
    state.native_work.extra_record = bytes(&source["controlled_postcombat_extra"]);
    if let Some(flags) = source["controlled_return_active_flags"].as_array() {
        assert_eq!(flags.len(), state.warriors.len());
        for (warrior, flag) in state.warriors.iter_mut().zip(flags) {
            warrior
                .record_mut()
                .set_active_flag(flag.as_u64().unwrap() as u8);
        }
    }
    let owner = started.selected_battle_treasure.take().unwrap();
    if getup {
        state.warriors[0].record_mut().set_movement_points(
            source["controlled_return_warrior0_movement"]
                .as_i64()
                .unwrap() as i16,
        );
        let work = &state.native_work.tactical_work;
        let mut arena = started.combat.take().unwrap();
        arena.map = work.map.clone();
        arena.squad.positions = work
            .positions
            .iter()
            .take(state.warriors.len())
            .copied()
            .collect();
        arena.squad.under_tiles = work
            .under_tiles
            .iter()
            .take(state.warriors.len())
            .copied()
            .collect();
        arena.retained_npcs = state.native_work.npc_backing.clone();
        arena.enemies.records = state.native_work.npc_backing.active_records().unwrap();
        arena.enemies.positions = state
            .native_work
            .npc_backing
            .slots
            .iter()
            .take(arena.enemies.records.len())
            .map(|slot| {
                let point = slot.as_ref().unwrap().position.unwrap();
                (
                    (point.0 - state.native_work.npc_backing.origin.0) as i16,
                    (point.1 - state.native_work.npc_backing.origin.1) as i16,
                )
            })
            .collect();
        arena.order = state.native_work.initiative_work.order.clone();
        arena.cursor = state.native_work.initiative_work.cursor;
        arena.round = state.native_work.turn_round.unwrap();
        arena.round_scripts.counter = state.native_work.round_work.counter.unwrap();
        arena.round_scripts.slots = state.native_work.round_work.slots.unwrap();
        arena.round_scripts.pending = None;
        arena.inactive_rounds = state.native_work.round_work.inactive_rounds.unwrap();
        arena.victory_work = state.native_work.victory_work;
        arena.neighbor_work = state.native_work.neighbor_work.clone();
        arena.ready = state.native_work.ready;
        arena.selected_enemy = state.native_work.selected_enemy;
        arena.script_enemy = state.native_work.script_enemy;
        arena.death_queue = state.native_work.retained_death_queue.clone().unwrap();
        arena.spell_fields = work.spell_fields.clone();
        arena.field_used = work.field_used.clone();
        arena.attack_packet = state.native_work.attack_packet.clone();
        arena.attack_identity = state.native_work.attack_identity;
        arena.weapon_work = state.native_work.weapon_work;
        arena.unavailable = state.native_work.health_context.unavailable;
        arena.selection_pending = false;
        arena.capture_actor = None;
        arena.turn_origin = None;
        arena.automatic = [false; 6];
        arena.direct_death = None;
        arena.pending_last_defeated = None;
        state.phase = Phase::Combat;
        state.combat = Some(arena);
        state.selected_battle_treasure = Some(owner);
        state.exploration.as_mut().unwrap().script = Some(caller);
        state.exploration.as_mut().unwrap().presentation = None;
    } else {
        // A completed retained round is a source-composed C0 return carrier. It
        // performs no death actor refresh or live-loop loadland before first booty.
        let mut declaration = serde_json::to_value(owner).unwrap();
        declaration["stage"] = "MovementCallback".into();
        state.selected_battle_treasure =
            Some(serde_json::from_value::<SelectedBattleTreasure>(declaration).unwrap());
        state.scripted_cleanup = Some(story_cleanup::Stage::Complete);
        state.round_dispatch = Some(RoundDispatch {
            retained: true,
            suspended_script: Some(caller),
            ..Default::default()
        });
        let position = state.exploration.as_mut().unwrap();
        position.script = Some(ScriptState::scheduled(vec![0; 42], 8));
        // This dismissal is constructed continuation metadata, not native UI.
        position.presentation = Some(Presentation::Messages {
            pages: vec![(0, String::new())],
            next: None,
        });
        // Literal controlled Combat return.
        state.native_work.early_booty = source["controlled_return_early_booty"]
            .as_i64()
            .unwrap_or(1) as i8;
    }
    state.validate(&content).unwrap();
    checkpoint(&mut state, &content);
    (state, content)
}

fn verify_stage(state: &Session, content: &Content, events: &[Event], case: &Value, count: usize) {
    let mut projection = case.clone();
    let preceding = case["comparison"]["rng_preceding_draws"]
        .as_u64()
        .unwrap_or(0);
    projection["draws"] = (case["draws"].as_u64().unwrap() + preceding).into();
    let final_load = case["comparison"]["final_loadland_noop"]
        .as_bool()
        .unwrap_or_else(|| case["pc"] == "0x5ffff0");
    if final_load {
        // Native loadland is a declared no-op file service. Source restores the
        // real start's saveland; those file-dependent owners are unmeasured here.
        projection["comparison"]["excluded_record_owners"] = json!(["map"]);
    }
    let party = bytes(&case["memory"]["warriors"]["hex"]);
    projection["memory"]["warriors"]["hex"] = party[..count * 3138]
        .iter()
        .map(|value| format!("{value:02x}"))
        .collect::<String>()
        .into();
    compare(state, events, &projection);
    for (name, actual) in [
        ("allies", state.allies.clone()),
        (
            "loot_scratch",
            state.booty_items.iter().flat_map(|item| item.0).collect(),
        ),
        (
            "loot_flags",
            state.native_work.loot_quality.clone().unwrap(),
        ),
        ("survivor_flags", state.flags[..100].to_vec()),
        ("extra_work", state.native_work.extra_record.clone()),
        ("treasure_work", state.native_work.treasure_record.clone()),
        (
            "current_shop_work",
            state.native_work.shop_work.record().to_vec(),
        ),
        (
            "selected_inventory_record",
            state.native_work.inventory_record.0.to_vec(),
        ),
        (
            "ammunition_reserves",
            state
                .ammunition
                .iter()
                .flat_map(|value| value.to_be_bytes())
                .collect(),
        ),
        (
            "rounds",
            state
                .native_work
                .round_work
                .slots
                .unwrap()
                .into_iter()
                .flat_map(i16::to_be_bytes)
                .collect(),
        ),
        (
            "death_queue",
            state
                .native_work
                .retained_death_queue
                .as_ref()
                .unwrap()
                .entries
                .iter()
                .flat_map(|entry| entry.bytes())
                .collect(),
        ),
    ] {
        let mut expected = bytes(&case["memory"][name]["hex"]);
        if name == "ammunition_reserves" {
            expected.truncate(800);
        }
        assert!(
            actual == expected,
            "{} {name}: {:?}",
            case["name"],
            actual
                .iter()
                .zip(&expected)
                .position(|(left, right)| left != right)
        );
    }
    for (name, actual) in [
        ("early_booty", state.native_work.early_booty as i16),
        (
            "battle_rewards_suppressed",
            state.native_work.battle_rewards_suppressed as i16,
        ),
        ("ally_count", state.ally_count as i16),
        ("npc_count", state.native_work.npc_backing.active_count),
        ("confusion_global", state.native_work.walking_confusion),
        (
            "character_work_bonus",
            state.native_work.character_bare_hand_bonus,
        ),
        (
            "round_counter",
            state.native_work.round_work.counter.unwrap() as i16,
        ),
        ("round_slot", state.native_work.round_work.slot.unwrap()),
        (
            "round_flag",
            state.native_work.round_work.processing.unwrap() as i16,
        ),
        (
            "death_queue_count",
            state
                .native_work
                .retained_death_queue
                .as_ref()
                .unwrap()
                .count as i16,
        ),
        (
            "death_queue_flag",
            state.native_work.death_processing.queue as i16,
        ),
    ] {
        assert_eq!(
            actual,
            scalar(&case["memory"][name]["hex"]),
            "{} {name}",
            case["name"]
        );
    }
    assert_eq!(
        state
            .native_work
            .retained_death_queue
            .as_ref()
            .unwrap()
            .pointer,
        i32::from_be_bytes(
            bytes(&case["memory"]["death_queue_pointer"]["hex"])
                .try_into()
                .unwrap()
        )
    );
    if !final_load {
        assert_eq!(
            state.exploration.as_ref().unwrap().event_headers,
            bytes(&case["memory"]["event_headers"]["hex"])
        );
    }
    for first in [0, 400, 200, 600, 800] {
        assert_eq!(
            content.items[first..first + 200]
                .iter()
                .flatten()
                .copied()
                .collect::<Vec<_>>(),
            bytes(&case["memory"][format!("definition_bank_{first}")]["hex"])
        );
    }
    if let Some(selection) = &state.survivors {
        let rows = words(&case["memory"]["survivor_rows"]["hex"]);
        assert_eq!(selection.eligible, [0]);
        assert_eq!(selection.selected, [rows[0] < 0]);
        assert_eq!(
            selection.limit,
            case["registers"]["D5"].as_u64().unwrap() as usize
        );
    }
}

fn normal_case(
    native: &Value,
    memory: &str,
    name: &str,
    calls: &[Value],
    final_load: bool,
) -> Value {
    let requests: Vec<_> = calls.iter().filter_map(|call| call.as_str().unwrap().strip_prefix("sound:")).map(|id| {
        json!({"id":"sound","arguments":{"id":format!("{:04x}",id.parse::<u16>().unwrap())}})
    }).collect();
    json!({"name":name,"memory":native[memory],"calls":requests,"draws":native["draws"],"seed":native["seed_after"],
        "comparison":{"rng_preceding_draws":0,"final_loadland_noop":final_load}})
}

#[test]
fn selected_reward_returns_match_native_shared_extra_and_owned_work() {
    let captures = fixture("selected-shared-extra.json");
    let find = |name: &str| {
        captures["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["name"] == name)
            .unwrap()
    };
    for name in [
        "selected-entry0-post8-early",
        "selected-entry8-post0-early",
        "selected-entry0-post8-bodycount-modal",
        "selected-combat-return-getup-bodycount-modal",
    ] {
        let case = find(name);
        let source = &case["declarations"]["source_input"];
        let getup = name.contains("getup-bodycount");
        let (mut state, content) = input(source, getup);
        let events = crate::apply(
            if getup {
                Command::EndTurn
            } else {
                Command::Continue
            },
            &mut state,
            &content,
        )
        .unwrap_or_else(|error| panic!("{name}: {error}"));
        verify_stage(&state, &content, &events, case, 6);
        if name.ends_with("modal") {
            assert!(state.selected_battle_treasure.is_some() && state.survivors.is_some());
            assert_eq!(
                state
                    .exploration
                    .as_ref()
                    .unwrap()
                    .script
                    .as_ref()
                    .unwrap()
                    .cursor,
                1
            );
            checkpoint(&mut state, &content);
            let events = crate::apply(Command::Continue, &mut state, &content).unwrap();
            verify_stage(
                &state,
                &content,
                &events,
                find(if getup {
                    "selected-Getup-confirm-through-first-and-bonus"
                } else {
                    "selected-bodycount-confirm-through-bonus"
                }),
                6,
            );
        }
        assert!(state.selected_battle_treasure.is_none() && state.survivors.is_none());
        assert!(state.exploration.as_ref().unwrap().script.is_none());
        assert_eq!(state.flags[5], 1);
    }
    let normal = &captures["normal_controls"][0]["native"];
    let mut source = captures["cases"][0]["declarations"]["source_input"].clone();
    source["before"] = normal["memory_before"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(key, row)| (key.clone(), row["hex"].clone()))
        .collect::<serde_json::Map<_, _>>()
        .into();
    source["logical_party_count"] = 3.into();
    source["entry_extra"] = normal["extra_entry"].clone();
    source["controlled_postcombat_extra"] = normal["extra_postcombat"].clone();
    source["treasure_record"] = normal["treasure"].clone();
    source["seed"] = normal["seed_before"].clone();
    source["draws"] = 0.into();
    source["controlled_return_early_booty"] = 0.into();
    // The normal verifier's controlled Combat service explicitly writes these
    // participating bytes. Its pre-service snapshot retains all three as one.
    source["controlled_return_active_flags"] = json!([1, 0, 1]);
    let (mut state, content) = input(&source, false);
    state.party_currency = normal["currency_before"].as_i64().unwrap() as i32;
    let calls = normal["calls"].as_array().unwrap();
    let closes: Vec<_> = calls
        .iter()
        .enumerate()
        .filter(|(_, call)| *call == "loot-Close-controlled-party-active1")
        .map(|(index, _)| index)
        .collect();
    let stages = [
        (
            "ordinary_prepared_memory",
            "selected-normal-first-prepared",
            &calls[..closes[0]],
            false,
        ),
        (
            "bonus_prepared_memory",
            "selected-normal-second-prepared",
            &calls[closes[0] + 1..closes[1]],
            false,
        ),
        (
            "memory_after",
            "selected-normal-final-return",
            &calls[closes[1] + 1..],
            true,
        ),
    ];
    for (memory, name, requests, final_load) in stages {
        checkpoint(&mut state, &content);
        let events = crate::apply(Command::Continue, &mut state, &content).unwrap();
        verify_stage(
            &state,
            &content,
            &events,
            &normal_case(normal, memory, name, requests, final_load),
            3,
        );
    }
    assert_eq!(
        state.party_currency,
        normal["currency_after"].as_i64().unwrap() as i32
    );
    assert!(state.selected_battle_treasure.is_none());
    assert_eq!(state.flags[5], 1);
}
