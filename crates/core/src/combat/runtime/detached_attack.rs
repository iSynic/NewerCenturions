//! attack2 over the retained arrays with combat mode clear.
use super::{attacks::AttackOutcome, reaction_calls::AttackCall};
use crate::{Content, RuleError, Session, combat::attack};
use centurions_content::records::AttackPacket;
mod npc_target;
mod preparation;
pub(super) mod results;
use preparation::{Prepared, positions, prepare};

pub(super) fn run(
    state: &mut Session,
    content: &Content,
    call: AttackCall,
) -> Result<AttackOutcome, RuleError> {
    // attack2 0x101eb6 copies the target's warrior record and branches on target < 9;
    // an NPC target reads its record at 0x439d3c instead.
    let target = match usize::try_from(call.recipient) {
        // Records past the squad keep the bytes the table holds for them.
        Ok(target @ (0..6 | 8..10)) => Ok(target),
        Ok(6..8) => return Err(unavailable("attack2 target in an Upgrade record")),
        Ok(target @ 10..110) => Err(target - 10),
        Ok(_) => return Err(unavailable("attack2 target past the NPC array")),
        Err(_) => return Err(unavailable("attack2 target before the warrior table")),
    };
    // It then copies the attacker's NPC record at 0x439d3c + (attacker - 10) * 0x120.
    let slot = usize::try_from(call.attacker.wrapping_sub(10))
        .ok()
        .filter(|slot| *slot < 100)
        .ok_or_else(|| unavailable("attack2 attacker outside the NPC array"))?;
    let mut enemy = state.native_work.npc_backing.slot(slot)?.record.clone();
    let mission = state.land(content).ok_or("detached attack map missing")?;
    let tables = mission
        .combat
        .as_ref()
        .ok_or("detached attack tables missing")?;
    let base = attack::attack_packet(&tables.attacks)?;
    state.native_work.round_work.inactive_rounds = Some(0);
    state.write_npc_inhibit(1);
    store_packet(state, base.clone());
    if !super::has_active_warrior(&state.warriors) {
        state.native_work.victory_work.finish();
        return Ok(AttackOutcome::default());
    }
    // Natural attack n reads bytes 0x93 + 4n of attack2's NPC record copy (local_124);
    // n from 35 reads the rest of its frame.
    if call.index >= 35 {
        return Err(unavailable(
            "detached attack index past attack2's NPC record copy",
        ));
    }
    let prepared = prepare(state, content, &enemy)?;
    let target = match target {
        Ok(target) => target,
        Err(npc) => {
            return match npc_target::strike(state, (slot, npc), enemy, prepared, (call, base))? {
                npc_target::Strike::Refused(outcome) => Ok(outcome),
                npc_target::Strike::Struck(struck) => {
                    results::finish(state, content, call, struck.result, struck.reflected)
                }
            };
        }
    };
    let (origin_x, target_x) = positions(state, slot, target)?;
    let copy = enemy.record().facing_flag();
    enemy.record_mut().face_towards(origin_x, target_x);
    // Attack 29 starts at 0x107, the facing byte attack2 rewrites in the live record
    // after taking its copy; the kernels read the live record.
    if call.index == 29 && enemy.record().facing_flag() != copy {
        return Err(unavailable(
            "detached attack 29 reads the facing byte attack2 rewrote",
        ));
    }
    // loadweapon's selected-item writer addresses the shared selected NPC, which
    // can differ from the guard attacker. The arithmetic kernel owns no such alias.
    let selection = state
        .native_work
        .npc_backing
        .slot(slot)?
        .record
        .record()
        .selected_item()?;
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
            &state.party_buffs,
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
                &state.party_buffs,
            )?;
            (result, packet, reflected, 0)
        }
    };
    enemy.record_mut().set_selected_item(selection);
    state.store_table_record(target, victim)?;
    state.native_work.npc_backing.slot_mut(slot)?.record = enemy;
    store_packet(state, packet);
    if total_delta != 0 {
        state.native_work.victory_work.add_total(total_delta);
        if !reflected {
            let health = &mut state.native_work.health_context;
            health.unavailable = health.unavailable.wrapping_add(total_delta as i8);
        }
    }
    results::finish(state, content, call, result, reflected)
}

pub(super) fn store_packet(state: &mut Session, packet: Vec<u8>) {
    let health = &mut state.native_work.health_context;
    health.effect = AttackPacket(&packet).health_effect();
    state.native_work.attack_packet = packet;
}

fn unavailable(dependency: &'static str) -> RuleError {
    RuleError::Unsupported {
        action: "combat.detached_attack",
        dependency,
    }
}
