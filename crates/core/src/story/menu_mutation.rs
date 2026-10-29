//! newland7's menu and typed-encounter file updates (14320c..1433f8).
use crate::{RuleError, story::MENU_RECORD_BYTES, typed_encounter};
use centurions_content::{
    records::{EventReplacementExtra, StoryRecord},
    span,
};

const MENU_RECORD_HEADER: usize = 106;

pub fn replace_encounter_branch(
    table: &[u8],
    extra: &[u8],
    replacement: &[u8],
) -> Result<Vec<u8>, RuleError> {
    let extra = EventReplacementExtra(span(extra, 0, EventReplacementExtra::BYTES)?);
    let (stride, header) = match extra.map()? {
        -1 => (MENU_RECORD_BYTES, MENU_RECORD_HEADER),
        -2 => (typed_encounter::RECORD_BYTES, 156),
        _ => {
            return Err(RuleError::Invalid(
                "encounter replacement table unavailable".into(),
            ));
        }
    };
    // Opcodes go to work + 8 * branch and parameters to work + 32 + 16 * branch, in the
    // header work 0x443396 (typed, 156 bytes) or 0x443432 (menu, 106 bytes); only the
    // header is written back. Typed branches 4..6 stay inside it.
    let branch = extra.branch()?;
    if branch < 0 {
        return Err(RuleError::Unsupported {
            action: "story.replace_encounter_branch",
            dependency: "branch writes before the encounter header work",
        });
    }
    if 48 + 16 * branch as usize > header {
        return Err(if header == MENU_RECORD_HEADER {
            RuleError::Unsupported {
                action: "story.replace_encounter_branch",
                dependency: "branch writes past the menu header work 0x443432",
            }
        } else {
            RuleError::Unsupported {
                action: "story.replace_encounter_branch",
                dependency: "branch writes past the typed header into the menu header work",
            }
        });
    }
    let index = usize::from(extra.slot()? as u16);
    let mut record = span(table, index * stride, header)?.to_vec();
    let script = StoryRecord(span(replacement, 0, 42)?);
    for action in 0..StoryRecord::ACTIONS {
        let (opcode, parameter) = script.instruction(action)?;
        record[branch as usize * 8 + action] = opcode as u8;
        let offset = 32 + branch as usize * 16 + action * 2;
        record[offset..offset + 2].copy_from_slice(&parameter.to_be_bytes());
    }
    // The typed caller reads at 516-byte strides but seeks with 426 for its write.
    let write = index * MENU_RECORD_BYTES;
    span(table, write, header)?;
    let mut result = table.to_vec();
    result[write..write + header].copy_from_slice(&record);
    Ok(result)
}
