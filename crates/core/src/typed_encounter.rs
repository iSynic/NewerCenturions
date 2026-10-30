use crate::combat::party_buffs;
use crate::combat::player_psi;
use crate::combat::spells::spell_identity;
use crate::{Event, RuleError, Session, feedback::GameFeedback};
use centurions_content::records::{
    Condition, PsiSelectionMemoryMut, QuestionExtra, SpellInfoRecord, TypedEncounterRecord,
};
use centurions_content::{Content, span};

pub use centurions_content::records::TYPED_ENCOUNTER_BYTES as RECORD_BYTES;

/// A typed encounter casts with the encounter active, so location limits do not
/// apply.
///
/// Original: encounter2
pub fn validate_spell(
    state: &Session,
    content: &Content,
    warrior: usize,
    spell: i16,
    power: i16,
) -> Result<Vec<u8>, RuleError> {
    if TypedEncounterRecord(&crate::exploration::typed_record(state, content)?)
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
            && (record.been_attacked()? != 0
                || record.spells_so_far()? >= record.max_spells_attacks()?)
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
    let view = SpellInfoRecord(&packet);
    let indexed = 0x442562_i64 + 2 * i64::from(view.effect()?);
    if view.is_party_duration()
        && !matches!(view.effect()?, 0..=9 | 50 | 110)
        && state.information_word(indexed).is_none()
    {
        return Err(RuleError::Unsupported {
            action: "story.encounter_spell",
            dependency: "indexed spell effect WORD 0x442562 + 2 * effect without an owner",
        });
    }
    if actor.psi_points()? < player_psi::cost(&packet, power)? {
        return Err(RuleError::Refused {
            feedback: vec![GameFeedback::Warning { index: 18 }],
        });
    }
    Ok(packet)
}

