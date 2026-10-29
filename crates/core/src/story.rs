use crate::RuleError;
use centurions_content::{span, word};

mod encounter_work;
pub use encounter_work::EncounterWork;
mod menu_mutation;
pub use menu_mutation::replace_encounter_branch;

pub const MENU_RECORD_BYTES: usize = 426;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuSelection {
    pub native_item: u8,
    pub selector: i8,
    pub script_body: [u8; 32],
}

/// Select an enabled displayed item, preserving the original signed branch selector.
pub fn select_menu_entry(menu: &[u8], index: usize) -> Result<MenuSelection, RuleError> {
    let menu = span(menu, 0, MENU_RECORD_BYTES)?;
    let (native_item, selector) = menu_selector(menu, index)?;
    let script_body = dispatch_menu_branch(menu, selector)?;
    Ok(MenuSelection {
        native_item,
        selector,
        script_body,
    })
}

pub fn dispatch_menu_branch(menu: &[u8], selector: i8) -> Result<[u8; 32], RuleError> {
    let menu = span(menu, 0, 96)?;
    if !(1..=4).contains(&selector) {
        return Err(RuleError::Unsupported {
            action: "story.select_menu",
            dependency: "encounter selector outside the four authored branches",
        });
    }
    let branch = usize::from(selector as u8 - 1);
    let mut script_body = [0; 32];
    for action in 0..8 {
        let opcode = i16::from(menu[branch * 8 + action] as i8);
        script_body[action * 2..action * 2 + 2].copy_from_slice(&opcode.to_be_bytes());
        let parameter = word(menu, 32 + branch * 16 + action * 2)?;
        script_body[16 + action * 2..18 + action * 2].copy_from_slice(&parameter.to_be_bytes());
    }
    Ok(script_body)
}

pub fn menu_selector(menu: &[u8], index: usize) -> Result<(u8, i8), RuleError> {
    let menu = span(menu, 0, 100)?;
    let native_slot = (0..4)
        .filter(|slot| menu[96 + *slot] != 0)
        .nth(index)
        .ok_or_else(|| RuleError::Invalid("menu entry is unavailable".into()))?;
    let selector = menu[96 + native_slot] as i8;
    Ok((native_slot as u8 + 1, selector))
}

/// The encounter caller replaces actions and parameters while retaining its event header.
pub fn replace_menu_branch(
    script_record: &[u8],
    selection: &MenuSelection,
) -> Result<Vec<u8>, RuleError> {
    let mut record = span(script_record, 0, 42)?.to_vec();
    record[8..40].copy_from_slice(&selection.script_body);
    Ok(record)
}

pub fn set_mission_flag(flags: &[u8], parameter: i16) -> Result<Vec<u8>, RuleError> {
    let index = usize::from(parameter.unsigned_abs());
    if index >= flags.len() {
        return Err(RuleError::Invalid(
            "mission flag index is unavailable".into(),
        ));
    }
    let mut result = flags.to_vec();
    result[index] = u8::from(parameter > 0);
    Ok(result)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConditionalJump {
    ReplaceScript(i16),
    MenuBranch { branch: u8, start: u8 },
    TypedBranch { branch: u8, start: u8 },
    EndMenu,
    LastAction,
}

pub fn flag_condition_jump(
    flags: &[u8],
    extra: &[u8],
) -> Result<Option<ConditionalJump>, RuleError> {
    let extra = span(extra, 0, 10)?;
    let flag_index = usize::try_from(word(extra, 0)?)
        .map_err(|_| RuleError::Invalid("mission flag index is negative".into()))?;
    let flag = *flags
        .get(flag_index)
        .ok_or_else(|| RuleError::Invalid("mission flag index is unavailable".into()))?;
    let matched = match word(extra, 2)? {
        0 => flag == 0,
        1 => flag != 0,
        2 => true,
        _ => false,
    };
    if !matched {
        return Ok(None);
    }
    condition_jump(extra)
}

pub fn condition_jump(extra: &[u8]) -> Result<Option<ConditionalJump>, RuleError> {
    Ok(match word(extra, 4)? {
        -1 => Some(ConditionalJump::LastAction),
        0 => Some(ConditionalJump::ReplaceScript(word(extra, 6)?)),
        1 | 2 => {
            let branch = word(extra, 6)?;
            let start = word(extra, 8)?;
            if !(0..4).contains(&branch) || !(0..=8).contains(&start) {
                return Err(RuleError::Invalid(
                    "conditional menu branch is unavailable".into(),
                ));
            }
            let branch = if word(extra, 4)? == 1 {
                ConditionalJump::MenuBranch {
                    branch: branch as u8,
                    start: start as u8,
                }
            } else {
                ConditionalJump::TypedBranch {
                    branch: branch as u8,
                    start: start as u8,
                }
            };
            Some(branch)
        }
        3 => Some(ConditionalJump::EndMenu),
        _ => None,
    })
}

pub fn disable_menu_entry(menu: &[u8], native_item: i16) -> Result<Vec<u8>, RuleError> {
    if !(1..=4).contains(&native_item) {
        return Err(RuleError::Invalid("native menu item is unavailable".into()));
    }
    let mut result = span(menu, 0, MENU_RECORD_BYTES)?.to_vec();
    result[96 + native_item as usize - 1] = 0;
    Ok(result)
}

pub fn add_mission_score(score: i16, parameter: i16) -> i16 {
    score.wrapping_add(parameter)
}
