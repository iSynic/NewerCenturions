//! Booty's integer following-record stores after the ordinary award loop.
use crate::{RuleError, Session};

/// The following-record stores address the native 100-record allocation.
pub(crate) fn validate(count: i16) -> Result<(), RuleError> {
    if count > 100 {
        return Err("reward cleanup count outside native record allocation".into());
    }
    Ok(())
}

pub(crate) fn finish(state: &mut Session, count: i16, mode: i16) -> Result<(), RuleError> {
    state.retain_monster_work();
    validate(count)?;
    // A negative signed count skips the loop with cursor0; exact zero skips
    // the following stores entirely.1060e4 clears the count in either case.
    // Mode5 reuses the cursor for1000 crate rows, so a positive count stores
    // into slot1001, past the top of the A5 world and so outside the
    // application partition. Compatibility policy: those stores are skipped.
    if count != 0 && !(mode == 5 && count > 0) {
        let following = count.max(0) as usize;
        if following == 100 {
            state.native_work.monster_work.clear_reward_fields();
        } else if let Some(slot) = state.native_work.npc_backing.slots[following].as_mut() {
            slot.record.record_mut().clear_conditions();
            slot.record.record_mut().set_faction(0);
            slot.record.record_mut().set_stamina(0);
        }
        // An absent whole NPC record remains unknown; do not invent its244
        // unwritten bytes or a position merely to retain these field stores.
    }
    state.native_work.npc_backing.active_count = 0;
    Ok(())
}
