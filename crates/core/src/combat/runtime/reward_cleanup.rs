//! booty's writes to the record after the last award.
use crate::{RuleError, Session};

/// The writes after the last award stay inside the 100 monster records.
pub(crate) fn validate(count: i16) -> Result<(), RuleError> {
    if count > 100 {
        return Err("reward cleanup count outside native record allocation".into());
    }
    Ok(())
}

/// Closes the reward after the last award: the monster record after the awarded ones is cleared of
/// conditions, faction and stamina (not in a mode 5 award), and no monsters are counted any
/// longer.
///
/// Original: booty
pub(crate) fn finish(state: &mut Session, count: i16, mode: i16) -> Result<(), RuleError> {
    state.store_monster_pick();
    validate(count)?;
    // A negative count skips the award loop and writes record 0; a count of zero
    // writes nothing. Mode 5 would write record 1001, outside the game's memory, so
    // those writes are skipped (see docs/source-notes/combat.md).
    if count != 0 && !(mode == 5 && count > 0) {
        let following = count.max(0) as usize;
        if following == 100 {
            state.globals.monster_pick.clear_reward_fields();
        } else if let Some(slot) = state.globals.monsters.slots[following].as_mut() {
            slot.record.record_mut().clear_conditions();
            slot.record.record_mut().set_faction(0);
            slot.record.record_mut().set_stamina(0);
        }
        // A missing monster record stays unknown; its other bytes and position are
        // not invented for these writes.
    }
    state.globals.monsters.num_mon = 0;
    Ok(())
}
