//! Shared test helpers for eaml-semantic tests.

use eaml_errors::{ErrorCode, Severity};
use eaml_parser::ParseOutput;
use eaml_semantic::AnalysisOutput;

/// Parse and analyze a source string, returning both outputs.
pub fn analyze_source(source: &str) -> (ParseOutput, AnalysisOutput) {
    let parse_output = eaml_parser::parse(source);
    let analysis = eaml_semantic::analyze(&parse_output, source);
    (parse_output, analysis)
}

/// Assert that the analysis output contains a diagnostic with the given code.
pub fn assert_has_code(output: &AnalysisOutput, code: ErrorCode) {
    assert!(
        output.diagnostics.iter().any(|d| d.code == code),
        "Expected diagnostic code {} but got: {:?}",
        code,
        output
            .diagnostics
            .iter()
            .map(|d| d.code.to_string())
            .collect::<Vec<_>>()
    );
}

/// Assert that the analysis produced no error or fatal diagnostics.
pub fn assert_no_errors(output: &AnalysisOutput) {
    let errors: Vec<_> = output
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error || d.severity == Severity::Fatal)
        .collect();
    assert!(
        errors.is_empty(),
        "Expected no errors but got: {:?}",
        errors
            .iter()
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect::<Vec<_>>()
    );
}

/// Count error-severity diagnostics.
pub fn error_count(output: &AnalysisOutput) -> usize {
    output
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error || d.severity == Severity::Fatal)
        .count()
}

/// Check if a diagnostic with the given code has a secondary label containing the text.
pub fn has_secondary_label_containing(
    output: &AnalysisOutput,
    code: ErrorCode,
    text: &str,
) -> bool {
    output
        .diagnostics
        .iter()
        .filter(|d| d.code == code)
        .any(|d| {
            d.secondary_labels
                .iter()
                .any(|(_, label)| label.contains(text))
        })
}
