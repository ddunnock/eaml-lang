//! Diagnostic types for the EAML compiler.
//!
//! A [`Diagnostic`] represents a single compiler message with an error code,
//! source span, severity, and optional hints.

use crate::codes::ErrorCode;
use crate::severity::Severity;

/// A byte-offset range in source text.
pub type Span = std::ops::Range<usize>;

/// A single compiler diagnostic message.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// The error code identifying this diagnostic.
    pub code: ErrorCode,
    /// Human-readable error message.
    pub message: String,
    /// Byte-offset span in the source text.
    pub span: Span,
    /// Severity level.
    pub severity: Severity,
    /// Label text displayed at the span location.
    pub label: String,
    /// Optional hint messages displayed after the diagnostic.
    pub hints: Vec<String>,
}

impl Diagnostic {
    /// Creates a new diagnostic with no hints.
    pub fn new(
        code: ErrorCode,
        message: String,
        span: Span,
        severity: Severity,
        label: String,
    ) -> Self {
        Self {
            code,
            message,
            span,
            severity,
            label,
            hints: Vec::new(),
        }
    }

    /// Adds a hint message to this diagnostic (builder pattern).
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hints.push(hint.into());
        self
    }
}
