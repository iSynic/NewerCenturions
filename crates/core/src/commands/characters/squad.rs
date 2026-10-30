//! Warrior creation, starting-squad import and squad management.
use crate::{
    Command, Event, Phase, RuleError, Session, combat, creation, initial_psi_memory, roster,
};
use centurions_content::Content;

/// Runs warrior creation, starting-squad import and squad management commands.
pub(in crate::commands) fn execute(
    command: Command,
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
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
            state.globals.define_spells = initial_psi_memory(&state.warriors)?;
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
            state.globals.character_bare_hand_bonus = warrior.record().bare_hand_bonus()?;
            state.replace_squad(vec![warrior])?;
            state.globals.define_spells = initial_psi_memory(&state.warriors)?;
            Ok(vec![Event::Sound { resource_id: 130 }])
        }
        Command::SelectWarriors { warriors } => {
            if state.phase != Phase::SquadSelection {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            roster::validate_squad(&warriors)?;
            install_starting_squad(state, content, warriors)
        }
        Command::ReplaceSquad { warriors } => {
            roster::admit_management(state)?;
            roster::validate_squad(&warriors)?;
            // partyselect clears `char_select_new` and moves `kill_party` by one
            // for each unavailable member it removes or adds.
            let context = &mut state.globals.health_context;
            let delta = unavailable_members(&warriors) - unavailable_members(&state.warriors);
            context.kill_party = context.kill_party.wrapping_add(delta as i8);
            context.char_select_new = 0;
            state.replace_squad(warriors)?;
            state.char_select_new = 0;
            state.globals.define_spells = initial_psi_memory(&state.warriors)?;
            // partyselect clears the Psi menu choices but not the remembered powers.
            let words = std::array::from_fn(|member| state.psi_power(member));
            state.psi_menu_choices = std::array::from_fn(|_| None);
            state.keep_psi_powers(words);
            // Done keeps the item, ammunition and weapon left by earlier
            // actions; it does not reload the final squad.
            Ok(vec![])
        }
        Command::ReorderSquad { destinations } => {
            roster::admit_management(state)?;
            if destinations.len() != state.warriors.len()
                || destinations.iter().any(|i| *i >= destinations.len())
                || has_repeated_index(&destinations)
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
            // Remembered Psi powers stay with their member slots.
            state.keep_psi_powers(words);
            state.globals.define_spells = initial_psi_memory(&state.warriors)?;
            Ok(vec![])
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
                || has_repeated_index(&indices)
            {
                return Err(RuleError::Invalid(
                    "select one to six distinct supplied warriors".into(),
                ));
            }
            install_starting_squad(
                state,
                content,
                indices
                    .iter()
                    .map(|i| content.warriors[*i].clone())
                    .collect(),
            )
        }
        _ => unreachable!("command routed to wrong subsystem"),
    }
}

/// Installs a selected or imported starting squad with its Psi memory and the
/// item work its weight calculation leaves behind.
fn install_starting_squad(
    state: &mut Session,
    content: &Content,
    warriors: Vec<centurions_content::Warrior>,
) -> Result<Vec<Event>, RuleError> {
    state.replace_squad(warriors)?;
    state.globals.define_spells = initial_psi_memory(&state.warriors)?;
    for warrior in &state.warriors {
        state.globals.loaded_item_field32 = combat::ally_turns::item_context_after_weight_calc(
            warrior,
            &content.items,
            state.globals.loaded_item_field32,
        )?;
    }
    calc_weight_item(state, content)?;
    Ok(vec![Event::StartersImported {
        count: state.warriors.len(),
    }])
}

/// Whether any index repeats an earlier one.
fn has_repeated_index(indices: &[usize]) -> bool {
    (0..indices.len()).any(|i| indices[..i].contains(&indices[i]))
}

/// partyselect counts a member with stamina below one, or on automatic, as
/// unavailable.
fn unavailable_members(warriors: &[centurions_content::Warrior]) -> i32 {
    warriors
        .iter()
        .filter(|warrior| {
            warrior.stamina().unwrap_or(0) < 1
                || warrior.record().computer_control().unwrap_or(0) != 0
        })
        .count() as i32
}

/// calcweight loads every carried item in turn, so the last one stays loaded.
fn calc_weight_item(state: &mut Session, content: &Content) -> Result<(), RuleError> {
    for warrior in &state.warriors {
        if let Some(last) = warrior.inventory()?.last() {
            state.globals.loaded_items.item =
                Some(crate::ammunition::definition(content, last.id())?);
        }
    }
    Ok(())
}
