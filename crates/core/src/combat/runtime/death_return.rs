//! Refreshing a body's weapon after its death story returns.
use crate::{RuleError, Session};
use centurions_content::Content;

/// Reloads the weapon of the body whose turn it is, which a death script may have changed.
pub(super) fn refresh_actor(state: &mut Session, content: &Content) -> Result<(), RuleError> {
    let body = if let Some(combat) = &state.combat {
        combat
            .battle_order
            .get(combat.battle_order_index)
            .copied()
            .unwrap_or(combat.battle_order_index as i16)
    } else {
        state
            .globals
            .battle_order
            .current(state.globals.monster_up)?
    };
    refresh_body(state, content, body)
}

/// Reloads the weapon and attack context of one body after a death script, for a warrior or a
/// monster.
pub(in crate::combat::runtime) fn refresh_body(
    state: &mut Session,
    content: &Content,
    body: i16,
) -> Result<(), RuleError> {
    if body == -1 {
        return Ok(());
    }
    if (0..10).contains(&body) {
        // Bodies 0..9 are warrior records: the squad, the records past it, the
        // Upgrade records and the first two editor backup records.
        let body = body as usize;
        let mut warrior = state
            .table_record(body)
            .ok_or("callback refresh body outside the warrior table")?;
        let record = warrior.record();
        let secondary = record.toggle();
        let item = if secondary {
            record.equipped_weapon()?
        } else {
            record.melee_weapon()?
        };
        // The active warrior's early panel branch loads even record zero.
        state.load_item(content, item)?;
        if secondary && item == 0 {
            warrior.record_mut().set_toggle(false);
            state.store_table_record(body, warrior)?;
        } else if item != 0 {
            state.load_item(content, item)?;
        }
    } else if (10..110).contains(&body) {
        let index = (body - 10) as usize;
        let monster = if let Some(combat) = &state.combat {
            combat.story_record(index)?
        } else {
            &state.globals.monsters.slot(index)?.record
        };
        let item = monster.record().weapon()?;
        if item != 0 {
            state.load_item(content, item)?;
        }
    } else if body < 0 {
        return Err(RuleError::Unsupported {
            action: "combat.death_return",
            dependency: "callback refresh body before the warrior table",
        });
    } else {
        return Err(RuleError::Unsupported {
            action: "combat.death_return",
            dependency: "callback refresh body past the NPC array",
        });
    }
    Ok(())
}
