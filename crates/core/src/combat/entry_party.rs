use crate::{RandomState, RuleError};
use centurions_content::WARRIOR_BYTES;
use centurions_content::{Warrior, span};

/// The warriors, after being dealt into the battle order, and the order itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartyOrder {
    pub warriors: Vec<Warrior>,
    pub order: Vec<i16>,
}

/// Deals the living warriors into random battle-order slots. A warrior on
/// automatic adds a slot, which `killparty` normally cancels. When `killparty`
/// undercounts, the draw runs past the living list and may place a warrior twice
/// or leave a drawn slot empty (see docs/source-notes/combat.md).
///
/// Original: getbattleorder
pub fn initiative(
    warriors: &[Warrior],
    enemy_speeds: &[i16],
    surprise: i16,
    kill_party: i8,
    random: &mut RandomState,
) -> Result<PartyOrder, RuleError> {
    if !(1..=6).contains(&warriors.len())
        || enemy_speeds.len() > 100
        || !(-1..=1).contains(&surprise)
    {
        return Err(RuleError::Invalid(
            "initiative party or enemy capacity invalid".into(),
        ));
    }
    let mut living = Vec::new();
    let mut speeds = Vec::new();
    let mut marked = 0_i8;
    for (index, warrior) in warriors.iter().enumerate() {
        span(&warrior.bytes, 0, WARRIOR_BYTES)?;
        speeds.push(warrior.record().quickness()?);
        if warrior.stamina()? > 0 {
            living.push(index as i16);
            marked += i8::from(warrior.record().computer_control()? != 0);
        }
    }
    let bound = warriors.len() as i16 - i16::from(kill_party)
        + enemy_speeds.len() as i16
        + i16::from(marked);
    let inserted = (warriors.len() as i8 - 1)
        .wrapping_sub(kill_party)
        .wrapping_add(marked)
        .wrapping_add(1) as u8 as usize;
    if !(1..110).contains(&bound) {
        return Err(RuleError::Unsupported {
            action: "combat.initiative",
            dependency: "initiative bound outside the 110-row order",
        });
    }
    let frame = |index: isize| -> i16 {
        let body = match index {
            0.. => return living[index as usize],
            -1 => i16::from(marked),
            -4..=-3 => 0,
            _ => -1,
        };
        if (0..warriors.len() as i16).contains(&body) {
            body
        } else {
            -1
        }
    };
    let mut order = vec![-1; super::BODY_SLOTS];
    match surprise {
        1 => {
            for (index, body) in living.iter().enumerate() {
                order[index + 1] = *body;
            }
        }
        -1 => {
            for (index, body) in living.iter().enumerate() {
                order[109 - index] = *body;
            }
        }
        _ => {}
    }
    let free = order[1..=bound as usize]
        .iter()
        .filter(|body| **body == -1)
        .count();
    if free < enemy_speeds.len() + if surprise == 0 { inserted } else { 0 } {
        return Err(RuleError::Unsupported {
            action: "combat.initiative",
            dependency: "unavailable counter leaves insufficient initiative slots",
        });
    }
    let mut next_random = random.clone();
    if surprise == 0 {
        let mut remaining = inserted;
        let mut next = living.len() as isize;
        while remaining != 0 {
            let slot = next_random.bound(bound) as usize;
            if order[slot] == -1 {
                remaining -= 1;
                next -= 1;
                order[slot] = frame(next);
            }
        }
    }
    let mut remaining = enemy_speeds.len();
    while remaining != 0 {
        let slot = next_random.bound(bound) as usize;
        if order[slot] == -1 {
            order[slot] = remaining as i16 + 9;
            remaining -= 1;
        }
    }
    if surprise == 0 {
        let speed = |body: i16| {
            if body < 10 {
                speeds[body as usize]
            } else {
                enemy_speeds[(body - 10) as usize]
            }
        };
        let last = enemy_speeds.len() + warriors.len() - 1;
        for start in 1..=last {
            for slot in start..=last {
                if order[slot] >= 0
                    && order[slot + 1] >= 0
                    && speed(order[slot]) < speed(order[slot + 1])
                {
                    order.swap(slot, slot + 1);
                }
            }
        }
    }
    let mut next_warriors = warriors.to_vec();
    for (slot, body) in order.iter().enumerate().skip(1) {
        if (0..6).contains(body) {
            next_warriors[*body as usize]
                .record_mut()
                .set_position(slot as i16);
        }
    }
    *random = next_random;
    Ok(PartyOrder {
        warriors: next_warriors,
        order,
    })
}
