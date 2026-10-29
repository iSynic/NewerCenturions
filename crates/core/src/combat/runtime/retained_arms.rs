//! resolvespell's movement (151418) and summon (1516ea) arms over the tactical arrays
//! and NPC slots that outlive the combat-mode flag.
use super::{TurnFlow, outdoor_death};
use crate::combat::player_psi::{MoveArena, land_mover};
use crate::combat::spell_summon::{self, SummonArena, Summoned};
use crate::combat::{retained::NpcSlot, spells::detached::Arm};
use crate::{Content, Event, RuleError, Session};

/// Runs `arm` for WORD4435d2: the movement arm at the retained absolute cell
/// WORD443722/443724, the summon arm at the triple WORD442bd4 selects, `current`
/// inside a cast loop.
pub(crate) fn run(
    state: &mut Session,
    content: &Content,
    arm: Arm,
    (packet, id, current): (&[u8], i16, Option<(i16, i16)>),
    events: &mut Vec<Event>,
) -> Result<TurnFlow, RuleError> {
    let mission = state.land(content).ok_or("retained spell map missing")?;
    let actor = state.selected_warrior;
    if actor >= state.warriors.len() {
        // The session admits only a selected warrior inside the squad.
        return Err("retained movement or summon arm for an inactive selected warrior".into());
    }
    let work = &mut state.native_work;
    let shift = work.tactical_work.map.shift;
    let relative = |target: (i16, i16)| {
        (
            target.0.wrapping_sub(shift.0).wrapping_sub(5),
            target.1.wrapping_sub(shift.1).wrapping_sub(5),
        )
    };
    match arm {
        Arm::Movement => {
            let relative = relative(work.target_work.arm_cell()?);
            let (moved, defeated) = land_mover(
                MoveArena {
                    map: &mut work.tactical_work.map,
                    positions: &mut work.tactical_work.positions,
                    under_tiles: &mut work.tactical_work.under_tiles,
                    cursor: &mut work.initiative_work.cursor,
                },
                &mut state.warriors,
                mission,
                actor,
                packet,
                relative,
            )?;
            // The arm's showresults clears the marker, as the live caster's does.
            state.bandage_ready = 0;
            events.extend(moved);
            // killbody with combat mode clear takes its outdoor suffix.
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
            let count = work.npc_backing.active_count;
            if spell_summon::roster_full(count) {
                state.clear_targets();
                events.push(spell_summon::refuse(&mut state.native_work.target_work));
                state.set_ready(0);
                // combatupdate2(WORD4435d2) at 151730.
                super::death_return::refresh_body(state, content, actor as i16)?;
                return Ok(TurnFlow::Returned);
            }
            if count < 0 {
                return Err("negative retained NPC count".into());
            }
            // spelllist reads the triple only after the full-roster branch.
            let target = work.target_work.summon_cell(current)?;
            let relative = relative(target);
            let faction = state.warriors[actor].record().faction()?;
            // Before any combatsetup no grid record has been skipped, so the held
            // slots are exact.
            let grid_exact = work.tactical_work.battle.map_or(Some(true), |battle| {
                spell_summon::grid_leaves_slots_exact(battle, mission)
            });
            let slots: Vec<Option<NpcSlot>> = work.npc_backing.slots.clone();
            let summoned = spell_summon::place(
                SummonArena {
                    map: &mut work.tactical_work.map,
                    count: count as usize,
                    slots: &slots,
                    grid_exact,
                    order: &mut work.initiative_work.order,
                    cursor: work.initiative_work.cursor,
                    difficulty: state.difficulty,
                    template: work.monster_work.known(),
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
                    let backing = &mut state.native_work.npc_backing;
                    backing.slots[count as usize] = Some(NpcSlot {
                        record,
                        position: Some((i32::from(target.0), i32::from(target.1))),
                    });
                    backing.active_count = count + 1;
                    events.extend(placed);
                }
            }
        }
    }
    Ok(TurnFlow::Returned)
}
