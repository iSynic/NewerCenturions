//! getup continues through shared turn work while the combat-mode flag is clear.
use super::TurnFlow;
use crate::{Content, Event, RuleError, Session};
mod actor;
pub(super) mod queue;
pub(super) mod reduction;
mod round;
pub(super) use reduction::collide as collide_fields;

pub(super) fn finish(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    // getup 125a50 calls cleartarget.
    state.clear_targets();
    state.bandage_ready = 1;
    // getup 125a92 clears WORD44719a.
    state.native_work.fire_item = 0;
    let packet = &mut state.native_work.attack_packet;
    centurions_content::records::AttackPacketMut(packet).reset_turn_attack_kind();
    let unavailable = state.native_work.health_context.unavailable;
    if state.warriors.len() as i16 - 1 < i16::from(unavailable) {
        return crate::mission::defeat(state, content);
    }
    let mut available = false;
    for warrior in &state.warriors {
        if warrior.record().active_flag() != 0 && warrior.record().faction()? == 0 {
            available = true;
        }
    }
    if !available {
        state.native_work.victory_work.finish();
        state.native_work.early_booty = 1;
        if super::selected_rewards::moving(state) {
            return super::selected_rewards::begin_getup(state, content);
        }
        return super::retained_rewards::begin_getup(state, content);
    }
    let current = state
        .native_work
        .initiative_work
        .current(state.native_work.selected_enemy)?;
    let guarding = if (0..9).contains(&current) {
        // Records past the squad, the Upgrade records and the editor backup 0x44ecb0.
        state
            .table_record(current as usize)
            .ok_or("getup body outside the warrior table")?
            .record()
            .guarding()?
            != 0
    } else if current > 9 {
        state
            .native_work
            .npc_backing
            .slot(state.native_work.selected_enemy)?
            .record
            .record()
            .guarding()
            != 0
    } else {
        false
    };
    let events = if guarding {
        vec![Event::Feedback(
            crate::feedback::NativeFeedback::CombatResult {
                body: current,
                code: -5,
                amount: 0,
                audible: false,
            },
        )]
    } else {
        vec![]
    };
    state.set_ready(0);
    state.reset_npc_work();
    state.native_work.walking_confusion = 0;
    if (0..9).contains(&current) && state.warriors[current as usize].record().automatic_mode()? > 0
    {
        state.warriors[current as usize]
            .record_mut()
            .set_automatic_mode(0);
    }
    select(state, content, events)
}

/// The detached getup scan after a round reduction that ran over retained records.
pub(super) fn continue_after_reduction(
    state: &mut Session,
    content: &Content,
    events: Vec<Event>,
) -> Result<Vec<Event>, RuleError> {
    select(state, content, events)
}

fn select(
    state: &mut Session,
    content: &Content,
    mut events: Vec<Event>,
) -> Result<Vec<Event>, RuleError> {
    // Ten ordinary empty rounds can scan all110 rows before inactivity completes.
    for _ in 0..1211 {
        let initiative = &mut state.native_work.initiative_work;
        initiative.cursor = usize::from((initiative.cursor as u16).wrapping_add(1));
        let cursor = initiative.cursor;
        let work = &mut state.native_work.tactical_work;
        work.spell_fields = crate::combat::spells::tick_fields(&work.spell_fields, cursor as i16)?;
        // 125cc8 returns before field-use clearing, emptyque or round work.
        if !state.native_work.victory_work.has_remaining() {
            return Ok(events);
        }
        work.field_used = crate::combat::spells::reset_used(&work.field_used)?;
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
            .native_work
            .initiative_work
            .current(state.native_work.selected_enemy)?;
        if body < 0 {
            continue;
        }
        if body < 9 {
            let warrior = state
                .table_record(body as usize)
                .ok_or("getup body outside the warrior table")?;
            if warrior.record().active_flag() == 0 || warrior.record().turn_skip_duration()? != 0 {
                continue;
            }
            if body as usize >= state.warriors.len() {
                return Err(unsupported(
                    "getup selects a warrior record outside the squad",
                ));
            }
            state.selected_warrior = body as usize;
            state.set_warrior_selector(body as usize);
            let mission = state.land(content).ok_or("retained turn map missing")?;
            let tables = mission
                .combat
                .as_ref()
                .ok_or("retained turn tables missing")?;
            if state.warriors[body as usize].record().faction()? != 0 {
                state.native_work.walking_confusion = 1;
                state.warriors[body as usize]
                    .record_mut()
                    .set_automatic_mode(1);
            }
            let (skipped, delta) = crate::combat::initialize_warrior_record(
                &mut state.warriors[body as usize],
                &tables.race_moves,
                &mut state.random,
            )?;
            let health = &mut state.native_work.health_context;
            health.unavailable = health.unavailable.wrapping_add(delta);
            state.native_work.victory_work.add_total(i16::from(delta));
            if skipped {
                continue;
            }
            super::death_return::refresh_actor(state, content)?;
            if state.warriors[body as usize].record().automatic_mode()? == 0 {
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
        let record = state.native_work.npc_backing.slot(index)?.record.record();
        if record.stamina()? < 1
            || record.mechanical_state() == 255
            || record.turn_skip_duration() != 0
        {
            continue;
        }
        state.native_work.selected_enemy = index;
        let chance = record.turn_script_chance()?;
        let script = record.final_turn_mode()?;
        if state.random.bound(100) <= chance {
            state.native_work.script_enemy = index;
            events.extend(super::turn_stories::begin(
                state,
                content,
                body,
                crate::combat::final_turn_stories::TurnProgram { script, cursor: 0 },
                super::TurnCaller::Retained,
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

pub(super) fn resume_enemy(
    state: &mut Session,
    content: &Content,
    announce: bool,
) -> Result<Vec<Event>, RuleError> {
    if state.combat.is_some() {
        return Err("retained enemy return while combat is active".into());
    }
    if state
        .native_work
        .npc_backing
        .slot(state.native_work.selected_enemy)?
        .record
        .stamina()?
        == 0
    {
        return select(state, content, vec![]);
    }
    state.native_work.script_enemy = 0;
    let body = state
        .native_work
        .initiative_work
        .current(state.native_work.selected_enemy)?;
    let (mut events, skipped) = actor::initialize(state, content, body)?;
    if skipped {
        return select(state, content, events);
    }
    if announce {
        events.extend(started(state, body)?);
    }
    Ok(events)
}

fn started(state: &Session, body: i16) -> Result<Vec<Event>, RuleError> {
    let round = state
        .native_work
        .turn_round
        .ok_or_else(|| RuleError::from("turn round missing after combat teardown"))?;
    Ok(vec![Event::TurnStarted { body, round }])
}

fn unsupported(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.retained_turn",
        dependency,
    }
}
