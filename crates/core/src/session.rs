use crate::{
    AMMUNITION_SLOTS, CommandPreferences, DEFINE_SPELLS_BYTES, EVENT_HEADER_BYTES,
    EXTRA_RECORD_BYTES, FLAG_BYTES, Globals, LOOT_QUALITY_BYTES, NOTE_BYTES, PARTY_CONDITION_BYTES,
    Phase, Presentation, REGION_BYTES, RandomState, RuleError, STORED_ITEM_RECORDS, Session,
    TREASURE_RECORD_BYTES, clinic, combat, empty_allies, empty_ammunition, empty_notes,
    empty_party_buffs, field_controls, initial_flags, menu_length, outdoor_psi, shop, story_actor,
    story_allies, story_cleanup, typed_encounter,
};
use centurions_content::records::{ItemDefinition, RegionTable};
use centurions_content::{Content, InventoryRecord, Warrior};

impl Session {
    /// Loads the definition of item `id` into the loaded-item globals.
    ///
    /// Original: loaditem
    pub(crate) fn load_item(&mut self, content: &Content, id: i16) -> Result<(), RuleError> {
        self.globals.loaded_item_field32 = self.globals.loaded_items.load_item(content, id)?;
        Ok(())
    }
    /// A new game: the squad is being selected, no mission has begun, and every record starts
    /// empty.
    pub fn new(content: &Content) -> Self {
        Self {
            content_identity: content.identity.clone(),
            difficulty: 0,
            command_preferences: CommandPreferences::default(),
            command_return: false,
            warriors: vec![],
            phase: Phase::SquadSelection,
            mission: None,
            scenario_level: 0,
            visited_levels: Default::default(),
            imported: None,
            shops: Default::default(),
            shop_access: None,
            field_controls: None,
            confiscated: None,
            recovery_return: None,
            suspend_allies: 0,
            loot_limit: None,
            typed_encounters: vec![],
            exploration: None,
            random: RandomState::new(12),
            crate_items: vec![InventoryRecord([0; 22]); STORED_ITEM_RECORDS],
            booty_list: vec![InventoryRecord([0; 22]); STORED_ITEM_RECORDS],
            ammunition: empty_ammunition(),
            combat: None,
            battle_treasure: 0,
            ordinary_battle_rewards: None,
            selected_battle_treasure: None,
            battle_after_message: 0,
            battle_group_script: None,
            battle_npc: None,
            char_select_new: 0,
            globals: Globals::startup(),
            party_condition: empty_party_buffs(),
            scenario_duration_tail: None,
            psi_disabled: 0,
            enemy_spell_disabled: 0,
            energy_recovery_disabled: 0,
            party_currency: 0,
            library_random_seed: 1,
            library_random_draws: 0,
            combat_story: None,
            out_of_combat_walk_choice: None,
            monster_action: None,
            survivors: None,
            round_dispatch: None,
            turn_dispatch: None,
            scripted_cleanup: None,
            actor_replacement: None,
            movement_continuation: None,
            time_continuation: None,
            zero_time: None,
            deferred_getup: false,
            booty_tally: crate::BootyTally::default(),
            psi_selection: None,
            target_quantity: None,
            outdoor_psi: None,
            encounter_actions: None,
            promotion: None,
            medical_continuation: None,
            psi_menu_choices: Default::default(),
            bandage_ready: 0,
            handled_deaths: vec![],
            menus: vec![],
            flags: initial_flags(),
            mission_score: 0,
            mission_stage: 0,
            map_picture: 0,
            notes: empty_notes(),
            saved_position: [0; 3],
            visited_maps: std::collections::BTreeMap::new(),
            holdover: empty_allies(),
            held_over: 0,
        }
    }

