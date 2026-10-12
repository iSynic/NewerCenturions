//! Existing typed children reached by spell application control.

use crate::recovered_application_control::{ApplicationControlEffect as Effect, Execution, Result};
use crate::recovered_character_panels::{self as character, CharacterPanelServices};
use crate::recovered_combat_center as center;
use crate::recovered_combat_results::{self as results, CombatResultReturn, CombatResultState};
use crate::recovered_feedback as feedback;
use crate::recovered_load_control::LoadFlow;
use crate::recovered_prompt_control::{self as prompt, PromptReturn};
use crate::recovered_runtime::{self as runtime, RuntimeResult};
use crate::recovered_world_display::WorldDisplayState;

pub(crate) fn dispatch(e: &mut Execution<'_>, entry: u32, frame: u32) -> Result<()> {
    match entry {
        0x112650 | 0x160a8e => {
            let limits = crate::recovered_inventory_selection::InventorySelectionLimits {
                maximum_steps: e.remaining,
                prompt: e.prompt_limits(),
                equipment: e.limits.equipment,
                maximum_random_draws: e.draws,
            };
            let plan = if entry == 0x112650 {
                crate::recovered_inventory_selection::check_for_item(
                    &e.plan.state,
                    frame,
                    limits,
                    e.services,
                )?
            } else {
                crate::recovered_inventory_selection::update_fat(
                    &e.plan.state,
                    frame,
                    limits,
                    e.services,
                )?
            };
            e.adopt(
                plan.state.memory,
                plan.state.random,
                plan.native_return,
                plan.flow,
                plan.effects.into_iter().map(Effect::Selection),
            )
        }
        0x13822a => {
            for write in crate::recovered_records::cleartarget().writes {
                e.step()?;
                e.write(write.address, &write.bytes)?;
            }
            e.full(109);
            Ok(())
        }
        0x150114 | 0x15075a | 0x15b2f6 | 0x158454 => {
            let state = CombatResultState {
                memory: e.plan.state.memory.clone(),
                random: e.plan.state.random.clone(),
            };
            let limits = e.prompt_limits();
            let plan = match entry {
                0x150114 => results::resist(&state, frame, limits, e.services)?,
                0x15075a => results::resolvedamage(&state, frame, limits, e.services)?,
                0x15b2f6 => results::spelleffect(&state, frame, limits, e.services)?,
                _ => results::showresults(&state, frame, limits, e.services)?,
            };
            let result = match plan.native_return {
                CombatResultReturn::Full(v) => PromptReturn::Full(v),
                CombatResultReturn::LowWord(v) => PromptReturn::LowWord(v),
                CombatResultReturn::NoReturn => PromptReturn::NoReturn,
            };
            e.adopt(
                plan.state.memory,
                plan.state.random,
                result,
                plan.flow,
                plan.effects.into_iter().map(Effect::CombatResult),
            )
        }
        0x1600fc | 0x15f66c => {
            let plan = if entry == 0x1600fc {
                character::update_character_short(
                    &e.plan.state.memory,
                    frame,
                    e.remaining,
                    e.services,
                )?
            } else {
                character::update_character(&e.plan.state.memory, frame, e.remaining, e.services)?
            };
            e.adopt(
                plan.memory,
                e.plan.state.random.clone(),
                PromptReturn::Full(plan.native_return),
                LoadFlow::Returned,
                plan.effects.into_iter().map(Effect::CharacterPanel),
            )
        }
        0x10e6b6 | 0x10f186 | 0x1107cc => {
            let plan = match entry {
                0x10e6b6 => {
                    center::centerfield(&e.plan.state.memory, frame, e.remaining, e.services)?
                }
                0x1107cc => {
                    center::centerstage2(&e.plan.state.memory, frame, e.remaining, e.services)?
                }
                _ => center::centerpict(
                    &WorldDisplayState {
                        memory: e.plan.state.memory.clone(),
                        random: e.plan.state.random.clone(),
                    },
                    frame,
                    e.remaining,
                    e.services,
                )?,
            };
            e.adopt(
                plan.memory,
                e.plan.state.random.clone(),
                plan.native_return
                    .map_or(PromptReturn::Unchanged, PromptReturn::Full),
                LoadFlow::Returned,
                plan.effects.into_iter().map(Effect::CombatCenter),
            )
        }
        0x16043e | 0x1500d8 => {
            let limits = e.prompt_limits();
            let plan = if entry == 0x16043e {
                prompt::update_controls(&e.plan.state, frame, limits, e.services)?
            } else {
                prompt::reset_text_area(&e.plan.state, frame, limits, e.services)?
            };
            e.adopt(
                plan.state.memory,
                plan.state.random,
                plan.native_return,
                plan.flow,
                plan.effects.into_iter().map(Effect::Prompt),
            )
        }
        0x133a18 | 0x16474a | 0x123d1c => {
            let plan = match entry {
                0x133a18 => feedback::sound(
                    &e.plan.state.memory,
                    e.sw(frame.wrapping_add(8))?,
                    e.services,
                )?,
                0x16474a => feedback::warn(
                    &e.plan.state.memory,
                    frame,
                    e.sw(frame.wrapping_add(8))?,
                    e.limits.prompt.maximum_polls,
                    e.services,
                )?,
                _ => feedback::flashmessage(
                    &e.plan.state.memory,
                    frame,
                    e.l(frame.wrapping_add(8))?,
                    e.sw(frame.wrapping_add(12))?,
                    e.sw(frame.wrapping_add(14))?,
                    e.sw(frame.wrapping_add(16))?,
                    e.sw(frame.wrapping_add(18))?,
                    e.limits.prompt.maximum_polls,
                    e.services,
                )?,
            };
            e.adopt(
                plan.memory,
                e.plan.state.random.clone(),
                plan.native_return
                    .map_or(PromptReturn::Unchanged, PromptReturn::Full),
                LoadFlow::Returned,
                plan.effects.into_iter().map(Effect::Feedback),
            )
        }
        0x13a498 => {
            let observations = CharacterPanelServices::runtime_observations(
                e.services,
                entry,
                &e.plan.state.memory,
            )?;
            let mut context =
                runtime::RuntimeContext::new(e.plan.state.random.clone(), e.remaining as usize);
            context.memory.bytes = e.plan.state.memory.bytes.clone();
            let plan = runtime::copy_in(&mut context, &observations)?;
            let result = match plan.result {
                RuntimeResult::Long(v) => PromptReturn::Full(v),
                _ => PromptReturn::Unchanged,
            };
            let mut memory = e.plan.state.memory.clone();
            memory.bytes = context.memory.bytes;
            e.adopt(
                memory,
                context.random,
                result,
                LoadFlow::Returned,
                plan.effects.into_iter().map(Effect::Runtime),
            )
        }
        0x13154c => {
            e.full(crate::recovered_interface::locationlog() as u32);
            Ok(())
        }
        _ => Err(crate::RuleError::Unsupported {
            action: "application_control.spell_child",
            dependency: "the reached entry has no existing spell child adapter",
        }
        .into()),
    }
}
