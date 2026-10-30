//! Shared decoding of native Psi combat snapshots; assertions stay with each case owner.
use crate::prelude::*;

pub fn combat(snapshot: &Value) -> Combat {
    let mut combat = crate::support::battle();
    combat.battle = 85;
    combat.field.tiles = snapshot.words("map");
    combat.squad.pos = crate::support::positions(&snapshot["party_positions"]);
    combat.roster.records = crate::support::monsters(&snapshot["records"]);
    combat.roster.mon_pos = crate::support::positions(&snapshot["monster_positions"]);
    if snapshot.get("order").is_some() {
        combat.battle_order = snapshot.words("order");
    }
    if snapshot.get("cursor").is_some() {
        combat.battle_order_index = snapshot.n("cursor");
    }
    if snapshot.get("round").is_some() {
        combat.combat_round = snapshot.n("round");
    }
    combat.spell_queue = snapshot.bytes("fields");
    combat.collide_check = snapshot.bytes("used");
    combat.spell_info = snapshot.bytes("packet");
    combat.cast_identity = snapshot.words("identity").try_into().unwrap();
    combat.kill_party = snapshot.n("unavailable");
    combat
}

/// Complete shared Psi record/work comparison; caller-specific state stays local.
pub fn compare(combat: &Combat, warriors: &[Warrior], snapshot: &Value, label: &str) {
    crate::support::native_expect! {
        bytes crate::support::record_bytes(warriors) => &snapshot["party"], label;
        bytes crate::support::record_bytes(&combat.roster.records) => &snapshot["records"], label;
        words combat.field.tiles => &snapshot["map"], label;
        bytes combat.spell_queue => &snapshot["fields"], label;
        bytes combat.collide_check => &snapshot["used"], label;
        bytes combat.spell_info => &snapshot["packet"], label;
        words combat.cast_identity.to_vec() => &snapshot["identity"], label;
    }
    assert_eq!(
        combat.squad.pos,
        crate::support::positions(&snapshot["party_positions"]),
        "{label} party positions"
    );
    assert_eq!(
        combat.roster.mon_pos,
        crate::support::positions(&snapshot["monster_positions"]),
        "{label} enemy positions"
    );
}

/// These historical snapshots record the three item/ammunition work projections,
/// not WORD4435d4. Its native controls and strict command/save comparison are separate.
pub fn captured_work(mut work: WeaponTraits) -> WeaponTraits {
    work.natural_attack_index = 0;
    work
}

pub fn compare_session(state: &Session, snapshot: &Value, label: &str) {
    let combat = state.combat.as_ref().unwrap();
    compare(combat, &state.warriors, snapshot, label);
    crate::support::native_expect! {
        bytes state.globals.define_spells => &snapshot["memory"], label;
        bytes state.party_condition => &snapshot["party_buffs"], label;
        words combat.battle_order => &snapshot["order"], label;
        scalar combat.battle_order_index => &snapshot["cursor"], label;
        scalar combat.combat_round => &snapshot["round"], label;
        scalar combat.kill_party => &snapshot["unavailable"], label;
        scalar state.globals.attack_count => &snapshot["count"], label;
    }
    assert_eq!(
        state.globals.loaded_item_field32,
        snapshot.n::<i16>("item_context")
    );
    let expected = crate::native_common::psi_work(snapshot);
    assert_eq!(captured_work(combat.weapon_traits), expected);
    assert_eq!(captured_work(state.globals.weapon_traits), expected);
    crate::support::assert_rng(&state.random, &snapshot["seed"], &snapshot["draws"]);
}
