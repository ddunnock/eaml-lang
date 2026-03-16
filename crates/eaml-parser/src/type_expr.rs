//! Type expression parser for EAML.
//!
//! Handles grammar productions [42]-[50]: typeExpr, typeModifiers, baseType,
//! namedType, boundedSuffix, boundParams, boundParam, arraySuffix,
//! optionalSuffix, literalUnion.

use eaml_errors::ErrorCode;
use eaml_lexer::TokenKind;

use crate::ast::*;
use crate::parser::Parser;

impl Parser {
    /// Parses a type expression (production [42]).
    ///
    /// Entry point for type parsing. Checks for literal union first,
    /// then falls through to base type with modifiers.
    pub fn parse_type_expr(&mut self) -> TypeExprId {
        // Try literal union: STRING ("|" STRING)+
        if self.peek() == TokenKind::TmplStart {
            if let Some(id) = self.try_literal_union() {
                return id;
            }
        }

        let base = self.parse_base_type();
        self.parse_type_modifiers(base)
    }

    /// Parses a base type (production [43]).
    ///
    /// baseType ::= namedType | literalUnion | "(" typeExpr ")"
    fn parse_base_type(&mut self) -> TypeExprId {
        match self.peek() {
            TokenKind::Ident(_) => self.parse_named_type(),
            TokenKind::LParen => {
                let start = self.peek_span().start;
                self.advance(); // (
                let inner = self.parse_type_expr();
                if self.expect(TokenKind::RParen).is_err() {
                    return self
                        .ast
                        .alloc_type_expr(TypeExpr::Error(start..self.previous_span().end));
                }
                let end = self.previous_span().end;
                self.ast
                    .alloc_type_expr(TypeExpr::Grouped(inner, start..end))
            }
            _ => {
                let span = self.peek_span();
                self.emit_error(
                    ErrorCode::Syn050,
                    "expected type expression".to_string(),
                    span.clone(),
                    "expected type expression".to_string(),
                );
                self.ast.alloc_type_expr(TypeExpr::Error(span))
            }
        }
    }

    /// Parses a named type with optional bounded suffix (production [44]).
    ///
    /// namedType ::= IDENT boundedSuffix?
    fn parse_named_type(&mut self) -> TypeExprId {
        let (spur, name_span) = match self.expect_ident() {
            Ok(result) => result,
            Err(()) => {
                let span = self.previous_span();
                return self.ast.alloc_type_expr(TypeExpr::Error(span));
            }
        };

        // Check for bounded suffix: IDENT "<" ...
        if self.peek() == TokenKind::LAngle {
            return self.parse_bounded_suffix(spur, name_span.start);
        }

        self.ast.alloc_type_expr(TypeExpr::Named(spur, name_span))
    }

    /// Parses bounded type parameters (productions [45]-[47]).
    ///
    /// boundedSuffix ::= "<" boundParams ">"
    /// boundParams ::= boundParam ("," boundParam)*
    /// boundParam ::= (IDENT ":")? (FLOAT | INT)
    fn parse_bounded_suffix(&mut self, base: lasso::Spur, start: usize) -> TypeExprId {
        // Consume "<"
        self.advance();

        let mut params = Vec::new();

        loop {
            let param_start = self.peek_span().start;

            // Check for named parameter: IDENT ":"
            let name = if self.at_ident() && self.peek_at(1) == TokenKind::Colon {
                let (spur, _) = self.expect_ident().unwrap();
                self.advance(); // consume ":"
                Some(spur)
            } else {
                None
            };

            // Expect a numeric value (FLOAT or INT)
            let value_span = match self.peek() {
                TokenKind::FloatLit | TokenKind::IntLit => {
                    let span = self.peek_span();
                    self.advance();
                    span
                }
                _ => {
                    let span = self.peek_span();
                    self.emit_error(
                        ErrorCode::Syn050,
                        "expected numeric value in bounded type parameter".to_string(),
                        span.clone(),
                        "expected number".to_string(),
                    );
                    // Try to recover by breaking out
                    break;
                }
            };

            let param_end = self.previous_span().end;
            params.push(BoundParam {
                name,
                value_span,
                span: param_start..param_end,
            });

            // Continue if comma, break otherwise
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }

        // Expect ">"
        let _ = self.expect(TokenKind::RAngle);
        let end = self.previous_span().end;

        self.ast.alloc_type_expr(TypeExpr::Bounded {
            base,
            params,
            span: start..end,
        })
    }

