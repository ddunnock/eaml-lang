//! EAML semantic analysis -- name resolution, type checking, and capability validation.
//!
//! Public API: [`analyze()`] function returning [`AnalysisOutput`].

pub mod cap_checker;
pub mod resolver;
pub mod scope;
pub mod symbol_table;
pub mod type_checker;

use eaml_errors::{Diagnostic, DiagnosticCollector, Severity};
use eaml_parser::ParseOutput;

use symbol_table::SymbolTable;
use type_checker::TypeAnnotations;

/// The output of semantic analysis.
pub struct AnalysisOutput {
    /// The populated symbol table.
    pub symbols: SymbolTable,
    /// Type annotations from the type checking pass.
    pub type_annotations: TypeAnnotations,
    /// All diagnostics emitted during analysis.
    pub diagnostics: Vec<Diagnostic>,
    /// Whether any fatal-severity diagnostic was emitted.
    pub has_fatal: bool,
}

/// Runs semantic analysis on a parsed EAML program.
pub fn analyze(parse_output: &ParseOutput, source: &str) -> AnalysisOutput {
    let mut diags = DiagnosticCollector::new(20);
    let symbols = resolver::resolve(
        &parse_output.program,
        &parse_output.ast,
        &parse_output.interner,
        &mut diags,
    );
    let type_annotations = type_checker::check(
        &parse_output.program,
        &parse_output.ast,
        &parse_output.interner,
        &symbols,
        source,
        &mut diags,
    );
    cap_checker::check(
        &parse_output.program,
        &parse_output.ast,
        &parse_output.interner,
        &type_annotations,
        &mut diags,
    );
    let diagnostics = diags.into_diagnostics();
    let has_fatal = diagnostics.iter().any(|d| d.severity == Severity::Fatal);
    AnalysisOutput {
        symbols,
        type_annotations,
        diagnostics,
        has_fatal,
    }
}
