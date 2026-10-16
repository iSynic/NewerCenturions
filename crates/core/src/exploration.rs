use super::*;
use centurions_content::{
    mission::{OpeningMission, long},
    span, word,
};

fn unsupported(action: &'static str, dependency: &'static str) -> RuleError {
    RuleError::Unsupported { action, dependency }
}
pub(crate) fn active_map<'a>(
    state: &Session,
    content: &'a Content,
) -> Result<&'a OpeningMission, RuleError> {
    content
        .map_at(
            state.mission.as_deref(),
            state.scenario_level,
            state
                .exploration
                .as_ref()
                .map_or(0, |position| position.map),
        )
        .ok_or_else(|| RuleError::Invalid("active map unavailable".into()))
}

pub fn manual_encounter(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    if state.phase != Phase::Exploration
        || state
            .exploration
            .as_ref()
            .is_none_or(|position| position.presentation.is_some() || position.script.is_some())
    {
        return Err(RuleError::Invalid(
            "encounter requires free exploration".into(),
        ));
    }
    let map = active_map(state, content)?;
    let position = state.exploration.as_mut().unwrap();
    let id = crate::story_manual_encounter::select(
        &mut position.region_state,
        (position.x, position.y),
        &mut state.random,
    )?;
    position.last_move = [0; 2];
    position.script = Some(ScriptState {
        record: map.script(id)?.to_vec(),
        cursor: 0,
        map_slot: 0,
        item_return: true,
        scheduled: true,
        menu: None,
        persist: false,
        origin: None,
    });
    run_script(state, content)
}

pub fn use_item(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    if state
        .exploration
        .as_ref()
        .is_some_and(|p| matches!(p.presentation, Some(Presentation::Typed { .. })))
    {
        return use_encounter_item(state, content, warrior, slot);
    }
    let item = state
        .warriors
        .get(warrior)
        .ok_or("item warrior unavailable")?
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or("inventory item unavailable")?;
    if (700..=703).contains(&item.id()) && state.phase == Phase::Combat {
        return crate::combat::runtime::begin_medical_item(state, content, warrior, slot);
    }
    let definition = crate::ammunition::definition(content, item.id())?;
    if state.combat.is_none() && word(&definition, 26)? == 28 && word(&definition, 44)? > 1100 {
        return crate::outdoor_psi::begin_item(state, content, warrior, slot);
    }
    if word(&definition, 26)? == 27 {
        return crate::item_modifiers::use_item(state, content, warrior, slot);
    }
    crate::story_item_use::reject_inert_item(&definition)?;
    if state.phase == Phase::Combat && word(&definition, 26)?.wrapping_abs() == 23 {
        return refuse_combat_script_item(state, content, warrior, item.id(), false);
    }
    if state.phase != Phase::Exploration
        || state
            .exploration
            .as_ref()
            .is_none_or(|p| p.presentation.is_some() || p.script.is_some())
    {
        return Err(RuleError::Invalid(
            "item use requires free exploration".into(),
        ));
    }
    let map = active_map(state, content)?;
    let races = &map.combat.as_ref().unwrap().race_moves;
    let used = crate::story_item_use::use_scenario_item(
        &state.warriors,
        warrior,
        slot,
        content,
        races,
        crate::story_item_use::ScriptItemContext::Exploration,
        state.inventory_record.as_ref(),
    )?;
    let id = state.warriors[warrior].inventory()?[slot].id();
    state.load_item_work(content, id)?;
    state.warriors = used.warriors;
    state.selected_warrior = warrior;
    let mut effects: Vec<Event> = used
        .warnings
        .into_iter()
        .map(|warning| Event::CollectionRejected { warning })
        .collect();
    effects.extend(
        used.sounds
            .into_iter()
            .map(|resource_id| Event::Sound { resource_id }),
    );
    if let Some(id) = used.script {
        state.exploration.as_mut().unwrap().script = Some(ScriptState {
            record: map.script(id)?.to_vec(),
            cursor: 0,
            map_slot: 0,
            item_return: false,
            scheduled: true,
            menu: None,
            persist: false,
            origin: None,
        });
        effects.extend(run_script(state, content)?);
    }
    Ok(effects)
}

pub fn move_squad(
    state: &mut Session,
    content: &Content,
    dx: i8,
    dy: i8,
) -> Result<Vec<Event>, RuleError> {
    if state.phase != Phase::Exploration
        || !(-1..=1).contains(&dx)
        || !(-1..=1).contains(&dy)
        || (dx == 0 && dy == 0)
    {
        return Err(RuleError::Invalid(
            "movement requires an exploration direction".into(),
        ));
    }
    if !crate::field_controls::free(state) {
        return Err(RuleError::Refused { feedback: vec![] });
    }
    if let Some(events) = crate::field_controls::leave_camp(state, content, dx, dy)? {
        return Ok(events);
    }
    let map = active_map(state, content)?;
    let position = state.exploration.as_mut().unwrap();
    if position.presentation.is_some()
        || position.script.is_some()
        || state.movement_continuation.is_some()
    {
        return Err(RuleError::Invalid(
            "finish the current story page before moving".into(),
        ));
    }
    if let Some(control) = state.field_controls.as_mut() {
        control.camping = false;
        control.clinic = None;
    }
    state.shop_access = None;
    position.last_move = [dx, dy];
    let x = position.x + i32::from(dx);
    let y = position.y + i32::from(dy);
    let index = OpeningMission::index(x, y)?;
    let tile = position.floor[index];
    let property = if tile.unsigned_abs() > 200 || tile < 0 {
        map.default_tile
    } else {
        tile
    };
    let sound = if tile < 0 {
        82
    } else {
        map.tile_word(tile, 0)?
    };
    let mut effects = vec![Event::Sound { resource_id: sound }];
    if map.tile_word(property, 8)? == 1 {
        return Err(unsupported(
            "exploration.move",
            "boarding choice and boat movement",
        ));
    }
    let event_branch = position.events[index] > 0;
    let moved =
        event_branch || (map.tile_word(property, 4)? == 0 && map.tile_word(property, 8)? != 2);
    if moved {
        position.x = x;
        position.y = y;
        effects.push(Event::Moved { x, y });
    }
    if dx != 0 {
        position.facing = u8::from(dx > 0);
    }
    let ticks = if position.events[index] > 0 {
        if tile < 0 { 1 } else { map.tile_word(tile, 2)? }
    } else {
        effects.push(Event::Sound { resource_id: sound });
        if tile < 0 {
            3
        } else {
            map.tile_word(property, 2)?
        }
    };
    if ticks < 0 {
        return Err(unsupported(
            "exploration.move",
            "negative-time tile semantics",
        ));
    }
    let px = position.x as i16;
    let py = position.y as i16;
    state.movement_continuation = Some(MovementContinuation::Region {
        x: px,
        y: py,
        check: ticks != 0,
        header: event_branch,
    });
    effects.extend(super::time::advance(
        state,
        content,
        ticks as u8,
        map.minutes_per_tick,
    )?);
    Ok(effects)
}

pub(crate) fn resume_movement(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    let mut effects = vec![];
    while state.phase == Phase::Exploration
        && state.combat_story.is_none()
        && state
            .exploration
            .as_ref()
            .is_some_and(|position| position.presentation.is_none() && position.script.is_none())
    {
        if state.time_continuation.is_some() {
            effects.extend(super::time::resume(state, content)?);
            if state.time_continuation.is_some()
                || state.phase != Phase::Exploration
                || state
                    .exploration
                    .as_ref()
                    .is_none_or(|p| p.presentation.is_some() || p.script.is_some())
            {
                break;
            }
        }
        match state.movement_continuation.take() {
            Some(MovementContinuation::Region {
                x,
                y,
                check,
                header,
            }) => {
                state.movement_continuation = Some(if header {
                    MovementContinuation::Header
                } else {
                    MovementContinuation::Secrets
                });
                if check && word(&state.party_buffs, 14)? == 0 {
                    let region = regional_selection(
                        &mut state.exploration.as_mut().unwrap().region_state,
                        (x, y),
                        &mut state.random,
                    )?;
                    if matches!(region, Regional::Script(_)) {
                        state.movement_continuation =
                            Some(MovementContinuation::RegionalReturn { header });
                    }
                    effects.extend(apply_region(state, content, region)?);
                }
            }
            Some(MovementContinuation::RegionalReturn { header }) => {
                // timeclick15ea8c resumes buttonchoice after clearing the movement deltas.
                state.exploration.as_mut().unwrap().last_move = [0; 2];
                state.movement_continuation = Some(if header {
                    MovementContinuation::Header
                } else {
                    MovementContinuation::Secrets
                });
            }
            Some(MovementContinuation::Header) => {
                state.movement_continuation = Some(MovementContinuation::Secrets);
                let position = state.exploration.as_mut().unwrap();
                if let Some(header) = crate::walking_header::lookup(
                    &position.event_headers,
                    position.map,
                    position.x,
                    position.y,
                    &mut state.random,
                )? {
                    position.script = Some(ScriptState {
                        record: header.record,
                        cursor: 0,
                        map_slot: header.slot,
                        item_return: false,
                        scheduled: false,
                        menu: None,
                        persist: false,
                        origin: Some(crate::ScriptOrigin {
                            map: position.map,
                            x: position.x,
                            y: position.y,
                            level: state.scenario_level,
                        }),
                    });
                    effects.extend(run_script(state, content)?);
                    if effects
                        .iter()
                        .any(|event| matches!(event, Event::Message { .. }))
                    {
                        break;
                    }
                }
            }
            Some(MovementContinuation::Secrets) => {
                if crate::field_controls::automatic_search(state) {
                    continue;
                }
                let skill = travel::secret_search_skill(&state.warriors)?;
                let position = state.exploration.as_mut().unwrap();
                let search = travel::discover_secrets(
                    &position.floor,
                    (position.x as i16, position.y as i16),
                    skill,
                    &mut state.random,
                )?;
                for discovery in &search.discoveries {
                    effects.push(Event::Feedback(
                        crate::feedback::NativeFeedback::ResourceFlash {
                            resource_id: 138,
                            index: usize::from(discovery.ordinal),
                            x: 30,
                            y: 375,
                            wait: 0,
                            sound: 5000,
                        },
                    ));
                }
                position.floor = search.floor;
                if search.center_revealed {
                    // Packed (mode0, argument1) returns to the walking-header branch.
                    state.movement_continuation = Some(MovementContinuation::Header);
                }
            }
            None => break,
        }
    }
    Ok(effects)
}

