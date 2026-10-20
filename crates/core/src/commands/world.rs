use crate::{
    Command, Event, Exploration, Phase, Presentation, RuleError, Session, clinic, combat,
    exploration, field_controls, reference_views, story_cleanup, story_promotion, typed_encounter,
};
use centurions_content::Content;

pub(super) fn execute(
    command: Command,
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    match command {
        Command::FieldControl { action } => field_controls::start(state, content, action),
        Command::ClinicControl { item } => clinic::control(state, content, item),
        Command::ViewCreature { index, ally } => reference_views::view(state, content, index, ally),
        Command::ViewNote { index } => reference_views::note(state, content, index),
        Command::EndMission => {
            if !matches!(state.phase, Phase::Exploration | Phase::Combat)
                || state.medical_continuation.is_some()
            {
                return Err(RuleError::Refused { feedback: vec![] });
            }
            let random = state.random.clone();
            let library = (state.library_random_seed, state.library_random_draws);
            let scenario_tail = state.scenario_duration_tail;
            *state = Session::new(content);
            state.random = random;
            state.scenario_duration_tail = scenario_tail;
            (state.library_random_seed, state.library_random_draws) = library;
            Ok(vec![])
        }
        Command::SetDifficulty { level } => {
            if state.phase != Phase::SquadSelection || !(-2..=2).contains(&level) {
                return Err(RuleError::Invalid(
                    "difficulty can only be selected before a mission".into(),
                ));
            }
            state.difficulty = level;
            Ok(vec![])
        }
        Command::EnterMission { name } => {
            if state.phase != Phase::SquadSelection || !(1..=6).contains(&state.warriors.len()) {
                return Err(RuleError::Invalid(
                    "mission entry requires one to six selected warriors".into(),
                ));
            }
            let scenario = content.scenario.as_ref().ok_or("scenario unavailable")?;
            let opening = content
                .map(scenario.opening_map)
                .ok_or_else(|| RuleError::Invalid("opening content unavailable".into()))?;
            state.phase = Phase::Exploration;
            state.mission = Some(name);
            state.scenario_level = scenario.level;
            state.exploration = Some(Exploration {
                map: scenario.opening_map,
                x: opening.start.0,
                y: opening.start.1,
                day: 1,
                hour: 12,
                minute: 0,
                facing: 0,
                last_move: [0; 2],
                script: None,
                presentation: None,
                fatigue: 4,
                event_headers: opening.header.clone(),
                schedule: opening.schedule.clone(),
                region_state: opening.regions.clone(),
                visibility: vec![0; 8100],
                floor: opening.floor.clone(),
                objects: opening.objects.clone(),
                events: opening.events.clone(),
            });
            Ok(vec![Event::MissionEntered {
                title: opening.name.clone(),
                x: opening.start.0,
                y: opening.start.1,
            }])
        }
        Command::Move { dx, dy } => {
            if state.phase == Phase::Combat {
                combat::move_warrior(state, content, dx, dy)
            } else {
                exploration::move_squad(state, content, dx, dy)
            }
        }
        Command::Continue => {
            if state.combat.as_ref().is_some_and(|c| {
                c.round_scripts
                    .pending
                    .as_ref()
                    .is_some_and(|program| !program.delegated)
            }) {
                return combat::runtime::continue_round(state, content);
            }
            if matches!(
                state
                    .exploration
                    .as_ref()
                    .and_then(|p| p.presentation.as_ref()),
                Some(Presentation::Loot)
            ) {
                exploration::continue_story(state, content)
            } else if matches!(
                state
                    .exploration
                    .as_ref()
                    .and_then(|p| p.presentation.as_ref()),
                Some(Presentation::Survivors)
            ) {
                if state.scripted_cleanup == Some(story_cleanup::Stage::Survivors) {
                    let mut events = story_cleanup::confirm(state, content)?;
                    events.extend(exploration::run_script(state, content)?);
                    Ok(events)
                } else {
                    combat::runtime::confirm_survivors(state, content)
                }
            } else if matches!(
                state
                    .exploration
                    .as_ref()
                    .and_then(|p| p.presentation.as_ref()),
                Some(Presentation::Messages { .. })
            ) {
                exploration::continue_messages(state, content)
            } else if state.combat.is_none()
                && state
                    .round_dispatch
                    .as_ref()
                    .is_some_and(|frame| frame.owns_script(state))
            {
                exploration::continue_story(state, content)
            } else if state
                .combat_story
                .as_ref()
                .is_some_and(|story| !story.waiting_child || state.combat.is_none())
            {
                combat::runtime::continue_death(state, content)
            } else {
                exploration::continue_story(state, content)
            }
        }
        Command::KeepCurrentSpells => exploration::keep_current_spells(state, content),
        Command::BeginPromotion => story_promotion::begin(state, content),
        Command::PromotionControl { item } => story_promotion::control(state, content, item),
        Command::PromotionAnswer { yes } => story_promotion::answer(state, content, yes),
        Command::Choose { answer } => exploration::choose(state, content, answer),
        Command::CancelMenu => exploration::cancel_menu(state, content),
        Command::SelectMenuEntry { index } => exploration::select_menu(state, content, index),
        Command::SubmitText { text } => exploration::submit_text(state, content, &text),
        Command::BeginEncounterActions => typed_encounter::begin_actions(state, content),
        Command::CastEncounterPsi {
            warrior,
            spell,
            power,
        } => typed_encounter::cast_spell(state, content, warrior, spell, power),
        Command::EncounterActionControl { item } => {
            typed_encounter::action_control(state, content, item)
        }
        Command::ChooseWarrior { index } => exploration::choose_warrior(state, content, index),
        Command::Encounter => exploration::manual_encounter(state, content),
        _ => unreachable!("command routed to wrong subsystem"),
    }
}
