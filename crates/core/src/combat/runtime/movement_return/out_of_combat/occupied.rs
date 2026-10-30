//! Moving onto an occupied cell, once the entry reaction has returned.
use crate::combat::runtime::reaction_calls;
use crate::combat::spells::SpellTarget;
use crate::{Content, Event, RuleError, Session};
use serde::{Deserialize, Serialize};

mod exchange;
mod firing;
mod melee;

/// The offer made to the player at an occupied cell: to exchange places or attack, or only to
/// attack a friend.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum Question {
    ExchangeOrAttack,
    AttackFriendly,
}

/// The player's answer to the offer: the first or second button, or cancel.
#[derive(Clone, Copy)]
pub(crate) enum Choice {
    First,
    Second,
    Cancel,
}

/// A shot to fire: the warrior, its targets, and whether the emptied weapon is discarded.
pub(crate) struct CastRequest {
    pub actor: usize,
    pub targets: Vec<SpellTarget>,
    pub discard_empty: bool,
}

/// What moving onto an occupied cell leaves to the walk: it returned, the battle ended, the move is
/// retried, a question is offered, or a shot is cast.
pub(crate) enum Control {
    Returned,
    Terminated,
    Retry,
    Question(Question),
    Cast(CastRequest),
}

/// The events of an occupied-cell action and what the walk does next.
pub(crate) struct Outcome {
    pub events: Vec<Event>,
    pub control: Control,
}

impl Outcome {
    /// An outcome that gives control back to the walk.
    fn returned(events: Vec<Event>) -> Self {
        Self {
            events,
            control: Control::Returned,
        }
    }
}

/// Moving onto an occupied cell. An opponent is attacked. A friend is offered an exchange when it
/// is a warrior or a monster of no size and the mover has more than four movement, otherwise only
/// an attack on the friend is offered; an involuntary mover exchanges or retries without asking.
///
/// Original: combatchoice
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
    if target == reaction_calls::turn_actor(state)? {
        return Ok(Outcome::returned(vec![]));
    }
    let exchange = (target < 10
        || state
            .globals
            .monsters
            .slot(monster_index(target)?)?
            .record
            .record()
            .size()?
            == 0)
        && state.warriors[actor].record().movement()? > 4;
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

/// Applies the player's answer. The exchange swaps places; agreeing to attack a friendly monster
/// turns it hostile, and then the attack is made. Cancelling does nothing.
///
/// Original: combatchoice
pub(crate) fn answer(
    state: &mut Session,
    content: &Content,
    target: i16,
    question: Question,
    choice: Choice,
) -> Result<Outcome, RuleError> {
    let actor = state.char_select_new;
    match (question, choice) {
        (Question::ExchangeOrAttack, Choice::Second) => {
            exchange::apply(state, content, actor, target)
        }
        (Question::ExchangeOrAttack, Choice::First)
        | (Question::AttackFriendly, Choice::Second) => {
            // Body 10 is not converted, as in the game (see docs/source-notes/combat.md).
            if target > 10 {
                let record = &mut state
                    .globals
                    .monsters
                    .slot_mut(monster_index(target)?)?
                    .record;
                if record.record().faction()? == 0 {
                    record.record_mut().set_faction(1);
                    state.globals.kill_counts.add_num_enemy(1);
                }
            }
            attack(state, content, actor, target)
        }
        _ => Ok(Outcome::returned(vec![])),
    }
}

/// The offer's sound and the question put to the player.
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

/// The attack on the body in the cell: with firing mode on and a second weapon equipped the warrior
/// shoots, otherwise it strikes in melee with its melee weapon.
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
    if warrior.record().toggle_flag() != 0 {
        if warrior.record().equipped_weapon()? != 0 {
            return firing::apply(state, content, actor, target);
        }
        let melee = warrior.record().melee_weapon()?;
        if melee != 0 {
            state.load_item(content, melee)?;
        }
    }
    melee::apply(state, content, actor, target)
}

/// The faction of the body in the cell, whether warrior or monster.
fn faction(state: &Session, target: i16) -> Result<i16, RuleError> {
    if (0..9).contains(&target) {
        // A warrior record: the table, the Upgrade records or the editor backup.
        return Ok(state
            .table_record(target as usize)
            .ok_or("occupied-cell body outside the warrior table")?
            .record()
            .faction()?);
    }
    Ok(state
        .globals
        .monsters
        .slot(monster_index(target)?)?
        .record
        .record()
        .faction()?)
}

/// The monster record of a target body.
fn monster_index(target: i16) -> Result<usize, RuleError> {
    match target {
        10..110 => Ok((target - 10) as usize),
        110.. => Err(unsupported("occupied-cell body past the NPC array")),
        9 => Err(unsupported("occupied-cell body 9 before the NPC array")),
        _ => Err(unsupported("occupied-cell body before the warrior table")),
    }
}

/// The refusal for an occupied cell that depends on an unrecovered part of the game.
fn unsupported(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.retained_occupied",
        dependency,
    }
}
