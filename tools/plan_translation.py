"""Index recovered action blocks, Rust ownership, and dependency-ordered batches.

Inputs are private Ghidra exports and the existing scoped action registry.
Generated Rust contains names, addresses and counts, never executable bytes or
decompiled source. Recovery and translation are independent of native comparison.
"""
from __future__ import annotations

import argparse
from collections import Counter
import json
import re
from pathlib import Path


SOURCE_PORTS = {
    0x100AF0: ("recovered_gameplay_control.attack", "crates/core/src/recovered_gameplay_control/attack.rs"),
    0x101EB6: ("recovered_gameplay_control.attack_secondary", "crates/core/src/recovered_gameplay_control/attack_secondary.rs"),
    0x10950E: ("recovered_gameplay_control.button_choice", "crates/core/src/recovered_gameplay_control/button_choice.rs"),
    0x10D4FE: ("recovered_gameplay_control.cast", "crates/core/src/recovered_gameplay_control/cast.rs"),
    0x110E96: ("recovered_gameplay_control.check_for_enemy", "crates/core/src/recovered_gameplay_control/check_for_enemy.rs"),
    0x1140AE: ("recovered_gameplay_control.collide", "crates/core/src/recovered_gameplay_control/collide.rs"),
    0x114378: ("recovered_gameplay_control.combat", "crates/core/src/recovered_gameplay_control/combat.rs"),
    0x117BE2: ("recovered_gameplay_control.combat_choice", "crates/core/src/recovered_gameplay_control/combat_choice.rs"),
    0x12185A: ("recovered_gameplay_control.encounter", "crates/core/src/recovered_gameplay_control/encounter.rs"),
    0x13E276: ("recovered_gameplay_control.move_monster", "crates/core/src/recovered_gameplay_control/move_monster.rs"),
    0x1400E4: ("recovered_gameplay_control.new_land", "crates/core/src/recovered_gameplay_control/new_land.rs"),
    0x15E148: ("recovered_gameplay_control.advance_time", "crates/core/src/recovered_gameplay_control/advance_time.rs"),
    0x11177E: ("recovered_system_control.check_for_secret", "crates/core/src/recovered_system_control.rs"),
    0x12F0CC: ("recovered_system_control.load", "crates/core/src/recovered_system_control.rs"),
    0x13164E: ("recovered_system_control.toolbox_init", "crates/core/src/recovered_system_control.rs"),
    0x132780: ("recovered_system_control.main", "crates/core/src/recovered_system_control.rs"),
    0x13283E: ("recovered_system_control.main_loop", "crates/core/src/recovered_system_control.rs"),
    0x132886: ("recovered_system_control.handle_event", "crates/core/src/recovered_system_control.rs"),
    0x134E82: ("recovered_system_control.show_map", "crates/core/src/recovered_system_control.rs"),
    0x135354: ("recovered_system_control.menu_init", "crates/core/src/recovered_system_control.rs"),
    0x135FC8: ("recovered_system_control.add_delete_scenario", "crates/core/src/recovered_system_control.rs"),
    0x1366F8: ("recovered_system_control.select_scenario", "crates/core/src/recovered_system_control.rs"),
    0x13BD52: ("recovered_system_control.modify", "crates/core/src/recovered_system_control.rs"),
    0x14913E: ("recovered_system_control.party_loss", "crates/core/src/recovered_system_control.rs"),
    0x154016: ("recovered_system_control.update_scenario_availability", "crates/core/src/recovered_system_control.rs"),
    0x1540C4: ("recovered_system_control.about", "crates/core/src/recovered_system_control.rs"),
    0x15D2F0: ("recovered_system_control.temple", "crates/core/src/recovered_system_control.rs"),
    0x129516: ("recovered_inventory_interaction.items", "crates/core/src/recovered_inventory_interaction.rs"),
    0x12E4E4: ("recovered_core_control.kill_body", "crates/core/src/recovered_core_control.rs"),
    0x121524: ("recovered_core_control.empty_queue", "crates/core/src/recovered_core_control.rs"),
    0x109210: ("recovered_core_control.build_monster", "crates/core/src/recovered_core_control.rs"),
    0x125A3A: ("recovered_combat_setup.get_up", "crates/core/src/recovered_combat_setup.rs"),
    0x11D406: ("recovered_combat_setup.combat_setup", "crates/core/src/recovered_combat_setup.rs"),
    0x151022: ("recovered_spell_control.resolve_spell", "crates/core/src/recovered_spell_control.rs"),
    0x15B9D8: ("recovered_spell_control.spell_list", "crates/core/src/recovered_spell_control.rs"),
    0x15CB0E: ("recovered_spell_control.spell_targets", "crates/core/src/recovered_spell_control.rs"),
    0x1287C2: ("recovered_spell_control.heal", "crates/core/src/recovered_spell_control.rs"),
    0x14DE04: ("recovered_spell_control.reduce", "crates/core/src/recovered_spell_control.rs"),
    0x13B624: ("recovered_main_control.main_screen", "crates/core/src/recovered_main_control.rs"),
    0x13BB20: ("recovered_main_control.update_main", "crates/core/src/recovered_main_control.rs"),
    0x1279BC: ("recovered_main_control.handle_menu_choice", "crates/core/src/recovered_main_control.rs"),
    0x149452: ("recovered_main_control.main_screen_init", "crates/core/src/recovered_main_control.rs"),
    0x14B63A: ("recovered_main_control.preference", "crates/core/src/recovered_main_control.rs"),
    0x105748: ("recovered_loot_control.booty", "crates/core/src/recovered_loot_control.rs"),
    0x1555F2: ("recovered_inventory_interaction.show_item_info", "crates/core/src/recovered_inventory_interaction.rs"),
    0x1300EC: ("recovered_inventory_interaction.load_ammo_proc", "crates/core/src/recovered_inventory_interaction.rs"),
    0x13C1EE: ("recovered_inventory_interaction.move_icon", "crates/core/src/recovered_inventory_interaction.rs"),
    0x112820: ("recovered_inventory_interaction.choice", "crates/core/src/recovered_inventory_interaction.rs"),
    0x152EC2: ("recovered_inventory_interaction.see_shop", "crates/core/src/recovered_inventory_interaction.rs"),
    0x162238: ("recovered_character_upgrade.upgrade_warrior", "crates/core/src/recovered_character_upgrade.rs"),
    0x16399C: ("recovered_character_upgrade.view_character", "crates/core/src/recovered_character_upgrade.rs"),
    0x1476CC: ("recovered_character_upgrade.party_select", "crates/core/src/recovered_character_upgrade.rs"),
    0x13F36A: ("recovered_session_control.generate", "crates/core/src/recovered_session_control.rs"),
    0x13A87C: ("recovered_session_control.quit_game", "crates/core/src/recovered_session_control.rs"),
    0x1378A2: ("recovered_session_control.register_scenario", "crates/core/src/recovered_session_control.rs"),
    0x152378: ("recovered_session_control.save", "crates/core/src/recovered_session_control.rs"),
    0x14D21A: ("recovered_session_control.question2", "crates/core/src/recovered_session_control.rs"),
    0x14D596: ("recovered_session_control.question3", "crates/core/src/recovered_session_control.rs"),
    0x10B89E: ("recovered_selection_control.cast_spell", "crates/core/src/recovered_selection_control.rs"),
    0x1047F8: ("recovered_selection_control.body_count", "crates/core/src/recovered_selection_control.rs"),
    0x103EBE: ("recovered_selection_control.beast", "crates/core/src/recovered_selection_control.rs"),
    0x117356: ("recovered_selection_control.combat_errors", "crates/core/src/recovered_selection_control.rs"),
    0x14D7CC: ("recovered_character_creation.race", "crates/core/src/recovered_character_creation.rs"),
    0x113988: ("recovered_character_creation.caste", "crates/core/src/recovered_character_creation.rs"),
    0x127264: ("recovered_character_creation.gender", "crates/core/src/recovered_character_creation.rs"),
    0x13B30C: ("recovered_character_creation.start_level", "crates/core/src/recovered_character_creation.rs"),
    0x13EF12: ("recovered_character_creation.name", "crates/core/src/recovered_character_creation.rs"),
    0x12ED42: ("recovered_character_creation.level_up", "crates/core/src/recovered_character_creation.rs"),
    0x14ABA0: ("recovered_character_creation.portrait", "crates/core/src/recovered_character_creation.rs"),
    0x1289FA: ("recovered_character_creation.icon_picture", "crates/core/src/recovered_character_creation.rs"),
    0x13F684: ("recovered_character_creation.new_character", "crates/core/src/recovered_character_creation.rs"),
    0x122E24: ("recovered_field_control.file_prep", "crates/core/src/recovered_field_control.rs"),
    0x1374BC: ("recovered_field_control.save_character", "crates/core/src/recovered_field_control.rs"),
    0x12FE54: ("recovered_field_control.load_shop", "crates/core/src/recovered_field_control.rs"),
    0x1538BA: ("recovered_field_control.setup_new_game", "crates/core/src/recovered_field_control.rs"),
    0x111984: ("recovered_field_control.check_keypad", "crates/core/src/recovered_field_control.rs"),
    0x121596: ("recovered_field_control.encounter", "crates/core/src/recovered_field_control.rs"),
    0x13EA82: ("recovered_field_control.move_party", "crates/core/src/recovered_field_control.rs"),
    0x13EBDE: ("recovered_field_control.movie", "crates/core/src/recovered_field_control.rs"),
    0x13ADD4: ("recovered_field_control.band_aid", "crates/core/src/recovered_field_control.rs"),
    0x14EDD0: ("recovered_field_control.do_reload", "crates/core/src/recovered_field_control.rs"),
    0x136928: ("recovered_field_control.enter_capture_mode", "crates/core/src/recovered_field_control.rs"),
    0x100642: ("recovered_inventory_selection.action_picker", "crates/core/src/recovered_inventory_selection.rs"),
    0x112650: ("recovered_inventory_selection.check_for_item", "crates/core/src/recovered_inventory_selection.rs"),
    0x13A5A6: ("recovered_inventory_selection.check_for_do_auto", "crates/core/src/recovered_inventory_selection.rs"),
    0x152188: ("recovered_inventory_selection.order", "crates/core/src/recovered_inventory_selection.rs"),
    0x154A46: ("recovered_inventory_selection.show_condition", "crates/core/src/recovered_inventory_selection.rs"),
    0x156B26: ("recovered_inventory_selection.show_items", "crates/core/src/recovered_inventory_selection.rs"),
    0x15A9CC: ("recovered_inventory_selection.scan_select", "crates/core/src/recovered_inventory_selection.rs"),
    0x15F4B8: ("recovered_inventory_selection.update_booty", "crates/core/src/recovered_inventory_selection.rs"),
    0x160A8E: ("recovered_inventory_selection.update_fat", "crates/core/src/recovered_inventory_selection.rs"),
    0x15075A: ("recovered_combat_results.resolvedamage", "crates/core/src/recovered_combat_results.rs"),
    0x150114: ("recovered_combat_results.resist", "crates/core/src/recovered_combat_results.rs"),
    0x158454: ("recovered_combat_results.showresults", "crates/core/src/recovered_combat_results.rs"),
    0x15B2F6: ("recovered_combat_results.spelleffect", "crates/core/src/recovered_combat_results.rs"),
    0x15EDFA: ("recovered_spell_animation.finddxdy", "crates/core/src/recovered_spell_animation.rs"),
    0x15F048: ("recovered_spell_animation.missileset", "crates/core/src/recovered_spell_animation.rs"),
    0x11084A: ("recovered_inventory_edit.change", "crates/core/src/recovered_inventory_edit.rs"),
    0x134034: ("recovered_inventory_edit.edit_items", "crates/core/src/recovered_inventory_edit.rs"),
    0x14A918: ("recovered_inventory_edit.plus", "crates/core/src/recovered_inventory_edit.rs"),
    0x14AA22: ("recovered_inventory_edit.minus", "crates/core/src/recovered_inventory_edit.rs"),
    0x13F52C: ("recovered_inventory_edit.add_initial_items", "crates/core/src/recovered_inventory_edit.rs"),
    0x14CFF6: ("recovered_prompt_control.question", "crates/core/src/recovered_prompt_control.rs"),
    0x133D46: ("recovered_prompt_control.submit_music", "crates/core/src/recovered_prompt_control.rs"),
    0x15DC34: ("recovered_prompt_control.text_box", "crates/core/src/recovered_prompt_control.rs"),
    0x12382A: ("recovered_prompt_control.first_tutorial", "crates/core/src/recovered_prompt_control.rs"),
    0x16043E: ("recovered_prompt_control.update_controls", "crates/core/src/recovered_prompt_control.rs"),
    0x1500D8: ("recovered_prompt_control.reset_text_area", "crates/core/src/recovered_prompt_control.rs"),
    0x12512A: ("recovered_prompt_control.get_choice", "crates/core/src/recovered_prompt_control.rs"),
    0x127462: ("recovered_startup.initialize_worlds", "crates/core/src/recovered_startup.rs"),
    0x15311A: ("recovered_startup.set_free", "crates/core/src/recovered_startup.rs"),
    0x1531C0: ("recovered_startup.set_free_name", "crates/core/src/recovered_startup.rs"),
    0x157D38: ("recovered_startup.show_logo", "crates/core/src/recovered_startup.rs"),
    0x14C318: ("recovered_startup.check_for_mismatch", "crates/core/src/recovered_startup.rs"),
    0x146B6A: ("recovered_order_form.order_form", "crates/core/src/recovered_order_form.rs"),
    0x1580BE: ("recovered_combat_center.showrange", "crates/core/src/recovered_combat_center.rs"),
    0x10E6FE: ("recovered_combat_center.centerfieldlarge", "crates/core/src/recovered_combat_center.rs"),
    0x10EC52: ("recovered_combat_center.centerfieldsmall", "crates/core/src/recovered_combat_center.rs"),
    0x10E6B6: ("recovered_combat_center.centerfield", "crates/core/src/recovered_combat_center.rs"),
    0x10F186: ("recovered_combat_center.centerpict", "crates/core/src/recovered_combat_center.rs"),
    0x110644: ("recovered_combat_center.centerstage", "crates/core/src/recovered_combat_center.rs"),
    0x1107CC: ("recovered_combat_center.centerstage2", "crates/core/src/recovered_combat_center.rs"),
    0x13A2C4: ("recovered_combat_center.center", "crates/core/src/recovered_combat_center.rs"),
    0x1368C8: ("recovered_combat_center.exitcapturemode", "crates/core/src/recovered_combat_center.rs"),
    0x120872: ("recovered_character_panels.display", "crates/core/src/recovered_character_panels.rs"),
    0x14A3E6: ("recovered_character_panels.plot_portrait", "crates/core/src/recovered_character_panels.rs"),
    0x14A29E: ("recovered_character_panels.update_port", "crates/core/src/recovered_character_panels.rs"),
    0x161B80: ("recovered_character_panels.update_picture_box", "crates/core/src/recovered_character_panels.rs"),
    0x15F66C: ("recovered_character_panels.update_character", "crates/core/src/recovered_character_panels.rs"),
    0x1600FC: ("recovered_character_panels.update_character_short", "crates/core/src/recovered_character_panels.rs"),
    0x1384D6: ("recovered_character_panels.update_preparation", "crates/core/src/recovered_character_panels.rs"),
    0x13A1DC: ("recovered_character_panels.short_update", "crates/core/src/recovered_character_panels.rs"),
    0x13A510: ("recovered_character_panels.copy_out", "crates/core/src/recovered_character_panels.rs"),
    0x13A234: ("recovered_character_panels.select_update", "crates/core/src/recovered_character_panels.rs"),
    0x139244: ("recovered_status_panels.update_shop", "crates/core/src/recovered_status_panels.rs"),
    0x1174F0: ("recovered_status_panels.change_weapon", "crates/core/src/recovered_status_panels.rs"),
    0x117A54: ("recovered_status_panels.combat_info", "crates/core/src/recovered_status_panels.rs"),
    0x1551A2: ("recovered_status_panels.show_condition2", "crates/core/src/recovered_status_panels.rs"),
    0x15741A: ("recovered_status_panels.show_special", "crates/core/src/recovered_status_panels.rs"),
    0x14F96E: ("recovered_storage.reset_game_files", "crates/core/src/recovered_storage.rs"),
    0x151E24: ("recovered_storage.save_land", "crates/core/src/recovered_storage.rs"),
    0x152DC4: ("recovered_storage.save_encounter", "crates/core/src/recovered_storage.rs"),
    0x13006E: ("recovered_storage.save_shop", "crates/core/src/recovered_storage.rs"),
    0x13B212: ("recovered_scenario_menus.update_map_menu", "crates/core/src/recovered_scenario_menus.rs"),
    0x135BB2: ("recovered_scenario_menus.update_monster_menu", "crates/core/src/recovered_scenario_menus.rs"),
    0x133F3E: ("recovered_scenario_menus.display_mission_name", "crates/core/src/recovered_scenario_menus.rs"),
    0x127132: ("recovered_scenario_menus.get_word", "crates/core/src/recovered_scenario_menus.rs"),
    0x134D9E: ("recovered_scenario_menus.fast_plot_map", "crates/core/src/recovered_scenario_menus.rs"),
    0x10F1C4: ("recovered_world_display.centerpictlarge", "crates/core/src/recovered_world_display.rs"),
    0x10FC26: ("recovered_world_display.centerpictsmall", "crates/core/src/recovered_world_display.rs"),
    0x11CE8E: ("recovered_world_display.combatmap", "crates/core/src/recovered_world_display.rs"),
    0x11F1A8: ("recovered_combat_panel.combat_update2", "crates/core/src/recovered_combat_panel.rs"),
    0x15FA20: ("recovered_combat_panel.update_char_info", "crates/core/src/recovered_combat_panel.rs"),
    0x15498E: ("recovered_equipment_ui.show_charge", "crates/core/src/recovered_equipment_ui.rs"),
    0x160D08: ("recovered_equipment_ui.update_items", "crates/core/src/recovered_equipment_ui.rs"),
    0x121152: ("recovered_equipment.dropitem", "crates/core/src/recovered_equipment.rs"),
    0x1212A8: ("recovered_equipment.forcedropitem", "crates/core/src/recovered_equipment.rs"),
    0x164A0E: ("recovered_equipment.wear", "crates/core/src/recovered_equipment.rs"),
    0x14E5E4: ("recovered_equipment.refit", "crates/core/src/recovered_equipment.rs"),
    0x138BE8: ("recovered_load_control.scratch", "crates/core/src/recovered_load_control.rs"),
    0x138EFE: ("recovered_load_control.scratch2", "crates/core/src/recovered_load_control.rs"),
    0x123C60: ("recovered_load_control.load_door", "crates/core/src/recovered_load_control.rs"),
    0x131182: ("recovered_load_control.load_pixmap", "crates/core/src/recovered_load_control.rs"),
    0x13107A: ("recovered_load_control.load_land", "crates/core/src/recovered_load_control.rs"),
    0x1383E6: ("recovered_load_control.load_extra_code", "crates/core/src/recovered_load_control.rs"),
    0x123D1C: ("recovered_feedback.flashmessage", "crates/core/src/recovered_feedback.rs"),
    0x14CE2A: ("recovered_feedback.pressbutton", "crates/core/src/recovered_feedback.rs"),
    0x16474A: ("recovered_feedback.warn", "crates/core/src/recovered_feedback.rs"),
    0x12C46E: ("recovered_inventory_rules.upgrade", "crates/core/src/recovered_inventory_rules.rs"),
    0x14EF76: ("recovered_inventory_rules.removeitem", "crates/core/src/recovered_inventory_rules.rs"),
    0x157586: ("recovered_inventory_rules.canuse", "crates/core/src/recovered_inventory_rules.rs"),
    0x13BEEA: ("recovered_inventory_rules.cancast", "crates/core/src/recovered_inventory_rules.rs"),
    0x159A96: ("recovered_character_ui.show_stats", "crates/core/src/recovered_character_ui.rs"),
    0x15A49A: ("recovered_character_ui.show_upgrade", "crates/core/src/recovered_character_ui.rs"),
    0x148F9E: ("recovered_character_ui.view_restrictions", "crates/core/src/recovered_character_ui.rs"),
    0x161D86: ("recovered_character_ui.update_specialties", "crates/core/src/recovered_character_ui.rs"),
    0x132B0C: ("recovered_detail_ui.quick_info", "crates/core/src/recovered_detail_ui.rs"),
    0x161068: ("recovered_detail_ui.update_odds", "crates/core/src/recovered_detail_ui.rs"),
    0x12DC58: ("recovered_item_ui.show_pack", "crates/core/src/recovered_item_ui.rs"),
    0x133886: ("recovered_item_ui.update_shop_wings", "crates/core/src/recovered_item_ui.rs"),
    0x134AE0: ("recovered_item_ui.update_edit_item", "crates/core/src/recovered_item_ui.rs"),
    0x157758: ("recovered_item_ui.show_item_stats", "crates/core/src/recovered_item_ui.rs"),
    0x146AA2: ("recovered_item_ui.print_stat", "crates/core/src/recovered_item_ui.rs"),
    0x10B6BA: ("recovered_spell_ui.showspelllist", "crates/core/src/recovered_spell_ui.rs"),
    0x159192: ("recovered_spell_ui.showspellinfo", "crates/core/src/recovered_spell_ui.rs"),
    0x15947C: ("recovered_spell_ui.spellinfoupdate", "crates/core/src/recovered_spell_ui.rs"),
    0x162084: ("recovered_spell_ui.updatespell", "crates/core/src/recovered_spell_ui.rs"),
    0x15B84C: ("recovered_spell_ui.spelleffectoutdoors", "crates/core/src/recovered_spell_ui.rs"),
    0x11EDC4: ("recovered_combat_ui.combatupdate", "crates/core/src/recovered_combat_ui.rs"),
    0x123B56: ("recovered_combat_ui.flashrange", "crates/core/src/recovered_combat_ui.rs"),
    0x123F96: ("recovered_combat_ui.flashrect", "crates/core/src/recovered_combat_ui.rs"),
    0x1553BC: ("recovered_combat_ui.showdamage", "crates/core/src/recovered_combat_ui.rs"),
    0x15A1F6: ("recovered_combat_ui.showtargets", "crates/core/src/recovered_combat_ui.rs"),
    0x15EC56: ("recovered_combat_ui.tickcheck", "crates/core/src/recovered_combat_ui.rs"),
    0x124D9E: ("recovered_battle_order.getbattleorder", "crates/core/src/recovered_battle_order.rs"),
    0x13A6D4: ("recovered_verification.doreg", "crates/core/src/recovered_verification.rs"),
    0x1469F8: ("recovered_verification.doreg3", "crates/core/src/recovered_verification.rs"),
    0x13AEB8: ("recovered_verification.setverify", "crates/core/src/recovered_verification.rs"),
    0x13B0EA: ("recovered_verification.checkverify", "crates/core/src/recovered_verification.rs"),
    0x100212: ("recovered_runtime.relocate", "crates/core/src/recovered_runtime.rs"),
    0x100286: ("recovered_runtime.relocate_three", "crates/core/src/recovered_runtime.rs"),
    0x1002B2: ("recovered_runtime.unrelocate_three", "crates/core/src/recovered_runtime.rs"),
    0x1002EC: ("recovered_runtime.runtime_transfer", "crates/core/src/recovered_runtime.rs"),
    0x1004AE: ("recovered_runtime.run_destructors", "crates/core/src/recovered_runtime.rs"),
    0x1004D2: ("recovered_runtime.multiply_long", "crates/core/src/recovered_runtime.rs"),
    0x100504: ("recovered_runtime.divide_unsigned_long", "crates/core/src/recovered_runtime.rs"),
    0x100584: ("recovered_runtime.remainder_unsigned_long", "crates/core/src/recovered_runtime.rs"),
    0x13E13C: ("recovered_runtime.compact_heap", "crates/core/src/recovered_runtime.rs"),
    0x153CA4: ("recovered_runtime.rand", "crates/core/src/recovered_runtime.rs"),
    0x13A498: ("recovered_runtime.copy_in", "crates/core/src/recovered_runtime.rs"),
    0x120C14: ("recovered_rendering.drawbody", "crates/core/src/recovered_rendering.rs"),
    0x157EF8: ("recovered_rendering.showque", "crates/core/src/recovered_rendering.rs"),
    0x15F210: ("recovered_rendering.updatearrow", "crates/core/src/recovered_rendering.rs"),
    0x10AE90: ("recovered_navigation.cansee", "crates/core/src/recovered_navigation.rs"),
    0x136994: ("recovered_navigation.movecalc", "crates/core/src/recovered_navigation.rs"),
    0x138A40: ("recovered_navigation.xy", "crates/core/src/recovered_navigation.rs"),
    0x13846A: ("recovered_navigation.twixt", "crates/core/src/recovered_navigation.rs"),
    0x1046E0: ("recovered_shop_ui.updatenpcmenu", "crates/core/src/recovered_shop_ui.rs"),
    0x139FAA: ("recovered_shop_ui.shop_credit_display", "crates/core/src/recovered_shop_ui.rs"),
    0x13A068: ("recovered_shop_ui.eraseshopstats", "crates/core/src/recovered_shop_ui.rs"),
    0x13A10A: ("recovered_shop_ui.eraseshopname", "crates/core/src/recovered_shop_ui.rs"),
    0x1315A2: ("recovered_fade_ui.fadeinout", "crates/core/src/recovered_fade_ui.rs"),
    0x124212: ("recovered_gamma_control.get_monitor_state", "crates/core/src/recovered_gamma_control.rs"),
    0x124242: ("recovered_gamma_control.release_monitor_state", "crates/core/src/recovered_gamma_control.rs"),
    0x12427C: ("recovered_gamma_control.capture_monitor_list", "crates/core/src/recovered_gamma_control.rs"),
    0x124308: ("recovered_gamma_control.capture_monitor_gamma", "crates/core/src/recovered_gamma_control.rs"),
    0x124632: ("recovered_gamma_control.fade_to_monitor_state", "crates/core/src/recovered_gamma_control.rs"),
    0x124666: ("recovered_gamma_control.fade_to_constant_color", "crates/core/src/recovered_gamma_control.rs"),
    0x1246A8: ("recovered_gamma_control.run_fade_sequence", "crates/core/src/recovered_gamma_control.rs"),
    0x1213F6: ("recovered_preferences.set_indexed_string", "crates/core/src/recovered_preferences.rs"),
    0x14BCA4: ("recovered_preferences.get_preferences", "crates/core/src/recovered_preferences.rs"),
    0x14C01C: ("recovered_preferences.save_preferences", "crates/core/src/recovered_preferences.rs"),
    0x14C440: ("recovered_preferences.create_preferences", "crates/core/src/recovered_preferences.rs"),
    0x14C622: ("recovered_preferences.open_preferences", "crates/core/src/recovered_preferences.rs"),
    0x12C874: ("recovered_install.installitem_complete", "crates/core/src/recovered_install.rs"),
    0x1241B4: ("recovered_gamma.is_color_gamma_available", "crates/core/src/recovered_gamma.rs"),
    0x124576: ("recovered_gamma.restore_gamma_list", "crates/core/src/recovered_gamma.rs"),
    0x1245CC: ("recovered_gamma.dispose_gamma_list", "crates/core/src/recovered_gamma.rs"),
    0x1247CC: ("recovered_gamma.x_calc_fade_to_color", "crates/core/src/recovered_gamma.rs"),
    0x12485E: ("recovered_gamma.fade_gamma_list", "crates/core/src/recovered_gamma.rs"),
    0x1248F0: ("recovered_gamma.calc_fade_to_color", "crates/core/src/recovered_gamma.rs"),
    0x124A44: ("recovered_gamma.x_calc_fade_to_gamma", "crates/core/src/recovered_gamma.rs"),
    0x124BBA: ("recovered_gamma.get_device_gamma_table", "crates/core/src/recovered_gamma.rs"),
    0x124C18: ("recovered_gamma.set_device_gamma_table", "crates/core/src/recovered_gamma.rs"),
    0x124CA2: ("recovered_gamma.color_gamma_trap_available", "crates/core/src/recovered_gamma.rs"),
    0x124D04: ("recovered_gamma.color_gamma_memset", "crates/core/src/recovered_gamma.rs"),
    0x12D558: ("recovered_random_item.randomitem", "crates/core/src/recovered_random_item.rs"),
    0x1056D4: ("recovered_drawing.updatebootymoney", "crates/core/src/recovered_drawing.rs"),
    0x10B814: ("recovered_drawing.updatetier", "crates/core/src/recovered_drawing.rs"),
    0x12946C: ("recovered_interface.inwindow", "crates/core/src/recovered_interface.rs"),
    0x13154C: ("recovered_interface.locationlog", "crates/core/src/recovered_interface.rs"),
    0x131564: ("recovered_interface.centercursor", "crates/core/src/recovered_interface.rs"),
    0x14EE9E: ("recovered_interface.hide_menu_bar", "crates/core/src/recovered_interface.rs"),
    0x14EF36: ("recovered_interface.show_menu_bar", "crates/core/src/recovered_interface.rs"),
    0x153720: ("recovered_interface.downbutton", "crates/core/src/recovered_interface.rs"),
    0x1537EE: ("recovered_interface.upbutton", "crates/core/src/recovered_interface.rs"),
    0x160FB2: ("recovered_interface.updatelight", "crates/core/src/recovered_interface.rs"),
    0x136CFA: ("recovered_files.get_filename", "crates/core/src/recovered_files.rs"),
    0x136D86: ("recovered_files.get_temp_filename", "crates/core/src/recovered_files.rs"),
    0x136E04: ("recovered_files.get_save_filename", "crates/core/src/recovered_files.rs"),
    0x13777A: ("recovered_files.set_file_info", "crates/core/src/recovered_files.rs"),
    0x13BA6C: ("recovered_files.open_race", "crates/core/src/recovered_files.rs"),
    0x13BAC6: ("recovered_files.open_caste", "crates/core/src/recovered_files.rs"),
    0x13A7C4: ("recovered_files.load_profile", "crates/core/src/recovered_files.rs"),
    0x14BFE8: ("recovered_files.get_application_name", "crates/core/src/recovered_files.rs"),
    0x136398: ("recovered_install.nobrassnoammo", "crates/core/src/recovered_install.rs"),
    0x122B18: ("recovered_drawing.allocatebuilding", "crates/core/src/recovered_drawing.rs"),
    0x122BEC: ("recovered_drawing.fastplot", "crates/core/src/recovered_drawing.rs"),
    0x122DC4: ("recovered_drawing.gcopy", "crates/core/src/recovered_drawing.rs"),
    0x149FC6: ("recovered_drawing.ploticon2", "crates/core/src/recovered_drawing.rs"),
    0x14A02C: ("recovered_drawing.ploticon3", "crates/core/src/recovered_drawing.rs"),
    0x14A074: ("recovered_drawing.ploticon", "crates/core/src/recovered_drawing.rs"),
    0x1377F4: ("recovered_drawing.drawspell", "crates/core/src/recovered_drawing.rs"),
    0x1382A8: ("recovered_drawing.pict", "crates/core/src/recovered_drawing.rs"),
    0x10518A: ("recovered_bodies.monsterupdate", "crates/core/src/recovered_bodies.rs"),
    0x105572: ("recovered_bodies.bodyground", "crates/core/src/recovered_bodies.rs"),
    0x12063C: ("recovered_bodies.destroymonster", "crates/core/src/recovered_bodies.rs"),
    0x149E0E: ("recovered_bodies.placemonster", "crates/core/src/recovered_bodies.rs"),
    0x133A18: ("recovered_audio.sound", "crates/core/src/recovered_audio.rs"),
    0x133BB8: ("recovered_audio.start_sound_cycle", "crates/core/src/recovered_audio.rs"),
    0x133C56: ("recovered_audio.sustain_sound_cycle", "crates/core/src/recovered_audio.rs"),
    0x13B56C: ("recovered_audio.quiet", "crates/core/src/recovered_audio.rs"),
    0x153584: ("recovered_audio.set_speaker_volume", "crates/core/src/recovered_audio.rs"),
    0x136F90: ("recovered_attack.calcdamage", "crates/core/src/recovered_attack.rs"),
    0x14C6C2: ("recovered_attack.prepattack", "crates/core/src/recovered_attack.rs"),
    0x100A38: ("recovered_inventory.addtocrate", "crates/core/src/recovered_inventory.rs"),
    0x100A9A: ("recovered_inventory.addtobooty", "crates/core/src/recovered_inventory.rs"),
    0x12006E: ("recovered_inventory.compactitems", "crates/core/src/recovered_inventory.rs"),
    0x139192: ("recovered_inventory.calcw", "crates/core/src/recovered_inventory.rs"),
    0x15D210: ("recovered_inventory.takegold", "crates/core/src/recovered_inventory.rs"),
    0x1205F8: ("recovered_timing.delay", "crates/core/src/recovered_timing.rs"),
    0x157EA4: ("recovered_timing.delay_interruptible", "crates/core/src/recovered_timing.rs"),
    0x110D12: ("recovered_spatial.checkformechanicalunits", "crates/core/src/recovered_spatial.rs"),
    0x112208: ("recovered_spatial.checklayout", "crates/core/src/recovered_spatial.rs"),
    0x11235E: ("recovered_spatial.checkrange", "crates/core/src/recovered_spatial.rs"),
    0x1124DC: ("recovered_spatial.getrangetobody", "crates/core/src/recovered_spatial.rs"),
    0x125538: ("recovered_spatial.getrange", "crates/core/src/recovered_spatial.rs"),
    0x125960: ("recovered_spatial.getclosest", "crates/core/src/recovered_spatial.rs"),
    0x151F30: ("recovered_spatial.direction", "crates/core/src/recovered_spatial.rs"),
    0x130AFE: ("recovered_records.loaditem", "crates/core/src/recovered_records.rs"),
    0x130C18: ("recovered_records.loaditemcompare", "crates/core/src/recovered_records.rs"),
    0x130D38: ("recovered_records.loadweapon", "crates/core/src/recovered_records.rs"),
    0x130E54: ("recovered_records.loadammo", "crates/core/src/recovered_records.rs"),
    0x130F6E: ("recovered_records.loaditemnotype", "crates/core/src/recovered_records.rs"),
    0x13145C: ("recovered_records.loadspell2", "crates/core/src/recovered_records.rs"),
    0x1314FC: ("recovered_records.loadspell", "crates/core/src/recovered_records.rs"),
    0x13822A: ("recovered_records.cleartarget", "crates/core/src/recovered_records.rs"),
    0x1270C0: ("recovered_text.get_version_string", "crates/core/src/recovered_text.rs"),
    0x1382D0: ("recovered_text.string", "crates/core/src/recovered_text.rs"),
    0x1382FC: ("recovered_text.string_long", "crates/core/src/recovered_text.rs"),
    0x13832A: ("recovered_text.string_plus", "crates/core/src/recovered_text.rs"),
    0x138368: ("recovered_text.string_no_zero_no_plus", "crates/core/src/recovered_text.rs"),
    0x1383A6: ("recovered_text.string_no_zero", "crates/core/src/recovered_text.rs"),
    0x1389E4: ("recovered_text.get_dialog_number", "crates/core/src/recovered_text.rs"),
    0x13A72A: ("recovered_text.dialog_number_nonzero", "crates/core/src/recovered_text.rs"),
    0x14695E: ("recovered_text.number", "crates/core/src/recovered_text.rs"),
    0x146A50: ("recovered_text.copy_part_of_string", "crates/core/src/recovered_text.rs"),
    0x153CF4: ("recovered_text.dialog_number_long", "crates/core/src/recovered_text.rs"),
    0x153D56: ("recovered_text.dialog_number", "crates/core/src/recovered_text.rs"),
    0x153DAA: ("recovered_text.dialog_string", "crates/core/src/recovered_text.rs"),
    0x153DE6: ("recovered_preparation.movement_cost", "crates/core/src/recovered_preparation.rs"),
    0x15EF50: ("recovered_preparation.to_weapon", "crates/core/src/recovered_preparation.rs"),
    0x15EFEE: ("recovered_preparation.to_missile", "crates/core/src/recovered_preparation.rs"),
    0x105366: ("combat.footprint.restore_body", "crates/core/src/combat/footprint.rs"),
    0x10B13A: ("recovered_geometry.cansee2", "crates/core/src/recovered_geometry.rs"),
    0x10B3FA: ("recovered_geometry.cansee3", "crates/core/src/recovered_geometry.rs"),
    0x10D3F6: ("recovered_geometry.calcbodyrect", "crates/core/src/recovered_geometry.rs"),
    0x11279A: ("party.item_charges", "crates/core/src/party.rs"),
    0x125512: ("rules.spell_count", "crates/core/src/lib.rs"),
    0x125A08: ("rules.selection_base", "crates/core/src/lib.rs"),
    0x1366A8: ("party.pool_credits", "crates/core/src/party.rs"),
    0x13684A: ("party.share_credits", "crates/core/src/party.rs"),
    0x138648: ("character.strength_adjustments", "crates/core/src/character.rs"),
    0x1380AC: ("recovered_economy.getexp", "crates/core/src/recovered_economy.rs"),
    0x1388AA: ("character.agility_adjustment", "crates/core/src/character.rs"),
    0x1363E4: ("recovered_ai.best_choice", "crates/core/src/recovered_ai.rs"),
    0x13849E: ("recovered_ai.random_range", "crates/core/src/recovered_ai.rs"),
    0x13A3AE: ("recovered_ai.save_against", "crates/core/src/recovered_ai.rs"),
    0x153E44: ("recovered_ai.shift", "crates/core/src/recovered_ai.rs"),
    0x13F65E: ("rules.pin", "crates/core/src/lib.rs"),
    0x13C018: ("recovered_economy.itemcost", "crates/core/src/recovered_economy.rs"),
    0x157C68: ("character.save_against_attribute", "crates/core/src/character.rs"),
    0x1634F8: ("character.apply_upgrade", "crates/core/src/character.rs"),
    0x1636EC: ("character.calculate_statistics", "crates/core/src/character.rs"),
}


