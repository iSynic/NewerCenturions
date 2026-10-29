//! Exchange's flat-array writers at 0x11c564..0x11c854.
use super::{Outcome, npc_index, unsupported};
use crate::{Content, Event, RuleError, Session};

pub(super) fn apply(
    state: &mut Session,
    content: &Content,
    actor: usize,
    target: i16,
) -> Result<Outcome, RuleError> {
    state.set_ready(0);
    let powers: [i16; 6] = std::array::from_fn(|member| state.psi_power(member));
    let work = &mut state.native_work.tactical_work;
    let from = work
        .positions
        .get(actor)
        .copied()
        .ok_or_else(|| RuleError::from("occupied-cell actor outside the squad"))?;
    let to;
    if target < 10 {
        let other = usize::try_from(target)
            .map_err(|_| RuleError::from("occupied-cell exchange before the party positions"))?;
        // Party positions 0x44709a + 4 * slot (0x11c57a..0x11c5ea) and under-tile WORDs
        // 0x437a6e + 2 * slot (0x11c5ee..0x11c624) are swapped with the actor's.
        match other {
            0..6 => {
                to = work.positions[other];
                work.positions[other] = from;
                work.under_tiles.swap(actor, other);
            }
            // Slots 6 and 7 are the WORDs 0x4470b2..0x4470b7 that presentation shares.
            6 | 7 => {
                return Err(unsupported(
                    "occupied-cell exchange with a presentation-shared party slot",
                ));
            }
            // Slots 8 and 9 are initiative order rows 1..4 at 0x4470ba; their under-tile
            // WORDs are the remembered Psi powers 0x437a7a + 2 * (slot - 6).
            _ => {
                let row = other * 2 - 15;
                let order = &mut state.native_work.initiative_work.order;
                to = (order[row], order[row + 1]);
                order[row] = from.0;
                order[row + 1] = from.1;
                let member = other - 6;
                let power = powers[member];
                let word = work.under_tiles[actor];
                if let Some(choice) = state.psi_menu_choices[member].as_mut() {
                    // castspell 0x10bc0a takes the remembered power back into its menu.
                    if !(1..=7).contains(&word) {
                        return Err(unsupported(
                            "occupied-cell exchange writes a remembered Psi power outside the menu",
                        ));
                    }
                    choice.power = word;
                    state.native_work.psi_powers[member] = None;
                } else {
                    state.native_work.psi_powers[member] = (word != 0).then_some(word);
                }
                work.under_tiles[actor] = power;
            }
        }
        work.positions[actor] = to;
    } else {
        let backing = &mut state.native_work.npc_backing;
        let origin = backing.origin;
        let slot = backing.slot_mut(npc_index(target)?)?;
        let position = slot.position()?;
        to = (
            (position.0.wrapping_sub(origin.0)) as i16,
            (position.1.wrapping_sub(origin.1)) as i16,
        );
        slot.position = Some((i32::from(from.0) + origin.0, i32::from(from.1) + origin.1));
        work.positions[actor] = to;
        let under = work.under_tiles[actor];
        let other_under = slot.record.record().underlying_tile()?;
        work.under_tiles[actor] = other_under;
        slot.record.record_mut().set_underlying_tile(under);
    }
    let map = &mut work.map;
    let shift = map.shift;
    let index = |point: (i16, i16)| {
        let x = i32::from(point.0) + i32::from(shift.0) + 5;
        let y = i32::from(point.1) + i32::from(shift.1) + 5;
        x.wrapping_mul(90).wrapping_add(y)
    };
    // The writers store through the linear map index; past its edges they reach the
    // edge WORD or the land arrays, which the session holds.
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
    let movement = warrior.record().movement_points()?.wrapping_sub(5);
    warrior.record_mut().set_movement_points(movement);
    crate::combat::runtime::death_return::refresh_body(state, content, actor as i16)?;
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
