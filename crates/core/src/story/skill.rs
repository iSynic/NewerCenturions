use crate::RuleError;
use centurions_content::{
    Warrior,
    records::{Attribute, DevelopmentField as Field, SkillCheckExtra},
};

/// What a skill check does to the script: carry on, or replace it with the script for success or
/// failure for the warrior tested.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillPlan {
    Continue,
    ReplaceScript {
        warrior: usize,
        success: bool,
        script: i16,
    },
}

/// The party selection that picks one warrior, who must have stamina.
pub fn single_warrior(warriors: &[Warrior], index: usize) -> Result<[i16; 6], RuleError> {
    if warriors.len() > 6 || index >= 6 {
        return Err(RuleError::Invalid(
            "warrior choice is outside the squad".into(),
        ));
    }
    let warrior = warriors
        .get(index)
        .ok_or_else(|| RuleError::Invalid("selected warrior is unavailable".into()))?;
    if warrior.record().stamina()? < 1 {
        return Err(RuleError::Invalid("selected warrior has no stamina".into()));
    }
    let mut selected = [0; 6];
    selected[index] = 1;
    Ok(selected)
}

/// Tests the first selected warrior against the extra: a special skill plus its bonus must reach
/// the threshold, or an attribute must exceed it. An unknown attribute continues the script;
/// otherwise the success or failure script replaces it.
pub fn predicate(
    extra: &[u8],
    warriors: &[Warrior],
    selected: &[i16; 6],
) -> Result<SkillPlan, RuleError> {
    let extra = SkillCheckExtra::new(extra)?;
    let warrior = selected
        .iter()
        .position(|value| *value != 0)
        .ok_or(RuleError::Unsupported {
            action: "story.skill_predicate",
            dependency: "original empty warrior selection reads beyond the six warrior slots",
        })?;
    let bytes = &warriors
        .get(warrior)
        .ok_or_else(|| RuleError::Invalid("selected warrior is unavailable".into()))?
        .bytes;
    let index = extra.subject()?;
    let threshold = extra.threshold()?;
    let success = if extra.checks_special_skill()? {
        let index = usize::try_from(index)
            .map_err(|_| RuleError::Invalid("negative story skill index".into()))?;
        Field::SpecialSkill(index)
            .read(bytes)?
            .wrapping_add(Field::SpecialSkillBonus(index).read(bytes)?)
            >= threshold
    } else {
        let attribute = match index {
            0 => Attribute::Brawn,
            1 => Attribute::Knowledge,
            2 => Attribute::Judgement,
            3 => Attribute::Quickness,
            4 => Attribute::Vitality,
            5 => Attribute::Psionics,
            _ => return Ok(SkillPlan::Continue),
        };
        Field::Attribute(attribute).read(bytes)? > threshold
    };
    Ok(SkillPlan::ReplaceScript {
        warrior,
        success,
        script: extra.script(success)?,
    })
}
