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
    /// Runtime detail attached to a diagnostic refusal by its owning interpreter.
    #[error("{source} [{context}]")]
    Context {
        context: String,
        source: Box<RuleError>,
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

impl RuleError {
    /// Attaches runtime detail to an unsupported-rule diagnostic. Native
    /// refusals and other errors pass through unchanged so callers that
    /// distinguish them keep matching the same variant.
    pub fn with_context(self, context: impl FnOnce() -> String) -> Self {
        match self {
            Self::Unsupported { .. } | Self::Context { .. } => Self::Context {
                context: context(),
                source: Box::new(self),
            },
            error => error,
        }
    }

    /// The diagnostic beneath any attached context.
    pub fn root(&self) -> &Self {
        match self {
            Self::Context { source, .. } => source.root(),
            error => error,
        }
    }
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
