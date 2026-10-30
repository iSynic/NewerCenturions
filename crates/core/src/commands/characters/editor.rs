//! The warrior editor: scans, levels, carried items, appearance and upgrades.
use crate::{
    Command, Event, RuleError, Session, character, editor_items, feedback, story_promotion,
    warrior_management_available,
};
use centurions_content::Content;
use centurions_content::records::ProfileTable;

/// Runs the warrior editor commands: scans, levels, carried items, appearance and upgrades.
pub(in crate::commands) fn execute(
    command: Command,
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    match command {
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
            state.globals.character_bare_hand_bonus =
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
            state.globals.character_bare_hand_bonus = record.record().bare_hand_bonus()?;
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
        Command::EditWarriorUpgrade { warrior, selector } => {
            upgrade(state, content, warrior, selector, true)
        }
        Command::UpgradeWarrior { warrior, selector } => {
            upgrade(state, content, warrior, selector, false)
        }
        Command::EnterWarriorUpgrade { warrior } => {
            if warrior >= state.warriors.len() {
                return Err(RuleError::Invalid(
                    "editor warrior is outside the squad".into(),
                ));
            }
            // Entering the upgrade screen recalculates weight and statistics,
            // selects the warrior, then copies the record into the character
            // work record and both Upgrade records.
            crate::items::calcw(state, content, warrior)?;
            character::calculate_statistics(&mut state.warriors[warrior])?;
            state.globals.health_context.char_select_new = warrior;
            let record = state.warriors[warrior].bytes.clone();
            state.globals.character_bare_hand_bonus =
                state.warriors[warrior].record().bare_hand_bonus()?;
            let table = &mut state.globals.warrior_table;
            table.store(crate::UPGRADE_ENTRY_BODY, record.clone())?;
            table.store(crate::UPGRADE_PREVIOUS_BODY, record)?;
            Ok(vec![])
        }
        _ => unreachable!("command routed to wrong subsystem"),
    }
}

/// An upgrade purchase; the editor's upgrades are free.
fn upgrade(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    selector: u16,
    free_upgrade: bool,
) -> Result<Vec<Event>, RuleError> {
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
            feedback: vec![feedback::GameFeedback::Warning { index: 149 }],
        });
    }
    // The purchase copies the record to the left Upgrade record before paying.
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
        let profile = ProfileTable::professions(&map.profiles).row(
            usize::try_from(mos.wrapping_sub(1))
                .map_err(|_| RuleError::Invalid("upgrade MOS index".into()))?,
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
    state
        .globals
        .warrior_table
        .store(crate::UPGRADE_PREVIOUS_BODY, scratch)?;
    Ok(vec![
        Event::Sound { resource_id: 10129 },
        Event::StatisticsCalculated { warrior },
    ])
}
