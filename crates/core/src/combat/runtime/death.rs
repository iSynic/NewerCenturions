//! Death in the original combat execution order.
use super::{alive, end_turn, finish_battle_wrapper, finish_victory, select_turn};
use crate::{Event, Phase, Presentation, RuleError, ScriptState, Session};
use centurions_content::Content;
use centurions_content::records::EnemyRecord;
use serde::{Deserialize, Serialize};

/// killbody12e8a4 uses showresults -14 for alternate bodies; both NPC
/// branches read the same death cue at showresults158ee0/158f14.
pub(super) fn enemy_death_feedback(
    monster: &centurions_content::combat::MonsterRecord,
    body: i16,
) -> Result<Vec<Event>, RuleError> {
    centurions_content::span(&monster.bytes, 0, 288)?;
    let mut events = vec![];
    if monster
        .record()
        .has_creature_category(EnemyRecord::MECHANICAL_CATEGORY)
    {
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
        resource_id: monster.record().death_sound()?,
    });
    Ok(events)
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeathStory {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub return_to: Option<Box<super::CombatReturn>>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outer_rewards: Option<super::OrdinaryBattleRewards>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outer_selected: Option<super::SelectedBattleTreasure>,
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
        let map = state.land(content).ok_or("combat story map unavailable")?;
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
            if let Some(caller) = &story.return_to {
                if !story.direct {
                    return Err("queued story owns an immediate combat return".into());
                }
                caller.validate()?;
            }
            if let Some(selected) = &story.outer_selected {
                selected.validate_suspended()?;
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

/// Each killbody frame saves DOOR actions locally, while its header stays global.
pub(crate) fn retain_death_header(state: &mut Session, record: &[u8]) {
    let mut current = state.combat_story.as_mut();
    while let Some(story) = current {
        if let Some(caller) = story.suspended_script.as_mut() {
            centurions_content::records::StoryRecordMut(&mut caller.record)
                .copy_return_header(record);
        }
        current = story.parent.as_deref_mut();
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
    process_action(state, content, actor, resume_turn, QueueScan::Begin)
}

enum QueueScan {
    Begin,
    Resume,
}

fn process_action(
    state: &mut Session,
    content: &Content,
    actor: i16,
    resume_turn: bool,
    queue: QueueScan,
) -> Result<Vec<Event>, RuleError> {
    state.target_quantity = None;
    retain_last_defeated(state)?;
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
        if matches!(queue, QueueScan::Begin) {
            c.death_queue.begin();
            state.native_work.death_processing.queue = 1;
        }
        c.death_queue
            .current()
            .map(|(index, entry)| (0, entry, false, Some(index - 1)))
    };
    if let Some(pending) = pending {
        return begin_callback(state, content, actor, resume_turn, pending, None);
    }
    c.death_queue.finish();
    state.native_work.death_processing.queue = 0;
    if !c.victory_work.has_remaining() {
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

pub(super) fn suspend_reaction(
    state: &mut Session,
    content: &Content,
    actor: i16,
    caller: super::CombatReturn,
) -> Result<Vec<Event>, RuleError> {
    caller.validate()?;
    state.retain_npc_work();
    retain_last_defeated(state)?;
    let combat = state
        .combat
        .as_mut()
        .ok_or("reaction callback combat missing")?;
    let (body, entry) = combat
        .direct_death
        .take()
        .ok_or("reaction callback entry missing")?;
    combat.script_enemy = (body - 10) as usize;
    combat.death_queue.processing = true;
    begin_callback(
        state,
        content,
        actor,
        false,
        ((body - 10) as usize, entry, true, None),
        Some(Box::new(caller)),
    )
}

fn begin_callback(
    state: &mut Session,
    content: &Content,
    actor: i16,
    resume_turn: bool,
    pending: (
        usize,
        crate::combat::death_scripts::DeathEntry,
        bool,
        Option<usize>,
    ),
    return_to: Option<Box<super::CombatReturn>>,
) -> Result<Vec<Event>, RuleError> {
    let (body, entry, direct, queue_cursor) = pending;
    state.native_work.death_processing.callback = 1;
    let parent = state.combat_story.take().map(Box::new);
    state.combat_story = Some(DeathStory {
        return_to,
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
        outer_rewards: state.ordinary_battle_rewards.clone(),
        outer_selected: state.selected_battle_treasure.clone(),
        outer_group: state.battle_group_script,
        outer_npc: state.battle_npc.clone(),
        shared_dispatch: false,
        suspended_script: None,
        parent,
    });
    continue_death(state, content)
}

pub(crate) fn retain_last_defeated(state: &mut Session) -> Result<(), RuleError> {
    state.retain_monster_work();
    let Some(body) = state
        .combat
        .as_ref()
        .and_then(|combat| combat.pending_last_defeated)
    else {
        return Ok(());
    };
    // Immediate newland runs before the native availability test.
    if state
        .combat
        .as_ref()
        .is_some_and(|combat| combat.direct_death.is_none())
    {
        crate::combat::reward_work::after_death(state)?;
    }
    let mut context = crate::story_health::retained_context(
        state.selected_warrior,
        &state.native_work.health_context,
    );
    context.last_body = body;
    state.native_work.health_context = context;
    state.combat.as_mut().unwrap().pending_last_defeated = None;
    Ok(())
}

pub(crate) fn finish_death_story(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    let story = state
        .combat_story
        .take()
        .ok_or_else(|| RuleError::Invalid("death continuation missing".into()))?;
    if story.direct {
        state.native_work.script_enemy = 0;
        if let Some(combat) = state.combat.as_mut() {
            combat.script_enemy = 0;
        }
    }
    super::death_return::refresh_actor(state, content)?;
    if story.shared_dispatch {
        state.exploration.as_mut().unwrap().script = story.suspended_script.clone();
    }
    state.native_work.death_processing.callback = 0;
    if let Some(combat) = state.combat.as_mut() {
        if story.direct {
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
        if story.direct {
            crate::combat::lifecycle::finish_if_party_unavailable(combat, &state.warriors);
            crate::combat::reward_work::after_death(state)?;
        }
        state.combat_story = story.parent.map(|parent| *parent);
        if let Some(caller) = story.return_to {
            return super::movement_return::resume(state, content, *caller);
        }
        // A queued callback returns to emptyque's local loop, without rewriting584.
        // A direct callback returns to the ordinary action's subsequent queue call.
        process_action(
            state,
            content,
            story.actor,
            story.resume_turn,
            if story.direct {
                QueueScan::Begin
            } else {
                QueueScan::Resume
            },
        )
    } else {
        if let Some(caller) = story.return_to {
            if story.outer_selected.is_some()
                || matches!(caller.as_ref(), super::CombatReturn::NpcAction(_))
            {
                state.selected_battle_treasure = story.outer_selected;
                state.battle_after_message = story.outer_after_message;
                state.battle_treasure = story.outer_treasure;
                state.ordinary_battle_rewards = story.outer_rewards;
                state.battle_group_script = story.outer_group;
                state.battle_npc = story.outer_npc;
            }
            state.combat_story = story.parent.map(|parent| *parent);
            return super::movement_return::resume(state, content, *caller);
        }
        if !story.direct
            && let Some(queue) = state.native_work.retained_death_queue.as_mut()
        {
            if let Some(cursor) = story.queue_cursor {
                queue.cursor = cursor;
            }
            queue.acknowledge()?;
            queue.finish();
            state.native_work.death_processing.queue = 0;
        }
        // Child combat replaces the global roster. The outer reward sees count zero.
        for item in &mut state.booty_items {
            item.0.fill(0);
        }
        state.battle_after_message = story.outer_after_message;
        state.battle_treasure = story.outer_treasure;
        state.ordinary_battle_rewards = story.outer_rewards;
        state.selected_battle_treasure = story.outer_selected;
        state.battle_group_script = story.outer_group;
        state.battle_npc = story.outer_npc;
        state.combat_story = story.parent.map(|parent| *parent);
        state.exploration.as_mut().unwrap().presentation = Some(Presentation::Loot);
        let mut events = if state.ordinary_battle_rewards.is_some()
            || state.selected_battle_treasure.is_some()
        {
            vec![]
        } else {
            vec![Event::Sound {
                resource_id: crate::feedback::BOOTY_SOUND,
            }]
        };
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
    retain_last_defeated(state)?;
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
    let map = state.land(content).ok_or("death story map missing")?;
    let story = state
        .combat_story
        .as_mut()
        .ok_or("death continuation missing")?;
    let position = state
        .exploration
        .as_mut()
        .ok_or("death story position missing")?;
    position.presentation = None;
    if !story.shared_dispatch {
        let mut record = map.script(story.script)?.to_vec();
        if let Some(caller) = position.script.as_ref() {
            record[..8].copy_from_slice(centurions_content::span(&caller.record, 0, 8)?);
        }
        story.shared_dispatch = true;
        story.suspended_script = position.script.take();
        position.script = Some(ScriptState::scheduled(record, story.cursor as u8));
    }
    crate::exploration::run_script(state, content)
}
