use crate::{
    Command, Event, Phase, RuleError, Session, character, combat, creation, editor_items, feedback,
    initial_psi_memory, roster, story_promotion, warrior_management_available,
};
use centurions_content::Content;

pub(super) fn execute(
    command: Command,
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    let free_upgrade = matches!(command, Command::EditWarriorUpgrade { .. });
    match command {
        Command::CreationDialog => {
            if state.phase != Phase::SquadSelection {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            Ok(vec![Event::Sound {
                resource_id: state.random.bound(2).wrapping_add(200),
            }])
        }
        Command::CreationEquipment => {
            if state.phase != Phase::SquadSelection || state.warriors.len() != 1 {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            creation::initial_items(state, content)
        }
        Command::CreationScans => {
            if state.phase != Phase::SquadSelection || state.warriors.len() != 1 {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            state.warriors[0].record_mut().clear_learned_scans();
            state.native_work.psi_memory = initial_psi_memory(&state.warriors)?;
            Ok(vec![])
        }
        Command::GenerateWarrior { request } => {
            if state.phase != Phase::SquadSelection || state.warriors.len() > 1 {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            let warrior = creation::generate(
                &request,
                content,
                &mut state.random,
                &mut state.library_random_seed,
                &mut state.library_random_draws,
            )?;
            state.native_work.character_bare_hand_bonus = warrior.record().bare_hand_bonus()?;
            state.replace_squad(vec![warrior])?;
            state.native_work.psi_memory = initial_psi_memory(&state.warriors)?;
            Ok(vec![Event::Sound { resource_id: 130 }])
        }
        Command::SelectWarriors { warriors } => {
            if state.phase != Phase::SquadSelection {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            roster::validate_squad(&warriors)?;
            state.replace_squad(warriors)?;
            state.native_work.psi_memory = initial_psi_memory(&state.warriors)?;
            for warrior in &state.warriors {
                state.native_work.loaded_item_field32 =
                    combat::ally_turns::item_context_after_weight_calc(
                        warrior,
                        &content.items,
                        state.native_work.loaded_item_field32,
                    )?;
            }
            weight_calc_item_work(state, content)?;
            Ok(vec![Event::StartersImported {
                count: state.warriors.len(),
            }])
        }
        Command::ReplaceSquad { warriors } => {
            roster::admit_management(state)?;
            roster::validate_squad(&warriors)?;
            // partyselect1476cc clears BYTE44273f (1478dc) and moves BYTE442739 by one for each
            // unavailable member it removes (1489d8) or adds (148f7e).
            let context = &mut state.native_work.health_context;
            let delta = unavailable_members(&warriors) - unavailable_members(&state.warriors);
            context.unavailable = context.unavailable.wrapping_add(delta as i8);
            context.selected_warrior = 0;
            state.replace_squad(warriors)?;
            state.selected_warrior = 0;
            state.native_work.psi_memory = initial_psi_memory(&state.warriors)?;
            // partyselect clears the menu BYTEs 0x442580 but not the power WORDs.
            let words = std::array::from_fn(|member| state.psi_power(member));
            state.psi_menu_choices = std::array::from_fn(|_| None);
            state.keep_psi_powers(words);
            // partyselect Done (148206..148366) preserves the item/ammo/weapon
            // work left by earlier actions; it does not reload the final squad.
            Ok(vec![])
        }
        Command::ReorderSquad { destinations } => {
            roster::admit_management(state)?;
            if destinations.len() != state.warriors.len()
                || destinations.iter().any(|i| *i >= destinations.len())
                || (0..destinations.len()).any(|i| destinations[..i].contains(&destinations[i]))
            {
                return Err(RuleError::Invalid(
                    "squad order is not a permutation".into(),
                ));
            }
            let before = state.warriors.clone();
            let choices = state.psi_menu_choices.clone();
            let words = std::array::from_fn(|member| state.psi_power(member));
            for (member, destination) in destinations.into_iter().enumerate() {
                state.warriors[destination] = before[member].clone();
                state.psi_menu_choices[destination] = choices[member].clone();
            }
            // The power WORDs 0x437a7a stay with their member slots.
            state.keep_psi_powers(words);
            state.native_work.psi_memory = initial_psi_memory(&state.warriors)?;
            Ok(vec![])
        }
        Command::EditWarriorScans { warrior, controls } => {
            if !warrior_management_available(state) || controls.len() > 4096 {
                return Err(RuleError::Invalid(
                    "scan editing requires bounded exploration input".into(),
                ));
            }
            let map = content
                .map(0)
                .ok_or_else(|| RuleError::Invalid("editor profiles unavailable".into()))?;
            let record = state
                .warriors
                .get(warrior)
                .ok_or_else(|| RuleError::Invalid("editor warrior is outside the squad".into()))?;
            let mut selection = story_promotion::ScanSelection::new(record, &map.profiles)?;
            let mut events = vec![Event::Sound { resource_id: 3000 }];
            if !matches!(controls.last(), Some(1 | 32)) {
                return Err(RuleError::Invalid("scan selection requires Done".into()));
            }
            for control in &controls[..controls.len() - 1] {
                match control {
                    2..=8 => {
                        selection.select_tier(usize::from(*control - 2))?;
                        events.push(Event::Sound { resource_id: 137 });
                    }
                    10..=16 => {
                        if let Some(resource_id) = selection.toggle(usize::from(*control - 10))? {
                            events.push(Event::Sound { resource_id });
                        }
                    }
                    _ => return Err(RuleError::Invalid("unknown scan-selection control".into())),
                }
            }
            let attacks = &map
                .combat
                .as_ref()
                .ok_or_else(|| RuleError::Invalid("scan records unavailable".into()))?
                .attacks;
            state.warriors[warrior] = selection.finish(attacks)?;
            story_promotion::cache_scans(state, warrior, attacks)?;
            state.native_work.character_bare_hand_bonus =
                state.warriors[warrior].record().bare_hand_bonus()?;
            Ok(events)
        }
        Command::EditWarriorLevel { warrior } => {
            if !warrior_management_available(state) {
                return Err(RuleError::Invalid(
                    "warrior editing requires exploration".into(),
                ));
            }
            let map = content
                .map(0)
                .ok_or_else(|| RuleError::Invalid("editor profiles unavailable".into()))?;
            let record = state
                .warriors
                .get_mut(warrior)
                .ok_or_else(|| RuleError::Invalid("editor warrior is outside the squad".into()))?;
            let mut random = story_promotion::LibraryRandom {
                seed: state.library_random_seed,
                draws: state.library_random_draws,
            };
            let information =
                story_promotion::editor_level_up(record, &map.races, &map.profiles, &mut random)?;
            state.native_work.character_bare_hand_bonus = record.record().bare_hand_bonus()?;
            state.library_random_seed = random.seed;
            state.library_random_draws = random.draws;
            Ok(vec![
                Event::Sound { resource_id: 133 },
                Event::EditorLevelAdvanced {
                    warrior,
                    level: record.record().level()?,
                    information,
                },
            ])
        }
        Command::EditCarriedItem {
            warrior,
            slot,
            control,
        } => editor_items::edit(state, content, warrior, slot, control),
        Command::SetWarriorAppearance {
            warrior,
            portrait,
            resource,
        } => {
            if !warrior_management_available(state) {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            let owner = if portrait { "/Portraits" } else { "/Tacticals" };
            let first = if portrait { 257 } else { 9257 };
            if !(first..first + 120).contains(&resource)
                || !content
                    .resources
                    .iter()
                    .any(|r| r.original.ends_with(owner) && r.r#type == "cicn" && r.id == resource)
            {
                return Err(RuleError::Invalid(
                    "warrior appearance resource is unavailable".into(),
                ));
            }
            let record = state.warriors.get_mut(warrior).ok_or_else(|| {
                RuleError::Invalid("appearance warrior is outside the squad".into())
            })?;
            record.record_mut().set_appearance(portrait, resource);
            Ok(vec![Event::Sound { resource_id: 130 }])
        }
        Command::EditWarriorUpgrade { warrior, selector }
        | Command::UpgradeWarrior { warrior, selector } => {
            if !warrior_management_available(state) || selector > 32 {
                return Err(RuleError::Invalid(format!(
                    "warrior editing requires exploration and a selector from 0 through 32 (received {selector})"
                )));
            }
            let record = state
                .warriors
                .get_mut(warrior)
                .ok_or_else(|| RuleError::Invalid("editor warrior is outside the squad".into()))?;
            if !record.record().upgrade(selector).can_increase()? {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            if record.record().upgrade_count(selector)? < 0 {
                return Err(RuleError::Refused {
                    feedback: vec![feedback::NativeFeedback::Warning { index: 149 }],
                });
            }
            // The purchase copies the record to 0x44d42c before paying (0x163258..0x163276).
            let scratch = record.bytes.clone();
            if !free_upgrade {
                let map = crate::Session::land_in(
                    content,
                    state.mission.as_deref(),
                    state.scenario_level,
                    state.exploration.as_ref(),
                )
                .ok_or_else(|| RuleError::Invalid("upgrade MOS tables missing".into()))?;
                let mos = record.record().profession()?;
                let profile = centurions_content::span(
                    &map.profiles,
                    usize::try_from(mos.wrapping_sub(1))
                        .map_err(|_| RuleError::Invalid("upgrade MOS index".into()))?
                        * 576,
                    576,
                )?;
                let Some(points) = character::upgrade_payment(record, selector, profile)? else {
                    return Err(RuleError::Refused { feedback: vec![] });
                };
                record.record_mut().set_upgrade_points(points);
            }
            let count = record.record().upgrade_count(selector)?.wrapping_add(1);
            record.record_mut().set_upgrade_count(selector, count);
            character::apply_upgrade(record, selector, 1)?;
            character::calculate_statistics(record)?;
            state.native_work.warrior_table.store(6, scratch)?;
            Ok(vec![
                Event::Sound { resource_id: 10129 },
                Event::StatisticsCalculated { warrior },
            ])
        }
        Command::EnterWarriorUpgrade { warrior } => {
            if warrior >= state.warriors.len() {
                return Err(RuleError::Invalid(
                    "editor warrior is outside the squad".into(),
                ));
            }
            // 0x162376..0x1623e2: calcw, calcstats, BYTE44273f, then the record to character
            // work 0x44721c (bare-hand WORD 0x447d96), 0x44e06e and 0x44d42c.
            crate::items::calcw(state, content, warrior)?;
            character::calculate_statistics(&mut state.warriors[warrior])?;
            state.native_work.health_context.selected_warrior = warrior;
            let record = state.warriors[warrior].bytes.clone();
            state.native_work.character_bare_hand_bonus =
                state.warriors[warrior].record().bare_hand_bonus()?;
            let table = &mut state.native_work.warrior_table;
            table.store(7, record.clone())?;
            table.store(6, record)?;
            Ok(vec![])
        }
        Command::ViewWarrior { warrior } => {
            if !(warrior_management_available(state)
                || state.phase == Phase::Combat && combat::controls::actor(state).is_ok())
            {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            let record = state.warriors.get(warrior).ok_or_else(|| {
                RuleError::Invalid("information warrior is outside the squad".into())
            })?;
            state.native_work.character_bare_hand_bonus = record.record().bare_hand_bonus()?;
            if state.combat.is_some() {
                // Combatchoice11b766 restores the tactical selector after the view.
                state.set_warrior_selector(state.selected_warrior);
            }
            let mut presentation_random = state.random.clone();
            let first = presentation_random.bound(5) as usize - 1;
            let second = presentation_random.bound(5) as usize - 1;
            let sounds = [200, 201, 202, 207, 208];
            let mut events = vec![Event::Sound {
                resource_id: sounds[first],
            }];
            if first != second {
                events.push(Event::Sound {
                    resource_id: sounds[second],
                });
            }
            Ok(events)
        }
        Command::RecalculateStatistics { warrior } => {
            let record = state.warriors.get_mut(warrior).ok_or_else(|| {
                RuleError::Invalid("statistics warrior is outside the squad".into())
            })?;
            character::calculate_statistics(record)?;
            Ok(vec![Event::StatisticsCalculated { warrior }])
        }
        Command::ImportStarters | Command::SelectStarters { .. } => {
            if state.phase != Phase::SquadSelection {
                return Err(RuleError::Invalid("squad already entered a mission".into()));
            }
            let indices = match command {
                Command::SelectStarters { indices } => indices,
                _ => (0..content.warriors.len()).collect(),
            };
            if !(1..=6).contains(&indices.len())
                || indices.iter().any(|i| *i >= content.warriors.len())
                || (0..indices.len()).any(|i| indices[..i].contains(&indices[i]))
            {
                return Err(RuleError::Invalid(
                    "select one to six distinct supplied warriors".into(),
                ));
            }
            state.replace_squad(
                indices
                    .iter()
                    .map(|i| content.warriors[*i].clone())
                    .collect(),
            )?;
            state.native_work.psi_memory = initial_psi_memory(&state.warriors)?;
            for warrior in &state.warriors {
                state.native_work.loaded_item_field32 =
                    combat::ally_turns::item_context_after_weight_calc(
                        warrior,
                        &content.items,
                        state.native_work.loaded_item_field32,
                    )?;
            }
            weight_calc_item_work(state, content)?;
            Ok(vec![Event::StartersImported {
                count: state.warriors.len(),
            }])
        }
        _ => unreachable!("command routed to wrong subsystem"),
    }
}

/// partyselect counts a member with stamina below one or an automatic-mode WORD as unavailable.
fn unavailable_members(warriors: &[centurions_content::Warrior]) -> i32 {
    warriors
        .iter()
        .filter(|warrior| {
            warrior.stamina().unwrap_or(0) < 1
                || warrior.record().automatic_mode().unwrap_or(0) != 0
        })
        .count() as i32
}

/// calcweight loads every carried item through ITEM work, which keeps the last one.
fn weight_calc_item_work(state: &mut Session, content: &Content) -> Result<(), RuleError> {
    for warrior in &state.warriors {
        if let Some(last) = warrior.inventory()?.last() {
            state.native_work.ammunition_work.item =
                Some(crate::ammunition::definition(content, last.id())?);
        }
    }
    Ok(())
}
