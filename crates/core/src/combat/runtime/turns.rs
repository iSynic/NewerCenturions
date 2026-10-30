//! Starting and ending turns.
use super::{advance_round, finish_action};
use crate::combat::final_turn_stories::TurnEffect;
use crate::combat::spells;
use crate::combat::{Combat, initialize_turn};
use crate::{Event, RandomState, RuleError, Session};
use centurions_content::{Content, Warrior, records::Condition};

/// Sets up the active body's turn. A warrior out of the battle or skipping turns is skipped; a
/// monster with a turn script runs it; a warrior's turn starts from its race moves, and a warrior
/// on automatic is put under computer control; a monster's faction change adjusts the hostile
/// count. Returns the events and whether the turn is skipped.
///
/// Original: getup
pub fn initialize_active_turn(
    combat: &mut Combat,
    warriors: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
) -> Result<(Vec<Event>, bool), RuleError> {
    let body = combat
        .active_body()
        .ok_or_else(|| RuleError::Invalid("active combatant missing".into()))?;
    let tables = mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("combat tables missing".into()))?;
    if body < 6
        && (warriors[body as usize].record().in_battle() == 0
            || warriors[body as usize].record().turn_skip_duration()? != 0)
    {
        return Ok((vec![], true));
    }
    if body >= 10 {
        combat.monster_up = (body - 10) as usize;
    }
    if body >= 10
        && combat.roster.records[(body - 10) as usize]
            .record()
            .turn_macro()?
            != 0
    {
        let turn = crate::combat::final_turn_stories::initialize(
            combat,
            warriors,
            mission,
            &tables.monsters,
            &tables.encounters,
            random,
        )?;
        let events = turn_feedback(mission, turn.effects)?;
        return finish_enemy_setup(combat, body, events, turn.skipped_turn);
    }
    combat.turn_origin = (body < 6).then(|| combat.squad.pos[body as usize]);
    if body < 6 {
        let (skipped, delta) = crate::combat::initialize_warrior_record(
            &mut warriors[body as usize],
            &tables.race_moves,
            random,
        )?;
        combat.kill_party = combat.kill_party.wrapping_add(delta);
        combat.kill_counts.add_num_enemy(i16::from(delta));
        if !skipped
            && combat.do_auto[body as usize]
            && warriors[body as usize].record().computer_control()? == 0
        {
            warriors[body as usize].record_mut().set_computer_control(1);
        }
        return Ok((vec![], skipped));
    }
    let prior_faction = combat.roster.records[(body - 10) as usize]
        .record()
        .faction()?;
    initialize_turn(
        body,
        warriors,
        &mut combat.roster.records,
        &tables.race_moves,
        random,
    )?;
    let faction = combat.roster.records[(body - 10) as usize]
        .record()
        .faction()?;
    combat
        .kill_counts
        .add_num_enemy(i16::from(faction != 0) - i16::from(prior_faction != 0));
    let skipped = body >= 10
        && combat.roster.records[(body - 10) as usize]
            .record()
            .turn_skip_duration()
            != 0;
    finish_enemy_setup(combat, body, vec![], skipped)
}

/// Turns the effects of a turn script into sounds, messages and monsters added or removed.
pub(super) fn turn_feedback(
    mission: &centurions_content::mission::OpeningMission,
    effects: Vec<TurnEffect>,
) -> Result<Vec<Event>, RuleError> {
    let mut events = vec![];
    for effect in effects {
        match effect {
            TurnEffect::Sound(resource_id) => events.push(Event::Sound { resource_id }),
            TurnEffect::Text(id) => events.push(Event::Message {
                id,
                text: mission.dialogue(id)?,
            }),
            TurnEffect::Spawn { body, monster } => events.push(Event::CombatantAdded {
                body,
                definition: monster,
            }),
            TurnEffect::SelfRemoval { body } => events.push(Event::CombatantRemoved { body }),
        }
    }
    Ok(events)
}

/// Finishes a monster's turn setup after its record is initialized, by testing its morale.
///
/// Original: getup
pub(super) fn finish_enemy_setup(
    combat: &mut Combat,
    body: i16,
    events: Vec<Event>,
    skipped: bool,
) -> Result<(Vec<Event>, bool), RuleError> {
    finish_monster_up_setup(combat, body, (body - 10) as usize, events, skipped)
}

