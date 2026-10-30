//! A monster's weapon and the positions its attack reads.
use crate::{Content, RuleError, Session};
use centurions_content::records::ItemDefinition;

/// What a monster strikes with: a natural attack (or its fallback when a melee weapon was refused)
/// or a weapon.
pub(super) enum Prepared {
    Natural { fallback: bool },
    Weapon(Vec<u8>),
}

/// Loads the weapon a monster strikes with: its melee weapon (else its primary weapon) when it
/// holds a weapon of item type 2, otherwise its natural attack.
pub(super) fn prepare(
    state: &mut Session,
    content: &Content,
    monster: &centurions_content::combat::MonsterRecord,
) -> Result<Prepared, RuleError> {
    let melee = monster.record().melee_weapon()?;
    if monster.record().weapon()? != 0 {
        let id = if melee != 0 {
            melee
        } else {
            monster.record().primary_weapon()?
        };
        state.globals.loaded_items.load_weapon(&content.items, id)?;
        let weapon = state.globals.loaded_items.weapon.as_ref().unwrap().clone();
        state.globals.weapon_traits.load_weapon(&weapon)?;
        if melee != 0 {
            state
                .globals
                .monsters
                .slot_mut(state.globals.monster_up)?
                .record
                .record_mut()
                .set_weapon(melee);
        }
        if ItemDefinition(&weapon).item_type()? == 2 {
            return Ok(Prepared::Weapon(weapon));
        }
        state
            .globals
            .monsters
            .slot_mut(state.globals.monster_up)?
            .record
            .record_mut()
            .set_weapon(0);
    }
    state.globals.loaded_items.load_natural(&content.items)?;
    let buffers = &state.globals.loaded_items;
    state.globals.weapon_traits.load_pair(
        buffers.weapon.as_ref().unwrap(),
        buffers.ammo.as_ref().unwrap(),
    )?;
    state.globals.loaded_item_field32 =
        ItemDefinition(buffers.item.as_ref().unwrap()).ammunition_caliber()?;
    Ok(Prepared::Natural {
        fallback: melee != 0,
    })
}

/// The columns of the attacker and its target, which decide which way the attacker faces.
pub(super) fn positions(
    state: &Session,
    slot: usize,
    target: usize,
) -> Result<(i16, i16), RuleError> {
    let backing = &state.globals.monsters;
    let origin = backing
        .slot(slot)?
        .position()?
        .0
        .wrapping_sub(backing.origin.0) as i16;
    // Party slots 8 and 9 read the battle order rows 1 and 3
    // (see docs/memory-alias-audit.md).
    let target = match state.globals.battlefield.pos.get(target) {
        Some(point) => point.0,
        None => state.globals.battle_order.order[target * 2 - 15],
    };
    Ok((origin, target))
}
