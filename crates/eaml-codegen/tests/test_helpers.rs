//! Shared test helpers for eaml-codegen integration tests.
//!
//! Provides parse_and_analyze() and generate_from_source() for
//! convenient end-to-end testing.

use eaml_codegen::generate;
use eaml_parser::parse;
use eaml_semantic::analyze;

/// Parses and analyzes EAML source, asserting no fatal errors.
#[allow(dead_code)]
pub fn parse_and_analyze(
    source: &str,
) -> (eaml_parser::ParseOutput, eaml_semantic::AnalysisOutput) {
    let parse_output = parse(source);
    assert!(
        parse_output
            .diagnostics
            .iter()
            .all(|d| d.severity != eaml_errors::Severity::Error
                && d.severity != eaml_errors::Severity::Fatal),
        "parse errors: {:?}",
        parse_output.diagnostics
    );
    let analysis = analyze(&parse_output, source);
    assert!(
        !analysis.has_fatal,
        "analysis errors: {:?}",
        analysis.diagnostics
    );
    (parse_output, analysis)
}

/// Parses, analyzes, and generates Python from EAML source.
#[allow(dead_code)]
pub fn generate_from_source(source: &str) -> String {
    let (parse_output, analysis) = parse_and_analyze(source);
    generate(&parse_output, &analysis, source, "test.eaml")
}

/// Parses, analyzes, and generates Python from EAML source with a custom filename.
#[allow(dead_code)]
pub fn generate_from_source_with_name(source: &str, filename: &str) -> String {
    let (parse_output, analysis) = parse_and_analyze(source);
    generate(&parse_output, &analysis, source, filename)
}
