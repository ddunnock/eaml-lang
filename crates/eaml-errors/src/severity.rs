//! Severity levels for EAML compiler diagnostics.

/// The severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Unrecoverable internal error (maps to codespan-reporting Bug).
    Fatal,
    /// Standard compilation error.
    Error,
    /// Non-fatal warning.
    Warning,
}
