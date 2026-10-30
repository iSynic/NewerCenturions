//! Items that affect a whole side at once, without a cast.
use crate::combat::{player_psi, runtime, spells};
use crate::{Content, Event, RuleError, Session};
use centurions_content::records::SpellInfoRecord;

/// Marks every standing body on the side the item names (everyone for mode 12, the user's own side
/// for mode 9, the other side otherwise) as a target, spends the user's attack action and casts the
/// item at them.
pub(super) fn resolve(state: &mut Session, content: &Content) -> Result<Vec<Event>, RuleError> {
    let pending = state
        .psi_selection
        .take()
        .ok_or("prepared item selection missing")?;
    let combat = state
        .combat
        .as_ref()
        .ok_or("prepared item combat missing")?;
    let packet = combat.spell_info.clone();
    let mode = SpellInfoRecord(&packet).target_type();
    let mut mask = [0; 110];
    for (body, warrior) in state.warriors.iter().enumerate() {
        if warrior.record().in_battle() != 0
            && (mode == 12 || (warrior.record().faction()? == 0) == (mode == 9))
        {
            mask[body] = 1;
        }
    }
    for (index, monster) in combat.roster.records.iter().enumerate() {
        if monster.record().stamina()? > 0
            && (mode == 12 || (monster.record().faction()? == 0) == (mode == 9))
        {
            mask[index + 10] = 1;
        }
    }
    state.publish_spell_targets(&mask);
    state.set_in_spell(1);
    let actor = &mut state.warriors[pending.actor];
    crate::combat::records::spend_attack_action(actor)?;
    let alias = spells::loaded_item_restriction(&state.globals.loaded_items)?;
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
            crate::combat::warrior_alias::backup_owners!(state),
            Some((&content.items, &mut state.globals.loaded_items)),
        ),
    );
    state.combat.as_mut().unwrap().item_condition_caller = previous;
    let mut events = result?;
    state.commit_alias_writes()?;
    state.clear_targets();
    state.set_in_spell(0);
    events.extend(runtime::finish_player_action(
        state,
        content,
        pending.actor as i16,
    )?);
    Ok(events)
}
