use crate::{Content, Event, Phase, Presentation, RuleError, Session, story_health};

/// The clinic's prices at the standard rate, before the percentage.
const BASE_PRICES: [i16; 9] = [250, 350, 850, 200, 0, 200, 350, 1500, 0];

/// Clinic prices at a percentage, computed in single precision and truncated.
///
/// Original: temple
pub fn prices(percent: i16) -> [i16; 9] {
    BASE_PRICES.map(|base| ((f32::from(base) * f32::from(percent) / 100.) as i32) as i16)
}

/// Checks a saved clinic visit. A clinic entry must follow non-empty messages and a running
/// script. A pending member must be in the squad during exploration, with a script running or a
/// clinic rate recorded.
pub(crate) fn validate(state: &Session) -> Result<(), RuleError> {
    let mut page = state
        .exploration
        .as_ref()
        .and_then(|p| p.presentation.as_ref());
    let mut wrapped = false;
    let mut valid_pages = true;
    while let Some(Presentation::Messages { pages, next }) = page {
        valid_pages &= !pages.is_empty();
        wrapped = true;
        page = next.as_deref();
    }
    if matches!(page, Some(Presentation::ClinicEntry { .. }))
        && (!wrapped
            || !valid_pages
            || state
                .exploration
                .as_ref()
                .is_none_or(|p| p.script.is_none()))
    {
        return Err("invalid clinic entry continuation".into());
    }
    if let Some(member) = pending(state)
        && (state.phase != Phase::Exploration
            || member >= state.warriors.len()
            || state.exploration.as_ref().is_none_or(|p| {
                p.script.is_none()
                    && state
                        .field_controls
                        .as_ref()
                        .and_then(|c| c.clinic)
                        .is_none()
            }))
    {
        return Err("invalid clinic continuation".into());
    }
    Ok(())
}

/// The squad member the clinic is for, looking past any messages shown first. A clinic entry
/// not yet shown uses the selected character.
pub(crate) fn pending(state: &Session) -> Option<usize> {
    let mut presentation = state
        .exploration
        .as_ref()
        .and_then(|position| position.presentation.as_ref())?;
    while let Presentation::Messages { next, .. } = presentation {
        presentation = next.as_deref()?;
    }
    match presentation {
        Presentation::Clinic { member, .. } => Some(*member),
        Presentation::ClinicEntry { .. } => Some(state.char_select_new),
        _ => None,
    }
}

/// Entering a clinic pools the party's credits, even if nothing is bought.
///
/// Original: newland, temple
pub(crate) fn visit(state: &mut Session, percent: i16) -> Result<Vec<Event>, RuleError> {
    state.field_controls.get_or_insert_default().clinic = Some(percent);
    crate::party::pool_credits(&mut state.warriors, &mut state.party_currency)?;
    state
        .exploration
        .as_mut()
        .ok_or("clinic requires a mission")?
        .presentation = Some(Presentation::Clinic {
        member: state.char_select_new,
        percent,
    });
    Ok(vec![
        Event::Sound { resource_id: 430 },
        Event::StoryPresented,
    ])
}

/// Handles a click on a clinic control. Items 1 and 11 leave and continue the script; 6 and 7 step
/// to the previous or next member; 15 to 18 and 20 to 22 are services. A service is refused with
/// a warning when the party's money is below its price; otherwise the price is paid and the
/// treatment applied (items 15 to 17 heal the member, 18, 20 and 21 clear a record field, and 22
/// only charges).
///
/// Original: temple
pub(crate) fn control(
    state: &mut Session,
    content: &Content,
    item: u16,
) -> Result<Vec<Event>, RuleError> {
    let Some(Presentation::Clinic {
        mut member,
        percent,
    }) = state
        .exploration
        .as_ref()
        .and_then(|position| position.presentation.clone())
    else {
        return Err(RuleError::Refused { feedback: vec![] });
    };
    match item {
        1 | 11 => {
            state.exploration.as_mut().unwrap().presentation = None;
            let mut events = vec![Event::Sound { resource_id: -141 }];
            events.extend(crate::exploration::run_script(state, content)?);
            return Ok(events);
        }
        6 | 7 => {
            member = if item == 6 {
                (member + state.warriors.len() - 1) % state.warriors.len()
            } else {
                (member + 1) % state.warriors.len()
            };
            state.char_select_new = member;
            state.exploration.as_mut().unwrap().presentation =
                Some(Presentation::Clinic { member, percent });
            return Ok(vec![Event::Sound { resource_id: 141 }]);
        }
        15..=18 | 20..=22 => {}
        _ => return Err(RuleError::Refused { feedback: vec![] }),
    }
    let price = i32::from(prices(percent)[usize::from(item - 15)]);
    if state.party_currency < price {
        return Err(RuleError::Refused {
            feedback: vec![crate::feedback::GameFeedback::Warning { index: 50 }],
        });
    }
    state.party_currency = state.party_currency.wrapping_sub(price);
    let mut events = vec![];
    match item {
        15..=17 => {
            let amount = match item {
                15 => state.random.bound(8),
                16 => state.random.range(3, 24),
                _ => 5000,
            };
            let context = story_health::reselected_context(member, &state.globals.health_context);
            let outcome = story_health::heal_outdoors(
                &state.warriors,
                member,
                amount,
                false,
                &context,
                &mut state.random,
            )?;
            state.warriors = outcome.warriors;
            state.globals.health_context = outcome.context;
            for event in outcome.events {
                match event {
                    story_health::HealthEvent::Sound(resource_id) => {
                        events.push(Event::Sound { resource_id })
                    }
                    story_health::HealthEvent::Defeated { .. } => {}
                    story_health::HealthEvent::PartyLost => {
                        events.extend(crate::mission::defeat(state, content)?)
                    }
                    _ => unreachable!("unanimated single-member heal has no presentation events"),
                }
            }
        }
        18 | 20 | 21 => {
            let offset = match item {
                18 => 0x96,
                20 => 0x64,
                _ => 0x94,
            };
            state.warriors[member].bytes[offset..offset + 2].fill(0);
        }
        // Item 22 is shown and charged but does nothing; the game has no case for it.
        22 => {}
        _ => unreachable!(),
    }
    events.push(Event::Sound { resource_id: -431 });
    Ok(events)
}
