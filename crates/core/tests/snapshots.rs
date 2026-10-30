use crate::prelude::*;

pub fn captured_party(state: &mut Session, fixture: &Value, snapshot: &Value) {
    let party = crate::support::warriors(&crate::support::buffer(fixture, &snapshot["party"]));
    assert_eq!(party.len(), state.warriors.len());
    for (warrior, native) in state.warriors.iter_mut().zip(party) {
        assert_eq!(warrior.name, native.name);
        warrior.bytes = native.bytes;
    }
}

/// Shared background for controlled helpers, not an earned campaign state.
pub fn helper_frame(fixture: &Value, snapshot: &Value, member: usize) -> Session {
    let background = crate::support::fixture("player-psi.json");
    let mut state: Session =
        serde_json::from_value(background["route_foundation"].clone()).unwrap();
    let read = |key: &str| crate::support::buffer(fixture, &snapshot[key]);
    let position = state.exploration.as_mut().unwrap();
    position.presentation = None;
    position.script = None;
    state.movement_continuation = None;
    state.medical_continuation = None;
    state.combat_story = None;
    state.handled_deaths.clear();
    captured_party(&mut state, fixture, snapshot);
    let reserves = read("reserves");
    state.ammunition = crate::support::longs(&reserves[..800]);
    state.globals.loot_quality = Some(read("loot_quality"));
    assert_eq!(
        &reserves[800..],
        &state.globals.loot_quality.as_ref().unwrap()[..4]
    );
    state.random = crate::native_common::random_state(snapshot);
    state.char_select_new = member;
    state.globals.inventory_record = InventoryRecord(read("selected_record").try_into().unwrap());
    state.flags = read("flags");
    state.party_condition = read("party_buffs");
    state.globals.define_spells = read("memory");
    state.crate_items = inventory(&read, "crate");
    state.booty_list = inventory(&read, "loot");
    state.globals.spell_info = read("packet");
    state.globals.cast_identity = crate::support::words(&read("identity")).try_into().unwrap();
    state.globals.attack_count = snapshot.n("count");
    state
        .globals
        .party_track
        .copy_from_slice(&crate::support::words(&read("mask"))[..6]);
    state.globals.health_context = HealthContext {
        kill_party: 0,
        char_select_new: member,
        effect: snapshot.n::<i8>("health_effect"),
        last_body: snapshot.n("health_last_body"),
        marks_affected: snapshot.n::<i16>("heal_marks_affected") != 0,
        affected: crate::support::words(&read("markers"))[..6]
            .try_into()
            .unwrap(),
    };
    let combat = state.combat.as_mut().unwrap();
    combat.field.tiles = crate::support::words(&read("map"));
    combat.field.map_shift = (
        snapshot.n::<i16>("shift_x") - 5,
        snapshot.n::<i16>("shift_y") - 5,
    );
    combat.squad.pos = crate::support::positions(&read("party_positions"));
    combat.battle_order = crate::support::words(&read("order"));
    combat.battle_order_index = snapshot.n("cursor");
    combat.combat_round = snapshot.n::<i16>("round_script_counter") - 1;
    combat.round_macros = RoundMacros::default();
    combat.todo_queue = TodoQueue::default();
    combat.spell_queue = read("fields");
    combat.collide_check = read("field_used");
    combat.spell_info = state.globals.spell_info.clone();
    combat.cast_identity = state.globals.cast_identity;
    combat.kill_party = 0;
    state
}

pub fn geometry(combat: &mut Combat, read: &dyn Fn(&str) -> Vec<u8>, delta: (i16, i16)) {
    for (positions, key) in [
        (&mut combat.squad.pos, "party_positions"),
        (&mut combat.roster.mon_pos, "monster_positions"),
    ] {
        *positions = crate::support::positions(&read(key))
            .into_iter()
            .map(|(x, y)| (x.wrapping_add(delta.0), y.wrapping_add(delta.1)))
            .collect();
    }
    combat.squad.char_under = crate::support::words(&read("under_tiles"));
    combat.field.tiles = crate::support::words(&read("map"));
    combat.battle_order = crate::support::words(&read("order"));
}

