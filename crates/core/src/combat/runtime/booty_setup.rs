//! What booty clears and keeps before refit and the body count.
use crate::{RuleError, Session, story_inventory};
use centurions_content::{InventoryRecord, records::SpellFieldMut};

/// The reward mode in force: after a squad has fled or fallen every mode but 2 and
/// 10 becomes 2.
///
/// Original: booty
pub(crate) fn effective_mode(state: &Session, requested: i16) -> Result<i16, RuleError> {
    if matches!(requested, 2 | 10) {
        return Ok(requested);
    }
    Ok(if state.globals.coward == 0 {
        requested
    } else {
        2
    })
}

/// Marks the squad as fled when every warrior is out: missing, dead, turned or under
/// computer control. killbody tests this after the death story returns.
///
/// Original: killbody
pub(crate) fn after_death(state: &mut Session) -> Result<(), RuleError> {
    let mut kill_party = 0;
    for warrior in &state.warriors {
        let record = warrior.record();
        if record.position()? == -1
            || record.stamina()? < 1
            || record.faction()? != 0
            || record.computer_control()? < 0
        {
            kill_party += 1;
        }
    }
    if kill_party == state.warriors.len() {
        state.globals.coward = 1;
    }
    Ok(())
}

/// Empties the booty list and loot quality before a reward window opens outside combat, and clears
/// the count of fallen warriors.
pub(crate) fn entry(state: &mut Session) -> Result<(), RuleError> {
    if state.combat.is_some() {
        return Err(unavailable("retained booty entry while combat is active"));
    }
    if state.booty_list.len() != 1000 {
        return Err("retained booty record array length".into());
    }
    let health = &mut state.globals.health_context;
    health.kill_party = 0;
    state.globals.round_scripts.blank_round = Some(0);
    // Every booty slot starts empty.
    state.booty_list.fill(InventoryRecord([0; 22]));
    state.globals.loot_quality = Some(vec![0; 1000]);
    Ok(())
}

/// Runs after battle entry and any refit; the health fields are left alone.
pub(crate) fn normalize_out_of_combat(state: &mut Session) -> Result<(), RuleError> {
    let health = &mut state.globals.health_context;
    for warrior in &mut state.warriors {
        let kill_party = warrior.stamina()? < 1 || warrior.record().computer_control()? < 0;
        story_inventory::normalize_warrior_after_battle(warrior)?;
        if kill_party {
            health.kill_party = health.kill_party.wrapping_add(1);
        }
    }
    let fields = &mut state.globals.battlefield.spell_queue;
    for record in fields.as_chunks_mut::<20>().0 {
        let mut field = SpellFieldMut(record);
        field.set_initiative(0);
        field.set_duration(0);
    }
    for item in &mut state.booty_list {
        item.clear_id();
    }
    Ok(())
}

/// Prepares for leaving a battle early: the party is normalized and put back in the battle, and
/// every monster except those that must come along as survivors is left with no stamina.
pub(crate) fn early_out_of_combat(state: &mut Session) -> Result<(), RuleError> {
    normalize_out_of_combat(state)?;
    for warrior in &mut state.warriors {
        warrior.record_mut().set_in_battle(1);
    }
    // Every slot is read, not only the roster's.
    for index in 0..100 {
        let record = &mut state.globals.monsters.slot_mut(index)?.record;
        if record.record().survivor_marker()? != -1 {
            record.record_mut().set_stamina(0);
        }
    }
    Ok(())
}

/// The refusal for a reward step that depends on an unrecovered part of the game.
fn unavailable(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.reward_work",
        dependency,
    }
}
