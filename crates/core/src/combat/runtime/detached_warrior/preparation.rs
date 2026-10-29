//! Loaded ITEM/AMMO/WEAPON and spell-weapon preflight in native call order.
use super::store_packet;
use crate::{Content, RuleError, Session};
use centurions_content::records::{DevelopmentField, ItemDefinition};

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
        // nobrassnoammo copies definition zero without attack2's subsequent WORD100 clear.
        state.load_item_work(content, 0)?;
        let item = state.native_work.ammunition_work.item.clone().unwrap();
        state.native_work.ammunition_work.ammo = Some(item.clone());
        state.native_work.ammunition_work.weapon = Some(item.clone());
        state.native_work.weapon_work.load_pair(&item, &item)?;
    } else {
        load_weapon(state, content, id)?;
        let spell =
            ItemDefinition(state.native_work.ammunition_work.weapon.as_ref().unwrap()).spell()?;
        if spell > 1100 {
            let attacks = &state
                .land(content)
                .and_then(|land| land.combat.as_ref())
                .ok_or("detached warrior spell tables missing")?
                .attacks;
            packet = crate::combat::spells::load_packet(attacks, spell)?;
            state.native_work.attack_identity = super::super::spell_identity(spell);
            store_packet(state, packet.clone());
            if packet[20] == 9 {
                let mut attacker = record(state)?;
                if attacker.record().automatic_mode()? == 0 {
                    return Ok(Prepared::Halt {
                        warned: state.selected_warrior as i16
                            == super::super::reaction_calls::retained_actor(state)?,
                    });
                }
                let bonus = state.native_work.character_bare_hand_bonus;
                if bonus.wrapping_add(attacker.record().bare_hand_damage()?) > 7 {
                    return Ok(Prepared::Halt { warned: false });
                }
                // toweap loads ITEM and stores its impact sound before reloading WEAPON.
                attacker.record_mut().set_firing_mode(false);
                state.store_table_record(actor, attacker)?;
                state.load_item_work(content, id)?;
                let sound =
                    ItemDefinition(state.native_work.ammunition_work.item.as_ref().unwrap())
                        .impact_sound()?;
                let mut attacker = record(state)?;
                DevelopmentField::MeleeSound.write(&mut attacker.bytes, sound);
                state.store_table_record(actor, attacker)?;
                load_weapon(state, content, id)?;
                rolls_condition = false;
            }
        }
    }
    let weapon = state.native_work.ammunition_work.weapon.clone().unwrap();
    Ok(Prepared::Strike {
        packet,
        rolls_condition,
        weapon,
        armed: id != 0,
    })
}

fn load_weapon(state: &mut Session, content: &Content, id: i16) -> Result<(), RuleError> {
    state
        .native_work
        .ammunition_work
        .load_weapon(&content.items, id)?;
    state
        .native_work
        .weapon_work
        .load_weapon(state.native_work.ammunition_work.weapon.as_ref().unwrap())?;
    Ok(())
}