    /// Checks that a session is one the game could have reached, so a bad save or command is
    /// refused before it is kept.
    pub fn validate(&self, content: &Content) -> Result<(), RuleError> {
        self.validate_global_records(content)?;
        let context = content.context(self.mission.as_deref(), self.scenario_level)?;
        let content = context.as_ref();
        self.validate_globals(content)?;
        if self.content_identity != content.identity {
            return Err(RuleError::Invalid("content identity mismatch".into()));
        }
        self.validate_capacities(content)?;
        self.validate_warrior_records()?;
        self.validate_command_scope_flags()?;
        if let Some(pending) = &self.medical_continuation {
            pending.validate(self, content)?;
        }
        if self
            .psi_menu_choices
            .iter()
            .flatten()
            .any(|choice| choice.row > 6 || choice.column > 6 || !(1..=7).contains(&choice.power))
        {
            return Err(RuleError::Invalid(
                "invalid remembered psi selection".into(),
            ));
        }
        if let Some(story) = &self.combat_story {
            story.validate(self, content)?;
        }
        if self
            .out_of_combat_walk_choice
            .as_ref()
            .is_some_and(|choice| {
                !(0..110).contains(&choice.body)
                    || self.combat.is_some()
                    || self.exploration.is_none()
            })
        {
            return Err("invalid retained occupied-cell choice".into());
        }
        if self.handled_deaths.iter().any(|i| {
            self.combat
                .as_ref()
                .is_none_or(|c| *i >= c.roster.records.len())
        }) {
            return Err(RuleError::Invalid("invalid combat story state".into()));
        }
        self.validate_combat_presence()?;
        self.validate_pending_frames(content)?;
        self.validate_survivors()?;
        self.validate_round_presentation(content)?;
        self.validate_targeting(content)?;
        self.validate_visited_maps(content)?;
        self.validate_mission_position(content)?;
        self.validate_visited_levels(content)
    }

    /// Checks the records kept across scenes: the boat, loot recovery, reward, monster, bestiary,
    /// battlefield, neighbor, encounter header and battle order records, the shops, confiscated
    /// equipment and imported save records.
    fn validate_global_records(&self, content: &Content) -> Result<(), RuleError> {
        crate::exploration::boat::validate(self)?;
        crate::loot_recovery::validate(self)?;
        if self.globals.treasure.len() != TREASURE_RECORD_BYTES
            || self.globals.extra_record.len() != EXTRA_RECORD_BYTES
        {
            return Err("invalid retained reward records".into());
        }
        self.globals.monsters.validate()?;
        self.globals.monster_pick.validate()?;
        self.globals.bestiary.validate()?;
        if self.globals.track_tail.len() != combat::TARGET_TAIL_WORDS {
            return Err("invalid retained spell target mask".into());
        }
        self.globals.battlefield.validate()?;
        self.globals.neighbors.validate()?;
        self.globals.encounter_headers.validate()?;
        self.globals.battle_order.validate()?;
        if let Some(queue) = &self.globals.outer_todo_queue {
            queue.validate()?;
        }
        shop::validate(self, content)?;
        if let Some(saved) = &self.confiscated {
            saved.validate()?;
        }
        if let Some(imported) = &self.imported {
            imported.validate()?;
        }
        Ok(())
    }

    /// Checks the loaded items, zero-time and deferred getup state, the map header row (0 to 99),
    /// the weapon traits and the loot quality record.
    fn validate_globals(&self, content: &Content) -> Result<(), RuleError> {
        self.globals.loaded_items.validate()?;
        crate::time::validate_zero(self, content)?;
        crate::combat::runtime::validate_deferred_getup(self)?;
        if !(0..100).contains(&self.globals.header_row) {
            return Err("invalid map header row WORD4427c2".into());
        }
        self.globals.weapon_traits.validate()?;
        if self
            .globals
            .loot_quality
            .as_ref()
            .is_some_and(|quality| quality.len() != LOOT_QUALITY_BYTES)
        {
            return Err(RuleError::Invalid("loot quality record length".into()));
        }
        Ok(())
    }

