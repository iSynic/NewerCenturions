pub use centurions_core::*;
pub use centurions_translated_application::recovered_application_children;
pub use centurions_translated_application::recovered_core_control;
pub use centurions_translated_application::recovered_items_control;
pub use centurions_translated_application::recovered_main_control;
pub use centurions_translated_application::recovered_spell_control;
pub use centurions_translated_gameplay::recovered_gameplay_control;
pub use centurions_translated_gameplay::recovered_gameplay_leaves;
pub use centurions_translated_system::recovered_remaining_control;
pub use centurions_translated_system::recovered_system_control;
mod children;
mod gameplay_children;
mod spell_children;

use centurions_core::recovered_runtime::{self as runtime, RuntimeResult};
use recovered_application_control::{
    ApplicationControlEffect, ApplicationControlLimits, ApplicationControlServices, Execution,
    RegisterBits, Result, RoutineResolver,
};
pub struct CompleteRoutines;
impl RoutineResolver for CompleteRoutines {
    fn dispatch(&self, execution: &mut Execution<'_>, entry: u32, frame: u32) -> Result<()> {
        dispatch(execution, entry, frame)
    }
    fn owns_native_stack(&self, entry: u32) -> bool {
        owns_native_stack(entry)
    }
}
pub fn execute(
    state: &recovered_application_control::ApplicationControlState,
    entry: u32,
    frame: u32,
    registers: recovered_application_control::ApplicationRegisters,
    limits: recovered_application_control::ApplicationControlLimits,
    services: &mut dyn ApplicationControlServices,
) -> std::result::Result<recovered_application_control::ApplicationControlPlan, RuleError> {
    recovered_application_control::execute(
        state,
        entry,
        frame,
        registers,
        limits,
        services,
        &CompleteRoutines,
    )
}

