use crate::{Command, Event, RuleError, Session};
use centurions_content::Content;

pub(super) fn execute(
    command: Command,
    _state: &mut Session,
    _content: &Content,
) -> Result<Vec<Event>, RuleError> {
    match command {
        Command::WithPreferences { .. } => unreachable!("preferences unwrap at command boundary"),
        Command::CombatClick { .. } => {
            unreachable!("combat clicks resolve at the shared command boundary")
        }
        _ => unreachable!("command routed to wrong subsystem"),
    }
}
