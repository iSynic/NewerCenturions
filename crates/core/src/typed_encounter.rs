use crate::{Event, RuleError, Session, feedback::NativeFeedback};
use centurions_content::records::{
    AttackPacket, Condition, PsiSelectionMemoryMut, QuestionExtra, TypedEncounterRecord,
};
use centurions_content::{Content, span};

pub use centurions_content::records::TYPED_ENCOUNTER_BYTES as RECORD_BYTES;

/// Encounter2 calls castspell with an encounter active, bypassing location flags.
pub fn validate_spell(
    state: &Session,
    content: &Content,
    warrior: usize,
    spell: i16,
    power: i16,
) -> Result<Vec<u8>, RuleError> {
    if TypedEncounterRecord(crate::exploration::typed_record(state, content)?)
        .spell_alternative(0)?
        == 0
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    let actor = state
        .warriors
        .get(warrior)
        .ok_or("encounter caster missing")?;
    if !(1..=7).contains(&power) || !actor.knows_spell(spell)? {
        return Err("encounter spell or power unavailable".into());
    }
    let record = actor.record();
    if [
        Condition::Confused,
        Condition::UnlabelledScanBlock,
        Condition::Helpless,
        Condition::ScanBlocked,
        Condition::ComputerControl,
    ]
    .into_iter()
    .any(|condition| record.condition(condition).is_ok_and(|value| value != 0))
        || actor.stamina()? <= 0
        || actor.psi_points()? <= 0
        || state.combat.is_some()
            && (record.damage_marker()? != 0 || record.scans_used()? >= record.scan_capacity()?)
        || state
            .combat
            .as_ref()
            .is_some_and(|combat| combat.active_body() != Some(warrior as i16))
    {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    let map = crate::exploration::active_map(state, content)?;
    let packet = crate::combat::spells::load_packet(
        &map.combat
            .as_ref()
            .ok_or("encounter spell content missing")?
            .attacks,
        spell,
    )?;
    let view = AttackPacket(&packet);
    if view.targeting_mode() == 7 && !matches!(view.effect()?, 0..=9 | 110) {
        return Err(RuleError::Unsupported {
            action: "story.encounter_spell",
            dependency: "indexed global spell effect outside recovered party durations",
        });
    }
    if actor.psi_points()? < crate::combat::player_psi::cost(&packet, power)? {
        return Err(RuleError::Refused {
            feedback: vec![NativeFeedback::Warning { index: 18 }],
        });
    }
    Ok(packet)
}

pub fn cast_spell(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    spell: i16,
    power: i16,
) -> Result<Vec<Event>, RuleError> {
    let packet = validate_spell(state, content, warrior, spell, power)?;
    let actor = &mut state.warriors[warrior];
    let remaining = actor
        .psi_points()?
        .wrapping_sub(crate::combat::player_psi::cost(&packet, power)?);
    actor.record_mut().set_psi_points(remaining);
    if state.combat.is_some() {
        actor.record_mut().count_scan_cast();
        let casts = actor.record().scans_used()?.wrapping_add(1);
        actor.record_mut().set_scans_used(casts);
    }
    let identity = [spell / 1000 - 1, spell % 1000 / 100 - 1, spell % 100 - 1];
    state.selected_warrior = warrior;
    state.native_work.attack_identity = Some(identity);
    state.native_work.attack_count = power;
    state.native_work.attack_packet = Some(packet.clone());
    if let Some(combat) = state.combat.as_mut() {
        combat.attack_packet = Some(packet.clone());
        combat.attack_identity = Some(identity);
    }
    state.psi_menu_choices[warrior] = Some(crate::PsiMenuChoice {
        row: identity[1] as u8,
        column: identity[2] as u8,
        power,
    });
    state.native_work.party_mask.fill(0);
    let view = AttackPacket(&packet);
    let mut events = vec![Event::Sound {
        resource_id: view.impact_sound()?,
    }];
    if view.targeting_mode() == 7 {
        if state.native_work.psi_memory.is_none() {
            state.native_work.psi_memory = Some(crate::initial_psi_memory(&state.warriors)?);
        }
        PsiSelectionMemoryMut(state.native_work.psi_memory.as_mut().unwrap())
            .remember_cast(warrior, identity, power);
        let (minimum, maximum) = view.duration_bounds();
        let mut duration = state.random.range(minimum, maximum);
        let (minimum, maximum) = view.additional_duration_bounds();
        if minimum != 0 {
            for _ in 0..power {
                duration = duration.wrapping_add(state.random.range(minimum, maximum));
            }
        }
        if view.effect()? == 110 {
            crate::combat::party_buffs::extend_scenario_tail(state, duration)?;
        } else {
            crate::combat::party_buffs::extend(&mut state.party_buffs, view.effect()?, duration)?;
        }
    }
    let record = TypedEncounterRecord(crate::exploration::typed_record(state, content)?);
    let mut selector = 4;
    for index in 0..TypedEncounterRecord::SPELL_ALTERNATIVES {
        let alternative = record.spell_alternative(index)?;
        if alternative == spell
            || (alternative != 0
                && alternative < 7
                && alternative == i16::from(view.defense_category() as i8))
        {
            selector = record.spell_selector(index);
            break;
        }
    }
    events.extend(crate::exploration::finish_typed_selection(
        state, content, selector,
    )?);
    Ok(events)
}

pub fn begin_actions(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let record = TypedEncounterRecord(crate::exploration::typed_record(state, content)?);
    if record.action_selector() == 0 {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    state.encounter_actions = Some([false; 8]);
    Ok(vec![])
}

pub fn action_control(
    state: &mut Session,
    content: &Content,
    item: u16,
) -> Result<Vec<Event>, RuleError> {
    let mut selected = state
        .encounter_actions
        .ok_or("no physical action selection")?;
    match item {
        9..=16 => {
            let choice = &mut selected[usize::from(item - 9)];
            *choice = !*choice;
            let resource_id = if *choice { 145 } else { 144 };
            state.encounter_actions = Some(selected);
            Ok(vec![Event::Sound { resource_id }])
        }
        17 => {
            let record = TypedEncounterRecord(crate::exploration::typed_record(state, content)?);
            let required = record.required_actions();
            let count = required.iter().filter(|value| **value != 0).count();
            if selected.iter().filter(|value| **value).count() != count {
                return Err(RuleError::Refused {
                    feedback: vec![NativeFeedback::Warning { index: 58 }],
                });
            }
            let matched = count != 0
                && required
                    .iter()
                    .zip(selected)
                    .all(|(value, chosen)| (*value != 0) == chosen);
            let selector = if matched { record.action_selector() } else { 4 };
            state.encounter_actions = None;
            crate::exploration::finish_typed_selection(state, content, selector)
        }
        18 => {
            state.encounter_actions = None;
            Ok(vec![])
        }
        _ => Err("physical action control unavailable".into()),
    }
}

pub fn validate_actions(state: &Session, content: &Content) -> Result<(), RuleError> {
    if state.encounter_actions.is_some() {
        let record = TypedEncounterRecord(crate::exploration::typed_record(state, content)?);
        if record.action_selector() == 0
            || !state
                .exploration
                .as_ref()
                .and_then(|position| position.script.as_ref())
                .and_then(|script| script.menu.as_ref())
                .is_some_and(|menu| menu.typed)
            || state.outdoor_psi.is_some()
            || state.psi_selection.is_some()
            || state.medical_continuation.is_some()
            || state.promotion.is_some()
            || state.combat.as_ref().is_some_and(|combat| {
                combat.firearm_selection.is_some() || combat.selection_pending
            })
        {
            return Err("invalid physical action continuation".into());
        }
    }
    Ok(())
}

/// encounter2 122a1a matches the selected item against five authored alternatives.
pub fn select_item(record: &[u8], item: i16) -> Result<i8, RuleError> {
    let record = TypedEncounterRecord(span(record, 0, RECORD_BYTES)?);
    if record.item_alternative(0)? == 0 {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    for index in 0..TypedEncounterRecord::ITEM_ALTERNATIVES {
        if record.item_alternative(index)? == item {
            return Ok(record.item_selector(index));
        }
    }
    Ok(4)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PasswordSelection {
    Retry,
    Selected(i8),
}

/// newland44 on the loaded buffer: each eliminated branch keeps only its last
/// action, which becomes opcode 24 with its parameter unchanged.
pub fn eliminate_branches(record: &mut [u8], eliminated: u8) {
    for branch in (0..4).filter(|branch| eliminated >> branch & 1 != 0) {
        record[branch * 8..branch * 8 + 7].fill(0);
        record[32 + branch * 16..32 + branch * 16 + 14].fill(0);
        record[branch * 8 + 7] = 24;
    }
}

pub fn attempt_count(record: &[u8]) -> Result<i16, RuleError> {
    let record = TypedEncounterRecord(span(record, 0, TypedEncounterRecord::HEADER_BYTES)?);
    Ok(i16::from(record.attempts()))
}

pub fn dialogue(record: &[u8]) -> Result<i16, RuleError> {
    Ok(TypedEncounterRecord(span(record, 0, TypedEncounterRecord::HEADER_BYTES)?).dialogue()?)
}

pub fn password_label(record: &[u8]) -> Result<&[u8], RuleError> {
    let label = TypedEncounterRecord(record).password_label()?;
    if label[0] > 39 {
        return Err(RuleError::Invalid(
            "typed encounter label is too long".into(),
        ));
    }
    Ok(&label[1..1 + usize::from(label[0])])
}

pub fn select_password(
    record: &[u8],
    input: &[u8],
    lowercase: &[u8],
) -> Result<PasswordSelection, RuleError> {
    let record = span(record, 0, RECORD_BYTES)?;
    let selector = TypedEncounterRecord(record).password_selector();
    if selector == 0 {
        return Err(RuleError::Invalid("typed input is unavailable".into()));
    }
    if input.is_empty() || input[0] == 0 {
        return Ok(PasswordSelection::Retry);
    }
    let lowercase = span(lowercase, 0, 256)?;
    let mut normalized = input.to_vec();
    let expected = password_label(record)?;
    let mut matched = true;
    // encounter2 0x121b52 folds only the compared prefix, stopping on a mismatch.
    // CODE 3 0x300a58 uses the initialized DATA table at A5-0x3b0.
    for (index, value) in expected.iter().take(40).enumerate() {
        if *value == 0 || *value == b' ' {
            break;
        }
        let Some(actual) = normalized.get_mut(index) else {
            matched = false;
            break;
        };
        *actual = lowercase[usize::from(*actual)];
        if *actual != *value {
            matched = false;
            break;
        }
    }
    if [b"map".as_slice(), b"boat", b"mission"].contains(&normalized.as_slice()) {
        return Err(RuleError::Unsupported {
            action: "story.typed_input",
            dependency: "original developer command",
        });
    }
    Ok(PasswordSelection::Selected(if matched {
        selector
    } else {
        4
    }))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedContinuation {
    Abort,
    Cancel,
    Branch { selector: i8, body: [u8; 32] },
}

pub fn continuation(
    record: &[u8],
    selected: i8,
    remaining: i16,
) -> Result<TypedContinuation, RuleError> {
    let record = span(record, 0, TypedEncounterRecord::HEADER_BYTES)?;
    match selected {
        -1 => Ok(TypedContinuation::Abort),
        0 => Ok(TypedContinuation::Cancel),
        1..=4 => {
            let selector =
                if selected == 4 && remaining == 1 && TypedEncounterRecord(record).attempts() > 1 {
                    3
                } else {
                    selected
                };
            Ok(TypedContinuation::Branch {
                selector,
                body: crate::story::dispatch_menu_branch(record, selector)?,
            })
        }
        _ => Err(RuleError::Unsupported {
            action: "story.typed_input",
            dependency: "typed encounter selector outside the four authored branches",
        }),
    }
}

pub fn finish_attempt(remaining: i16) -> (i16, bool) {
    (remaining.wrapping_sub(1), remaining > 1)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QuestionPlan {
    Continue,
    MoveAway,
    ReplaceScript(i16),
    MenuBranch(u8),
    TypedBranch(u8),
    EndScript,
}

pub fn question_labels(extra: &[u8]) -> Result<(i16, i16), RuleError> {
    Ok(QuestionExtra(extra).labels()?)
}

pub fn question_plan(extra: &[u8], answer: bool) -> Result<QuestionPlan, RuleError> {
    let extra = QuestionExtra(span(extra, 0, QuestionExtra::BYTES)?);
    if answer != (extra.answer()? == 0) {
        return Ok(QuestionPlan::Continue);
    }
    Ok(match extra.outcome()? {
        0 => QuestionPlan::MoveAway,
        1 => QuestionPlan::ReplaceScript(extra.outcome_parameter()?),
        2 | 3 => {
            let branch = extra.outcome_parameter()?;
            if !(0..4).contains(&branch) {
                return Err(RuleError::Invalid("question branch is unavailable".into()));
            }
            if extra.outcome()? == 2 {
                QuestionPlan::MenuBranch(branch as u8)
            } else {
                QuestionPlan::TypedBranch(branch as u8)
            }
        }
        4 => QuestionPlan::EndScript,
        _ => QuestionPlan::Continue,
    })
}
