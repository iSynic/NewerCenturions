//! Presentation script operations, preserving original dispatch order.
use super::super::{
    StoryBattle, abandon_script, begin_story_battle, dialogue, finish_script, unsupported,
};
use super::{Operation, ScriptStep};
use crate::{Event, Presentation, RuleError, Session};
use centurions_content::Content;
use centurions_content::mission::OpeningMission;
use centurions_content::records::{ExtraTable, StoryRecord};
use centurions_content::word;

pub(super) fn execute(
    operation: Operation,
    parameter: i16,
    state: &mut Session,
    content: &Content,
    map: &OpeningMission,
    cursor: usize,
    effects: &mut Vec<Event>,
) -> Result<ScriptStep, RuleError> {
    let position = state.exploration.as_mut().unwrap();
    let script = position.script.as_mut().unwrap();
    match operation {
        Operation::SuppressAllies => state.suppress_allies = parameter,
        Operation::SetMapPicture => {
            crate::story_map_picture::assign(&mut state.map_picture, parameter)
        }
        // Registration is not reimplemented. The original checks, 84 (the registered
        // scenario byte 442604), 98 (doreg) and 99 (regscen), are omitted and the
        // script continues as for a registered copy.
        Operation::Registration
        | Operation::CheckRegisteredScenario
        | Operation::ScenarioRegistration => {}
        // newland14 and -14 run getchoice(|p| - 1, 0, p < 0): up to |p| members are
        // picked, dead ones only for a negative p, the count shrinking by the
        // unavailable members otherwise; -14 then inverts the pick mask.
        Operation::ChooseParty | Operation::ChoosePartyInverted => {
            let unconscious = parameter < 0;
            let unavailable = if unconscious {
                0
            } else {
                i16::from(
                    crate::story_health::context(
                        &state.warriors,
                        state.selected_warrior,
                        state.native_work.health_context.as_ref(),
                    )
                    .unavailable,
                )
            };
            let highest = state.warriors.len() as i16 - 1;
            let remaining = parameter
                .wrapping_abs()
                .wrapping_sub(1)
                .min(highest - unavailable)
                .wrapping_add(1);
            if remaining < 1 {
                return Err(unsupported("story.opcode", "pick with nobody to pick"));
            }
            state.native_work.party_mask.fill(0);
            let position = state.exploration.as_mut().unwrap();
            position.presentation = Some(Presentation::PartyPick {
                remaining,
                unconscious,
                invert: operation == Operation::ChoosePartyInverted,
            });
            effects.push(Event::Feedback(crate::feedback::NativeFeedback::Tutorial {
                first: 105,
                last: 105,
            }));
            effects.push(Event::StoryPresented);
            return Ok(ScriptStep::Pause);
        }
        // newland44 eliminates complex-encounter branch p - 1 in the loaded buffer. Only
        // a running complex encounter uses that buffer before it is reloaded.
        Operation::EliminateComplexBranch => {
            if !(1..=4).contains(&parameter) {
                return Err(unsupported("story.opcode", "branch outside the encounter"));
            }
            if let Some(menu) = script.menu.as_mut().filter(|menu| menu.typed) {
                menu.eliminated |= 1 << (parameter - 1);
            }
        }
        Operation::NoAction | Operation::Unarmed(_) => {}
        // newland28 only redraws the screen.
        Operation::RedrawScreen => effects.push(Event::ScreenRedrawn),
        // newland27 draws a PICT from the open resource files: the Cargo Pod or the
        // scenario. GetPicture's nil handle for a missing one is dereferenced.
        Operation::DrawPicture => {
            let scenario = content
                .scenario
                .as_ref()
                .map(|s| format!("TheNewCenturions/Scenarios/{}/{}/Scenario", s.name, s.level));
            let available = content.resources.iter().any(|resource| {
                resource.r#type == "PICT"
                    && resource.id == parameter
                    && (resource.original == "TheNewCenturions/Data Files/Cargo Pod"
                        || Some(&resource.original) == scenario.as_ref())
            });
            if !available {
                return Err(unsupported("story.opcode", "picture resource missing"));
            }
            effects.push(Event::PictureDrawn {
                resource_id: parameter,
            });
        }
        Operation::ShiftParty => {
            let extra = story_extra(map, parameter)?;
            let random = word(extra, 6)? != 0;
            let mut shift = |amount: i16| {
                if !random {
                    amount
                } else if state.random.bound(100) < 50 {
                    state.random.range(1, amount)
                } else {
                    state.random.range(1, amount).wrapping_neg()
                }
            };
            let (dx, dy) = (shift(word(extra, 2)?), shift(word(extra, 4)?));
            let position = state.exploration.as_mut().unwrap();
            let (x, y) = (position.x + i32::from(dx), position.y + i32::from(dy));
            // The original writes outside the 90-by-90 land without a check.
            OpeningMission::index(x, y)?;
            position.x = x;
            position.y = y;
            let map_id = position.map;
            let script = position.script.as_mut().unwrap();
            centurions_content::records::StoryRecordMut(&mut script.record).relocate(map_id, x, y);
            effects.push(Event::Moved { x, y });
        }
        Operation::BoatCampingBranch => {
            let extra = story_extra(map, parameter)?;
            if word(extra, 4)? != 0 {
                return Err(unsupported("story.opcode", "boat status change"));
            }
            // The reconstruction never boards, so BYTE442748 stays clear.
            let camping = state.field_controls.as_ref().is_some_and(|c| c.camping);
            let condition = word(extra, 2)?;
            if word(extra, 0)? == 1 || (condition == 1 && !camping) || (condition == 2 && camping) {
                script.cursor = StoryRecord::ACTIONS as u8;
            }
        }
        Operation::SetLandLight => {
            let extra = story_extra(map, parameter)?;
            let level = word(extra, 0)?.wrapping_sub(1);
            let regions = &mut position.region_state;
            centurions_content::span(regions, LIGHT_LEVEL, 1)?;
            // The stored byte is zero-extended against the parameter word.
            if word(extra, 2)? != 0 && u16::from(regions[LIGHT_LEVEL]) == level as u16 {
                effects.extend(abandon_script(state, content)?);
                return Ok(ScriptStep::Pause);
            }
            regions[LIGHT_LEVEL] = level as u8;
        }
        // newland63 sets (1) or offsets (2) the day, hour and minute, with no -1 keep,
        // carrying a day and an hour once each. Its timeclick(0) tick skips the hourly
        // work; only an hour of exactly 24 starts the next day and its time encounters.
        Operation::AlterGameTime => {
            let extra = story_extra(map, parameter)?;
            let (day, hour, minute) = (word(extra, 2)?, word(extra, 4)?, word(extra, 6)?);
            let (mut d, mut h, mut m) = (position.day, position.hour, position.minute);
            match word(extra, 0)? {
                1 => (d, h, m) = (day, hour, minute),
                2 => {
                    (d, h) = (d.wrapping_add(day), h.wrapping_add(hour));
                    if h > 23 {
                        (d, h) = (d.wrapping_add(1), h - 24);
                    }
                    m = m.wrapping_add(minute);
                    if m > 59 {
                        (h, m) = (h.wrapping_add(1), m - 60);
                    }
                }
                _ => {}
            }
            // The reconstruction's clock holds hours 0-23, minutes 0-59 and days from 1.
            if !(0..24).contains(&h) || !(0..60).contains(&m) || d < 1 {
                return Err(unsupported("story.opcode", "time outside the clock"));
            }
            (position.day, position.hour, position.minute) = (d, h, m);
        }
        // newland71 stores the parameter in WORD4354fe and redraws the fatigue bar
        // and coordinates.
        Operation::CoordinateDisplay => state.native_work.coordinates_hidden = parameter,
        // newland66 flashes when the old flag differs from the raw parameter word.
        Operation::SetCampingAbility => {
            if u16::from(state.native_work.camp_disabled) != parameter as u16 {
                let address = if parameter == 0 { 0x401c86 } else { 0x401c60 };
                effects.push(Event::Feedback(crate::feedback::camp_flash(address)));
            }
            state.native_work.camp_disabled = parameter != 0;
        }
        // newland104 stores the parameter's low byte in BYTE402f42.
        Operation::SetRandomBattles => state.native_work.random_battles = parameter as u8,
        Operation::ContinueIfMonster => {
            let combat = state.combat.as_ref().ok_or_else(|| {
                unsupported("story.opcode", "monster test outside a battle macro")
            })?;
            if !crate::story_combat::monster_present(combat, parameter)? {
                // newland returns at once; the battle macro owner then finishes.
                script.cursor = StoryRecord::ACTIONS as u8;
            }
        }
        Operation::DisplayRandomString | Operation::AdjustPickedPsi => {
            let extra = story_extra(map, parameter)?;
            let id = if operation == Operation::DisplayRandomString {
                state.random.range(word(extra, 0)?, word(extra, 2)?)
            } else {
                effects.extend(adjust_picked_psi(state, extra)?);
                word(extra, 8)?
            };
            if id != 0 || operation == Operation::DisplayRandomString {
                let position = state.exploration.as_mut().unwrap();
                position.presentation = Some(Presentation::Dialogue {
                    id,
                    text: dialogue(map, &state.warriors, id)?,
                });
                effects.push(Event::StoryPresented);
                return Ok(ScriptStep::Pause);
            }
        }
        Operation::Dialogue => {
            let text = dialogue(map, &state.warriors, parameter)?;
            position.presentation = Some(Presentation::Dialogue {
                id: parameter,
                text,
            });
            effects.push(Event::StoryPresented);
            return Ok(ScriptStep::Pause);
        }
        Operation::Sound => effects.push(Event::Sound {
            resource_id: parameter,
        }),
        Operation::Delay => effects.push(Event::Delay {
            ticks: crate::story_delay::elapsed_ticks(parameter)?,
        }),
        Operation::Question => {
            let previous = cursor
                .checked_sub(1)
                .ok_or_else(|| RuleError::Invalid("question text missing".into()))?;
            let text = map.dialogue(StoryRecord(&script.record).parameter(previous)?)?;
            position.presentation = Some(Presentation::Question {
                extra: parameter,
                text,
            });
            effects.push(Event::Sound { resource_id: 20005 });
            effects.push(Event::StoryPresented);
            return Ok(ScriptStep::Pause);
        }
        Operation::EndScript => {
            effects.extend(finish_script(state, content, false)?);
            return Ok(ScriptStep::Pause);
        }
        Operation::PersistEvent => {
            script.persist = !script.scheduled;
        }
        Operation::Movie => {
            if !map.movies.contains_key(&parameter) && parameter != 1000 {
                return Err(unsupported("story.movie", "other TEXT resources"));
            }
            position.presentation = Some(Presentation::Prologue {
                resource_id: parameter,
            });
            effects.push(Event::StoryPresented);
            return Ok(ScriptStep::Pause);
        }
        Operation::Battle => {
            effects.extend(begin_story_battle(
                state,
                content,
                StoryBattle::Ordinary,
                parameter,
            )?);
            return Ok(ScriptStep::Pause);
        }
        _ => {
            let slot = script.map_slot;
            return Err(
                unsupported("story.opcode", "this story operation is not recovered").with_context(
                    || {
                        format!(
                            "exploration script slot {slot} opcode {} cursor {cursor}",
                            operation.native()
                        )
                    },
                ),
            );
        }
    }
    Ok(ScriptStep::Continue)
}

