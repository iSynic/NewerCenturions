//! Regions for maintained exploration.
#[cfg(all(test, feature = "original-fixtures"))]
use super::unsupported;
use super::{active_map, resume_movement, run_script};
use crate::{Event, Presentation, RandomState, RuleError, ScriptState, Session, rules};
use centurions_content::{Content, word};

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

pub(super) enum Regional {
    None,
    Battle(i16, i16),
    Script(i16),
    Avoidance(usize),
}

pub(super) fn regional_selection(
    regions: &mut [u8],
    position: (i16, i16),
    random: &mut RandomState,
) -> Result<Regional, RuleError> {
    regional_selection_from(regions, position, 20, random)
}

pub(super) fn regional_selection_from(
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

pub(super) fn regional_selection_chance(
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
