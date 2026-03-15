//! Shared error types and diagnostic display for the EAML compiler.
//!
//! All compiler phases depend on this crate for error codes,
//! diagnostic structs, and formatted error output.

pub mod codes;
pub mod diagnostic;
pub mod render;
pub mod severity;

pub use codes::ErrorCode;
pub use diagnostic::{Diagnostic, DiagnosticCollector, Span};
pub use severity::Severity;
