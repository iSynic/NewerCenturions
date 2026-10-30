//! Firing at an occupied cell: preparing the shot, reloading and paying for targets.
use super::super::target_list;
use super::{CastRequest, Control, Outcome, unsupported};
use crate::combat::spells::FIRST_SPELL_DESCRIPTOR;
use crate::combat::{firearm, runtime::out_of_combat_attack::store_packet, spells::SpellTarget};
use crate::{Content, RuleError, Session, ammunition};
use centurions_content::records::{ItemDefinition, SpellInfoRecordMut};

/// A warrior in firing mode shoots the body in the cell. A weapon too empty for one shot is
/// reloaded. Otherwise, when the warrior has more than one attack left, each shot the weapon allows
/// takes its rounds and weight from the load and adds a target. With no shot, the warrior strikes
/// in melee.
///
/// Original: combatchoice
pub(super) fn apply(
    state: &mut Session,
    content: &Content,
    actor: usize,
    target: i16,
) -> Result<Outcome, RuleError> {
    prepare(state, content, actor)?;
    state.globals.attack_count = 0;
    let weapon = state
        .globals
        .loaded_items
        .weapon
        .as_deref()
        .ok_or_else(|| RuleError::from("WEAPON work missing"))?;
    let rounds = ItemDefinition(weapon).rounds_per_shot()?;
    // The load is checked in slot 0, whichever slot prepattack used.
    if state.warriors[actor].record().inventory_quantity(0)? < rounds {
        return reload(state, content, actor);
    }
    let mut targets = vec![];
    let mut discard_empty = false;
    if state.warriors[actor].record().attacks()? > 1 {
        let limit =
            ItemDefinition(state.globals.loaded_items.weapon.as_deref().unwrap()).attack_count()?;
        for _ in 0..limit {
            if state.warriors[actor].record().inventory_quantity(0)? >= rounds {
                let loaded = state.warriors[actor]
                    .record()
                    .inventory_quantity(0)?
                    .wrapping_sub(rounds);
                state.warriors[actor]
                    .record_mut()
                    .set_inventory_quantity(0, loaded);
                let ammo = state
                    .globals
                    .loaded_items
                    .ammo
                    .as_deref()
                    .ok_or_else(|| RuleError::from("AMMO work missing"))?;
                let weight = state.warriors[actor]
                    .record()
                    .load()?
                    .wrapping_sub(rounds.wrapping_mul(ItemDefinition(ammo).charge_weight()?));
                state.warriors[actor].record_mut().set_load(weight);
                let item = state
                    .globals
                    .loaded_items
                    .item
                    .as_deref()
                    .ok_or_else(|| RuleError::from("ITEM work missing"))?;
                if ItemDefinition(item).discarded_when_empty()? != 0 && loaded == 0 {
                    discard_empty = true;
                }
                state.globals.attack_count = state.globals.attack_count.wrapping_add(1);
                let kind = if target < 9 { 1 } else { 2 };
                if target_list::aliased(targets.len())
                    && !target_list::store(state, targets.len(), target, kind)
                {
                    return Err(unsupported(
                        "occupied firing target-list alias past the attack packet",
                    ));
                }
                targets.push(SpellTarget {
                    first: target,
                    second: 0,
                    kind,
                });
            }
        }
    }
    if targets.is_empty() {
        return super::melee::apply(state, content, actor, target);
    }
    Ok(Outcome {
        events: vec![],
        control: Control::Cast(CastRequest {
            actor,
            targets,
            discard_empty,
        }),
    })
}

/// Refits the weapon after a shot found it empty; the move ends. The warrior is released from
/// computer control and warned of the lack of ammunition when the refit does not complete.
///
/// Original: doreload
fn reload(state: &mut Session, content: &Content, actor: usize) -> Result<Outcome, RuleError> {
    // A reload prepares the shot again and ends the move.
    prepare(state, content, actor)?;
    let result = ammunition::refit_with_record(
        &state.warriors[actor],
        &state.ammunition,
        content,
        ammunition::RefitFlags {
            in_combat: true,
            force: true,
            silent: true,
            warnings: false,
        },
        &state.globals.loaded_items,
        &mut state.random,
        Some(&state.globals.inventory_record),
    )?;
    let completed = result.completed_refit();
    let mut events = ammunition::commit_result(state, actor, result)?;
    if !completed {
        state.warriors[actor].record_mut().set_computer_control(0);
        state.globals.possessed = 0;
        state.set_in_spell(0);
        events.push(crate::Event::Feedback(
            crate::feedback::GameFeedback::Warning {
                index: crate::feedback::OUT_OF_AMMUNITION_WARNING,
            },
        ));
    }
    Ok(Outcome::returned(events))
}

/// Loads the equipped weapon. When the equipped copy holds ammunition, also loads that, builds the
/// attack packet of a shot at the cell and takes the weapon spell's identity if it has one.
///
/// Original: prepattack
fn prepare(state: &mut Session, content: &Content, actor: usize) -> Result<(), RuleError> {
    let id = state.warriors[actor].record().equipped_weapon()?;
    state.globals.loaded_items.load_weapon(&content.items, id)?;
    let weapon = state.globals.loaded_items.weapon.clone().unwrap();
    state.globals.weapon_traits.load_weapon(&weapon)?;
    let description = ItemDefinition(&weapon).description_id()?;
    let inventory = state.warriors[actor].inventory()?;
    let Some((slot, item)) = inventory
        .iter()
        .enumerate()
        .find(|(_, item)| item.id() == description && item.equipped() == 1)
    else {
        return Ok(());
    };
    if item.charge()? == 0 {
        return Ok(());
    }
    let ammo = ammunition::definition(content, item.ammunition_id()?)?;
    state.globals.loaded_items.prepare_pair(&weapon, &ammo)?;
    state.globals.weapon_traits.prepare(&weapon, &ammo)?;
    let spell = ItemDefinition(&weapon).spell()?;
    let attacks = &state
        .land(content)
        .and_then(|map| map.combat.as_ref())
        .ok_or("occupied firing attack table missing")?
        .attacks;
    let mut packet = firearm::prepare_packet_with_enhancement(
        &weapon,
        &ammo,
        attacks,
        item.damage_enhancement(),
    )?;
    SpellInfoRecordMut(&mut packet).add_range_modifier(
        state.warriors[actor]
            .record()
            .inventory_slot(slot)
            .range_modifier(),
    );
    state.globals.attack_count = i16::from(ItemDefinition(&weapon).selected_attack_count());
    // A plain weapon shot keeps the previous spell identity.
    if spell >= FIRST_SPELL_DESCRIPTOR {
        state.globals.cast_identity = crate::combat::runtime::spell_identity(spell);
    }
    store_packet(state, packet);
    Ok(())
}
