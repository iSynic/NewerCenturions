//! Existing state/frame child APIs for the main and battle controllers.
//!
//! Called only inside Execution::call. Its caller owns
//! source save masks, A6, call depth and ordinary versus terminal stack unwind.
//! Existing child internals remain projected. Their plans report RNG deltas,
//! but not processing-step consumption; no instruction-count claim is added.

use crate::RuleError;
use crate::recovered_application_control::{ApplicationControlEffect as Effect, Execution, Result};
use crate::recovered_character_creation as creation;
use crate::recovered_character_panels as character;
use crate::recovered_character_upgrade as upgrade;
use crate::recovered_combat_center as center;
use crate::recovered_combat_ui as combat_ui;
use crate::recovered_feedback as feedback;
use crate::recovered_field_control::{self as field, FieldControlLimits};
use crate::recovered_inventory_selection::{self as selection, InventorySelectionLimits};
use crate::recovered_load_control::{self as load, LoadFlow, LoadLimits};
use crate::recovered_loot_control::{self as loot, LootControlLimits, LootRegisters};
use crate::recovered_prompt_control::{self as prompt, PromptReturn};
use crate::recovered_selection_control::{
    self as survivor, SelectionControlLimits, SelectionRegisters,
};
use crate::recovered_session_control::{
    self as session, GenerationRegisters, SessionControlLimits,
};
use crate::recovered_storage::StorageLimits;