#[cfg(all(test, feature = "original-fixtures"))]
pub fn regional_encounter(
    regions: &[u8],
    position: (i16, i16),
    random: &mut RandomState,
) -> Result<Option<(i16, i16)>, RuleError> {
    match regional_selection(&mut regions.to_vec(), position, random)? {
        Regional::None => Ok(None),
        Regional::Battle(battle, advantage) => Ok(Some((battle, advantage))),
        Regional::Script(_) | Regional::Avoidance(_) => Err(unsupported(
            "exploration.encounter",
            "regional event scripts",
        )),
    }
}

enum Regional {
    None,
    Battle(i16, i16),
    Script(i16),
    Avoidance(usize),
}

fn regional_selection(
    regions: &mut [u8],
    position: (i16, i16),
    random: &mut RandomState,
) -> Result<Regional, RuleError> {
    regional_selection_from(regions, position, 20, random)
}

fn regional_selection_from(
    regions: &mut [u8],
    position: (i16, i16),
    before: usize,
    random: &mut RandomState,
) -> Result<Regional, RuleError> {
    regional_selection_chance(regions, position, before, 1, random)
}

pub(crate) fn check_region(
    state: &mut Session,
    content: &Content,
    divisor: i16,
) -> Result<Vec<Event>, RuleError> {
    let position = state
        .exploration
        .as_mut()
        .ok_or("regional position unavailable")?;
    let region = regional_selection_chance(
        &mut position.region_state,
        (position.x as i16, position.y as i16),
        20,
        divisor,
        &mut state.random,
    )?;
    apply_region(state, content, region)
}

fn regional_selection_chance(
    regions: &mut [u8],
    position: (i16, i16),
    before: usize,
    divisor: i16,
    random: &mut RandomState,
) -> Result<Regional, RuleError> {
    let (px, py) = position;
    for region in (0..before.min(20)).rev() {
        let offset = region * 8;
        if rules::inclusive_range(px, word(regions, offset + 2)?, word(regions, offset + 6)?)
            && rules::inclusive_range(py, word(regions, offset)?, word(regions, offset + 4)?)
        {
            let result = random.bound(10000);
            if result <= word(regions, 160 + region * 2)? / divisor {
                for event in 0..3 {
                    if random.bound(100)
                        <= word(regions, 400 + region * 6 + event * 2)?.wrapping_abs()
                    {
                        let chance = 400 + region * 6 + event * 2;
                        if word(regions, chance)? > 0 {
                            regions[chance..chance + 2].fill(0);
                        }
                        return Ok(Regional::Script(word(
                            regions,
                            280 + region * 6 + event * 2,
                        )?));
                    }
                }
                let minimum = word(regions, 200 + region * 4)?;
                if minimum != 0 {
                    if random.bound(100) < i16::from(regions[544 + region] as i8) {
                        return Ok(Regional::Avoidance(region));
                    }
                    let advantage = if random.bound(100) < 10 { -1 } else { 0 };
                    let battle = random.range(minimum, word(regions, 202 + region * 4)?);
                    return Ok(Regional::Battle(battle, advantage));
                }
            }
            if regions[0x20c + region] != 0 {
                break;
            }
        }
    }
    Ok(Regional::None)
}

fn apply_region(
    state: &mut Session,
    content: &Content,
    region: Regional,
) -> Result<Vec<Event>, RuleError> {
    match region {
        Regional::None => Ok(vec![]),
        Regional::Battle(battle, advantage) => {
            super::combat::begin_battle(state, content, battle, advantage)
        }
        Regional::Script(id) => {
            let map = active_map(state, content)?;
            state.exploration.as_mut().unwrap().script = Some(ScriptState {
                record: map.script(id)?.to_vec(),
                cursor: 0,
                map_slot: 0,
                item_return: false,
                scheduled: true,
                menu: None,
                persist: false,
                origin: None,
            });
            run_script(state, content)
        }
        Regional::Avoidance(region) => {
            let map = active_map(state, content)?;
            let position = state.exploration.as_mut().unwrap();
            let id = word(&position.region_state, 604 + region * 2)?.wrapping_abs();
            let sound = word(&position.region_state, 564 + region * 2)?;
            position.presentation = Some(Presentation::Avoidance {
                region: region as u8,
                text: map.dialogue(id)?,
            });
            Ok(vec![
                Event::Sound { resource_id: sound },
                Event::StoryPresented,
            ])
        }
    }
}

fn choose_avoidance(
    state: &mut Session,
    content: &Content,
    region: usize,
    fight: bool,
) -> Result<Vec<Event>, RuleError> {
    let position = state
        .exploration
        .as_mut()
        .ok_or("encounter position missing")?;
    if region >= 20 {
        return Err(RuleError::Invalid("encounter region outside map".into()));
    }
    position.presentation = None;
    let next = if fight {
        let battle = state.random.range(
            word(&position.region_state, 200 + region * 4)?,
            word(&position.region_state, 202 + region * 4)?,
        );
        Regional::Battle(battle, 1)
    } else if position.region_state[524 + region] != 0 {
        Regional::None
    } else {
        regional_selection_from(
            &mut position.region_state,
            (position.x as i16, position.y as i16),
            region,
            &mut state.random,
        )?
    };
    let mut events = apply_region(state, content, next)?;
    events.extend(resume_movement(state, content)?);
    Ok(events)
}

pub fn continue_messages(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let position = state
        .exploration
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("no active mission".into()))?;
    let Some(Presentation::Messages { mut pages, next }) = position.presentation.take() else {
        return Err(RuleError::Invalid("no message active".into()));
    };
    if pages.is_empty() {
        return Err(RuleError::Invalid("message continuation is empty".into()));
    }
    pages.remove(0);
    if !pages.is_empty() {
        position.presentation = Some(Presentation::Messages { pages, next });
        return Ok(vec![Event::StoryPresented]);
    }
    position.presentation = next.map(|page| *page);
    if position.presentation.is_some() {
        return Ok(vec![Event::StoryPresented]);
    }
    if state.phase == crate::Phase::Combat {
        Ok(vec![])
    } else {
        run_script(state, content)
    }
}

pub fn continue_story(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    continue_with_spells(state, content, false)
}

pub fn needs_spell_confirmation(state: &Session) -> bool {
    state.warriors.iter().any(|warrior| {
        warrior.stamina().unwrap_or(0) > 0
            && i32::from_be_bytes(warrior.bytes[0x9ce..0x9d2].try_into().unwrap()) < 0
            && word(&warrior.bytes, 0x5a).unwrap_or(0) != 0
            && word(&warrior.bytes, 0x36).unwrap_or(0) != 0
    })
}

pub fn keep_current_spells(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    if !needs_spell_confirmation(state)
        || !state
            .exploration
            .as_ref()
            .is_some_and(|position| matches!(position.presentation, Some(Presentation::Loot)))
    {
        return Err(RuleError::Invalid(
            "no pending spell-promotion choice".into(),
        ));
    }
    continue_with_spells(state, content, true)
}

fn continue_with_spells(
    state: &mut Session,
    content: &Content,
    keep_spells: bool,
) -> Result<Vec<Event>, RuleError> {
    if state
        .exploration
        .as_ref()
        .is_some_and(|p| p.presentation == Some(Presentation::Shop))
    {
        return crate::shop::close(state, content);
    }
    let loot = state
        .exploration
        .as_ref()
        .is_some_and(|position| matches!(position.presentation, Some(Presentation::Loot)));
    if loot && needs_spell_confirmation(state) && !keep_spells {
        return crate::story_promotion::begin(state, content);
    }
    let mut events = vec![];
    if loot && !state.booty_items.iter().any(|item| item.id() != 0) {
        let map = active_map(state, content)?;
        let mut library_random = crate::story_promotion::LibraryRandom {
            seed: state.library_random_seed,
            draws: state.library_random_draws,
        };
        let closed = crate::story_promotion::close(
            &state.warriors,
            &map.races,
            &map.profiles,
            &map.combat.as_ref().unwrap().attacks,
            true,
            &mut library_random,
        )?;
        state.warriors = closed.warriors;
        state.library_random_seed = library_random.seed;
        state.library_random_draws = library_random.draws;
        events.extend(
            closed
                .sounds
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id }),
        );
        events.extend(
            closed
                .warnings
                .into_iter()
                .map(|warning| Event::CollectionRejected { warning }),
        );
        for warrior in closed.promoted {
            events.push(Event::LevelAdvanced {
                warrior,
                level: word(&state.warriors[warrior].bytes, 0x3a)?,
            });
        }
    }
    events.extend(finish_story_page(state, content)?);
    Ok(events)
}

pub(crate) fn finish_story_page(
    state: &mut Session,
    content: &Content,
) -> Result<Vec<Event>, RuleError> {
    let mut events = vec![];
    let position = state
        .exploration
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("no active mission".into()))?;
    if matches!(
        position.presentation,
        Some(
            Presentation::Question { .. }
                | Presentation::Menu { .. }
                | Presentation::Typed { .. }
                | Presentation::StageChange { .. }
                | Presentation::WarriorChoice { .. }
        )
    ) {
        return Err(RuleError::Invalid(
            "choose or cancel the current question or menu".into(),
        ));
    }
    if position.presentation.take().is_none() {
        return Err(RuleError::Invalid("no story page to continue".into()));
    }
    if state.booty_items.iter().any(|item| item.id() != 0) {
        return Err(RuleError::Invalid(
            "collect or store the equipment before continuing".into(),
        ));
    }
    if state.actor_replacement.is_some() {
        let (feedback, progress) = crate::story_actor::resume(state, content)?;
        events.extend(feedback);
        match progress {
            crate::story_actor::Progress::Waiting => return Ok(events),
            crate::story_actor::Progress::Stop => {
                events.extend(finish_script(state, content, false)?);
                return Ok(events);
            }
            crate::story_actor::Progress::Continue => {}
        }
    }
    if state
        .combat_story
        .as_ref()
        .is_some_and(|story| story.waiting_child)
    {
        events.extend(combat::runtime::continue_death(state, content)?);
    } else {
        events.extend(run_script(state, content)?);
    }
    Ok(events)
}

