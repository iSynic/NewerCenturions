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
    /// Party positions 0x44709a for every slot; a battle's squad holds only its own.
    pub party_positions: [(i16, i16); 6],
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RoundOutcome {
    pub extra_record: Option<Vec<u8>>,
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
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub retained: bool,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rewards: Option<super::runtime::OrdinaryBattleRewards>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected: Option<super::runtime::SelectedBattleTreasure>,
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
                if let Some(selected) = &outer.selected {
                    selected.validate_suspended()?;
                }
                let map = state.land(content).ok_or("round caller map unavailable")?;
                if outer.after_message != 0 {
                    map.dialogue(outer.after_message)?;
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
                && !self.retained
                && !self.child_battle
                && !matches!(
                    state.scripted_cleanup,
                    Some(
                        crate::story_cleanup::Stage::Complete
                            | crate::story_cleanup::Stage::Rewards
                            | crate::story_cleanup::Stage::RetainedSurvivors { .. }
                    )
                ))
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
            if self.text.is_some() || !shared_operation(word(&script, 8 + previous * 2)?) {
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

/// getup125d74..125d8a calls newland(0, 0, -1, 0, 0) for each loaded round slot.
/// Operations with round-local implementations stay here; the shared interpreter
/// owns the rest of that script, including any modal continuation or replacement.
pub(crate) fn shared_operation(opcode: i16) -> bool {
    !matches!(
        Operation::from_native(opcode),
        Operation::NoAction
            | Operation::Unarmed(_)
            | Operation::EndScript
            | Operation::ContinueIfMonster
            | Operation::Dialogue
            | Operation::Sound
            | Operation::NeutralizeActors
            | Operation::SummonActors
            | Operation::RemoveActorClass
            | Operation::ReplaceRoundScript
    )
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
    replacement_slot(
        Some(state.counter),
        Some(&mut state.slots),
        Some(i16::from(slot)),
        predicate,
        random,
    )
}

/// newland 0x7e clears WORD 0x439cec + 2 * slot; Getup's slot loop leaves WORD 0x408576
/// at 5 when it finishes, and only clearing predicates write through it.
pub(crate) fn replacement_slot(
    counter: Option<i8>,
    slots: Option<&mut [i16; 5]>,
    slot: Option<i16>,
    predicate: &[u8],
    random: &mut RandomState,
) -> Result<Option<i16>, RuleError> {
    // BYTE44255c and WORD408576 are written only by battle setup and Getup's
    // round scan; before either they keep their zero DATA launch values, as
    // do WORDs439cec, so clearing an unloaded slot writes nothing new.
    let counter = counter.unwrap_or(0);
    let mode = word(predicate, 0)?;
    let eligible = if mode == 1 {
        random.bound(100) <= word(predicate, 2)?
    } else {
        mode != 0 || i16::from(counter) > word(predicate, 2)?
    };
    if !eligible {
        return Ok(None);
    }
    let repeat = word(predicate, 4)?;
    if mode != 1 && repeat != 1 {
        let slot = slot.unwrap_or(0);
        match (slot, slots) {
            (0..5, Some(slots)) => slots[slot as usize] = 0,
            // Slot 5 is battle record WORD 0x439cf6, the distance BYTE. Its only readers
            // (combatsetup 0x11d630, 0x11d66a and 0x11d6e6) follow the record fread at
            // 0x11d5a2, and updatemonstermenu reads it only in the cold tail, which ends
            // at the first battle; so the clear changes nothing any reader sees.
            (0..=5, _) => {}
            // Getup's loop (clr.w 0x125d3a, addq.w 0x125da8, cmpi.w #5 0x125dae) is the
            // only writer of WORD 0x408576, and session validation holds it in 0..=5.
            _ => return Err("round slot outside Getup's 0..=5 range".into()),
        }
    }
    Ok(Some(if repeat == 2 {
        random.range(word(predicate, 6)?, word(predicate, 8)?)
    } else {
        word(predicate, 6)?
    }))
}

pub(crate) fn neutralize(combat: &mut Combat, extra: &[u8]) -> Result<(), RuleError> {
    let reverse = word(extra, 2)? != 0;
    let class = word(extra, 0)?;
    for index in 0..100 {
        let record = combat.story_record_mut(index)?;
        if i16::from(record.record().definition_id()) == class
            && record.record().stamina()? > 0
            && (record.record().faction()? == 0) == reverse
        {
            // newland opcode122 writes zero in both arms, even in reverse mode.
            record.record_mut().set_faction(0);
            combat
                .victory_work
                .add_defeated(if reverse { -1 } else { 1 });
        }
    }
    Ok(())
}

/// Party position 0x44709a + 4 * slot of killbody's body WORD 0x408574 (0x12e506).
fn killbody_origin(
    combat: &Combat,
    body: i16,
    party_positions: &[(i16, i16); 6],
) -> Result<(i16, i16), RuleError> {
    let slot = usize::try_from(body).map_err(|_| "killbody origin before the party positions")?;
    if let Some(point) = combat.squad.positions.get(slot) {
        return Ok(*point);
    }
    match slot {
        // The battle squad holds only the warriors it entered with; combatsetup
        // leaves the later slots as the last writer left them.
        0..6 => Ok(party_positions[slot]),
        // Slots 6 and 7 are the WORDs 0x4470b2..0x4470b7 that presentation shares.
        6 | 7 => Err(unsupported(
            "killbody origin in a presentation-shared party slot",
        )),
        // Slots 8 and 9 are initiative order rows 1..4 at 0x4470ba.
        _ => combat.script_target_position(body, false, party_positions),
    }
}

/// newland125 uses killbody's retained body, rather than the queued story origin.
pub(crate) fn destroy_class(
    combat: &mut Combat,
    extra: &[u8],
    last_defeated: Option<i16>,
    party_positions: &[(i16, i16); 6],
    random: &mut RandomState,
) -> Result<Vec<i16>, RuleError> {
    let count = match word(extra, 2)? {
        0 => 999,
        value if value < 0 => random.bound(value.wrapping_abs()),
        value => value,
    };
    if count < 0 {
        let index = combat.selected_enemy;
        // destroymonster takes the selected NPC WORD 0x447196 into the 100 records;
        // every writer stores an NPC body - 10 and combat validation keeps it below 100.
        if index >= 100 {
            return Err("selected NPC past the NPC array".into());
        }
        // Records past the battle roster are the retained NPC slots.
        super::lifecycle::destroy_enemy(combat, index)?;
        return Ok(vec![index as i16 + 10]);
    }
    let class = word(extra, 0)?;
    let mut center = None;
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
            let center = match center {
                Some(point) => point,
                None => {
                    let body = last_defeated
                        .ok_or_else(|| RuleError::from("round caller without a killbody origin"))?;
                    let point = if body < 10 {
                        killbody_origin(combat, body, party_positions)?
                    } else {
                        combat.story_position((body - 10) as usize)?
                    };
                    center = Some(point);
                    point
                }
            };
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
    advance_inner(
        combat,
        state,
        mission,
        definitions,
        extras,
        caller,
        random,
        false,
    )
}

/// Session predicates execute in the shared interpreter before their RNG draw.
pub(crate) fn advance_for_session(
    combat: &mut Combat,
    state: &mut RoundScripts,
    mission: &OpeningMission,
    definitions: &[u8],
    extras: &[u8],
    caller: RoundCaller,
    random: &mut RandomState,
) -> Result<RoundOutcome, RuleError> {
    advance_inner(
        combat,
        state,
        mission,
        definitions,
        extras,
        caller,
        random,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
fn advance_inner(
    combat: &mut Combat,
    state: &mut RoundScripts,
    mission: &OpeningMission,
    definitions: &[u8],
    extras: &[u8],
    caller: RoundCaller,
    random: &mut RandomState,
    dispatch_predicates: bool,
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
        dispatch_predicates,
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
    dispatch_predicates: bool,
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
    // getup125a3a's slot loop runs each round macro without testing the queue byte
    // 438584 or the killbody callback byte 438585.
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
    // Only a script replacement moves the cursor back, so every cycle passes a
    // script entry. A repeated entry state means the original never leaves the
    // loop; an RNG-driven loop keeps changing state and runs to its end.
    let mut entries = std::collections::HashSet::new();
    loop {
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
        if program.cursor == 0
            && !entries.insert(serde_json::to_vec(&(
                &program,
                &next_state,
                &next,
                &next_random,
            ))?)
        {
            return Err(unsupported(
                "round script repeats without change; the original loop never ends",
            ));
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
            Operation::NeutralizeActors => {
                let record = extra(extras, value)?;
                outcome.extra_record = Some(record.to_vec());
                neutralize(&mut next, record)?;
            }
            Operation::SummonActors => {
                let record = extra(extras, value)?;
                outcome.extra_record = Some(record.to_vec());
                spawn(
                    &mut next,
                    mission,
                    definitions,
                    record,
                    caller.active_enemy,
                    &mut next_random,
                    &mut outcome,
                )?;
            }
            Operation::RemoveActorClass => {
                let record = extra(extras, value)?;
                outcome.extra_record = Some(record.to_vec());
                outcome.effects.extend(
                    destroy_class(
                        &mut next,
                        record,
                        caller.last_defeated,
                        &caller.party_positions,
                        &mut next_random,
                    )?
                    .into_iter()
                    .map(RoundEffect::Removed),
                );
            }
            Operation::ReplaceRoundScript => {
                let record = extra(extras, value)?;
                outcome.extra_record = Some(record.to_vec());
                let Some(target) =
                    replacement(&mut next_state, program.slot, record, &mut next_random)?
                else {
                    program.cursor = 8;
                    continue;
                };
                program.script = Some(target);
                program.cursor = 0;
            }
            Operation::ChanceBranch if !dispatch_predicates => {
                let predicate = extra(extras, value)?;
                outcome.extra_record = Some(predicate.to_vec());
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
                program.delegated = true;
                outcome.effects.push(RoundEffect::Dispatch {
                    script: program.script.unwrap(),
                    cursor: cursor as u8,
                });
                break;
            }
        }
    }
    *step = None;
    if !completed {
        next_state.pending = Some(program);
    }
    *combat = next;
    *state = next_state;
    *random = next_random;
    Ok(outcome)
}
