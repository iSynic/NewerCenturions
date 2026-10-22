use crate::{
    CommandPreferences, MedicalOrigin, NativeWorkContext, Phase, Presentation, RandomState,
    RuleError, Session, clinic, combat, empty_allies, empty_ammunition, empty_notes,
    empty_party_buffs, field_controls, initial_flags, menu_length, outdoor_psi, shop, story_actor,
    story_allies, story_cleanup, typed_encounter,
};
use centurions_content::{Content, InventoryRecord, Warrior};

impl Session {
    pub(crate) fn load_item_work(&mut self, content: &Content, id: i16) -> Result<(), RuleError> {
        self.native_work.loaded_item_field32 =
            Some(self.native_work.ammunition_work.load_item(content, id)?);
        Ok(())
    }
    pub fn new(content: &Content) -> Self {
        Self {
            content_identity: content.identity.clone(),
            difficulty: 0,
            command_preferences: CommandPreferences::default(),
            warriors: vec![],
            phase: Phase::SquadSelection,
            mission: None,
            scenario_level: 0,
            visited_levels: Default::default(),
            native_import: None,
            shops: Default::default(),
            shop_access: None,
            field_controls: None,
            confiscated: None,
            suppress_allies: 0,
            loot_limit: None,
            typed_encounters: vec![],
            exploration: None,
            random: RandomState::new(12),
            crate_items: vec![InventoryRecord([0; 22]); 1000],
            booty_items: vec![InventoryRecord([0; 22]); 1000],
            ammunition: empty_ammunition(),
            combat: None,
            battle_treasure: 0,
            battle_after_message: 0,
            battle_group_script: None,
            battle_npc: None,
            selected_warrior: 0,
            native_work: NativeWorkContext::startup(),
            party_buffs: empty_party_buffs(),
            scenario_duration_tail: None,
            psi_disabled: 0,
            enemy_spell_disabled: 0,
            energy_recovery_disabled: 0,
            party_currency: 0,
            library_random_seed: 1,
            library_random_draws: 0,
            combat_story: None,
            survivors: None,
            round_dispatch: None,
            scripted_cleanup: None,
            actor_replacement: None,
            movement_continuation: None,
            time_continuation: None,
            psi_selection: None,
            target_quantity: None,
            outdoor_psi: None,
            encounter_actions: None,
            promotion: None,
            medical_continuation: None,
            psi_menu_choices: Default::default(),
            bandage_ready: None,
            handled_deaths: vec![],
            menus: vec![],
            flags: initial_flags(),
            mission_score: 0,
            mission_stage: 0,
            map_picture: 0,
            notes: empty_notes(),
            saved_position: [0; 3],
            visited_maps: std::collections::BTreeMap::new(),
            allies: empty_allies(),
            ally_count: 0,
        }
    }

