//! getup continuing after the battle has ended.
use super::TurnFlow;
use crate::combat::final_turn_stories::TurnProgram;
use crate::combat::spells;
use crate::{Content, Event, RuleError, Session};
mod actor;
pub(super) mod queue;
pub(super) mod reduction;
mod round;
pub(super) use reduction::collide as collide_fields;

/// Ends the active turn after the battle has ended. The turn's targets and attack kind are cleared;
/// the party is defeated when too many have fallen, and when no warrior is left the battle ends
/// with the early-booty flag (coward) set and the rewards follow. Otherwise a guarding body is
/// announced and the next turn is selected.
///
/// Original: getup
pub(super) fn finish(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    state.clear_targets();
    state.bandage_ready = 1;
    state.globals.item_used = 0;
    let packet = &mut state.globals.spell_info;
    centurions_content::records::SpellInfoRecordMut(packet).reset_turn_attack_kind();
    let kill_party = state.globals.health_context.kill_party;
    if state.warriors.len() as i16 - 1 < i16::from(kill_party) {
        return crate::mission::defeat(state, content);
    }
    let mut available = false;
    for warrior in &state.warriors {
        if warrior.record().in_battle() != 0 && warrior.record().faction()? == 0 {
            available = true;
        }
    }
    if !available {
        state.globals.kill_counts.finish();
        state.globals.coward = 1;
        if super::selected_rewards::moving(state) {
            return super::selected_rewards::begin_getup(state, content);
        }
        return super::out_of_combat_rewards::begin_getup(state, content);
    }
    let current = state
        .globals
        .battle_order
        .current(state.globals.monster_up)?;
    let guarding = if (0..9).contains(&current) {
        // Records past the squad, the Upgrade records and the editor backup.
        state
            .table_record(current as usize)
            .ok_or("getup body outside the warrior table")?
            .record()
            .guarding()?
            != 0
    } else if current > 9 {
        state
            .globals
            .monsters
            .slot(state.globals.monster_up)?
            .record
            .record()
            .guarding()
            != 0
    } else {
        false
    };
    let events = if guarding {
        vec![Event::Feedback(
            crate::feedback::GameFeedback::CombatResult {
                body: current,
                code: -5,
                amount: 0,
                audible: false,
            },
        )]
    } else {
        vec![]
    };
    state.set_in_spell(0);
    state.reset_monster_turn_flags();
    state.globals.possessed = 0;
    if (0..9).contains(&current)
        && state.warriors[current as usize]
            .record()
            .computer_control()?
            > 0
    {
        state.warriors[current as usize]
            .record_mut()
            .set_computer_control(0);
    }
    select(state, content, events)
}

/// getup's scan for the next actor after a round's reduction, once the battle has
/// ended.
///
/// Original: getup
pub(super) fn continue_after_reduction(
    state: &mut Session,
    content: &Content,
    events: Vec<Event>,
) -> Result<Vec<Event>, RuleError> {
    select(state, content, events)
}

