use crate::{RandomState, RuleError};
use centurions_content::{Warrior, span, word};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartyOrder {
    pub warriors: Vec<Warrior>,
    pub order: Vec<i16>,
}

pub fn initiative(
    warriors: &[Warrior],
    enemy_speeds: &[i16],
    surprise: i16,
    unavailable: i8,
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
    for (index, warrior) in warriors.iter().enumerate() {
        span(&warrior.bytes, 0, 3138)?;
        if word(&warrior.bytes, 0x90)? != 0 {
            return Err(RuleError::Unsupported {
                action: "combat.initiative",
                dependency: "confused warrior initiative",
            });
        }
        speeds.push(word(&warrior.bytes, 0x9e8)?);
        if warrior.stamina()? > 0 {
            living.push(index as i16);
        }
    }
    let bound = warriors.len() as i16 - i16::from(unavailable) + enemy_speeds.len() as i16;
    let inserted = (warriors.len() as i8 - 1)
        .wrapping_sub(unavailable)
        .wrapping_add(1) as u8 as usize;
    if !(1..110).contains(&bound) || (surprise == 0 && inserted > living.len()) {
        return Err(RuleError::Unsupported {
            action: "combat.initiative",
            dependency: "unavailable counter exceeds living-list capacity",
        });
    }
    let mut order = vec![-1; 110];
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
        let first = living.len() - inserted;
        while remaining != 0 {
            let slot = next_random.bound(bound) as usize;
            if order[slot] == -1 {
                remaining -= 1;
                order[slot] = living[first + remaining];
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
            next_warriors[*body as usize].bytes[0x20..0x22]
                .copy_from_slice(&(slot as i16).to_be_bytes());
        }
    }
    *random = next_random;
    Ok(PartyOrder {
        warriors: next_warriors,
        order,
    })
}
