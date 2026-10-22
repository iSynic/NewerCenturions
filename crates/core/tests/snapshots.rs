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
    state.native_work.loot_quality = Some(read("loot_quality"));
    assert_eq!(
        &reserves[800..],
        &state.native_work.loot_quality.as_ref().unwrap()[..4]
    );
    state.random = crate::native_common::random_state(snapshot);
    state.selected_warrior = member;
    state.native_work.inventory_record =
        Some(InventoryRecord(read("selected_record").try_into().unwrap()));
    state.flags = read("flags");
    state.party_buffs = read("party_buffs");
    state.native_work.psi_memory = Some(read("memory"));
    state.crate_items = inventory(&read, "crate");
    state.booty_items = inventory(&read, "loot");
    state.native_work.attack_packet = Some(read("packet"));
    state.native_work.attack_identity =
        Some(crate::support::words(&read("identity")).try_into().unwrap());
    state.native_work.attack_count = snapshot.n("count");
    state
        .native_work
        .party_mask
        .copy_from_slice(&crate::support::words(&read("mask"))[..6]);
    state.native_work.health_context = Some(HealthContext {
        unavailable: 0,
        selected_warrior: member,
        effect: snapshot.n::<i8>("health_effect"),
        last_body: snapshot.n("health_last_body"),
        marks_affected: snapshot.n::<i16>("heal_marks_affected") != 0,
        affected: crate::support::words(&read("markers"))[..6]
            .try_into()
            .unwrap(),
    });
    let combat = state.combat.as_mut().unwrap();
    combat.map.tiles = crate::support::words(&read("map"));
    combat.map.shift = (
        snapshot.n::<i16>("shift_x") - 5,
        snapshot.n::<i16>("shift_y") - 5,
    );
    combat.squad.positions = crate::support::positions(&read("party_positions"));
    combat.order = crate::support::words(&read("order"));
    combat.cursor = snapshot.n("cursor");
    combat.round = snapshot.n::<i16>("round_script_counter") - 1;
    combat.round_scripts = RoundScripts::default();
    combat.death_queue = DeathQueue::default();
    combat.spell_fields = read("fields");
    combat.field_used = read("field_used");
    combat.attack_packet = state.native_work.attack_packet.clone();
    combat.attack_identity = state.native_work.attack_identity;
    combat.unavailable = 0;
    state
}

pub fn geometry(combat: &mut Combat, read: &dyn Fn(&str) -> Vec<u8>, delta: (i16, i16)) {
    for (positions, key) in [
        (&mut combat.squad.positions, "party_positions"),
        (&mut combat.enemies.positions, "monster_positions"),
    ] {
        *positions = crate::support::positions(&read(key))
            .into_iter()
            .map(|(x, y)| (x.wrapping_add(delta.0), y.wrapping_add(delta.1)))
            .collect();
    }
    combat.squad.under_tiles = crate::support::words(&read("under_tiles"));
    combat.map.tiles = crate::support::words(&read("map"));
    combat.order = crate::support::words(&read("order"));
}

pub fn spell_work(combat: &mut Combat, read: &dyn Fn(&str) -> Vec<u8>) {
    combat.spell_fields = read("fields");
    combat.field_used = read("used");
    combat.attack_packet = Some(read("packet"));
    combat.attack_identity = Some(crate::support::words(&read("identity")).try_into().unwrap());
}

pub fn records(combat: &mut Combat, read: &dyn Fn(&str) -> Vec<u8>) {
    let native = crate::support::monsters(&read("records"));
    assert_eq!(combat.enemies.records.len(), native.len());
    combat.enemies.records = native;
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
    combat.attack_packet = Some(snapshot.bytes("packet"));
    combat.attack_identity = Some(snapshot.words("identity")[..3].try_into().unwrap());
    combat.cursor = snapshot.n::<usize>("cursor");
    combat.round = snapshot.n::<i16>("round");
    combat.order = snapshot.words("order");
    combat.map.tiles = snapshot.words("map");
    combat.squad.positions = crate::support::positions(&snapshot.bytes("party_positions"));
    combat.squad.under_tiles = snapshot.words("under_tiles");
    combat.unavailable = snapshot.n::<i8>("unavailable");
    combat.spell_fields = snapshot.bytes("fields");
    combat.field_used = snapshot.bytes("used");
    combat
        .weapon_work
        .load_pair(&snapshot.bytes("weapon_work"), &snapshot.bytes("ammo_work"))
        .unwrap();
}

