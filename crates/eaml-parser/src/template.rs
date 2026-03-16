//! Template string parser for EAML.
//!
//! Handles grammar productions [51]-[53]: templateStr, tmplPart, tmplInterp.
//! The lexer has already decomposed template strings into token sequences:
//! TmplStart, TmplText*, (TmplInterpStart expr TmplInterpEnd)*, TmplEnd.

use eaml_errors::ErrorCode;
use eaml_lexer::TokenKind;

use crate::ast::*;
use crate::parser::Parser;

impl Parser {
    /// Parses a template string (production [51]).
    ///
    /// Expects the current token to be TmplStart.
    /// Returns a TemplateString with Text and Interpolation parts.
    pub fn parse_template_string(&mut self) -> TemplateString {
        let start = self.peek_span().start;

        // Expect TmplStart (opening ")
        if self.expect(TokenKind::TmplStart).is_err() {
            return TemplateString {
                parts: vec![],
                span: start..self.previous_span().end,
            };
        }

        let mut parts = Vec::new();

        loop {
            match self.peek() {
                TokenKind::TmplText => {
                    let span = self.peek_span();
                    self.advance();
                    parts.push(TemplatePart::Text(span));
                }
                TokenKind::TmplInterpStart => {
                    let interp_start = self.peek_span().start;
                    self.advance(); // consume {
                    let expr_id = self.parse_expr(0);
                    let interp_end = self.peek_span().end;
                    if self.expect(TokenKind::TmplInterpEnd).is_err() {
                        // Unterminated interpolation -- lexer may have emitted SYN045
                        break;
                    }
                    parts.push(TemplatePart::Interpolation(
                        expr_id,
                        interp_start..interp_end,
                    ));
                }
                TokenKind::TmplEnd => {
                    self.advance(); // closing "
                    break;
                }
                TokenKind::Eof => {
                    // Unterminated string -- lexer already emitted SYN002
                    break;
                }
                _ => {
                    let span = self.peek_span();
                    self.emit_error(
                        ErrorCode::Syn050,
                        "unexpected token in template string".to_string(),
                        span,
                        "unexpected token".to_string(),
                    );
                    break;
                }
            }
        }

        let end = self.previous_span().end;
        TemplateString {
            parts,
            span: start..end,
        }
    }
}