pub fn spell_work(combat: &mut Combat, read: &dyn Fn(&str) -> Vec<u8>) {
    combat.spell_queue = read("fields");
    combat.collide_check = read("used");
    combat.spell_info = read("packet");
    combat.cast_identity = crate::support::words(&read("identity")).try_into().unwrap();
}

pub fn records(combat: &mut Combat, read: &dyn Fn(&str) -> Vec<u8>) {
    let native = crate::support::monsters(&read("records"));
    assert_eq!(combat.roster.records.len(), native.len());
    combat.roster.records = native;
}

pub fn inventory(read: &dyn Fn(&str) -> Vec<u8>, key: &str) -> Vec<InventoryRecord> {
    let raw = read(key);
    let (records, tail) = raw.as_chunks::<22>();
    assert!(tail.is_empty());
    records
        .iter()
        .map(|record| InventoryRecord(*record))
        .collect()
}

pub fn medical_input(state: &mut Session, snapshot: &Value) {
    state.warriors = crate::support::warriors(&snapshot["party"]);
    state.random = crate::native_common::random_state(snapshot);
    let combat = state.combat.as_mut().unwrap();
    combat.spell_info = snapshot.bytes("packet");
    combat.cast_identity = snapshot.words("identity")[..3].try_into().unwrap();
    combat.battle_order_index = snapshot.n::<usize>("cursor");
    combat.combat_round = snapshot.n::<i16>("round");
    combat.battle_order = snapshot.words("order");
    combat.field.tiles = snapshot.words("map");
    combat.squad.pos = crate::support::positions(&snapshot.bytes("party_positions"));
    combat.squad.char_under = snapshot.words("under_tiles");
    combat.kill_party = snapshot.n::<i8>("unavailable");
    combat.spell_queue = snapshot.bytes("fields");
    combat.collide_check = snapshot.bytes("used");
    combat
        .weapon_traits
        .load_pair(&snapshot.bytes("weapon_work"), &snapshot.bytes("ammo_work"))
        .unwrap();
}

pub fn weapon_traits(read: &dyn Fn(&str) -> Vec<u8>) -> WeaponTraits {
    let weapon = read("weapon_work");
    let ammunition = read("ammo_work");
    WeaponTraits {
        natural_attack_index: 0,
        sound: word(&weapon, 14).unwrap(),
        weapon_armor_filter: word(&weapon, 136).unwrap(),
        ammunition_armor_filter: word(&ammunition, 136).unwrap(),
    }
}

pub fn session(
    state: &mut Session,
    snapshot: &Value,
    read: &dyn Fn(&str) -> Vec<u8>,
    selected: usize,
) {
    party_work(state, snapshot, read);
    state.char_select_new = selected;
    state.globals.loaded_item_field32 = snapshot.n::<i16>("item_context");
    state.bandage_ready = snapshot.n::<i16>("aid_gate");
    state.party_currency = snapshot.n::<i32>("currency");
    state.flags = read("flags");
    state.ammunition = crate::support::longs(&read("ammunition"));
}

pub fn actor_work(state: &mut Session, snapshot: &Value) {
    let combat = state.combat.as_mut().unwrap();
    if let Some(selected) = snapshot.get("current_npc_index").and_then(Value::as_u64) {
        combat.monster_up = selected as usize;
    } else if let Some(body) = combat.active_body().filter(|body| *body >= 10) {
        combat.monster_up = (body - 10) as usize;
    }
    state.globals.monster_up = combat.monster_up;
    state.globals.macro_monster = combat.macro_monster;
}

