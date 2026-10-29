//! Turns in the original combat execution order.
use super::{advance_round, alive, finish_action};
use crate::combat::{Combat, initialize_turn};
use crate::{Event, RandomState, RuleError, Session};
use centurions_content::{Content, Warrior, records::Condition};

pub fn initialize_active_turn(
    c: &mut Combat,
    w: &mut [Warrior],
    mission: &centurions_content::mission::OpeningMission,
    random: &mut RandomState,
) -> Result<(Vec<Event>, bool), RuleError> {
    let body = c
        .active_body()
        .ok_or_else(|| RuleError::Invalid("active combatant missing".into()))?;
    let tables = mission
        .combat
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("combat tables missing".into()))?;
    if body < 6
        && (w[body as usize].record().active_flag() == 0
            || w[body as usize].record().turn_skip_duration()? != 0)
    {
        return Ok((vec![], true));
    }
    if body >= 10 {
        c.selected_enemy = (body - 10) as usize;
    }
    if body >= 10
        && c.enemies.records[(body - 10) as usize]
            .record()
            .final_turn_mode()?
            != 0
    {
        let turn = crate::combat::final_turn_stories::initialize(
            c,
            w,
            mission,
            &tables.monsters,
            &tables.encounters,
            random,
        )?;
        let events = turn_feedback(mission, turn.effects)?;
        return finish_enemy_setup(c, body, events, turn.skipped_turn);
    }
    c.turn_origin = (body < 6).then(|| c.squad.positions[body as usize]);
    if body < 6 {
        let (skipped, delta) = crate::combat::initialize_warrior_record(
            &mut w[body as usize],
            &tables.race_moves,
            random,
        )?;
        c.unavailable = c.unavailable.wrapping_add(delta);
        c.victory_work.add_total(i16::from(delta));
        if !skipped
            && c.automatic[body as usize]
            && w[body as usize].record().automatic_mode()? == 0
        {
            w[body as usize].record_mut().set_automatic_mode(1);
        }
        return Ok((vec![], skipped));
    }
    let prior_faction = c.enemies.records[(body - 10) as usize].record().faction()?;
    initialize_turn(body, w, &mut c.enemies.records, &tables.race_moves, random)?;
    let faction = c.enemies.records[(body - 10) as usize].record().faction()?;
    c.victory_work
        .add_total(i16::from(faction != 0) - i16::from(prior_faction != 0));
    let skipped = body >= 10
        && c.enemies.records[(body - 10) as usize]
            .record()
            .turn_skip_duration()
            != 0;
    finish_enemy_setup(c, body, vec![], skipped)
}

