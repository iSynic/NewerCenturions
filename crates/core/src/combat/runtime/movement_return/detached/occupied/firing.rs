//! Occupied firing preparation, doreload and the paid target list before cast.
use super::super::target_list;
use super::{CastRequest, Control, Outcome, unsupported};
use crate::combat::{firearm, runtime::detached_attack::store_packet, spells::SpellTarget};
use crate::{Content, RuleError, Session, ammunition};
use centurions_content::records::{AttackPacketMut, ItemDefinition};

pub(super) fn apply(
    state: &mut Session,
    content: &Content,
    actor: usize,
    target: i16,
) -> Result<Outcome, RuleError> {
    prepare(state, content, actor)?;
    state.native_work.attack_count = 0;
    let weapon = state
        .native_work
        .ammunition_work
        .weapon
        .as_deref()
        .ok_or_else(|| RuleError::from("WEAPON work missing"))?;
    let rounds = ItemDefinition(weapon).rounds_per_shot()?;
    // This caller addresses row0, independently of prepattack's matching row.
    if state.warriors[actor].record().inventory_quantity(0)? < rounds {
        return reload(state, content, actor);
    }
    let mut targets = vec![];
    let mut discard_empty = false;
    if state.warriors[actor].record().action_points()? > 1 {
        let limit = ItemDefinition(state.native_work.ammunition_work.weapon.as_deref().unwrap())
            .attack_count()?;
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
                    .native_work
                    .ammunition_work
                    .ammo
                    .as_deref()
                    .ok_or_else(|| RuleError::from("AMMO work missing"))?;
                let weight = state.warriors[actor]
                    .record()
                    .carried_weight()?
                    .wrapping_sub(rounds.wrapping_mul(ItemDefinition(ammo).unit_weight()?));
                state.warriors[actor]
                    .record_mut()
                    .set_carried_weight(weight);
                let item = state
                    .native_work
                    .ammunition_work
                    .item
                    .as_deref()
                    .ok_or_else(|| RuleError::from("ITEM work missing"))?;
                if ItemDefinition(item).discarded_when_empty()? != 0 && loaded == 0 {
                    discard_empty = true;
                }
                state.native_work.attack_count = state.native_work.attack_count.wrapping_add(1);
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

fn reload(state: &mut Session, content: &Content, actor: usize) -> Result<Outcome, RuleError> {
    // doreload calls prepattack again, then always returns from this occupied path.
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
        &state.native_work.ammunition_work,
        &mut state.random,
        Some(&state.native_work.inventory_record),
    )?;
    let completed = result.completed_refit();
    let mut events = ammunition::commit_result(state, actor, result)?;
    if !completed {
        state.warriors[actor].record_mut().set_automatic_mode(0);
        state.native_work.walking_confusion = 0;
        state.set_ready(0);
        events.push(crate::Event::Feedback(
            crate::feedback::NativeFeedback::Warning {
                index: crate::feedback::OUT_OF_AMMUNITION_WARNING,
            },
        ));
    }
    Ok(Outcome::returned(events))
}

fn prepare(state: &mut Session, content: &Content, actor: usize) -> Result<(), RuleError> {
    let id = state.warriors[actor].record().equipped_weapon()?;
    state
        .native_work
        .ammunition_work
        .load_weapon(&content.items, id)?;
    let weapon = state.native_work.ammunition_work.weapon.clone().unwrap();
    state.native_work.weapon_work.load_weapon(&weapon)?;
    let description = ItemDefinition(&weapon).description_id()?;
    let inventory = state.warriors[actor].inventory()?;
    let Some((slot, item)) = inventory
        .iter()
        .enumerate()
        .find(|(_, item)| item.id() == description && item.equipment_flag() == 1)
    else {
        return Ok(());
    };
    if item.quantity()? == 0 {
        return Ok(());
    }
    let ammo = ammunition::definition(content, item.ammunition_id()?)?;
    state
        .native_work
        .ammunition_work
        .prepare_pair(&weapon, &ammo)?;
    state.native_work.weapon_work.prepare(&weapon, &ammo)?;
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
    AttackPacketMut(&mut packet).add_range_modifier(
        state.warriors[actor]
            .record()
            .inventory_slot(slot)
            .range_modifier(),
    );
    state.native_work.attack_count = i16::from(ItemDefinition(&weapon).selected_attack_count());
    // A fallback descriptor skips loadspell2 and keeps the previous identity.
    if spell >= 1101 {
        state.native_work.attack_identity = crate::combat::runtime::spell_identity(spell);
    }
    store_packet(state, packet);
    Ok(())
}