fn dispatch(e: &mut Execution<'_>, entry: u32, frame: u32) -> Result<()> {
    match entry {
        0x100212 => crate::recovered_remaining_control::relocate_body(e, frame),
        0x100286 => crate::recovered_remaining_control::relocate_three_body(e, frame),
        0x1002b2 => crate::recovered_remaining_control::unrelocate_three_body(e, frame),
        0x1002ec => crate::recovered_remaining_control::runtime_transfer_body(e, frame),
        0x1004ae => crate::recovered_remaining_control::run_destructors_body(e, frame),
        0x1004d2 => crate::recovered_remaining_control::multiply_long_body(e, frame),
        0x100504 => crate::recovered_remaining_control::divide_unsigned_long_body(e, frame),
        0x100584 => crate::recovered_remaining_control::remainder_unsigned_long_body(e, frame),
        0x100a9a => crate::recovered_remaining_control::addtobooty_body(e, frame),
        0x10518a => crate::recovered_remaining_control::monsterupdate_body(e, frame),
        0x1056d4 => crate::recovered_remaining_control::updatebootymoney_body(e, frame),
        0x10b13a => crate::recovered_remaining_control::cansee2_body(e, frame),
        0x10b3fa => crate::recovered_remaining_control::cansee3_body(e, frame),
        0x10b814 => crate::recovered_remaining_control::updatetier_body(e, frame),
        0x10d3f6 => crate::recovered_remaining_control::calcbodyrect_body(e, frame),
        0x10f1c4 => crate::recovered_remaining_control::centerpictlarge_body(e, frame),
        0x10fc26 => crate::recovered_remaining_control::centerpictsmall_body(e, frame),
        0x11084a => crate::recovered_remaining_control::change_body(e, frame),
        0x112208 => crate::recovered_remaining_control::checklayout_body(e, frame),
        0x113988 => crate::recovered_remaining_control::caste_body(e, frame),
        0x120872 => crate::recovered_remaining_control::display_body(e, frame),
        0x123b56 => crate::recovered_remaining_control::flashrange_body(e, frame),
        0x123f96 => crate::recovered_remaining_control::flashrect_body(e, frame),
        0x125512 => crate::recovered_remaining_control::spell_count_body(e, frame),
        0x127264 => crate::recovered_remaining_control::gender_body(e, frame),
        0x12c874 => crate::recovered_remaining_control::installitem_complete_body(e, frame),
        0x12d558 => crate::recovered_remaining_control::randomitem_body(e, frame),
        0x12dc58 => crate::recovered_remaining_control::show_pack_body(e, frame),
        0x12ed42 => crate::recovered_remaining_control::level_up_body(e, frame),
        0x130c18 => crate::recovered_remaining_control::loaditemcompare_body(e, frame),
        0x130f6e => crate::recovered_remaining_control::loaditemnotype_body(e, frame),
        0x133886 => crate::recovered_remaining_control::update_shop_wings_body(e, frame),
        0x134ae0 => crate::recovered_remaining_control::update_edit_item_body(e, frame),
        0x13684a => crate::recovered_remaining_control::share_credits_body(e, frame),
        0x136f90 => crate::recovered_remaining_control::calcdamage_body(e, frame),
        0x1380ac => crate::recovered_remaining_control::getexp_body(e, frame),
        0x1382fc => crate::recovered_remaining_control::string_long_body(e, frame),
        0x13832a => crate::recovered_remaining_control::string_plus_body(e, frame),
        0x138368 => crate::recovered_remaining_control::string_no_zero_no_plus_body(e, frame),
        0x1383a6 => crate::recovered_remaining_control::string_no_zero_body(e, frame),
        0x1384d6 => crate::recovered_remaining_control::update_preparation_body(e, frame),
        0x138648 => crate::recovered_remaining_control::strength_adjustments_body(e, frame),
        0x1388aa => crate::recovered_remaining_control::agility_adjustment_body(e, frame),
        0x1389e4 => crate::recovered_remaining_control::get_dialog_number_body(e, frame),
        0x139244 => crate::recovered_remaining_control::update_shop_body(e, frame),
        0x139faa => crate::recovered_remaining_control::shop_credit_display_body(e, frame),
        0x13a068 => crate::recovered_remaining_control::eraseshopstats_body(e, frame),
        0x13a10a => crate::recovered_remaining_control::eraseshopname_body(e, frame),
        0x13a72a => crate::recovered_remaining_control::dialog_number_nonzero_body(e, frame),
        0x13aeb8 => crate::recovered_remaining_control::setverify_body(e, frame),
        0x13b0ea => crate::recovered_remaining_control::checkverify_body(e, frame),
        0x13b30c => crate::recovered_remaining_control::start_level_body(e, frame),
        0x13bac6 => crate::recovered_remaining_control::open_caste_body(e, frame),
        0x13c018 => crate::recovered_remaining_control::itemcost_body(e, frame),
        0x13f52c => crate::recovered_remaining_control::add_initial_items_body(e, frame),
        0x13f684 => crate::recovered_remaining_control::new_character_body(e, frame),
        0x14695e => crate::recovered_remaining_control::number_body(e, frame),
        0x146a50 => crate::recovered_remaining_control::copy_part_of_string_body(e, frame),
        0x146aa2 => crate::recovered_remaining_control::print_stat_body(e, frame),
        0x148f9e => crate::recovered_remaining_control::view_restrictions_body(e, frame),
        0x149fc6 => crate::recovered_remaining_control::ploticon2_body(e, frame),
        0x14a074 => crate::recovered_remaining_control::ploticon_body(e, frame),
        0x14a29e => crate::recovered_remaining_control::update_port_body(e, frame),
        0x14a918 => crate::recovered_remaining_control::plus_body(e, frame),
        0x14aa22 => crate::recovered_remaining_control::minus_body(e, frame),
        0x14d7cc => crate::recovered_remaining_control::race_body(e, frame),
        0x15498e => crate::recovered_remaining_control::show_charge_body(e, frame),
        0x1553bc => crate::recovered_remaining_control::showdamage_body(e, frame),
        0x15741a => crate::recovered_remaining_control::show_special_body(e, frame),
        0x157758 => crate::recovered_remaining_control::show_item_stats_body(e, frame),
        0x157ef8 => crate::recovered_remaining_control::showque_body(e, frame),
        0x159a96 => crate::recovered_remaining_control::show_stats_body(e, frame),
        0x15a49a => crate::recovered_remaining_control::show_upgrade_body(e, frame),
        0x161068 => crate::recovered_remaining_control::update_odds_body(e, frame),
        0x161d86 => crate::recovered_remaining_control::update_specialties_body(e, frame),
        0x162238 => crate::recovered_remaining_control::upgrade_warrior_body(e, frame),
        0x1634f8 => crate::recovered_remaining_control::apply_upgrade_body(e, frame),
        0x1636ec => crate::recovered_remaining_control::calculate_statistics_body(e, frame),
        0x1213f6 => crate::recovered_application_children::set_ind_string_body(e, frame),
        0x122b18 => crate::recovered_application_children::allocate_building_body(e, frame),
        0x122dc4 => crate::recovered_application_children::gcopy_body(e, frame),
        0x1241b4 => crate::recovered_application_children::is_color_gamma_available_body(e, frame),
        0x124212 => crate::recovered_application_children::get_monitor_state_body(e, frame),
        0x124242 => crate::recovered_application_children::release_monitor_state_body(e, frame),
        0x12427c => crate::recovered_application_children::capture_monitor_list_body(e, frame),
        0x124308 => crate::recovered_application_children::capture_monitor_gamma_body(e, frame),
        0x124576 => crate::recovered_application_children::restore_gamma_list_body(e, frame),
        0x1245cc => crate::recovered_application_children::dispose_gamma_list_body(e, frame),
        0x124632 => crate::recovered_application_children::fade_to_monitor_state_body(e, frame),
        0x124666 => crate::recovered_application_children::fade_to_constant_color_body(e, frame),
        0x1246a8 => crate::recovered_application_children::run_fade_sequence_body(e, frame),
        0x1247cc => crate::recovered_application_children::x_calc_fade_to_color_body(e, frame),
        0x12485e => crate::recovered_application_children::fade_gamma_list_body(e, frame),
        0x1248f0 => crate::recovered_application_children::calc_fade_to_color_body(e, frame),
        0x124a44 => crate::recovered_application_children::x_calc_fade_to_gamma_body(e, frame),
        0x124bba => crate::recovered_application_children::get_device_gamma_table_body(e, frame),
        0x124c18 => crate::recovered_application_children::set_device_gamma_table_body(e, frame),
        0x124ca2 => {
            crate::recovered_application_children::color_gamma_trap_available_body(e, frame)
        }
        0x124d04 => crate::recovered_application_children::color_gamma_memset_body(e, frame),
        0x1270c0 => crate::recovered_application_children::get_version_string_body(e, frame),
        0x127462 => crate::recovered_application_children::gworld_init_body(e, frame),
        0x1315a2 => crate::recovered_application_children::fade_in_out_body(e, frame),
        0x1366a8 => crate::recovered_application_children::pool_body(e, frame),
        0x136e04 => crate::recovered_application_children::get_save_filename_body(e, frame),
        0x13777a => crate::recovered_application_children::set_file_info_body(e, frame),
        0x13ba6c => crate::recovered_application_children::open_race_body(e, frame),
        0x13ef12 => crate::recovered_application_children::name_body(e, frame),
        0x14aba0 => crate::recovered_application_children::portrait_body(e, frame),
        0x14bca4 => crate::recovered_application_children::load_preferences_body(e, frame),
        0x14bfe8 => crate::recovered_application_children::application_name_body(e, frame),
        0x14c01c => crate::recovered_application_children::save_preferences_body(e, frame),
        0x14c318 => crate::recovered_application_children::check_for_mismatch_body(e, frame),
        0x14c440 => crate::recovered_application_children::create_preferences_body(e, frame),
        0x14c622 => crate::recovered_application_children::open_preferences_body(e, frame),
        0x14ee9e => crate::recovered_application_children::hide_menu_bar_body(e, frame),
        0x14ef36 => crate::recovered_application_children::show_menu_bar_body(e, frame),
        0x15311a => crate::recovered_application_children::set_free_body(e, frame),
        0x1531c0 => crate::recovered_application_children::set_free2_body(e, frame),
        0x153584 => crate::recovered_application_children::set_speaker_volume_body(e, frame),
        0x153cf4 => crate::recovered_application_children::dialog_number_long_body(e, frame),
        0x153d56 => crate::recovered_application_children::dialog_number_body(e, frame),
        0x153daa => crate::recovered_application_children::dialog_string_body(e, frame),
        0x157d38 => crate::recovered_application_children::show_logo_body(e, frame),
        0x157ea4 => crate::recovered_application_children::delay_interruptible_body(e, frame),
        0x105366 => crate::recovered_gameplay_leaves::restore_body_body(e, frame),
        0x105572 => crate::recovered_gameplay_leaves::bodyground_body(e, frame),
        0x10ae90 => crate::recovered_gameplay_leaves::cansee_body(e, frame),
        0x110d12 => crate::recovered_gameplay_leaves::checkformechanicalunits_body(e, frame),
        0x11235e => crate::recovered_gameplay_leaves::checkrange_body(e, frame),
        0x11279a => crate::recovered_gameplay_leaves::item_charges_body(e, frame),
        0x12063c => crate::recovered_gameplay_leaves::destroymonster_body(e, frame),
        0x125538 => crate::recovered_gameplay_leaves::getrange_body(e, frame),
        0x125960 => crate::recovered_gameplay_leaves::getclosest_body(e, frame),
        0x133bb8 => crate::recovered_gameplay_leaves::start_sound_cycle_body(e, frame),
        0x133c56 => crate::recovered_gameplay_leaves::sustain_sound_cycle_body(e, frame),
        0x136398 => crate::recovered_gameplay_leaves::nobrassnoammo_body(e, frame),
        0x1363e4 => crate::recovered_gameplay_leaves::best_choice_body(e, frame),
        0x1377f4 => crate::recovered_gameplay_leaves::drawspell_body(e, frame),
        0x1382d0 => crate::recovered_gameplay_leaves::string_body(e, frame),
        0x138a40 => crate::recovered_gameplay_leaves::xy_body(e, frame),
        0x13a3ae => crate::recovered_gameplay_leaves::save_against_body(e, frame),
        0x13f65e => crate::recovered_gameplay_leaves::pin_body(e, frame),
        0x149e0e => crate::recovered_gameplay_leaves::placemonster_body(e, frame),
        0x151f30 => crate::recovered_gameplay_leaves::direction_body(e, frame),
        0x153de6 => crate::recovered_gameplay_leaves::movement_cost_body(e, frame),
        0x153e44 => crate::recovered_gameplay_leaves::shift_body(e, frame),
        0x157c68 => crate::recovered_gameplay_leaves::save_against_attribute_body(e, frame),
        0x15b84c => crate::recovered_gameplay_leaves::spelleffectoutdoors_body(e, frame),
        0x15d210 => crate::recovered_gameplay_leaves::takegold_body(e, frame),
        0x15edfa => crate::recovered_gameplay_leaves::finddxdy_body(e, frame),
        0x15ef50 => crate::recovered_gameplay_leaves::to_weapon_body(e, frame),
        0x15efee => crate::recovered_gameplay_leaves::to_missile_body(e, frame),
        0x15f048 => crate::recovered_gameplay_leaves::missileset_body(e, frame),
        0x160fb2 => crate::recovered_gameplay_leaves::updatelight_body(e, frame),
        0x1124dc => crate::recovered_gameplay_leaves::getrangetobody_body(e, frame),
        0x122bec => crate::recovered_gameplay_leaves::fastplot_body(e, frame),
        0x100af0 => crate::recovered_gameplay_control::attack_body(e, frame),
        0x101eb6 => crate::recovered_gameplay_control::attack_secondary_body(e, frame),
        0x10950e => crate::recovered_gameplay_control::button_choice_body(e, frame),
        0x10d4fe => crate::recovered_gameplay_control::cast_body(e, frame),
        0x110e96 => crate::recovered_gameplay_control::check_for_enemy_body(e, frame),
        0x1140ae => crate::recovered_gameplay_control::collide_body(e, frame),
        0x114378 => crate::recovered_gameplay_control::combat_body(e, frame),
        0x117be2 => crate::recovered_gameplay_control::combat_choice_body(e, frame),
        0x12185a => crate::recovered_gameplay_control::encounter_body(e, frame),
        0x13e276 => crate::recovered_gameplay_control::move_monster_body(e, frame),
        0x1400e4 => crate::recovered_gameplay_control::new_land_body(e, frame),
        0x15e148 => crate::recovered_gameplay_control::advance_time_body(e, frame),
        0x1469f8 => {
            e.full(crate::recovered_verification::doreg3() as u32);
            Ok(())
        }
        0x153ca4 | 0x13849e => {
            let (minimum, bound) = if entry == 0x153ca4 {
                (0, e.sw(frame.wrapping_add(8))?)
            } else {
                let minimum = e.sw(frame.wrapping_add(8))?;
                let maximum = e.sw(frame.wrapping_add(10))?;
                (minimum, maximum.wrapping_sub(minimum).wrapping_add(1))
            };
            e.draws = e
                .draws
                .checked_sub(1)
                .ok_or_else(|| RuleError::Invalid("application random budget exhausted".into()))?;
            let mut context =
                runtime::RuntimeContext::new(e.plan.state.random.clone(), e.remaining as usize);
            let plan = runtime::rand(&mut context, bound)?;
            e.plan.state.random = context.random;
            e.plan.effects.extend(
                plan.effects
                    .into_iter()
                    .map(ApplicationControlEffect::Runtime),
            );
            if let RuntimeResult::Word { value, native_d0 } = plan.result {
                e.full(native_d0);
                if entry == 0x13849e {
                    e.low(minimum.wrapping_add(value).wrapping_sub(1) as u16);
                }
                Ok(())
            } else {
                Err(RuleError::Invalid("unexpected source Rand return".into()).into())
            }
        }
        0x11177e => crate::recovered_system_control::check_for_secret_body(e, frame),
        0x12f0cc => crate::recovered_system_control::load_body(e, frame),
        0x13164e => crate::recovered_system_control::toolbox_init_body(e, frame),
        0x132780 => crate::recovered_system_control::main_body(e, frame),
        0x13283e => crate::recovered_system_control::main_loop_body(e, frame),
        0x132886 => crate::recovered_system_control::handle_event_body(e, frame),
        0x134e82 => crate::recovered_system_control::show_map_body(e, frame),
        0x135354 => crate::recovered_system_control::menu_init_body(e, frame),
        0x135fc8 => crate::recovered_system_control::add_delete_scenario_body(e, frame),
        0x1366f8 => crate::recovered_system_control::select_scenario_body(e, frame),
        0x13bd52 => crate::recovered_system_control::modify_body(e, frame),
        0x14913e => crate::recovered_system_control::party_loss_body(e, frame),
        0x154016 => crate::recovered_system_control::update_scenario_availability_body(e, frame),
        0x1540c4 => crate::recovered_system_control::about_body(e, frame),
        0x15d2f0 => crate::recovered_system_control::temple_body(e, frame),
        0x129516 => {
            use crate::recovered_inventory_interaction::{
                InventoryInteractionEffect, ItemsRegisters,
            };
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
            let limits = ApplicationControlLimits {
                maximum_steps: e.remaining,
                maximum_random_draws: e.draws,
                maximum_call_depth: e.limits.maximum_call_depth.saturating_sub(e.depth),
                ..e.limits
            };
            let child = recovered_items_control::items_body_plan(
                &e.plan.state,
                frame,
                &registers,
                limits,
                e.services,
                e.resolver,
            )?;
            let snapshot = child.effects.iter().rev().find_map(|effect| match effect {
                InventoryInteractionEffect::ItemsRegisters(snapshot) => Some(snapshot.clone()),
                _ => None,
            });
            e.adopt(
                child.state.memory,
                child.state.random,
                child.native_return,
                child.flow,
                child
                    .effects
                    .into_iter()
                    .map(ApplicationControlEffect::Interaction),
            )?;
            if let Some(snapshot) = snapshot {
                e.registers.data = std::array::from_fn(|n| RegisterBits {
                    value: snapshot.data[n],
                    known_mask: snapshot.known_data_bits[n],
                });
                e.registers.address = snapshot
                    .address
                    .map(|value| value.map_or(RegisterBits::default(), RegisterBits::full));
                e.registers.ccr = snapshot.condition_codes;
                e.registers.ccr_known_mask = snapshot.known_condition_codes;
            }
            Ok(())
        }
        0x12e4e4 => crate::recovered_core_control::kill_body_body(e, frame),
        0x121524 => crate::recovered_core_control::empty_queue_body(e, frame),
        0x109210 => crate::recovered_core_control::build_monster_body(e, frame),
        0x125a3a => crate::recovered_combat_setup::get_up_body(e, frame),
        0x11d406 => crate::recovered_combat_setup::combat_setup_body(e, frame),
        0x151022 => crate::recovered_spell_control::resolve_spell_body(e, frame),
        0x15b9d8 => crate::recovered_spell_control::spell_list_body(e, frame),
        0x15cb0e => crate::recovered_spell_control::spell_targets_body(e, frame),
        0x1287c2 => crate::recovered_spell_control::heal_body(e, frame),
        0x14de04 => crate::recovered_spell_control::reduce_body(e, frame),
        0x13b624 => crate::recovered_main_control::main_screen_body(e, frame),
        0x13bb20 => crate::recovered_main_control::update_main_body(e, frame),
        0x1279bc => crate::recovered_main_control::handle_menu_choice_body(e, frame),
        0x149452 => crate::recovered_main_control::main_screen_init_body(e, frame),
        0x14b63a => crate::recovered_main_control::preference_body(e, frame),
        0x112650 | 0x160a8e | 0x13822a | 0x150114 | 0x15075a | 0x15b2f6 | 0x158454 | 0x1600fc
        | 0x15f66c | 0x10e6b6 | 0x10f186 | 0x1107cc | 0x16043e | 0x1500d8 | 0x133a18 | 0x16474a
        | 0x123d1c | 0x13a498 | 0x13154c => spell_children::dispatch(e, entry, frame),
        _ => children::dispatch(e, entry, frame),
    }
}

pub fn owns_native_stack(entry: u32) -> bool {
    entry == 0x129516
        || crate::recovered_system_control::owns_frame(entry)
        || crate::recovered_gameplay_control::owns_frame(entry)
        || crate::recovered_gameplay_leaves::owns_frame(entry)
        || crate::recovered_application_children::owns_frame(entry)
        || crate::recovered_remaining_control::owns_frame(entry)
}