pub(crate) fn dispatch(e: &mut Execution<'_>, entry: u32, frame: u32) -> Result<()> {
    let prompt_limits = e.prompt_limits();
    match entry {
        0x1205f8 => {
            let duration = e.sw(frame.wrapping_add(8))?;
            let duration = if duration == 0 {
                e.sw(0x439568)?
            } else {
                duration
            };
            let limit = i32::from(duration) as u32;
            let slot = frame.wrapping_sub(16);
            e.step()?;
            let start = crate::recovered_timing::TimingServices::tick_count(e.services)?;
            e.long(slot, start)?;
            e.plan.effects.push(Effect::Timing(
                crate::recovered_timing::TimingEffect::TickCount(start),
            ));
            loop {
                e.step()?;
                e.long(frame.wrapping_sub(4), limit)?;
                let tick = crate::recovered_timing::TimingServices::tick_count(e.services)?;
                e.long(slot, tick)?;
                e.plan.effects.push(Effect::Timing(
                    crate::recovered_timing::TimingEffect::TickCount(tick),
                ));
                let elapsed = tick.wrapping_sub(start);
                e.full(elapsed);
                if elapsed > limit {
                    return Ok(());
                }
            }
        }
        0x124d9e => {
            let mut context = crate::recovered_runtime::RuntimeContext::new(
                e.plan.state.random.clone(),
                e.remaining as usize,
            );
            context.memory.bytes = e.plan.state.memory.bytes.clone();
            let plan = crate::recovered_battle_order::getbattleorder(
                &context,
                frame,
                e.sw(frame.wrapping_add(8))?,
                e.draws,
            )?;
            // The typed order plan does not expose the final working D0/CCR.
            e.adopt(
                crate::recovered_rendering::RenderingMemory {
                    bytes: plan.context.memory.bytes,
                },
                plan.context.random,
                PromptReturn::Unchanged,
                LoadFlow::Returned,
                plan.effects.into_iter().map(Effect::Order),
            )
        }
        0x130afe => {
            use crate::recovered_records::{self as records, ITEM_TABLE_ADDRESSES, ItemTables};
            let id = e.sw(frame.wrapping_add(8))?;
            let current: [u8; 150] = e.bytes(0x45363c, 150)?.try_into().expect("item record");
            let mut families: [Vec<u8>; 5] = std::array::from_fn(|_| vec![]);
            let absolute = id.wrapping_abs();
            let family = absolute / 200;
            if let Ok(family) = usize::try_from(family)
                && family < families.len()
            {
                let offset = (absolute % 200) as usize * 150;
                families[family].resize(offset + 150, 0);
                let source = ITEM_TABLE_ADDRESSES[family].wrapping_add(offset as u32);
                families[family][offset..].copy_from_slice(&e.bytes(source, 150)?);
            }
            let tables = ItemTables {
                families: families.each_ref().map(Vec::as_slice),
            };
            let plan = records::loaditem(&tables, &current, id)?;
            for write in plan.writes {
                e.step()?;
                e.write(write.address, &write.bytes)?;
            }
            e.full(plan.native_return as u32);
            Ok(())
        }
        0x136994 => {
            let member = e.w(frame.wrapping_add(8))?;
            let address: u32 = match member {
                0xffff => 0x44721c,
                0xfffe => 0x447e5e,
                _ => 0x448aa0u32.wrapping_add(u32::from(member) * 3138),
            };
            let record = e.bytes(address, 3138)?;
            let race = i16::from_be_bytes([record[50], record[51]]).wrapping_sub(1);
            let race_address = 0x435664u32.wrapping_add(i32::from(race) as u32);
            let start = address.min(race_address);
            let end = address
                .checked_add(3138)
                .and_then(|end| {
                    race_address
                        .checked_add(1)
                        .map(|race_end| end.max(race_end))
                })
                .ok_or_else(|| RuleError::Invalid("application movement arena wraps".into()))?;
            if end - start > 1024 * 1024 {
                return Err(RuleError::Invalid(
                    "application movement arena exceeds one MiB".into(),
                )
                .into());
            }
            // Only the full reached warrior and race byte are read by movecalc.
            // Host padding is never published into the sparse native memory.
            let mut bytes = vec![0; (end - start) as usize];
            let offset = (address - start) as usize;
            bytes[offset..offset + 3138].copy_from_slice(&record);
            bytes[(race_address - start) as usize] = e.b(race_address)?;
            let plan = crate::recovered_navigation::movecalc(
                &crate::recovered_install::InstallMemory {
                    base_address: start,
                    bytes,
                },
                member,
            )?;
            for write in &plan.writes {
                e.step()?;
                e.write(write.address, &write.bytes)?;
            }
            e.full(plan.native_return as u32);
            e.plan.effects.push(Effect::Movement(plan));
            Ok(())
        }
        0x13846a => {
            let value = crate::recovered_navigation::twixt(
                e.sw(frame.wrapping_add(8))?,
                e.sw(frame.wrapping_add(10))?,
                e.sw(frame.wrapping_add(12))?,
            );
            e.full(value as u32);
            Ok(())
        }
        0x13a6d4 => {
            e.full(crate::recovered_verification::doreg() as u32);
            Ok(())
        }
        0x13a1dc | 0x13a234 | 0x13a510 | 0x161b80 => {
            let plan = match entry {
                0x13a1dc => {
                    character::short_update(&e.plan.state.memory, frame, e.remaining, e.services)?
                }
                0x13a234 => {
                    character::select_update(&e.plan.state.memory, frame, e.remaining, e.services)?
                }
                0x13a510 => {
                    character::copy_out(&e.plan.state.memory, frame, e.remaining, e.services)?
                }
                _ => character::update_picture_box(
                    &e.plan.state.memory,
                    frame,
                    e.remaining,
                    e.services,
                )?,
            };
            e.adopt(
                plan.memory,
                e.plan.state.random.clone(),
                PromptReturn::Full(plan.native_return),
                LoadFlow::Returned,
                plan.effects.into_iter().map(Effect::CharacterPanel),
            )
        }
        0x13a2c4 | 0x1368c8 => {
            let plan = if entry == 0x13a2c4 {
                center::center(&e.plan.state.memory, frame, e.remaining, e.services)?
            } else {
                center::exitcapturemode(&e.plan.state.memory, frame, e.remaining, e.services)?
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
        0x123c60 | 0x138be8 => {
            let plan = if entry == 0x123c60 {
                let limits = LoadLimits {
                    filename_capacity: prompt_limits.filename_capacity,
                    maximum_polls: prompt_limits.maximum_polls,
                };
                load::load_door(&e.plan.state.memory, frame, limits, e.services)?
            } else {
                load::scratch(&e.plan.state.memory, frame, e.services)?
            };
            let result = if plan.flow == LoadFlow::Returned {
                plan.native_return
                    .map_or(PromptReturn::Unchanged, PromptReturn::Full)
            } else {
                PromptReturn::NoReturn
            };
            e.adopt(
                plan.memory,
                e.plan.state.random.clone(),
                result,
                plan.flow,
                plan.effects.into_iter().map(Effect::Load),
            )
        }
        0x15ec56 => {
            let plan = combat_ui::tickcheck(&e.plan.state.memory, frame, e.services)?;
            e.adopt(
                plan.memory,
                e.plan.state.random.clone(),
                plan.native_return
                    .map_or(PromptReturn::Unchanged, PromptReturn::Full),
                LoadFlow::Returned,
                plan.effects.into_iter().map(Effect::CombatUi),
            )
        }
        0x14ce2a => {
            let kind = e.sw(frame.wrapping_add(8))?;
            let phase = e.sw(frame.wrapping_add(10))?;
            let sound = e.sw(frame.wrapping_add(12))?;
            let plan =
                feedback::pressbutton(&e.plan.state.memory, frame, kind, phase, sound, e.services)?;
            e.adopt(
                plan.memory,
                e.plan.state.random.clone(),
                plan.native_return
                    .map_or(PromptReturn::Unchanged, PromptReturn::Full),
                LoadFlow::Returned,
                plan.effects.into_iter().map(Effect::Feedback),
            )
        }
        0x146b6a => {
            let limits = StorageLimits {
                filename_capacity: prompt_limits.filename_capacity,
                maximum_iterations: e.remaining as usize,
                maximum_polls: prompt_limits.maximum_polls,
            };
            let plan = crate::recovered_order_form::order_form(
                &e.plan.state.memory,
                &e.plan.state.random,
                frame,
                limits,
                e.services,
            )?;
            let result = if plan.flow == LoadFlow::Returned {
                plan.native_return
                    .map_or(PromptReturn::Unchanged, PromptReturn::Full)
            } else {
                PromptReturn::NoReturn
            };
            e.adopt(
                plan.memory,
                plan.random,
                result,
                plan.flow,
                plan.effects.into_iter().map(Effect::Startup),
            )
        }
        0x12382a | 0x133d46 | 0x14cff6 | 0x1500d8 | 0x16043e => {
            let plan = match entry {
                0x12382a => {
                    prompt::first_tutorial(&e.plan.state, frame, prompt_limits, e.services)?
                }
                0x133d46 => prompt::submit_music(&e.plan.state, frame, prompt_limits, e.services)?,
                0x14cff6 => prompt::question(&e.plan.state, frame, prompt_limits, e.services)?,
                0x1500d8 => {
                    prompt::reset_text_area(&e.plan.state, frame, prompt_limits, e.services)?
                }
                _ => prompt::update_controls(&e.plan.state, frame, prompt_limits, e.services)?,
            };
            e.adopt(
                plan.state.memory,
                plan.state.random,
                plan.native_return,
                plan.flow,
                plan.effects.into_iter().map(Effect::Prompt),
            )
        }
        0x13a87c | 0x13f36a | 0x152378 | 0x1378a2 => {
            let limits = SessionControlLimits {
                maximum_steps: e.remaining,
                maximum_polls: prompt_limits.maximum_polls,
                filename_capacity: prompt_limits.filename_capacity,
                maximum_random_draws: e.draws,
                equipment: e.limits.equipment,
            };
            let plan = match entry {
                0x13a87c => session::quit_game(&e.plan.state, frame, limits, e.services)?,
                0x152378 => session::save(&e.plan.state, frame, limits, e.services)?,
                0x1378a2 => session::register_scenario(&e.plan.state, frame, limits, e.services)?,
                _ => {
                    // The existing API diagnoses missing inherited values only
                    // when its portrait/icon path actually requires them.
                    let registers = match (
                        e.registers.data[3].word("D3"),
                        e.registers.data[6].word("D6"),
                    ) {
                        (Ok(d3), Ok(d6)) => Some(GenerationRegisters {
                            d3_word: d3 as i16,
                            d6_word: d6 as i16,
                        }),
                        _ => None,
                    };
                    session::generate(&e.plan.state, frame, registers, limits, e.services)?
                }
            };
            e.adopt(
                plan.state.memory,
                plan.state.random,
                plan.native_return,
                plan.flow,
                plan.effects.into_iter().map(Effect::Session),
            )
        }
        0x111984 | 0x1374bc | 0x13ebde | 0x1538ba => {
            let limits = FieldControlLimits {
                maximum_steps: e.remaining,
                maximum_polls: prompt_limits.maximum_polls,
                filename_capacity: prompt_limits.filename_capacity,
                maximum_random_draws: e.draws,
                equipment: e.limits.equipment,
            };
            let plan = match entry {
                0x111984 => field::check_keypad(&e.plan.state, frame, limits, e.services)?,
                0x1374bc => field::save_character(&e.plan.state, frame, limits, e.services)?,
                0x13ebde => field::movie(&e.plan.state, frame, limits, e.services)?,
                _ => field::setup_new_game(&e.plan.state, frame, limits, e.services)?,
            };
            e.adopt(
                plan.state.memory,
                plan.state.random,
                plan.native_return,
                plan.flow,
                plan.effects.into_iter().map(Effect::Field),
            )
        }
        0x1289fa => {
            let incoming_d6 = e.registers.data[6].word("D6")? as i16;
            let plan = creation::icon_picture(
                &e.plan.state,
                frame,
                incoming_d6,
                prompt_limits,
                e.services,
            )?;
            e.adopt(
                plan.state.memory,
                plan.state.random,
                plan.native_return,
                plan.flow,
                plan.effects.into_iter().map(Effect::Creation),
            )
        }
        0x1476cc => {
            let plan = upgrade::party_select(&e.plan.state, frame, prompt_limits, e.services)?;
            e.adopt(
                plan.state.memory,
                plan.state.random,
                plan.native_return,
                plan.flow,
                plan.effects.into_iter().map(Effect::Upgrade),
            )
        }
        0x152188 | 0x160a8e => {
            let limits = InventorySelectionLimits {
                maximum_steps: e.remaining,
                prompt: prompt_limits,
                equipment: e.limits.equipment,
                maximum_random_draws: e.draws,
            };
            let plan = if entry == 0x152188 {
                selection::order(&e.plan.state, frame, limits, e.services)?
            } else {
                selection::update_fat(&e.plan.state, frame, limits, e.services)?
            };
            e.adopt(
                plan.state.memory,
                plan.state.random,
                plan.native_return,
                plan.flow,
                plan.effects.into_iter().map(Effect::Selection),
            )
        }
        0x103ebe => {
            let registers = SelectionRegisters {
                d3_byte: (e.registers.data[3].known_mask & 0xff == 0xff)
                    .then_some(e.registers.data[3].value as u8),
                d4_word: e.registers.data[4].word("D4").ok().map(|v| v as i16),
            };
            let limits = SelectionControlLimits {
                prompt: prompt_limits,
                maximum_random_draws: e.draws,
            };
            let plan = survivor::beast(&e.plan.state, frame, registers, limits, e.services)?;
            e.adopt(
                plan.state.memory,
                plan.state.random,
                plan.native_return,
                plan.flow,
                plan.effects.into_iter().map(Effect::Survivor),
            )
        }
        0x105748 => {
            let registers = LootRegisters {
                a2: e.registers.address[2].long("A2").ok(),
                a4: e.registers.address[4].long("A4").ok(),
            };
            let limits = LootControlLimits {
                maximum_steps: e.remaining,
                prompt: prompt_limits,
                equipment: e.limits.equipment,
                maximum_random_draws: e.draws,
            };
            let plan = loot::booty(&e.plan.state, frame, registers, limits, e.services)?;
            e.adopt(
                plan.state.memory,
                plan.state.random,
                plan.native_return,
                plan.flow,
                plan.effects.into_iter().map(Effect::Loot),
            )
        }
        _ => super::gameplay_children::dispatch(e, entry, frame),
    }
}