pub fn weapon_work(read: &dyn Fn(&str) -> Vec<u8>) -> WeaponWork {
    let weapon = read("weapon_work");
    let ammunition = read("ammo_work");
    WeaponWork {
        natural_attack_index: None,
        sound: Some(word(&weapon, 14).unwrap()),
        weapon_armor_filter: Some(word(&weapon, 136).unwrap()),
        ammunition_armor_filter: Some(word(&ammunition, 136).unwrap()),
    }
}

pub fn session(
    state: &mut Session,
    snapshot: &Value,
    read: &dyn Fn(&str) -> Vec<u8>,
    selected: usize,
) {
    party_work(state, snapshot, read);
    state.selected_warrior = selected;
    state.native_work.loaded_item_field32 = Some(snapshot.n::<i16>("item_context"));
    state.bandage_ready = Some(snapshot.n::<i16>("aid_gate"));
    state.party_currency = snapshot.n::<i32>("currency");
    state.flags = read("flags");
    state.ammunition = crate::support::longs(&read("ammunition"));
}

pub fn actor_work(state: &mut Session, snapshot: &Value) {
    let combat = state.combat.as_mut().unwrap();
    if let Some(selected) = snapshot.get("current_npc_index").and_then(Value::as_u64) {
        combat.selected_enemy = selected as usize;
    } else if let Some(body) = combat.active_body().filter(|body| *body >= 10) {
        combat.selected_enemy = (body - 10) as usize;
    }
    state.native_work.selected_enemy = combat.selected_enemy;
    state.native_work.script_enemy = combat.script_enemy;
}

pub fn party_work(state: &mut Session, snapshot: &Value, read: &dyn Fn(&str) -> Vec<u8>) {
    let party = crate::support::warriors(&read("party"));
    assert_eq!(party.len(), state.warriors.len());
    for (warrior, native) in state.warriors.iter_mut().zip(party) {
        warrior.bytes = native.bytes;
    }
    state.random = crate::native_common::random_state(snapshot);
    state.native_work.attack_count = snapshot.n::<i16>("count");
    state.native_work.attack_packet = Some(read("packet"));
    state.native_work.attack_identity =
        Some(crate::support::words(&read("identity")).try_into().unwrap());
    state.native_work.psi_memory = Some(read("memory"));
    state.party_buffs = read("party_buffs");
    state
        .native_work
        .party_mask
        .copy_from_slice(&crate::support::words(&read("mask"))[..6]);
}

pub fn carried_items(state: &mut Session, read: &dyn Fn(&str) -> Vec<u8>) {
    state.crate_items = inventory(read, "crate");
    state.booty_items = inventory(read, "loot");
}

pub fn health(state: &mut Session, snapshot: &Value, read: &dyn Fn(&str) -> Vec<u8>) {
    let health = state.native_work.health_context.as_mut().unwrap();
    health.selected_warrior = state.selected_warrior;
    health.unavailable = state.combat.as_ref().unwrap().unavailable;
    health.effect = snapshot.n::<i8>("health_effect");
    health.last_body = snapshot.n::<i16>("health_last_body");
    health.marks_affected = snapshot.n::<i16>("heal_marks_affected") != 0;
    health
        .affected
        .copy_from_slice(&crate::support::words(&read("markers"))[..6]);
}

