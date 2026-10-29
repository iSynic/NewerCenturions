//! Combatchoice119258..119588 resolves faction groups without entering cast.
use crate::combat::{player_psi, runtime, spells};
use crate::{Content, Event, RuleError, Session};
use centurions_content::records::AttackPacket;

pub(super) fn resolve(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let pending = state
        .psi_selection
        .take()
        .ok_or("prepared item selection missing")?;
    let combat = state
        .combat
        .as_ref()
        .ok_or("prepared item combat missing")?;
    let packet = combat.attack_packet.clone();
    let mode = AttackPacket(&packet).targeting_mode();
    let mut mask = [0; 110];
    for (body, warrior) in state.warriors.iter().enumerate() {
        if warrior.record().active_flag() != 0
            && (mode == 12 || (warrior.record().faction()? == 0) == (mode == 9))
        {
            mask[body] = 1;
        }
    }
    for (index, enemy) in combat.enemies.records.iter().enumerate() {
        if enemy.record().stamina()? > 0
            && (mode == 12 || (enemy.record().faction()? == 0) == (mode == 9))
        {
            mask[index + 10] = 1;
        }
    }
    state.publish_spell_targets(&mask);
    state.set_ready(1);
    let actor = &mut state.warriors[pending.actor];
    let movement = actor.record().movement_points()?.wrapping_sub(3).max(0);
    let actions = actor.record().action_points()?.wrapping_sub(2);
    actor.record_mut().set_movement_points(movement);
    actor.record_mut().set_action_points(actions);
    let alias = spells::retained_item_alias(&state.native_work.ammunition_work)?;
    let map = state.land(content).ok_or("prepared item map missing")?;
    let combat = state.combat.as_mut().unwrap();
    let previous = combat.item_condition_caller;
    combat.item_condition_caller = combat.active_body();
    let result = player_psi::resolve_mask(
        combat,
        &mut state.warriors,
        map,
        &mut state.random,
        pending.actor,
        player_psi::CastRequest {
            spell: pending.spell,
            power: pending.power,
            target: None,
            rotation: 0,
        },
        &packet,
        &mask,
        (
            alias,
            crate::combat::warrior_alias::Owners::new(
                None,
                &state.native_work.shop_work,
                state
                    .exploration
                    .as_ref()
                    .map(|position| position.event_headers.as_slice()),
            )
            .with_backup(&state.native_work.warrior_table),
            Some((&content.items, &mut state.native_work.ammunition_work)),
        ),
    );
    state.combat.as_mut().unwrap().item_condition_caller = previous;
    let mut events = result?;
    state.commit_alias_writes()?;
    state.clear_targets();
    state.set_ready(0);
    events.extend(runtime::finish_player_action(
        state,
        content,
        pending.actor as i16,
    )?);
    Ok(events)
}
