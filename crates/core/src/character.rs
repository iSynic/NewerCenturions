use crate::{RandomState, RuleError};
use centurions_content::{
    WARRIOR_BYTES, Warrior,
    records::{
        Attribute, CharacterInformation, DevelopmentField as Field, MosProfile, WarriorRecordMut,
    },
    span,
};

/// The price of a warrior's next upgrade. Each repeat purchase multiplies the
/// price by the MOS's percentage, rounding to single precision each time.
///
/// Original: upgradewarrior
pub fn upgrade_cost(
    warrior: &Warrior,
    selector: u16,
    profile: &[u8],
) -> Result<(i32, i32), RuleError> {
    let (cost, percent) = upgrade_price(warrior, selector, profile)?;
    Ok((cost.trunc() as i32, percent))
}

/// Whether the warrior can pay for an upgrade, and the points left if so. The
/// points are compared in single precision and the remainder truncated.
///
/// Original: upgradewarrior
pub fn upgrade_payment(
    warrior: &Warrior,
    selector: u16,
    profile: &[u8],
) -> Result<Option<i32>, RuleError> {
    let (cost, _) = upgrade_cost(warrior, selector, profile)?;
    let cost = cost as f32;
    let points = CharacterInformation(&warrior.bytes).upgrade_points()?;
    let rounded = points as f32;
    Ok((rounded >= cost).then(|| (f64::from(points) - f64::from(cost)).trunc() as i32))
}

/// The price of the next purchase of one upgrade: its base price multiplied by 100 plus the
/// MOS's percentage, over 100, once for every purchase already made, and held to 50,000,000
/// at most. Returns the price and the MOS's percentage.
///
/// Original: upgradewarrior
fn upgrade_price(
    warrior: &Warrior,
    selector: u16,
    profile: &[u8],
) -> Result<(f32, i32), RuleError> {
    /// Base prices of the 33 upgrades, before the MOS's percentage.
    const BASE: [i16; 33] = [
        100, 100, 100, 100, 100, 100, 100, 50, 200, 200, 150, 150, 150, 150, 150, 150, 175, 125,
        200, 175, 100, 100, 300, 250, 350, 400, 100, 100, 100, 200, 250, 300, 1000,
    ];
    let base = *BASE.get(usize::from(selector)).ok_or_else(|| {
        RuleError::Invalid("upgrade selector is outside the original table".into())
    })?;
    let percent = MosProfile(profile).upgrade_percent(selector)?;
    let count = warrior.record().upgrade(selector).purchases()?;
    // Percentages below -100 give a negative factor, so the cost alternates sign.
    let factor = (f64::from((100. + f64::from(percent)) as f32) / 100.) as f32;
    let mut cost = f32::from(base);
    for _ in 0..count.max(0) {
        cost = (f64::from(cost) * f64::from(factor)) as f32;
    }
    // The cap applies only after the loop. For nonnegative percentages this
    // equals stopping at the first cost above the cap.
    if cost >= 50_000_000. {
        cost = 50_000_000.;
    }
    Ok((cost, i32::from(percent)))
}

/// Applies one upgrade. The caller has already checked the price and
/// eligibility.
///
/// Original: applyupgrade
pub fn apply_upgrade(
    warrior: &mut Warrior,
    selector: u16,
    increment: i16,
) -> Result<(), RuleError> {
    span(&warrior.bytes, 0, WARRIOR_BYTES)?;
    let bytes = &mut warrior.bytes;
    let target = Field::upgrade(selector);
    if let Some(field) = target {
        let delta = match selector {
            7 => increment.wrapping_mul(10),
            32 => 1,
            _ => increment,
        };
        field.add(bytes, delta)?;
        if selector == 0 {
            Field::CarryCapacity.add(bytes, 100)?;
        }
    }
    let stamina = Field::BaseStamina
        .read(bytes)?
        .wrapping_add(Field::StaminaBonus.read(bytes)?);
    let psi = Field::BasePsi
        .read(bytes)?
        .wrapping_add(Field::PsiBonus.read(bytes)?);
    WarriorRecordMut::from_bytes(bytes).set_stamina_max(stamina);
    WarriorRecordMut::from_bytes(bytes).set_psi_points_max(psi);
    Ok(())
}

/// The two strength adjustments for a Brawn value, in the original order.
///
/// Original: strength
pub fn strength_adjustments(attribute: i16) -> [i16; 2] {
    match attribute {
        ..=3 => [-20, 0],
        4 => [-15, -1],
        5 => [-10, 0],
        6 => [-5, 0],
        7..=15 => [0, 0],
        16 => [1, 0],
        17..=28 => [attribute - 15, (attribute - 13) / 4],
        29 => [14, 5],
        30..=33 => [attribute - 15, 4],
        34 => [19, 5],
        _ => [20, 5],
    }
}

/// The agility adjustment, looked up from the signed 16-bit average of
/// Quickness and Coordination.
///
/// Original: agility
pub fn agility_adjustment(warrior: &Warrior) -> Result<i16, RuleError> {
    let sum = Field::Attribute(Attribute::Quickness)
        .read(&warrior.bytes)?
        .wrapping_add(Field::Attribute(Attribute::Coordination).read(&warrior.bytes)?)
        .wrapping_add(1);
    let mean = sum.wrapping_add(((sum as u16) >> 15) as i16) >> 1;
    Ok(match mean {
        3..=6 => mean - 8,
        7..=14 => 0,
        15..=34 => (mean - 13) / 2,
        _ => 11,
    })
}

