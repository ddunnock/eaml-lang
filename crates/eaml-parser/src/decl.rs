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
        let is_python = self.at(TokenKind::KwPython);
        if is_python {
            self.advance(); // consume `python`
        }

        let path_or_module = self.parse_template_string();

        let alias = if self.at_contextual("as") {
            self.advance(); // consume `as`
            match self.expect_ident() {
                Ok((spur, _)) => Some(spur),
                Err(()) => {
                    self.synchronize();
                    return DeclId::Error(start..self.previous_span().end);
                }
            }
        } else {
            None
        };

        self.eat(TokenKind::Semicolon);
        let end = self.previous_span().end;

        let id = if is_python {
            self.ast.alloc_import(ImportDecl::Python {
                module: path_or_module,
                alias,
                span: start..end,
            })
        } else {
            self.ast.alloc_import(ImportDecl::Eaml {
                path: path_or_module,
                alias,
                span: start..end,
            })
        };
        DeclId::Import(id)
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
        if self.at_ident() {
            match self.expect_ident() {
                Ok((spur, span)) => caps.push((spur, span)),
                Err(()) => {
                    self.synchronize();
                    return DeclId::Error(start..self.previous_span().end);
                }
            }
            while self.eat(TokenKind::Comma) {
                match self.expect_ident() {
                    Ok((spur, span)) => caps.push((spur, span)),
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
    // Parameter list (productions [72]-[73])
    // ========================================================================

    fn parse_param_list(&mut self) -> Vec<Param> {
        let mut params = Vec::new();

        // Empty param list
        if self.at(TokenKind::RParen) {
            return params;
        }

        loop {
            let param_start = self.peek_span().start;
            let name = match self.expect_ident() {
                Ok((spur, _)) => spur,
                Err(()) => break,
            };

            if self.expect(TokenKind::Colon).is_err() {
                break;
            }

            let type_expr = self.parse_type_expr();

            // Optional default value: "=" literal
            let default = if self.eat(TokenKind::Eq) {
                Some(self.parse_expr(0))
            } else {
                None
            };

            let param_end = self.previous_span().end;
            params.push(Param {
                name,
                type_expr,
                default,
                span: param_start..param_end,
            });

            if !self.eat(TokenKind::Comma) {
                break;
            }
        }

        params
    }

    // ========================================================================
    // Requires clause (production [76])
    // ========================================================================

    fn parse_requires_clause(&mut self) -> Option<RequiresClause> {
        if !self.at_contextual("requires") {
            return None;
        }

        let start = self.peek_span().start;
        self.advance(); // consume "requires"

        let mut caps = Vec::new();

        if self.eat(TokenKind::LBracket) {
            // Bracketed list: [cap1, cap2] or []
            if self.at_ident() {
                if let Ok((spur, span)) = self.expect_ident() {
                    caps.push((spur, span));
                }
                while self.eat(TokenKind::Comma) {
                    match self.expect_ident() {
                        Ok((spur, span)) => caps.push((spur, span)),
                        Err(()) => break,
                    }
                }
            }
            let _ = self.expect(TokenKind::RBracket);
        } else if self.at_ident() {
            // Bare single capability
            if let Ok((spur, span)) = self.expect_ident() {
                caps.push((spur, span));
            }
        } else {
            let span = self.peek_span();
            self.emit_error(
                ErrorCode::Syn050,
                "expected capability identifier or '[' after 'requires'".into(),
                span,
                "expected capability".into(),
            );
        }

        let end = self.previous_span().end;
        Some(RequiresClause {
            caps,
            span: start..end,
        })
    }

    // ========================================================================
    // Prompt declaration (production [31])
    // ========================================================================

    fn parse_prompt_decl(&mut self) -> DeclId {
        let start = self.peek_span().start;
        self.advance(); // consume `prompt`

        let name = match self.expect_ident() {
            Ok((spur, _)) => spur,
            Err(()) => {
                self.synchronize();
                return DeclId::Error(start..self.previous_span().end);
            }
        };

        if self.expect(TokenKind::LParen).is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }

        let params = self.parse_param_list();

        if self.expect(TokenKind::RParen).is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }

        let requires = self.parse_requires_clause();

        if self.expect(TokenKind::Arrow).is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }

        let return_type = self.parse_type_expr();

        let body = self.parse_prompt_body();

        self.eat(TokenKind::Semicolon);
        let end = self.previous_span().end;

        let id = self.ast.alloc_prompt(PromptDecl {
            name,
            params,
            requires,
            return_type,
            body,
            span: start..end,
        });
        DeclId::Prompt(id)
    }

    // ========================================================================
    // Prompt body (productions [32]-[33])
    // ========================================================================

    fn parse_prompt_body(&mut self) -> PromptBody {
        let start = self.peek_span().start;

        if self.expect(TokenKind::LBrace).is_err() {
            return PromptBody {
                fields: vec![],
                span: start..self.previous_span().end,
            };
        }

        let mut fields = Vec::new();

        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            if self.at_contextual("user") {
                self.advance();
                if self.expect(TokenKind::Colon).is_err() {
                    break;
                }
                let ts = self.parse_template_string();
                fields.push(PromptField::User(ts));
            } else if self.at_contextual("system") {
                self.advance();
                if self.expect(TokenKind::Colon).is_err() {
                    break;
                }
                let ts = self.parse_template_string();
                fields.push(PromptField::System(ts));
            } else if self.at_contextual("temperature") {
                self.advance();
                if self.expect(TokenKind::Colon).is_err() {
                    break;
                }
                let span = self.peek_span();
                if self.at(TokenKind::FloatLit) || self.at(TokenKind::IntLit) {
                    self.advance();
                    fields.push(PromptField::Temperature(span));
                } else {
                    self.emit_error(
                        ErrorCode::Syn050,
                        "expected numeric value for temperature".into(),
                        span,
                        "expected number".into(),
                    );
                    break;
                }
            } else if self.at_contextual("max_tokens") {
                self.advance();
                if self.expect(TokenKind::Colon).is_err() {
                    break;
                }
                let span = self.peek_span();
                if self.at(TokenKind::IntLit) {
                    self.advance();
                    fields.push(PromptField::MaxTokens(span));
                } else {
                    self.emit_error(
                        ErrorCode::Syn050,
                        "expected integer for max_tokens".into(),
                        span,
                        "expected integer".into(),
                    );
                    break;
                }
            } else if self.at_contextual("max_retries") {
                self.advance();
                if self.expect(TokenKind::Colon).is_err() {
                    break;
                }
                let span = self.peek_span();
                if self.at(TokenKind::IntLit) {
                    self.advance();
                    fields.push(PromptField::MaxRetries(span));
                } else {
                    self.emit_error(
                        ErrorCode::Syn050,
                        "expected integer for max_retries".into(),
                        span,
                        "expected integer".into(),
                    );
                    break;
                }
            } else {
                let span = self.peek_span();
                self.emit_error(
                    ErrorCode::Syn061,
                    "unexpected field in prompt body".into(),
                    span,
                    "expected user, system, temperature, max_tokens, or max_retries".into(),
                );
                // Skip to next field or closing brace
                self.advance();
            }

            // Optional comma between fields
            self.eat(TokenKind::Comma);
        }

        let _ = self.expect(TokenKind::RBrace);
        let end = self.previous_span().end;

        PromptBody {
            fields,
            span: start..end,
        }
    }

    // ========================================================================
    // Tool declaration (production [34])
    // ========================================================================

    fn parse_tool_decl(&mut self) -> DeclId {
        let start = self.peek_span().start;
        self.advance(); // consume `tool`

        let name = match self.expect_ident() {
            Ok((spur, _)) => spur,
            Err(()) => {
                self.synchronize();
                return DeclId::Error(start..self.previous_span().end);
            }
        };

        if self.expect(TokenKind::LParen).is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }

        let params = self.parse_param_list();

        if self.expect(TokenKind::RParen).is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }

        if self.expect(TokenKind::Arrow).is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }

        let return_type = self.parse_type_expr();

        let body = self.parse_tool_body();

        self.eat(TokenKind::Semicolon);
        let end = self.previous_span().end;

        let id = self.ast.alloc_tool(ToolDecl {
            name,
            params,
            return_type,
            body,
            span: start..end,
        });
        DeclId::Tool(id)
    }

    // ========================================================================
    // Tool body (productions [35]-[37])
    // ========================================================================

    fn parse_tool_body(&mut self) -> ToolBody {
        let start = self.peek_span().start;

        if self.expect(TokenKind::LBrace).is_err() {
            return ToolBody::Empty(start..self.previous_span().end);
        }

        // Left-factored dispatch after '{'
        if self.at(TokenKind::KwPythonBridge) {
            self.parse_python_bridge(None, start)
        } else if self.at_contextual("description") {
            // description: "..." python %{ ... }%
            self.advance(); // consume "description"
            if self.expect(TokenKind::Colon).is_err() {
                let _ = self.expect(TokenKind::RBrace);
                return ToolBody::Empty(start..self.previous_span().end);
            }
            let description = self.parse_template_string();

            if self.at(TokenKind::KwPythonBridge) {
                self.parse_python_bridge(Some(description), start)
            } else {
                let span = self.peek_span();
                self.emit_error(
                    ErrorCode::Syn050,
                    "expected 'python %{' after description in tool body".into(),
                    span,
                    "expected python bridge".into(),
                );
                self.synchronize();
                ToolBody::Empty(start..self.previous_span().end)
            }
        } else if self.at(TokenKind::RBrace) {
            // Empty body
            self.advance(); // consume '}'
            let end = self.previous_span().end;
            ToolBody::Empty(start..end)
        } else {
            // Native body (Post-MVP)
            let span = self.peek_span();
            self.emit_error(
                ErrorCode::Syn050,
                "Native tool bodies are not supported in EAML v0.1. Use python %{ }% for tool implementations.".into(),
                span,
                "native bodies not supported".into(),
            );
            // Skip to matching RBrace
            let mut depth = 1i32;
            while !self.at(TokenKind::Eof) && depth > 0 {
                match self.peek() {
                    TokenKind::LBrace => {
                        depth += 1;
                        self.advance();
                    }
                    TokenKind::RBrace => {
                        depth -= 1;
                        if depth == 0 {
                            self.advance();
                            break;
                        }
                        self.advance();
                    }
                    _ => {
                        self.advance();
                    }
                }
            }
            let end = self.previous_span().end;
            ToolBody::Native {
                stmts: vec![],
                span: start..end,
            }
        }
    }

    /// Parses a `python %{ ... }%` block with an optional description.
    fn parse_python_bridge(
        &mut self,
        description: Option<TemplateString>,
        start: usize,
    ) -> ToolBody {
        self.advance(); // consume KwPythonBridge
        let code_span = self.peek_span();
        if self.at(TokenKind::PythonBlock) {
            self.advance();
        } else {
            self.emit_error(
                ErrorCode::Syn050,
                "expected Python block content".into(),
                code_span.clone(),
                "expected Python code".into(),
            );
        }
        let _ = self.expect(TokenKind::RBrace);
        let end = self.previous_span().end;
        ToolBody::PythonBridge {
            description,
            code_span,
            span: start..end,
        }
    }

    // ========================================================================
    // Agent declaration (production [38])
    // ========================================================================

    fn parse_agent_decl(&mut self) -> DeclId {
        let start = self.peek_span().start;
        self.advance(); // consume `agent`

        let name = match self.expect_ident() {
            Ok((spur, _)) => spur,
            Err(()) => {
                self.synchronize();
                return DeclId::Error(start..self.previous_span().end);
            }
        };

        if self.expect(TokenKind::LBrace).is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }

        let mut fields = Vec::new();

        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            // "model" is KwModel (a keyword), not an ident
            if self.at(TokenKind::KwModel) {
                let field_start = self.peek_span().start;
                self.advance();
                if self.expect(TokenKind::Colon).is_err() {
                    break;
                }
                match self.expect_ident() {
                    Ok((spur, _)) => {
                        let end = self.previous_span().end;
                        fields.push(AgentField::Model(spur, field_start..end));
                    }
                    Err(()) => break,
                }
            } else if self.at_contextual("tools") {
                let field_start = self.peek_span().start;
                self.advance();
                if self.expect(TokenKind::Colon).is_err() {
                    break;
                }
                if self.expect(TokenKind::LBracket).is_err() {
                    break;
                }

                let mut tools = Vec::new();
                if self.at_ident() {
                    match self.expect_ident() {
                        Ok((spur, span)) => tools.push((spur, span)),
                        Err(()) => break,
                    }
                    while self.eat(TokenKind::Comma) {
                        match self.expect_ident() {
                            Ok((spur, span)) => tools.push((spur, span)),
                            Err(()) => break,
                        }
                    }
                }

                if self.expect(TokenKind::RBracket).is_err() {
                    break;
                }
                let end = self.previous_span().end;
                fields.push(AgentField::Tools(tools, field_start..end));
            } else if self.at_contextual("system") {
                self.advance();
                if self.expect(TokenKind::Colon).is_err() {
                    break;
                }
                let ts = self.parse_template_string();
                fields.push(AgentField::System(ts));
            } else if self.at_contextual("max_turns") {
                self.advance();
                if self.expect(TokenKind::Colon).is_err() {
                    break;
                }
                let span = self.peek_span();
                if self.at(TokenKind::IntLit) {
                    self.advance();
                    fields.push(AgentField::MaxTurns(span));
                } else {
                    self.emit_error(
                        ErrorCode::Syn050,
                        "expected integer for max_turns".into(),
                        span,
                        "expected integer".into(),
                    );
                    break;
                }
            } else if self.at_contextual("on_error") {
                self.advance();
                if self.expect(TokenKind::Colon).is_err() {
                    break;
                }
                let field_start = self.previous_span().start;
                let policy = self.parse_error_policy();
                let end = self.previous_span().end;
                fields.push(AgentField::OnError(policy, field_start..end));
            } else {
                let span = self.peek_span();
                self.emit_error(
                    ErrorCode::Syn061,
                    "unexpected field in agent body".into(),
                    span,
                    "expected model, tools, system, max_turns, or on_error".into(),
                );
                self.advance();
            }

            // Optional comma between fields
            self.eat(TokenKind::Comma);
        }

        if self.expect(TokenKind::RBrace).is_err() {
            self.synchronize();
            return DeclId::Error(start..self.previous_span().end);
        }

        self.eat(TokenKind::Semicolon);
        let end = self.previous_span().end;

        let id = self.ast.alloc_agent(AgentDecl {
            name,
            fields,
            span: start..end,
        });
        DeclId::Agent(id)
    }

    // ========================================================================
    // Error policy (production [40])
    // ========================================================================

    fn parse_error_policy(&mut self) -> ErrorPolicy {
        if self.at_contextual("fail") {
            self.advance();
            ErrorPolicy::Fail
        } else if self.at_contextual("retry") {
            self.advance(); // consume "retry"
            if self.expect(TokenKind::LParen).is_err() {
                return ErrorPolicy::Fail;
            }
            let retries_span = self.peek_span();
            if self.at(TokenKind::IntLit) {
                self.advance();
            } else {
                self.emit_error(
                    ErrorCode::Syn050,
                    "expected integer for retry count".into(),
                    retries_span.clone(),
                    "expected integer".into(),
                );
            }
            let _ = self.expect(TokenKind::RParen);
            let _ = self.expect_contextual("then");
            let _ = self.expect_contextual("fail");
            ErrorPolicy::RetryThenFail { retries_span }
        } else {
            let span = self.peek_span();
            self.emit_error(
                ErrorCode::Syn050,
                "expected 'fail' or 'retry' for error policy".into(),
                span,
                "expected error policy".into(),
            );
            ErrorPolicy::Fail
        }
    }
}
