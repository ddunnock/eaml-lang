//! Pratt expression parser for EAML.
//!
//! Handles grammar productions [54]-[65]: expr through literal,
//! plus [74]-[75]: argList and arg.

use eaml_errors::ErrorCode;
use eaml_lexer::TokenKind;

use crate::ast::*;
use crate::parser::Parser;

// ============================================================================
// Binding power functions
// ============================================================================

/// Returns the binding power for prefix operators.
fn prefix_bp(kind: TokenKind) -> Option<u8> {
    match kind {
        TokenKind::Bang | TokenKind::Minus => Some(70),
        TokenKind::KwAwait => Some(65),
        _ => None,
    }
}

/// Returns (left_bp, right_bp) for infix operators.
/// Left-associative: left_bp < right_bp.
fn infix_bp(kind: TokenKind) -> Option<(u8, u8)> {
    match kind {
        TokenKind::PipePipe => Some((10, 11)), // || left-assoc
        TokenKind::AmpAmp => Some((20, 21)),   // && left-assoc
        TokenKind::EqEq | TokenKind::BangEq => Some((30, 31)), // == != left-assoc
        TokenKind::LAngle | TokenKind::RAngle | TokenKind::LessEq | TokenKind::GreaterEq => {
            Some((35, 36))
        } // comparisons left-assoc
        TokenKind::Plus | TokenKind::Minus => Some((40, 41)), // + - left-assoc
        TokenKind::Star | TokenKind::Slash => Some((50, 51)), // * / left-assoc
        _ => None,
    }
}

/// Returns binding power for postfix operators.
fn postfix_bp(kind: TokenKind) -> Option<u8> {
    match kind {
        TokenKind::Dot | TokenKind::LParen | TokenKind::LBracket => Some(80),
        _ => None,
    }
}

/// Converts a token kind to a BinOp.
fn token_to_binop(kind: TokenKind) -> BinOp {
    match kind {
        TokenKind::Plus => BinOp::Add,
        TokenKind::Minus => BinOp::Sub,
        TokenKind::Star => BinOp::Mul,
        TokenKind::Slash => BinOp::Div,
        TokenKind::EqEq => BinOp::Eq,
        TokenKind::BangEq => BinOp::NotEq,
        TokenKind::LAngle => BinOp::Lt,
        TokenKind::RAngle => BinOp::Gt,
        TokenKind::LessEq => BinOp::LtEq,
        TokenKind::GreaterEq => BinOp::GtEq,
        TokenKind::AmpAmp => BinOp::And,
        TokenKind::PipePipe => BinOp::Or,
        _ => unreachable!("token_to_binop called with non-operator token"),
    }
}

impl Parser {
    /// Parses an expression using Pratt parsing (production [54]).
    ///
    /// `min_bp` is the minimum binding power for the current context.
    /// Call with `min_bp = 0` for a full expression.
    pub fn parse_expr(&mut self, min_bp: u8) -> ExprId {
        let start = self.peek_span().start;

        // Check for prefix operators
        let mut lhs = if let Some(bp) = prefix_bp(self.peek()) {
            let op_kind = self.peek();
            self.advance();

            let operand = self.parse_expr(bp);
            let end = self.previous_span().end;

            match op_kind {
                TokenKind::KwAwait => self.ast.alloc_expr(Expr::Await {
                    operand,
                    span: start..end,
                }),
                TokenKind::Bang => self.ast.alloc_expr(Expr::UnaryOp {
                    op: UnaryOp::Not,
                    operand,
                    span: start..end,
                }),
                TokenKind::Minus => self.ast.alloc_expr(Expr::UnaryOp {
                    op: UnaryOp::Neg,
                    operand,
                    span: start..end,
                }),
                _ => unreachable!(),
            }
        } else {
            self.parse_primary_expr()
        };

        // Loop: try postfix then infix
        loop {
            let current = self.peek();

            // Check postfix operators first (highest precedence)
            if let Some(bp) = postfix_bp(current) {
                if bp < min_bp {
                    break;
                }

                match current {
                    TokenKind::Dot => {
                        self.advance(); // .
                        match self.expect_ident() {
                            Ok((field, _)) => {
                                let end = self.previous_span().end;
                                lhs = self.ast.alloc_expr(Expr::FieldAccess {
                                    object: lhs,
                                    field,
                                    span: start..end,
                                });
                            }
                            Err(()) => {
                                let end = self.previous_span().end;
                                lhs = self.ast.alloc_expr(Expr::Error(start..end));
                                break;
                            }
                        }
                        continue;
                    }
                    TokenKind::LParen => {
                        self.advance(); // (
                        let args = self.parse_arg_list();
                        if self.expect(TokenKind::RParen).is_err() {
                            let end = self.previous_span().end;
                            lhs = self.ast.alloc_expr(Expr::Error(start..end));
                            break;
                        }
                        let end = self.previous_span().end;
                        lhs = self.ast.alloc_expr(Expr::FnCall {
                            callee: lhs,
                            args,
                            span: start..end,
                        });
                        continue;
                    }
                    TokenKind::LBracket => {
                        self.advance(); // [
                        let index = self.parse_expr(0);
                        if self.expect(TokenKind::RBracket).is_err() {
                            let end = self.previous_span().end;
                            lhs = self.ast.alloc_expr(Expr::Error(start..end));
                            break;
                        }
                        let end = self.previous_span().end;
                        lhs = self.ast.alloc_expr(Expr::Index {
                            object: lhs,
                            index,
                            span: start..end,
                        });
                        continue;
                    }
                    _ => {}
                }
            }

            // Check infix operators
            if let Some((l_bp, r_bp)) = infix_bp(current) {
                if l_bp < min_bp {
                    break;
                }

                let op = token_to_binop(current);
                self.advance();

                let rhs = self.parse_expr(r_bp);
                let end = self.previous_span().end;
                lhs = self.ast.alloc_expr(Expr::BinaryOp {
                    left: lhs,
                    op,
                    right: rhs,
                    span: start..end,
                });
                continue;
            }

            // Handle reserved operators
            if current == TokenKind::PipelineOp {
                let span = self.peek_span();
                self.emit_error(
                    ErrorCode::Syn081,
                    "pipeline operator >> is not supported in EAML v0.1".to_string(),
                    span,
                    "reserved operator".to_string(),
                );
                self.advance();
                continue;
            }

            break;
        }

        lhs
    }