pub fn choose_warrior(
    state: &mut Session,
    content: &Content,
    index: usize,
) -> Result<Vec<Event>, RuleError> {
    let map = active_map(state, content)?;
    let warrior = state
        .warriors
        .get(index)
        .ok_or_else(|| RuleError::Invalid("warrior unavailable".into()))?;
    if warrior.stamina()? < 1 {
        return Err(RuleError::Invalid("choose a living warrior".into()));
    }
    let position = state
        .exploration
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("no active mission".into()))?;
    let Some(Presentation::WarriorChoice { extra }) = position.presentation.as_ref() else {
        return Err(RuleError::Invalid("no warrior choice active".into()));
    };
    let selected = crate::story_skill::single_warrior(&state.warriors, index)?;
    state.party_mask = selected;
    state.selected_warrior = index;
    if *extra == -1 {
        position.presentation = None;
        return run_script(state, content);
    }
    let record = span(
        &map.combat.as_ref().unwrap().encounters,
        *extra as usize * 10,
        10,
    )?;
    let plan = crate::story_skill::predicate(record, &state.warriors, &selected)?;
    let script = position
        .script
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("choice script missing".into()))?;
    if let crate::story_skill::SkillPlan::ReplaceScript { script: id, .. } = plan {
        script.record[8..].copy_from_slice(&map.script(id)?[8..]);
        script.cursor = 0;
    }
    position.presentation = None;
    run_script(state, content)
}

pub fn choose(
    state: &mut Session,
    content: &Content,
    answer: bool,
) -> Result<Vec<Event>, RuleError> {
    if let Some(Presentation::Avoidance { region, .. }) = state
        .exploration
        .as_ref()
        .and_then(|p| p.presentation.as_ref())
    {
        return choose_avoidance(state, content, *region as usize, answer);
    }
    if let Some(Presentation::StageChange { parameter }) = state
        .exploration
        .as_ref()
        .and_then(|p| p.presentation.as_ref())
    {
        let parameter = *parameter;
        if answer {
            return mission::begin_stage(state, content, parameter);
        }
        let position = state.exploration.as_mut().unwrap();
        position.presentation = None;
        position.script = None;
        return Ok(vec![]);
    }
    let map = active_map(state, content)?;
    let position = state
        .exploration
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("no active mission".into()))?;
    let Some(Presentation::Question { extra, .. }) = position.presentation.as_ref() else {
        return Err(RuleError::Invalid("no question active".into()));
    };
    let record = span(
        &map.combat.as_ref().unwrap().encounters,
        *extra as usize * 10,
        10,
    )?;
    match typed_encounter::question_plan(record, answer)? {
        typed_encounter::QuestionPlan::Continue => {}
        typed_encounter::QuestionPlan::MoveAway => {
            let movement = back_out(position)?;
            position.presentation = None;
            position.script = None;
            let mut events = vec![movement];
            if let Some(closed) = finish_dispatch(state, content)? {
                events.extend(closed);
            }
            return Ok(events);
        }
        typed_encounter::QuestionPlan::ReplaceScript(id) => {
            let replacement = map.script(id)?;
            let script = position
                .script
                .as_mut()
                .ok_or_else(|| RuleError::Invalid("question script missing".into()))?;
            script.record[8..].copy_from_slice(&replacement[8..]);
            script.cursor = 0;
        }
        typed_encounter::QuestionPlan::MenuBranch(branch) => {
            let script = position
                .script
                .as_mut()
                .ok_or_else(|| RuleError::Invalid("question script missing".into()))?;
            let id = script
                .menu
                .as_ref()
                .ok_or_else(|| RuleError::Invalid("question menu missing".into()))?
                .id;
            let table = if state.menus.is_empty() {
                &map.menus
            } else {
                &state.menus
            };
            script.record[8..40].copy_from_slice(&story::dispatch_menu_branch(
                span(
                    table,
                    id as usize * story::MENU_RECORD_BYTES,
                    story::MENU_RECORD_BYTES,
                )?,
                branch as i8 + 1,
            )?);
            script.cursor = 0;
        }
        typed_encounter::QuestionPlan::TypedBranch(branch) => {
            let script = position.script.as_mut().ok_or("question script missing")?;
            let menu = script
                .menu
                .as_ref()
                .filter(|m| m.typed)
                .ok_or("question typed continuation missing")?;
            let record = span(
                if state.typed_encounters.is_empty() {
                    &map.typed_encounters
                } else {
                    &state.typed_encounters
                },
                menu.id as usize * typed_encounter::RECORD_BYTES,
                typed_encounter::RECORD_BYTES,
            )?;
            if let typed_encounter::TypedContinuation::Branch { body, .. } =
                typed_encounter::continuation(record, branch as i8 + 1, menu.remaining)?
            {
                script.record[8..40].copy_from_slice(&body);
                script.cursor = 0;
            }
        }
        typed_encounter::QuestionPlan::EndScript => position.script = None,
    }
    position.presentation = None;
    run_script(state, content)
}

fn back_out(position: &mut Exploration) -> Result<Event, RuleError> {
    let [dx, dy] = position.last_move;
    let forward = OpeningMission::index(position.x + i32::from(dx), position.y + i32::from(dy))?;
    let undone = crate::story_move_away::move_away(
        position.x,
        position.y,
        dx,
        dy,
        position.floor[forward],
        [0; 6],
    );
    OpeningMission::index(undone.x, undone.y)?;
    position.x = undone.x;
    position.y = undone.y;
    Ok(Event::Moved {
        x: undone.x,
        y: undone.y,
    })
}

pub fn cancel_menu(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let position = state
        .exploration
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("no active mission".into()))?;
    if !matches!(
        position.presentation,
        Some(Presentation::Menu { .. } | Presentation::Typed { .. })
    ) {
        return Err(RuleError::Invalid("no recovered menu active".into()));
    }
    let script = position
        .script
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("menu script missing".into()))?;
    if let Some(menu) = &script.menu
        && !script.scheduled
    {
        position.event_headers[script.map_slot * 42..(script.map_slot + 1) * 42]
            .copy_from_slice(&menu.original);
    }
    // buttonchoice10a5a8 marks the manual caller; newland142cca skips its menu backstep.
    let backstep =
        matches!(position.presentation, Some(Presentation::Typed { .. })) || !script.item_return;
    let mut events =
        if state.phase == Phase::Exploration && backstep && position.last_move != [0; 2] {
            vec![back_out(position)?]
        } else {
            vec![]
        };
    position.presentation = None;
    position.script = None;
    if let Some(closed) = finish_dispatch(state, content)? {
        events.extend(closed);
    }
    Ok(events)
}

pub fn select_menu(
    state: &mut Session,
    content: &Content,
    index: usize,
) -> Result<Vec<Event>, RuleError> {
    let map = active_map(state, content)?;
    let position = state
        .exploration
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("no active mission".into()))?;
    let Some(Presentation::Menu { id, .. }) = position.presentation.as_ref() else {
        return Err(RuleError::Invalid("no menu active".into()));
    };
    let table = if state.menus.is_empty() {
        &map.menus
    } else {
        &state.menus
    };
    let record = span(
        table,
        *id as usize * story::MENU_RECORD_BYTES,
        story::MENU_RECORD_BYTES,
    )?;
    let selection = story::select_menu_entry(record, index)?;
    let script = position
        .script
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("menu script missing".into()))?;
    if script.menu.is_none() {
        return Err(unsupported(
            "story.select_menu",
            "legacy menu without a recovered continuation",
        ));
    }
    script.record = story::replace_menu_branch(&script.record, &selection)?;
    script.cursor = 0;
    position.presentation = None;
    run_script(state, content)
}

fn present_menu(
    map: &OpeningMission,
    table: &[u8],
    script: &mut ScriptState,
    id: i16,
) -> Result<Option<Presentation>, RuleError> {
    if id < 0 {
        return Err(RuleError::Invalid("negative encounter menu".into()));
    }
    let menu = span(
        table,
        id as usize * story::MENU_RECORD_BYTES,
        story::MENU_RECORD_BYTES,
    )?;
    if script.menu.is_none() {
        script.menu = Some(MenuContinuation {
            id,
            remaining: i16::from(menu[101] as i8),
            original: script.record.clone(),
            typed: false,
        });
    }
    let continuation = script.menu.as_mut().unwrap();
    let before = continuation.remaining;
    if menu[96] as i8 != -4 {
        continuation.remaining = before.wrapping_sub(1);
    }
    if menu[96] as i8 == -4 || before == 0 {
        script.record[8..40].copy_from_slice(&story::dispatch_menu_branch(menu, 4)?);
        script.cursor = 0;
        return Ok(None);
    }
    let labels = (0..4)
        .map(|i| {
            let label = span(menu, 106 + i * 80, 80)?;
            Ok(encoding_rs::MACINTOSH
                .decode(span(label, 1, usize::from(label[0]))?)
                .0
                .into_owned())
        })
        .collect::<Result<Vec<_>, ContentError>>()?;
    Ok(Some(Presentation::Menu {
        id,
        text: map.dialogue(word(menu, 104)?.wrapping_neg())?,
        labels,
    }))
}

fn present_typed(
    map: &OpeningMission,
    table: &[u8],
    script: &mut ScriptState,
    id: i16,
) -> Result<Presentation, RuleError> {
    if id < 0 {
        return Err(RuleError::Invalid("negative typed encounter".into()));
    }
    let record = span(
        table,
        id as usize * typed_encounter::RECORD_BYTES,
        typed_encounter::RECORD_BYTES,
    )?;
    if script.menu.is_none() {
        script.menu = Some(MenuContinuation {
            id,
            remaining: typed_encounter::attempt_count(record)?,
            original: script.record.clone(),
            typed: true,
        });
    }
    Ok(Presentation::Typed {
        id,
        text: map.dialogue(typed_encounter::dialogue(record)?.wrapping_neg())?,
    })
}