# Additional native bodies compose through the shared application boundary.
# Keep existing typed owners and independent reference cases in the catalog.
SHARED_NATIVE_LEAVES = {
    0x105366: "restore_body",
    0x105572: "bodyground",
    0x10AE90: "cansee",
    0x110D12: "checkformechanicalunits",
    0x11235E: "checkrange",
    0x1124DC: "getrangetobody",
    0x11279A: "item_charges",
    0x12063C: "destroymonster",
    0x122BEC: "fastplot",
    0x125538: "getrange",
    0x125960: "getclosest",
    0x133BB8: "start_sound_cycle",
    0x133C56: "sustain_sound_cycle",
    0x136398: "nobrassnoammo",
    0x1363E4: "best_choice",
    0x1377F4: "drawspell",
    0x1382D0: "string",
    0x138A40: "xy",
    0x13A3AE: "save_against",
    0x13F65E: "pin",
    0x149E0E: "placemonster",
    0x151F30: "direction",
    0x153DE6: "movement_cost",
    0x153E44: "shift",
    0x157C68: "save_against_attribute",
    0x15B84C: "spelleffectoutdoors",
    0x15D210: "takegold",
    0x15EDFA: "finddxdy",
    0x15EF50: "to_weapon",
    0x15EFEE: "to_missile",
    0x15F048: "missileset",
    0x160FB2: "updatelight",
}


