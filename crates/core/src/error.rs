use crate::feedback;
use centurions_content::ContentError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuleError {
    #[error("action refused by the original rules")]
    Refused {
        feedback: Vec<feedback::NativeFeedback>,
    },
    /// A native refusal reached after deliberate rule-state effects.
    #[error("action refused by the original rules")]
    RefusedAfterEffects {
        feedback: Vec<feedback::NativeFeedback>,
    },
    #[error("unsupported action {action}: {dependency}")]
    Unsupported {
        action: &'static str,
        dependency: &'static str,
    },
    #[error("invalid session: {0}")]
    Invalid(String),
    #[error("{0}")]
    Content(#[from] ContentError),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
}

impl From<String> for RuleError {
    fn from(message: String) -> Self {
        Self::Invalid(message)
    }
}
impl From<&str> for RuleError {
    fn from(message: &str) -> Self {
        Self::Invalid(message.into())
    }
}
