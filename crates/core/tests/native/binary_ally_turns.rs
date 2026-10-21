use crate::{prelude::*, support};
use centurions_core::combat::ally_turns::{
    item_context_after_load, item_context_after_weight_calc, natural_attack_enemy, result_audio,
    scientist_turn_with_item_context,
};

use support::monsters as records;
fn mission(f: &serde_json::Value, content: &centurions_content::Content) -> OpeningMission {
    let p = f.bytes("properties");
    let mut m = content.map(0).unwrap().clone();
    m.tile_properties = p[64..12064].to_vec();
    m.terrain = 2;
    m.default_tile = 155;
    m
}
fn combat(case: &serde_json::Value, distance: i16) -> Combat {
    let mut combat = support::battle();
    combat.battle = 8;
    combat.map.tiles = if case["map_before"].is_string() {
        case.words("map_before")
    } else {
        vec![1155; 8100]
    };
    combat.squad.positions = (0..6).map(|i| (40 + i, 40)).collect();
    combat.enemies.records = records(&case.bytes("records_before"));
    combat.enemies.positions = vec![(20, 20), (20 + distance, 20)];
    // The original oracle starts from the captured zero NPC backing array.
    combat.retained_npcs = Session::new(&support::fixture_content("native backing")).npc_backing;
    combat.retained_npcs.origin = (5, 5);
    for slot in combat.retained_npcs.slots.iter_mut().flatten() {
        slot.position = (5, 5);
    }
    if let Some(position) = case.get("retained_slot99") {
        let [x, y]: [i32; 2] = serde_json::from_value(position.clone()).unwrap();
        combat.retained_npcs.slots[99].as_mut().unwrap().position = (x + 5, y + 5);
    }
    if let Some(target) = case.get("retained_target") {
        let [x, y]: [i32; 2] = target.n("position");
        let slot = combat.retained_npcs.slots[98].as_mut().unwrap();
        slot.record.bytes = target.bytes("record");
        slot.position = (x + 5, y + 5);
    }
    combat.order = vec![-1, 10];
    combat
}
use support::event_sounds as sounds;

#[test]
fn scientist_turn_matches_original_records_map_and_rng() {
    let f = support::fixture("ally-turns.json");
    let content = content();
    let mission = mission(&f, &content);
    let verified = support::fixture("rifle-retained-targets-final.json");
    let opaque = support::fixture("rifle-caller-contexts.json");
    let outer = support::fixture("rifle-getup-callers.json");
    for (index, case) in f
        .array("turn_cases")
        .iter()
        .chain(verified.array("cases"))
        .chain(
            opaque
                .array("cases")
                .iter()
                .filter(|case| case["mode"] == 1),
        )
        .enumerate()
    {
        let label = format!(
            "rifle MD{} distance{} mode{} context{}",
            case["monster_id"], case["distance"], case["mode"], case["item_context"]
        );
        let mut c = combat(case, case.n("distance"));
        let mut w = support::warriors(&case.bytes("warriors_before"));
        let mut rng = RandomState::new(case.n("seed_before"));
        let events = scientist_turn_with_item_context(
            &mut c,
            &mut w,
            &mission,
            &mut rng,
            &content.items,
            case["item_context"].as_i64().unwrap_or(0) as i16,
        )
        .unwrap();
        support::native_expect! {
            records &c.enemies.records => &case["records_after"], &label;
            records &w => &case["warriors_after"], "rifle party";
            words c.map.tiles => &case["map_after"], "rifle map";
            rng rng => (&case["seed_after"], &case["rng_draws"]), "rifle RNG";
            scalar sounds(&events) => &case["sounds"], "rifle sounds";
            positions c.enemies.positions => &case["positions"], "native positions";
        };
        if case.get("monster_id").is_none() {
            continue;
        }
        let mut state = support::mission_session(&content);
        state.warriors = support::warriors(&case.bytes("warriors_before"));
        let mut command_combat = combat(case, case.n("distance"));
        command_combat.order.resize(110, -1);
        command_combat.order[2] = 0;
        state.combat = Some(command_combat);
        state.phase = centurions_core::Phase::Combat;
        state.loaded_item_field32 = Some(case.n("item_context"));
        state.random = RandomState::new(12);
        if case["mode"] == 3 {
            let mut legacy = state.clone();
            legacy.combat.as_mut().unwrap().retained_npcs.slots[99] = None;
            support::reject_command(&legacy, &content, Command::RunEnemyTurn);
        }
        let command_events = support::saved_command(Command::RunEnemyTurn, &mut state, &content);
        let actual = current_combat(&state);
        let expected = &outer["cases"][index - f.array("turn_cases").len()];
        assert_eq!(case["map_before"], expected["map_before"]);
        support::native_expect! {
            records &actual.enemies.records => &expected["records_after"], "rifle command NPCs";
            records &state.warriors => &expected["warriors_after"], "rifle Getup party";
            bytes state.ammunition_work.item.as_ref().unwrap() => &expected["item_after"], "rifle item work";
            bytes state.ammunition_work.weapon.as_ref().unwrap() => &expected["weapon_after"], "rifle weapon work";
            bytes state.ammunition_work.ammo.as_ref().unwrap() => &expected["ammo_after"], "rifle ammo work";
            bytes state.attack_packet.as_ref().unwrap() => &expected["packet_after"], "rifle packet";
            scalar state.attack_count => &expected["count_after"], "rifle count";
            scalar actual.cursor => &expected["cursor_after"], "rifle next turn";
            scalar state.selected_warrior => &expected["selected_after"], "rifle selected warrior";
            positions actual.enemies.positions => &expected["positions"], "rifle final positions";
            scalar sounds(&command_events) => &expected["sounds"], "rifle ordered sounds";
            words actual.map.tiles => &expected["map_after"], "rifle outer map";
            rng state.random => (&expected["seed_after"], &expected["rng_draws"]), "rifle outer RNG";
        };
    }
}
#[test]
fn monster_natural_attack_matches_original_scientist_damage() {
    let f = support::fixture("ally-turns.json");
    for (index, case) in f.array("attack_cases").iter().enumerate() {
        let mut c = combat(case, 0);
        c.enemies.positions = vec![(0, 0); 2];
        let mut w: Vec<Warrior> = support::warriors(&case.bytes("warriors_before"));
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
        support::native_expect! {
            records &c.enemies.records => &case["records_after"], format!("records{index}");
            records &w => &case["warriors_after"], format!("party{index}");
            words c.map.tiles => &case["map_after"], format!("map{index}");
            scalar i16::from(result.killed) => &case["result"], "binary_ally_turns";
            scalar random.draws => &case["rng_draws"], "binary_ally_turns";
            scalar random.seed => &case["seed_after"], "binary_ally_turns";
            positions c.enemies.positions => &case.bytes("positions"), "positions{index}";
        };
        let mut observed_sounds = result.sounds;
        // The direct native probe leaves the result-audio index word at zero.
        observed_sounds.push(result_audio(&c.enemies.records[0], 0, result.damage, 0).unwrap());
        support::expect_array(&(observed_sounds), &case["sounds"]);
    }
}

#[test]
fn item_work_context_matches_original_load_and_weight_calculation() {
    let f = support::fixture("ally-turns.json");
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
        support::expect(result, &case["field32"]);
    }
}
