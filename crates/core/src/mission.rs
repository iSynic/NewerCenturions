use crate::{Event, MapState, Presentation, RuleError, ScriptState, Session};
use centurions_content::{Content, Warrior};

/// partyloss14913e clears its screen state, not killbody/heal's retained work.
pub(crate) fn end(state: &mut Session, content: &Content) -> Result<(), RuleError> {
    crate::combat::runtime::retain_last_defeated(state)?;
    state.retain_combat_work();
    state.clear_tactical_fields();
    if let Some(combat) = &state.combat {
        state.native_work.tactical_work = crate::combat::tactical_work::TacticalWork::capture(
            combat,
            &state.native_work.tactical_work,
        );
    }
    state.retain_npcs();
    let mut next = Session::new(content);
    // partyloss14913e leaves work globals in place while clearing the active party.
    std::mem::swap(&mut next.native_work, &mut state.native_work);
    // partyloss has no writer for the menu records440edc or count44257c.
    std::mem::swap(&mut next.allies, &mut state.allies);
    next.ally_count = state.ally_count;
    // cleartarget13822a clears110 WORDs at4434e0, including these six party entries.
    next.clear_targets();
    // partyloss1493aa clears WORD44374c, leaving total WORD4435c4 untouched.
    next.native_work.victory_work.defeated = 0;
    // partyloss1493b8 clears Ready WORD438760 independently of combat mode.
    next.native_work.ready = 0;
    let health = &mut next.native_work.health_context;
    // 1493b0/1493dc/1493ec; last body, effect and affected words survive.
    health.unavailable = 0;
    health.marks_affected = false;
    health.selected_warrior = 0;
    next.confiscated = state.confiscated.take();
    next.random = state.random.clone();
    next.library_random_seed = state.library_random_seed;
    next.library_random_draws = state.library_random_draws;
    next.scenario_duration_tail = state.scenario_duration_tail;
    *state = next;
    Ok(())
}

pub(crate) fn defeat(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    end(state, content)?;
    Ok(vec![
        Event::Sound { resource_id: 26260 },
        Event::MissionEnded { defeated: true },
    ])
}

pub(crate) fn travel(
    state: &mut Session,
    content: &Content,
    target: crate::story_destination::Location,
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

pub fn stage_target(parameter: i16) -> Result<usize, RuleError> {
    let target = parameter.wrapping_abs().wrapping_sub(1);
    usize::try_from(target).map_err(|_| RuleError::Invalid("stage target is unavailable".into()))
}

pub fn restore_stage_warriors(warriors: &[Warrior]) -> Result<Vec<Warrior>, RuleError> {
    let mut restored = warriors.to_vec();
    for warrior in restored.iter_mut().take(warriors.len().saturating_sub(1)) {
        // The original stage caller restores all but the final party member.
        // Keep its read/write sequence, including partial failure behavior.
        let stamina = warrior.record().maximum_stamina()?;
        warrior.record_mut().set_stamina(stamina);
        let psi = warrior.record().maximum_psi()?;
        warrior.record_mut().set_psi_points(psi);
    }
    Ok(restored)
}

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
    state.allies = crate::story_allies::clear_stage_allies(&state.allies)?;
    state.ally_count = 0;
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
