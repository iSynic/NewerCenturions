//! Warrior attacks that finish after the battle has ended.
mod preparation;
mod results;
use super::{
    attacks::AttackOutcome, out_of_combat_attack::store_packet, reaction_calls::AttackCall,
};
use crate::{
    Content, RuleError, Session,
    combat::{attack, warrior_attack},
};
use warrior_attack::PreparedMeleeWeapon;

/// A warrior's attack after the battle has ended: any record of the warrior table can strike a
/// warrior or monster, and its result and any death follow.
///
/// Original: attack
pub(super) fn run(
    state: &mut Session,
    content: &Content,
    call: AttackCall,
) -> Result<AttackOutcome, RuleError> {
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
                crate::feedback::GameFeedback::Warning { index: 45 },
            )],
            returned: false,
            terminated: false,
        });
    }
    let used = record.attacks_used()?.wrapping_add(1);
    state.write_did_attack(1);
    attacker.record_mut().set_attacks_used(used);
    state.store_table_record(actor, attacker)?;
    let attacks = &state
        .land(content)
        .and_then(|land| land.combat.as_ref())
        .ok_or("detached warrior attack tables missing")?
        .attacks;
    let base = attack::spell_info(attacks)?;
    state.globals.round_scripts.blank_round = Some(0);
    store_packet(state, base.clone());
    if !super::has_active_warrior(&state.warriors) {
        state.globals.kill_counts.finish();
        return Ok(AttackOutcome::default());
    }
    let prepared = preparation::prepare(state, content, actor, base)?;
    let (base, rolls_condition, weapon, armed) = match prepared {
        preparation::Prepared::Halt { warned } => {
            let events = if warned {
                super::death_return::refresh_actor(state, content)?;
                vec![crate::Event::Feedback(
                    crate::feedback::GameFeedback::Warning { index: 39 },
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
        .get(state.char_select_new)
        .ok_or_else(|| RuleError::from("selected warrior outside the squad"))?
        .bytes
        .clone();
    let bare_hand_bonus = state.globals.character_bare_hand_bonus;
    let weapon = Some(PreparedMeleeWeapon {
        definition: &weapon,
        armed,
    });
    let (result, packet, actual, refusal) = if (10..110).contains(&call.recipient) {
        let slot = (call.recipient - 10) as usize;
        let mut attacker = state
            .table_record(actor)
            .ok_or("detached warrior attacker outside the table")?;
        let monster = &mut state.globals.monsters.slot_mut(slot)?.record;
        let (result, packet, reflected, refusal) = attack::warrior_attack_prepared(
            &mut attacker,
            monster,
            &mut state.random,
            base,
            state.globals.attack_count,
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
        // attack takes recipients below 10 as warrior records, the rest as monster records.
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
            state.globals.attack_count,
            rolls_condition,
            crate::combat::party_buffs::mind_shield(&state.party_condition)?,
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

/// The refusal for a warrior attack that depends on an unrecovered part of the game.
fn unavailable(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.detached_warrior",
        dependency,
    }
}

/// Loads the weapon of a guarding warrior who is about to react; a warrior in firing mode with no
/// second weapon leaves firing mode.
pub(super) fn refresh_guard(
    state: &mut Session,
    content: &Content,
    body: i16,
) -> Result<(), RuleError> {
    let mut warrior = usize::try_from(body)
        .ok()
        .and_then(|body| state.table_record(body))
        .ok_or_else(|| RuleError::from("retained guard outside the warrior table"))?;
    let secondary = warrior.record().toggle();
    let id = if secondary {
        warrior.record().equipped_weapon()?
    } else {
        warrior.record().melee_weapon()?
    };
    // Only the active body loads item 0; a missing second weapon clears firing mode.
    if id != 0 || super::reaction_calls::turn_actor(state)? == body {
        state.load_item(content, id)?;
    }
    if secondary && id == 0 {
        warrior.record_mut().set_toggle(false);
        state.store_table_record(body as usize, warrior)?;
    }
    Ok(())
}
