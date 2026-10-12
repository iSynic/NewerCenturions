use crate::prelude::*;

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
    let party = crate::support::warriors(&read("party"));
    assert_eq!(party.len(), state.warriors.len());
    for (warrior, native) in state.warriors.iter_mut().zip(party) {
        warrior.bytes = native.bytes;
    }
    state.random = crate::native_common::random_state(snapshot);
    state.selected_warrior = selected;
    state.loaded_item_field32 = Some(snapshot.n::<i16>("item_context"));
    state.attack_count = snapshot.n::<i16>("count");
    state.bandage_ready = Some(snapshot.n::<i16>("aid_gate"));
    state.party_currency = snapshot.n::<i32>("currency");
    state.flags = read("flags");
    state.ammunition = crate::support::longs(&read("ammunition"));
    state.psi_memory = Some(read("memory"));
    state.party_buffs = read("party_buffs");
    state
        .party_mask
        .copy_from_slice(&crate::support::words(&read("mask"))[..6]);
}

pub fn health(state: &mut Session, snapshot: &Value, read: &dyn Fn(&str) -> Vec<u8>) {
    let health = state.health_context.as_mut().unwrap();
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
    let read = |key: &str| snapshot.bytes(key);
    session(
        state,
        snapshot,
        &read,
        snapshot.n::<usize>("selected_member"),
    );
    let combat = state.combat.as_mut().unwrap();
    records(combat, &read);
    geometry(
        combat,
        &read,
        (
            snapshot
                .n::<i16>("shift_x")
                .wrapping_sub(baseline.n::<i16>("shift_x")),
            snapshot
                .n::<i16>("shift_y")
                .wrapping_sub(baseline.n::<i16>("shift_y")),
        ),
    );
    combat.cursor = snapshot.n::<usize>("cursor");
    combat.round += (snapshot.n::<i64>("round") - baseline.n::<i64>("round")) as i16;
    combat.round_scripts.counter = snapshot.n::<i8>("round_script_counter");
    combat.round_scripts.slots = snapshot.words("round_slots").try_into().unwrap();
    spell_work(combat, &read);
    combat.unavailable = snapshot.n::<i8>("unavailable");
    combat.weapon_work = weapon_work(&read);
    state.attack_packet = combat.attack_packet.clone();
    state.attack_identity = combat.attack_identity;
    state.weapon_work = combat.weapon_work;
    health(state, snapshot, &read);
}
