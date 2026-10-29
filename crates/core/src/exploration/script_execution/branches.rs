//! newland "Branch on" operations that jump to an extra action point or encounter.
use super::super::{finish_script, present_menu, present_typed, unsupported};
use super::{Operation, ScriptStep};
use crate::{Event, RuleError, Session};
use centurions_content::mission::OpeningMission;
use centurions_content::records::{
    Condition, ExtraTable, ProfessionEquipment, RaceEquipment, WarriorRecord,
};
use centurions_content::{Content, span, word};

/// Where a passed test sends the script.
enum Jump {
    /// loaddoor2 and a restart at the first action.
    Script(i16),
    /// The simple or complex encounter body entered past its record save
    /// (0x142b3e, 0x142d96), so the encounter returns to the record saved earlier.
    Simple(i16),
    Complex(i16),
    /// The complex encounter case itself, which saves the current record.
    ComplexSaved(i16),
    /// The cursor moves past the last action.
    End,
}

pub(super) fn execute(
    operation: Operation,
    parameter: i16,
    state: &mut Session,
    content: &Content,
    map: &OpeningMission,
    effects: &mut Vec<Event>,
) -> Result<ScriptStep, RuleError> {
    let extra = ExtraTable(
        &map.combat
            .as_ref()
            .ok_or("story extras unavailable")?
            .encounters,
    )
    .record(parameter as usize)?;
    state.native_work.extra_record = extra.to_vec();
    let e = |offset: usize| word(extra, offset);
    let encounter = |mode: i16, id: i16| match mode {
        0 => Some(Jump::Script(id)),
        1 => Some(Jump::Simple(id)),
        2 => Some(Jump::Complex(id)),
        _ => None,
    };
    let jump = match operation {
        // newland40 tests one party-protection word for set (1) or clear (2).
        Operation::PartyConditionBranch => {
            let buff = usize::try_from(e(6)?)
                .ok()
                .and_then(|index| word(&state.party_buffs, index * 2).ok())
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
        // newland64 compares the hour twice and has no skip value.
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
        // newland75: picked members (1) or living, uncontrolled members (2).
        Operation::PsiPointsBranch => {
            let required = e(2)?;
            let found = state.warriors.iter().enumerate().any(|(member, warrior)| {
                let record = warrior.record();
                let eligible = match e(0) {
                    Ok(1) => state.native_work.party_mask[member] != 0,
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
                // The failed test exits like opcode 24 and keeps the codes.
                effects.extend(finish_script(state, content, false)?);
                return Ok(ScriptStep::Pause);
            } else {
                None
            }
        }
        // newland81: every picked member (-1), the whole party (0) or member e2,
        // indexed from zero.
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
                    .filter(|&m| state.native_work.party_mask[m] != 0)
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
        // newland55: modes 1-6 all test the first member's pick word, and other
        // modes need exactly |mode| picked members.
        Operation::PickedBranch => {
            let mask = &state.native_work.party_mask;
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
        // newland86: a negative code tests only picked members for its magnitude.
        // Caste and race classes come from each member's profession and race record.
        Operation::MiscellaneousBranch => {
            let code = e(2)?;
            let wanted = if code < 0 { code.wrapping_abs() } else { code };
            let mask = &state.native_work.party_mask;
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

fn profile(table: &[u8], id: i16, bytes: usize) -> Result<&[u8], RuleError> {
    usize::try_from(id)
        .ok()
        .filter(|&id| id > 0)
        .and_then(|id| span(table, (id - 1) * bytes, bytes).ok())
        .ok_or_else(|| unsupported("story.opcode", "member outside the profiles"))
}

/// loadprofile(0, caste), then the Data MOS record's caste class (WORD 254). A zero
/// caste keeps the previously loaded record, which is not held.
pub(super) fn caste_class(map: &OpeningMission, caste: i16) -> Result<i16, RuleError> {
    Ok(ProfessionEquipment(profile(&map.profiles, caste, 576)?).psi_school()?)
}

/// loadprofile(race, 0), then BitTst of race class `code - 1` from byte 344.
pub(super) fn race_class(map: &OpeningMission, race: i16, code: i16) -> Result<bool, RuleError> {
    let class = usize::try_from(code.wrapping_sub(1))
        .ok()
        .filter(|&bit| bit < 512)
        .ok_or_else(|| unsupported("story.opcode", "race class outside the record"))?;
    Ok(RaceEquipment(profile(&map.races, race, 408)?).has_trait(class)?)
}

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
            // Without an earlier encounter the original restores an unwritten frame record.
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
            effects.push(Event::Sound { resource_id: 20005 });
            let table = if state.menus.is_empty() {
                &map.menus
            } else {
                &state.menus
            };
            position.presentation = present_menu(
                map,
                table,
                script,
                id,
                &mut state.native_work.encounter_work,
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
            let table = if state.typed_encounters.is_empty() {
                &map.typed_encounters
            } else {
                &state.typed_encounters
            };
            position.presentation = Some(present_typed(
                map,
                table,
                script,
                id,
                &mut state.native_work.encounter_work,
                true,
            )?);
            effects.push(Event::Sound { resource_id: 20005 });
            effects.push(Event::StoryPresented);
            Ok(ScriptStep::Pause)
        }
    }
}
