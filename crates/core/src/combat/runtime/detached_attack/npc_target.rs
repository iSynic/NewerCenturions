//! attack2 0x101eb6's NPC-target branch over the retained NPC records.
use super::{preparation::Prepared, store_packet, unavailable};
use crate::combat::ally_turns::{NpcMelee, natural_core};
use crate::combat::runtime::{attacks::AttackOutcome, reaction_calls::AttackCall};
use crate::{Event, RuleError, Session, feedback::NativeFeedback};
use centurions_content::combat::MonsterRecord;

pub(super) enum Strike {
    /// Weapon immunity: attack2 shows the refusal on the attacker and returns.
    Refused(AttackOutcome),
    Struck(NpcMelee),
}

/// From accuracy through the NPC condition switch and stamina. The caller presents
/// the result and runs killbody with combat mode clear.
pub(super) fn strike(
    state: &mut Session,
    (slot, target): (usize, usize),
    mut enemy: MonsterRecord,
    prepared: Prepared,
    (call, base): (AttackCall, Vec<u8>),
) -> Result<Strike, RuleError> {
    if target == slot {
        return Err(unavailable("attack2 against its own NPC record"));
    }
    let backing = &state.native_work.npc_backing;
    let victim = backing.slot(target)?.record.clone();
    let x = |slot: usize| -> Result<i16, RuleError> {
        Ok(backing
            .slot(slot)?
            .position()?
            .0
            .wrapping_sub(backing.origin.0) as i16)
    };
    let positions = (x(slot)?, x(target)?);
    // loadweapon's selected-item writer addresses the shared selected NPC; see run.
    let selection = backing.slot(slot)?.record.record().selected_item()?;
    // attack2 reads the attacker through its entry copy (local_124).
    let source = enemy.bytes.clone();
    let mut records = [enemy.bytes.clone(), victim.bytes.clone()];
    let outcome = match prepared {
        // attack2 takes local_fc's weapon damage bounds; the NPC-target core reads the
        // natural attack.
        Prepared::Natural { fallback: true } => {
            return Err(unavailable(
                "attack2 NPC target after a rejected melee weapon",
            ));
        }
        Prepared::Natural { fallback: false } => natural_core(
            (&source, &victim.bytes),
            &mut records,
            positions,
            call.index,
            state.difficulty,
            base,
            &mut state.random,
        )?,
        Prepared::Weapon(weapon) => crate::combat::enemy_weapons::melee_enemy_core(
            (&source, &victim.bytes),
            &mut records,
            positions,
            (call.index, &weapon),
            state.difficulty,
            base,
            &mut state.random,
        )?,
    };
    let [attacker, struck] = records;
    enemy.bytes = attacker;
    let mut victim = victim;
    victim.bytes = struck;
    // A hit sets the struck record's BYTE 0x105 (0x439e41) before resolvedamage.
    if outcome.hit && outcome.immune.is_none() {
        let record = if outcome.reflected {
            &mut enemy
        } else {
            &mut victim
        };
        record.record_mut().mark_damaged();
    }
    enemy.record_mut().set_selected_item(selection);
    // -50 and -51 drop the weapon: WORD 0x439d8a of the attacker, then -52.
    let dropped = matches!(outcome.immune, Some(-50 | -51));
    if dropped {
        enemy.record_mut().set_selected_item(0);
    }
    let backing = &mut state.native_work.npc_backing;
    backing.slot_mut(slot)?.record = enemy;
    backing.slot_mut(target)?.record = victim;
    store_packet(state, outcome.packet.clone());
    if let Some(code) = outcome.immune {
        state.native_work.neighbor_work.departure = 0;
        let mut events = vec![];
        for code in std::iter::once(code).chain(dropped.then_some(-52)) {
            events.push(Event::Feedback(NativeFeedback::CombatResult {
                body: call.attacker,
                code,
                amount: 0,
                audible: crate::feedback::combat_result(code).is_some(),
            }));
        }
        events.push(Event::AttackResolved {
            attacker: call.attacker,
            target: call.recipient,
            damage: 0,
            defeated: false,
            icon: crate::event::PresentationIcon(0),
            flight: Default::default(),
        });
        return Ok(Strike::Refused(AttackOutcome {
            events,
            returned: false,
            terminated: false,
        }));
    }
    if outcome.victory_delta != 0 {
        state
            .native_work
            .victory_work
            .add_total(outcome.victory_delta);
    }
    Ok(Strike::Struck(outcome))
}
