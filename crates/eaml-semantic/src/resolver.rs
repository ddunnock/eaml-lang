//! Two-pass name resolution for EAML semantic analysis.
//!
//! Pass 1: Register all top-level declarations in the symbol table.
//! Pass 2: Resolve all references against the symbol table.
//! Pass 3: Detect cyclic schema references.

use eaml_errors::DiagnosticCollector;
use eaml_lexer::Interner;
use eaml_parser::ast::{Ast, Program};

use crate::symbol_table::SymbolTable;

/// Performs name resolution on the AST, returning a populated symbol table.
pub fn resolve(
    program: &Program,
    ast: &Ast,
    interner: &Interner,
    _source: &str,
    _diags: &mut DiagnosticCollector,
) -> SymbolTable {
    let symbols = SymbolTable::new(interner);
    // Stub: passes 1-3 will be implemented in Task 2
    let _ = (program, ast);
    symbols
}
