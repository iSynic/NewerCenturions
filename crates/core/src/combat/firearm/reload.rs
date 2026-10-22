use crate::combat::Combat;
use crate::{Event, Phase, RuleError, Session};
use centurions_content::Content;

pub fn reload_command(
    state: &mut Session,
    content: &Content,
    index: usize,
) -> Result<Vec<Event>, RuleError> {
    if state.phase == Phase::SquadSelection
        || state
            .exploration
            .as_ref()
            .is_some_and(|p| p.presentation.is_some())
    {
        return Err(RuleError::Invalid(
            "reload requires an unobstructed mission".into(),
        ));
    }
    if state.phase == Phase::Combat
        && state.combat.as_ref().and_then(Combat::active_body) != Some(index as i16)
    {
        return Err(RuleError::Invalid(
            "reload requires the active warrior".into(),
        ));
    }
    if state.phase == Phase::Exploration {
        let warrior = state
            .warriors
            .get(index)
            .ok_or_else(|| RuleError::Invalid("warrior outside squad".into()))?;
        let result = crate::ammunition::refit_with_record(
            warrior,
            &state.ammunition,
            content,
            crate::ammunition::RefitFlags {
                in_combat: false,
                force: false,
                silent: false,
                warnings: true,
            },
            &state.native_work.ammunition_work,
            &mut state.random,
            state.native_work.inventory_record.as_ref(),
        )?;
        return crate::ammunition::commit_result(state, index, result);
    }
    let warrior = state.warriors.get(index).ok_or("warrior outside squad")?;
    let id = if warrior.record().firing_mode() {
        warrior.record().equipped_weapon()?
    } else {
        warrior.record().melee_weapon()?
    };
    let mut work = state.native_work.ammunition_work.clone();
    work.load_weapon(&content.items, id)?;
    if work.item.is_none() {
        work.item = Some(crate::ammunition::definition(content, 0)?);
    }
    let result = crate::ammunition::refit_with_record(
        warrior,
        &state.ammunition,
        content,
        crate::ammunition::RefitFlags {
            in_combat: true,
            force: true,
            silent: false,
            warnings: true,
        },
        &work,
        &mut state.random,
        state.native_work.inventory_record.as_ref(),
    )?;
    let complete = result.completed_refit();
    let mut events = crate::ammunition::commit_result(state, index, result)?;
    if !complete {
        state.warriors[index].record_mut().set_automatic_mode(0);
        if !events.iter().any(|event| {
            matches!(
                event,
                Event::CollectionRejected {
                    warning: crate::feedback::OUT_OF_AMMUNITION_WARNING,
                }
            )
        }) {
            events.push(Event::CollectionRejected {
                warning: crate::feedback::OUT_OF_AMMUNITION_WARNING,
            });
        }
    }
    Ok(events)
}