/// BYTE443290, the light level in the loaded land's region table (base 0x443086).
const LIGHT_LEVEL: usize = 522;

fn story_extra(map: &OpeningMission, id: i16) -> Result<&[u8], RuleError> {
    Ok(ExtraTable(
        &map.combat
            .as_ref()
            .ok_or("story extras unavailable")?
            .encounters,
    )
    .record(id as usize)?)
}

/// newland74: each picked member with nonzero maximum Psi takes the last of
/// |e1| draws of randrange(e2, e3), negated for negative e1, clamped to 0..maximum.
/// Word e4 enables a sound, which plays word e2.
fn adjust_picked_psi(state: &mut Session, extra: &[u8]) -> Result<Vec<Event>, RuleError> {
    let (count, low, high) = (word(extra, 0)?, word(extra, 2)?, word(extra, 4)?);
    let picked: Vec<usize> = (0..state.warriors.len())
        .filter(|&member| state.native_work.party_mask[member] != 0)
        .filter(|&member| {
            state.warriors[member]
                .record()
                .maximum_psi()
                .is_ok_and(|m| m != 0)
        })
        .collect();
    // With no draws the original adds a stale frame word.
    if matches!(count, 0 | i16::MIN) && !picked.is_empty() {
        return Err(unsupported(
            "story.opcode",
            "Psi change without a draw count",
        ));
    }
    let mut effects = Vec::new();
    for member in picked {
        if word(extra, 6)? != 0 {
            effects.push(Event::Sound { resource_id: low });
        }
        let mut amount = 0;
        for _ in 0..count.wrapping_abs() {
            amount = state.random.range(low, high);
        }
        if count < 0 {
            amount = amount.wrapping_neg();
        }
        let record = state.warriors[member].record();
        let maximum = record.maximum_psi()?;
        let psi = record.psi_points()?.wrapping_add(amount).max(0);
        state.warriors[member]
            .record_mut()
            .set_psi_points(if maximum < psi { maximum } else { psi });
    }
    Ok(effects)
}