/// Tests a monster's morale as its turn begins, unless the turn is skipped. A surrender chance
/// above 100 removes the monster from the field; a run chance above 100 makes it retreat; each
/// shows its message. A hostile monster that is removed counts as defeated.
///
/// Original: getup
pub(super) fn finish_monster_up_setup(
    combat: &mut Combat,
    body: i16,
    selected: usize,
    mut events: Vec<Event>,
    skipped: bool,
) -> Result<(Vec<Event>, bool), RuleError> {
    if skipped {
        return Ok((events, true));
    }
    let monster = combat.story_record_mut(selected)?;
    let Some((code, removed)) = enemy_morale(monster)? else {
        return Ok((events, false));
    };
    events.push(Event::Feedback(
        crate::feedback::GameFeedback::CombatResult {
            body,
            code,
            amount: 0,
            audible: false,
        },
    ));
    if removed {
        let hostile = monster.record().faction()? != 0;
        monster.record_mut().set_stamina(0);
        combat.field = crate::combat::footprint::restore(
            &combat.field,
            combat.story_record(selected)?,
            combat.story_position(selected)?,
        )?;
        // Every monster with a faction counts, mechanical or not.
        if hostile {
            combat.kill_counts.add_kill_mon(1);
        }
        events.push(Event::CombatantRemoved {
            body: selected as i16 + 10,
        });
    }
    Ok((events, removed))
}

/// Only monsters with stamina act, so the health ratio is always 100.
pub(super) fn enemy_morale(
    monster: &mut centurions_content::combat::MonsterRecord,
) -> Result<Option<(i16, bool)>, RuleError> {
    let chance = monster.record().surrender_percent()?;
    let removed = chance > 100;
    let code = if removed {
        if chance == 101 { -8 } else { -7 }
    } else if monster.record().run_percent()? > 100 {
        monster
            .record_mut()
            .set_condition(Condition::Retreating, -1);
        -6
    } else {
        return Ok(None);
    };
    Ok(Some((code, removed)))
}

/// Ends the active turn and selects the next one.
pub fn end_turn(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    select_turn(state, content, true)
}

/// Selects the next body to act, advancing past the current one if `advance` is set.
pub(super) fn select_turn(
    state: &mut Session,
    content: &Content,
    advance: bool,
) -> Result<Vec<Event>, RuleError> {
    select_turn_bounded(state, content, advance, 2)
}

/// Selects the next turn after the pause that follows a full pass of the battle order.
pub(crate) fn resume_selection(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    let combat = state.combat.as_mut().ok_or("combat selection missing")?;
    if !combat.selection_pending {
        return Err(RuleError::Invalid("combat selection is not pending".into()));
    }
    combat.selection_pending = false;
    select_turn_bounded(state, content, false, 2)
}

/// Selects the next body to act; after `remaining_wraps` passes of the whole battle order the
/// selection pauses.
pub(super) fn select_turn_bounded(
    state: &mut Session,
    content: &Content,
    advance: bool,
    remaining_wraps: u8,
) -> Result<Vec<Event>, RuleError> {
    select_turn_inner(state, content, advance, remaining_wraps, advance)
}

/// Moves on to the next body after a monster's turn script left its turn skipped or spent, the way
/// getup does from the top of its scan; the turn flags are not reset.
///
/// Original: getup
pub(super) fn resume_getup(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    select_turn_inner(state, content, true, 2, false)
}