SHARED_NATIVE_CHILDREN = {
    0x1213F6: 'set_ind_string',
    0x122B18: 'allocate_building',
    0x122DC4: 'gcopy',
    0x1241B4: 'is_color_gamma_available',
    0x124212: 'get_monitor_state',
    0x124242: 'release_monitor_state',
    0x12427C: 'capture_monitor_list',
    0x124308: 'capture_monitor_gamma',
    0x124576: 'restore_gamma_list',
    0x1245CC: 'dispose_gamma_list',
    0x124632: 'fade_to_monitor_state',
    0x124666: 'fade_to_constant_color',
    0x1246A8: 'run_fade_sequence',
    0x1247CC: 'x_calc_fade_to_color',
    0x12485E: 'fade_gamma_list',
    0x1248F0: 'calc_fade_to_color',
    0x124A44: 'x_calc_fade_to_gamma',
    0x124BBA: 'get_device_gamma_table',
    0x124C18: 'set_device_gamma_table',
    0x124CA2: 'color_gamma_trap_available',
    0x124D04: 'color_gamma_memset',
    0x1270C0: 'get_version_string',
    0x127462: 'gworld_init',
    0x1315A2: 'fade_in_out',
    0x1366A8: 'pool',
    0x136E04: 'get_save_filename',
    0x13777A: 'set_file_info',
    0x13BA6C: 'open_race',
    0x13EF12: 'name',
    0x14ABA0: 'portrait',
    0x14BCA4: 'load_preferences',
    0x14BFE8: 'application_name',
    0x14C01C: 'save_preferences',
    0x14C318: 'check_for_mismatch',
    0x14C440: 'create_preferences',
    0x14C622: 'open_preferences',
    0x14EE9E: 'hide_menu_bar',
    0x14EF36: 'show_menu_bar',
    0x15311A: 'set_free',
    0x1531C0: 'set_free2',
    0x153584: 'set_speaker_volume',
    0x153CF4: 'dialog_number_long',
    0x153D56: 'dialog_number',
    0x153DAA: 'dialog_string',
    0x157D38: 'show_logo',
    0x157EA4: 'delay_interruptible',
}


