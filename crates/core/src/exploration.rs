use super::*;
use centurions_content::{
    mission::{OpeningMission, long},
    span, word,
};

fn unsupported(action: &'static str, dependency: &'static str) -> RuleError {
    RuleError::Unsupported { action, dependency }
}
fn active_map<'a>(state: &Session, content: &'a Content) -> Result<&'a OpeningMission, RuleError> {
    content
        .map(
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
    let used =
        crate::story_item_use::use_scenario_item(&state.warriors, warrior, slot, content, races)?;
    let id = state.warriors[warrior].inventory()?[slot].id();
    state.load_item_work(content, id)?;
    state.warriors = used.warriors;
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
    state.movement_continuation = Some(if event_branch {
        MovementContinuation::Header
    } else {
        MovementContinuation::Secrets
    });
    effects.extend(super::time::advance(
        state,
        content,
        ticks as u8,
        map.minutes_per_tick,
    )?);
    if ticks != 0 {
        let region = regional_selection(
            &mut state.exploration.as_mut().unwrap().region_state,
            (px, py),
            &mut state.random,
        )?;
        match region {
            Regional::Battle(battle, advantage) => {
                effects.extend(super::combat::begin_battle(
                    state, content, battle, advantage,
                )?);
                return Ok(effects);
            }
            Regional::Script(id) => {
                state.exploration.as_mut().unwrap().script = Some(ScriptState {
                    record: map.script(id)?.to_vec(),
                    cursor: 0,
                    map_slot: 0,
                    scheduled: true,
                    menu: None,
                    persist: false,
                    origin: None,
                });
                effects.extend(run_script(state, content)?);
                return Ok(effects);
            }
            Regional::None => {}
        }
    }
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
        match state.movement_continuation.take() {
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
                        scheduled: false,
                        menu: None,
                        persist: false,
                        origin: Some(crate::ScriptOrigin {
                            map: position.map,
                            x: position.x,
                            y: position.y,
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
                let skill = travel::secret_search_skill(&state.warriors)?;
                let position = state.exploration.as_mut().unwrap();
                let search = travel::discover_secrets(
                    &position.floor,
                    (position.x as i16, position.y as i16),
                    skill,
                    &mut state.random,
                )?;
                if search.center_revealed {
                    return Err(unsupported(
                        "exploration.move",
                        "revealed-center NewLand continuation",
                    ));
                }
                position.floor = search.floor;
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
        Regional::Script(_) => Err(unsupported(
            "exploration.encounter",
            "regional event scripts",
        )),
    }
}

enum Regional {
    None,
    Battle(i16, i16),
    Script(i16),
}

fn regional_selection(
    regions: &mut [u8],
    position: (i16, i16),
    random: &mut RandomState,
) -> Result<Regional, RuleError> {
    let (px, py) = position;
    for region in (0..20).rev() {
        let offset = region * 8;
        if rules::inclusive_range(px, word(regions, offset + 2)?, word(regions, offset + 6)?)
            && rules::inclusive_range(py, word(regions, offset)?, word(regions, offset + 4)?)
        {
            let result = random.bound(10000);
            if result <= word(regions, 160 + region * 2)? {
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
                    if regions[544 + region] != 0 {
                        return Err(unsupported("exploration.encounter", "avoidance choice"));
                    }
                    random.bound(100);
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
    let loot = state
        .exploration
        .as_ref()
        .is_some_and(|position| matches!(position.presentation, Some(Presentation::Loot)));
    if loot && needs_spell_confirmation(state) && !keep_spells {
        return Err(RuleError::Invalid(
            "confirm keeping the current psi selections to complete promotion".into(),
        ));
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
            let [dx, dy] = position.last_move;
            let forward =
                OpeningMission::index(position.x + i32::from(dx), position.y + i32::from(dy))?;
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
            position.presentation = None;
            position.script = None;
            return Ok(vec![Event::Moved {
                x: undone.x,
                y: undone.y,
            }]);
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
        typed_encounter::QuestionPlan::EndScript => position.script = None,
        _ => {
            return Err(unsupported(
                "story.choose",
                "typed branch or move-away caller",
            ));
        }
    }
    position.presentation = None;
    run_script(state, content)
}

pub fn cancel_menu(state: &mut Session, _content: &Content) -> Result<Vec<Event>, RuleError> {
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
    position.presentation = None;
    position.script = None;
    Ok(vec![])
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
    script: &mut ScriptState,
    id: i16,
) -> Result<Presentation, RuleError> {
    if id < 0 {
        return Err(RuleError::Invalid("negative typed encounter".into()));
    }
    let record = span(
        &map.typed_encounters,
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
    let map = active_map(state, content)?;
    let position = state
        .exploration
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("no active mission".into()))?;
    let Some(Presentation::Typed { id, .. }) = position.presentation.as_ref() else {
        return Err(RuleError::Invalid("no typed encounter active".into()));
    };
    let record = span(
        &map.typed_encounters,
        *id as usize * typed_encounter::RECORD_BYTES,
        typed_encounter::RECORD_BYTES,
    )?;
    let typed_encounter::PasswordSelection::Selected(selected) =
        typed_encounter::select_password(record, text.as_bytes())?
    else {
        return Ok(vec![]);
    };
    let script = position
        .script
        .as_mut()
        .ok_or_else(|| RuleError::Invalid("typed script missing".into()))?;
    let remaining = script
        .menu
        .as_ref()
        .ok_or_else(|| RuleError::Invalid("typed continuation missing".into()))?
        .remaining;
    match typed_encounter::continuation(record, selected, remaining)? {
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
        let collected = if item.id() == 700 {
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
            state.load_item_work(content, id)?;
        }
    } else {
        effects = insert(item, &mut state.crate_items, true)?;
        if effects.contains(&Event::StorageFull) {
            return Ok(effects);
        }
    }
    state.booty_items[slot].0[..2].fill(0);
    Ok(effects)
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
    if script.scheduled {
        return Ok(vec![]);
    }
    let origin = script.origin.clone().unwrap_or_else(|| {
        let key = long(&script.record, 0).unwrap_or(0);
        crate::ScriptOrigin {
            map: key / 10000,
            x: key % 100,
            y: key / 100 % 100,
        }
    });
    if exhausted && script.menu.is_none() {
        script.record[7] = 255;
        let headers = if origin.map == position.map {
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
        level: 0,
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
    if plan.target.level != 0 {
        return Err(unsupported("story.destination", "other scenario levels"));
    }
    OpeningMission::index(plan.target.x, plan.target.y)?;
    let authored = content
        .map(plan.target.map)
        .ok_or_else(|| RuleError::Invalid("door destination map unavailable".into()))?;
    if script.persist {
        let record =
            crate::story_destination::persistent_record(&script.record, origin.x, origin.y)?;
        let headers = if origin.map == position.map {
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
    if plan.target.map != position.map {
        let records = state
            .visited_maps
            .remove(&plan.target.map)
            .unwrap_or_else(|| MapState::authored(authored));
        state
            .visited_maps
            .insert(position.map, position.map_state());
        position.restore_map(plan.target.map, records);
    }
    position.x = plan.target.x;
    position.y = plan.target.y;
    Ok(vec![Event::Moved {
        x: position.x,
        y: position.y,
    }])
}

pub(crate) fn run_script(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let mut effects = Vec::new();
    for _ in 0..128 {
        let map = active_map(state, content)?;
        let position = state.exploration.as_mut().unwrap();
        let Some(script) = position.script.as_mut() else {
            return Ok(effects);
        };
        if script.cursor >= 8 {
            if let Some(menu) = script.menu.as_mut() {
                let before = menu.remaining;
                menu.remaining = before.wrapping_sub(1);
                if before > 1 {
                    let id = menu.id;
                    if menu.typed {
                        position.presentation = Some(present_typed(map, script, id)?);
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
            113 => crate::story_map_picture::assign(&mut state.map_picture, parameter),
            98 => {}
            15 | 16 => {
                if state.phase != Phase::Exploration {
                    return Err(unsupported("story.health", "combat body removal"));
                }
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
                for event in outcome.events {
                    match event {
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
                        crate::story_health::HealthEvent::Defeated { .. } => {}
                        crate::story_health::HealthEvent::PartyLost => {
                            return Err(unsupported("story.health", "party defeat transition"));
                        }
                    }
                }
                if position.presentation.is_some() {
                    return Ok(effects);
                }
            }
            52 => {
                let extra = span(
                    &map.combat.as_ref().unwrap().encounters,
                    parameter.unsigned_abs() as usize * 10,
                    10,
                )?;
                if matches!(word(extra, 0)?, 4 | 5) {
                    return Err(unsupported(
                        "story.select_party",
                        "casting context for attribute or resistance selection",
                    ));
                }
                state.party_mask = crate::story_party::select_party(
                    &state.warriors,
                    state.selected_warrior,
                    extra,
                    crate::story_party::PartySaveContext::default(),
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
                effects.extend(combat::begin_group(state, content, parameter)?);
                return Ok(effects);
            }
            56 => {
                effects.extend(combat::begin_npc(state, content, parameter)?);
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
                let mut text = map.dialogue(parameter)?;
                for index in 0..state.warriors.len() {
                    let selected = if state.warriors[index].stamina()? < 1 {
                        (0..state.warriors.len())
                            .rev()
                            .find(|i| state.warriors[*i].stamina().unwrap_or(0) > 0)
                            .unwrap_or(index)
                    } else {
                        index
                    };
                    text = text.replace(&format!("^{}", index + 1), &state.warriors[selected].name);
                }
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
            26 => effects.push(Event::StoryPresented),
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
                let consumed = story_inventory::consume_party_items(
                    &state.warriors,
                    content,
                    &tables.race_moves,
                    extra,
                )?;
                if consumed.consumed != 0 {
                    state.loaded_item_field32 =
                        Some(state.ammunition_work.load_item(content, word(extra, 0)?)?);
                }
                state.warriors = consumed.warriors;
                effects.extend(
                    consumed
                        .sounds
                        .into_iter()
                        .map(|resource_id| Event::Sound { resource_id }),
                );
            }
            31 => {
                position.presentation = Some(Presentation::WarriorChoice { extra: parameter });
                effects.push(Event::StoryPresented);
                return Ok(effects);
            }
            38 => {
                let extra = span(
                    &map.combat.as_ref().unwrap().encounters,
                    parameter as usize * 10,
                    10,
                )?;
                if story_inventory::item_prerequisite_matches(&state.warriors, extra)? {
                    if word(extra, 4)? != 0 {
                        return Err(unsupported("story.item_condition", "menu branches"));
                    }
                    script.record[8..].copy_from_slice(&map.script(word(extra, 6)?)?[8..]);
                    script.cursor = 0;
                }
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
                    let authored = content.map(target).ok_or_else(|| {
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
                position.presentation = Some(present_typed(map, script, parameter)?);
                effects.push(Event::Sound { resource_id: 20005 });
                effects.push(Event::StoryPresented);
                return Ok(effects);
            }
            21 | 76 | 77 => {
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
                    _ => story_variables::variable_predicate(&state.flags, extra)?.continuation,
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
                        position.presentation = Some(present_typed(map, script, id)?);
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
            46 => {
                let extra = span(
                    &map.combat.as_ref().unwrap().encounters,
                    usize::from(parameter.unsigned_abs()) * 10,
                    10,
                )?;
                if let Some(jump) = story::flag_condition_jump(&state.flags, extra)? {
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
            10 => {
                if parameter < 0 {
                    return Err(RuleError::Invalid("negative treasure record".into()));
                }
                let treasure = span(&map.treasure, parameter as usize * 88, 88)?;
                state.booty_items.fill(InventoryRecord([0; 22]));
                let prepared = crate::story_treasure::prepare(
                    &state.warriors,
                    treasure,
                    content,
                    &mut state.random,
                    state.difficulty,
                )?;
                state.warriors = prepared.warriors;
                for (slot, item) in prepared.items.into_iter().enumerate() {
                    state.loaded_item_field32 =
                        Some(state.ammunition_work.load_item(content, item.id())?);
                    state.booty_items[slot] = item;
                }
                position.presentation = Some(Presentation::Loot);
                effects.push(Event::StoryPresented);
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
                effects.extend(super::combat::begin(state, content, parameter)?);
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
