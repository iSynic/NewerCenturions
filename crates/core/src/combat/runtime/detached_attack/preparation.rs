//! attack2 weapon selection and retained geometry reads.
use crate::{Content, RuleError, Session};
use centurions_content::records::ItemDefinition;

pub(super) enum Prepared {
    Natural { fallback: bool },
    Weapon(Vec<u8>),
}

pub(super) fn prepare(
    state: &mut Session,
    content: &Content,
    enemy: &centurions_content::combat::MonsterRecord,
) -> Result<Prepared, RuleError> {
    let melee = enemy.record().melee_weapon()?;
    if enemy.record().selected_item()? != 0 {
        let id = if melee != 0 {
            melee
        } else {
            enemy.record().primary_weapon()?
        };
        state
            .native_work
            .ammunition_work
            .load_weapon(&content.items, id)?;
        let weapon = state
            .native_work
            .ammunition_work
            .weapon
            .as_ref()
            .unwrap()
            .clone();
        state.native_work.weapon_work.load_weapon(&weapon)?;
        if melee != 0 {
            state
                .native_work
                .npc_backing
                .slot_mut(state.native_work.selected_enemy)?
                .record
                .record_mut()
                .set_selected_item(melee);
        }
        if ItemDefinition(&weapon).category()? == 2 {
            return Ok(Prepared::Weapon(weapon));
        }
        state
            .native_work
            .npc_backing
            .slot_mut(state.native_work.selected_enemy)?
            .record
            .record_mut()
            .set_selected_item(0);
    }
    state
        .native_work
        .ammunition_work
        .load_natural(&content.items)?;
    let buffers = &state.native_work.ammunition_work;
    state.native_work.weapon_work.load_pair(
        buffers.weapon.as_ref().unwrap(),
        buffers.ammo.as_ref().unwrap(),
    )?;
    state.native_work.loaded_item_field32 =
        ItemDefinition(buffers.item.as_ref().unwrap()).retained_field32()?;
    Ok(Prepared::Natural {
        fallback: melee != 0,
    })
}

pub(super) fn positions(
    state: &Session,
    slot: usize,
    target: usize,
) -> Result<(i16, i16), RuleError> {
    let backing = &state.native_work.npc_backing;
    let origin = backing
        .slot(slot)?
        .position()?
        .0
        .wrapping_sub(backing.origin.0) as i16;
    // Party positions 0x44709a run on into the order 0x4470b8: slots 8 and 9 begin at
    // order rows 1 and 3.
    let target = match state.native_work.tactical_work.positions.get(target) {
        Some(point) => point.0,
        None => state.native_work.initiative_work.order[target * 2 - 15],
    };
    Ok((origin, target))
}