SHARED_NATIVE_REMAINING = {
    0x100212: 'relocate',
    0x100286: 'relocate_three',
    0x1002B2: 'unrelocate_three',
    0x1002EC: 'runtime_transfer',
    0x1004AE: 'run_destructors',
    0x1004D2: 'multiply_long',
    0x100504: 'divide_unsigned_long',
    0x100584: 'remainder_unsigned_long',
    0x100A9A: 'addtobooty',
    0x10518A: 'monsterupdate',
    0x1056D4: 'updatebootymoney',
    0x10B13A: 'cansee2',
    0x10B3FA: 'cansee3',
    0x10B814: 'updatetier',
    0x10D3F6: 'calcbodyrect',
    0x10F1C4: 'centerpictlarge',
    0x10FC26: 'centerpictsmall',
    0x11084A: 'change',
    0x112208: 'checklayout',
    0x113988: 'caste',
    0x120872: 'display',
    0x123B56: 'flashrange',
    0x123F96: 'flashrect',
    0x125512: 'spell_count',
    0x127264: 'gender',
    0x12C874: 'installitem_complete',
    0x12D558: 'randomitem',
    0x12DC58: 'show_pack',
    0x12ED42: 'level_up',
    0x130C18: 'loaditemcompare',
    0x130F6E: 'loaditemnotype',
    0x133886: 'update_shop_wings',
    0x134AE0: 'update_edit_item',
    0x13684A: 'share_credits',
    0x136F90: 'calcdamage',
    0x1380AC: 'getexp',
    0x1382FC: 'string_long',
    0x13832A: 'string_plus',
    0x138368: 'string_no_zero_no_plus',
    0x1383A6: 'string_no_zero',
    0x1384D6: 'update_preparation',
    0x138648: 'strength_adjustments',
    0x1388AA: 'agility_adjustment',
    0x1389E4: 'get_dialog_number',
    0x139244: 'update_shop',
    0x139FAA: 'shop_credit_display',
    0x13A068: 'eraseshopstats',
    0x13A10A: 'eraseshopname',
    0x13A72A: 'dialog_number_nonzero',
    0x13AEB8: 'setverify',
    0x13B0EA: 'checkverify',
    0x13B30C: 'start_level',
    0x13BAC6: 'open_caste',
    0x13C018: 'itemcost',
    0x13F52C: 'add_initial_items',
    0x13F684: 'new_character',
    0x14695E: 'number',
    0x146A50: 'copy_part_of_string',
    0x146AA2: 'print_stat',
    0x148F9E: 'view_restrictions',
    0x149FC6: 'ploticon2',
    0x14A074: 'ploticon',
    0x14A29E: 'update_port',
    0x14A918: 'plus',
    0x14AA22: 'minus',
    0x14D7CC: 'race',
    0x15498E: 'show_charge',
    0x1553BC: 'showdamage',
    0x15741A: 'show_special',
    0x157758: 'show_item_stats',
    0x157EF8: 'showque',
    0x159A96: 'show_stats',
    0x15A49A: 'show_upgrade',
    0x161068: 'update_odds',
    0x161D86: 'update_specialties',
    0x162238: 'upgrade_warrior',
    0x1634F8: 'apply_upgrade',
    0x1636EC: 'calculate_statistics',
}