pub fn submit_text(
    state: &mut Session,
    content: &Content,
    text: &str,
) -> Result<Vec<Event>, RuleError> {
    let input = centurions_content::native_text::NativeText::encode(text)?;
    if input.len() > 40 {
        return Err(RuleError::Invalid(
            "typed input exceeds the original 40-byte field".into(),
        ));
    }
    let record = typed_record(state, content)?;
    if input.is_empty() {
        return Ok(vec![]);
    }
    let native =
        centurions_content::native_text::NativeText::load(content, std::path::Path::new("."))?;
    let typed_encounter::PasswordSelection::Selected(selected) =
        typed_encounter::select_password(record, &input, native.bytes(0x3ffc50, 256)?)?
    else {
        return Ok(vec![]);
    };
    finish_typed_selection(state, content, selected)
}

pub fn typed_record<'a>(state: &'a Session, content: &'a Content) -> Result<&'a [u8], RuleError> {
    let map = active_map(state, content)?;
    let position = state
        .exploration
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("no active mission".into()))?;
    let Some(Presentation::Typed { id, .. }) = position.presentation.as_ref() else {
        return Err(RuleError::Invalid("no typed encounter active".into()));
    };
    Ok(span(
        if state.typed_encounters.is_empty() {
            &map.typed_encounters
        } else {
            &state.typed_encounters
        },
        *id as usize * typed_encounter::RECORD_BYTES,
        typed_encounter::RECORD_BYTES,
    )?)
}

pub(crate) fn check_item_actor(state: &Session, warrior: usize) -> Result<(), RuleError> {
    let actor = state
        .warriors
        .get(warrior)
        .ok_or("item warrior unavailable")?;
    if let Some(combat) = &state.combat {
        let warning = if combat.active_body() != Some(warrior as i16) {
            Some(10)
        } else if word(&actor.bytes, 0x40)? < 1 {
            Some(17)
        } else {
            None
        };
        if let Some(index) = warning {
            return Err(RuleError::Refused {
                feedback: vec![crate::feedback::NativeFeedback::Warning { index }],
            });
        }
    }
    Ok(())
}

fn refuse_combat_script_item(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    item: i16,
    encounter: bool,
) -> Result<Vec<Event>, RuleError> {
    check_item_actor(state, warrior)?;
    state.load_item_work(content, item)?;
    let mut feedback = Vec::new();
    if !encounter && word(state.ammunition_work.item.as_ref().unwrap(), 44)? < 1100 {
        feedback.extend([88, 89].map(|index| crate::feedback::NativeFeedback::Warning { index }));
    }
    feedback.push(crate::feedback::NativeFeedback::Warning { index: 52 });
    Err(RuleError::RefusedAfterEffects { feedback })
}

fn use_encounter_item(
    state: &mut Session,
    content: &Content,
    warrior: usize,
    slot: usize,
) -> Result<Vec<Event>, RuleError> {
    check_item_actor(state, warrior)?;
    let item = state.warriors[warrior]
        .inventory()?
        .get(slot)
        .cloned()
        .ok_or("inventory item unavailable")?;
    let selected = typed_encounter::select_item(typed_record(state, content)?, item.id())?;
    let definition = crate::ammunition::definition(content, item.id())?;
    let class = word(&definition, 26)?.wrapping_abs();
    if word(&definition, 26)? == 27 {
        return crate::item_modifiers::use_item(state, content, warrior, slot);
    }
    if class == 23 {
        if state.combat.is_some() {
            return refuse_combat_script_item(state, content, warrior, item.id(), true);
        }
        let frame = state
            .exploration
            .as_ref()
            .unwrap()
            .script
            .as_ref()
            .ok_or("typed item caller missing")?;
        let manual = frame.item_return;
        let walking =
            !frame.scheduled && frame.origin.is_some() && state.time_continuation.is_none();
        let calendar = frame.scheduled
            && state.time_continuation.is_some()
            && state.combat_story.is_none()
            && state.round_dispatch.is_none();
        let regional = frame.scheduled
            && frame.origin.is_none()
            && state.time_continuation.is_none()
            && state.combat_story.is_none()
            && state.round_dispatch.is_none()
            && matches!(
                state.movement_continuation,
                Some(MovementContinuation::RegionalReturn { .. })
            );
        let shared_death = state
            .combat_story
            .as_ref()
            .is_some_and(|story| story.shared_dispatch);
        let completed_round = state.round_dispatch.is_some()
            && state.scripted_cleanup == Some(crate::story_cleanup::Stage::Complete);
        if !manual && !walking && !calendar && !regional && !shared_death && !completed_round {
            return Err(unsupported(
                "story.encounter_item",
                "script-item return caller",
            ));
        }
        let map = active_map(state, content)?;
        let used = crate::story_item_use::use_scenario_item(
            &state.warriors,
            warrior,
            slot,
            content,
            &map.combat
                .as_ref()
                .ok_or("item movement tables unavailable")?
                .race_moves,
            crate::story_item_use::ScriptItemContext::Encounter,
            state.inventory_record.as_ref(),
        )?;
        // Only manual buttonchoice10a9ee decodes the script-item return.
        let record = if manual {
            Some(
                map.script(used.return_code.wrapping_abs().wrapping_sub(100))?
                    .to_vec(),
            )
        } else {
            None
        };
        state.load_item_work(content, item.id())?;
        state.warriors = used.warriors;
        state.selected_warrior = warrior;
        state.party_mask[warrior] = 1;
        let position = state.exploration.as_mut().unwrap();
        position.presentation = None;
        position.script = record.map(|record| ScriptState {
            record,
            cursor: 0,
            map_slot: 0,
            item_return: false,
            scheduled: true,
            menu: None,
            persist: false,
            origin: None,
        });
        let mut effects: Vec<_> = used
            .sounds
            .into_iter()
            .map(|resource_id| Event::Sound { resource_id })
            .collect();
        if manual || shared_death || completed_round {
            effects.extend(run_script(state, content)?);
        }
        return Ok(effects);
    }
    let activating =
        word(&definition, 44)? > 1100 && (word(&item.0, 2)? > 0 || word(&definition, 20)? < 0);
    if activating && class == 28 && state.combat.is_none() {
        return crate::outdoor_psi::begin_item(state, content, warrior, slot);
    }
    if class == 27 || activating {
        return Err(unsupported(
            "story.encounter_item",
            "activating item encounter caller",
        ));
    }
    // Items12b04a reloads the item and clears targets;12c422 selects its owner.
    state.load_item_work(content, item.id())?;
    state.party_mask.fill(0);
    state.party_mask[warrior] = 1;
    state.selected_warrior = warrior;
    finish_typed_selection(state, content, selected)
}

pub(crate) fn finish_typed_selection(
    state: &mut Session,
    content: &Content,
    selected: i8,
) -> Result<Vec<Event>, RuleError> {
    let record = typed_record(state, content)?.to_vec();
    let position = state.exploration.as_mut().unwrap();
    let script = position
        .script
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("typed script missing".into()))?;
    let remaining = script
        .menu
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("typed continuation missing".into()))?
        .remaining;
    match typed_encounter::continuation(&record, selected, remaining)? {
        typed_encounter::TypedContinuation::Branch { body, .. } => {
            script.record[8..40].copy_from_slice(&body);
            script.cursor = 0;
        }
        typed_encounter::TypedContinuation::Cancel => return cancel_menu(state, content),
        typed_encounter::TypedContinuation::Abort => {
            position.script = None;
        }
    }
    position.presentation = None;
    run_script(state, content)
}

pub fn take_loot(
    state: &mut Session,
    content: &Content,
    slot: usize,
    warrior: Option<usize>,
) -> Result<Vec<Event>, RuleError> {
    if !state
        .exploration
        .as_ref()
        .is_some_and(|position| matches!(position.presentation, Some(Presentation::Loot)))
    {
        return Err(RuleError::Invalid("no loot choice active".into()));
    }
    let item = state
        .booty_items
        .get(slot)
        .filter(|item| item.id() != 0)
        .ok_or_else(|| RuleError::Invalid("loot slot is empty".into()))?
        .clone();
    state.load_item_work(content, item.id())?;
    let mut effects = Vec::new();
    if (200..400).contains(&item.id()) && item.0[10] == 26 {
        effects = items::add_ammunition(state, content, &item)?;
    } else if let Some(index) = warrior {
        let recipient = state
            .warriors
            .get_mut(index)
            .ok_or_else(|| RuleError::Invalid("recipient outside squad".into()))?;
        let stacked =
            item.id() == 700 && recipient.inventory()?.iter().any(|owned| owned.id() == 700);
        let race = word(&recipient.bytes, 0x32)?.wrapping_sub(1);
        let movement = *content
            .map(state.exploration.as_ref().unwrap().map)
            .unwrap()
            .combat
            .as_ref()
            .unwrap()
            .race_moves
            .get(race as usize)
            .ok_or_else(|| RuleError::Invalid("recipient race unavailable".into()))?;
        let collected = if item.0[10] != 23 {
            crate::equipment::collect_item(
                recipient,
                &item,
                content,
                content
                    .map(state.exploration.as_ref().unwrap().map)
                    .unwrap(),
            )?
        } else {
            story_collection::collect_story_item(recipient, &item, content, movement)?
        };
        effects.extend(
            collected
                .sounds
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id }),
        );
        effects.extend(
            collected
                .warnings
                .into_iter()
                .map(|warning| Event::CollectionRejected { warning }),
        );
        *recipient = collected.warrior;
        state.booty_items[slot] = collected.item;
        if !collected.collected {
            return Ok(effects);
        }
        if !stacked && let Some(last) = recipient.inventory()?.last() {
            let id = last.id();
            state.inventory_record = Some(item.clone());
            state.load_item_work(content, id)?;
        }
    } else {
        effects = insert(item, &mut state.crate_items, true)?;
        if effects.contains(&Event::StorageFull) {
            return Ok(effects);
        }
    }
    state.booty_items[slot].0[..2].fill(0);
    if let Some(limit) = &mut state.loot_limit
        && *limit != 0
    {
        *limit = limit.wrapping_sub(1);
        if *limit == 0 {
            state.booty_items.fill(InventoryRecord([0; 22]));
            if !needs_spell_confirmation(state) {
                effects.extend(continue_with_spells(state, content, false)?);
            }
        }
    }
    Ok(effects)
}

