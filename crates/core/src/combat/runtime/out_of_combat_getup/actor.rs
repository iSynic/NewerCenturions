//! A monster's turn setup in getup after the battle has ended.
use crate::{Content, Event, RuleError, Session};

/// Sets up a monster's turn after the battle has ended. Its morale is tried: a monster whose
/// surrender chance is above 100 leaves the field, and a hostile one that leaves counts as
/// defeated; one whose run chance is above 100 is set retreating and stays.
///
/// Original: getup
pub(super) fn initialize(
    state: &mut Session,
    content: &Content,
    body: i16,
) -> Result<(Vec<Event>, bool), RuleError> {
    let index = state.globals.monster_up;
    let record = &mut state.globals.monsters.slot_mut(index)?.record;
    let (skipped, delta) =
        crate::combat::initialize_enemy_record(&mut record.bytes, &mut state.random)?;
    state.globals.kill_counts.add_num_enemy(delta);
    if skipped {
        return Ok((vec![], true));
    }
    super::super::death_return::refresh_actor(state, content)?;
    let record = &mut state.globals.monsters.slot_mut(index)?.record;
    let Some((code, removed)) = super::super::turns::enemy_morale(record)? else {
        return Ok((vec![], false));
    };
    let mut events = vec![Event::Feedback(
        crate::feedback::GameFeedback::CombatResult {
            body,
            code,
            amount: 0,
            audible: false,
        },
    )];
    if removed {
        let hostile = record.record().faction()? != 0;
        record.record_mut().set_stamina(0);
        let slot = state.globals.monsters.slot(index)?;
        let point = slot.position()?;
        let battlefield = &mut state.globals.battlefield;
        let map = &battlefield.field;
        let position = crate::combat::records::grid_position(point, map.map_shift);
        battlefield.field = crate::combat::footprint::restore(map, &slot.record, position)?;
        if hostile {
            state.globals.kill_counts.add_kill_mon(1);
        }
        events.push(Event::CombatantRemoved {
            body: index as i16 + 10,
        });
    }
    Ok((events, removed))
}
