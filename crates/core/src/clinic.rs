use crate::{Content, Event, Phase, Presentation, RuleError, Session, story_health};

const BASE_PRICES: [i16; 9] = [250, 350, 850, 200, 0, 200, 350, 1500, 0];

/// temple15d328..15d412 converts each WORD through single precision and FTINTX.
pub fn prices(percent: i16) -> [i16; 9] {
    BASE_PRICES.map(|base| ((f32::from(base) * f32::from(percent) / 100.) as i32) as i16)
}

pub(crate) fn validate(state: &Session) -> Result<(), RuleError> {
    if let Some(member) = pending(state)
        && (state.phase != Phase::Exploration
            || member >= state.warriors.len()
            || state
                .exploration
                .as_ref()
                .is_none_or(|p| p.script.is_none()))
    {
        return Err("invalid clinic continuation".into());
    }
    Ok(())
}

pub(crate) fn pending(state: &Session) -> Option<usize> {
    let mut presentation = state
        .exploration
        .as_ref()
        .and_then(|position| position.presentation.as_ref())?;
    while let Presentation::Messages { next, .. } = presentation {
        presentation = next.as_deref()?;
    }
    if let Presentation::Clinic { member, .. } = presentation {
        Some(*member)
    } else {
        None
    }
}

/// newland144bd8 and temple15d2f0: pool on entry, even if no treatment is bought.
pub(crate) fn visit(state: &mut Session, percent: i16) -> Result<Vec<Event>, RuleError> {
    crate::party::pool_credits(&mut state.warriors, &mut state.party_currency)?;
    state
        .exploration
        .as_mut()
        .ok_or("clinic requires a mission")?
        .presentation = Some(Presentation::Clinic {
        member: state.selected_warrior,
        percent,
    });
    Ok(vec![
        Event::Sound { resource_id: 430 },
        Event::StoryPresented,
    ])
}

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
            state.selected_warrior = member;
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
            feedback: vec![crate::feedback::NativeFeedback::Warning { index: 50 }],
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
            let context =
                story_health::context(&state.warriors, member, state.health_context.as_ref());
            let outcome = story_health::heal_outdoors(
                &state.warriors,
                member,
                amount,
                false,
                &context,
                &mut state.random,
            )?;
            state.warriors = outcome.warriors;
            state.health_context = Some(outcome.context);
            for event in outcome.events {
                match event {
                    story_health::HealthEvent::Sound(resource_id) => {
                        events.push(Event::Sound { resource_id })
                    }
                    story_health::HealthEvent::Defeated { .. } => {}
                    story_health::HealthEvent::PartyLost => {
                        events.extend(crate::mission::defeat(state, content))
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
        // The original visible item22 charges its price but has no switch body.
        22 => {}
        _ => unreachable!(),
    }
    events.push(Event::Sound { resource_id: -431 });
    Ok(events)
}