pub fn earned_projection(state: &mut Session, snapshot: &Value, baseline: &Value) {
    let before = state.combat.as_ref().map(|c| (c.cursor, c.round));
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
    combat.cursor = snapshot.n::<usize>("cursor");
    combat.round += (snapshot.n::<i64>("round") - baseline.n::<i64>("round")) as i16;
    combat.round_scripts.counter = snapshot.n::<i8>("round_script_counter");
    combat.round_scripts.slots = snapshot.words("round_slots").try_into().unwrap();
    spell_work(combat, &read);
    combat.unavailable = snapshot.n::<i8>("unavailable");
    combat.weapon_work = weapon_work(&read);
    state.native_work.attack_packet = combat.attack_packet.clone();
    state.native_work.attack_identity = combat.attack_identity;
    state.native_work.weapon_work = combat.weapon_work;
    health(state, snapshot, &read);
    turn_origin(state.combat.as_mut().unwrap(), before.unwrap());
    if before != state.combat.as_ref().map(|c| (c.cursor, c.round)) {
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
        (c.cursor, c.round)
    };
    let read = |key: &str| crate::support::buffer(f, &snapshot[key]);
    let delta = viewport_delta(snapshot, baseline);
    party_work(state, snapshot, &read);
    state.flags = crate::support::buffer(f, &snapshot["flags"]);
    state.allies = crate::support::buffer(f, &snapshot["references"]);
    carried_items(state, &read);
    let reserves = crate::support::buffer(f, &snapshot["reserves"]);
    state.ammunition = crate::support::longs(&reserves[..800]);
    state.native_work.ammunition_work.item =
        Some(crate::support::buffer(f, &snapshot["item_work"]));
    state.native_work.ammunition_work.weapon =
        Some(crate::support::buffer(f, &snapshot["weapon_work"]));
    state.native_work.ammunition_work.ammo =
        Some(crate::support::buffer(f, &snapshot["ammo_work"]));
    state.native_work.loaded_item_field32 =
        Some(word(state.native_work.ammunition_work.item.as_ref().unwrap(), 32).unwrap());
    state.selected_warrior = snapshot.n::<usize>("active_member");
    state.bandage_ready = Some(snapshot.n::<i16>("aid_gate"));
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
    combat.enemies.records = crate::support::monsters(&all_npcs[..count * MONSTER_BYTES]);
    combat.enemies.positions = positions[..count].to_vec();
    combat.squad.positions = shifted_positions(&read, "party_positions", delta);
    combat.map.tiles = crate::native_common::buffer_words(f, &snapshot["map"]);
    combat.order = crate::native_common::buffer_words(f, &snapshot["order"]);
    combat.cursor = snapshot.n::<usize>("cursor");
    turn_origin(combat, before);
    combat.spell_fields = crate::support::buffer(f, &snapshot["fields"]);
    combat.field_used = crate::support::buffer(f, &snapshot["field_used"]);
    combat.attack_packet = Some(crate::support::buffer(f, &snapshot["packet"]));
    combat.attack_identity = Some(
        crate::native_common::buffer_words(f, &snapshot["identity"])
            .try_into()
            .unwrap(),
    );
    state.native_work.attack_packet = combat.attack_packet.clone();
    state.native_work.attack_identity = combat.attack_identity;
    let health = state.native_work.health_context.as_mut().unwrap();
    health.selected_warrior = state.selected_warrior;
    health.effect = snapshot.n::<i8>("health_effect");
    health.last_body = snapshot.n::<i16>("health_last_body");
    health.marks_affected = snapshot.n::<i64>("heal_marks_affected") != 0;
}

// Source getup1265c2..1265e0 stores the selected body's native turn-start coordinates.
pub fn turn_origin(combat: &mut Combat, before: (usize, i16)) {
    if before != (combat.cursor, combat.round) {
        combat.turn_origin = combat
            .active_body()
            .filter(|body| *body < 6)
            .map(|body| combat.squad.positions[body as usize]);
    }
}

// Source-derived undo metadata reconstructed from frozen prefix events, not new game output.
pub fn recorded_turn_origin(receipt: &Value, combat: &Combat) -> Option<(i16, i16)> {
    let body = combat.active_body().filter(|body| *body < 6)?;
    let mut origin = combat.squad.positions[body as usize];
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
