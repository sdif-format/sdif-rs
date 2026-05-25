//! Error types for the SDIF parser and policy enforcement.

use crate::span::Span;

/// A parse error with an SDIF error code, human-readable message, source
/// location, and an optional remediation hint.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    /// Machine-readable error code, e.g. `"SDIF_STRING_UNCLOSED"`.
    pub code: String,
    pub message: String,
    pub span: Span,
    /// Optional hint shown to the user to help resolve the error.
    pub hint: Option<String>,
}

impl ParseError {
    pub fn new(code: impl Into<String>, message: impl Into<String>, span: Span) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            span,
            hint: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Create a `ParseError` from a policy violation. Uses a sentinel span
    /// (line 1, col 1) since `PolicyError` has no source location.
    pub fn from_policy(e: PolicyError) -> Self {
        Self::new(e.code, e.message, Span::single_line(1, 1, 1))
    }

    /// Create a `ParseError` for a policy violation at a known line.
    pub fn policy_at(code: impl Into<String>, message: impl Into<String>, line: u32) -> Self {
        Self::new(code, message, Span::single_line(line, 1, 1))
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {}:{}: {}",
            self.code, self.span.start_line, self.span.start_col, self.message
        )
    }
}

impl std::error::Error for ParseError {}

// ---------------------------------------------------------------------------

/// An error raised when a security or resource policy is violated.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyError {
    /// Machine-readable policy error code, e.g. `"POLICY_SIZE_EXCEEDED"`.
    pub code: String,
    pub message: String,
}

impl PolicyError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for PolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for PolicyError {}
