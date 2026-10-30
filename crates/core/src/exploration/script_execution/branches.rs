//! "Branch on" script operations that jump to another action or encounter.
use super::super::{active_table, finish_script, present_menu, present_typed, unsupported};
use super::{ENCOUNTER_SOUND, ExtraIndex, Operation, ScriptStep, load_extra};
use crate::{Event, RuleError, Session};
use centurions_content::mission::OpeningMission;
use centurions_content::records::{
    Condition, ProfessionEquipment, ProfileTable, RaceEquipment, WarriorRecord,
};
use centurions_content::{Content, word};

/// Where a passed test sends the script.
enum Jump {
    /// Load another script and restart at its first action.
    Script(i16),
    /// Enter a simple or complex encounter without saving the current record, so
    /// the encounter returns to the record saved earlier.
    Simple(i16),
    Complex(i16),
    /// The complex encounter case itself, which saves the current record.
    ComplexSaved(i16),
    /// The cursor moves past the last action.
    End,
}

/// Runs the script's branch operations on the party's and members' conditions, game time, psi
/// points, item charges, the member picked, quest ranges and miscellaneous tests, jumping to
/// another action or encounter when the test holds.
pub(super) fn execute(
    operation: Operation,
    parameter: i16,
    state: &mut Session,
    content: &Content,
    map: &OpeningMission,
    effects: &mut Vec<Event>,
) -> Result<ScriptStep, RuleError> {
    let extra = load_extra(
        &mut state.globals,
        map,
        ExtraIndex::Signed(parameter),
        Some("story extras unavailable"),
    )?;
    let e = |offset: usize| word(extra, offset);
    let encounter = |mode: i16, id: i16| match mode {
        0 => Some(Jump::Script(id)),
        1 => Some(Jump::Simple(id)),
        2 => Some(Jump::Complex(id)),
        _ => None,
    };
    let jump = match operation {
        // Tests one party protection for set (1) or clear (2).
        Operation::PartyConditionBranch => {
            let buff = usize::try_from(e(6)?)
                .ok()
                .and_then(|index| word(&state.party_condition, index * 2).ok())
                .ok_or_else(|| {
                    unsupported(
                        "story.opcode",
                        "party condition outside the protection buffer",
                    )
                })?;
            if (e(0)? == 1 && buff != 0) || (e(0)? == 2 && buff == 0) {
                Some(match e(2)? {
                    1 => Jump::Script(e(4)?),
                    2 => Jump::Simple(e(4)?),
                    3 => Jump::ComplexSaved(e(4)?),
                    _ => Jump::End,
                })
            } else {
                None
            }
        }
        // Compares the hour twice and has no skip value.
        Operation::GameTimeBranch => {
            let position = state.exploration.as_ref().unwrap();
            let (day, hour) = (position.day, position.hour);
            let after = e(0)? < day || e(2)? < hour || e(4)? < hour;
            Some(Jump::Script(if after { e(8)? } else { e(6)? }))
        }
        Operation::ItemChargesBranch => {
            let charges = crate::party::item_charges(&state.warriors, e(0)?)?;
            encounter(e(2)?, if charges < e(4)? { e(8)? } else { e(6)? })
        }
        Operation::QuestRangeBranch => {
            let (low, high) = (e(0)?, e(2)?);
            let mut all = true;
            for flag in low..=high {
                let flag = usize::try_from(flag).ok().and_then(|i| state.flags.get(i));
                all &= *flag
                    .ok_or_else(|| unsupported("story.opcode", "quest outside the flags"))?
                    != 0;
            }
            if all { encounter(e(6)?, e(8)?) } else { None }
        }
        // Tests picked members (1) or living, uncontrolled members (2).
        Operation::PsiPointsBranch => {
            let required = e(2)?;
            let found = state.warriors.iter().enumerate().any(|(member, warrior)| {
                let record = warrior.record();
                let eligible = match e(0) {
                    Ok(1) => state.globals.party_track[member] != 0,
                    Ok(2) => {
                        record.stamina().is_ok_and(|s| s > 0)
                            && record
                                .condition(Condition::ComputerControl)
                                .is_ok_and(|c| c == 0)
                    }
                    _ => false,
                };
                eligible && record.psi_points().is_ok_and(|psi| required <= psi)
            });
            if found {
                encounter(e(6)?, e(8)?)
            } else if e(4)? == 1 {
                // A failed test ends the script like End Script and keeps the codes.
                effects.extend(finish_script(state, content, false)?);
                return Ok(ScriptStep::Pause);
            } else {
                None
            }
        }
        // Tests every picked member (-1), the whole party (0) or member e2,
        // counted from zero.
        Operation::CharacterConditionBranch => {
            let slot = usize::try_from(e(0)?)
                .ok()
                .filter(|&slot| slot < 40)
                .ok_or_else(|| unsupported("story.opcode", "condition outside the record"))?;
            let has = |warrior: &centurions_content::Warrior| {
                WarriorRecord::from_bytes(&warrior.bytes)
                    .condition_slot(slot)
                    .is_ok_and(|d| d != 0)
            };
            let all = match e(2)? {
                -1 => (0..state.warriors.len())
                    .filter(|&m| state.globals.party_track[m] != 0)
                    .all(|m| has(&state.warriors[m])),
                0 => state.warriors.iter().all(has),
                member => has(usize::try_from(member)
                    .ok()
                    .and_then(|m| state.warriors.get(m))
                    .ok_or_else(|| {
                        unsupported("story.opcode", "condition member outside the party")
                    })?),
            };
            Some(Jump::Script(if all { e(6)? } else { e(8)? }))
        }
        // Modes 1-6 all test the first member's pick; other modes need exactly
        // |mode| picked members.
        Operation::PickedBranch => {
            let mask = &state.globals.party_track;
            let picked = (0..state.warriors.len()).filter(|&m| mask[m] != 0).count();
            let passed = match e(0)? {
                0 => picked > 0,
                1..=6 => mask[0] != 0,
                mode => i32::from(mode.wrapping_abs()) == picked as i32,
            };
            Some(if passed {
                Jump::Script(e(6)?)
            } else if e(2)? == 1 {
                Jump::Script(e(8)?)
            } else {
                Jump::End
            })
        }
        // A negative code tests only picked members, for its magnitude.
        // Caste and race classes come from each member's profession and race record.
        Operation::MiscellaneousBranch => {
            let code = e(2)?;
            let wanted = if code < 0 { code.wrapping_abs() } else { code };
            let mask = &state.globals.party_track;
            let passed = match e(0)? {
                test @ (0..=2 | 5 | 6) => {
                    let mut found = false;
                    for (member, warrior) in state.warriors.iter().enumerate() {
                        let record = warrior.record();
                        let matched = match test {
                            0 => record.profession()? == wanted,
                            1 => record.race()? == wanted,
                            2 => record.gender()? == wanted,
                            5 => caste_class(map, record.profession()?)? == wanted,
                            _ => race_class(map, record.race()?, wanted)?,
                        };
                        found |= matched && (code >= 0 || mask[member] != 0);
                    }
                    found
                }
                3 => crate::exploration::boat::aboard(state),
                4 => state.field_controls.as_ref().is_some_and(|c| c.camping),
                test @ (7 | 8) => {
                    let mut total = 0_i16;
                    for (member, warrior) in state.warriors.iter().enumerate() {
                        if test == 7 || mask[member] != 0 {
                            total = total.wrapping_add(warrior.record().level()?);
                        }
                    }
                    code < total
                }
                _ => false,
            };
            let target = if passed { e(6)? } else { e(8)? };
            if target != 0 {
                encounter(e(4)?, target)
            } else {
                None
            }
        }
        _ => unreachable!("branch operation routing"),
    };
    match jump {
        None => Ok(ScriptStep::Continue),
        Some(jump) => take(jump, state, map, effects),
    }
}

