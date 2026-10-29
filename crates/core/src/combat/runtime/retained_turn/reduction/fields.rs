//! Retained collide scan, shared packet work and the literal usage-byte writers.
use crate::{
    Content, RuleError, Session,
    combat::{
        runtime::{TurnFlow, TurnOutcome},
        spells,
    },
};
use centurions_content::records::{AttackPacketMut, SpellField};
mod targeting;

pub(super) fn clear_usage(state: &mut Session) -> Result<(), RuleError> {
    let work = &mut state.native_work.tactical_work;
    work.field_used = spells::reset_used(&work.field_used)?;
    Ok(())
}

pub(in crate::combat::runtime) fn collide(
    state: &mut Session,
    content: &Content,
    body: i16,
) -> Result<TurnOutcome, RuleError> {
    if body < 10 && state.warriors[body as usize].record().active_flag() == 0 {
        return Ok(TurnOutcome::default());
    }
    let mut events = vec![];
    for slot in 0..60 {
        let work = &mut state.native_work.tactical_work;
        work.field_used[62] = (slot + 1) as u8;
        let record: [u8; 20] = work.spell_fields[slot * 20..slot * 20 + 20]
            .try_into()
            .unwrap();
        let field = SpellField(&record);
        if field.duration() == 0
            || work.field_used[slot] != 0
            || !targeting::intersects(state, body, &field)?
        {
            continue;
        }
        let mission = state.land(content).ok_or("retained field map missing")?;
        let tables = mission
            .combat
            .as_ref()
            .ok_or("retained field tables missing")?;
        let id = field.spell()?;
        let mut packet = spells::load_packet(&tables.attacks, id)?;
        AttackPacketMut(&mut packet).prepare_field_collision();
        state.native_work.attack_packet = packet.clone();
        state.native_work.attack_identity = [id / 1000 - 1, field.level(), id % 100 - 1];
        state.native_work.attack_count = field.power();
        // collide 1142a6 calls cleartarget; 1142e0..1142fe store the centre as target 0.
        state.clear_targets();
        state.native_work.target_work.cell = field.center();
        let mask = targeting::mask(state, body, &field, &packet, &mission.areas)?;
        state.publish_spell_targets(&mask);
        state.set_ready(1);
        let mut usage = spells::FieldUsage {
            slot,
            used: state.native_work.tactical_work.field_used.clone(),
        };
        if crate::combat::runtime::retained_spell::apply(
            state,
            content,
            &mask,
            &mut usage,
            crate::combat::runtime::retained_spell::Invocation {
                id,
                level: field.level(),
                count: field.power(),
                collision: true,
                // collide sets WORD443722/443724 from the field centre; WORD442bd4
                // keeps cast's last value.
                current: None,
            },
            &mut events,
        )? == TurnFlow::Terminated
        {
            return Ok(TurnOutcome {
                events,
                flow: TurnFlow::Terminated,
            });
        }
        state.set_ready(0);
        // spelltargets' triple argument is 0 (collide 114326).
        state.native_work.target_work.resolved(0);
    }
    state.native_work.tactical_work.field_used[62] = 0;
    Ok(TurnOutcome {
        events,
        flow: TurnFlow::Returned,
    })
}
