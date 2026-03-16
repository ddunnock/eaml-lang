//! Recursive descent parser for the EAML language.
//!
//! Provides the [`Parser`] struct with cursor methods, diagnostic emission,
//! and error recovery (synchronization).

use eaml_errors::{Diagnostic, ErrorCode, Severity, Span};
use eaml_lexer::{Interner, Token, TokenKind};
use lasso::Spur;

use crate::ast::*;

/// The EAML recursive descent parser.
///
/// Consumes a token stream and produces an AST with typed arena allocation.
pub struct Parser {
    tokens: Vec<Token>,
    pub(crate) pos: usize,
    source: String,
    pub(crate) ast: Ast,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) interner: Interner,
}

impl Parser {
    /// Creates a new parser from lexer output.
    pub fn new(
        source: String,
        tokens: Vec<Token>,
        interner: Interner,
        lex_diagnostics: Vec<Diagnostic>,
    ) -> Self {
        Self {
            tokens,
            pos: 0,
            source,
            ast: Ast::new(),
            diagnostics: lex_diagnostics,
            interner,
        }
    }

    // ========================================================================
    // Cursor methods
    // ========================================================================

    /// Returns the current token kind without advancing.
    pub fn peek(&self) -> TokenKind {
        if self.pos < self.tokens.len() {
            self.tokens[self.pos].kind
        } else {
            TokenKind::Eof
        }
    }

    /// Returns the span of the current token.
    pub fn peek_span(&self) -> Span {
        if self.pos < self.tokens.len() {
            self.tokens[self.pos].span.clone()
        } else {
            let end = self.source.len();
            end..end
        }
    }

    /// Lookahead by `offset` tokens from the current position.
    pub fn peek_at(&self, offset: usize) -> TokenKind {
        let idx = self.pos + offset;
        if idx < self.tokens.len() {
            self.tokens[idx].kind
        } else {
            TokenKind::Eof
        }
    }

    /// Returns the current token and advances the position.
    pub fn advance(&mut self) -> &Token {
        let idx = self.pos;
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        &self.tokens[idx.min(self.tokens.len() - 1)]
    }

    /// Returns the span of the previously consumed token.
    pub fn previous_span(&self) -> Span {
        let idx = self.pos.saturating_sub(1);
        if idx < self.tokens.len() {
            self.tokens[idx].span.clone()
        } else {
            let end = self.source.len();
            end..end
        }
    }

    /// Returns `true` if the current token matches the given kind.
    ///
    /// Uses `std::mem::discriminant` to compare, so `Ident(spur)` payloads
    /// are ignored -- any Ident matches any other Ident.
    pub fn at(&self, kind: TokenKind) -> bool {
        std::mem::discriminant(&self.peek()) == std::mem::discriminant(&kind)
    }

    /// Returns `true` if the current token is any `Ident`.
    pub fn at_ident(&self) -> bool {
        matches!(self.peek(), TokenKind::Ident(_))
    }

