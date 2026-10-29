//! Rule-side combatupdate2 work after killbody's newland returns.
use crate::{RuleError, Session};
use centurions_content::Content;

pub(super) fn refresh_actor(state: &mut Session, content: &Content) -> Result<(), RuleError> {
    let body = if let Some(combat) = &state.combat {
        combat
            .order
            .get(combat.cursor)
            .copied()
            .unwrap_or(combat.cursor as i16)
    } else {
        state
            .native_work
            .initiative_work
            .current(state.native_work.selected_enemy)?
    };
    refresh_body(state, content, body)
}

pub(in crate::combat::runtime) fn refresh_body(
    state: &mut Session,
    content: &Content,
    body: i16,
) -> Result<(), RuleError> {
    if body == -1 {
        return Ok(());
    }
    if (0..10).contains(&body) {
        // Bodies 0..9 address 0x448aa0 + body * 0xc42: the squad, the records past
        // it, the Upgrade records and the first two editor backup records.
        let body = body as usize;
        let mut warrior = state
            .table_record(body)
            .ok_or("callback refresh body outside the warrior table")?;
        let record = warrior.record();
        let secondary = record.firing_mode();
        let item = if secondary {
            record.equipped_weapon()?
        } else {
            record.melee_weapon()?
        };
        // The active warrior's early panel branch loads even record zero.
        state.load_item_work(content, item)?;
        if secondary && item == 0 {
            warrior.record_mut().set_firing_mode(false);
            state.store_table_record(body, warrior)?;
        } else if item != 0 {
            state.load_item_work(content, item)?;
        }
    } else if (10..110).contains(&body) {
        let index = (body - 10) as usize;
        let enemy = if let Some(combat) = &state.combat {
            combat.story_record(index)?
        } else {
            &state.native_work.npc_backing.slot(index)?.record
        };
        let item = enemy.record().selected_item()?;
        if item != 0 {
            state.load_item_work(content, item)?;
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