pub fn party_work(state: &mut Session, snapshot: &Value, read: &dyn Fn(&str) -> Vec<u8>) {
    let party = crate::support::warriors(&read("party"));
    assert_eq!(party.len(), state.warriors.len());
    for (warrior, native) in state.warriors.iter_mut().zip(party) {
        warrior.bytes = native.bytes;
    }
    state.random = crate::native_common::random_state(snapshot);
    state.globals.attack_count = snapshot.n::<i16>("count");
    state.globals.spell_info = read("packet");
    state.globals.cast_identity = crate::support::words(&read("identity")).try_into().unwrap();
    state.globals.define_spells = read("memory");
    state.party_condition = read("party_buffs");
    state
        .globals
        .party_track
        .copy_from_slice(&crate::support::words(&read("mask"))[..6]);
}

pub fn carried_items(state: &mut Session, read: &dyn Fn(&str) -> Vec<u8>) {
    state.crate_items = inventory(read, "crate");
    state.booty_list = inventory(read, "loot");
}

pub fn health(state: &mut Session, snapshot: &Value, read: &dyn Fn(&str) -> Vec<u8>) {
    let health = &mut state.globals.health_context;
    health.char_select_new = state.char_select_new;
    health.kill_party = state.combat.as_ref().unwrap().kill_party;
    health.effect = snapshot.n::<i8>("health_effect");
    health.last_body = snapshot.n::<i16>("health_last_body");
    health.marks_affected = snapshot.n::<i16>("heal_marks_affected") != 0;
    health
        .affected
        .copy_from_slice(&crate::support::words(&read("markers"))[..6]);
}

pub fn earned_projection(state: &mut Session, snapshot: &Value, baseline: &Value) {
    let before = state
        .combat
        .as_ref()
        .map(|c| (c.battle_order_index, c.combat_round));
    let read = |key: &str| snapshot.bytes(key);
    session(
        state,
        snapshot,
        &read,
        snapshot.n::<usize>("selected_member"),
    );
    let combat = state.combat.as_mut().unwrap();
    records(combat, &read);
    geometry(combat, &read, viewport_delta(snapshot, baseline));
    combat.battle_order_index = snapshot.n::<usize>("cursor");
    combat.combat_round += (snapshot.n::<i64>("round") - baseline.n::<i64>("round")) as i16;
    combat.round_macros.counter = snapshot.n::<i8>("round_script_counter");
    combat.round_macros.slots = snapshot.words("round_slots").try_into().unwrap();
    spell_work(combat, &read);
    combat.kill_party = snapshot.n::<i8>("unavailable");
    combat.weapon_traits = weapon_traits(&read);
    state.globals.spell_info = combat.spell_info.clone();
    state.globals.cast_identity = combat.cast_identity;
    state.globals.weapon_traits = combat.weapon_traits;
    health(state, snapshot, &read);
    turn_origin(state.combat.as_mut().unwrap(), before.unwrap());
    if before
        != state
            .combat
            .as_ref()
            .map(|c| (c.battle_order_index, c.combat_round))
    {
        actor_work(state, snapshot);
    }
}

pub fn viewport_delta(snapshot: &Value, baseline: &Value) -> (i16, i16) {
    (
        snapshot
            .n::<i16>("shift_x")
            .wrapping_sub(baseline.n::<i16>("shift_x")),
        snapshot
            .n::<i16>("shift_y")
            .wrapping_sub(baseline.n::<i16>("shift_y")),
    )
}

pub fn shifted_positions(
    read: &dyn Fn(&str) -> Vec<u8>,
    key: &str,
    delta: (i16, i16),
) -> Vec<(i16, i16)> {
    crate::support::positions(&read(key))
        .into_iter()
        .map(|(x, y)| (x.wrapping_add(delta.0), y.wrapping_add(delta.1)))
        .collect()
}

