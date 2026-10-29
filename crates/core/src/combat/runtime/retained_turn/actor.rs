//! NPC getup suffix over retained records and the current shared tactical map.
use crate::{Content, Event, RuleError, Session};

pub(super) fn initialize(
    state: &mut Session,
    content: &Content,
    body: i16,
) -> Result<(Vec<Event>, bool), RuleError> {
    let index = state.native_work.selected_enemy;
    let record = &mut state.native_work.npc_backing.slot_mut(index)?.record;
    let (skipped, delta) =
        crate::combat::initialize_enemy_record(&mut record.bytes, &mut state.random)?;
    state.native_work.victory_work.add_total(delta);
    if skipped {
        return Ok((vec![], true));
    }
    super::super::death_return::refresh_actor(state, content)?;
    let record = &mut state.native_work.npc_backing.slot_mut(index)?.record;
    let Some((code, removed)) = super::super::turns::enemy_morale(record)? else {
        return Ok((vec![], false));
    };
    let mut events = vec![Event::Feedback(
        crate::feedback::NativeFeedback::CombatResult {
            body,
            code,
            amount: 0,
            audible: false,
        },
    )];
    if removed {
        let hostile = record.record().faction()? != 0;
        record.record_mut().set_stamina(0);
        let slot = state.native_work.npc_backing.slot(index)?;
        let point = slot.position()?;
        let work = &mut state.native_work.tactical_work;
        let map = &work.map;
        let position = (
            point.0.wrapping_sub(i32::from(map.shift.0) + 5) as i16,
            point.1.wrapping_sub(i32::from(map.shift.1) + 5) as i16,
        );
        work.map = crate::combat::footprint::restore(map, &slot.record, position)?;
        if hostile {
            state.native_work.victory_work.add_defeated(1);
        }
        events.push(Event::CombatantRemoved {
            body: index as i16 + 10,
        });
    }
    Ok((events, removed))
}