# Source-derived presentation covers selected branches of these routines.
PRESENTATION = {
    "main", "MainLoop", "HandleEvent", "HandleMenuChoice", "MenuInit",
    "showlogo", "delay2", "partyselect", "quitgame", "question", "sound",
}


def address(value: str) -> int:
    return int(value, 16)


def dependency_batches(graph: dict[int, set[int]]) -> list[list[int]]:
    """Tarjan components, then dependency-first layers (cycles stay together)."""
    index, low, stack, active, components = {}, {}, [], set(), []

    def visit(node):
        index[node] = low[node] = len(index)
        stack.append(node)
        active.add(node)
        for called in sorted(graph[node]):
            if called not in index:
                visit(called)
                low[node] = min(low[node], low[called])
            elif called in active:
                low[node] = min(low[node], index[called])
        if low[node] == index[node]:
            component = []
            while True:
                member = stack.pop()
                active.remove(member)
                component.append(member)
                if member == node:
                    break
            components.append(sorted(component))

    for node in sorted(graph):
        if node not in index:
            visit(node)
    owner = {node: group for group, members in enumerate(components) for node in members}
    pending = {
        group: {owner[called] for node in members for called in graph[node] if owner[called] != group}
        for group, members in enumerate(components)
    }
    batches = []
    while pending:
        ready = sorted((group for group, dependencies in pending.items() if not dependencies),
                       key=lambda group: components[group][0])
        if not ready:
            raise ValueError("Component graph contains a cycle")
        batches.append([node for group in ready for node in components[group]])
        for group in ready:
            del pending[group]
        for dependencies in pending.values():
            dependencies.difference_update(ready)
    return batches


