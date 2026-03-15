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

/// Accumulates diagnostics up to a configurable error limit.
///
/// When the number of error-severity (Error or Fatal) diagnostics exceeds
/// `max_errors`, the collector stops storing new error/fatal diagnostics.
/// Warning diagnostics are always collected regardless of the error limit.
pub struct DiagnosticCollector {
    diagnostics: Vec<Diagnostic>,
    max_errors: usize,
    error_count: usize,
}

impl DiagnosticCollector {
    /// Creates a new collector with the given maximum error count.
    pub fn new(max_errors: usize) -> Self {
        Self {
            diagnostics: Vec::new(),
            max_errors,
            error_count: 0,
        }
    }

    /// Emits a diagnostic into the collector.
    ///
    /// Error and Fatal severity diagnostics count toward the error limit.
    /// Once the limit is exceeded, new error/fatal diagnostics are dropped.
    /// Warning diagnostics are always collected.
    pub fn emit(&mut self, diag: Diagnostic) {
        if diag.severity == Severity::Error || diag.severity == Severity::Fatal {
            self.error_count += 1;
            if self.error_count > self.max_errors {
                return;
            }
        }
        self.diagnostics.push(diag);
    }

    /// Returns true if any Error or Fatal diagnostics have been emitted.
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    /// Returns true if the error limit was exceeded.
    pub fn overflow(&self) -> bool {
        self.error_count > self.max_errors
    }

    /// Returns the count of Error and Fatal severity diagnostics emitted.
    pub fn error_count(&self) -> usize {
        self.error_count
    }

    /// Returns a slice of all collected diagnostics.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Consumes the collector and returns the diagnostics vector.
    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }
}