    /// Checks the sizes and limits of the squad, difficulty (-2 to 2), crate, booty, ammunition,
    /// flags, notes, allies, selection indices, time continuation, spell tables, menus and typed
    /// encounters.
    fn validate_capacities(&self, content: &Content) -> Result<(), RuleError> {
        if self.warriors.len() > 6
            || !(-2..=2).contains(&self.difficulty)
            || self.crate_items.len() != STORED_ITEM_RECORDS
            || self.booty_list.len() != STORED_ITEM_RECORDS
            || self.ammunition.len() != AMMUNITION_SLOTS
            || self.ammunition.iter().any(|v| *v < 0)
            || self.flags.len() != FLAG_BYTES
            || self.notes.len() != NOTE_BYTES
            || self.holdover.len() != story_allies::ALLY_CAPACITY * story_allies::ALLY_BYTES
            || self.held_over > story_allies::ALLY_CAPACITY
            || self.char_select_new >= self.warriors.len().max(1)
            || self.globals.monster_up >= 100
            || self.globals.macro_monster >= 100
            || self.globals.inventory_scroll > 30
            // Only getup's loop sets the round slot.
            || self
                .globals
                .round_scripts
                .slot
                .is_some_and(|slot| !(0..=5).contains(&slot))
            || self.party_condition.len() != PARTY_CONDITION_BYTES
            || self.time_continuation.as_ref().is_some_and(|pending| {
                self.exploration
                    .as_ref()
                    .is_none_or(|position| pending.schedule_cursor > position.schedule.len() / 40)
                    || !(0..=60).contains(&pending.minutes_per_tick)
            })
            || self.globals.define_spells.len() != DEFINE_SPELLS_BYTES
            || self.globals.spell_info.len() != combat::ATTACK_PACKET_BYTES
            || (!self.menus.is_empty()
                && !menu_length(
                    self.menus.len(),
                    content.map(0).map_or(0, |map| map.menus.len()),
                ))
            || (!self.typed_encounters.is_empty()
                && self.typed_encounters.len()
                    != content.map(0).map_or(0, |map| map.typed_encounters.len()))
        {
            return Err(RuleError::Invalid(
                "record counts exceed original capacities".into(),
            ));
        }
        Ok(())
    }

    /// Checks that each warrior's name matches its record and its inventory can be read.
    fn validate_warrior_records(&self) -> Result<(), RuleError> {
        for warrior in &self.warriors {
            if Warrior::read(&warrior.bytes)?.name != warrior.name {
                return Err(RuleError::Invalid(
                    "warrior name differs from record".into(),
                ));
            }
            warrior.inventory()?;
        }
        Ok(())
    }

    /// Checks that no state meant to last for one command has outlived it.
    fn validate_command_scope_flags(&self) -> Result<(), RuleError> {
        if self
            .combat
            .as_ref()
            .is_some_and(|combat| combat.item_condition_caller.is_some())
        {
            return Err("item spell caller escaped its command scope".into());
        }
        if self
            .combat
            .as_ref()
            .is_some_and(|combat| combat.monster_pick_written)
        {
            return Err("monster work copy escaped its command scope".into());
        }
        if self
            .combat
            .as_ref()
            .is_some_and(|c| c.did_attack_write.is_some())
        {
            return Err("NPC inhibit writer escaped its command scope".into());
        }
        Ok(())
    }

    /// Checks that battle state exists exactly in the combat phase, that its difficulty is the
    /// session's, and that it is itself valid.
    fn validate_combat_presence(&self) -> Result<(), RuleError> {
        if self.phase == Phase::Combat {
            if self
                .combat
                .as_ref()
                .is_some_and(|combat| combat.difficulty != self.difficulty)
            {
                return Err(RuleError::Invalid(
                    "combat difficulty differs from the session".into(),
                ));
            }
            self.combat
                .as_ref()
                .ok_or_else(|| RuleError::Invalid("combat state missing".into()))?
                .validate(&self.warriors)?;
        } else if self.combat.is_some() {
            return Err(RuleError::Invalid(
                "combat state present outside battle".into(),
            ));
        }
        Ok(())
    }

