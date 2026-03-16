//! EAML code generator -- emits Python code from an analyzed AST.
//!
//! Public API: [`generate()`] function producing Python source.

pub mod names;
pub mod types;
pub mod writer;

/// Generates Python source code from a parsed and analyzed EAML program.
///
/// This is the main entry point for code generation. It takes the parse
/// output (AST + interner), the analysis output (symbol table + type
/// annotations), the original source text, and the filename.
///
/// Returns the complete Python module as a string.
pub fn generate(
    _parse_output: &eaml_parser::ParseOutput,
    _analysis: &eaml_semantic::AnalysisOutput,
    _source: &str,
    _filename: &str,
) -> String {
    // Placeholder -- will be wired up in plan 04-04
    String::new()
}