    pub fn validate(&self, content: &Content) -> Result<(), RuleError> {
        self.native_work.npc_backing.validate()?;
        if let Some(queue) = &self.native_work.retained_death_queue {
            queue.validate()?;
        }
        shop::validate(self, content)?;
        if let Some(saved) = &self.confiscated {
            saved.validate()?;
        }
        if let Some(imported) = &self.native_import {
            imported.validate()?;
        }
        let context = content.context(self.mission.as_deref(), self.scenario_level)?;
        let content = context.as_ref();
        self.native_work.ammunition_work.validate()?;
        self.native_work.weapon_work.validate()?;
        if self
            .native_work
            .loot_quality
            .as_ref()
            .is_some_and(|quality| quality.len() != 1000)
        {
            return Err(RuleError::Invalid("loot quality record length".into()));
        }
        if self.content_identity != content.identity {
            return Err(RuleError::Invalid("content identity mismatch".into()));
        }
        if self.warriors.len() > 6
            || !(-2..=2).contains(&self.difficulty)
            || self.crate_items.len() != 1000
            || self.booty_items.len() != 1000
            || self.ammunition.len() != 200
            || self.ammunition.iter().any(|v| *v < 0)
            || self.flags.len() != 128
            || self.notes.len() != 19
            || self.allies.len() != story_allies::ALLY_CAPACITY * story_allies::ALLY_BYTES
            || self.ally_count > story_allies::ALLY_CAPACITY
            || self.selected_warrior >= self.warriors.len().max(1)
            || self.native_work.selected_enemy >= 100
            || self.native_work.script_enemy >= 100
            || self.party_buffs.len() != 20
            || self.time_continuation.as_ref().is_some_and(|pending| {
                self.exploration
                    .as_ref()
                    .is_none_or(|position| pending.schedule_cursor > position.schedule.len() / 40)
                    || !(0..=60).contains(&pending.minutes_per_tick)
            })
            || self
                .native_work
                .psi_memory
                .as_ref()
                .is_some_and(|memory| memory.len() != 480)
            || self
                .native_work
                .attack_packet
                .as_ref()
                .is_some_and(|packet| packet.len() != 58)
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
        for warrior in &self.warriors {
            if Warrior::read(&warrior.bytes)?.name != warrior.name {
                return Err(RuleError::Invalid(
                    "warrior name differs from record".into(),
                ));
            }
            warrior.inventory()?;
        }
        if self.medical_continuation.as_ref().is_some_and(|pending| {
            let descriptor = match pending.origin {
                MedicalOrigin::OwnedNeedle => [0, 3, 0],
                MedicalOrigin::LearnedHeal => [0, 0, 0],
            };
            let valid_identity = match pending.origin {
                MedicalOrigin::OwnedNeedle => {
                    (700..=703).contains(&pending.item)
                        && pending.spell == 1401
                        && pending.power == 1
                }
                MedicalOrigin::LearnedHeal => {
                    pending.item == 0 && pending.spell == 1101 && (1..=7).contains(&pending.power)
                }
            };
            self.phase != Phase::Combat
                || pending.actor >= self.warriors.len()
                || !valid_identity
                || self.native_work.attack_count != pending.power
                || self.combat.as_ref().is_none_or(|arena| {
                    arena.active_body() != Some(pending.actor as i16)
                        || arena.attack_identity != Some(descriptor)
                        || arena.attack_packet.as_ref().is_none_or(|packet| {
                            packet.len() != 58
                                || centurions_content::word(packet, 40).ok() != Some(57)
                        })
                })
        }) {
            return Err(RuleError::Invalid("invalid medical continuation".into()));
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
        if self.handled_deaths.iter().any(|i| {
            self.combat
                .as_ref()
                .is_none_or(|c| *i >= c.enemies.records.len())
        }) {
            return Err(RuleError::Invalid("invalid combat story state".into()));
        }
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
        if let Some(pending) = &self.promotion {
            pending.validate(self, content)?;
        }
        clinic::validate(self)?;
        field_controls::validate(self)?;
        story_cleanup::validate(self)?;
        story_actor::validate(self, content)?;
        if let Some(frame) = &self.round_dispatch {
            frame.validate(self, content)?;
        }
        if let Some(selection) = &self.survivors {
            selection.validate(
                &self
                    .combat
                    .as_ref()
                    .ok_or("survivor roster missing")?
                    .enemies
                    .records,
            )?;
            if !matches!(
                self.exploration
                    .as_ref()
                    .and_then(|p| p.presentation.as_ref()),
                Some(Presentation::Survivors)
            ) {
                return Err(RuleError::Invalid("survivor dialog missing".into()));
            }
        }
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
            if let Some(program) = &combat.round_scripts.pending {
                program.validate(
                    self.map(
                        content,
                        self.exploration.as_ref().ok_or("round map missing")?.map,
                    )
                    .ok_or("round content missing")?,
                )?;
                if combat.cursor != 110
                    || (program.delegated && self.round_dispatch.is_none())
                    || (!program.delegated
                        && !matches!(self.exploration.as_ref().and_then(|p| p.presentation.as_ref()), Some(Presentation::Dialogue { id, .. }) if Some(*id) == program.text))
                {
                    return Err(RuleError::Invalid(
                        "invalid suspended round presentation".into(),
                    ));
                }
            } else if combat.cursor == 110
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
        }
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
                || !(0..24).contains(&position.hour)
                || !(0..60).contains(&position.minute)
                || position.day < 1
                || !(4..=135).contains(&position.fatigue)
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
                            .is_some_and(|combat| combat.round_scripts.pending.is_some()))
                        && matches!(position.presentation, Some(Presentation::Dialogue { .. })))
                    && !matches!(
                        position.presentation,
                        Some(
                            Presentation::Prologue { .. }
                                | Presentation::Messages { .. }
                                | Presentation::Survivors
                                | Presentation::Loot
                                | Presentation::Avoidance { .. }
                                | Presentation::Shop
                                | Presentation::Clinic { .. }
                        )
                    ))
                || position.event_headers.len() != 4200
                || position.schedule.len() != content.map(0).map_or(0, |map| map.schedule.len())
                || !position.schedule.len().is_multiple_of(40)
                || position.region_state.len() != 644
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

    pub fn map<'a>(
        &self,
        content: &'a Content,
        index: i32,
    ) -> Option<&'a centurions_content::mission::OpeningMission> {
        content.map_at(self.mission.as_deref(), self.scenario_level, index)
    }

    /// The current land under its live tile set (region-table byte 520): loadland reloads
    /// the tile properties with the land, so a story's tile-set switch follows it.
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

    /// loadland's loadpixmap quits when a land names a tile set the installation cannot
    /// supply, so a land change is refused before it alters the session.
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
        match saved.and_then(|records| records.regions.get(0x208)) {
            Some(&terrain)
                if content
                    .land_at(self.mission.as_deref(), level, map, terrain)
                    .is_none() =>
            {
                Err(RuleError::Unsupported {
                    action: "exploration.land",
                    dependency: "land tile set outside the installed BD files",
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
        match position.region_state.get(0x208) {
            Some(&terrain) => content.land_at(mission, level, position.map, terrain),
            None => content.map_at(mission, level, position.map),
        }
    }
}
