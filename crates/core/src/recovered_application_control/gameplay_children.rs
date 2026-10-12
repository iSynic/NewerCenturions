//! Addressed adapters shared by gameplay and outer application control.

use super::{ApplicationControlEffect as Effect, ApplicationControlServices, Execution, Result};
use crate::RuleError;
use crate::recovered_inventory_interaction::{self as interaction, ItemsRegisters};
use crate::recovered_load_control::{self as load, LoadFlow, LoadLimits};
use crate::recovered_prompt_control::{self as prompt, PromptReturn};
use crate::recovered_storage::{self as storage, StorageLimits};

pub(crate) fn dispatch<S: ApplicationControlServices>(
    e: &mut Execution<'_, S>,
    entry: u32,
    frame: u32,
) -> Result<()> {
    let prompt_limits = e.prompt_limits();
    match entry {
        0x100642 | 0x13a5a6 | 0x15a9cc | 0x156b26 | 0x15f4b8 => {
            use crate::recovered_inventory_selection as selection;
            let limits = selection::InventorySelectionLimits {
                maximum_steps: e.remaining,
                prompt: prompt_limits,
                equipment: e.limits.equipment,
                maximum_random_draws: e.draws,
            };
            let plan = match entry {
                0x100642 => selection::action_picker(&e.plan.state, frame, limits, e.services)?,
                0x13a5a6 => selection::check_for_do_auto(&e.plan.state, frame, limits, e.services)?,
                0x15a9cc => selection::scan_select(&e.plan.state, frame, limits, e.services)?,
                0x156b26 => selection::show_items(&e.plan.state, frame, limits, e.services)?,
                _ => selection::update_booty(&e.plan.state, frame, limits, e.services)?,
            };
            e.adopt(
                plan.state.memory,
                plan.state.random,
                plan.native_return,
                plan.flow,
                plan.effects.into_iter().map(Effect::Selection),
            )
        }
        0x112820 | 0x13c1ee | 0x152ec2 => {
            let limits = interaction::InventoryInteractionLimits {
                maximum_steps: e.remaining,
                prompt: prompt_limits,
                equipment: e.limits.equipment,
                maximum_random_draws: e.draws,
            };
            let plan = match entry {
                0x112820 => interaction::choice(&e.plan.state, frame, limits, e.services)?,
                0x13c1ee => interaction::move_icon(&e.plan.state, frame, limits, e.services)?,
                _ => interaction::see_shop(&e.plan.state, frame, limits, e.services)?,
            };
            e.adopt(
                plan.state.memory,
                plan.state.random,
                plan.native_return,
                plan.flow,
                plan.effects.into_iter().map(Effect::Interaction),
            )
        }
        0x121596 | 0x122e24 | 0x12fe54 | 0x136928 | 0x13add4 | 0x13ea82 | 0x14edd0 => {
            use crate::recovered_field_control as field;
            let limits = field::FieldControlLimits {
                maximum_steps: e.remaining,
                maximum_polls: prompt_limits.maximum_polls,
                filename_capacity: prompt_limits.filename_capacity,
                maximum_random_draws: e.draws,
                equipment: e.limits.equipment,
            };
            let plan = match entry {
                0x121596 => field::encounter(&e.plan.state, frame, limits, e.services)?,
                0x122e24 => field::file_prep(&e.plan.state, frame, limits, e.services)?,
                0x12fe54 => field::load_shop(&e.plan.state, frame, limits, e.services)?,
                0x136928 => field::enter_capture_mode(&e.plan.state, frame, limits, e.services)?,
                0x13add4 => field::band_aid(&e.plan.state, frame, limits, e.services)?,
                0x13ea82 => field::move_party(&e.plan.state, frame, limits, e.services)?,
                _ => field::do_reload(&e.plan.state, frame, limits, e.services)?,
            };
            e.adopt(
                plan.state.memory,
                plan.state.random,
                plan.native_return,
                plan.flow,
                plan.effects.into_iter().map(Effect::Field),
            )
        }
        0x14d21a | 0x14d596 => {
            use crate::recovered_session_control as session;
            let limits = session::SessionControlLimits {
                maximum_steps: e.remaining,
                maximum_polls: prompt_limits.maximum_polls,
                filename_capacity: prompt_limits.filename_capacity,
                maximum_random_draws: e.draws,
                equipment: e.limits.equipment,
            };
            let plan = if entry == 0x14d21a {
                session::question2(&e.plan.state, frame, limits, e.services)?
            } else {
                session::question3(&e.plan.state, frame, limits, e.services)?
            };
            e.adopt(
                plan.state.memory,
                plan.state.random,
                plan.native_return,
                plan.flow,
                plan.effects.into_iter().map(Effect::Session),
            )
        }
        0x15dc34 => {
            let plan = prompt::text_box(&e.plan.state, frame, prompt_limits, e.services)?;
            e.adopt(
                plan.state.memory,
                plan.state.random,
                plan.native_return,
                plan.flow,
                plan.effects.into_iter().map(Effect::Prompt),
            )
        }
        0x1047f8 | 0x10b89e | 0x117356 => {
            use crate::recovered_selection_control as selection;
            let limits = selection::SelectionControlLimits {
                prompt: prompt_limits,
                maximum_random_draws: e.draws,
            };
            let plan = match entry {
                0x1047f8 => selection::body_count(&e.plan.state, frame, limits, e.services)?,
                0x117356 => selection::combat_errors(&e.plan.state, frame, limits, e.services)?,
                _ => {
                    let registers = selection::SelectionRegisters {
                        d3_byte: (e.registers.data[3].known_mask & 0xff == 0xff)
                            .then_some(e.registers.data[3].value as u8),
                        d4_word: e.registers.data[4]
                            .word("D4")
                            .ok()
                            .map(|value| value as i16),
                    };
                    selection::cast_spell(&e.plan.state, frame, registers, limits, e.services)?
                }
            };
            e.adopt(
                plan.state.memory,
                plan.state.random,
                plan.native_return,
                plan.flow,
                plan.effects.into_iter().map(Effect::Survivor),
            )
        }
        0x13107a | 0x1383e6 | 0x131182 | 0x138efe => {
            let limits = LoadLimits {
                filename_capacity: prompt_limits.filename_capacity,
                maximum_polls: prompt_limits.maximum_polls,
            };
            let plan = match entry {
                0x13107a => load::load_land(&e.plan.state.memory, frame, limits, e.services)?,
                0x1383e6 => load::load_extra_code(&e.plan.state.memory, frame, limits, e.services)?,
                0x131182 => load::load_pixmap(&e.plan.state.memory, frame, limits, e.services)?,
                _ => load::scratch2(&e.plan.state.memory, frame, e.services)?,
            };
            e.adopt(
                plan.memory,
                e.plan.state.random.clone(),
                if plan.flow == LoadFlow::Returned {
                    plan.native_return
                        .map_or(PromptReturn::Unchanged, PromptReturn::Full)
                } else {
                    PromptReturn::NoReturn
                },
                plan.flow,
                plan.effects.into_iter().map(Effect::Load),
            )
        }
        0x13006e | 0x151e24 | 0x152dc4 | 0x14f96e => {
            let limits = StorageLimits {
                filename_capacity: prompt_limits.filename_capacity,
                maximum_iterations: e.remaining as usize,
                maximum_polls: prompt_limits.maximum_polls,
            };
            let plan = match entry {
                0x13006e => storage::save_shop(&e.plan.state.memory, limits, e.services)?,
                0x151e24 => storage::save_land(&e.plan.state.memory, frame, limits, e.services)?,
                0x152dc4 => {
                    storage::save_encounter(&e.plan.state.memory, frame, limits, e.services)?
                }
                _ => storage::reset_game_files(&e.plan.state.memory, frame, limits, e.services)?,
            };
            e.adopt(
                plan.memory,
                e.plan.state.random.clone(),
                if plan.flow == LoadFlow::Returned {
                    plan.native_return
                        .map_or(PromptReturn::Unchanged, PromptReturn::Full)
                } else {
                    PromptReturn::NoReturn
                },
                plan.flow,
                plan.effects.into_iter().map(Effect::Storage),
            )
        }
        0x127132 | 0x133f3e | 0x13b212 | 0x135bb2 | 0x134d9e => {
            use crate::recovered_scenario_menus as scenario;
            let limits = StorageLimits {
                filename_capacity: prompt_limits.filename_capacity,
                maximum_iterations: e.remaining as usize,
                maximum_polls: prompt_limits.maximum_polls,
            };
            let plan = match entry {
                0x127132 => scenario::get_word(&e.plan.state.memory, frame, limits, e.services)?,
                0x133f3e => {
                    scenario::display_mission_name(&e.plan.state.memory, frame, limits, e.services)?
                }
                0x13b212 => {
                    scenario::update_map_menu(&e.plan.state.memory, frame, limits, e.services)?
                }
                0x135bb2 => {
                    scenario::update_monster_menu(&e.plan.state.memory, frame, limits, e.services)?
                }
                _ => scenario::fast_plot_map(&e.plan.state.memory, frame, limits, e.services)?,
            };
            e.adopt(
                plan.memory,
                e.plan.state.random.clone(),
                if plan.flow == LoadFlow::Returned {
                    plan.native_return
                        .map_or(PromptReturn::Unchanged, PromptReturn::Full)
                } else {
                    PromptReturn::NoReturn
                },
                plan.flow,
                plan.effects.into_iter().map(Effect::ScenarioMenu),
            )
        }
        0x11edc4 | 0x15a1f6 => {
            use crate::recovered_combat_ui as combat;
            let plan = if entry == 0x11edc4 {
                combat::combatupdate(
                    &e.plan.state.memory,
                    frame,
                    e.sw(frame.wrapping_add(8))?,
                    &*e.services,
                )?
            } else {
                combat::showtargets(&e.plan.state.memory, &*e.services)?
            };
            e.adopt(
                plan.memory,
                e.plan.state.random.clone(),
                plan.native_return
                    .map_or(PromptReturn::Unchanged, PromptReturn::Full),
                LoadFlow::Returned,
                plan.effects.into_iter().map(Effect::CombatUi),
            )
        }
        0x1580be | 0x10e6fe | 0x10ec52 => {
            use crate::recovered_combat_center as center;
            let plan = match entry {
                0x1580be => {
                    center::showrange(&e.plan.state.memory, frame, e.remaining, &*e.services)?
                }
                0x10e6fe => {
                    center::centerfieldlarge(&e.plan.state.memory, frame, e.remaining, e.services)?
                }
                _ => {
                    center::centerfieldsmall(&e.plan.state.memory, frame, e.remaining, e.services)?
                }
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
        0x110644 | 0x11f1a8 => {
            let damage =
                crate::recovered_inventory_selection::InventorySelectionServices::damage_frame(
                    e.services,
                    &e.plan.state.memory,
                )?;
            if entry == 0x110644 {
                let plan = crate::recovered_combat_center::centerstage(
                    &e.plan.state.memory,
                    frame,
                    damage,
                    e.remaining,
                    e.services,
                )?;
                e.adopt(
                    plan.memory,
                    e.plan.state.random.clone(),
                    plan.native_return
                        .map_or(PromptReturn::Unchanged, PromptReturn::Full),
                    LoadFlow::Returned,
                    plan.effects.into_iter().map(Effect::CombatCenter),
                )
            } else {
                let plan = crate::recovered_combat_panel::combat_update2(
                    &e.plan.state.memory,
                    frame,
                    e.sw(frame.wrapping_add(8))?,
                    damage,
                    e.services,
                )?;
                e.adopt(
                    plan.memory,
                    e.plan.state.random.clone(),
                    PromptReturn::Full(plan.native_return as u32),
                    LoadFlow::Returned,
                    plan.effects.into_iter().map(Effect::CombatPanel),
                )
            }
        }
        0x117a54 | 0x1551a2 => {
            use crate::recovered_status_panels as status;
            let plan = if entry == 0x117a54 {
                let damage =
                    crate::recovered_inventory_selection::InventorySelectionServices::damage_frame(
                        e.services,
                        &e.plan.state.memory,
                    )?;
                status::combat_info(
                    &e.plan.state.memory,
                    &e.plan.state.random,
                    frame,
                    prompt_limits.maximum_polls,
                    damage,
                    e.services,
                )?
            } else {
                status::show_condition2(
                    &e.plan.state.memory,
                    &e.plan.state.random,
                    frame,
                    e.remaining as usize,
                    &*e.services,
                )?
            };
            let returned = if let Some(value) = plan.native_return.known_long() {
                PromptReturn::Full(value)
            } else {
                plan.native_return
                    .low_word
                    .map_or(PromptReturn::Unchanged, PromptReturn::LowWord)
            };
            e.adopt(
                plan.memory,
                plan.random,
                returned,
                LoadFlow::Returned,
                plan.effects.into_iter().map(|effect| {
                    Effect::Interaction(interaction::InventoryInteractionEffect::Status(effect))
                }),
            )
        }
        0x11ce8e => {
            let state = crate::recovered_world_display::WorldDisplayState {
                memory: e.plan.state.memory.clone(),
                random: e.plan.state.random.clone(),
            };
            let plan = crate::recovered_world_display::combatmap(&state, frame, e.remaining)?;
            e.adopt(
                plan.state.memory,
                plan.state.random,
                PromptReturn::Full(plan.native_return as u32),
                LoadFlow::Returned,
                plan.effects.into_iter().map(|effect| {
                    Effect::CombatCenter(crate::recovered_combat_center::CombatCenterEffect::World(
                        effect,
                    ))
                }),
            )
        }
        0x120c14 | 0x15f210 => {
            let plan = if entry == 0x120c14 {
                crate::recovered_rendering::drawbody(
                    &e.plan.state.memory,
                    frame,
                    e.sw(frame.wrapping_add(8))?,
                    e.sw(frame.wrapping_add(10))?,
                    e.sw(frame.wrapping_add(12))?,
                    &*e.services,
                )?
            } else {
                crate::recovered_rendering::updatearrow(
                    &e.plan.state.memory,
                    frame,
                    e.sw(frame.wrapping_add(8))?,
                    e.registers.address[2].long("A2")?,
                    &*e.services,
                )?
            };
            e.adopt(
                plan.memory,
                e.plan.state.random.clone(),
                PromptReturn::Unchanged,
                LoadFlow::Returned,
                plan.effects.into_iter().map(Effect::Rendering),
            )
        }
        0x1212a8 => {
            let plan = crate::recovered_equipment::forcedropitem(
                &e.plan.state.memory,
                &e.plan.state.random,
                frame,
                e.limits.equipment,
                e.services,
            )?;
            let returned = plan.native_return.known_long().map_or(
                PromptReturn::LowWord(plan.native_return.low_word),
                PromptReturn::Full,
            );
            e.adopt(
                plan.memory,
                plan.random,
                returned,
                LoadFlow::Returned,
                plan.effects.into_iter().map(Effect::Equipment),
            )
        }
        0x12946c | 0x131564 => {
            let plan = if entry == 0x12946c {
                crate::recovered_interface::inwindow(
                    e.b(frame.wrapping_add(9))? as i8,
                    e.sw(frame.wrapping_add(10))?,
                    e.services,
                )?
            } else {
                crate::recovered_interface::centercursor(e.services)?
            };
            e.full(plan.native_return as u32);
            e.plan
                .effects
                .extend(plan.effects.into_iter().map(Effect::Interface));
            Ok(())
        }
        0x136cfa | 0x136d86 | 0x13a7c4 | 0x14c6c2 => {
            let limits = crate::recovered_field_control::FieldControlLimits {
                maximum_steps: e.remaining,
                maximum_polls: prompt_limits.maximum_polls,
                filename_capacity: prompt_limits.filename_capacity,
                maximum_random_draws: e.draws,
                equipment: e.limits.equipment,
            };
            let plan = crate::recovered_field_control::existing_child_plan(
                &e.plan.state,
                entry,
                frame,
                limits,
                e.services,
            )?;
            e.adopt(
                plan.state.memory,
                plan.state.random,
                plan.native_return,
                plan.flow,
                plan.effects.into_iter().map(Effect::Field),
            )
        }
        0x130d38 | 0x130e54 => {
            use crate::recovered_records as records;
            let id = e.sw(frame.wrapping_add(8))?;
            let target = if entry == 0x130d38 {
                0x453768
            } else {
                0x4536d2
            };
            let current = e
                .bytes(target, centurions_content::ITEM_BYTES)?
                .try_into()
                .expect("item record size");
            let backing = crate::recovered_equipment::item_tables(&e.plan.state.memory, id)?;
            let tables = records::ItemTables {
                families: backing.each_ref().map(Vec::as_slice),
            };
            let plan = if entry == 0x130d38 {
                records::loadweapon(&tables, &current, id)?
            } else {
                records::loadammo(&tables, &current, id)?
            };
            for write in plan.writes {
                e.step()?;
                e.write(write.address, &write.bytes)?;
            }
            // RecordLoad's logical void result is not an observed native D0.
            e.unknown();
            Ok(())
        }
        0x1314fc => {
            let coordinates = [
                e.b(frame.wrapping_add(9))? as i8,
                e.b(frame.wrapping_add(11))? as i8,
                e.b(frame.wrapping_add(13))? as i8,
            ];
            let offset = usize::from(i16::from(coordinates[0]) as u16) * 2842
                + usize::from(i16::from(coordinates[1]) as u16) * 406
                + usize::from(i16::from(coordinates[2]) as u16)
                    * crate::recovered_records::SPELL_BYTES;
            if offset > 1024 * 1024 {
                return Err(
                    RuleError::Invalid("application spell view exceeds one MiB".into()).into(),
                );
            }
            // Padding satisfies the typed table view and is never published.
            let mut table = vec![0; offset + crate::recovered_records::SPELL_BYTES];
            table[offset..].copy_from_slice(&e.bytes(
                0x443788u32.wrapping_add(offset as u32),
                crate::recovered_records::SPELL_BYTES,
            )?);
            let plan = crate::recovered_records::loadspell(
                &table,
                coordinates[0],
                coordinates[1],
                coordinates[2],
            )?;
            for write in plan.writes {
                e.step()?;
                e.write(write.address, &write.bytes)?;
            }
            e.unknown();
            Ok(())
        }
        0x13beea => {
            let plan = crate::recovered_inventory_rules::cancast(
                &e.plan.state.memory,
                frame,
                e.registers.data[0].long("D0")?,
                prompt_limits.maximum_polls,
                &*e.services,
            )?;
            e.adopt(
                plan.memory,
                e.plan.state.random.clone(),
                PromptReturn::Full(plan.native_return),
                LoadFlow::Returned,
                plan.effects.into_iter().map(|effect| {
                    Effect::Interaction(interaction::InventoryInteractionEffect::Restriction(
                        effect,
                    ))
                }),
            )
        }
        0x1046e0 | 0x10b6ba | 0x159192 | 0x15947c | 0x162084 => {
            let limits = crate::recovered_selection_control::SelectionControlLimits {
                prompt: prompt_limits,
                maximum_random_draws: e.draws,
            };
            let plan = crate::recovered_selection_control::existing_child_plan(
                &e.plan.state,
                entry,
                limits,
                e.services,
            )?;
            e.adopt(
                plan.state.memory,
                plan.state.random,
                plan.native_return,
                plan.flow,
                plan.effects.into_iter().map(Effect::Survivor),
            )
        }
        value if interaction::has_existing_child(value) => {
            let registers = ItemsRegisters {
                data: e.registers.data.map(|value| value.value),
                known_data_bits: e.registers.data.map(|value| value.known_mask),
                address: e
                    .registers
                    .address
                    .map(|value| (value.known_mask == u32::MAX).then_some(value.value)),
                condition_codes: e.registers.ccr,
                known_condition_codes: e.registers.ccr_known_mask,
            };
            let limits = super::ApplicationControlLimits {
                maximum_steps: e.remaining,
                maximum_random_draws: e.draws,
                prompt: prompt_limits,
                ..e.limits
            };
            let (plan, observed) = interaction::existing_child_plan(
                &e.plan.state,
                entry,
                frame,
                &registers,
                limits,
                e.services,
            )?;
            e.adopt(
                plan.state.memory,
                plan.state.random,
                plan.native_return,
                plan.flow,
                plan.effects.into_iter().map(Effect::Interaction),
            )?;
            e.registers.data = std::array::from_fn(|n| super::RegisterBits {
                value: observed.data[n],
                known_mask: observed.known_data_bits[n],
            });
            for n in 0..7 {
                e.registers.address[n] = observed.address[n]
                    .map_or(super::RegisterBits::default(), super::RegisterBits::full);
            }
            e.registers.ccr = observed.condition_codes;
            e.registers.ccr_known_mask = observed.known_condition_codes;
            Ok(())
        }
        _ => Err(RuleError::Unsupported {
            action: "application_control.existing_child",
            dependency: "the reached source entry has no concrete adapter in this dispatcher",
        }
        .into()),
    }
}
