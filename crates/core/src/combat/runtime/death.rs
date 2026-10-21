//! Death in the original combat execution order.
use super::{alive, end_turn, finish_battle_wrapper, finish_victory, select_turn};
use crate::{Event, Phase, Presentation, RuleError, ScriptState, Session};
use centurions_content::{Content, word};
use serde::{Deserialize, Serialize};

/// killbody12e8a4 uses showresults -14 for alternate bodies; both NPC
/// branches read the same death cue at showresults158ee0/158f14.
pub(super) fn enemy_death_feedback(
    monster: &centurions_content::combat::MonsterRecord,
    body: i16,
) -> Result<Vec<Event>, RuleError> {
    centurions_content::span(&monster.bytes, 0, 288)?;
    let mut events = vec![];
    if monster.bytes[0xab] != 0 {
        events.push(Event::Feedback(
            crate::feedback::NativeFeedback::CombatResult {
                body,
                code: -14,
                amount: 0,
                audible: false,
            },
        ));
    }
    events.push(Event::Sound {
        resource_id: word(&monster.bytes, 0x6a)?,
    });
    Ok(events)
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeathStory {
    #[serde(default)]
    pub direct: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue_cursor: Option<usize>,
    pub body: usize,
    pub script: i16,
    pub cursor: usize,
    pub actor: i16,
    #[serde(default)]
    pub resume_turn: bool,
    #[serde(default)]
    pub entry: crate::combat::death_scripts::DeathEntry,
    #[serde(default)]
    pub waiting_child: bool,
    #[serde(default)]
    pub child_returned: bool,
    #[serde(default)]
    pub outer_after_message: i16,
    #[serde(default)]
    pub outer_treasure: i16,
    #[serde(default)]
    pub outer_group: Option<i16>,
    #[serde(default)]
    pub outer_npc: Option<crate::story_npc_battle::NpcBattleStart>,
    #[serde(default)]
    pub shared_dispatch: bool,
    #[serde(default)]
    pub suspended_script: Option<ScriptState>,
    #[serde(default)]
    pub parent: Option<Box<DeathStory>>,
}

impl DeathStory {
    pub(crate) fn validate(&self, state: &Session, content: &Content) -> Result<(), RuleError> {
        let map = state
            .map(content, state.exploration.as_ref().map_or(0, |p| p.map))
            .ok_or("combat story map unavailable")?;
        let mut current = Some(self);
        let mut depth = 0;
        while let Some(story) = current {
            depth += 1;
            if depth > 32
                || story
                    .queue_cursor
                    .is_some_and(|index| index >= 100 || story.direct)
                || story.cursor > 8
                || !(0..110).contains(&story.actor)
                || story.body >= 100
                || map.script(story.script).is_err()
                || (depth > 1 && !story.waiting_child)
                || (state.phase != Phase::Combat && !story.waiting_child && !story.child_returned)
            {
                return Err(RuleError::Invalid(
                    "invalid combat story continuation".into(),
                ));
            }
            if let Some(script) = &story.suspended_script
                && (!story.shared_dispatch
                    || script.record.len() != 42
                    || script.cursor > 8
                    || script.map_slot >= 100
                    || script.menu.as_ref().is_some_and(|menu| {
                        menu.original.len() != 42
                            || menu.id < 0
                            || if menu.typed {
                                (menu.id as usize + 1) * crate::typed_encounter::RECORD_BYTES
                                    > map.typed_encounters.len()
                            } else {
                                (menu.id as usize + 1) * crate::story::MENU_RECORD_BYTES
                                    > map.menus.len()
                            }
                    }))
            {
                return Err(RuleError::Invalid("invalid suspended combat script".into()));
            }
            current = story.parent.as_deref();
        }
        Ok(())
    }
}

pub(crate) fn finish_player_action(
    state: &mut Session,
    content: &Content,
    actor: i16,
) -> Result<Vec<Event>, RuleError> {
    finish_action(state, content, actor, false)
}

pub(super) fn finish_action(
    state: &mut Session,
    content: &Content,
    actor: i16,
    resume_turn: bool,
) -> Result<Vec<Event>, RuleError> {
    state.target_quantity = None;
    retain_last_defeated(state);
    let c = state
        .combat
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("combat missing".into()))?;
    // A death during an executing story remains queued until that caller returns.
    if state
        .combat_story
        .as_ref()
        .is_some_and(|story| !story.waiting_child)
    {
        return Ok(vec![]);
    }
    let pending = if let Some((body, entry)) = c.direct_death.take() {
        c.script_enemy = (body - 10) as usize;
        c.death_queue.processing = true;
        Some(((body - 10) as usize, entry, true, None))
    } else {
        if !c.death_queue.processing {
            c.death_queue.begin();
        }
        c.death_queue
            .current()
            .map(|(index, entry)| (0, entry, false, Some(index - 1)))
    };
    if let Some((body, entry, direct, queue_cursor)) = pending {
        let parent = state.combat_story.take().map(Box::new);
        state.combat_story = Some(DeathStory {
            body,
            direct,
            queue_cursor,
            script: entry.script,
            cursor: 0,
            actor,
            resume_turn,
            entry,
            waiting_child: false,
            child_returned: false,
            outer_after_message: state.battle_after_message,
            outer_treasure: state.battle_treasure,
            outer_group: state.battle_group_script,
            outer_npc: state.battle_npc.clone(),
            shared_dispatch: false,
            suspended_script: None,
            parent,
        });
        return continue_death(state, content);
    }
    c.death_queue.finish();
    if c.enemies.records.iter().all(|m| {
        m.stamina().unwrap_or(0) <= 0
            || m.bytes[0xb0] == 255
            || word(&m.bytes, 0xe).unwrap_or(1) == 0
    }) {
        return finish_victory(state, content);
    }
    if resume_turn {
        return select_turn(state, content, false);
    }
    if actor >= 10
        || state.warriors[actor as usize].record().action_points()? < 2
        || c.active_body().is_none() && !alive(c, &state.warriors, actor)
    {
        return end_turn(state, content);
    }
    Ok(vec![])
}

