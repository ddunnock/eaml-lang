//! Declaration parsers for EAML.
//!
//! Handles grammar productions [24]-[41]: Program, declaration dispatch,
//! importDecl, modelDecl, schemaDecl, promptDecl, toolDecl, agentDecl, letDecl,
//! plus helper productions for param lists, requires clauses, prompt/tool/agent bodies.

use eaml_errors::ErrorCode;
use eaml_lexer::TokenKind;

use crate::ast::*;
use crate::parser::Parser;
use crate::ParseOutput;

impl Parser {
    /// Parses the token stream into a Program AST (production [24]).
    ///
    /// This is the main entry point. Dispatches on keyword to the
    /// appropriate declaration parser. Handles error recovery and
    /// error limit checking.
    pub fn parse_program(mut self) -> ParseOutput {
        let mut declarations = Vec::new();

        while !self.at(TokenKind::Eof) {
            if self.error_limit_reached() {
                break;
            }

            let decl = match self.peek() {
                TokenKind::KwImport => self.parse_import_decl(),
                TokenKind::KwModel => self.parse_model_decl(),
                TokenKind::KwSchema => self.parse_schema_decl(),
                TokenKind::KwPrompt => self.parse_prompt_decl(),
                TokenKind::KwTool => self.parse_tool_decl(),
                TokenKind::KwAgent => self.parse_agent_decl(),
                TokenKind::KwLet => self.parse_let_decl(),
                // Post-MVP reserved keywords
                TokenKind::KwPipeline => self.parse_reserved_decl(
                    ErrorCode::Syn080,
                    "Pipeline declarations are not supported in EAML v0.1.",
                ),
                TokenKind::KwEnum => self.parse_reserved_decl(
                    ErrorCode::Syn082,
                    "Enum declarations are not supported in EAML v0.1.",
                ),
                // @ annotation
                TokenKind::At => {
                    let span = self.peek_span();
                    self.emit_error(
                        ErrorCode::Syn090,
                        "@annotations are not supported in EAML v0.1.".into(),
                        span.clone(),
                        "reserved syntax".into(),
                    );
                    self.advance();
                    self.synchronize();
                    DeclId::Error(span)
                }
                _ => {
                    let span = self.peek_span();
                    self.emit_error(
                        ErrorCode::Syn060,
                        format!("expected declaration, found {:?}", self.peek()),
                        span.clone(),
                        "expected model, schema, prompt, tool, agent, import, or let".into(),
                    );
                    self.synchronize();
                    DeclId::Error(span)
                }
            };

            declarations.push(decl);
        }

        let source_len = self.source().len();
        let program = Program {
            declarations,
            span: 0..source_len,
        };

        ParseOutput {
            ast: self.ast,
            program,
            diagnostics: self.diagnostics,
            interner: self.interner,
        }
    }

    // ========================================================================
    // Import declaration (production [26])
    // ========================================================================

    fn parse_import_decl(&mut self) -> DeclId {
        let start = self.peek_span().start;
        self.advance(); // consume `import`

        // LL(2): if next is KwPython -> Python import, else EAML import
        if self.at(TokenKind::KwPython) {
            self.advance(); // consume `python`
            let module = self.parse_template_string();

            let alias = if self.at_contextual("as") {
                self.advance(); // consume `as`
                match self.expect_ident() {
                    Ok((spur, _)) => Some(spur),
                    Err(()) => {
                        self.synchronize();
                        let end = self.previous_span().end;
                        return DeclId::Error(start..end);
                    }
                }
            } else {
                None
            };

            self.eat(TokenKind::Semicolon);
            let end = self.previous_span().end;
            let id = self.ast.alloc_import(ImportDecl::Python {
                module,
                alias,
                span: start..end,
            });
            DeclId::Import(id)
        } else {
            // EAML file import
            let path = self.parse_template_string();

            let alias = if self.at_contextual("as") {
                self.advance(); // consume `as`
                match self.expect_ident() {
                    Ok((spur, _)) => Some(spur),
                    Err(()) => {
                        self.synchronize();
                        let end = self.previous_span().end;
                        return DeclId::Error(start..end);
                    }
                }
            } else {
                None
            };

            self.eat(TokenKind::Semicolon);
            let end = self.previous_span().end;
            let id = self.ast.alloc_import(ImportDecl::Eaml {
                path,
                alias,
                span: start..end,
            });
            DeclId::Import(id)
        }
    }

    // ========================================================================
    // Model declaration (production [27])
    // ========================================================================

    fn parse_model_decl(&mut self) -> DeclId {
        let start = self.peek_span().start;
        self.advance(); // consume `model`

        let name = match self.expect_ident() {
            Ok((spur, _)) => spur,
            Err(()) => {
                self.synchronize();
                return DeclId::Error(start..self.previous_span().end);
            }
        };

        if self.expect(TokenKind::Eq).is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }

        // Expect contextual "Model"
        if self.expect_contextual("Model").is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }

        if self.expect(TokenKind::LParen).is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }

        // Fixed order: id, provider, caps
        if self.expect_contextual("id").is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }
        if self.expect(TokenKind::Colon).is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }
        let model_id = self.parse_template_string();
        if self.expect(TokenKind::Comma).is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }

        if self.expect_contextual("provider").is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }
        if self.expect(TokenKind::Colon).is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }
        let provider = self.parse_template_string();
        if self.expect(TokenKind::Comma).is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }

        if self.expect_contextual("caps").is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }
        if self.expect(TokenKind::Colon).is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }
        if self.expect(TokenKind::LBracket).is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }

        // Parse capability list (comma-separated idents, may be empty)
        let mut caps = Vec::new();
        let mut cap_spans = Vec::new();
        if self.at_ident() {
            match self.expect_ident() {
                Ok((spur, span)) => {
                    caps.push(spur);
                    cap_spans.push(span);
                }
                Err(()) => {
                    self.synchronize();
                    return DeclId::Error(start..self.previous_span().end);
                }
            }
            while self.eat(TokenKind::Comma) {
                match self.expect_ident() {
                    Ok((spur, span)) => {
                        caps.push(spur);
                        cap_spans.push(span);
                    }
                    Err(()) => {
                        self.synchronize();
                        return DeclId::Error(start..self.previous_span().end);
                    }
                }
            }
        }

        if self.expect(TokenKind::RBracket).is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }

        if self.expect(TokenKind::RParen).is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }

        self.eat(TokenKind::Semicolon);
        let end = self.previous_span().end;

        let id = self.ast.alloc_model(ModelDecl {
            name,
            model_id,
            provider,
            caps,
            cap_spans,
            span: start..end,
        });
        DeclId::Model(id)
    }

    // ========================================================================
    // Schema declaration (production [29])
    // ========================================================================

    fn parse_schema_decl(&mut self) -> DeclId {
        let start = self.peek_span().start;
        self.advance(); // consume `schema`

        let name = match self.expect_ident() {
            Ok((spur, _)) => spur,
            Err(()) => {
                self.synchronize();
                return DeclId::Error(start..self.previous_span().end);
            }
        };

        // Check for Post-MVP `extends`
        if self.at(TokenKind::KwExtends) {
            let span = self.peek_span();
            self.emit_error(
                ErrorCode::Syn083,
                "Schema inheritance is not supported in EAML v0.1.".into(),
                span,
                "reserved syntax".into(),
            );
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }

        if self.expect(TokenKind::LBrace).is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }

        // Parse field definitions
        let mut fields = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            if !self.at_ident() {
                break;
            }
            let field_start = self.peek_span().start;
            let (field_name, _) = self.expect_ident().unwrap();

            if self.expect(TokenKind::Colon).is_err() {
                self.synchronize();
                return DeclId::Error(start..self.previous_span().end);
            }

            let type_expr = self.parse_type_expr();
            let field_end = self.previous_span().end;

            fields.push(FieldDef {
                name: field_name,
                type_expr,
                span: field_start..field_end,
            });

            // Optional comma separator
            self.eat(TokenKind::Comma);
        }

        if self.expect(TokenKind::RBrace).is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }

        self.eat(TokenKind::Semicolon);
        let end = self.previous_span().end;

        let id = self.ast.alloc_schema(SchemaDecl {
            name,
            fields,
            span: start..end,
        });
        DeclId::Schema(id)
    }

    // ========================================================================
    // Let declaration (production [41])
    // ========================================================================

    fn parse_let_decl(&mut self) -> DeclId {
        let start = self.peek_span().start;
        self.advance(); // consume `let`

        let name = match self.expect_ident() {
            Ok((spur, _)) => spur,
            Err(()) => {
                self.synchronize();
                return DeclId::Error(start..self.previous_span().end);
            }
        };

        if self.expect(TokenKind::Colon).is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }

        let type_expr = self.parse_type_expr();

        if self.expect(TokenKind::Eq).is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }

        let value = self.parse_expr(0);

        self.eat(TokenKind::Semicolon);
        let end = self.previous_span().end;

        let id = self.ast.alloc_let(LetDecl {
            name,
            type_expr,
            value,
            span: start..end,
        });
        DeclId::Let(id)
    }

    // ========================================================================
    // Reserved declaration (Post-MVP)
    // ========================================================================

    fn parse_reserved_decl(&mut self, code: ErrorCode, msg: &str) -> DeclId {
        let span = self.peek_span();
        self.emit_error(
            code,
            msg.to_string(),
            span.clone(),
            "reserved syntax".into(),
        );
        self.advance(); // skip keyword
        self.synchronize();
        DeclId::Error(span)
    }

    // ========================================================================
    // Prompt declaration (production [31]) -- stub for Task 1
    // ========================================================================

    pub(crate) fn parse_prompt_decl(&mut self) -> DeclId {
        let start = self.peek_span().start;
        self.advance(); // consume `prompt`
        self.synchronize();
        DeclId::Error(start..self.previous_span().end)
    }

    // ========================================================================
    // Tool declaration (production [34]) -- stub for Task 1
    // ========================================================================

    pub(crate) fn parse_tool_decl(&mut self) -> DeclId {
        let start = self.peek_span().start;
        self.advance(); // consume `tool`
        self.synchronize();
        DeclId::Error(start..self.previous_span().end)
    }

    // ========================================================================
    // Agent declaration (production [38]) -- stub for Task 1
    // ========================================================================

    pub(crate) fn parse_agent_decl(&mut self) -> DeclId {
        let start = self.peek_span().start;
        self.advance(); // consume `agent`
        self.synchronize();
        DeclId::Error(start..self.previous_span().end)
    }
}