/// Casts a known psi spell for a warrior during a typed encounter. It spends the psi points, plays
/// the impact sound and applies the spell's party-duration effect if it has one. The encounter then
/// takes the branch whose spell alternative matches the spell or its defense category, or branch 4.
pub fn cast_spell(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    spell: i16,
    power: i16,
) -> Result<Vec<Event>, RuleError> {
    let packet = validate_spell(state, content, warrior, spell, power)?;
    player_psi::select_combat_caster(state);
    let actor = &mut state.warriors[warrior];
    let remaining = actor
        .psi_points()?
        .wrapping_sub(player_psi::cost(&packet, power)?);
    actor.record_mut().set_psi_points(remaining);
    if state.combat.is_some() {
        actor.record_mut().count_scan_cast();
        let casts = actor.record().spells_so_far()?.wrapping_add(1);
        actor.record_mut().set_spells_so_far(casts);
    }
    let identity = spell_identity(spell);
    state.char_select_new = warrior;
    state.globals.cast_identity = identity;
    state.globals.attack_count = power;
    state.globals.spell_info = packet.clone();
    if let Some(combat) = state.combat.as_mut() {
        combat.spell_info = packet.clone();
        combat.cast_identity = identity;
    }
    // castspell remembers the cast power with the menu choice.
    state.globals.old_power_level[warrior] = None;
    state.psi_menu_choices[warrior] = Some(crate::PsiMenuChoice {
        row: identity[1] as u8,
        column: identity[2] as u8,
        power,
    });
    // Targets clear before the impact sound.
    state.clear_targets();
    let view = SpellInfoRecord(&packet);
    let mut events = vec![Event::Sound {
        resource_id: view.impact_sound()?,
    }];
    if view.is_party_duration() {
        PsiSelectionMemoryMut(&mut state.globals.define_spells)
            .remember_cast(warrior, identity, power);
        let (minimum, maximum) = view.duration_range();
        let mut duration = state.random.range(minimum, maximum);
        let (minimum, maximum) = view.power_duration_range();
        if minimum != 0 {
            for _ in 0..power {
                duration = duration.wrapping_add(state.random.range(minimum, maximum));
            }
        }
        match view.effect()? {
            110 => party_buffs::extend_scenario_tail(state, duration)?,
            50 => {
                events.push(Event::Sound { resource_id: 601 });
                party_buffs::extend_from_count(&mut state.party_condition, power)?;
            }
            effect @ 0..=9 => party_buffs::extend(&mut state.party_condition, effect, duration)?,
            effect => {
                if !party_buffs::extend_indexed(state, effect, duration)? {
                    return Err(RuleError::Unsupported {
                        action: "story.encounter_spell",
                        dependency: "indexed spell effect WORD 0x442562 + 2 * effect without an owner",
                    });
                }
            }
        }
    }
    // `in_spell` clears after the cast, before the selected branch.
    state.set_in_spell(0);
    let bytes = crate::exploration::typed_record(state, content)?;
    let record = TypedEncounterRecord(&bytes);
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

/// Opens the choice of physical actions for a typed encounter that offers them.
pub fn begin_actions(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let bytes = crate::exploration::typed_record(state, content)?;
    let record = TypedEncounterRecord(&bytes);
    if record.action_selector() == 0 {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    state.encounter_actions = Some([false; 8]);
    Ok(vec![])
}

/// Handles a click while the physical actions are chosen: toggles one of eight actions, confirms
/// the choice against the encounter's required actions, or cancels.
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
            let bytes = crate::exploration::typed_record(state, content)?;
            let record = TypedEncounterRecord(&bytes);
            let required = record.required_actions();
            let count = required.iter().filter(|value| **value != 0).count();
            if selected.iter().filter(|value| **value).count() != count {
                return Err(RuleError::Refused {
                    feedback: vec![GameFeedback::Warning { index: 58 }],
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

/// Checks that a pending choice of actions belongs to a typed encounter that offers them.
pub fn validate_actions(state: &Session, content: &Content) -> Result<(), RuleError> {
    if state.encounter_actions.is_some() {
        let bytes = crate::exploration::typed_record(state, content)?;
        let record = TypedEncounterRecord(&bytes);
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

/// Matches the selected item against the encounter's five items.
///
/// Original: encounter2
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

/// What a typed password gave: nothing typed, so try again, or the branch it selects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PasswordSelection {
    Retry,
    Selected(i8),
}

/// Eliminate Complex Branch: each eliminated branch keeps only its last action,
/// which becomes End Script with its parameter unchanged.
pub fn eliminate_branches(record: &mut [u8], eliminated: u8) {
    for branch in (0..4).filter(|branch| eliminated >> branch & 1 != 0) {
        record[branch * 8..branch * 8 + 7].fill(0);
        record[32 + branch * 16..32 + branch * 16 + 14].fill(0);
        record[branch * 8 + 7] = 24;
    }
}

/// How many attempts the typed encounter allows.
pub fn attempt_count(record: &[u8]) -> Result<i16, RuleError> {
    let record = TypedEncounterRecord(span(record, 0, TypedEncounterRecord::HEADER_BYTES)?);
    Ok(i16::from(record.attempts()))
}

/// The dialogue the typed encounter shows.
pub fn dialogue(record: &[u8]) -> Result<i16, RuleError> {
    Ok(TypedEncounterRecord(span(record, 0, TypedEncounterRecord::HEADER_BYTES)?).dialogue()?)
}

/// The word the player is asked to type, up to 39 characters.
pub fn password_label(record: &[u8]) -> Result<&[u8], RuleError> {
    let label = TypedEncounterRecord(record).password_label()?;
    if label[0] > 39 {
        return Err(RuleError::Invalid(
            "typed encounter label is too long".into(),
        ));
    }
    Ok(&label[1..1 + usize::from(label[0])])
}

/// Compares what the player typed, folded to lowercase, with the password up to its first space;
/// extra typed characters after the password do not matter. A match selects the encounter's
/// branch and a miss selects branch 4. Nothing typed asks again.
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
    // Only the compared prefix is case-folded, stopping on a mismatch.
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

/// How a typed encounter ends: abandoned, cancelled, or a branch with its script body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedContinuation {
    Abort,
    Cancel,
    Branch { selector: i8, body: [u8; 32] },
}

/// Decides how the encounter ends from the branch selected: -1 abandons it, 0 cancels it and 1 to 4
/// run that branch. A miss (4) on the last of several attempts runs branch 3.
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

/// Uses up an attempt, returning the attempts left and whether the player may try again.
pub fn finish_attempt(remaining: i16) -> (i16, bool) {
    (remaining.wrapping_sub(1), remaining > 1)
}

/// Where a yes or no question sends the script: on, back a step, to another script, to a menu or
/// typed branch, or to the end.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QuestionPlan {
    Continue,
    MoveAway,
    ReplaceScript(i16),
    MenuBranch(u8),
    TypedBranch(u8),
    EndScript,
}

/// The labels of a question's two answers.
pub fn question_labels(extra: &[u8]) -> Result<(i16, i16), RuleError> {
    Ok(QuestionExtra(extra).labels()?)
}

/// Decides what a question does for the answer given: nothing for the answer the question does not
/// act on, otherwise the outcome the question names.
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
