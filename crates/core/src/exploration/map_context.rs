//! Map context for maintained exploration.
use crate::{RuleError, Session};
use centurions_content::Content;
use centurions_content::mission::OpeningMission;

pub(super) fn unsupported(action: &'static str, dependency: &'static str) -> RuleError {
    RuleError::Unsupported { action, dependency }
}
pub(crate) fn active_map<'a>(
    state: &Session,
    content: &'a Content,
) -> Result<&'a OpeningMission, RuleError> {
    state
        .land(content)
        .ok_or_else(|| RuleError::Invalid("active map unavailable".into()))
}
