//! A warrior's item, ammunition and weapon, and the spell-weapon check.
use super::store_packet;
use crate::{Content, RuleError, Session};
use centurions_content::records::{DevelopmentField, ItemDefinition};

/// What a warrior strikes with: a melee strike with its weapon and attack packet, or a halt because
/// a spell weapon stopped the attack (with a warning when it was warned).
pub(super) enum Prepared {
    Strike {
        packet: Vec<u8>,
        rolls_condition: bool,
        weapon: Vec<u8>,
        armed: bool,
    },
    Halt {
        warned: bool,
    },
}

/// Loads the attacker's melee weapon, ammunition and spell-weapon packet; a spell weapon can halt
/// the attack.
///
/// Original: attack
pub(super) fn prepare(
    state: &mut Session,
    content: &Content,
    actor: usize,
    mut packet: Vec<u8>,
) -> Result<Prepared, RuleError> {
    // The attacker is any warrior table record: the squad or a held record.
    let record = |state: &Session| {
        state
            .table_record(actor)
            .ok_or_else(|| RuleError::from("detached warrior attacker outside the table"))
    };
    let id = record(state)?.record().melee_weapon()?;
    let mut rolls_condition = true;
    if id == 0 {
        rolls_condition = false;
        // nobrassnoammo loads definition zero; unlike attack2 it does not clear the
        // impact sound.
        state.load_item(content, 0)?;
        let item = state.globals.loaded_items.item.clone().unwrap();
        state.globals.loaded_items.ammo = Some(item.clone());
        state.globals.loaded_items.weapon = Some(item.clone());
        state.globals.weapon_traits.load_pair(&item, &item)?;
    } else {
        load_weapon(state, content, id)?;
        let spell = ItemDefinition(state.globals.loaded_items.weapon.as_ref().unwrap()).spell()?;
        if spell > 1100 {
            let attacks = &state
                .land(content)
                .and_then(|land| land.combat.as_ref())
                .ok_or("detached warrior spell tables missing")?
                .attacks;
            packet = crate::combat::spells::load_packet(attacks, spell)?;
            state.globals.cast_identity = super::super::spell_identity(spell);
            store_packet(state, packet.clone());
            if packet[20] == 9 {
                let mut attacker = record(state)?;
                if attacker.record().computer_control()? == 0 {
                    return Ok(Prepared::Halt {
                        warned: state.char_select_new as i16
                            == super::super::reaction_calls::turn_actor(state)?,
                    });
                }
                let bonus = state.globals.character_bare_hand_bonus;
                if bonus.wrapping_add(attacker.record().bare_hand_damage()?) > 7 {
                    return Ok(Prepared::Halt { warned: false });
                }
                // toweap loads the item and its impact sound before the weapon.
                attacker.record_mut().set_toggle(false);
                state.store_table_record(actor, attacker)?;
                state.load_item(content, id)?;
                let sound = ItemDefinition(state.globals.loaded_items.item.as_ref().unwrap())
                    .impact_sound()?;
                let mut attacker = record(state)?;
                DevelopmentField::MeleeSound.write(&mut attacker.bytes, sound);
                state.store_table_record(actor, attacker)?;
                load_weapon(state, content, id)?;
                rolls_condition = false;
            }
        }
    }
    let weapon = state.globals.loaded_items.weapon.clone().unwrap();
    Ok(Prepared::Strike {
        packet,
        rolls_condition,
        weapon,
        armed: id != 0,
    })
}

/// Loads a weapon and its traits.
fn load_weapon(state: &mut Session, content: &Content, id: i16) -> Result<(), RuleError> {
    state.globals.loaded_items.load_weapon(&content.items, id)?;
    state
        .globals
        .weapon_traits
        .load_weapon(state.globals.loaded_items.weapon.as_ref().unwrap())?;
    Ok(())
}
