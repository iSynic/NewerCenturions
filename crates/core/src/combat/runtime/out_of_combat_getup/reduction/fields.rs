//! Spell fields acting after the battle has ended, and their usage marks.
use crate::combat::runtime::out_of_combat_spell;
use crate::combat::runtime::out_of_combat_spell::Invocation;
use crate::{
    Content, RuleError, Session,
    combat::{
        runtime::{TurnFlow, TurnOutcome},
        spells,
    },
};
use centurions_content::records::{SpellField, SpellInfoRecordMut};
mod targeting;

/// Clears which spell fields have already acted this round.
pub(super) fn clear_usage(state: &mut Session) -> Result<(), RuleError> {
    let battlefield = &mut state.globals.battlefield;
    battlefield.collide_check = spells::reset_used(&battlefield.collide_check)?;
    Ok(())
}

/// Resolves the spell fields that reach a body after the battle has ended, one slot at a time, each
/// at most once a round.
///
/// Original: collide
pub(in crate::combat::runtime) fn collide(
    state: &mut Session,
    content: &Content,
    body: i16,
) -> Result<TurnOutcome, RuleError> {
    if body < 10 && state.warriors[body as usize].record().in_battle() == 0 {
        return Ok(TurnOutcome::default());
    }
    let mut events = vec![];
    for slot in 0..60 {
        let battlefield = &mut state.globals.battlefield;
        battlefield.collide_check[62] = (slot + 1) as u8;
        let record: [u8; 20] = battlefield.spell_queue[slot * 20..slot * 20 + 20]
            .try_into()
            .unwrap();
        let field = SpellField(&record);
        if field.duration() == 0
            || battlefield.collide_check[slot] != 0
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
        SpellInfoRecordMut(&mut packet).prepare_field_collision();
        state.globals.spell_info = packet.clone();
        state.globals.cast_identity = [id / 1000 - 1, field.level(), id % 100 - 1];
        state.globals.attack_count = field.power();
        // The field's centre becomes the first target.
        state.clear_targets();
        state.globals.target_picks.cell = field.center();
        let mask = targeting::mask(state, body, &field, &packet, &mission.areas)?;
        state.publish_spell_targets(&mask);
        state.set_in_spell(1);
        let mut usage = spells::FieldUsage {
            slot,
            used: state.globals.battlefield.collide_check.clone(),
        };
        if out_of_combat_spell::apply(
            state,
            content,
            &mask,
            &mut usage,
            Invocation {
                id,
                level: field.level(),
                count: field.power(),
                collision: true,
                // The first target is the field centre; the cast index keeps its last
                // value.
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
        state.set_in_spell(0);
        state.globals.target_picks.resolved(0);
    }
    state.globals.battlefield.collide_check[62] = 0;
    Ok(TurnOutcome {
        events,
        flow: TurnFlow::Returned,
    })
}