fn present_treasure(
    state: &mut Session,
    content: &Content,
    id: i16,
) -> Result<Vec<Event>, RuleError> {
    if id < 0 {
        return Err("negative treasure record".into());
    }
    let position = state
        .exploration
        .as_ref()
        .ok_or("treasure mission unavailable")?;
    let map = state
        .map(content, position.map)
        .ok_or("treasure map unavailable")?;
    let record = span(&map.treasure, id as usize * 88, 88)?;
    let prepared = crate::story_treasure::prepare(
        &state.warriors,
        record,
        content,
        &mut state.random,
        state.difficulty,
    )?;
    state.warriors = prepared.warriors;
    state.party_currency = state.party_currency.wrapping_add(prepared.credits);
    if let Some(record) = prepared.inventory_record {
        state.inventory_record = Some(record);
    }
    state.booty_items.fill(InventoryRecord([0; 22]));
    for (slot, item) in prepared.items.into_iter().enumerate() {
        state.loaded_item_field32 = Some(state.ammunition_work.load_item(content, item.id())?);
        state.booty_items[slot] = item;
    }
    state.exploration.as_mut().unwrap().presentation = Some(Presentation::Loot);
    Ok(vec![Event::StoryPresented])
}

fn finish_dispatch(
    state: &mut Session,
    content: &Content,
) -> Result<Option<Vec<Event>>, RuleError> {
    if state
        .round_dispatch
        .as_ref()
        .is_some_and(|frame| frame.owns_script(state))
    {
        return combat::runtime::finish_round_dispatch(state, content).map(Some);
    }
    if state
        .combat_story
        .as_ref()
        .is_some_and(|story| story.shared_dispatch && !story.waiting_child)
    {
        return combat::runtime::finish_death_story(state, content).map(Some);
    }
    Ok(None)
}

#[derive(Clone, Copy)]
pub(crate) enum StoryBattle {
    Ordinary,
    Group,
    Selected,
    Npc,
}

pub(crate) fn begin_story_battle(
    state: &mut Session,
    content: &Content,
    entry: StoryBattle,
    parameter: i16,
) -> Result<Vec<Event>, RuleError> {
    let round_owns = state
        .round_dispatch
        .as_ref()
        .is_some_and(|frame| frame.owns_script(state));
    let prior_round = state.round_dispatch.clone();
    let prior_cleanup = state.scripted_cleanup.clone();
    let outer = (round_owns
        && state.combat.is_some()
        && prior_cleanup != Some(crate::story_cleanup::Stage::Complete))
    .then(|| combat::round_stories::RoundBattleReturn {
        after_message: state.battle_after_message,
        treasure: state.battle_treasure,
        group: state.battle_group_script,
        npc: state.battle_npc.clone(),
    });
    let death_depth = combat::round_stories::RoundDispatch::death_depth(state);
    if let Some(frame) = state
        .round_dispatch
        .as_mut()
        .filter(|frame| !frame.child_battle)
    {
        frame.child_battle = true;
        frame.outer_battle = outer;
        frame.death_depth.get_or_insert(death_depth);
    }
    if prior_cleanup == Some(crate::story_cleanup::Stage::Complete) {
        state.scripted_cleanup = None;
    }
    let mut continuation = state.combat_story.take();
    let prior_wait = continuation.as_ref().map(|story| story.waiting_child);
    if let Some(story) = continuation.as_mut().filter(|_| !round_owns) {
        story.waiting_child = true;
    }
    let events = match entry {
        StoryBattle::Ordinary => combat::begin(state, content, parameter),
        StoryBattle::Group => combat::begin_group(state, content, parameter),
        StoryBattle::Selected => combat::begin_selected_battle(state, content, parameter),
        StoryBattle::Npc => combat::begin_npc(state, content, parameter),
    }?;
    if !events
        .iter()
        .any(|event| matches!(event, Event::CombatStarted { .. }))
    {
        state.round_dispatch = prior_round;
        state.scripted_cleanup = prior_cleanup;
        if let Some(story) = continuation.as_mut() {
            story.waiting_child = prior_wait.unwrap();
        }
    }
    combat::runtime::retain_death_parent(state, continuation);
    Ok(events)
}

fn finish_script(
    state: &mut Session,
    content: &Content,
    exhausted: bool,
) -> Result<Vec<Event>, RuleError> {
    let position = state
        .exploration
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("no active map".into()))?;
    let Some(mut script) = position.script.take() else {
        return Ok(vec![]);
    };
    if let Some(events) = finish_dispatch(state, content)? {
        return Ok(events);
    }
    let position = state.exploration.as_mut().unwrap();
    state.scripted_cleanup = None;
    if script.scheduled {
        return Ok(vec![]);
    }
    let origin = script.origin.clone().unwrap_or_else(|| {
        let key = long(&script.record, 0).unwrap_or(0);
        crate::ScriptOrigin {
            map: key / 10000,
            x: key % 100,
            y: key / 100 % 100,
            level: state.scenario_level,
        }
    });
    if exhausted && script.menu.is_none() {
        script.record[7] = 255;
        let headers = if origin.level != state.scenario_level {
            &mut state
                .visited_levels
                .get_mut(&origin.level)
                .and_then(|l| l.maps.get_mut(&origin.map))
                .ok_or("event origin level missing")?
                .event_headers
        } else if origin.map == position.map {
            &mut position.event_headers
        } else {
            &mut state
                .visited_maps
                .get_mut(&origin.map)
                .ok_or_else(|| RuleError::Invalid("event origin map missing".into()))?
                .event_headers
        };
        headers[script.map_slot * 42 + 7] = 255;
    }
    let current = crate::story_destination::Location {
        map: position.map,
        x: position.x,
        y: position.y,
        level: state.scenario_level,
    };
    let Some(plan) = crate::story_destination::destination(
        &script.record,
        current,
        state.phase == crate::Phase::Combat,
        false,
    )?
    else {
        return Ok(vec![]);
    };
    OpeningMission::index(plan.target.x, plan.target.y)?;
    content
        .map_at(state.mission.as_deref(), plan.target.level, plan.target.map)
        .ok_or_else(|| RuleError::Invalid("door destination map unavailable".into()))?;
    if script.persist {
        let record =
            crate::story_destination::persistent_record(&script.record, origin.x, origin.y)?;
        let headers = if origin.level != state.scenario_level {
            &mut state
                .visited_levels
                .get_mut(&origin.level)
                .and_then(|l| l.maps.get_mut(&origin.map))
                .ok_or("event origin level missing")?
                .event_headers
        } else if origin.map == position.map {
            &mut position.event_headers
        } else {
            &mut state
                .visited_maps
                .get_mut(&origin.map)
                .ok_or_else(|| RuleError::Invalid("event origin map missing".into()))?
                .event_headers
        };
        headers[script.map_slot * 42..(script.map_slot + 1) * 42].copy_from_slice(&record);
    }
    crate::mission::travel(state, content, plan.target)?;
    let position = state.exploration.as_ref().unwrap();
    Ok(vec![Event::Moved {
        x: position.x,
        y: position.y,
    }])
}

pub(crate) fn dialogue(
    map: &OpeningMission,
    warriors: &[Warrior],
    id: i16,
) -> Result<String, RuleError> {
    let mut text = map.dialogue(id)?;
    for index in 0..warriors.len() {
        let selected = if warriors[index].stamina()? < 1 {
            (0..warriors.len())
                .rev()
                .find(|i| warriors[*i].stamina().unwrap_or(0) > 0)
                .unwrap_or(index)
        } else {
            index
        };
        text = text.replace(&format!("^{}", index + 1), &warriors[selected].name);
    }
    Ok(text)
}

/// loaddoor2 preserves the first eight bytes; caller frames restore actions and byte40 only.
pub(crate) fn retain_script_stamp(state: &mut Session, stamp: u8) {
    fn write(script: Option<&mut ScriptState>, stamp: u8) {
        if let Some(byte) = script.and_then(|s| s.record.get_mut(41)) {
            *byte = stamp;
        }
    }
    write(
        state.exploration.as_mut().and_then(|p| p.script.as_mut()),
        stamp,
    );
    let mut round = state.round_dispatch.as_mut();
    while let Some(frame) = round {
        write(frame.suspended_script.as_mut(), stamp);
        round = frame.parent.as_deref_mut();
    }
    let mut death = state.combat_story.as_mut();
    while let Some(story) = death {
        write(story.suspended_script.as_mut(), stamp);
        death = story.parent.as_deref_mut();
    }
}

