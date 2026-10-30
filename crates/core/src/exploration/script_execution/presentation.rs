//! Script operations that show text, pictures, sounds and choices.
use super::super::{StoryBattle, begin_story_battle, dialogue, finish_script, unsupported};
use super::{ExtraIndex, Operation, ScriptStep, load_extra};
use crate::{Event, Presentation, RuleError, Session};
use centurions_content::Content;
use centurions_content::mission::OpeningMission;
use centurions_content::records::{MapEffectExtra, StoryRecord};
use centurions_content::word;

/// Runs the script operations that show text, pictures, sounds, movies and questions, pause or end
/// the script, and let the player pick party members.
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
        Operation::SuppressAllies => state.suspend_allies = parameter,
        Operation::SetMapPicture => {
            crate::story::map_picture::assign(&mut state.map_picture, parameter)
        }
        // Registration is always satisfied: the registration checks (84, 98 and
        // 99) are omitted and the script continues as for a registered copy.
        Operation::Registration
        | Operation::CheckRegisteredScenario
        | Operation::ScenarioRegistration => {}
        // Up to |p| members are picked, dead ones only for a negative p; otherwise
        // the count shrinks by the unavailable members. The inverted form then
        // flips the picks.
        Operation::ChooseParty | Operation::ChoosePartyInverted => {
            let unconscious = parameter < 0;
            let kill_party = if unconscious {
                0
            } else {
                i16::from(
                    crate::story::health::reselected_context(
                        state.char_select_new,
                        &state.globals.health_context,
                    )
                    .kill_party,
                )
            };
            let highest = state.warriors.len() as i16 - 1;
            let remaining = parameter
                .wrapping_abs()
                .wrapping_sub(1)
                .min(highest - kill_party)
                .wrapping_add(1);
            if remaining < 1 {
                return Err(unsupported("story.opcode", "pick with nobody to pick"));
            }
            state.globals.party_track.fill(0);
            let position = state.exploration.as_mut().unwrap();
            position.presentation = Some(Presentation::PartyPick {
                remaining,
                unconscious,
                invert: operation == Operation::ChoosePartyInverted,
            });
            effects.push(Event::Feedback(crate::feedback::GameFeedback::Tutorial {
                first: PARTY_PICK_TUTORIAL,
                last: PARTY_PICK_TUTORIAL,
            }));
            effects.push(Event::StoryPresented);
            return Ok(ScriptStep::Pause);
        }
        // Removes branch p - 1 of the loaded complex encounter. Only a running
        // complex encounter uses it before it is reloaded.
        Operation::EliminateComplexBranch => {
            if !(1..=4).contains(&parameter) {
                return Err(unsupported("story.opcode", "branch outside the encounter"));
            }
            state
                .globals
                .encounter_headers
                .eliminate(1 << (parameter - 1))?;
            if let Some(menu) = script.menu.as_mut().filter(|menu| menu.typed) {
                menu.eliminated |= 1 << (parameter - 1);
            }
        }
        Operation::NoAction | Operation::Unarmed(_) => {}
        Operation::RedrawScreen => effects.push(Event::ScreenRedrawn),
        // Draws a PICT from the Cargo Pod or the scenario. The game crashes on a
        // missing picture.
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
        // Also redraws the fatigue bar and coordinates.
        Operation::CoordinateDisplay => state.globals.coordinates_hidden = parameter,
        // Flashes when the old setting differs from the raw parameter.
        Operation::SetCampingAbility => {
            if u16::from(state.globals.camp_disabled) != parameter as u16 {
                let address = if parameter == 0 { 0x401c86 } else { 0x401c60 };
                effects.push(Event::Feedback(crate::feedback::camp_flash(address)));
            }
            state.globals.camp_disabled = parameter != 0;
        }
        // Keeps the parameter's low byte.
        Operation::SetRandomBattles => state.globals.random_battles = parameter as u8,
        Operation::DisplayRandomString | Operation::AdjustPickedPsi => {
            let extra = load_extra(
                &mut state.globals,
                map,
                ExtraIndex::Signed(parameter),
                Some("story extras unavailable"),
            )?;
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
            ticks: crate::story::delay::elapsed_ticks_with_default(
                parameter,
                state.command_preferences.animation_delay,
            )?,
        }),
        Operation::Question => {
            load_extra(
                &mut state.globals,
                map,
                ExtraIndex::Signed(parameter),
                Some("story extras unavailable"),
            )?;
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
        // Saves the script's position so the end of the script can write the event
        // into the header row.
        Operation::PersistEvent => {
            script.persist = true;
            script.persist_cell = script.registers;
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
        Operation::Pause => {
            position.presentation = Some(Presentation::ScriptPause);
            effects.push(Event::Feedback(crate::feedback::script_pause()));
            return Ok(ScriptStep::Pause);
        }
        Operation::MapEffect => {
            let extra = load_extra(&mut state.globals, map, ExtraIndex::Signed(parameter), None)?;
            let effect = MapEffectExtra(extra);
            effects.push(Event::OutdoorEffect {
                effect: effect.effect()?,
                x: effect.x()?,
                y: effect.y()?,
                size: effect.size()?,
                delay: effect.delay()?,
            });
        }
        Operation::ChooseWarrior => {
            position.presentation = Some(Presentation::WarriorChoice { extra: parameter });
            effects.push(Event::StoryPresented);
            return Ok(ScriptStep::Pause);
        }
        // Every operation names its adapter, so none reaches this arm; it keeps
        // the unsupported-operation diagnostic.
        _ => {
            let slot = script.map_slot;
            return Err(
                unsupported("story.opcode", "this story operation is not recovered").with_context(
                    || {
                        format!(
                            "exploration script slot {slot} opcode {} cursor {cursor}",
                            operation.opcode()
                        )
                    },
                ),
            );
        }
    }
    Ok(ScriptStep::Continue)
}

/// Tutorial 105, shown as a party pick begins.
const PARTY_PICK_TUTORIAL: usize = 105;

/// Each picked member with nonzero maximum Psi takes the last of |e1| draws of
/// randrange(e2, e3), negated for negative e1, clamped to 0..maximum. A nonzero
/// e4 plays sound e2.
///
/// Original: newland
fn adjust_picked_psi(state: &mut Session, extra: &[u8]) -> Result<Vec<Event>, RuleError> {
    let (count, low, high) = (word(extra, 0)?, word(extra, 2)?, word(extra, 4)?);
    let picked: Vec<usize> = (0..state.warriors.len())
        .filter(|&member| state.globals.party_track[member] != 0)
        .filter(|&member| {
            state.warriors[member]
                .record()
                .psi_points_max()
                .is_ok_and(|m| m != 0)
        })
        .collect();
    // With no draws the game adds a leftover value; that is refused here.
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
        let maximum = record.psi_points_max()?;
        let psi = record.psi_points()?.wrapping_add(amount).max(0);
        state.warriors[member]
            .record_mut()
            .set_psi_points(if maximum < psi { maximum } else { psi });
    }
    Ok(effects)
}
