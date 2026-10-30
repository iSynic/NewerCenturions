use crate::combat::battlefield::Battlefield;
use crate::{Event, MapState, Presentation, RuleError, ScriptState, Session};
use centurions_content::{Content, Warrior};

/// Ends the mission when the party is lost. The screen state clears; killbody and
/// heal's last results stay.
///
/// Original: partyloss
pub(crate) fn end(state: &mut Session, content: &Content) -> Result<(), RuleError> {
    crate::combat::runtime::store_last_defeated(state)?;
    state.store_combat_globals();
    state.clear_tactical_fields();
    if let Some(combat) = &state.combat {
        state.globals.battlefield = Battlefield::capture(combat, &state.globals.battlefield);
    }
    state.store_monsters();
    let mut next = Session::new(content);
    // The game values stay while the active party clears.
    std::mem::swap(&mut next.globals, &mut state.globals);
    // The allies stay.
    std::mem::swap(&mut next.holdover, &mut state.holdover);
    next.held_over = state.held_over;
    next.clear_targets();
    // Defeated enemies reset; the enemy total stays.
    next.globals.kill_counts.kill_mon = 0;
    next.globals.in_spell = 0;
    let health = &mut next.globals.health_context;
    // The last body, effect and affected members stay.
    health.kill_party = 0;
    health.marks_affected = false;
    health.char_select_new = 0;
    next.confiscated = state.confiscated.take();
    next.random = state.random.clone();
    next.library_random_seed = state.library_random_seed;
    next.library_random_draws = state.library_random_draws;
    next.scenario_duration_tail = state.scenario_duration_tail;
    *state = next;
    Ok(())
}

/// Ends the mission in defeat: the party is lost with the defeat sound.
pub(crate) fn defeat(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    end(state, content)?;
    Ok(vec![
        Event::Sound { resource_id: 26260 },
        Event::MissionEnded { defeated: true },
    ])
}

/// Takes the party to a square of a map and level, keeping the maps it leaves so they come back as
/// they were left.
pub(crate) fn travel(
    state: &mut Session,
    content: &Content,
    target: crate::story::destination::Location,
) -> Result<(), RuleError> {
    centurions_content::mission::OpeningMission::index(target.x, target.y)?;
    let authored = content
        .map_at(state.mission.as_deref(), target.level, target.map)
        .ok_or_else(|| {
            RuleError::Invalid("destination scenario level or map unavailable".into())
        })?;
    let here = state.exploration.as_ref().ok_or("no active map")?.map;
    let changed = target.level != state.scenario_level || target.map != here;
    if changed {
        state.check_land_tile_set(content, target.level, target.map)?;
    }
    let position = state.exploration.as_mut().ok_or("no active map")?;
    if target.level != state.scenario_level {
        state
            .visited_maps
            .insert(position.map, position.map_state());
        let previous = crate::LevelState {
            maps: std::mem::take(&mut state.visited_maps),
            schedule: std::mem::take(&mut position.schedule),
            menus: std::mem::take(&mut state.menus),
            typed_encounters: std::mem::take(&mut state.typed_encounters),
        };
        state.visited_levels.insert(state.scenario_level, previous);
        if let Some(saved) = state.visited_levels.remove(&target.level) {
            state.visited_maps = saved.maps;
            position.schedule = saved.schedule;
            state.menus = saved.menus;
            state.typed_encounters = saved.typed_encounters;
        } else {
            position.schedule = authored.schedule.clone();
        }
        let records = state
            .visited_maps
            .remove(&target.map)
            .unwrap_or_else(|| MapState::authored(authored));
        position.restore_map(target.map, records);
        state.scenario_level = target.level;
    } else if target.map != position.map {
        let records = state
            .visited_maps
            .remove(&target.map)
            .unwrap_or_else(|| MapState::authored(authored));
        state
            .visited_maps
            .insert(position.map, position.map_state());
        position.restore_map(target.map, records);
    }
    position.x = target.x;
    position.y = target.y;
    if changed {
        state.record_battle_land();
    }
    Ok(())
}

/// The zero-based stage a script's stage change names: its parameter ignoring sign, less one.
pub fn stage_target(parameter: i16) -> Result<usize, RuleError> {
    let target = parameter.wrapping_abs().wrapping_sub(1);
    usize::try_from(target).map_err(|_| RuleError::Invalid("stage target is unavailable".into()))
}

/// Restores stamina and psi to their maximums for every warrior but the last, as a stage begins.
pub fn restore_stage_warriors(warriors: &[Warrior]) -> Result<Vec<Warrior>, RuleError> {
    let mut restored = warriors.to_vec();
    for warrior in restored.iter_mut().take(warriors.len().saturating_sub(1)) {
        // Every member but the last is restored, as the game does; keep the
        // order, including where a failure stops it.
        let stamina = warrior.record().stamina_max()?;
        warrior.record_mut().set_stamina(stamina);
        let psi = warrior.record().psi_points_max()?;
        warrior.record_mut().set_psi_points(psi);
    }
    Ok(restored)
}

/// Begins a scenario stage: the party goes to the stage's starting square and map, the warriors are
/// restored, held-over allies are cleared, and the stage's start script and prologue are queued.
pub(crate) fn begin_stage(
    state: &mut Session,
    content: &Content,
    parameter: i16,
) -> Result<Vec<Event>, RuleError> {
    let target = stage_target(parameter)?;
    let scenario = content
        .scenario
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("scenario unavailable".into()))?;
    let stage = scenario
        .stages
        .get(target)
        .filter(|s| !s.name.is_empty())
        .ok_or_else(|| RuleError::Invalid("stage unavailable".into()))?;
    let here = state
        .exploration
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("no active map".into()))?
        .map;
    if here != stage.start.2 {
        state.check_land_tile_set(content, state.scenario_level, stage.start.2)?;
    }
    let position = state
        .exploration
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("no active map".into()))?;
    let authored = content
        .map(stage.start.2)
        .ok_or_else(|| RuleError::Invalid("stage map unavailable".into()))?;
    let records = if position.map == stage.start.2 {
        position.map_state()
    } else {
        let records = state
            .visited_maps
            .remove(&stage.start.2)
            .unwrap_or_else(|| MapState::authored(authored));
        state
            .visited_maps
            .insert(position.map, position.map_state());
        records
    };
    position.restore_map(stage.start.2, records);
    position.x = stage.start.0;
    position.y = stage.start.1;
    state.mission_stage = target as i16;
    state.warriors = restore_stage_warriors(&state.warriors)?;
    state.holdover = crate::story::allies::clear_stage_allies(&state.holdover)?;
    state.held_over = 0;
    position.script = if stage.start_script != 0 {
        Some(ScriptState::scheduled(
            authored.script(stage.start_script)?.to_vec(),
            0,
        ))
    } else {
        None
    };
    position.presentation = Some(Presentation::Prologue {
        resource_id: 1000 + target as i16,
    });
    Ok(vec![
        Event::MissionEntered {
            title: stage.name.clone(),
            x: position.x,
            y: position.y,
        },
        Event::StoryPresented,
    ])
}