    /// Parses a primary expression (production [64]).
    fn parse_primary_expr(&mut self) -> ExprId {
        match self.peek() {
            TokenKind::IntLit => {
                let span = self.peek_span();
                self.advance();
                self.ast.alloc_expr(Expr::IntLit(span))
            }
            TokenKind::FloatLit => {
                let span = self.peek_span();
                self.advance();
                self.ast.alloc_expr(Expr::FloatLit(span))
            }
            TokenKind::KwTrue => {
                let span = self.peek_span();
                self.advance();
                self.ast.alloc_expr(Expr::BoolLit(true, span))
            }
            TokenKind::KwFalse => {
                let span = self.peek_span();
                self.advance();
                self.ast.alloc_expr(Expr::BoolLit(false, span))
            }
            TokenKind::KwNull => {
                let span = self.peek_span();
                self.advance();
                self.ast.alloc_expr(Expr::NullLit(span))
            }
            TokenKind::Ident(spur) => {
                let span = self.peek_span();
                self.advance();
                self.ast.alloc_expr(Expr::Ident(spur, span))
            }
            TokenKind::LParen => {
                let start = self.peek_span().start;
                self.advance(); // (
                let inner = self.parse_expr(0);
                if self.expect(TokenKind::RParen).is_err() {
                    let end = self.previous_span().end;
                    return self.ast.alloc_expr(Expr::Error(start..end));
                }
                let end = self.previous_span().end;
                self.ast.alloc_expr(Expr::Paren {
                    inner,
                    span: start..end,
                })
            }
            TokenKind::TmplStart => {
                let ts = self.parse_template_string();
                self.ast.alloc_expr(Expr::TemplateStr(ts))
            }
            _ => {
                let span = self.peek_span();
                self.emit_error(
                    ErrorCode::Syn080,
                    "expected expression".to_string(),
                    span.clone(),
                    "expected expression".to_string(),
                );
                self.ast.alloc_expr(Expr::Error(span))
            }
        }
    }

    /// Parses a comma-separated argument list (productions [74]-[75]).
    ///
    /// Handles both positional and named arguments.
    /// Named args use LL(2): IDENT ":" expr.
    pub fn parse_arg_list(&mut self) -> Vec<Arg> {
        let mut args = Vec::new();

        // Empty arg list
        if self.peek() == TokenKind::RParen {
            return args;
        }

        loop {
            let arg_start = self.peek_span().start;

            // LL(2) check for named argument: IDENT ":"
            let name = if self.at_ident() && self.peek_at(1) == TokenKind::Colon {
                let (spur, _) = self.expect_ident().unwrap();
                self.advance(); // consume ":"
                Some(spur)
            } else {
                None
            };

            let value = self.parse_expr(0);
            let arg_end = self.previous_span().end;

            args.push(Arg {
                name,
                value,
                span: arg_start..arg_end,
            });

            if !self.eat(TokenKind::Comma) {
                break;
            }
        }

        args
    }
}
