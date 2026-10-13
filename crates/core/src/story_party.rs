use crate::{RandomState, RuleError, character, combat, story_variables::StoryContinuation};
use centurions_content::{Warrior, mission::OpeningMission, span, word};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PartySaveContext {
    pub casting: bool,
    pub packet_type: i8,
    pub bonus: i8,
    pub bonus_per_attack: i8,
    pub attack_count: i16,
    pub mind_shield: bool,
}

/// newland 52 clears the party mask before testing each member.
pub fn select_party(
    warriors: &[Warrior],
    selected_warrior: usize,
    extra: &[u8],
    context: PartySaveContext,
    random: &mut RandomState,
) -> Result<[i16; 6], RuleError> {
    if warriors.len() > 6 {
        return Err(RuleError::Invalid("party exceeds six warriors".into()));
    }
    let extra = span(extra, 0, 10)?;
    let kind = word(extra, 0)?;
    let value = word(extra, 2)?;
    let eligible_mode = word(extra, 4)?;
    let mut mask = [0; 6];
    let mut next_random = random.clone();
    for (member, warrior) in warriors.iter().enumerate() {
        let bytes = span(&warrior.bytes, 0, 3138)?;
        let eligible = eligible_mode == 0 || (eligible_mode == 1 && word(bytes, 0x50)? > 0);
        if !eligible {
            continue;
        }
        let holds_item = || -> Result<bool, RuleError> {
            let count = word(bytes, 0x1a)?;
            if count > 100 {
                return Err(RuleError::Invalid(
                    "inventory exceeds original record".into(),
                ));
            }
            for slot in 0..count.max(0) as usize {
                if word(bytes, 0x132 + slot * 22)? == value {
                    return Ok(true);
                }
            }
            Ok(false)
        };
        let matched = match kind {
            0 => word(bytes, 0x3e)? < value,
            1 => (member as i16).wrapping_add(1) < value,
            2 => holds_item()?,
            3 => next_random.bound(100) <= value,
            4 => !character::save_against_attribute(warrior, value, &mut next_random)?,
            5 => {
                if context.casting && context.packet_type > 1 {
                    true
                } else {
                    if !(0..9).contains(&value) {
                        return Err(RuleError::Unsupported {
                            action: "story.select_party",
                            dependency: "save selector",
                        });
                    }
                    let mut bonus = i16::from(context.bonus_per_attack)
                        .wrapping_mul(context.attack_count)
                        .wrapping_add(i16::from(context.bonus));
                    if value == 0 && context.mind_shield {
                        bonus = bonus.wrapping_add(50);
                    }
                    next_random.bound(100)
                        > word(bytes, 0x112 + value as usize * 2)?.wrapping_add(bonus)
                }
            }
            6 => {
                let slot = mask.get_mut(selected_warrior).ok_or_else(|| {
                    RuleError::Invalid("selected warrior outside party mask".into())
                })?;
                *slot = 1;
                false
            }
            7 => {
                let mut equipped = false;
                if holds_item()? {
                    for slot in 0..=word(bytes, 0x1a)?.max(0) as usize {
                        let offset = 0x132 + slot * 22;
                        if word(bytes, offset)? == value && bytes[offset + 9] != 0 {
                            equipped = true;
                        }
                    }
                }
                equipped
            }
            _ => false,
        };
        if matched {
            mask[member] = 1;
        }
    }
    *random = next_random;
    Ok(mask)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupEncounter {
    pub battle: i16,
    pub sound: i16,
    pub message: i16,
    pub failure_script: i16,
}

/// newland 107 shares battle selection with opcode 2; its fifth word is a retreat script.
pub fn group_encounter(
    extra: &[u8],
    random: &mut RandomState,
) -> Result<GroupEncounter, RuleError> {
    let plan = combat::encounter_battle(extra, random)?;
    Ok(GroupEncounter {
        battle: plan.battle,
        sound: plan.sound,
        message: plan.message,
        failure_script: plan.treasure,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupSetup {
    pub warriors: Vec<Warrior>,
    pub map: combat::TacticalMap,
    pub squad: combat::SquadPlacement,
    pub order: Vec<i16>,
}

/// combatsetup 11ec2a removes inactive members after formation and initiative.
pub fn apply_group_mask(
    warriors: &[Warrior],
    map: &combat::TacticalMap,
    squad: &combat::SquadPlacement,
    order: &[i16],
    mask: &[i16; 6],
    group: bool,
) -> Result<GroupSetup, RuleError> {
    if !(1..=6).contains(&warriors.len())
        || squad.positions.len() != warriors.len()
        || squad.under_tiles.len() != warriors.len()
    {
        return Err(RuleError::Invalid("group formation missing".into()));
    }
    let mut result = GroupSetup {
        warriors: warriors.to_vec(),
        map: map.clone(),
        squad: squad.clone(),
        order: order.to_vec(),
    };
    for (member, mask_value) in mask.iter().take(warriors.len()).enumerate() {
        let bytes = &mut result.warriors[member].bytes;
        span(bytes, 0, 3138)?;
        bytes[0x9df] = 1;
        if word(bytes, 0x50)? > 0 && (!group || *mask_value != 0) {
            continue;
        }
        let position = result.squad.positions[member];
        let index = OpeningMission::index(
            i32::from(position.0) + i32::from(map.shift.0) + 5,
            i32::from(position.1) + i32::from(map.shift.1) + 5,
        )?;
        let tile = result
            .map
            .tiles
            .get_mut(index)
            .ok_or_else(|| RuleError::Invalid("group formation tile unavailable".into()))?;
        *tile = result.squad.under_tiles[member];
        bytes[0x9df] = 0;
        let initiative = word(bytes, 0x20)?;
        if initiative != -1 {
            let initiative = usize::try_from(initiative)
                .map_err(|_| RuleError::Invalid("warrior initiative is negative".into()))?;
            *result
                .order
                .get_mut(initiative)
                .ok_or_else(|| RuleError::Invalid("warrior initiative unavailable".into()))? = -1;
        }
        bytes[0x20..0x22].copy_from_slice(&(-1i16).to_be_bytes());
        result.squad.positions[member] = (-1, -1);
    }
    Ok(result)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupReturn {
    pub warriors: Vec<Warrior>,
    pub continuation: StoryContinuation,
}

/// newland 107 restores participation after combat; only retreat replaces the script.
pub fn group_return(
    warriors: &[Warrior],
    retreated: bool,
    failure_script: i16,
) -> Result<GroupReturn, RuleError> {
    let mut warriors = warriors.to_vec();
    for warrior in &mut warriors {
        span(&warrior.bytes, 0, 3138)?;
        warrior.bytes[0x9df] = 1;
    }
    Ok(GroupReturn {
        warriors,
        continuation: if retreated {
            StoryContinuation::ReplaceScript(failure_script)
        } else {
            StoryContinuation::Continue
        },
    })
}
