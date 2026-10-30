use crate::combat::spells::RestrictionMemoryView;
use crate::combat::warrior_alias::Alias;
use crate::{Event, RandomState, RuleError, character, combat, story_variables::StoryContinuation};
use centurions_content::records::{
    ATTRIBUTES, DevelopmentField as Field, GiveConditionExtra, InventorySlots, PartyFilterExtra,
    PartySelectionExtra, PickByCasteExtra, PickByTraitExtra, StatisticAdjustmentExtra,
    VictoryPointsExtra, WarriorRecord, WarriorRecordMut,
};
use centurions_content::{WARRIOR_BYTES, Warrior, mission::OpeningMission, span};

/// The party's saving throw context: whether a spell is being cast, the packet's type and bonuses,
/// the attack count, and whether a mind shield protects.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PartySaveContext {
    pub casting: bool,
    pub packet_type: i8,
    pub bonus: i8,
    pub bonus_per_attack: i8,
    pub attack_count: i16,
    pub mind_shield: bool,
}

/// Adjust Statistics changes the chosen statistics of the selected members.
///
/// Original: newland
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

/// Filter Party narrows the existing picks.
///
/// Original: newland
pub fn filter_party(
    warriors: &[Warrior],
    mut mask: [i16; 6],
    action: usize,
    extra: &[u8],
    random: &mut RandomState,
    memory: RestrictionMemoryView<'_>,
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
            let skill = usize::try_from(selector)
                .map_err(|_| RuleError::Invalid("negative skill selector".into()))?;
            // The skill is read from warrior record `action`, the script's action
            // index, not from the member being tested: an original quirk (see
            // docs/memory-alias-audit.md).
            let (value, bonus) = if let Some(source) = warriors.get(action) {
                (
                    source.record().spec(skill)?,
                    source.record().special_skill_bonus(skill)?,
                )
            } else {
                (
                    memory.alias_word(action, 0xf4 + skill * 2, skill_alias_missing)?,
                    memory.alias_word(action, 0xb62 + skill * 2, skill_alias_missing)?,
                )
            };
            let chance = value.wrapping_add(bonus).wrapping_add(extra.test_word()?);
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

/// Select Party By Save clears the picks before testing each member.
pub fn select_party(
    warriors: &[Warrior],
    char_select_new: usize,
    extra: &[u8],
    context: PartySaveContext,
    random: &mut RandomState,
) -> Result<[i16; 6], RuleError> {
    select_party_with_ready(
        warriors,
        char_select_new,
        extra,
        context,
        i16::from(context.casting),
        random,
    )
}

/// The saving throw reads the live `in_spell`. Direct callers supply it through
/// `PartySaveContext::casting`.
///
/// Original: newland, savevs
pub(crate) fn select_party_with_ready(
    warriors: &[Warrior],
    char_select_new: usize,
    extra: &[u8],
    context: PartySaveContext,
    in_spell: i16,
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
            0 => warrior.record().movement_max()? < value,
            1 => (member as i16).wrapping_add(1) < value,
            2 => holds_item()?,
            3 => next_random.bound(100) <= value,
            4 => !character::save_against_attribute(warrior, value, &mut next_random)?,
            5 => {
                let skip_save = context.packet_type >= 2 && in_spell != 0;
                if skip_save {
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
                let slot = mask.get_mut(char_select_new).ok_or_else(|| {
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
                            && InventorySlots(bytes).equipped(slot) != 0
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

/// A group battle a script starts: the battle, the sound and message that open it, and the script
/// that runs on retreat.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupEncounter {
    pub battle: i16,
    pub sound: i16,
    pub message: i16,
    pub failure_script: i16,
}

/// Group Battle picks its battle like Battle; its fifth word is a retreat script.
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

/// The warriors, battlefield, squad placement and order of a group battle after members are left
/// out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupSetup {
    pub warriors: Vec<Warrior>,
    pub map: combat::CombatField,
    pub squad: combat::SquadPlacement,
    pub order: Vec<i16>,
}

/// Removes members left out of the battle, after formation and initiative.
///
/// Original: combatsetup
pub fn apply_group_mask(
    warriors: &[Warrior],
    map: &combat::CombatField,
    squad: &combat::SquadPlacement,
    order: &[i16],
    mask: &[i16; 6],
    group: bool,
) -> Result<GroupSetup, RuleError> {
    if !(1..=6).contains(&warriors.len())
        || squad.pos.len() != warriors.len()
        || squad.char_under.len() != warriors.len()
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
        WarriorRecordMut::from_bytes(bytes).set_in_battle(1);
        if WarriorRecord::from_bytes(bytes).stamina()? > 0 && (!group || *mask_value != 0) {
            continue;
        }
        let position = result.squad.pos[member];
        let index = OpeningMission::index(
            i32::from(position.0) + i32::from(map.map_shift.0) + 5,
            i32::from(position.1) + i32::from(map.map_shift.1) + 5,
        )?;
        let tile = result
            .map
            .tiles
            .get_mut(index)
            .ok_or_else(|| RuleError::Invalid("group formation tile unavailable".into()))?;
        *tile = result.squad.char_under[member];
        WarriorRecordMut::from_bytes(bytes).set_in_battle(0);
        let initiative = WarriorRecord::from_bytes(bytes).position()?;
        if initiative != -1 {
            let initiative = usize::try_from(initiative)
                .map_err(|_| RuleError::Invalid("warrior initiative is negative".into()))?;
            *result
                .order
                .get_mut(initiative)
                .ok_or_else(|| RuleError::Invalid("warrior initiative unavailable".into()))? = -1;
        }
        WarriorRecordMut::from_bytes(bytes).set_position(-1);
        result.squad.pos[member] = (-1, -1);
    }
    Ok(result)
}

/// The warriors after a group battle and how the script goes on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupReturn {
    pub warriors: Vec<Warrior>,
    pub continuation: StoryContinuation,
}

/// After a group battle every member takes part again; only a retreat replaces
/// the script.
pub fn group_return(
    warriors: &[Warrior],
    retreated: bool,
    failure_script: i16,
) -> Result<GroupReturn, RuleError> {
    let mut warriors = warriors.to_vec();
    for warrior in &mut warriors {
        span(&warrior.bytes, 0, WARRIOR_BYTES)?;
        warrior.record_mut().set_in_battle(1);
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

/// The refusal given when a party filter needs a skill alias the reconstruction cannot read.
fn skill_alias_missing(alias: Alias) -> RuleError {
    if alias == Alias::AboveA5 {
        RuleError::Unsupported {
            action: "story.filter_party",
            dependency: "native action-index skill alias above the A5 world",
        }
    } else {
        RuleError::Unsupported {
            action: "story.filter_party",
            dependency: "native action-index skill alias outside the represented owners",
        }
    }
}

/// Give Condition clears condition e1 on every member that has it, then adds
/// duration e3 to it on the members chosen by e2: everyone (0), picked members
/// (1) or those above -10 stamina (2).
pub(crate) fn give_condition(
    warriors: &mut [Warrior],
    party_track: &[i16; 6],
    extra: GiveConditionExtra<'_>,
    effects: &mut Vec<Event>,
) -> Result<(), RuleError> {
    let slot = usize::try_from(extra.condition()?)
        .ok()
        .filter(|&slot| slot < 40)
        .ok_or(RuleError::Unsupported {
            action: "story.opcode",
            dependency: "condition outside the record",
        })?;
    let (who, amount, sound) = (extra.recipients()?, extra.duration()?, extra.sound()?);
    for member in 0..warriors.len() {
        let warrior = &mut warriors[member];
        let current = WarriorRecord::from_bytes(&warrior.bytes).condition_slot(slot)?;
        if current > 0 {
            warrior.record_mut().set_condition_duration(slot, 0);
        }
        let chosen = match who {
            0 => true,
            1 => party_track[member] != 0,
            2 => warrior.record().stamina()? > -10,
            _ => false,
        };
        if chosen {
            let current = WarriorRecord::from_bytes(&warrior.bytes).condition_slot(slot)?;
            warrior
                .record_mut()
                .set_condition_duration(slot, current.wrapping_add(amount));
            if sound != 0 {
                effects.push(Event::Sound { resource_id: sound });
            }
            effects.push(Event::Feedback(
                crate::feedback::GameFeedback::CombatResult {
                    body: member as i16,
                    code: slot as i16 + 1,
                    amount: 0,
                    audible: false,
                },
            ));
        }
    }
    Ok(())
}

/// Pick By Trait picks members (living ones when e5 is set) by race e3 (0),
/// gender e2 (1), caste e3 (2), race class e3 (3) or caste class e3 (4).
pub(crate) fn pick_by_trait(
    warriors: &[Warrior],
    extra: PickByTraitExtra<'_>,
    race_class: impl Fn(i16, i16) -> Result<bool, RuleError>,
    caste_class: impl Fn(i16) -> Result<i16, RuleError>,
) -> Result<[i16; 6], RuleError> {
    let (mode, gender, code) = (extra.mode()?, extra.gender()?, extra.code()?);
    let mut mask = [0; 6];
    for (member, warrior) in warriors.iter().enumerate() {
        let record = warrior.record();
        if extra.living_only()? && record.stamina()? <= 0 {
            continue;
        }
        let picked = match mode {
            0 => record.race()? == code,
            1 => record.gender()? == gender,
            2 => record.profession()? == code,
            3 => race_class(record.race()?, code)?,
            4 => caste_class(record.profession()?)? == code,
            _ => false,
        };
        mask[member] = i16::from(picked);
    }
    Ok(mask)
}

/// Pick By Caste picks caste e1 and the caste families of e2. The picks are
/// cleared first, so its picked-only test (e3 = 2) admits nobody.
pub(crate) fn pick_by_caste(
    warriors: &[Warrior],
    extra: PickByCasteExtra<'_>,
) -> Result<[i16; 6], RuleError> {
    let (caste, family, check) = (extra.caste()?, extra.family()?, extra.admission()?);
    let families: &[i16] = match family {
        1 => &[1, 3, 4],
        2 => &[3, 6, 7, 8],
        3 => &[2, 5],
        _ => &[],
    };
    let mut mask = [0; 6];
    for (member, warrior) in warriors.iter().enumerate() {
        let record = warrior.record();
        let admitted = check == 0
            || (check == 1 && record.stamina()? > 0)
            || (check == 2 && mask[member] != 0);
        let profession = record.profession()?;
        if admitted && (profession == caste || families.contains(&profession)) {
            mask[member] = 1;
        }
    }
    Ok(mask)
}

/// Take Victory Points moves e1 from upgrade points into the remaining
/// experience. Its picked mode (e2 = 1) tests the pick at the action's own index
/// for every member; mode 2 spreads the signed quotient over the party.
pub(crate) fn take_victory_points(
    warriors: &mut [Warrior],
    party_track: &[i16; 6],
    cursor: usize,
    extra: VictoryPointsExtra<'_>,
) -> Result<(), RuleError> {
    let amount = extra.amount()?;
    let share = match extra.mode()? {
        1 => {
            let picked = *party_track.get(cursor).ok_or(RuleError::Unsupported {
                action: "story.opcode",
                dependency: "pick word past the party",
            })?;
            if picked == 0 {
                return Ok(());
            }
            amount
        }
        2 => amount / warriors.len() as i16,
        _ => amount,
    };
    for warrior in warriors {
        let record = warrior.record();
        let remaining = record
            .remaining_experience()?
            .wrapping_add(i32::from(share));
        let points = record.upgrade_points().wrapping_sub(i32::from(share));
        warrior.record_mut().set_remaining_experience(remaining);
        warrior.record_mut().set_upgrade_points(points);
    }
    Ok(())
}