def build_plan(inventory: dict, registry: dict) -> dict:
    functions = sorted(inventory["functions"], key=lambda f: address(f["entry"]))
    entries = {address(f["entry"]) for f in functions}
    if len(entries) != len(functions):
        raise ValueError("Duplicate recovered routine entry")
    ranges = [(address(r["start"]), address(r["end"]), address(f["entry"]))
              for f in functions for r in f["ranges"]]
    bindings = {entry: [] for entry in entries}
    recovery_bindings = {entry: [] for entry in entries}
    for action in registry["actions"]:
        if action.get("stage") not in {"implemented", "integrated"}:
            continue
        for value in action.get("mapped_entry_addresses", []):
            owners = {entry for start, end, entry in ranges if start <= address(value) <= end}
            if len(owners) > 1:
                raise ValueError(f"Ambiguous routine owner for {value}")
            for entry in owners:
                implementation = action.get("implementation", "")
                paths = sorted(set(re.findall(r"crates/[A-Za-z0-9_./-]+\.rs", implementation)))
                binding = {"action": action["id"], "rust": paths,
                           "implementation_notes": implementation,
                           "native_cases": action.get("case_count", 0), "fixture": action.get("fixture")}
                destination = bindings[entry] if paths else recovery_bindings[entry]
                if binding not in destination:
                    destination.append(binding)
    graph, routines, actions = {}, [], []
    for function in functions:
        entry = address(function["entry"])
        calls = sorted({address(c["entry"]) for c in function["calls"]})
        graph[entry] = set(calls).intersection(entries)
        source_port = SOURCE_PORTS.get(entry)
        rust = bindings[entry]
        if function["name"] in PRESENTATION:
            rust.append({"action": "presentation.selected_branches", "rust": ["crates/desktop/src/application.rs"],
                         "native_cases": 0, "fixture": None})
        if source_port:
            rust.append({"action": source_port[0], "rust": [source_port[1]], "native_cases": 0, "fixture": None})
        if entry in SHARED_NATIVE_LEAVES:
            rust.append({"action": "recovered_gameplay_leaves." + SHARED_NATIVE_LEAVES[entry],
                         "rust": ["crates/core/src/recovered_gameplay_leaves.rs"],
                         "native_cases": 0, "fixture": None})
        if entry in SHARED_NATIVE_CHILDREN:
            rust.append({"action": "recovered_application_children." + SHARED_NATIVE_CHILDREN[entry],
                         "rust": ["crates/core/src/recovered_application_children.rs"],
                         "native_cases": 0, "fixture": None})
        if entry in SHARED_NATIVE_REMAINING:
            rust.append({"action": "recovered_remaining_control." + SHARED_NATIVE_REMAINING[entry],
                         "rust": ["crates/core/src/recovered_remaining_control.rs"],
                         "native_cases": 0, "fixture": None})
        status = "source_port" if source_port else "partial" if rust else "recovered"
        instruction_count = operation_count = 0
        for block in function["blocks"]:
            steps = []
            for instruction in block["instructions"]:
                operations = instruction["operations"]
                instruction_count += 1
                operation_count += len(operations)
                widths = sorted({node["bytes"] for op in operations
                                 for node in op["inputs"] + ([op["output"]] if "output" in op else [])})
                steps.append({"address": instruction["address"],
                              "operations": [op["opcode"] for op in operations], "operand_bytes": widths})
            actions.append({"id": f"code1.{function['entry']}.{block['entry']}",
                            "routine": function["entry"], "entry": block["entry"],
                            "level": "ordered_instruction_block", "destinations": block["destinations"],
                            "steps": steps})
        routines.append({"name": function["name"], "entry": function["entry"],
                         "body_bytes": function["body_bytes"], "decompiled": function["decompiled"],
                         "blocks": len(function["blocks"]), "instructions": instruction_count,
                         "operations": operation_count, "status": status, "rust_bindings": rust,
                         "recovery_bindings": recovery_bindings[entry],
                         "application_calls": sorted(graph[entry]), "external_calls": sorted(set(calls) - entries),
                         "native_comparison": "scoped_cases" if any(b["native_cases"] for b in rust) else "none"})
    return {"schema_version": 1, "scope": inventory["scope"],
            "qualification": "Action blocks are recovered IR, not implemented Rust or whole-routine fidelity. Source ports use explicit record/service adapters; scoped comparisons cover only their declared cases.",
            "summary": {"routines": len(routines), "decompiled": sum(f["decompiled"] for f in routines),
                        "blocks": len(actions), "instructions": sum(f["instructions"] for f in routines),
                        "operations": sum(f["operations"] for f in routines),
                        "translation": dict(Counter(f["status"] for f in routines))},
            "routines": routines, "actions": actions, "dependency_batches": dependency_batches(graph)}