pub(crate) fn retain_last_defeated(state: &mut Session) {
    let Some(body) = state
        .combat
        .as_mut()
        .and_then(|combat| combat.pending_last_defeated.take())
    else {
        return;
    };
    let mut context = crate::story_health::context(
        &state.warriors,
        state.selected_warrior,
        state.health_context.as_ref(),
    );
    context.last_body = body;
    state.health_context = Some(context);
}

pub(crate) fn finish_death_story(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    let story = state
        .combat_story
        .take()
        .ok_or_else(|| RuleError::Invalid("death continuation missing".into()))?;
    if story.shared_dispatch {
        state.exploration.as_mut().unwrap().script = story.suspended_script.clone();
    }
    if let Some(combat) = state.combat.as_mut() {
        if story.direct {
            combat.script_enemy = 0;
            combat.death_queue.processing = false;
        } else {
            if let Some(cursor) = story.queue_cursor {
                combat.death_queue.cursor = cursor;
            }
            combat.death_queue.acknowledge()?;
        }
        crate::combat::death_scripts::clear_body_references(
            combat,
            &mut state.warriors,
            if story.direct {
                story.body as i16 + 10
            } else {
                0
            },
        )?;
        state.combat_story = story.parent.map(|parent| *parent);
        finish_action(state, content, story.actor, story.resume_turn)
    } else {
        if !story.direct
            && let Some(queue) = state.retained_death_queue.as_mut()
        {
            if let Some(cursor) = story.queue_cursor {
                queue.cursor = cursor;
            }
            queue.acknowledge()?;
            queue.finish();
        }
        // Child combat replaces the global roster. The outer reward sees count zero.
        for item in &mut state.booty_items {
            item.0.fill(0);
        }
        state.battle_after_message = story.outer_after_message;
        state.battle_treasure = story.outer_treasure;
        state.battle_group_script = story.outer_group;
        state.battle_npc = story.outer_npc;
        state.combat_story = story.parent.map(|parent| *parent);
        state.exploration.as_mut().unwrap().presentation = Some(Presentation::Loot);
        let mut events = vec![Event::Sound { resource_id: 20 }];
        events.extend(finish_battle_wrapper(state, content)?);
        Ok(events)
    }
}

