//! EAML parser -- recursive descent parser producing an AST.
//!
//! Public API: [`parse()`] function, [`ParseOutput`] struct, AST types.

pub mod ast;
pub mod decl;
pub mod expr;
pub mod parser;
pub mod template;
pub mod type_expr;

use eaml_errors::Diagnostic;
use eaml_lexer::Interner;

pub use ast::*;

/// The output of parsing an EAML source string.
pub struct ParseOutput {
    /// The AST arena containing all allocated nodes.
    pub ast: Ast,
    /// The top-level program with declaration references.
    pub program: Program,
    /// All diagnostics (from both lexing and parsing).
    pub diagnostics: Vec<Diagnostic>,
    /// The string interner (shared with lexer).
    pub interner: Interner,
}

/// Parses an EAML source string into an AST.
///
/// This is the main entry point for the parser. It runs the lexer,
/// then the parser, and returns the combined output.
pub fn parse(source: &str) -> ParseOutput {
    let lex_output = eaml_lexer::lex(source);
    let parser = parser::Parser::new(
        source.to_string(),
        lex_output.tokens,
        lex_output.interner,
        lex_output.diagnostics,
    );
    parser.parse_program()
}
