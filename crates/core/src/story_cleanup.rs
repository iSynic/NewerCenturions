use crate::{Content, Event, Phase, RuleError, Session, combat, story_health, story_inventory};
use centurions_content::word;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Stage {
    Survivors,
    Complete,
}

/// newland100 -> booty(mode2): early cleanup, without the ordinary reward path.
pub(crate) fn begin(
    state: &mut Session,
    content: &Content,
    parameter: i16,
) -> Result<Vec<Event>, RuleError> {
    if parameter != 1 {
        return Err(RuleError::Unsupported {
            action: "story.combat_cleanup",
            dependency: "other native booty modes",
        });
    }
    if state.phase != Phase::Combat || state.scripted_cleanup.is_some() {
        return Err(RuleError::Invalid(
            "scripted combat cleanup is not available".into(),
        ));
    }
    let mut events = combat::runtime::refit_after_combat(state, content)?;
    let unavailable = state
        .warriors
        .iter()
        .filter(|warrior| {
            warrior.stamina().unwrap_or(0) < 1 || word(&warrior.bytes, 0x90).unwrap_or(0) < 0
        })
        .count() as i8;
    state.warriors = story_inventory::normalize_story_loot(&state.warriors)?;
    for warrior in &mut state.warriors {
        warrior.bytes[0x9df] = 1;
    }
    let mut health = story_health::context(
        &state.warriors,
        state.selected_warrior,
        state.health_context.as_ref(),
    );
    health.unavailable = unavailable;
    state.health_context = Some(health);
    let arena = state.combat.as_mut().ok_or("cleanup combat missing")?;
    arena.unavailable = unavailable;
    for record in &mut arena.enemies.records {
        if word(&record.bytes, 0x5a)? != -1 {
            record.bytes[0x110..0x112].fill(0);
        }
    }
    state
        .booty_items
        .fill(centurions_content::InventoryRecord([0; 22]));
    state.scripted_cleanup = Some(Stage::Survivors);
    if let Some(presentation) = combat::runtime::begin_survivors(state)? {
        events.extend(presentation);
    } else {
        events.extend(confirm(state, content)?);
    }
    Ok(events)
}

pub(crate) fn confirm(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    if state.scripted_cleanup != Some(Stage::Survivors) {
        return Err(RuleError::Invalid(
            "scripted survivor cleanup is not pending".into(),
        ));
    }
    combat::runtime::retain_last_defeated(state);
    let arena = state.combat.as_mut().ok_or("cleanup combat missing")?;
    let mut events = vec![];
    if state.suppress_allies == 0 {
        let result = if let Some(selection) = state.survivors.take() {
            combat::allies::commit_survivors(
                &state.allies,
                state.ally_count,
                &mut arena.enemies.records,
                Some(&selection),
            )?
        } else {
            combat::allies::synchronize(
                &state.allies,
                state.ally_count,
                &mut arena.enemies.records,
                &mut state.random,
            )?
        };
        state.allies = result.menu;
        state.ally_count = result.count;
        events.extend(
            result
                .sounds
                .into_iter()
                .map(|resource_id| Event::Sound { resource_id }),
        );
    }
    for record in state
        .allies
        .as_chunks_mut::<288>()
        .0
        .iter_mut()
        .take(state.ally_count)
    {
        if word(record, 0x26)? < 200 {
            let weapon = [record[0x26], record[0x27]];
            record[0x4e..0x50].copy_from_slice(&weapon);
        }
    }
    state.retain_npcs();
    let arena = state.combat.take().unwrap();
    state.attack_packet = arena.attack_packet;
    state.attack_identity = arena.attack_identity;
    state.weapon_work = arena.weapon_work;
    state.phase = Phase::Exploration;
    state.scripted_cleanup = Some(Stage::Complete);
    state.handled_deaths.clear();
    let position = state.exploration.as_mut().ok_or("cleanup map missing")?;
    position
        .map_state()
        .validate(content.map(position.map).ok_or("cleanup content missing")?)?;
    position.presentation = None;
    // Current mutable records are the reconstruction's read-only replacement for loadland.
    events.push(Event::Moved {
        x: position.x,
        y: position.y,
    });
    Ok(events)
}

pub(crate) fn validate(state: &Session) -> Result<(), RuleError> {
    match state.scripted_cleanup {
        Some(Stage::Survivors) if state.phase != Phase::Combat || state.survivors.is_none() => Err(
            RuleError::Invalid("cleanup survivor continuation missing".into()),
        ),
        Some(Stage::Complete)
            if state.phase != Phase::Exploration
                || state.combat.is_some()
                || state
                    .exploration
                    .as_ref()
                    .is_none_or(|p| p.script.is_none()) =>
        {
            Err(RuleError::Invalid(
                "cleanup script continuation missing".into(),
            ))
        }
        _ => Ok(()),
    }
}