pub(crate) fn retain_death_parent(state: &mut Session, parent: Option<DeathStory>) {
    if let Some(parent) = parent {
        if let Some(current) = state.combat_story.as_mut() {
            let mut current = current;
            while current.parent.is_some() {
                current = current.parent.as_deref_mut().unwrap();
            }
            current.parent = Some(Box::new(parent));
        } else {
            state.combat_story = Some(parent);
        }
    }
}

pub fn continue_death(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    retain_last_defeated(state);
    let mut events = vec![];
    if state
        .combat_story
        .as_ref()
        .is_some_and(|story| story.waiting_child)
    {
        if state.combat.is_some() {
            return Err(RuleError::Invalid(
                "finish the child battle before continuing".into(),
            ));
        }
        if state.booty_items.iter().any(|item| item.id() != 0) {
            return Err(RuleError::Invalid(
                "collect or store the equipment before continuing".into(),
            ));
        }
        let story = state.combat_story.as_mut().unwrap();
        story.waiting_child = false;
        story.child_returned = true;
    }
    state.exploration.as_mut().unwrap().presentation = None;
    if state
        .combat_story
        .as_ref()
        .is_some_and(|story| story.shared_dispatch)
    {
        return crate::exploration::run_script(state, content);
    }
    let map = content
        .map(
            state
                .exploration
                .as_ref()
                .map_or(0, |position| position.map),
        )
        .unwrap();
    if state
        .combat_story
        .as_ref()
        .is_some_and(|story| story.cursor == 0)
    {
        let stamp = map.script(state.combat_story.as_ref().unwrap().script)?[41];
        crate::exploration::retain_script_stamp(state, stamp);
    }
    for _ in 0..8 {
        let story = state.combat_story.as_mut().unwrap();
        if story.cursor >= 8 {
            events.extend(finish_death_story(state, content)?);
            return Ok(events);
        }
        let record = map.script(story.script)?;
        let opcode = word(&record, 8 + story.cursor * 2)?;
        let parameter = word(&record, 24 + story.cursor * 2)?;
        story.cursor += 1;
        match opcode {
            0 => {}
            119 | 122 | 123 | 124 | 125 => {
                let (effects, stopped) =
                    crate::story_combat::apply(state, content, opcode, parameter)?;
                events.extend(effects);
                if stopped {
                    events.extend(finish_death_story(state, content)?);
                    return Ok(events);
                }
            }
            1 => {
                state.exploration.as_mut().unwrap().presentation = Some(Presentation::Dialogue {
                    id: parameter,
                    text: map.dialogue(parameter)?,
                });
                events.push(Event::StoryPresented);
                return Ok(events);
            }
            9 => events.push(Event::Sound {
                resource_id: parameter,
            }),
            47 => state.flags = crate::story::set_mission_flag(&state.flags, parameter)?,
            2 => {
                events.extend(crate::exploration::begin_story_battle(
                    state,
                    content,
                    crate::exploration::StoryBattle::Ordinary,
                    parameter,
                )?);
                return Ok(events);
            }
            _ => {
                let mut record = map.script(story.script)?.to_vec();
                if let Some(parent) = state.exploration.as_ref().unwrap().script.as_ref() {
                    record[..8].copy_from_slice(centurions_content::span(&parent.record, 0, 8)?);
                }
                let cursor = story.cursor - 1;
                story.shared_dispatch = true;
                story.suspended_script = state.exploration.as_mut().unwrap().script.take();
                state.exploration.as_mut().unwrap().script = Some(ScriptState {
                    record,
                    cursor: cursor as u8,
                    map_slot: 0,
                    item_return: false,
                    scheduled: true,
                    menu: None,
                    persist: false,
                    origin: None,
                });
                events.extend(crate::exploration::run_script(state, content)?);
                return Ok(events);
            }
        }
    }
    events.extend(finish_death_story(state, content)?);
    Ok(events)
}