pub(crate) fn run_script(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let mut effects = Vec::new();
    for _ in 0..128 {
        if let Some(stamp) = state
            .exploration
            .as_ref()
            .and_then(|p| p.script.as_ref())
            .and_then(|s| s.record.get(41))
            .copied()
        {
            retain_script_stamp(state, stamp);
        }
        let map = active_map(state, content)?;
        let position = state.exploration.as_mut().unwrap();
        let Some(script) = position.script.as_mut() else {
            if let Some(closed) = finish_dispatch(state, content)? {
                effects.extend(closed);
            }
            return Ok(effects);
        };
        if script.cursor >= 8 {
            if let Some(menu) = script.menu.as_mut() {
                let before = menu.remaining;
                menu.remaining = before.wrapping_sub(1);
                if before > 1 {
                    let id = menu.id;
                    if menu.typed {
                        position.presentation = Some(present_typed(
                            map,
                            if state.typed_encounters.is_empty() {
                                &map.typed_encounters
                            } else {
                                &state.typed_encounters
                            },
                            script,
                            id,
                        )?);
                        effects.push(Event::StoryPresented);
                        return Ok(effects);
                    }
                    let table = if state.menus.is_empty() {
                        &map.menus
                    } else {
                        &state.menus
                    };
                    effects.push(Event::Sound { resource_id: 20005 });
                    position.presentation = present_menu(map, table, script, id)?;
                    if position.presentation.is_some() {
                        effects.push(Event::StoryPresented);
                        return Ok(effects);
                    }
                    continue;
                }
            }
            effects.extend(finish_script(state, content, true)?);
            return Ok(effects);
        }
        let cursor = usize::from(script.cursor);
        let opcode = word(&script.record, 8 + cursor * 2)?;
        let parameter = word(&script.record, 24 + cursor * 2)?;
        script.cursor += 1;
        match opcode {
            100 => {
                effects.extend(crate::story_cleanup::begin(state, content, parameter)?);
                if state.scripted_cleanup == Some(crate::story_cleanup::Stage::Survivors) {
                    return Ok(effects);
                }
            }
            101 => {
                // newland141340 returns before door persistence and destination handling.
                position.x -= i32::from(position.last_move[0]);
                position.y -= i32::from(position.last_move[1]);
                OpeningMission::index(position.x, position.y)?;
                position.script = None;
                state.movement_continuation = None;
                state.time_continuation = None;
                effects.push(Event::Moved {
                    x: position.x,
                    y: position.y,
                });
                return Ok(effects);
            }
            120 => {
                let (events, progress) = crate::story_actor::begin(state, content, parameter)?;
                effects.extend(events);
                match progress {
                    crate::story_actor::Progress::Waiting => return Ok(effects),
                    crate::story_actor::Progress::Stop => {
                        effects.extend(finish_script(state, content, false)?);
                        return Ok(effects);
                    }
                    crate::story_actor::Progress::Continue => {}
                }
            }
            119 | 122 | 123 | 124 | 125 => {
                let (events, stopped) =
                    crate::story_combat::apply(state, content, opcode, parameter)?;
                effects.extend(events);
                if stopped {
                    effects.extend(finish_script(state, content, false)?);
                    return Ok(effects);
                }
            }
            126 => {
                if state.round_dispatch.is_none() {
                    return Err(unsupported("story.round_jump", "native round-slot caller"));
                }
                let arena = state.combat.as_mut().ok_or("round combat missing")?;
                let slot = arena
                    .round_scripts
                    .pending
                    .as_ref()
                    .ok_or("round program missing")?
                    .slot;
                let extra = span(
                    &map.combat
                        .as_ref()
                        .ok_or("round tables missing")?
                        .encounters,
                    parameter.unsigned_abs() as usize * 10,
                    10,
                )?;
                let Some(target) = combat::round_stories::replacement(
                    &mut arena.round_scripts,
                    slot,
                    extra,
                    &mut state.random,
                )?
                else {
                    effects.extend(finish_script(state, content, false)?);
                    return Ok(effects);
                };
                script.record[8..].copy_from_slice(&map.script(target)?[8..]);
                script.cursor = 0;
            }
            17 | 18 => {
                let extra = span(
                    &map.combat.as_ref().unwrap().encounters,
                    parameter.unsigned_abs() as usize * 10,
                    10,
                )?;
                effects.extend(crate::story_spell::apply(
                    state,
                    content,
                    extra,
                    opcode == 18,
                )?);
                if state
                    .health_context
                    .as_ref()
                    .is_some_and(|health| health.unavailable >= state.warriors.len() as i8)
                {
                    effects.extend(crate::mission::defeat(state, content));
                    return Ok(effects);
                }
            }
            105 => state.suppress_allies = parameter,
            108 => {
                let extra = span(
                    &map.combat
                        .as_ref()
                        .ok_or("story combat tables missing")?
                        .encounters,
                    parameter.unsigned_abs() as usize * 10,
                    10,
                )?;
                state.warriors =
                    story_party::adjust_statistics(&state.warriors, &state.party_mask, extra)?;
            }
            36 => {
                effects.extend(crate::confiscation::apply(state, content, parameter)?);
                if state.exploration.as_ref().unwrap().presentation.is_some() {
                    return Ok(effects);
                }
            }
            6 => {
                if crate::shop::visit(state, content, parameter)? {
                    effects.push(Event::StoryPresented);
                    return Ok(effects);
                }
            }
            -32 => {
                let extra = span(
                    &map.combat
                        .as_ref()
                        .ok_or("clinic extra records missing")?
                        .encounters,
                    parameter.unsigned_abs() as usize * 10,
                    10,
                )?;
                let percent = word(extra, 0)?;
                let sound = word(extra, 2)?;
                let text = word(extra, 4)?;
                if text != 0 {
                    return Err(RuleError::Unsupported {
                        action: "story.clinic",
                        dependency: "clinic introductory textbox caller",
                    });
                }
                if sound != 0 {
                    effects.push(Event::Sound { resource_id: sound });
                }
                effects.extend(crate::clinic::visit(state, percent)?);
                return Ok(effects);
            }
            29 => {
                let index = usize::from(parameter.unsigned_abs());
                let flag = state
                    .notes
                    .get_mut(index)
                    .ok_or("map or note index unavailable")?;
                *flag = 1;
                if parameter > -1 {
                    effects.push(Event::Feedback(crate::feedback::NativeFeedback::Flash {
                        address: 0x401c28,
                        x: 30,
                        y: 355,
                        wait: 0,
                        sound: 30005,
                    }));
                }
            }
            113 => crate::story_map_picture::assign(&mut state.map_picture, parameter),
            98 => {}
            15 | 16 => {
                let extra = span(
                    &map.combat.as_ref().unwrap().encounters,
                    parameter.unsigned_abs() as usize * 10,
                    10,
                )?;
                let context = crate::story_health::context(
                    &state.warriors,
                    state.selected_warrior,
                    state.health_context.as_ref(),
                );
                let outcome = crate::story_health::party_health(
                    &state.warriors,
                    extra,
                    opcode == 16,
                    &state.party_mask,
                    &context,
                    &mut state.random,
                )?;
                state.warriors = outcome.warriors;
                state.health_context = Some(outcome.context);
                let mut party_lost = false;
                for event in outcome.events {
                    match event {
                        crate::story_health::HealthEvent::Flash { address, sound } => {
                            effects.push(Event::Feedback(crate::feedback::NativeFeedback::Flash {
                                address,
                                x: 100,
                                y: 30,
                                wait: 100,
                                sound,
                            }))
                        }
                        crate::story_health::HealthEvent::Animation { effect, .. } => {
                            effects.push(Event::OutdoorEffect {
                                effect: i16::from(effect),
                                x: position.x as i16,
                                y: position.y as i16,
                                size: 32,
                                delay: 0,
                            })
                        }
                        crate::story_health::HealthEvent::Sound(resource_id) => {
                            effects.push(Event::Sound { resource_id })
                        }
                        crate::story_health::HealthEvent::Dialogue(id) => {
                            position.presentation = Some(Presentation::Dialogue {
                                id,
                                text: map.dialogue(id)?,
                            });
                            effects.push(Event::StoryPresented);
                        }
                        crate::story_health::HealthEvent::Defeated { warrior, .. } => {
                            if let Some(combat) = state.combat.as_mut() {
                                crate::combat::lifecycle::kill_body(
                                    combat,
                                    &mut state.warriors,
                                    warrior as i16,
                                )?;
                            }
                        }
                        crate::story_health::HealthEvent::PartyLost => {
                            party_lost = true;
                        }
                    }
                }
                if party_lost {
                    effects.extend(crate::mission::defeat(state, content));
                    return Ok(effects);
                }
                if position.presentation.is_some() {
                    return Ok(effects);
                }
            }
            30 => {
                let extra = span(
                    &map.combat.as_ref().unwrap().encounters,
                    parameter.unsigned_abs() as usize * 10,
                    10,
                )?;
                state.party_mask = crate::story_party::filter_party(
                    &state.warriors,
                    state.party_mask,
                    cursor,
                    extra,
                    &mut state.random,
                )?;
            }
            52 => {
                let extra = span(
                    &map.combat.as_ref().unwrap().encounters,
                    parameter.unsigned_abs() as usize * 10,
                    10,
                )?;
                let packet = state
                    .combat
                    .as_ref()
                    .and_then(|c| c.attack_packet.as_ref())
                    .or(state.attack_packet.as_ref());
                let context = crate::story_party::PartySaveContext {
                    casting: false,
                    packet_type: packet.map_or(0, |p| p[9] as i8),
                    bonus: packet.map_or(0, |p| p[6] as i8),
                    bonus_per_attack: packet.map_or(0, |p| p[8] as i8),
                    attack_count: state.attack_count,
                    mind_shield: word(&state.party_buffs, 14)? != 0,
                };
                state.party_mask = crate::story_party::select_party(
                    &state.warriors,
                    state.selected_warrior,
                    extra,
                    context,
                    &mut state.random,
                )?;
            }
            54 => {
                let extra_index = usize::try_from(parameter)
                    .map_err(|_| RuleError::Invalid("negative schedule extra index".into()))?;
                let extra = span(
                    &map.combat.as_ref().unwrap().encounters,
                    extra_index * 10,
                    10,
                )?;
                crate::story_schedule::edit(&mut position.schedule, extra, position.day)?;
            }
            107 => {
                effects.extend(begin_story_battle(
                    state,
                    content,
                    StoryBattle::Group,
                    parameter,
                )?);
                return Ok(effects);
            }
            48 => {
                effects.extend(begin_story_battle(
                    state,
                    content,
                    StoryBattle::Selected,
                    parameter,
                )?);
                return Ok(effects);
            }
            56 => {
                effects.extend(begin_story_battle(
                    state,
                    content,
                    StoryBattle::Npc,
                    parameter,
                )?);
                return Ok(effects);
            }
            14 if parameter == 1 => {
                state.party_mask.fill(0);
                position.presentation = Some(Presentation::WarriorChoice { extra: -1 });
                effects.push(Event::StoryPresented);
                return Ok(effects);
            }
            0 => {}
            1 => {
                let text = dialogue(map, &state.warriors, parameter)?;
                position.presentation = Some(Presentation::Dialogue {
                    id: parameter,
                    text,
                });
                effects.push(Event::StoryPresented);
                return Ok(effects);
            }
            9 => effects.push(Event::Sound {
                resource_id: parameter,
            }),
            73 => effects.push(Event::Delay {
                ticks: crate::story_delay::elapsed_ticks(parameter)?,
            }),
            3 => {
                let previous = cursor
                    .checked_sub(1)
                    .ok_or_else(|| RuleError::Invalid("question text missing".into()))?;
                let text = map.dialogue(word(&script.record, 24 + previous * 2)?)?;
                position.presentation = Some(Presentation::Question {
                    extra: parameter,
                    text,
                });
                effects.push(Event::Sound { resource_id: 20005 });
                effects.push(Event::StoryPresented);
                return Ok(effects);
            }
            23 => {
                let extra = span(
                    &map.combat.as_ref().unwrap().encounters,
                    parameter as usize * 10,
                    10,
                )?;
                let target = i32::from(word(extra, 0)?);
                let authored = content
                    .map(target)
                    .ok_or_else(|| RuleError::Invalid("region target map unavailable".into()))?;
                if !(0..20).contains(&word(extra, 2)?) {
                    return Err(RuleError::Invalid("region index unavailable".into()));
                }
                let regions = if target == position.map {
                    &mut position.region_state
                } else {
                    &mut state
                        .visited_maps
                        .entry(target)
                        .or_insert_with(|| MapState::authored(authored))
                        .regions
                };
                let region = word(extra, 2)? as usize;
                regions[160 + region * 2..162 + region * 2].copy_from_slice(&extra[4..6]);
                for (source, target) in [(6, 200 + region * 4), (8, 202 + region * 4)] {
                    if word(extra, source)? >= 0 {
                        regions[target..target + 2].copy_from_slice(&extra[source..source + 2]);
                    }
                }
            }
            -12 => {
                let extra = span(
                    &map.combat.as_ref().unwrap().encounters,
                    parameter.unsigned_abs() as usize * 10,
                    10,
                )?;
                let target = i32::from(word(extra, 0)?);
                let authored = content
                    .map_at(state.mission.as_deref(), state.scenario_level, target)
                    .ok_or("object clearing map unavailable")?;
                let first = (i32::from(word(extra, 2)?), i32::from(word(extra, 6)?));
                let last = (i32::from(word(extra, 4)?), i32::from(word(extra, 8)?));
                OpeningMission::index(first.0, first.1)?;
                OpeningMission::index(last.0, last.1)?;
                let objects = if target == position.map {
                    &mut position.objects
                } else {
                    &mut state
                        .visited_maps
                        .entry(target)
                        .or_insert_with(|| MapState::authored(authored))
                        .objects
                };
                for x in first.0..=last.0 {
                    for y in first.1..=last.1 {
                        objects[OpeningMission::index(x, y)?] = 0;
                    }
                }
            }
            69 => {
                let extra = span(
                    &map.combat
                        .as_ref()
                        .ok_or("script extra records unavailable")?
                        .encounters,
                    parameter.unsigned_abs() as usize * 10,
                    10,
                )?;
                // newland 0x143ef6 copies the low byte of each EXTRA word.
                state.psi_disabled = extra[1];
                state.enemy_spell_disabled = extra[3];
                state.energy_recovery_disabled = extra[5];
            }
            68 => {
                let extra = span(
                    &map.combat.as_ref().unwrap().encounters,
                    parameter.unsigned_abs() as usize * 10,
                    10,
                )?;
                position.fatigue = match word(extra, 0)? {
                    1 => 135,
                    2 => 4,
                    3 => (word(extra, 4)? / 100).wrapping_mul(position.fatigue),
                    _ => position.fatigue,
                };
            }
            7 | 12 | 13 => {
                let extra = span(
                    &map.combat.as_ref().unwrap().encounters,
                    parameter as usize * 10,
                    10,
                )?;
                let target = i32::from(word(extra, 0)?);
                let authored = content
                    .map(target)
                    .ok_or_else(|| RuleError::Invalid("map mutation target unavailable".into()))?;
                if opcode == 12 {
                    let mutation = travel::tile_mutation(extra)?;
                    if target == position.map {
                        let layer = match mutation.layer {
                            travel::TileLayer::Floor => &mut position.floor,
                            travel::TileLayer::Object => &mut position.objects,
                        };
                        *layer = travel::apply_tile(layer, &mutation)?;
                    } else {
                        let records = state
                            .visited_maps
                            .entry(target)
                            .or_insert_with(|| MapState::authored(authored));
                        let layer = match mutation.layer {
                            travel::TileLayer::Floor => &mut records.floor,
                            travel::TileLayer::Object => &mut records.objects,
                        };
                        *layer = travel::apply_tile(layer, &mutation)?;
                    }
                } else {
                    let headers = if target == position.map {
                        &mut position.event_headers
                    } else {
                        &mut state
                            .visited_maps
                            .entry(target)
                            .or_insert_with(|| MapState::authored(authored))
                            .event_headers
                    };
                    *headers = if opcode == 13 {
                        travel::set_event_chances(headers, extra)?
                    } else {
                        let replacement = travel::event_replacement(extra)?;
                        travel::replace_event_script(
                            headers,
                            replacement.slot,
                            &map.script(replacement.script)?,
                        )?
                    };
                }
            }
            24 => {
                effects.extend(finish_script(state, content, false)?);
                return Ok(effects);
            }
            25 => {
                script.persist = !script.scheduled;
            }
            26 => {
                position.presentation = Some(Presentation::ScriptPause);
                effects.push(Event::Feedback(crate::feedback::script_pause()));
                return Ok(effects);
            }
            128 => {
                let extra = span(
                    &map.combat.as_ref().unwrap().encounters,
                    parameter as usize * 10,
                    10,
                )?;
                effects.push(Event::OutdoorEffect {
                    effect: word(extra, 0)?,
                    x: word(extra, 2)?,
                    y: word(extra, 4)?,
                    size: word(extra, 6)?,
                    delay: word(extra, 8)?,
                });
            }
            22 => {
                let tables = map.combat.as_ref().unwrap();
                let extra = span(&tables.encounters, parameter as usize * 10, 10)?;
                let consumed = story_inventory::consume_party_items_retained(
                    &state.warriors,
                    content,
                    &tables.race_moves,
                    extra,
                    state.inventory_record.as_ref(),
                )?;
                if let Some(definition) = consumed.definition {
                    state.loaded_item_field32 = Some(word(&definition, 32)?);
                    state.ammunition_work.item = Some(definition);
                } else if consumed.consumed != 0 {
                    let loaded = word(extra, if word(extra, 4)? == 3 { 8 } else { 0 })?;
                    state.loaded_item_field32 =
                        Some(state.ammunition_work.load_item(content, loaded)?);
                }
                state.inventory_record = consumed.record;
                state.warriors = consumed.warriors;
                effects.extend(consumed.events);
            }
            31 => {
                position.presentation = Some(Presentation::WarriorChoice { extra: parameter });
                effects.push(Event::StoryPresented);
                return Ok(effects);
            }
            45 => {
                let extra = span(
                    &map.combat.as_ref().unwrap().encounters,
                    parameter as usize * 10,
                    10,
                )?;
                let target = if word(extra, 0)? < 0 {
                    position.map
                } else {
                    i32::from(word(extra, 0)?)
                };
                let x = if word(extra, 2)? < 0 {
                    position.x
                } else {
                    i32::from(word(extra, 2)?)
                };
                let y = if word(extra, 4)? < 0 {
                    position.y
                } else {
                    i32::from(word(extra, 4)?)
                };
                OpeningMission::index(x, y)?;
                if target != position.map {
                    let authored = content
                        .map_at(state.mission.as_deref(), state.scenario_level, target)
                        .ok_or_else(|| {
                            RuleError::Invalid("travel target map unavailable".into())
                        })?;
                    let records = state
                        .visited_maps
                        .remove(&target)
                        .unwrap_or_else(|| MapState::authored(authored));
                    state
                        .visited_maps
                        .insert(position.map, position.map_state());
                    // The script stays active across loadland; only its coordinates change.
                    position.restore_map(target, records);
                }
                position.x = x;
                position.y = y;
                let script = position.script.as_mut().unwrap();
                script.record[4] = position.map as u8;
                script.record[5] = x as u8;
                script.record[6] = y as u8;
                effects.push(Event::Moved { x, y });
                if word(extra, 6)? != 0 {
                    effects.push(Event::Sound {
                        resource_id: word(extra, 6)?,
                    });
                }
                if word(extra, 8)? != 0 {
                    position.presentation = Some(Presentation::Dialogue {
                        id: word(extra, 8)?,
                        text: map.dialogue(word(extra, 8)?)?,
                    });
                    effects.push(Event::StoryPresented);
                    return Ok(effects);
                }
            }
            4 => {
                effects.push(Event::Sound { resource_id: 20005 });
                let table = if state.menus.is_empty() {
                    &map.menus
                } else {
                    &state.menus
                };
                position.presentation = present_menu(map, table, script, parameter)?;
                if position.presentation.is_some() {
                    effects.push(Event::StoryPresented);
                    return Ok(effects);
                }
            }
            5 => {
                script.menu = None;
                position.presentation = Some(present_typed(
                    map,
                    if state.typed_encounters.is_empty() {
                        &map.typed_encounters
                    } else {
                        &state.typed_encounters
                    },
                    script,
                    parameter,
                )?);
                effects.push(Event::Sound { resource_id: 20005 });
                effects.push(Event::StoryPresented);
                return Ok(effects);
            }
            21 | 76 | 77 | 85 | 87 => {
                let extra = span(
                    &map.combat.as_ref().unwrap().encounters,
                    parameter as usize * 10,
                    10,
                )?;
                let continuation = match opcode {
                    21 => story_variables::party_item_condition(&state.warriors, extra)?,
                    76 => {
                        let update = story_variables::update_variable(&state.flags, extra)?;
                        state.flags = update.flags;
                        update.continuation
                    }
                    77 => story_variables::variable_predicate(&state.flags, extra)?.continuation,
                    85 => {
                        for (offset, sound) in [(6, true), (8, false)] {
                            let id = word(extra, offset)?;
                            if id != 0 {
                                if sound {
                                    effects.push(Event::Sound { resource_id: id });
                                } else {
                                    effects.push(Event::Message {
                                        id,
                                        text: map.dialogue(id)?,
                                    });
                                }
                            }
                        }
                        let target = state.random.range(word(extra, 2)?, word(extra, 4)?);
                        story_variables::branch(word(extra, 0)?, target)
                    }
                    _ => {
                        let matched = state
                            .allies
                            .as_chunks::<288>()
                            .0
                            .iter()
                            .take(state.ally_count)
                            .any(|record| {
                                i16::from(record[0x103] as i8) == word(extra, 0).unwrap()
                            });
                        if matched || word(extra, 4)? == 0 {
                            story_variables::branch(
                                word(extra, 2)?,
                                word(extra, if matched { 6 } else { 8 })?,
                            )
                        } else {
                            story_variables::StoryContinuation::Continue
                        }
                    }
                };
                match continuation {
                    story_variables::StoryContinuation::Continue => {}
                    story_variables::StoryContinuation::ReplaceScript(id) => {
                        script.record[8..].copy_from_slice(&map.script(id)?[8..]);
                        script.cursor = 0;
                    }
                    story_variables::StoryContinuation::Encounter(id) => {
                        script.menu = None;
                        let table = if state.menus.is_empty() {
                            &map.menus
                        } else {
                            &state.menus
                        };
                        position.presentation = present_menu(map, table, script, id)?;
                        if position.presentation.is_some() {
                            effects.push(Event::StoryPresented);
                            return Ok(effects);
                        }
                    }
                    story_variables::StoryContinuation::TypedEncounter(id) => {
                        script.menu = None;
                        position.presentation = Some(present_typed(
                            map,
                            if state.typed_encounters.is_empty() {
                                &map.typed_encounters
                            } else {
                                &state.typed_encounters
                            },
                            script,
                            id,
                        )?);
                        effects.push(Event::StoryPresented);
                        return Ok(effects);
                    }
                    story_variables::StoryContinuation::EndScript => {
                        position.script = None;
                        return Ok(effects);
                    }
                }
            }
            35 => {
                let menu = script.menu.as_ref().ok_or_else(|| {
                    RuleError::Invalid("menu mutation has no active encounter".into())
                })?;
                let id = menu.id;
                if state.menus.is_empty() {
                    state.menus = map.menus.clone();
                }
                let offset = id as usize * story::MENU_RECORD_BYTES;
                let replacement = story::disable_menu_entry(
                    span(&state.menus, offset, story::MENU_RECORD_BYTES)?,
                    parameter,
                )?;
                state.menus[offset..offset + story::MENU_RECORD_BYTES]
                    .copy_from_slice(&replacement);
                script.record = menu.original.clone();
                position.presentation = present_menu(map, &state.menus, script, id)?;
                if position.presentation.is_some() {
                    effects.push(Event::StoryPresented);
                    return Ok(effects);
                }
            }
            33 | 38 | 42 | 46 => {
                let extra = span(
                    &map.combat.as_ref().unwrap().encounters,
                    usize::from(parameter.unsigned_abs()) * 10,
                    10,
                )?;
                let jump = if opcode == 33 {
                    let payment = crate::recovered_inventory::takegold(
                        &crate::recovered_inventory::GoldState {
                            party_gold: state.party_currency,
                            highest_party_index: state.warriors.len() as i8 - 1,
                            warriors: state.warriors.clone(),
                        },
                        word(extra, 0)?,
                    )?;
                    state.party_currency = payment.state.party_gold;
                    state.warriors = payment.state.warriors;
                    for effect in payment.effects {
                        if let crate::recovered_inventory::InventoryEffect::Warning(index) = effect
                        {
                            effects.push(Event::Feedback(
                                crate::feedback::NativeFeedback::Warning { index },
                            ));
                        }
                    }
                    let paid = payment.native_return != 0;
                    if !paid && word(extra, 2)? == -1 {
                        script.cursor = 6;
                    }
                    if matches!(word(extra, 2)?, 2)
                        || (word(extra, 2)? == 0 && !paid)
                        || (word(extra, 2)? == 1 && paid)
                    {
                        story::condition_jump(extra)?
                    } else {
                        None
                    }
                } else if opcode == 38 {
                    if story_inventory::item_prerequisite_matches(&state.warriors, extra)? {
                        story::condition_jump(extra)?
                    } else {
                        None
                    }
                } else if opcode == 42 {
                    if state.random.bound(100) <= word(extra, 0)? {
                        match word(extra, 2)? {
                            -2 => {
                                script.cursor = 8;
                                None
                            }
                            1 => story::condition_jump(extra)?,
                            2 => Some(story::ConditionalJump::EndMenu),
                            _ => None,
                        }
                    } else {
                        None
                    }
                } else {
                    story::flag_condition_jump(&state.flags, extra)?
                };
                if let Some(jump) = jump {
                    match jump {
                        story::ConditionalJump::ReplaceScript(id) => {
                            let replacement = map.script(id)?;
                            script.record[8..].copy_from_slice(&replacement[8..]);
                            script.cursor = 0;
                        }
                        story::ConditionalJump::MenuBranch { branch, start } => {
                            let id = script
                                .menu
                                .as_ref()
                                .ok_or_else(|| {
                                    RuleError::Invalid(
                                        "conditional branch has no active menu".into(),
                                    )
                                })?
                                .id;
                            let table = if state.menus.is_empty() {
                                &map.menus
                            } else {
                                &state.menus
                            };
                            let menu = span(
                                table,
                                id as usize * story::MENU_RECORD_BYTES,
                                story::MENU_RECORD_BYTES,
                            )?;
                            script.record[8..40].copy_from_slice(&story::dispatch_menu_branch(
                                menu,
                                branch as i8 + 1,
                            )?);
                            script.cursor = start;
                        }
                        story::ConditionalJump::TypedBranch { branch, start } => {
                            let id = script
                                .menu
                                .as_ref()
                                .filter(|m| m.typed)
                                .ok_or("conditional branch has no active typed encounter")?
                                .id;
                            let table = if state.typed_encounters.is_empty() {
                                &map.typed_encounters
                            } else {
                                &state.typed_encounters
                            };
                            let record = span(
                                table,
                                id as usize * typed_encounter::RECORD_BYTES,
                                typed_encounter::RECORD_BYTES,
                            )?;
                            script.record[8..40].copy_from_slice(&story::dispatch_menu_branch(
                                record,
                                branch as i8 + 1,
                            )?);
                            script.cursor = start;
                        }
                        story::ConditionalJump::EndMenu => {
                            position.script = None;
                            return Ok(effects);
                        }
                        story::ConditionalJump::LastAction => {
                            script.cursor = 7;
                        }
                    }
                }
            }
            47 => {
                state.flags = story::set_mission_flag(&state.flags, parameter)?;
            }
            109 => {
                state.mission_score = story::add_mission_score(state.mission_score, parameter);
            }
            89 => {
                let definition = span(
                    &map.combat.as_ref().unwrap().monsters,
                    parameter as usize * story_allies::ALLY_BYTES,
                    story_allies::ALLY_BYTES,
                )?;
                let addition = story_allies::add_ally(
                    &state.allies,
                    state.ally_count,
                    definition,
                    state.difficulty,
                    &mut state.random,
                )?;
                state.allies = addition.records;
                state.ally_count = addition.count;
                if let Some(warning) = addition.warning {
                    effects.push(Event::CollectionRejected { warning });
                } else {
                    effects.push(Event::AllyAdded {
                        definition: parameter,
                    });
                }
            }
            88 => {
                let removed = crate::story_remove_allies::remove_class(
                    &state.allies,
                    state.ally_count,
                    parameter,
                )?;
                state.allies = removed.records;
                state.ally_count = removed.count;
            }
            8 => {
                // newland1434d4 copies the current map's mutable header, not Data ED3.
                let slot = usize::try_from(parameter)
                    .map_err(|_| RuleError::Invalid("negative event header slot".into()))?;
                let replacement = span(&position.event_headers, slot * 42, 42)?;
                script.record[8..41].copy_from_slice(&replacement[8..41]);
                script.cursor = 0;
            }
            39 => {
                let replacement = map.script(parameter)?;
                script.record[8..].copy_from_slice(&replacement[8..]);
                script.cursor = 0;
            }
            62 => {
                if !map.movies.contains_key(&parameter) && parameter != 1000 {
                    return Err(unsupported("story.movie", "other TEXT resources"));
                }
                position.presentation = Some(Presentation::Prologue {
                    resource_id: parameter,
                });
                effects.push(Event::StoryPresented);
                return Ok(effects);
            }
            111 => {
                let extra = span(
                    &map.combat.as_ref().unwrap().encounters,
                    parameter.unsigned_abs() as usize * 10,
                    10,
                )?;
                let id = word(extra, 0)?;
                if id <= 0 {
                    crate::shop::visit_restricted(state, content, extra)?;
                    effects.push(Event::StoryPresented);
                } else {
                    state.loot_limit = if word(extra, 4)?.wrapping_add(1) != 0 {
                        Some(word(extra, 2)?)
                    } else {
                        None
                    };
                    effects.extend(present_treasure(state, content, id)?);
                }
                return Ok(effects);
            }
            10 => {
                effects.extend(present_treasure(state, content, parameter)?);
                return Ok(effects);
            }
            110 => {
                if parameter > 0 {
                    position.presentation = Some(Presentation::StageChange { parameter });
                    effects.push(Event::StoryPresented);
                    return Ok(effects);
                }
                effects.extend(mission::begin_stage(state, content, parameter)?);
                return Ok(effects);
            }
            11 => {
                let awarded =
                    story_inventory::award_story_experience(&state.warriors, i32::from(parameter))?;
                state.warriors = awarded.warriors;
                position.presentation = Some(Presentation::Loot);
                effects.push(Event::StoryPresented);
                return Ok(effects);
            }
            2 => {
                effects.extend(begin_story_battle(
                    state,
                    content,
                    StoryBattle::Ordinary,
                    parameter,
                )?);
                return Ok(effects);
            }
            _ => {
                return Err(unsupported(
                    "story.opcode",
                    "this story operation is not recovered",
                ));
            }
        }
    }
    Err(RuleError::Invalid(
        "story exceeds instruction budget".into(),
    ))
}
#[cfg(all(test, feature = "original-fixtures"))]
#[path = "../tests/unit/exploration_reference.rs"]
mod tests;
