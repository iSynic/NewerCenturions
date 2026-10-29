//! attack 0x100af0 after the combat owner has returned to exploration.
mod preparation;
mod results;
use super::{attacks::AttackOutcome, detached_attack::store_packet, reaction_calls::AttackCall};
use crate::{
    Content, RuleError, Session,
    combat::{attack, warrior_attack},
};
use warrior_attack::PreparedMeleeWeapon;

pub(super) fn run(
    state: &mut Session,
    content: &Content,
    call: AttackCall,
) -> Result<AttackOutcome, RuleError> {
    // attack 0x100af0 indexes the attacker's warrior record and counts its attack.
    let actor = match usize::try_from(call.attacker) {
        // The squad, the records past it, the Upgrade records and the editor backup.
        Ok(actor @ 0..14) => actor,
        _ => {
            return Err(unavailable(
                "detached warrior attacker outside the warrior table",
            ));
        }
    };
    let mut attacker = state
        .table_record(actor)
        .ok_or("detached warrior attacker outside the table")?;
    let record = attacker.record();
    if record.attacks_used()? >= record.attack_limit()? {
        return Ok(AttackOutcome {
            events: vec![crate::Event::Feedback(
                crate::feedback::NativeFeedback::Warning { index: 45 },
            )],
            returned: false,
            terminated: false,
        });
    }
    let used = record.attacks_used()?.wrapping_add(1);
    state.write_npc_inhibit(1);
    attacker.record_mut().set_attacks_used(used);
    state.store_table_record(actor, attacker)?;
    let attacks = &state
        .land(content)
        .and_then(|land| land.combat.as_ref())
        .ok_or("detached warrior attack tables missing")?
        .attacks;
    let base = attack::attack_packet(attacks)?;
    state.native_work.round_work.inactive_rounds = Some(0);
    store_packet(state, base.clone());
    if !super::has_active_warrior(&state.warriors) {
        state.native_work.victory_work.finish();
        return Ok(AttackOutcome::default());
    }
    let prepared = preparation::prepare(state, content, actor, base)?;
    let (base, rolls_condition, weapon, armed) = match prepared {
        preparation::Prepared::Halt { warned } => {
            let events = if warned {
                super::death_return::refresh_actor(state, content)?;
                vec![crate::Event::Feedback(
                    crate::feedback::NativeFeedback::Warning { index: 39 },
                )]
            } else {
                vec![]
            };
            return Ok(AttackOutcome {
                events,
                returned: false,
                terminated: false,
            });
        }
        preparation::Prepared::Strike {
            packet,
            rolls_condition,
            weapon,
            armed,
        } => (packet, rolls_condition, weapon, armed),
    };
    let selected = state
        .warriors
        .get(state.selected_warrior)
        .ok_or_else(|| RuleError::from("selected warrior outside the squad"))?
        .bytes
        .clone();
    let bare_hand_bonus = state.native_work.character_bare_hand_bonus;
    let weapon = Some(PreparedMeleeWeapon {
        definition: &weapon,
        armed,
    });
    let (result, packet, actual, refusal) = if (10..110).contains(&call.recipient) {
        let slot = (call.recipient - 10) as usize;
        let mut attacker = state
            .table_record(actor)
            .ok_or("detached warrior attacker outside the table")?;
        let enemy = &mut state.native_work.npc_backing.slot_mut(slot)?.record;
        let (result, packet, reflected, refusal) = attack::warrior_attack_prepared(
            &mut attacker,
            enemy,
            &mut state.random,
            base,
            state.native_work.attack_count,
            state.difficulty,
            weapon,
            rolls_condition,
            &selected,
            bare_hand_bonus,
        )?;
        state.store_table_record(actor, attacker)?;
        (
            result,
            packet,
            if reflected {
                call.attacker
            } else {
                call.recipient
            },
            refusal,
        )
    } else {
        // attack takes recipients below 10 as warrior records, the rest as NPC records.
        let target = match usize::try_from(call.recipient) {
            // Records past the squad, the Upgrade records and the editor backup.
            Ok(target @ 0..10) => target,
            Ok(_) => return Err(unavailable("detached warrior target past the NPC array")),
            Err(_) => {
                return Err(unavailable(
                    "detached warrior target before the warrior table",
                ));
            }
        };
        let squad = state.warriors.len();
        let mut records = state.warriors.clone();
        for body in squad..=target.max(actor) {
            records.push(
                state
                    .table_record(body)
                    .ok_or("detached warrior target outside the warrior table")?,
            );
        }
        let (result, packet, actual) = warrior_attack::attack_prepared(
            &mut records,
            actor,
            target,
            base,
            state.native_work.attack_count,
            rolls_condition,
            crate::combat::party_buffs::mind_shield(&state.party_buffs)?,
            weapon,
            &selected,
            bare_hand_bonus,
            state.difficulty,
            &mut state.random,
        )?;
        for (body, record) in records.into_iter().enumerate() {
            state.store_table_record(body, record)?;
        }
        (result, packet, actual as i16, None)
    };
    store_packet(state, packet);
    results::finish(state, content, call, actual, result, refusal)
}

fn unavailable(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.detached_warrior",
        dependency,
    }
}

pub(super) fn refresh_guard(
    state: &mut Session,
    content: &Content,
    body: i16,
) -> Result<(), RuleError> {
    let mut warrior = usize::try_from(body)
        .ok()
        .and_then(|body| state.table_record(body))
        .ok_or_else(|| RuleError::from("retained guard outside the warrior table"))?;
    let secondary = warrior.record().firing_mode();
    let id = if secondary {
        warrior.record().equipped_weapon()?
    } else {
        warrior.record().melee_weapon()?
    };
    // combatupdate2 loads item 0 only for the active actor; 11fb7a clears the mode byte.
    if id != 0 || super::reaction_calls::retained_actor(state)? == body {
        state.load_item_work(content, id)?;
    }
    if secondary && id == 0 {
        warrior.record_mut().set_firing_mode(false);
        state.store_table_record(body as usize, warrior)?;
    }
    Ok(())
}
