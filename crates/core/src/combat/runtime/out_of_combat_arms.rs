//! Movement and summon spells that resolve after the battle has ended.
use super::{TurnFlow, outdoor_death};
use crate::combat::player_psi::{MoveArena, land_mover};
use crate::combat::spells::summon::{self as spell_summon, SummonArena, Summoned};
use crate::combat::{globals::MonsterSlot, spells::out_of_combat::Arm};
use crate::{Content, Event, RuleError, Session};

/// Runs `arm` for the acting warrior (charup): movement at the first target's cell,
/// a summon at the cast's current target (`current` inside a cast).
///
/// Original: resolvespell
pub(crate) fn run(
    state: &mut Session,
    content: &Content,
    arm: Arm,
    (packet, id, current): (&[u8], i16, Option<(i16, i16)>),
    events: &mut Vec<Event>,
) -> Result<TurnFlow, RuleError> {
    let mission = state.land(content).ok_or("retained spell map missing")?;
    let actor = state.char_select_new;
    if actor >= state.warriors.len() {
        // The session admits only a selected warrior inside the squad.
        return Err("retained movement or summon arm for an inactive selected warrior".into());
    }
    let globals = &mut state.globals;
    let shift = globals.battlefield.field.map_shift;
    let relative = |target: (i16, i16)| {
        (
            target.0.wrapping_sub(shift.0).wrapping_sub(5),
            target.1.wrapping_sub(shift.1).wrapping_sub(5),
        )
    };
    match arm {
        Arm::Movement => {
            let relative = relative(globals.target_picks.arm_cell()?);
            let (moved, defeated) = land_mover(
                MoveArena {
                    map: &mut globals.battlefield.field,
                    positions: &mut globals.battlefield.pos,
                    under_tiles: &mut globals.battlefield.char_under,
                    cursor: &mut globals.battle_order.index,
                },
                &mut state.warriors,
                mission,
                actor,
                packet,
                relative,
            )?;
            // Showing the result clears the marker, as during combat.
            state.bandage_ready = 0;
            events.extend(moved);
            if defeated && outdoor_death::defeat(state, content, actor as i16, events)? {
                return Ok(TurnFlow::Terminated);
            }
            events.push(Event::SpellResolved {
                caster: None,
                spell: id,
                target: actor as i16,
                damage: 0,
                defeated,
                icon: crate::event::PresentationIcon(0),
                flight: Default::default(),
            });
        }
        Arm::Summon => {
            let count = globals.monsters.num_mon;
            if spell_summon::roster_full(count) {
                state.clear_targets();
                events.push(spell_summon::refuse(&mut state.globals.target_picks));
                state.set_in_spell(0);
                super::death_return::refresh_body(state, content, actor as i16)?;
                return Ok(TurnFlow::Returned);
            }
            if count < 0 {
                return Err("negative retained NPC count".into());
            }
            // A full roster refuses the summon before its target is read.
            let target = globals.target_picks.summon_cell(current)?;
            let relative = relative(target);
            let faction = state.warriors[actor].record().faction()?;
            // Before any combatsetup no grid record has been skipped, so the held
            // slots are exact.
            let grid_exact = globals.battlefield.battle.map_or(Some(true), |battle| {
                spell_summon::grid_leaves_slots_exact(battle, mission)
            });
            let slots: Vec<Option<MonsterSlot>> = globals.monsters.slots.clone();
            let summoned = spell_summon::place(
                SummonArena {
                    map: &mut globals.battlefield.field,
                    count: count as usize,
                    slots: &slots,
                    grid_exact,
                    order: &mut globals.battle_order.order,
                    cursor: globals.battle_order.index,
                    difficulty: state.difficulty,
                    template: globals.monster_pick.known(),
                },
                &state.warriors,
                mission,
                &mut state.random,
                (faction, relative),
                packet,
            )?;
            match summoned {
                Summoned::Refused(refused) => events.extend(refused),
                Summoned::Placed {
                    events: placed,
                    record,
                } => {
                    let backing = &mut state.globals.monsters;
                    backing.slots[count as usize] = Some(MonsterSlot {
                        record,
                        position: Some((i32::from(target.0), i32::from(target.1))),
                    });
                    backing.num_mon = count + 1;
                    events.extend(placed);
                }
            }
        }
    }
    Ok(TurnFlow::Returned)
}