/// Walks the battle order to the next body that can act. When advancing, the departing body's
/// guard is announced, the position in the order moves on and the spell fields tick; deaths queued
/// by the last action run next, and no hostile monster left ends the action. A new round starts at
/// the end of the order. Bodies that are dead, absent or skipping turns are passed over; the first
/// body able to act has its turn set up and started.
///
/// Original: getup
fn select_turn_inner(
    state: &mut Session,
    content: &Content,
    mut advance: bool,
    remaining_wraps: u8,
    mut reset_confusion: bool,
) -> Result<Vec<Event>, RuleError> {
    let mut events = vec![];
    if advance {
        let combat = state
            .combat
            .as_mut()
            .ok_or_else(|| RuleError::Invalid("combat missing".into()))?;
        combat.capture_actor = None;
        combat.turn_origin = None;
        state.clear_targets();
        state.bandage_ready = 1;
        state.globals.item_used = 0;
    }
    let c = state
        .combat
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("combat missing".into()))?;
    if advance {
        centurions_content::records::SpellInfoRecordMut(&mut c.spell_info).reset_turn_attack_kind();
    }
    if state.selected_battle_treasure.is_some()
        && i16::from(c.kill_party) >= state.warriors.len() as i16
    {
        return crate::mission::defeat(state, content);
    }
    if !super::has_active_warrior(&state.warriors) {
        c.kill_counts.finish();
        state.globals.coward = 1;
        if state.selected_battle_treasure.is_some() {
            return super::selected_rewards::begin_getup(state, content);
        }
        return crate::mission::defeat(state, content);
    }
    if advance
        && let Some(body) = c.active_body()
        && body < 10
    {
        let warrior = &mut state.warriors[body as usize];
        if warrior.record().computer_control()? > 0 {
            warrior.record_mut().set_computer_control(0);
        }
    }
    let mut announce_guard = advance;
    for _ in 0..220 {
        let c = state.combat.as_mut().ok_or("combat missing")?;
        if advance {
            // The current guard is shown before the turn advances.
            if announce_guard && let Some(body) = c.active_body() {
                let guarding = if body < 6 {
                    state.warriors[body as usize].record().guarding()? != 0
                } else {
                    c.roster.records[(body - 10) as usize].record().guarding() != 0
                };
                if guarding {
                    events.push(Event::Feedback(
                        crate::feedback::GameFeedback::CombatResult {
                            body,
                            code: -5,
                            amount: 0,
                            audible: false,
                        },
                    ));
                }
            }
            announce_guard = false;
            c.in_spell = 0;
            if reset_confusion {
                if let Some(value) = c.did_attack_write.take() {
                    state.globals.monster_turn_flags.did_attack = value;
                }
                state.globals.monster_turn_flags.getup();
                state.globals.possessed = 0;
                reset_confusion = false;
            }
            c.battle_order_index += 1;
            c.spell_queue = spells::tick_fields(&c.spell_queue, c.battle_order_index as i16)?;
            // With no monsters left, getup returns before clearing field use and
            // emptying the queue.
            if !c.kill_counts.has_remaining() {
                let actor = c.active_body().unwrap_or(0);
                events.extend(finish_action(state, content, actor, true)?);
                return Ok(events);
            }
            c.collide_check = spells::reset_used(&c.collide_check)?;
            if c.todo_queue.count == 0 {
                // Emptying an empty queue still clears doing_que.
                state.globals.death_processing.doing_queue = 0;
            }
            if c.todo_queue.count > 0 {
                let actor = c.active_body().unwrap_or(0);
                events.extend(finish_action(state, content, actor, true)?);
                return Ok(events);
            }
        }
        advance = true;
        if c.battle_order_index == 110 {
            if remaining_wraps == 0 {
                c.selection_pending = true;
                return Ok(events);
            }
            c.combat_round = c.combat_round.wrapping_add(1);
            events.extend(advance_round(state, content, remaining_wraps - 1)?);
            return Ok(events);
        }
        let Some(body) = c.active_body() else {
            continue;
        };
        if !c.body_alive(&state.warriors, body) {
            continue;
        }
        if body < 10
            && state.warriors[body as usize]
                .record()
                .turn_skip_duration()?
                != 0
            || body >= 10
                && c.roster.records[(body - 10) as usize]
                    .record()
                    .turn_skip_duration()
                    != 0
        {
            continue;
        }
        if body < 6 {
            // Both selections name the actor before its turn starts.
            state.char_select_new = body as usize;
            state.set_warrior_selector(body as usize);
        }
        let (turn, initialized) =
            super::turn_stories::initialize_session(state, content, super::TurnCaller::Advance)?;
        events.extend(turn);
        let super::turn_stories::TurnInitialization::Ready(skipped) = initialized else {
            return Ok(events);
        };
        if skipped {
            // A skipped actor returns to turn setup; dead or absent slots stay in the
            // scan and show no further guard.
            announce_guard = true;
            continue;
        }
        let c = state.combat.as_mut().ok_or("combat missing")?;
        if body < 10 && state.warriors[body as usize].record().computer_control()? == 0 {
            events.push(Event::Sound { resource_id: 138 });
        }
        events.push(Event::TurnStarted {
            body,
            round: c.combat_round,
        });
        return Ok(events);
    }
    Err(RuleError::Invalid("no active combatant".into()))
}
