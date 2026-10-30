//! Land regions and the random battles they roll.
#[cfg(all(test, feature = "original-fixtures"))]
use super::unsupported;
use super::{active_map, resume_movement, run_script};
use crate::{Event, Presentation, RandomState, RuleError, ScriptState, Session, rules};
use centurions_content::Content;
use centurions_content::records::{RegionTable, RegionTableMut};

/// Rolls the random battle of the region the party stands in, if any, returning the battle and the
/// advantage it starts with. A region that would run an event script or an avoidance question is
/// not supported here.
#[cfg(all(test, feature = "original-fixtures"))]
pub fn regional_encounter(
    regions: &[u8],
    position: (i16, i16),
    random: &mut RandomState,
) -> Result<Option<(i16, i16)>, RuleError> {
    match regional_selection(&mut regions.to_vec(), position, true, random)? {
        Regional::None => Ok(None),
        Regional::Battle(battle, advantage) => Ok(Some((battle, advantage))),
        Regional::Script(_) | Regional::Avoidance(_) => Err(unsupported(
            "exploration.encounter",
            "regional event scripts",
        )),
    }
}

/// What a region's roll gives the party: nothing, a battle with its advantage, an event script, or
/// an avoidance question for a region.
pub(super) enum Regional {
    None,
    Battle(i16, i16),
    Script(i16),
    Avoidance(usize),
}

/// Rolls the regions under the party from the last toward the first.
pub(super) fn regional_selection(
    regions: &mut [u8],
    position: (i16, i16),
    battles: bool,
    random: &mut RandomState,
) -> Result<Regional, RuleError> {
    regional_selection_from(regions, position, 20, battles, random)
}

/// Rolls the regions before the given one, so a declined avoidance carries on with the regions
/// beneath it.
pub(super) fn regional_selection_from(
    regions: &mut [u8],
    position: (i16, i16),
    before: usize,
    battles: bool,
    random: &mut RandomState,
) -> Result<Regional, RuleError> {
    regional_selection_chance(regions, position, before, 1, battles, random)
}

/// A region rolls a random battle only while the party has no mind shield
/// (party protection 7) and random battles are on; its event scripts still roll.
///
/// Original: timeclick
pub(super) fn battles_allowed(state: &Session) -> Result<bool, RuleError> {
    Ok(state.globals.random_battles != 0
        && !crate::combat::party_buffs::mind_shield(&state.party_condition)?)
}

/// Checks the regions as time passes and starts what the roll gives: a battle, a script or an
/// avoidance question.
pub(crate) fn check_region(
    state: &mut Session,
    content: &Content,
    divisor: i16,
) -> Result<Vec<Event>, RuleError> {
    let battles = battles_allowed(state)?;
    let position = state
        .exploration
        .as_mut()
        .ok_or("regional position unavailable")?;
    let region = regional_selection_chance(
        &mut position.region_state,
        (position.x as i16, position.y as i16),
        20,
        divisor,
        battles,
        &mut state.random,
    )?;
    apply_region(state, content, region)
}

/// Rolls the regions that cover the party's square, last region first. A roll from 1 to 10000 not
/// above the region's chance divided by the divisor tries each of its events by the event's
/// percent; if none is picked and random battles are allowed, the region gives its avoidance
/// question or a battle. An exclusive region stops the search unless that roll picked something.
pub(super) fn regional_selection_chance(
    regions: &mut [u8],
    position: (i16, i16),
    before: usize,
    divisor: i16,
    battles: bool,
    random: &mut RandomState,
) -> Result<Regional, RuleError> {
    let (px, py) = position;
    for region in (0..before.min(20)).rev() {
        let table = RegionTable(regions);
        if rules::inclusive_range(px, table.first_x(region)?, table.last_x(region)?)
            && rules::inclusive_range(py, table.first_y(region)?, table.last_y(region)?)
        {
            let result = random.bound(10000);
            if result <= table.encounter_chance(region)? / divisor {
                for event in 0..3 {
                    if random.bound(100)
                        <= RegionTable(regions)
                            .event_chance(region, event)?
                            .wrapping_abs()
                    {
                        if RegionTable(regions).event_chance(region, event)? > 0 {
                            RegionTableMut(regions).clear_event_chance(region, event);
                        }
                        return Ok(Regional::Script(
                            RegionTable(regions).event_script(region, event)?,
                        ));
                    }
                }
                let table = RegionTable(regions);
                let minimum = table.battle_minimum(region)?;
                if battles && minimum != 0 {
                    if random.bound(100) < i16::from(table.avoidance_chance(region)) {
                        return Ok(Regional::Avoidance(region));
                    }
                    let advantage = if random.bound(100) < 10 { -1 } else { 0 };
                    let battle = random.range(minimum, table.battle_maximum(region)?);
                    return Ok(Regional::Battle(battle, advantage));
                }
            }
            if RegionTable(regions).exclusive(region) {
                break;
            }
        }
    }
    Ok(Regional::None)
}

/// Starts what a region roll gave: the battle, the region's event script, or the avoidance question
/// with its dialogue and sound.
pub(super) fn apply_region(
    state: &mut Session,
    content: &Content,
    region: Regional,
) -> Result<Vec<Event>, RuleError> {
    match region {
        Regional::None => Ok(vec![]),
        Regional::Battle(battle, advantage) => {
            crate::combat::begin_battle(state, content, battle, advantage)
        }
        Regional::Script(id) => {
            let map = active_map(state, content)?;
            state.exploration.as_mut().unwrap().script =
                Some(ScriptState::scheduled(map.script(id)?.to_vec(), 0));
            run_script(state, content)
        }
        Regional::Avoidance(region) => {
            let map = active_map(state, content)?;
            let position = state.exploration.as_mut().unwrap();
            let table = RegionTable(&position.region_state);
            let id = table.avoidance_dialogue(region)?.wrapping_abs();
            let sound = table.avoidance_sound(region)?;
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

/// Takes the player's answer to an avoidance question. Fighting starts a battle from the region's
/// battle range with the party in advantage; declining carries on with the regions beneath, unless
/// the region is exclusive.
pub(super) fn choose_avoidance(
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
        let table = RegionTable(&position.region_state);
        let battle = state
            .random
            .range(table.battle_minimum(region)?, table.battle_maximum(region)?);
        Regional::Battle(battle, 1)
    } else if RegionTable(&position.region_state).exclusive(region) {
        Regional::None
    } else {
        regional_selection_from(
            &mut position.region_state,
            (position.x as i16, position.y as i16),
            region,
            state.globals.random_battles != 0,
            &mut state.random,
        )?
    };
    let mut events = apply_region(state, content, next)?;
    events.extend(resume_movement(state, content)?);
    Ok(events)
}