    /// Checks the pending frames: the warrior table, promotion, clinic, field controls, scripted
    /// cleanup, battle rewards and treasure, story actor, and turn, monster action and round
    /// dispatch. A pending monster action must wait on getup.
    fn validate_pending_frames(&self, content: &Content) -> Result<(), RuleError> {
        self.globals.warrior_table.validate(self.warriors.len())?;
        if let Some(pending) = &self.promotion {
            pending.validate(self, content)?;
        }
        clinic::validate(self)?;
        field_controls::validate(self)?;
        story_cleanup::validate(self)?;
        if let Some(pending) = &self.ordinary_battle_rewards {
            pending.validate(self)?;
        }
        if let Some(pending) = &self.selected_battle_treasure {
            pending.validate(self)?;
        }
        story_actor::validate(self, content)?;
        if let Some(frame) = &self.turn_dispatch {
            frame.validate(self, content)?;
        }
        if let Some(frame) = &self.monster_action {
            frame.validate()?;
            if !frame.waiting_getup() || self.exploration.is_none() {
                return Err("pending NPC action has no Getup return owner".into());
            }
        }
        if let Some(frame) = &self.round_dispatch {
            frame.validate(self, content)?;
        }
        Ok(())
    }

    /// Checks the pending survivor choice: its records are valid, outside battle a scripted cleanup
    /// stage owns it, and the survivors dialog is showing.
    fn validate_survivors(&self) -> Result<(), RuleError> {
        if let Some(selection) = &self.survivors {
            if let Some(combat) = &self.combat {
                selection.validate(&combat.roster.records)?;
            } else {
                selection.validate(&self.globals.monsters.active_records()?)?;
                if !matches!(
                    self.scripted_cleanup,
                    Some(
                        crate::story::cleanup::Stage::OutOfCombatSurvivors { .. }
                            | crate::story::cleanup::Stage::OutOfCombatSelectedSurvivors { .. }
                            | crate::story::cleanup::Stage::OutOfCombatGetupSurvivors { .. }
                    )
                ) {
                    return Err("retained survivor owner missing".into());
                }
            }
            if !matches!(
                self.exploration
                    .as_ref()
                    .and_then(|p| p.presentation.as_ref()),
                Some(Presentation::Survivors)
            ) {
                return Err(RuleError::Invalid("survivor dialog missing".into()));
            }
        }
        Ok(())
    }

    /// Checks that a battle's pending selection does not coexist with a dialog, that a suspended
    /// round program is valid and has its dialogue or dispatch, and that a round wrap has a
    /// continuation.
    fn validate_round_presentation(&self, content: &Content) -> Result<(), RuleError> {
        if let Some(combat) = &self.combat {
            if combat.selection_pending
                && self
                    .exploration
                    .as_ref()
                    .is_none_or(|position| position.presentation.is_some())
            {
                return Err(RuleError::Invalid(
                    "combat selection conflicts with a dialog".into(),
                ));
            }
            if let Some(program) = &combat.round_macros.pending {
                program.validate(
                    self.map(
                        content,
                        self.exploration.as_ref().ok_or("round map missing")?.map,
                    )
                    .ok_or("round content missing")?,
                )?;
                if combat.battle_order_index != 110
                    || (program.delegated && self.round_dispatch.is_none())
                    || (!program.delegated
                        && !matches!(self.exploration.as_ref().and_then(|p| p.presentation.as_ref()), Some(Presentation::Dialogue { id, .. }) if Some(*id) == program.text))
                {
                    return Err(RuleError::Invalid(
                        "invalid suspended round presentation".into(),
                    ));
                }
            } else if combat.battle_order_index == 110
                && !combat.selection_pending
                && !matches!(
                    self.exploration
                        .as_ref()
                        .and_then(|p| p.presentation.as_ref()),
                    Some(Presentation::Survivors)
                )
            {
                return Err(RuleError::Invalid("round wrap has no continuation".into()));
            }
        }
        Ok(())
    }