/// Scans the battle order for the next body that can act. Spell fields tick, the death queue is
/// emptied, and a new round starts at the end of the order. Bodies that are absent, dead or
/// skipping turns are passed over; a warrior or monster that can act has its turn set up and
/// started, but a monster first runs its turn script when a roll against its script chance
/// succeeds.
///
/// Original: getup
fn select(
    state: &mut Session,
    content: &Content,
    mut events: Vec<Event>,
) -> Result<Vec<Event>, RuleError> {
    // Ten empty rounds can scan all 110 rows before inactivity ends the battle.
    for _ in 0..1211 {
        let initiative = &mut state.globals.battle_order;
        initiative.index = usize::from((initiative.index as u16).wrapping_add(1));
        let cursor = initiative.index;
        let battlefield = &mut state.globals.battlefield;
        battlefield.spell_queue = spells::tick_fields(&battlefield.spell_queue, cursor as i16)?;
        // With no monsters left, getup returns before clearing field use, emptying the
        // queue or running the round's work.
        if !state.globals.kill_counts.has_remaining() {
            return Ok(events);
        }
        battlefield.collide_check = spells::reset_used(&battlefield.collide_check)?;
        let queued = queue::drain(state, content)?;
        events.extend(queued.events);
        if queued.flow == TurnFlow::Terminated {
            return Ok(events);
        }
        if cursor >= 110 {
            match round::begin(state, content)? {
                round::Progress::Returned(suffix) | round::Progress::Terminated(suffix) => {
                    events.extend(suffix);
                    return Ok(events);
                }
                round::Progress::Reduced(suffix) => events.extend(suffix),
                round::Progress::Completed => continue,
            }
        }
        let body = state
            .globals
            .battle_order
            .current(state.globals.monster_up)?;
        if body < 0 {
            continue;
        }
        if body < 9 {
            let warrior = state
                .table_record(body as usize)
                .ok_or("getup body outside the warrior table")?;
            if warrior.record().in_battle() == 0 || warrior.record().turn_skip_duration()? != 0 {
                continue;
            }
            if body as usize >= state.warriors.len() {
                return Err(unsupported(
                    "getup selects a warrior record outside the squad",
                ));
            }
            state.char_select_new = body as usize;
            state.set_warrior_selector(body as usize);
            let mission = state.land(content).ok_or("retained turn map missing")?;
            let tables = mission
                .combat
                .as_ref()
                .ok_or("retained turn tables missing")?;
            if state.warriors[body as usize].record().faction()? != 0 {
                state.globals.possessed = 1;
                state.warriors[body as usize]
                    .record_mut()
                    .set_computer_control(1);
            }
            let (skipped, delta) = crate::combat::initialize_warrior_record(
                &mut state.warriors[body as usize],
                &tables.race_moves,
                &mut state.random,
            )?;
            let health = &mut state.globals.health_context;
            health.kill_party = health.kill_party.wrapping_add(delta);
            state.globals.kill_counts.add_num_enemy(i16::from(delta));
            if skipped {
                continue;
            }
            super::death_return::refresh_actor(state, content)?;
            if state.warriors[body as usize].record().computer_control()? == 0 {
                events.push(Event::Sound { resource_id: 138 });
            }
            events.extend(started(state, body)?);
            return Ok(events);
        }
        if body == 9 {
            return Err(unsupported("getup NPC body 9 before the NPC array"));
        }
        if body >= 110 {
            return Err(unsupported("getup NPC past the NPC array"));
        }
        let index = (body - 10) as usize;
        let record = state.globals.monsters.slot(index)?.record.record();
        if record.stamina()? < 1
            || record.mechanical_state() == 255
            || record.turn_skip_duration() != 0
        {
            continue;
        }
        state.globals.monster_up = index;
        let chance = record.turn_macro_chance()?;
        let script = record.turn_macro()?;
        if state.random.bound(100) <= chance {
            state.globals.macro_monster = index;
            events.extend(super::turn_stories::begin(
                state,
                content,
                body,
                TurnProgram { script, cursor: 0 },
                super::TurnCaller::OutOfCombat,
            )?);
            return Ok(events);
        }
        let (suffix, skipped) = actor::initialize(state, content, body)?;
        events.extend(suffix);
        if skipped {
            continue;
        }
        events.extend(started(state, body)?);
        return Ok(events);
    }
    Err("retained turn scan exceeded initiative capacity".into())
}

/// Returns to a monster's turn after its turn script ended: the monster is set up and started, or
/// the scan goes on if it has no stamina.
///
/// Original: getup
pub(super) fn resume_enemy(
    state: &mut Session,
    content: &Content,
    announce: bool,
) -> Result<Vec<Event>, RuleError> {
    if state.combat.is_some() {
        return Err("retained enemy return while combat is active".into());
    }
    if state
        .globals
        .monsters
        .slot(state.globals.monster_up)?
        .record
        .stamina()?
        == 0
    {
        return select(state, content, vec![]);
    }
    state.globals.macro_monster = 0;
    let body = state
        .globals
        .battle_order
        .current(state.globals.monster_up)?;
    let (mut events, skipped) = actor::initialize(state, content, body)?;
    if skipped {
        return select(state, content, events);
    }
    if announce {
        events.extend(started(state, body)?);
    }
    Ok(events)
}

/// The event that starts the turn of `body` in the current round.
fn started(state: &Session, body: i16) -> Result<Vec<Event>, RuleError> {
    let round = state
        .globals
        .turn_round
        .ok_or_else(|| RuleError::from("turn round missing after combat teardown"))?;
    Ok(vec![Event::TurnStarted { body, round }])
}

/// The refusal for a turn that depends on an unrecovered part of the game.
fn unsupported(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.retained_turn",
        dependency,
    }
}
