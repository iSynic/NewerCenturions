use crate::{Event, RuleError, Session, feedback::NativeFeedback};
use centurions_content::{Content, span, word};

pub const RECORD_BYTES: usize = 516;

/// Encounter2 calls castspell with an encounter active, bypassing location flags.
pub fn validate_spell(
    state: &Session,
    content: &Content,
    warrior: usize,
    spell: i16,
    power: i16,
) -> Result<Vec<u8>, RuleError> {
    if word(crate::exploration::typed_record(state, content)?, 106)? == 0 {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    let actor = state
        .warriors
        .get(warrior)
        .ok_or("encounter caster missing")?;
    if !(1..=7).contains(&power) || !actor.knows_spell(spell)? {
        return Err("encounter spell or power unavailable".into());
    }
    if [0x98, 0xac, 0x60, 0x68, 0x90]
        .into_iter()
        .any(|offset| word(&actor.bytes, offset).is_ok_and(|value| value != 0))
        || actor.stamina()? <= 0
        || actor.psi_points()? <= 0
        || state.combat.is_some()
            && (word(&actor.bytes, 0x14)? != 0
                || word(&actor.bytes, 0xac8)? >= word(&actor.bytes, 0xac6)?)
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
    if packet[1] == 7 && !matches!(word(&packet, 40)?, 0..=9 | 110) {
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
    actor.bytes[0x58..0x5a].copy_from_slice(&remaining.to_be_bytes());
    if state.combat.is_some() {
        let casts =
            i32::from_be_bytes(actor.bytes[0xa66..0xa6a].try_into().unwrap()).wrapping_add(1);
        actor.bytes[0xa66..0xa6a].copy_from_slice(&casts.to_be_bytes());
        let casts = word(&actor.bytes, 0xac8)?.wrapping_add(1);
        actor.bytes[0xac8..0xaca].copy_from_slice(&casts.to_be_bytes());
    }
    let identity = [spell / 1000 - 1, spell % 1000 / 100 - 1, spell % 100 - 1];
    state.selected_warrior = warrior;
    state.attack_identity = Some(identity);
    state.attack_count = power;
    state.attack_packet = Some(packet.clone());
    if let Some(combat) = state.combat.as_mut() {
        combat.attack_packet = Some(packet.clone());
        combat.attack_identity = Some(identity);
    }
    state.psi_menu_choices[warrior] = Some(crate::PsiMenuChoice {
        row: identity[1] as u8,
        column: identity[2] as u8,
        power,
    });
    state.party_mask.fill(0);
    let mut events = vec![Event::Sound {
        resource_id: word(&packet, 38)?,
    }];
    if packet[1] == 7 {
        if state.psi_memory.is_none() {
            state.psi_memory = Some(crate::initial_psi_memory(&state.warriors)?);
        }
        for (index, value) in identity.into_iter().chain([power]).enumerate() {
            let offset = warrior * 80 + 72 + index * 2;
            state.psi_memory.as_mut().unwrap()[offset..offset + 2]
                .copy_from_slice(&value.to_be_bytes());
        }
        let mut duration = state
            .random
            .range(i16::from(packet[13] as i8), i16::from(packet[14] as i8));
        if packet[15] != 0 {
            for _ in 0..power {
                duration = duration.wrapping_add(
                    state
                        .random
                        .range(i16::from(packet[15] as i8), i16::from(packet[16] as i8)),
                );
            }
        }
        if word(&packet, 40)? == 110 {
            crate::combat::party_buffs::extend_scenario_tail(state, duration)?;
        } else {
            crate::combat::party_buffs::extend(
                &mut state.party_buffs,
                word(&packet, 40)?,
                duration,
            )?;
        }
    }
    let record = crate::exploration::typed_record(state, content)?;
    let mut selector = 4;
    for index in 0..10 {
        let alternative = word(record, 106 + index * 2)?;
        if alternative == spell
            || (alternative != 0 && alternative < 7 && alternative == i16::from(packet[19] as i8))
        {
            selector = record[126 + index] as i8;
            break;
        }
    }
    events.extend(crate::exploration::finish_typed_selection(
        state, content, selector,
    )?);
    Ok(events)
}

pub fn begin_actions(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let record = crate::exploration::typed_record(state, content)?;
    if record[96] == 0 {
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
            let record = crate::exploration::typed_record(state, content)?;
            let required = &record[98..106];
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
            let selector = if matched { record[96] as i8 } else { 4 };
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
        let record = crate::exploration::typed_record(state, content)?;
        if record[96] == 0
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
    span(record, 0, RECORD_BYTES)?;
    if word(record, 136)? == 0 {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    for index in 0..5 {
        if word(record, 136 + index * 2)? == item {
            return Ok(record[146 + index] as i8);
        }
    }
    Ok(4)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PasswordSelection {
    Retry,
    Selected(i8),
}

pub fn attempt_count(record: &[u8]) -> Result<i16, RuleError> {
    Ok(i16::from(span(record, 0, 156)?[152] as i8))
}

pub fn dialogue(record: &[u8]) -> Result<i16, RuleError> {
    Ok(word(span(record, 0, 156)?, 154)?)
}

pub fn password_label(record: &[u8]) -> Result<&[u8], RuleError> {
    let label = span(record, 156 + 8 * 40, 40)?;
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
    if record[97] == 0 {
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
        record[97] as i8
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
    let record = span(record, 0, 156)?;
    match selected {
        -1 => Ok(TypedContinuation::Abort),
        0 => Ok(TypedContinuation::Cancel),
        1..=4 => {
            let selector = if selected == 4 && remaining == 1 && (record[152] as i8) > 1 {
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
    Ok((word(extra, 6)?, word(extra, 8)?))
}

pub fn question_plan(extra: &[u8], answer: bool) -> Result<QuestionPlan, RuleError> {
    let extra = span(extra, 0, 10)?;
    if answer != (word(extra, 0)? == 0) {
        return Ok(QuestionPlan::Continue);
    }
    Ok(match word(extra, 2)? {
        0 => QuestionPlan::MoveAway,
        1 => QuestionPlan::ReplaceScript(word(extra, 4)?),
        2 | 3 => {
            let branch = word(extra, 4)?;
            if !(0..4).contains(&branch) {
                return Err(RuleError::Invalid("question branch is unavailable".into()));
            }
            if word(extra, 2)? == 2 {
                QuestionPlan::MenuBranch(branch as u8)
            } else {
                QuestionPlan::TypedBranch(branch as u8)
            }
        }
        4 => QuestionPlan::EndScript,
        _ => QuestionPlan::Continue,
    })
}
