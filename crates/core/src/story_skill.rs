use crate::RuleError;
use centurions_content::{Warrior, span, word};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillPlan {
    Continue,
    ReplaceScript {
        warrior: usize,
        success: bool,
        script: i16,
    },
}

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

pub fn predicate(
    extra: &[u8],
    warriors: &[Warrior],
    selected: &[i16; 6],
) -> Result<SkillPlan, RuleError> {
    let extra = span(extra, 0, 10)?;
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
    let index = word(extra, 0)?;
    let threshold = word(extra, 2)?;
    let success = if word(extra, 4)? == 0 {
        let index = usize::try_from(index)
            .map_err(|_| RuleError::Invalid("negative story skill index".into()))?;
        word(bytes, 0xf4 + index * 2)?.wrapping_add(word(bytes, 0xb62 + index * 2)?) >= threshold
    } else {
        let offset = match index {
            0 => 0x9e2,
            1 => 0x9e4,
            2 => 0x9e6,
            3 => 0x9e8,
            4 => 0x9ec,
            5 => 0x9ee,
            _ => return Ok(SkillPlan::Continue),
        };
        word(bytes, offset)? > threshold
    };
    Ok(SkillPlan::ReplaceScript {
        warrior,
        success,
        script: word(extra, if success { 6 } else { 8 })?,
    })
}