    /// Consumes the current token if it matches `kind`, returning `true`.
    /// Otherwise returns `false` without advancing.
    pub fn eat(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Expects the current token to match `kind`. Advances and returns the
    /// span on success, or emits a SYN050 diagnostic and returns `Err(())`.
    #[allow(clippy::result_unit_err)]
    pub fn expect(&mut self, kind: TokenKind) -> Result<Span, ()> {
        if self.at(kind) {
            let span = self.peek_span();
            self.advance();
            Ok(span)
        } else {
            let actual = self.peek();
            let span = self.peek_span();
            self.emit_error(
                ErrorCode::Syn050,
                format!("expected {kind:?}, found {actual:?}"),
                span.clone(),
                format!("expected {kind:?}"),
            );
            Err(())
        }
    }

    /// Expects the current token to be an `Ident`. Returns the interned
    /// `Spur` and span on success, or emits SYN050 and returns `Err(())`.
    #[allow(clippy::result_unit_err)]
    pub fn expect_ident(&mut self) -> Result<(Spur, Span), ()> {
        if let TokenKind::Ident(spur) = self.peek() {
            let span = self.peek_span();
            self.advance();
            Ok((spur, span))
        } else {
            let actual = self.peek();
            let span = self.peek_span();
            self.emit_error(
                ErrorCode::Syn050,
                format!("expected identifier, found {actual:?}"),
                span.clone(),
                "expected identifier".to_string(),
            );
            Err(())
        }
    }

    /// Returns `true` if the current token is an `Ident` whose resolved
    /// string matches `name` (contextual keyword check).
    pub fn at_contextual(&self, name: &str) -> bool {
        if let TokenKind::Ident(spur) = self.peek() {
            self.interner.resolve(&spur) == name
        } else {
            false
        }
    }

    /// Expects the current token to be a contextual keyword matching `name`.
    /// Advances and returns the span on success, or emits SYN050 and `Err(())`.
    #[allow(clippy::result_unit_err)]
    pub fn expect_contextual(&mut self, name: &str) -> Result<Span, ()> {
        if self.at_contextual(name) {
            let span = self.peek_span();
            self.advance();
            Ok(span)
        } else {
            let actual = self.peek();
            let span = self.peek_span();
            self.emit_error(
                ErrorCode::Syn050,
                format!("expected '{name}', found {actual:?}"),
                span.clone(),
                format!("expected '{name}'"),
            );
            Err(())
        }
    }

    // ========================================================================
    // Diagnostic emission
    // ========================================================================

    /// Emits an error diagnostic.
    pub fn emit_error(&mut self, code: ErrorCode, message: String, span: Span, label: String) {
        self.diagnostics
            .push(Diagnostic::new(code, message, span, Severity::Error, label));
    }

    /// Emits an error diagnostic with a hint message.
    pub fn emit_error_with_hint(
        &mut self,
        code: ErrorCode,
        message: String,
        span: Span,
        label: String,
        hint: String,
    ) {
        self.diagnostics
            .push(Diagnostic::new(code, message, span, Severity::Error, label).with_hint(hint));
    }

    // ========================================================================
    // Error recovery
    // ========================================================================

    /// Skips tokens until reaching a declaration keyword at brace depth 0
    /// or a closing brace that brings depth to 0.
    pub fn synchronize(&mut self) {
        let mut brace_depth: i32 = 0;

        loop {
            match self.peek() {
                TokenKind::Eof => break,
                TokenKind::LBrace => {
                    brace_depth += 1;
                    self.advance();
                }
                TokenKind::RBrace => {
                    brace_depth -= 1;
                    self.advance();
                    if brace_depth <= 0 {
                        break;
                    }
                }
                TokenKind::KwModel
                | TokenKind::KwSchema
                | TokenKind::KwPrompt
                | TokenKind::KwTool
                | TokenKind::KwAgent
                | TokenKind::KwImport
                | TokenKind::KwLet
                | TokenKind::KwPipeline
                | TokenKind::KwEnum
                    if brace_depth == 0 =>
                {
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    /// Returns `true` if the error limit (20 errors) has been reached.
    pub fn error_limit_reached(&self) -> bool {
        let error_count = self
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error || d.severity == Severity::Fatal)
            .count();
        error_count >= 20
    }

    // ========================================================================
    // Source text access
    // ========================================================================

    /// Returns the source text covered by a span.
    pub fn span_text(&self, span: &Span) -> &str {
        &self.source[span.clone()]
    }

    /// Resolves a `Spur` to the original interned string.
    pub fn resolve_spur(&self, spur: &Spur) -> &str {
        self.interner.resolve(spur)
    }

    // ========================================================================
    // Testing support
    // ========================================================================

    /// Consumes the parser and returns the AST and diagnostics.
    /// Used by tests that call individual parse methods directly.
    pub fn finish(self) -> (Ast, Vec<Diagnostic>) {
        (self.ast, self.diagnostics)
    }

    /// Consumes the parser and returns the AST, diagnostics, and interner.
    /// Used by tests that need to resolve Spur values after parsing.
    pub fn finish_with_interner(self) -> (Ast, Vec<Diagnostic>, Interner) {
        (self.ast, self.diagnostics, self.interner)
    }

    /// Returns a reference to the interner.
    pub fn interner(&self) -> &Interner {
        &self.interner
    }

    /// Returns a reference to the source string.
    pub fn source(&self) -> &str {
        &self.source
    }
}
