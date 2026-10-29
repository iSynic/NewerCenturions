//! combatchoice's occupied destination after its entry reaction has returned.
use crate::{Content, Event, RuleError, Session};
use serde::{Deserialize, Serialize};

mod exchange;
mod firing;
mod melee;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum Question {
    ExchangeOrAttack,
    AttackFriendly,
}

#[derive(Clone, Copy)]
pub(crate) enum Choice {
    First,
    Second,
    Cancel,
}

pub(crate) struct CastRequest {
    pub actor: usize,
    pub targets: Vec<crate::combat::spells::SpellTarget>,
    pub discard_empty: bool,
}

pub(crate) enum Control {
    Returned,
    Terminated,
    Retry,
    Question(Question),
    Cast(CastRequest),
}

pub(crate) struct Outcome {
    pub events: Vec<Event>,
    pub control: Control,
}

impl Outcome {
    fn returned(events: Vec<Event>) -> Self {
        Self {
            events,
            control: Control::Returned,
        }
    }
}

pub(crate) fn begin(
    state: &mut Session,
    content: &Content,
    actor: usize,
    target: i16,
    involuntary: bool,
) -> Result<Outcome, RuleError> {
    let own = state
        .warriors
        .get(actor)
        .ok_or_else(|| RuleError::from("selected warrior outside the squad"))?
        .record()
        .faction()?;
    if faction(state, target)? != own {
        return attack(state, content, actor, target);
    }
    if target == crate::combat::runtime::reaction_calls::retained_actor(state)? {
        return Ok(Outcome::returned(vec![]));
    }
    let exchange = (target < 10
        || state
            .native_work
            .npc_backing
            .slot(npc_index(target)?)?
            .record
            .record()
            .size()?
            == 0)
        && state.warriors[actor].record().movement_points()? > 4;
    if exchange {
        if involuntary {
            return exchange::apply(state, content, actor, target);
        }
        return Ok(question(target, Question::ExchangeOrAttack));
    }
    if involuntary {
        return Ok(Outcome {
            events: vec![],
            control: Control::Retry,
        });
    }
    Ok(question(target, Question::AttackFriendly))
}

pub(crate) fn answer(
    state: &mut Session,
    content: &Content,
    target: i16,
    question: Question,
    choice: Choice,
) -> Result<Outcome, RuleError> {
    let actor = state.selected_warrior;
    match (question, choice) {
        (Question::ExchangeOrAttack, Choice::Second) => {
            exchange::apply(state, content, actor, target)
        }
        (Question::ExchangeOrAttack, Choice::First)
        | (Question::AttackFriendly, Choice::Second) => {
            // combatchoice 0x11c8a8 excludes body10 from this conversion.
            if target > 10 {
                let record = &mut state
                    .native_work
                    .npc_backing
                    .slot_mut(npc_index(target)?)?
                    .record;
                if record.record().faction()? == 0 {
                    record.record_mut().set_faction(1);
                    state.native_work.victory_work.add_total(1);
                }
            }
            attack(state, content, actor, target)
        }
        _ => Ok(Outcome::returned(vec![])),
    }
}

fn question(target: i16, kind: Question) -> Outcome {
    Outcome {
        events: vec![
            Event::Sound { resource_id: 20005 },
            match kind {
                Question::ExchangeOrAttack => Event::ExchangeOffered { body: target },
                Question::AttackFriendly => Event::FriendAttackOffered { body: target },
            },
        ],
        control: Control::Question(kind),
    }
}

fn attack(
    state: &mut Session,
    content: &Content,
    actor: usize,
    target: i16,
) -> Result<Outcome, RuleError> {
    let warrior = state
        .warriors
        .get(actor)
        .ok_or_else(|| RuleError::from("selected warrior outside the squad"))?;
    if warrior.record().firing_mode_flag() != 0 {
        if warrior.record().equipped_weapon()? != 0 {
            return firing::apply(state, content, actor, target);
        }
        let melee = warrior.record().melee_weapon()?;
        if melee != 0 {
            state.load_item_work(content, melee)?;
        }
    }
    melee::apply(state, content, actor, target)
}

fn faction(state: &Session, target: i16) -> Result<i16, RuleError> {
    if (0..9).contains(&target) {
        // 0x448aa0 + target * 0xc42: the table, the Upgrade records or the editor backup.
        return Ok(state
            .table_record(target as usize)
            .ok_or("occupied-cell body outside the warrior table")?
            .record()
            .faction()?);
    }
    Ok(state
        .native_work
        .npc_backing
        .slot(npc_index(target)?)?
        .record
        .record()
        .faction()?)
}

/// NPC records at 0x439d3c + (target - 10) * 0x120.
fn npc_index(target: i16) -> Result<usize, RuleError> {
    match target {
        10..110 => Ok((target - 10) as usize),
        110.. => Err(unsupported("occupied-cell body past the NPC array")),
        9 => Err(unsupported("occupied-cell body 9 before the NPC array")),
        _ => Err(unsupported("occupied-cell body before the warrior table")),
    }
}

fn unsupported(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.retained_occupied",
        dependency,
    }
}
