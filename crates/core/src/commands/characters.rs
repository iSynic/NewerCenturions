use crate::{
    Command, Event, Phase, RuleError, Session, ammunition, character, combat, creation,
    editor_items, feedback, initial_psi_memory, roster, story_promotion,
    warrior_management_available,
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
            state.psi_memory = Some(initial_psi_memory(&state.warriors)?);
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
            state.warriors = vec![warrior];
            state.psi_memory = Some(initial_psi_memory(&state.warriors)?);
            Ok(vec![Event::Sound { resource_id: 130 }])
        }
        Command::SelectWarriors { warriors } => {
            if state.phase != Phase::SquadSelection {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            roster::validate_squad(&warriors)?;
            state.warriors = warriors;
            state.psi_memory = Some(initial_psi_memory(&state.warriors)?);
            for warrior in &state.warriors {
                state.loaded_item_field32 =
                    Some(combat::ally_turns::item_context_after_weight_calc(
                        warrior,
                        &content.items,
                        state.loaded_item_field32.unwrap_or(0),
                    )?);
            }
            state.ammunition_work.item = None;
            Ok(vec![Event::StartersImported {
                count: state.warriors.len(),
            }])
        }
        Command::ReplaceSquad { warriors } => {
            roster::admit_management(state)?;
            roster::validate_squad(&warriors)?;
            state.warriors = warriors;
            state.selected_warrior = 0;
            state.psi_memory = Some(initial_psi_memory(&state.warriors)?);
            state.psi_menu_choices = std::array::from_fn(|_| None);
            state.health_context = None;
            state.loaded_item_field32 = None;
            state.ammunition_work = ammunition::WorkBuffers::default();
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
            for (member, destination) in destinations.into_iter().enumerate() {
                state.warriors[destination] = before[member].clone();
                state.psi_menu_choices[destination] = choices[member].clone();
            }
            state.psi_memory = Some(initial_psi_memory(&state.warriors)?);
            state.health_context = None;
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
            if !free_upgrade {
                let map = content
                    .map(state.exploration.as_ref().map_or(0, |p| p.map))
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
            Ok(vec![
                Event::Sound { resource_id: 10129 },
                Event::StatisticsCalculated { warrior },
            ])
        }
        Command::ViewWarrior { warrior } => {
            if !(warrior_management_available(state)
                || state.phase == Phase::Combat && combat::controls::actor(state).is_ok())
            {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            state.warriors.get(warrior).ok_or_else(|| {
                RuleError::Invalid("information warrior is outside the squad".into())
            })?;
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
            state.warriors = indices
                .iter()
                .map(|i| content.warriors[*i].clone())
                .collect();
            state.psi_memory = Some(initial_psi_memory(&state.warriors)?);
            for warrior in &state.warriors {
                state.loaded_item_field32 =
                    Some(combat::ally_turns::item_context_after_weight_calc(
                        warrior,
                        &content.items,
                        state.loaded_item_field32.unwrap_or(0),
                    )?);
            }
            state.ammunition_work.item = None;
            Ok(vec![Event::StartersImported {
                count: state.warriors.len(),
            }])
        }
        _ => unreachable!("command routed to wrong subsystem"),
    }
}