pub fn npc_projection(state: &mut Session, f: &Value, snapshot: &Value, baseline: &Value) {
    let before = {
        let c = state.combat.as_ref().unwrap();
        (c.battle_order_index, c.combat_round)
    };
    let read = |key: &str| crate::support::buffer(f, &snapshot[key]);
    let delta = viewport_delta(snapshot, baseline);
    party_work(state, snapshot, &read);
    state.flags = crate::support::buffer(f, &snapshot["flags"]);
    state.holdover = crate::support::buffer(f, &snapshot["references"]);
    carried_items(state, &read);
    let reserves = crate::support::buffer(f, &snapshot["reserves"]);
    state.ammunition = crate::support::longs(&reserves[..800]);
    state.globals.loaded_items.item = Some(crate::support::buffer(f, &snapshot["item_work"]));
    state.globals.loaded_items.weapon = Some(crate::support::buffer(f, &snapshot["weapon_work"]));
    state.globals.loaded_items.ammo = Some(crate::support::buffer(f, &snapshot["ammo_work"]));
    state.globals.loaded_item_field32 =
        word(state.globals.loaded_items.item.as_ref().unwrap(), 32).unwrap();
    state.char_select_new = snapshot.n::<usize>("active_member");
    state.bandage_ready = snapshot.n::<i16>("aid_gate");
    state.enemy_spell_disabled = snapshot.n::<u8>("enemy_spell_disabled");
    let count = snapshot.n::<usize>("body_count");
    let all_npcs = crate::support::buffer(f, &snapshot["npcs"]);
    assert!(
        all_npcs[count * MONSTER_BYTES..]
            .iter()
            .all(|byte| *byte == 0)
    );
    let positions = shifted_positions(&read, "npc_positions", delta);
    assert!(positions[count..].iter().all(|point| *point == (0, 0)));
    let combat = state.combat.as_mut().unwrap();
    combat.roster.records = crate::support::monsters(&all_npcs[..count * MONSTER_BYTES]);
    combat.roster.mon_pos = positions[..count].to_vec();
    combat.squad.pos = shifted_positions(&read, "party_positions", delta);
    combat.field.tiles = crate::native_common::buffer_words(f, &snapshot["map"]);
    combat.battle_order = crate::native_common::buffer_words(f, &snapshot["order"]);
    combat.battle_order_index = snapshot.n::<usize>("cursor");
    turn_origin(combat, before);
    combat.spell_queue = crate::support::buffer(f, &snapshot["fields"]);
    combat.collide_check = crate::support::buffer(f, &snapshot["field_used"]);
    combat.spell_info = crate::support::buffer(f, &snapshot["packet"]);
    combat.cast_identity = crate::native_common::buffer_words(f, &snapshot["identity"])
        .try_into()
        .unwrap();
    state.globals.spell_info = combat.spell_info.clone();
    state.globals.cast_identity = combat.cast_identity;
    let health = &mut state.globals.health_context;
    health.char_select_new = state.char_select_new;
    health.effect = snapshot.n::<i8>("health_effect");
    health.last_body = snapshot.n::<i16>("health_last_body");
    health.marks_affected = snapshot.n::<i64>("heal_marks_affected") != 0;
}

// Source getup1265c2..1265e0 stores the selected body's native turn-start coordinates.
pub fn turn_origin(combat: &mut Combat, before: (usize, i16)) {
    if before != (combat.battle_order_index, combat.combat_round) {
        combat.turn_origin = combat
            .active_body()
            .filter(|body| *body < 6)
            .map(|body| combat.squad.pos[body as usize]);
    }
}

// Source-derived undo metadata reconstructed from frozen prefix events, not new game output.
pub fn recorded_turn_origin(receipt: &Value, combat: &Combat) -> Option<(i16, i16)> {
    let body = combat.active_body().filter(|body| *body < 6)?;
    let mut origin = combat.squad.pos[body as usize];
    for (command, events) in receipt
        .array("commands")
        .iter()
        .zip(receipt.array("ordered_events"))
        .rev()
    {
        let events = events.as_array().unwrap();
        if events.iter().any(|event| {
            event
                .get("TacticalMoved")
                .is_some_and(|m| m.n::<i16>("body") == body)
        }) {
            let movement = &command["Move"];
            origin.0 -= movement.n::<i16>("dx");
            origin.1 -= movement.n::<i16>("dy");
        }
        if events
            .iter()
            .any(|event| event.get("TurnStarted").is_some())
        {
            return Some(origin);
        }
    }
    None
}