    /// Checks the pending target choices: capture, outdoor psi, typed encounter actions, quantity
    /// and the psi selection, whose item must carry the selected spell.
    fn validate_targeting(&self, content: &Content) -> Result<(), RuleError> {
        combat::capture::validate_selection(self)?;
        outdoor_psi::validate(self, content)?;
        typed_encounter::validate_actions(self, content)?;
        if let Some(quantity) = &self.target_quantity {
            quantity.validate(
                self.combat.as_ref().ok_or("targeting combat missing")?,
                &self.warriors,
            )?;
        }
        if let Some(pending) = &self.psi_selection {
            pending.validate(self)?;
            if pending.item.is_some_and(|item| {
                content.items.get(item as usize).is_none_or(|definition| {
                    ItemDefinition(definition).spell().ok() != Some(pending.spell)
                })
            }) {
                return Err("prepared item selection definition differs from its spell".into());
            }
        }
        Ok(())
    }

    /// Checks the saved state of each visited map against its authored map; the active map must
    /// not also be saved there.
    fn validate_visited_maps(&self, content: &Content) -> Result<(), RuleError> {
        for (index, records) in &self.visited_maps {
            records.validate(
                content
                    .map(*index)
                    .ok_or_else(|| RuleError::Invalid("visited map is unavailable".into()))?,
            )?;
            if self
                .exploration
                .as_ref()
                .is_some_and(|position| position.map == *index)
            {
                return Err(RuleError::Invalid(
                    "active map duplicated in visited state".into(),
                ));
            }
        }
        Ok(())
    }

    /// Checks a mission in progress: during exploration or combat, the active map, stage, squad
    /// size, position, fatigue, facing, script, presentation and tables are valid; in squad
    /// selection no mission state may exist.
    fn validate_mission_position(&self, content: &Content) -> Result<(), RuleError> {
        if matches!(self.phase, Phase::Exploration | Phase::Combat) {
            let position = self
                .exploration
                .as_ref()
                .ok_or_else(|| RuleError::Invalid("missing exploration state".into()))?;
            let map = content
                .map(position.map)
                .ok_or_else(|| RuleError::Invalid("active map is unavailable".into()))?;
            position.map_state().validate(map)?;
            if self.mission_stage < 0
                || self.mission_stage >= 20
                || !(1..=6).contains(&self.warriors.len())
                || self.mission.as_deref() != content.scenario.as_ref().map(|s| s.name.as_str())
                || !(0..90).contains(&position.x)
                || !(0..90).contains(&position.y)
                || !(position.fatigue == 1 || (4..=135).contains(&position.fatigue))
                || position.facing > 1
                || position
                    .script
                    .as_ref()
                    .is_some_and(|script| !script.valid(content))
                || (position.presentation.is_some()
                    && position.script.is_none()
                    && !((self.combat_story.is_some()
                        || self
                            .combat
                            .as_ref()
                            .is_some_and(|combat| combat.round_macros.pending.is_some()))
                        && matches!(position.presentation, Some(Presentation::Dialogue { .. })))
                    && !matches!(
                        position.presentation,
                        Some(
                            Presentation::Prologue { .. }
                                | Presentation::Messages { .. }
                                | Presentation::Survivors
                                | Presentation::Loot
                                | Presentation::Avoidance { .. }
                                | Presentation::BoatQuestion { .. }
                                | Presentation::Shop
                                | Presentation::Clinic { .. }
                        )
                    ))
                || position.event_headers.len() != EVENT_HEADER_BYTES
                || position.schedule.len() != content.map(0).map_or(0, |map| map.schedule.len())
                || !position.schedule.len().is_multiple_of(40)
                || position.region_state.len() != REGION_BYTES
                || matches!(
                    position.presentation,
                    Some(Presentation::Avoidance { region: 20.., .. })
                )
            {
                return Err(RuleError::Invalid("invalid mission state".into()));
            }
        } else if self.exploration.is_some()
            || self.mission.is_some()
            || self.movement_continuation.is_some()
        {
            return Err(RuleError::Invalid(
                "mission state present during squad selection".into(),
            ));
        }
        Ok(())
    }

