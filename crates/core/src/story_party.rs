use crate::{RandomState, RuleError, character, combat, story_variables::StoryContinuation};
use centurions_content::records::{
    ATTRIBUTES, DevelopmentField as Field, InventorySlots, PartyFilterExtra, PartySelectionExtra,
    StatisticAdjustmentExtra, WarriorRecord, WarriorRecordMut,
};
use centurions_content::{WARRIOR_BYTES, Warrior, mission::OpeningMission, span};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PartySaveContext {
    pub casting: bool,
    pub packet_type: i8,
    pub bonus: i8,
    pub bonus_per_attack: i8,
    pub attack_count: i16,
    pub mind_shield: bool,
}

/// newland108, 0x1414d8..0x141918, updates selected signed WORD statistics.
pub fn adjust_statistics(
    warriors: &[Warrior],
    mask: &[i16; 6],
    extra: &[u8],
) -> Result<Vec<Warrior>, RuleError> {
    let extra = StatisticAdjustmentExtra::new(extra)?;
    if warriors.len() > mask.len() {
        return Err("party exceeds six warriors".into());
    }
    let selector = extra.selector()?;
    let amount = extra.amount()?;
    let mut result = warriors.to_vec();
    for (warrior, selected) in result.iter_mut().zip(mask) {
        let bytes = &mut warrior.bytes;
        span(bytes, 0, WARRIOR_BYTES)?;
        if *selected == 0 {
            continue;
        }
        let (field, minimum, conditional, current) = match selector {
            1 => (Field::ActionsPerRound, 0, false, None),
            2 => (Field::ScanCapacity, 1, true, None),
            3 => (
                Field::MovementPerRound,
                3,
                false,
                Some(Field::MovementPoints),
            ),
            4 => (Field::BareHandMinimum, 0, false, None),
            5 => (Field::MaximumPsi, 0, true, Some(Field::Psi)),
            6 => (Field::BareHandMaximum, 1, true, None),
            7 => (Field::MaximumStamina, 2, false, Some(Field::Stamina)),
            8 => (Field::Armor, 0, false, None),
            9 => (Field::MeleeAccuracyBonus, 0, false, None),
            10 => (Field::MissileAccuracyBonus, 0, false, None),
            11 => (Field::PsiResistanceBonus, 0, false, None),
            _ => continue,
        };
        let previous = field.read(bytes)?;
        if conditional && previous == 0 {
            continue;
        }
        let value = previous.wrapping_add(amount).max(minimum);
        field.write(bytes, value);
        if let Some(current) = current
            && current.read(bytes)? > value
        {
            current.write(bytes, value);
        }
    }
    Ok(result)
}

/// newland 30, 0x1447e0..0x144a5c, filters the existing party mask.
pub fn filter_party(
    warriors: &[Warrior],
    mut mask: [i16; 6],
    action: usize,
    extra: &[u8],
    random: &mut RandomState,
) -> Result<[i16; 6], RuleError> {
    if warriors.len() > 6 {
        return Err("party exceeds six warriors".into());
    }
    let extra = PartyFilterExtra::new(extra)?;
    for (member, warrior) in warriors.iter().enumerate() {
        if extra.inclusion()? == 1 || (extra.inclusion()? == 2 && warrior.stamina()? > 0) {
            mask[member] = 1;
        }
    }
    let selector = extra.selector()?;
    for (member, warrior) in warriors.iter().enumerate() {
        if mask[member] == 0 {
            continue;
        }
        mask[member] = 0;
        let matched = if extra.test_word()? == 0 {
            let source = warriors.get(action).ok_or(RuleError::Unsupported {
                action: "story.filter_party",
                dependency: "native action-index skill alias beyond party records",
            })?;
            let skill = usize::try_from(selector)
                .map_err(|_| RuleError::Invalid("negative skill selector".into()))?;
            let chance = source
                .record()
                .special_skill(skill)?
                .wrapping_add(source.record().special_skill_bonus(skill)?)
                .wrapping_add(extra.test_word()?);
            random.bound(100) <= chance
        } else if (0..=6).contains(&selector) {
            Field::Attribute(ATTRIBUTES[selector as usize]).read(&warrior.bytes)?
                > extra.attribute_threshold()?
        } else {
            false
        };
        mask[member] = i16::from(matched);
    }
    if extra.invert()? != 0 {
        for value in mask.iter_mut().take(warriors.len()) {
            *value = i16::from(*value == 0);
        }
    }
    Ok(mask)
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
    let extra = PartySelectionExtra::new(extra)?;
    let kind = extra.kind()?;
    let value = extra.value()?;
    let eligible_mode = extra.eligibility()?;
    let mut mask = [0; 6];
    let mut next_random = random.clone();
    for (member, warrior) in warriors.iter().enumerate() {
        let bytes = span(&warrior.bytes, 0, WARRIOR_BYTES)?;
        let eligible = eligible_mode == 0
            || (eligible_mode == 1 && WarriorRecord::from_bytes(bytes).stamina()? > 0);
        if !eligible {
            continue;
        }
        let holds_item = || -> Result<bool, RuleError> {
            let count = InventorySlots(bytes).count()?;
            if count > 100 {
                return Err(RuleError::Invalid(
                    "inventory exceeds original record".into(),
                ));
            }
            for slot in 0..count.max(0) as usize {
                if InventorySlots(bytes).item_id(slot)? == value {
                    return Ok(true);
                }
            }
            Ok(false)
        };
        let matched = match kind {
            0 => warrior.record().movement_per_round()? < value,
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
                        > Field::Resistance(value as usize)
                            .read(bytes)?
                            .wrapping_add(bonus)
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
                    for slot in 0..=InventorySlots(bytes).count()?.max(0) as usize {
                        if InventorySlots(bytes).item_id(slot)? == value
                            && InventorySlots(bytes).equipment_flag(slot) != 0
                        {
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
        span(bytes, 0, WARRIOR_BYTES)?;
        WarriorRecordMut::from_bytes(bytes).set_active_flag(1);
        if WarriorRecord::from_bytes(bytes).stamina()? > 0 && (!group || *mask_value != 0) {
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
        WarriorRecordMut::from_bytes(bytes).set_active_flag(0);
        let initiative = WarriorRecord::from_bytes(bytes).initiative_index()?;
        if initiative != -1 {
            let initiative = usize::try_from(initiative)
                .map_err(|_| RuleError::Invalid("warrior initiative is negative".into()))?;
            *result
                .order
                .get_mut(initiative)
                .ok_or_else(|| RuleError::Invalid("warrior initiative unavailable".into()))? = -1;
        }
        WarriorRecordMut::from_bytes(bytes).set_initiative_index(-1);
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
        span(&warrior.bytes, 0, WARRIOR_BYTES)?;
        warrior.record_mut().set_active_flag(1);
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
