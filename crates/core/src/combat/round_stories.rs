use crate::combat::Combat;
use crate::exploration::Operation;
mod spawning;
use crate::{RandomState, RuleError};
use centurions_content::{mission::OpeningMission, records::StoryRecord, span, word};
use spawning::spawn;
pub(crate) use spawning::spawn_at;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RoundEffect {
    Sound(i16),
    Text(i16),
    Spawn { body: i16, monster: i16 },
    Removed(i16),
    Dispatch { script: i16, cursor: u8 },
}

#[derive(Clone, Copy)]
pub struct RoundCaller {
    pub active_enemy: usize,
    pub last_defeated: Option<i16>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RoundOutcome {
    pub effects: Vec<RoundEffect>,
    pub hostile_additions: i16,
}

fn unsupported(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.round_story",
        dependency,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RoundScripts {
    pub counter: i8,
    pub slots: [i16; 5],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending: Option<RoundProgram>,
}

impl Default for RoundScripts {
    fn default() -> Self {
        Self {
            counter: 1,
            slots: [0; 5],
            pending: None,
        }
    }
}

impl RoundScripts {
    pub fn read(battles: &[u8], battle: i16) -> Result<Self, RuleError> {
        let record = span(battles, usize::from(battle.unsigned_abs()) * 1372, 1372)?;
        Ok(Self {
            counter: 1,
            pending: None,
            slots: [
                word(record, 1360)?,
                word(record, 1362)?,
                word(record, 1364)?,
                word(record, 1366)?,
                word(record, 1368)?,
            ],
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RoundProgram {
    pub slot: u8,
    pub script: Option<i16>,
    pub cursor: u8,
    pub text: Option<i16>,
    #[serde(default)]
    pub delegated: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RoundDispatch {
    pub suspended_script: Option<crate::ScriptState>,
    #[serde(default)]
    pub child_battle: bool,
    #[serde(default)]
    pub outer_battle: Option<RoundBattleReturn>,
    #[serde(default)]
    pub parent: Option<Box<RoundDispatch>>,
    #[serde(default)]
    pub death_depth: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RoundBattleReturn {
    pub after_message: i16,
    pub treasure: i16,
    pub group: Option<i16>,
    pub npc: Option<crate::story_npc_battle::NpcBattleStart>,
}

impl RoundDispatch {
    pub(crate) fn death_depth(state: &crate::Session) -> usize {
        std::iter::successors(state.combat_story.as_ref(), |story| story.parent.as_deref())
            .take(33)
            .count()
    }

    pub(crate) fn owns_script(&self, state: &crate::Session) -> bool {
        let current = Self::death_depth(state);
        current <= self.death_depth.unwrap_or(current)
    }

    pub(crate) fn validate(
        &self,
        state: &crate::Session,
        content: &centurions_content::Content,
    ) -> Result<(), RuleError> {
        let mut current = Some(self);
        let mut depth = 0;
        while let Some(frame) = current {
            depth += 1;
            if depth > 32
                || frame
                    .death_depth
                    .is_some_and(|saved| saved > 32 || saved > Self::death_depth(state))
                || (frame.child_battle && frame.death_depth.is_none())
                || (depth > 1 && !frame.child_battle)
                || (frame.outer_battle.is_some() && !frame.child_battle)
                || frame
                    .suspended_script
                    .as_ref()
                    .is_some_and(|script| !script.valid(content))
            {
                return Err(RuleError::Invalid("invalid round caller ownership".into()));
            }
            if let Some(outer) = &frame.outer_battle {
                let map = state.land(content).ok_or("round caller map unavailable")?;
                if outer.after_message != 0 {
                    map.dialogue(outer.after_message)?;
                }
                if outer.treasure != 0 {
                    span(
                        &map.treasure,
                        usize::from(outer.treasure.unsigned_abs()) * 88,
                        88,
                    )?;
                }
                if let Some(script) = outer.group {
                    map.script(script)?;
                }
                if let Some(npc) = &outer.npc {
                    if !(0..=8).contains(&npc.frame.action_index) {
                        return Err("invalid round NPC caller cursor".into());
                    }
                    let battles = &map
                        .combat
                        .as_ref()
                        .ok_or("round battle definitions missing")?
                        .battles;
                    centurions_content::combat::BattleRecord::read(battles, npc.battle)?;
                    if npc.maximum_battle != 0 {
                        centurions_content::combat::BattleRecord::read(
                            battles,
                            npc.maximum_battle,
                        )?;
                    }
                    if npc.escape_script >= 0 {
                        map.script(npc.escape_script)?;
                    }
                }
            }
            current = frame.parent.as_deref();
        }
        if state
            .exploration
            .as_ref()
            .is_none_or(|position| position.script.is_none())
            || (state.combat.is_none()
                && !self.child_battle
                && state.scripted_cleanup != Some(crate::story_cleanup::Stage::Complete))
            || (state.combat.is_some()
                && !self.child_battle
                && state
                    .combat
                    .as_ref()
                    .unwrap()
                    .round_scripts
                    .pending
                    .as_ref()
                    .is_none_or(|program| !program.delegated))
        {
            return Err(RuleError::Invalid(
                "invalid active round dispatch frame".into(),
            ));
        }
        Ok(())
    }
}

impl RoundProgram {
    pub fn validate(&self, mission: &OpeningMission) -> Result<(), RuleError> {
        if self.slot >= 5 || !(1..=8).contains(&self.cursor) {
            return Err(RuleError::Invalid("invalid round script cursor".into()));
        }
        let script = mission.script(self.script.ok_or("round script missing")?)?;
        let previous = usize::from(self.cursor) - 1;
        if self.delegated {
            if self.text.is_some() || word(&script, 8 + previous * 2)? != 3 {
                return Err(RuleError::Invalid("invalid delegated round script".into()));
            }
            return Ok(());
        }
        if word(&script, 8 + previous * 2)? != 1
            || self.text != Some(word(&script, 24 + previous * 2)?)
        {
            return Err(RuleError::Invalid(
                "round dialogue does not match its script".into(),
            ));
        }
        Ok(())
    }
}

fn extra(extras: &[u8], id: i16) -> Result<&[u8], RuleError> {
    Ok(span(extras, usize::from(id.unsigned_abs()) * 10, 10)?)
}

/// newland126 tests the signed round byte and clears its slot before choosing a replacement.
pub(crate) fn replacement(
    state: &mut RoundScripts,
    slot: u8,
    predicate: &[u8],
    random: &mut RandomState,
) -> Result<Option<i16>, RuleError> {
    let mode = word(predicate, 0)?;
    let eligible = if mode == 1 {
        random.bound(100) <= word(predicate, 2)?
    } else {
        mode != 0 || i16::from(state.counter) > word(predicate, 2)?
    };
    if !eligible {
        return Ok(None);
    }
    let repeat = word(predicate, 4)?;
    if mode != 1 && repeat != 1 {
        *state
            .slots
            .get_mut(usize::from(slot))
            .ok_or("round slot unavailable")? = 0;
    }
    Ok(Some(if repeat == 2 {
        random.range(word(predicate, 6)?, word(predicate, 8)?)
    } else {
        word(predicate, 6)?
    }))
}

pub(crate) fn neutralize(combat: &mut Combat, extra: &[u8]) -> Result<(), RuleError> {
    if word(extra, 2)? != 0 {
        return Err(unsupported("native reverse-mode victory-counter context"));
    }
    for record in &mut combat.enemies.records {
        if i16::from(record.record().definition_id()) == word(extra, 0)?
            && record.record().stamina()? > 0
        {
            record.record_mut().set_faction(0);
        }
    }
    Ok(())
}

/// newland125 uses killbody's retained body, rather than the queued story origin.
pub(crate) fn destroy_class(
    combat: &mut Combat,
    extra: &[u8],
    last_defeated: Option<i16>,
    random: &mut RandomState,
) -> Result<Vec<i16>, RuleError> {
    let body = last_defeated.ok_or_else(|| unsupported("retained killbody origin"))?;
    let center = if body < 10 {
        usize::try_from(body)
            .ok()
            .and_then(|index| combat.squad.positions.get(index))
            .copied()
            .ok_or_else(|| unsupported("retained killbody position"))?
    } else {
        combat.story_position((body - 10) as usize)?
    };
    let class = word(extra, 0)?;
    let count = match word(extra, 2)? {
        0 => 999,
        value if value < 0 => random.bound(value.wrapping_abs()),
        value => value,
    };
    if count < 0 {
        return Err(unsupported("negative removal count and native active NPC"));
    }
    let radius = match word(extra, 4)? {
        0 => 1000,
        value => value,
    };
    let mut removed = Vec::new();
    for _ in 0..count {
        let mut nearest = None;
        let mut distance = radius;
        for (index, record) in combat.enemies.records.iter().enumerate() {
            if record.stamina()? <= 0 || i16::from(record.record().definition_id()) != class {
                continue;
            }
            let point = combat.enemies.positions[index];
            let dx = f64::from(point.0.wrapping_sub(center.0));
            let dy = f64::from(point.1.wrapping_sub(center.1));
            let candidate = (dx * dx + dy * dy).sqrt().trunc() as i16;
            if candidate < distance {
                distance = candidate;
                nearest = Some(index);
            }
        }
        let Some(index) = nearest else { break };
        super::lifecycle::destroy_enemy(combat, index)?;
        removed.push(index as i16 + 10);
    }
    Ok(removed)
}

/// getup125d34..125db8 dispatches BD round slots before hourly reduction.
pub fn advance(
    combat: &mut Combat,
    state: &mut RoundScripts,
    mission: &OpeningMission,
    definitions: &[u8],
    extras: &[u8],
    caller: RoundCaller,
    random: &mut RandomState,
) -> Result<RoundOutcome, RuleError> {
    let mut step = None;
    let result = run(
        combat,
        state,
        mission,
        definitions,
        extras,
        caller,
        random,
        &mut step,
    );
    result.map_err(|error| match step {
        Some((script, opcode, cursor)) => {
            error.with_context(|| format!("round script {script} opcode {opcode} cursor {cursor}"))
        }
        None => error,
    })
}

/// `step` names the instruction being executed when an error leaves it.
#[allow(clippy::too_many_arguments)]
fn run(
    combat: &mut Combat,
    state: &mut RoundScripts,
    mission: &OpeningMission,
    definitions: &[u8],
    extras: &[u8],
    caller: RoundCaller,
    random: &mut RandomState,
    step: &mut Option<(i16, i16, usize)>,
) -> Result<RoundOutcome, RuleError> {
    if combat.enemies.records.len() > 100
        || caller.active_enemy >= 100
        || combat.enemies.positions.len() != combat.enemies.records.len()
        || combat
            .enemies
            .records
            .iter()
            .any(|record| record.bytes.len() != 288)
        || combat.order.len() != 110
        || combat.map.tiles.len() != 8100
    {
        return Err(RuleError::Invalid(
            "invalid battle round story input".into(),
        ));
    }
    if state
        .pending
        .as_ref()
        .is_some_and(|p| p.slot >= 5 || !(1..=8).contains(&p.cursor) || p.script.is_none())
    {
        return Err(RuleError::Invalid("invalid round continuation".into()));
    }
    if combat.death_queue.processing {
        return Err(unsupported(
            "battle round invoked during death-script processing",
        ));
    }
    let mut next = combat.clone();
    let mut next_state = state.clone();
    let mut next_random = random.clone();
    let mut outcome = RoundOutcome::default();
    if next_state
        .pending
        .as_ref()
        .is_some_and(|p| p.text.is_some() || p.delegated)
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    let mut program = next_state.pending.take().unwrap_or_else(|| {
        next_state.counter = next_state.counter.wrapping_add(1);
        RoundProgram {
            slot: 0,
            script: None,
            cursor: 0,
            text: None,
            delegated: false,
        }
    });
    let mut completed = false;
    for _ in 0..256 {
        if program.script.is_none() {
            while program.slot < 5 && next_state.slots[usize::from(program.slot)] == 0 {
                program.slot += 1;
            }
            if program.slot == 5 {
                completed = true;
                break;
            }
            program.script = Some(next_state.slots[usize::from(program.slot)]);
            program.cursor = 0;
        }
        if program.cursor == 8 {
            program.slot += 1;
            program.script = None;
            continue;
        }
        let record = mission.script(program.script.unwrap())?;
        let cursor = usize::from(program.cursor);
        let (opcode, value) = StoryRecord(&record).instruction(cursor)?;
        *step = Some((program.script.unwrap(), opcode, cursor));
        crate::trace::story(
            "round",
            i32::from(program.script.unwrap()),
            cursor,
            opcode,
            value,
        );
        program.cursor += 1;
        match Operation::from_native(opcode) {
            Operation::NoAction | Operation::Unarmed(_) => {}
            Operation::EndScript => program.cursor = 8,
            Operation::ContinueIfMonster => {
                if !crate::story_combat::monster_present(&next, value)? {
                    program.cursor = 8;
                }
            }
            Operation::Dialogue => {
                outcome.effects.push(RoundEffect::Text(value));
                program.text = Some(value);
                break;
            }
            Operation::Sound => outcome.effects.push(RoundEffect::Sound(value)),
            Operation::Question => {
                program.delegated = true;
                outcome.effects.push(RoundEffect::Dispatch {
                    script: program.script.unwrap(),
                    cursor: cursor as u8,
                });
                break;
            }
            Operation::NeutralizeActors => neutralize(&mut next, extra(extras, value)?)?,
            Operation::SummonActors => spawn(
                &mut next,
                mission,
                definitions,
                extra(extras, value)?,
                caller.active_enemy,
                &mut next_random,
                &mut outcome,
            )?,
            Operation::RemoveActorClass => {
                outcome.effects.extend(
                    destroy_class(
                        &mut next,
                        extra(extras, value)?,
                        caller.last_defeated,
                        &mut next_random,
                    )?
                    .into_iter()
                    .map(RoundEffect::Removed),
                );
            }
            Operation::ReplaceRoundScript => {
                let Some(target) = replacement(
                    &mut next_state,
                    program.slot,
                    extra(extras, value)?,
                    &mut next_random,
                )?
                else {
                    program.cursor = 8;
                    continue;
                };
                program.script = Some(target);
                program.cursor = 0;
            }
            Operation::ChanceBranch => {
                let predicate = extra(extras, value)?;
                if next_random.bound(100) <= word(predicate, 0)? {
                    match word(predicate, 2)? {
                        -2 | 2 => program.cursor = 8,
                        1 => match crate::story::condition_jump(predicate)? {
                            Some(crate::story::ConditionalJump::ReplaceScript(id)) => {
                                program.script = Some(id);
                                program.cursor = 0;
                            }
                            Some(crate::story::ConditionalJump::LastAction) => program.cursor = 7,
                            None => {}
                            _ => return Err(unsupported("round predicate menu continuation")),
                        },
                        _ => {}
                    }
                }
            }
            _ => {
                return Err(unsupported(
                    "round instruction needs a connected command adapter",
                ));
            }
        }
    }
    *step = None;
    if !completed {
        if program.text.is_none() && !program.delegated {
            return Err(unsupported("round instruction budget exhausted"));
        }
        next_state.pending = Some(program);
    }
    *combat = next;
    *state = next_state;
    *random = next_random;
    Ok(outcome)
}