    /// Checks the saved tables and maps of each level the party has left; the active level must
    /// not also be saved there.
    fn validate_visited_levels(&self, content: &Content) -> Result<(), RuleError> {
        if self.visited_levels.contains_key(&self.scenario_level) {
            return Err(RuleError::Invalid(
                "active level duplicated in visited state".into(),
            ));
        }
        for (level, records) in &self.visited_levels {
            let context = content.context(self.mission.as_deref(), *level)?;
            let map = context.map(0).ok_or("visited level unavailable")?;
            if records.schedule.len() != map.schedule.len()
                || (!records.menus.is_empty() && !menu_length(records.menus.len(), map.menus.len()))
                || (!records.typed_encounters.is_empty()
                    && records.typed_encounters.len() != map.typed_encounters.len())
            {
                return Err(RuleError::Invalid("invalid persistent level tables".into()));
            }
            for (index, records) in &records.maps {
                records.validate(context.map(*index).ok_or("visited level map unavailable")?)?;
            }
        }
        Ok(())
    }

    /// The authored map with the given number, for the current mission and level.
    pub fn map<'a>(
        &self,
        content: &'a Content,
        index: i32,
    ) -> Option<&'a centurions_content::mission::OpeningMission> {
        content.map_at(self.mission.as_deref(), self.scenario_level, index)
    }

    /// The current land under its current tile set: loadland reloads the tile
    /// properties with the land, so a story's tile-set switch follows it.
    ///
    /// Original: loadland
    pub fn land<'a>(
        &self,
        content: &'a Content,
    ) -> Option<&'a centurions_content::mission::OpeningMission> {
        Self::land_in(
            content,
            self.mission.as_deref(),
            self.scenario_level,
            self.exploration.as_ref(),
        )
    }

    /// The original quits when a land names a tile set the installation cannot
    /// supply, so a land change is refused before it alters the session.
    ///
    /// Original: loadland
    pub(crate) fn check_land_tile_set(
        &self,
        content: &Content,
        level: u8,
        map: i32,
    ) -> Result<(), RuleError> {
        let saved = if level == self.scenario_level {
            self.visited_maps.get(&map)
        } else {
            self.visited_levels
                .get(&level)
                .and_then(|saved| saved.maps.get(&map))
        };
        match saved.and_then(|records| RegionTable(&records.regions).stored_terrain()) {
            Some(terrain)
                if content
                    .land_at(self.mission.as_deref(), level, map, terrain)
                    .is_none() =>
            {
                Err(RuleError::Unsupported {
                    action: "exploration.land",
                    dependency: "land tile set without an installed BD file quits the original",
                })
            }
            _ => Ok(()),
        }
    }

    /// Session::land from its fields, for callers holding another field mutably.
    pub fn land_in<'a>(
        content: &'a Content,
        mission: Option<&str>,
        level: u8,
        exploration: Option<&crate::Exploration>,
    ) -> Option<&'a centurions_content::mission::OpeningMission> {
        let Some(position) = exploration else {
            return content.map_at(mission, level, 0);
        };
        match RegionTable(&position.region_state).stored_terrain() {
            Some(terrain) => content.land_at(mission, level, position.map, terrain),
            None => content.map_at(mission, level, position.map),
        }
    }
}