/// A saving throw against an attribute: the roll must be strictly below four
/// times the attribute, which can wrap.
///
/// Original: savevsattr
pub fn save_against_attribute(
    warrior: &Warrior,
    selector: i16,
    random: &mut RandomState,
) -> Result<bool, RuleError> {
    let attribute = match selector {
        0 => Attribute::Brawn,
        1 => Attribute::Knowledge,
        2 => Attribute::Judgement,
        3 => Attribute::Quickness,
        4 => Attribute::Vitality,
        5 => Attribute::Psionics,
        _ => {
            return Err(RuleError::Unsupported {
                action: "character.attribute_save",
                dependency: "original selector leaves the attribute register uninitialized",
            });
        }
    };
    let threshold = Field::Attribute(attribute)
        .read(&warrior.bytes)?
        .wrapping_mul(4);
    Ok(random.bound(100) < threshold)
}

/// Derives a warrior's statistics from the current record.
///
/// Original: calcstats
pub fn calculate_statistics(warrior: &mut Warrior) -> Result<(), RuleError> {
    span(&warrior.bytes, 0, WARRIOR_BYTES)?;
    let bytes = &mut warrior.bytes;
    let brawn = Field::Attribute(Attribute::Brawn).read(bytes)?;
    let knowledge = Field::Attribute(Attribute::Knowledge).read(bytes)?;
    let judgement = Field::Attribute(Attribute::Judgement).read(bytes)?;
    let quickness = Field::Attribute(Attribute::Quickness).read(bytes)?;
    let coordination = Field::Attribute(Attribute::Coordination).read(bytes)?;
    let vitality = Field::Attribute(Attribute::Vitality).read(bytes)?;
    let psionics = Field::Attribute(Attribute::Psionics).read(bytes)?;
    for (field, value) in [
        (
            Field::ResistanceBonus(0),
            judgement.wrapping_add(psionics) / 5,
        ),
        (Field::ResistanceBonus(1), vitality / 4),
        (Field::ResistanceBonus(2), vitality / 4),
        (Field::ResistanceBonus(3), vitality / 4),
        (Field::ResistanceBonus(4), vitality / 4),
        (
            Field::ResistanceBonus(5),
            judgement.wrapping_add(knowledge) / 6,
        ),
        (
            Field::ResistanceBonus(6),
            vitality.wrapping_add(psionics) / 6,
        ),
        (Field::ResistanceBonus(7), vitality / 4),
        (
            Field::MissileAccuracyBonus,
            coordination.wrapping_add(judgement) / 3,
        ),
        (
            Field::MeleeAccuracyBonus,
            coordination.wrapping_add(brawn) / 3,
        ),
        (
            Field::MissileDodgeBonus,
            coordination.wrapping_add(quickness) / 7,
        ),
        (
            Field::MeleeDodgeBonus,
            coordination.wrapping_add(quickness) / 5,
        ),
        (
            Field::PsiResistanceBonus,
            judgement.wrapping_add(psionics) / 7,
        ),
        (Field::StaminaBonus, vitality / 2),
        (Field::BareHandBonus, brawn / 10),
    ] {
        field.write(bytes, value);
    }
    if centurions_content::records::WarriorRecord::from_bytes(bytes).psi_school()? == 0 {
        Field::BasePsi.write(bytes, 0);
        Field::PsiBonus.write(bytes, 0);
    } else {
        Field::PsiBonus.write(
            bytes,
            (judgement.wrapping_add(knowledge) / 6).wrapping_add(psionics / 3),
        );
    }
    // Disabled skills keep the prior record value; they are not reset to zero.
    for (skill, destination, value) in [
        (
            Field::SpecialSkill(0),
            Field::SpecialSkillBonus(0),
            coordination.wrapping_add(knowledge).wrapping_add(brawn) / 6,
        ),
        (
            Field::SpecialSkill(1),
            Field::SpecialSkillBonus(1),
            coordination.wrapping_add(knowledge) / 6,
        ),
        (
            Field::SpecialSkill(2),
            Field::SpecialSkillBonus(2),
            coordination.wrapping_add(brawn) / 15,
        ),
        (
            Field::SpecialSkill(3),
            Field::SpecialSkillBonus(3),
            judgement.wrapping_add(knowledge) / 10,
        ),
        (
            Field::SpecialSkill(4),
            Field::SpecialSkillBonus(4),
            coordination.wrapping_add(brawn) / 10,
        ),
        (
            Field::SpecialSkill(5),
            Field::SpecialSkillBonus(5),
            judgement.wrapping_add(knowledge) / 12,
        ),
        (
            Field::SpecialSkill(6),
            Field::SpecialSkillBonus(6),
            judgement.wrapping_add(knowledge) / 15,
        ),
        (
            Field::SpecialSkill(7),
            Field::SpecialSkillBonus(7),
            brawn / 3,
        ),
        (
            Field::SpecialSkill(8),
            Field::SpecialSkillBonus(8),
            coordination.wrapping_add(knowledge) / 6,
        ),
    ] {
        if skill.read(bytes)? != 0 {
            destination.write(bytes, value);
        }
    }
    Ok(())
}