def rust_catalog(plan: dict) -> str:
    states = {"recovered": "Recovered", "partial": "Partial", "source_port": "SourcePort"}
    lines = ["// Generated by tools/plan_translation.py. Contains recovery metadata only.",
             "use super::{Routine, Translation};", "", "#[rustfmt::skip]",
             "pub const ROUTINES: &[Routine] = &["]
    for row in plan["routines"]:
        paths = sorted({path for b in row["rust_bindings"] for path in b["rust"]})
        calls = ", ".join(f"0x{entry:x}" for entry in row["application_calls"])
        quoted_paths = ", ".join(json.dumps(path) for path in paths)
        lines.append("    Routine { " +
                     f'name: {json.dumps(row["name"])}, entry: 0x{row["entry"]}, decompiled: {str(row["decompiled"]).lower()}, ' +
                     f'blocks: {row["blocks"]}, instructions: {row["instructions"]}, ' +
                     f'translation: Translation::{states[row["status"]]}, rust: &[{quoted_paths}], calls: &[{calls}]' + " },")
    lines.extend(["];", ""])
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("inventory", type=Path)
    parser.add_argument("--registry", type=Path, default=Path("evidence/actions.json"))
    parser.add_argument("--plan", type=Path, default=Path("analysis/reconstruction/translation-plan.json"))
    parser.add_argument("--catalog", type=Path, default=Path("crates/core/src/reconstruction/catalog.rs"))
    args = parser.parse_args()
    plan = build_plan(json.loads(args.inventory.read_text()), json.loads(args.registry.read_text()))
    args.plan.parent.mkdir(parents=True, exist_ok=True)
    args.plan.write_text(json.dumps(plan, indent=2) + "\n")
    args.catalog.parent.mkdir(parents=True, exist_ok=True)
    args.catalog.write_text(rust_catalog(plan))
    print(json.dumps(plan["summary"], indent=2))
    print(f"Dependency layers: {len(plan['dependency_batches'])}; plan: {args.plan}; Rust catalog: {args.catalog}")


if __name__ == "__main__":
    main()
