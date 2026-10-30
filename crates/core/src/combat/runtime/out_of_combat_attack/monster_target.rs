//! A monster striking another monster after the battle has ended.
use super::{preparation::Prepared, store_packet, unavailable};
use crate::combat::ally_turns::{MonsterMelee, natural_core};
use crate::combat::runtime::{attacks::AttackOutcome, reaction_calls::AttackCall};
use crate::{Event, RuleError, Session, feedback::GameFeedback};
use centurions_content::combat::MonsterRecord;

/// How a monster's attack on another monster ended: refused because of weapon immunity, or struck.
pub(super) enum Strike {
    /// Weapon immunity: the refusal shows on the attacker and the attack ends.
    Refused(AttackOutcome),
    Struck(MonsterMelee),
}

/// Resolves a monster's attack on another monster: accuracy, damage, conditions and stamina. A
/// weapon-immune target refuses the attack (results -50 and -51 also drop the attacker's weapon);
/// otherwise the caller shows the result and defeats a body that fell.
///
/// Original: attack2
pub(super) fn strike(
    state: &mut Session,
    (slot, target): (usize, usize),
    mut monster: MonsterRecord,
    prepared: Prepared,
    (call, base): (AttackCall, Vec<u8>),
) -> Result<Strike, RuleError> {
    if target == slot {
        return Err(unavailable("attack2 against its own NPC record"));
    }
    let backing = &state.globals.monsters;
    let victim = backing.slot(target)?.record.clone();
    let x = |slot: usize| -> Result<i16, RuleError> {
        Ok(backing
            .slot(slot)?
            .position()?
            .0
            .wrapping_sub(backing.origin.0) as i16)
    };
    let positions = (x(slot)?, x(target)?);
    // Loading the weapon writes the selected monster's item; see run.
    let selection = backing.slot(slot)?.record.record().weapon()?;
    // The attacker is read as it stood when the attack began.
    let source = monster.bytes.clone();
    let mut records = [monster.bytes.clone(), victim.bytes.clone()];
    let outcome = match prepared {
        // The weapon's damage bounds stand; the natural attack supplies the rest.
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
    monster.bytes = attacker;
    let mut victim = victim;
    victim.bytes = struck;
    // A hit marks the struck monster before damage is resolved.
    if outcome.hit && outcome.immune.is_none() {
        let record = if outcome.reflected {
            &mut monster
        } else {
            &mut victim
        };
        record.record_mut().mark_been_attacked();
    }
    monster.record_mut().set_weapon(selection);
    // Immunity results -50 and -51 drop the attacker's weapon, then show -52.
    let dropped = matches!(outcome.immune, Some(-50 | -51));
    if dropped {
        monster.record_mut().set_weapon(0);
    }
    let backing = &mut state.globals.monsters;
    backing.slot_mut(slot)?.record = monster;
    backing.slot_mut(target)?.record = victim;
    store_packet(state, outcome.packet.clone());
    if let Some(code) = outcome.immune {
        state.globals.neighbors.departure = 0;
        let mut events = vec![];
        for code in std::iter::once(code).chain(dropped.then_some(-52)) {
            events.push(Event::Feedback(GameFeedback::CombatResult {
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
            .globals
            .kill_counts
            .add_num_enemy(outcome.victory_delta);
    }
    Ok(Strike::Struck(outcome))
}
