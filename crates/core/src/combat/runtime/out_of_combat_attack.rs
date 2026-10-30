//! Monster attacks that finish after the battle has ended.
use super::{attacks::AttackOutcome, reaction_calls::AttackCall};
use crate::{Content, RuleError, Session, combat::attack};
use centurions_content::records::SpellInfoRecord;
mod monster_target;
mod preparation;
pub(super) mod results;
use preparation::{Prepared, positions, prepare};

/// A monster's attack after the battle has ended. It strikes a warrior or another monster with its
/// melee weapon or a natural attack, turns to face the target, and its result and any death follow.
///
/// Original: attack2
pub(super) fn run(
    state: &mut Session,
    content: &Content,
    call: AttackCall,
) -> Result<AttackOutcome, RuleError> {
    // Targets below 9 are warrior records; the rest are monster records.
    let target = match usize::try_from(call.recipient) {
        // Records past the squad keep the bytes the table holds for them.
        Ok(target @ (0..6 | 8..10)) => Ok(target),
        Ok(6..8) => return Err(unavailable("attack2 target in an Upgrade record")),
        Ok(target @ 10..110) => Err(target - 10),
        Ok(_) => return Err(unavailable("attack2 target past the NPC array")),
        Err(_) => return Err(unavailable("attack2 target before the warrior table")),
    };
    let slot = usize::try_from(call.attacker.wrapping_sub(10))
        .ok()
        .filter(|slot| *slot < 100)
        .ok_or_else(|| unavailable("attack2 attacker outside the NPC array"))?;
    let mut enemy = state.globals.monsters.slot(slot)?.record.clone();
    let mission = state.land(content).ok_or("detached attack map missing")?;
    let tables = mission
        .combat
        .as_ref()
        .ok_or("detached attack tables missing")?;
    let base = attack::spell_info(&tables.attacks)?;
    state.globals.round_scripts.blank_round = Some(0);
    state.write_did_attack(1);
    store_packet(state, base.clone());
    if !super::has_active_warrior(&state.warriors) {
        state.globals.kill_counts.finish();
        return Ok(AttackOutcome::default());
    }
    // Natural attacks from 35 lie past the monster record.
    if call.index >= 35 {
        return Err(unavailable(
            "detached attack index past attack2's NPC record copy",
        ));
    }
    let prepared = prepare(state, content, &enemy)?;
    let target = match target {
        Ok(target) => target,
        Err(monster) => {
            return match monster_target::strike(
                state,
                (slot, monster),
                enemy,
                prepared,
                (call, base),
            )? {
                monster_target::Strike::Refused(outcome) => Ok(outcome),
                monster_target::Strike::Struck(struck) => {
                    results::finish(state, content, call, struck.result, struck.reflected)
                }
            };
        }
    };
    let (origin_x, target_x) = positions(state, slot, target)?;
    let copy = enemy.record().facing_left();
    enemy.record_mut().face_towards(origin_x, target_x);
    // Natural attack 29 overlaps the facing byte, which the attack rewrites in the
    // live record after copying it (see docs/source-notes/combat.md).
    if call.index == 29 && enemy.record().facing_left() != copy {
        return Err(unavailable(
            "detached attack 29 reads the facing byte attack2 rewrote",
        ));
    }
    // Loading the weapon writes the selected monster's item, which can differ from
    // the guarding attacker.
    let selection = state
        .globals
        .monsters
        .slot(slot)?
        .record
        .record()
        .weapon()?;
    let mut victim = state
        .table_record(target)
        .ok_or("attack2 target outside the warrior table")?;
    let (result, packet, reflected, total_delta) = match prepared {
        Prepared::Natural { fallback } => attack::natural_attack_prepared(
            &mut victim,
            &mut enemy,
            call.index,
            &mut state.random,
            base,
            &state.party_condition,
            state.difficulty,
            fallback,
        )?,
        Prepared::Weapon(weapon) => {
            let (result, packet, reflected) = crate::combat::enemy_weapons::melee_party_prepared(
                &mut victim,
                &mut enemy,
                &weapon,
                &mut state.random,
                state.difficulty,
                origin_x,
                target_x,
                call.index,
                base,
                &state.party_condition,
            )?;
            (result, packet, reflected, 0)
        }
    };
    enemy.record_mut().set_weapon(selection);
    state.store_table_record(target, victim)?;
    state.globals.monsters.slot_mut(slot)?.record = enemy;
    store_packet(state, packet);
    if total_delta != 0 {
        state.globals.kill_counts.add_num_enemy(total_delta);
        if !reflected {
            let health = &mut state.globals.health_context;
            health.kill_party = health.kill_party.wrapping_add(total_delta as i8);
        }
    }
    results::finish(state, content, call, result, reflected)
}

/// Keeps the attack packet and notes its health effect for the party.
pub(super) fn store_packet(state: &mut Session, packet: Vec<u8>) {
    let health = &mut state.globals.health_context;
    health.effect = SpellInfoRecord(&packet).health_effect();
    state.globals.spell_info = packet;
}

/// The refusal for a monster attack that depends on an unrecovered part of the game.
fn unavailable(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.detached_attack",
        dependency,
    }
}
