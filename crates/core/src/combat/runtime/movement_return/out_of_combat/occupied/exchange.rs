//! Swapping places with a friend.
use super::{Outcome, monster_index, unsupported};
use crate::combat::runtime::death_return;
use crate::{Content, Event, RuleError, Session};

/// A warrior swaps places with the friend in the cell: their positions and the tiles beneath them
/// are exchanged, and the warrior pays 5 movement.
///
/// Original: combatchoice
pub(super) fn apply(
    state: &mut Session,
    content: &Content,
    actor: usize,
    target: i16,
) -> Result<Outcome, RuleError> {
    state.set_in_spell(0);
    let powers: [i16; 6] = std::array::from_fn(|member| state.psi_power(member));
    let battlefield = &mut state.globals.battlefield;
    let from = battlefield
        .pos
        .get(actor)
        .copied()
        .ok_or_else(|| RuleError::from("occupied-cell actor outside the squad"))?;
    let to;
    if target < 10 {
        let other = usize::try_from(target)
            .map_err(|_| RuleError::from("occupied-cell exchange before the party positions"))?;
        // The positions and the tiles beneath are swapped with the actor's.
        match other {
            0..6 => {
                to = battlefield.pos[other];
                battlefield.pos[other] = from;
                battlefield.char_under.swap(actor, other);
            }
            // Slots 6 and 7 lie in memory that presentation shares.
            6 | 7 => {
                return Err(unsupported(
                    "occupied-cell exchange with a presentation-shared party slot",
                ));
            }
            // Slots 8 and 9 are battle order rows 1..4; their tiles beneath are the
            // remembered psi powers (see docs/memory-alias-audit.md).
            _ => {
                let row = other * 2 - 15;
                let order = &mut state.globals.battle_order.order;
                to = (order[row], order[row + 1]);
                order[row] = from.0;
                order[row + 1] = from.1;
                let member = other - 6;
                let power = powers[member];
                let word = battlefield.char_under[actor];
                if let Some(choice) = state.psi_menu_choices[member].as_mut() {
                    // castspell offers the remembered power in its menu.
                    if !(1..=7).contains(&word) {
                        return Err(unsupported(
                            "occupied-cell exchange writes a remembered Psi power outside the menu",
                        ));
                    }
                    choice.power = word;
                    state.globals.old_power_level[member] = None;
                } else {
                    state.globals.old_power_level[member] = (word != 0).then_some(word);
                }
                battlefield.char_under[actor] = power;
            }
        }
        battlefield.pos[actor] = to;
    } else {
        let backing = &mut state.globals.monsters;
        let origin = backing.origin;
        let slot = backing.slot_mut(monster_index(target)?)?;
        let position = slot.position()?;
        to = (
            (position.0.wrapping_sub(origin.0)) as i16,
            (position.1.wrapping_sub(origin.1)) as i16,
        );
        slot.position = Some((i32::from(from.0) + origin.0, i32::from(from.1) + origin.1));
        battlefield.pos[actor] = to;
        let under = battlefield.char_under[actor];
        let other_under = slot.record.record().underlying_tile()?;
        battlefield.char_under[actor] = other_under;
        slot.record.record_mut().set_underlying_tile(under);
    }
    let map = &mut battlefield.field;
    let shift = map.map_shift;
    let index = |point: (i16, i16)| {
        let (x, y) = crate::combat::records::grid_point(point, shift);
        x.wrapping_mul(90).wrapping_add(y)
    };
    // Writes past the map's edges reach the edge word or the land arrays.
    let mut land = state
        .exploration
        .as_mut()
        .map(|position| (&mut position.objects[..], &mut position.events[..]));
    for (point, value) in [(to, actor as i16), (from, target)] {
        let land = land
            .as_mut()
            .map(|(objects, events)| (&mut **objects, &mut **events));
        if !crate::combat::map_alias::write(map, land, index(point), value) {
            return Err(unsupported(
                "occupied-cell exchange writes beside the tactical map outside its edge WORD and land arrays",
            ));
        }
    }
    let mut events = vec![Event::Sound { resource_id: 654 }];
    state
        .exploration
        .as_mut()
        .ok_or_else(|| RuleError::from("occupied-cell walk outside a mission"))?
        .last_move = [0, 0];
    let warrior = state
        .warriors
        .get_mut(actor)
        .ok_or_else(|| RuleError::from("selected warrior outside the squad"))?;
    let movement = warrior.record().movement()?.wrapping_sub(5);
    warrior.record_mut().set_movement(movement);
    death_return::refresh_body(state, content, actor as i16)?;
    events.extend([
        Event::TacticalMoved {
            body: actor as i16,
            x: to.0,
            y: to.1,
        },
        Event::TacticalMoved {
            body: target,
            x: from.0,
            y: from.1,
        },
    ]);
    Ok(Outcome::returned(events))
}