/// The profile row of a race or profession number.
fn profile(table: ProfileTable<'_>, id: i16) -> Result<&[u8], RuleError> {
    usize::try_from(id)
        .ok()
        .filter(|&id| id > 0)
        .and_then(|id| table.row(id - 1).ok())
        .ok_or_else(|| unsupported("story.opcode", "member outside the profiles"))
}

/// The class of a profession. A zero caste keeps the previously loaded profile,
/// which is not held.
///
/// Original: loadprofile
pub(super) fn caste_class(map: &OpeningMission, caste: i16) -> Result<i16, RuleError> {
    Ok(
        ProfessionEquipment(profile(ProfileTable::professions(&map.profiles), caste)?)
            .psi_school()?,
    )
}

/// Whether a race belongs to race class `code - 1`.
///
/// Original: loadprofile
pub(super) fn race_class(map: &OpeningMission, race: i16, code: i16) -> Result<bool, RuleError> {
    let class = usize::try_from(code.wrapping_sub(1))
        .ok()
        .filter(|&bit| bit < 512)
        .ok_or_else(|| unsupported("story.opcode", "race class outside the record"))?;
    Ok(RaceEquipment(profile(ProfileTable::races(&map.races), race)?).has_trait(class)?)
}

/// Jumps the script to another action or runs another encounter, as the branch asked.
fn take(
    jump: Jump,
    state: &mut Session,
    map: &OpeningMission,
    effects: &mut Vec<Event>,
) -> Result<ScriptStep, RuleError> {
    let position = state.exploration.as_mut().unwrap();
    let script = position.script.as_mut().unwrap();
    let saved =
        |script: &crate::ScriptState| {
            // Without an earlier encounter the game would restore a record it never
            // wrote, so this is refused.
            script.menu.is_some().then_some(()).ok_or_else(|| {
                unsupported("story.opcode", "encounter branch without a saved record")
            })
        };
    match jump {
        Jump::Script(id) => {
            script.restart_from(&map.script(id)?);
            Ok(ScriptStep::Continue)
        }
        Jump::End => {
            script.cursor = centurions_content::records::StoryRecord::ACTIONS as u8;
            Ok(ScriptStep::Continue)
        }
        Jump::Simple(id) => {
            saved(script)?;
            effects.push(Event::Sound {
                resource_id: ENCOUNTER_SOUND,
            });
            let table = active_table(&state.menus, &map.menus);
            position.presentation = present_menu(
                map,
                table,
                script,
                id,
                &mut state.globals.encounter_headers,
                true,
            )?;
            if position.presentation.is_none() {
                return Ok(ScriptStep::Continue);
            }
            effects.push(Event::StoryPresented);
            Ok(ScriptStep::Pause)
        }
        Jump::Complex(id) | Jump::ComplexSaved(id) => {
            if matches!(jump, Jump::ComplexSaved(_)) {
                script.menu = None;
            } else {
                saved(script)?;
            }
            let table = active_table(&state.typed_encounters, &map.typed_encounters);
            position.presentation = Some(present_typed(
                map,
                table,
                script,
                id,
                &mut state.globals.encounter_headers,
                true,
            )?);
            effects.push(Event::Sound {
                resource_id: ENCOUNTER_SOUND,
            });
            effects.push(Event::StoryPresented);
            Ok(ScriptStep::Pause)
        }
    }
}
