//! Shared decoding of native Psi combat snapshots; assertions stay with each case owner.
use crate::prelude::*;

pub fn combat(snapshot: &Value) -> Combat {
    let mut combat = crate::support::battle();
    combat.battle = 85;
    combat.map.tiles = snapshot.words("map");
    combat.squad.positions = crate::support::positions(&snapshot["party_positions"]);
    combat.enemies.records = crate::support::monsters(&snapshot["records"]);
    combat.enemies.positions = crate::support::positions(&snapshot["monster_positions"]);
    if snapshot.get("order").is_some() {
        combat.order = snapshot.words("order");
    }
    if snapshot.get("cursor").is_some() {
        combat.cursor = snapshot.n("cursor");
    }
    if snapshot.get("round").is_some() {
        combat.round = snapshot.n("round");
    }
    combat.spell_fields = snapshot.bytes("fields");
    combat.field_used = snapshot.bytes("used");
    combat.attack_packet = Some(snapshot.bytes("packet"));
    combat.attack_identity = Some(snapshot.words("identity").try_into().unwrap());
    combat.unavailable = snapshot.n("unavailable");
    combat
}

/// Complete shared Psi record/work comparison; caller-specific state stays local.
pub fn compare(combat: &Combat, warriors: &[Warrior], snapshot: &Value, label: &str) {
    crate::support::native_expect! {
        bytes crate::support::record_bytes(warriors) => &snapshot["party"], label;
        bytes crate::support::record_bytes(&combat.enemies.records) => &snapshot["records"], label;
        words combat.map.tiles => &snapshot["map"], label;
        bytes combat.spell_fields => &snapshot["fields"], label;
        bytes combat.field_used => &snapshot["used"], label;
        bytes combat.attack_packet.as_ref().unwrap() => &snapshot["packet"], label;
        words combat.attack_identity.unwrap().to_vec() => &snapshot["identity"], label;
    }
    assert_eq!(
        combat.squad.positions,
        crate::support::positions(&snapshot["party_positions"]),
        "{label} party positions"
    );
    assert_eq!(
        combat.enemies.positions,
        crate::support::positions(&snapshot["monster_positions"]),
        "{label} enemy positions"
    );
}

/// These historical snapshots record the three item/ammunition work projections,
/// not WORD4435d4. Its native controls and strict command/save comparison are separate.
pub fn captured_work(mut work: WeaponWork) -> WeaponWork {
    work.natural_attack_index = None;
    work
}

pub fn compare_session(state: &Session, snapshot: &Value, label: &str) {
    let combat = state.combat.as_ref().unwrap();
    compare(combat, &state.warriors, snapshot, label);
    crate::support::native_expect! {
        bytes state.psi_memory.as_ref().unwrap() => &snapshot["memory"], label;
        bytes state.party_buffs => &snapshot["party_buffs"], label;
        words combat.order => &snapshot["order"], label;
        scalar combat.cursor => &snapshot["cursor"], label;
        scalar combat.round => &snapshot["round"], label;
        scalar combat.unavailable => &snapshot["unavailable"], label;
        scalar state.attack_count => &snapshot["count"], label;
    }
    assert_eq!(state.loaded_item_field32, Some(snapshot.n("item_context")));
    let expected = crate::native_common::psi_work(snapshot);
    assert_eq!(captured_work(combat.weapon_work), expected);
    assert_eq!(captured_work(state.weapon_work), expected);
    crate::support::assert_rng(&state.random, &snapshot["seed"], &snapshot["draws"]);
}