    /// Applies type modifiers (production [42a]).
    ///
    /// typeModifiers ::= arraySuffix optionalSuffix?
    ///                 | optionalSuffix (arraySuffix optionalSuffix?)?
    ///                 | (empty)
    fn parse_type_modifiers(&mut self, base: TypeExprId) -> TypeExprId {
        let base_start = match &self.ast[base] {
            TypeExpr::Named(_, span)
            | TypeExpr::Bounded { span, .. }
            | TypeExpr::Array(_, span)
            | TypeExpr::Optional(_, span)
            | TypeExpr::LiteralUnion { span, .. }
            | TypeExpr::Grouped(_, span)
            | TypeExpr::Error(span) => span.start,
        };

        match self.peek() {
            TokenKind::LBracket => {
                // [] first, then optional ?
                self.advance(); // [
                if self.expect(TokenKind::RBracket).is_err() {
                    return base;
                }
                let end = self.previous_span().end;

                // Check for SYN042 multi-dimensional array
                if self.peek() == TokenKind::LBracket {
                    let err_span = self.peek_span();
                    self.emit_error(
                        ErrorCode::Syn042,
                        "multi-dimensional arrays are not supported in EAML v0.1".to_string(),
                        err_span,
                        "multi-dimensional arrays not supported".to_string(),
                    );
                }

                let array_id = self
                    .ast
                    .alloc_type_expr(TypeExpr::Array(base, base_start..end));

                if self.eat(TokenKind::Question) {
                    let opt_end = self.previous_span().end;
                    self.ast
                        .alloc_type_expr(TypeExpr::Optional(array_id, base_start..opt_end))
                } else {
                    array_id
                }
            }
            TokenKind::Question => {
                // ? first, then optional []
                self.advance(); // ?
                let end = self.previous_span().end;
                let opt_id = self
                    .ast
                    .alloc_type_expr(TypeExpr::Optional(base, base_start..end));

                if self.peek() == TokenKind::LBracket {
                    self.advance(); // [
                    if self.expect(TokenKind::RBracket).is_err() {
                        return opt_id;
                    }
                    let arr_end = self.previous_span().end;
                    let array_id = self
                        .ast
                        .alloc_type_expr(TypeExpr::Array(opt_id, base_start..arr_end));

                    // T?[]? case
                    if self.eat(TokenKind::Question) {
                        let final_end = self.previous_span().end;
                        self.ast
                            .alloc_type_expr(TypeExpr::Optional(array_id, base_start..final_end))
                    } else {
                        array_id
                    }
                } else {
                    opt_id
                }
            }
            _ => base, // bare type, no modifiers
        }
    }

    /// Attempts to parse a literal union. Returns Some if successful, None if
    /// the current position doesn't start a literal union.
    ///
    /// A literal union requires: STRING "|" STRING ("|" STRING)*
    /// Since the lexer tokenizes all strings as template strings, we need to
    /// look ahead past the first template string to check for "|".
    fn try_literal_union(&mut self) -> Option<TypeExprId> {
        // Save position for potential backtrack
        let saved_pos = self.save_pos();

        // Try to skip past the first template string
        if self.peek() != TokenKind::TmplStart {
            return None;
        }
        self.advance(); // TmplStart

        // Skip template contents (only Text allowed, no interpolation)
        loop {
            match self.peek() {
                TokenKind::TmplText => {
                    self.advance();
                }
                TokenKind::TmplEnd => {
                    self.advance();
                    break;
                }
                TokenKind::TmplInterpStart => {
                    // Has interpolation -- not a literal union candidate
                    self.restore_pos(saved_pos);
                    return None;
                }
                _ => {
                    self.restore_pos(saved_pos);
                    return None;
                }
            }
        }

        // Check if "|" follows
        if self.peek() != TokenKind::Pipe {
            self.restore_pos(saved_pos);
            return None;
        }

        // This IS a literal union. Restore and parse properly.
        self.restore_pos(saved_pos);
        Some(self.parse_literal_union())
    }

    /// Parses a literal union (production [50]).
    ///
    /// literalUnion ::= STRING ("|" STRING)+
    fn parse_literal_union(&mut self) -> TypeExprId {
        let start = self.peek_span().start;
        let mut members = Vec::new();

        // Parse first string member
        let member_span = self.parse_literal_union_member();
        members.push(member_span);

        // Parse remaining "|" STRING members
        while self.eat(TokenKind::Pipe) {
            let member_span = self.parse_literal_union_member();
            members.push(member_span);
        }

        let end = self.previous_span().end;
        self.ast.alloc_type_expr(TypeExpr::LiteralUnion {
            members,
            span: start..end,
        })
    }

    /// Parses a single string member of a literal union.
    /// Consumes TmplStart, TmplText*, TmplEnd and returns the text span.
    fn parse_literal_union_member(&mut self) -> eaml_errors::Span {
        let start = self.peek_span().start;
        if self.expect(TokenKind::TmplStart).is_err() {
            return start..self.previous_span().end;
        }

        // Collect the text span (should be a single TmplText, or empty string)
        let text_span = if self.peek() == TokenKind::TmplText {
            let span = self.peek_span();
            self.advance();
            span
        } else {
            // Empty string ""
            let pos = self.peek_span().start;
            pos..pos
        };

        // Check for interpolation (not allowed in literal union members)
        if self.peek() == TokenKind::TmplInterpStart {
            let span = self.peek_span();
            self.emit_error(
                ErrorCode::Syn050,
                "interpolation not allowed in literal union member".to_string(),
                span,
                "interpolation not allowed here".to_string(),
            );
        }

        if self.expect(TokenKind::TmplEnd).is_err() {
            return text_span;
        }

        text_span
    }

    /// Saves the current parser position for backtracking.
    pub(crate) fn save_pos(&self) -> usize {
        self.pos
    }

    /// Restores the parser to a previously saved position.
    pub(crate) fn restore_pos(&mut self, pos: usize) {
        self.pos = pos;
    }
}