pub(super) fn turn_feedback(
    mission: &centurions_content::mission::OpeningMission,
    effects: Vec<crate::combat::final_turn_stories::TurnEffect>,
) -> Result<Vec<Event>, RuleError> {
    let mut events = vec![];
    for effect in effects {
        use crate::combat::final_turn_stories::TurnEffect;
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

/// getup126e50..127090 follows record initialization in both enemy callers.
pub(super) fn finish_enemy_setup(
    c: &mut Combat,
    body: i16,
    events: Vec<Event>,
    skipped: bool,
) -> Result<(Vec<Event>, bool), RuleError> {
    finish_selected_enemy_setup(c, body, (body - 10) as usize, events, skipped)
}

pub(super) fn finish_selected_enemy_setup(
    c: &mut Combat,
    body: i16,
    selected: usize,
    mut events: Vec<Event>,
    skipped: bool,
) -> Result<(Vec<Event>, bool), RuleError> {
    if skipped {
        return Ok((events, true));
    }
    let enemy = c.story_record_mut(selected)?;
    let Some((code, removed)) = enemy_morale(enemy)? else {
        return Ok((events, false));
    };
    events.push(Event::Feedback(
        crate::feedback::NativeFeedback::CombatResult {
            body,
            code,
            amount: 0,
            audible: false,
        },
    ));
    if removed {
        let hostile = enemy.record().faction()? != 0;
        enemy.record_mut().set_stamina(0);
        c.map = crate::combat::footprint::restore(
            &c.map,
            c.story_record(selected)?,
            c.story_position(selected)?,
        )?;
        // getup127008 counts nonzero faction without killbody's mechanical predicate.
        if hostile {
            c.victory_work.add_defeated(1);
        }
        events.push(Event::CombatantRemoved {
            body: selected as i16 + 10,
        });
    }
    Ok((events, removed))
}

/// Selection admits positive HP, so the original single-rounded HP / HP *100 is100.
pub(super) fn enemy_morale(
    enemy: &mut centurions_content::combat::MonsterRecord,
) -> Result<Option<(i16, bool)>, RuleError> {
    let chance = enemy.record().fleeing_chance()?;
    let removed = chance > 100;
    let code = if removed {
        if chance == 101 { -8 } else { -7 }
    } else if enemy.record().retreat_threshold()? > 100 {
        enemy.record_mut().set_condition(Condition::Retreating, -1);
        -6
    } else {
        return Ok(None);
    };
    Ok(Some((code, removed)))
}

pub fn end_turn(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    select_turn(state, content, true)
}

pub(super) fn select_turn(
    state: &mut Session,
    content: &Content,
    advance: bool,
) -> Result<Vec<Event>, RuleError> {
    select_turn_bounded(state, content, advance, 2)
}

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

pub(super) fn select_turn_bounded(
    state: &mut Session,
    content: &Content,
    advance: bool,
    remaining_wraps: u8,
) -> Result<Vec<Event>, RuleError> {
    select_turn_inner(state, content, advance, remaining_wraps, advance)
}

pub(super) fn resume_getup(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    select_turn_inner(state, content, true, 2, false)
}

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
        // getup 125a50 calls cleartarget.
        state.clear_targets();
        state.bandage_ready = 1;
        // getup 125a92 clears WORD44719a.
        state.native_work.fire_item = 0;
    }
    let c = state
        .combat
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("combat missing".into()))?;
    if advance {
        centurions_content::records::AttackPacketMut(&mut c.attack_packet).reset_turn_attack_kind();
    }
    if state.selected_battle_treasure.is_some()
        && i16::from(c.unavailable) >= state.warriors.len() as i16
    {
        return crate::mission::defeat(state, content);
    }
    if !super::has_active_warrior(&state.warriors) {
        c.victory_work.finish();
        state.native_work.early_booty = 1;
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
        if warrior.record().automatic_mode()? > 0 {
            warrior.record_mut().set_automatic_mode(0);
        }
    }
    let mut announce_guard = advance;
    for _ in 0..220 {
        let c = state.combat.as_mut().ok_or("combat missing")?;
        if advance {
            // getup125b42..125bd8 presents the current guard before advancing.
            if announce_guard && let Some(body) = c.active_body() {
                let guarding = if body < 6 {
                    state.warriors[body as usize].record().guarding()? != 0
                } else {
                    c.enemies.records[(body - 10) as usize].record().guarding() != 0
                };
                if guarding {
                    events.push(Event::Feedback(
                        crate::feedback::NativeFeedback::CombatResult {
                            body,
                            code: -5,
                            amount: 0,
                            audible: false,
                        },
                    ));
                }
            }
            announce_guard = false;
            // getup125bda follows guard feedback, before its inner slot scan.
            c.ready = 0;
            if reset_confusion {
                if let Some(value) = c.npc_inhibit_write.take() {
                    state.native_work.npc_work.inhibit = value;
                }
                state.native_work.npc_work.getup();
                state.native_work.walking_confusion = 0;
                reset_confusion = false;
            }
            c.cursor += 1;
            c.spell_fields = crate::combat::spells::tick_fields(&c.spell_fields, c.cursor as i16)?;
            // getup125cc8 returns before field-use clearing and emptyque.
            if !c.victory_work.has_remaining() {
                let actor = c.active_body().unwrap_or(0);
                events.extend(finish_action(state, content, actor, true)?);
                return Ok(events);
            }
            c.field_used = crate::combat::spells::reset_used(&c.field_used)?;
            if c.death_queue.count == 0 {
                // getup's emptyque call still establishes584=0 with no entries.
                state.native_work.death_processing.queue = 0;
            }
            if c.death_queue.count > 0 {
                let actor = c.active_body().unwrap_or(0);
                events.extend(finish_action(state, content, actor, true)?);
                return Ok(events);
            }
        }
        advance = true;
        if c.cursor == 110 {
            if remaining_wraps == 0 {
                c.selection_pending = true;
                return Ok(events);
            }
            c.round = c.round.wrapping_add(1);
            events.extend(advance_round(state, content, remaining_wraps - 1)?);
            return Ok(events);
        }
        let Some(body) = c.active_body() else {
            continue;
        };
        if !alive(c, &state.warriors, body) {
            continue;
        }
        if body < 10
            && state.warriors[body as usize]
                .record()
                .turn_skip_duration()?
                != 0
            || body >= 10
                && c.enemies.records[(body - 10) as usize]
                    .record()
                    .turn_skip_duration()
                    != 0
        {
            continue;
        }
        if body < 6 {
            // Getup12659c/1265a4 publishes both selectors before initialization.
            state.selected_warrior = body as usize;
            state.set_warrior_selector(body as usize);
        }
        let (turn, initialized) =
            super::turn_stories::initialize_session(state, content, super::TurnCaller::Advance)?;
        events.extend(turn);
        let super::turn_stories::TurnInitialization::Ready(skipped) = initialized else {
            return Ok(events);
        };
        if skipped {
            // Initialized skips return to prepare; dead/inactive slots above
            // stay in Getup's inner scan and do not present another guard.
            announce_guard = true;
            continue;
        }
        let c = state.combat.as_mut().ok_or("combat missing")?;
        if body < 10 && state.warriors[body as usize].record().automatic_mode()? == 0 {
            events.push(Event::Sound { resource_id: 138 });
        }
        events.push(Event::TurnStarted {
            body,
            round: c.round,
        });
        return Ok(events);
    }
    Err(RuleError::Invalid("no active combatant".into()))
}
