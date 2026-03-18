//! Tests for parser infrastructure: AST types, arenas, and parser cursor.

use eaml_lexer::TokenKind;
use eaml_parser::ast::*;

// === Task 1: AST node types, typed arenas, ID newtypes ===

#[test]
fn expr_id_equality() {
    let mut ast = Ast::new();
    let id0 = ast.alloc_expr(Expr::IntLit(0..1));
    let id1 = ast.alloc_expr(Expr::IntLit(1..2));
    assert_ne!(id0, id1);

    // Same allocation gives deterministic IDs
    let mut ast2 = Ast::new();
    let id0_again = ast2.alloc_expr(Expr::IntLit(0..1));
    assert_eq!(id0, id0_again);
}

#[test]
fn ast_new_creates_empty_arenas() {
    let ast = Ast::new();
    assert!(ast.models.is_empty());
    assert!(ast.schemas.is_empty());
    assert!(ast.prompts.is_empty());
    assert!(ast.tools.is_empty());
    assert!(ast.agents.is_empty());
    assert!(ast.imports.is_empty());
    assert!(ast.lets.is_empty());
    assert!(ast.exprs.is_empty());
    assert!(ast.type_exprs.is_empty());
}

#[test]
fn alloc_expr_returns_sequential_ids() {
    let mut ast = Ast::new();
    let id0 = ast.alloc_expr(Expr::IntLit(0..1));
    let id1 = ast.alloc_expr(Expr::NullLit(1..2));
    assert_ne!(id0, id1);
    assert_eq!(ast.exprs.len(), 2);
}

#[test]
fn alloc_type_expr_returns_sequential_ids() {
    let mut interner = eaml_lexer::Interner::new();
    let name = interner.intern("string");
    let mut ast = Ast::new();
    let id0 = ast.alloc_type_expr(TypeExpr::Named(name, 0..6));
    let id1 = ast.alloc_type_expr(TypeExpr::Named(name, 7..13));
    assert_ne!(id0, id1);
    assert_eq!(ast.type_exprs.len(), 2);
}

#[test]
fn alloc_model_returns_model_decl_id() {
    let mut interner = eaml_lexer::Interner::new();
    let name = interner.intern("gpt4");
    let mut ast = Ast::new();
    let id = ast.alloc_model(ModelDecl {
        name,
        model_id: TemplateString {
            parts: vec![],
            span: 0..0,
        },
        provider: TemplateString {
            parts: vec![],
            span: 0..0,
        },
        caps: vec![],
        span: 0..10,
    });
    assert_eq!(ast[id].name, name);
}

#[test]
fn alloc_schema_returns_schema_decl_id() {
    let mut interner = eaml_lexer::Interner::new();
    let name = interner.intern("User");
    let mut ast = Ast::new();
    let id = ast.alloc_schema(SchemaDecl {
        name,
        fields: vec![],
        span: 0..10,
    });
    assert_eq!(ast[id].name, name);
}

#[test]
fn alloc_prompt_returns_prompt_decl_id() {
    let mut interner = eaml_lexer::Interner::new();
    let name = interner.intern("greet");
    let mut ast = Ast::new();
    let type_id = ast.alloc_type_expr(TypeExpr::Named(name, 0..5));
    let id = ast.alloc_prompt(PromptDecl {
        name,
        params: vec![],
        requires: None,
        return_type: type_id,
        body: PromptBody {
            fields: vec![],
            span: 0..0,
        },
        span: 0..10,
    });
    assert_eq!(ast[id].name, name);
}

#[test]
fn alloc_tool_returns_tool_decl_id() {
    let mut interner = eaml_lexer::Interner::new();
    let name = interner.intern("search");
    let mut ast = Ast::new();
    let type_id = ast.alloc_type_expr(TypeExpr::Named(name, 0..6));
    let id = ast.alloc_tool(ToolDecl {
        name,
        params: vec![],
        return_type: type_id,
        body: ToolBody::Empty(0..0),
        span: 0..10,
    });
    assert_eq!(ast[id].name, name);
}

#[test]
fn alloc_agent_returns_agent_decl_id() {
    let mut interner = eaml_lexer::Interner::new();
    let name = interner.intern("assistant");
    let mut ast = Ast::new();
    let id = ast.alloc_agent(AgentDecl {
        name,
        fields: vec![],
        span: 0..10,
    });
    assert_eq!(ast[id].name, name);
}

#[test]
fn alloc_import_returns_import_decl_id() {
    let mut ast = Ast::new();
    let id = ast.alloc_import(ImportDecl::Eaml {
        path: TemplateString {
            parts: vec![],
            span: 0..0,
        },
        alias: None,
        span: 0..10,
    });
    assert_eq!(ast[id].span(), &(0..10));
}

#[test]
fn alloc_let_returns_let_decl_id() {
    let mut interner = eaml_lexer::Interner::new();
    let name = interner.intern("x");
    let mut ast = Ast::new();
    let type_id = ast.alloc_type_expr(TypeExpr::Named(name, 0..1));
    let expr_id = ast.alloc_expr(Expr::IntLit(0..1));
    let id = ast.alloc_let(LetDecl {
        name,
        type_expr: type_id,
        value: expr_id,
        span: 0..10,
    });
    assert_eq!(ast[id].name, name);
}

#[test]
fn decl_id_preserves_typed_id() {
    let mut ast = Ast::new();
    let mut interner = eaml_lexer::Interner::new();
    let name = interner.intern("Foo");
    let model_id = ast.alloc_model(ModelDecl {
        name,
        model_id: TemplateString {
            parts: vec![],
            span: 0..0,
        },
        provider: TemplateString {
            parts: vec![],
            span: 0..0,
        },
        caps: vec![],
        span: 0..10,
    });
    let decl = DeclId::Model(model_id);
    match decl {
        DeclId::Model(id) => assert_eq!(id, model_id),
        _ => panic!("Expected Model variant"),
    }
}

#[test]
fn every_ast_node_has_span() {
    let mut interner = eaml_lexer::Interner::new();
    let name = interner.intern("test");

    let mut ast = Ast::new();
    let type_id = ast.alloc_type_expr(TypeExpr::Named(name, 0..4));
    let expr_id = ast.alloc_expr(Expr::IntLit(0..1));

    let model = ModelDecl {
        name,
        model_id: TemplateString {
            parts: vec![],
            span: 0..0,
        },
        provider: TemplateString {
            parts: vec![],
            span: 0..0,
        },
        caps: vec![],
        span: 0..10,
    };
    assert_eq!(model.span, 0..10);

    let schema = SchemaDecl {
        name,
        fields: vec![],
        span: 0..10,
    };
    assert_eq!(schema.span, 0..10);

    let field = FieldDef {
        name,
        type_expr: type_id,
        span: 0..5,
    };
    assert_eq!(field.span, 0..5);

    let prompt = PromptDecl {
        name,
        params: vec![],
        requires: None,
        return_type: type_id,
        body: PromptBody {
            fields: vec![],
            span: 0..0,
        },
        span: 0..10,
    };
    assert_eq!(prompt.span, 0..10);

    let tool = ToolDecl {
        name,
        params: vec![],
        return_type: type_id,
        body: ToolBody::Empty(0..0),
        span: 0..10,
    };
    assert_eq!(tool.span, 0..10);

    let agent = AgentDecl {
        name,
        fields: vec![],
        span: 0..10,
    };
    assert_eq!(agent.span, 0..10);

    let let_decl = LetDecl {
        name,
        type_expr: type_id,
        value: expr_id,
        span: 0..10,
    };
    assert_eq!(let_decl.span, 0..10);

    let param = Param {
        name,
        type_expr: type_id,
        default: None,
        span: 0..5,
    };
    assert_eq!(param.span, 0..5);

    let program = Program {
        declarations: vec![],
        span: 0..100,
    };
    assert_eq!(program.span, 0..100);
}

#[test]
fn index_impl_for_expr_id() {
    let mut ast = Ast::new();
    let id = ast.alloc_expr(Expr::IntLit(0..1));
    match &ast[id] {
        Expr::IntLit(span) => assert_eq!(span, &(0..1)),
        _ => panic!("Expected IntLit"),
    }
}

#[test]
fn index_impl_for_type_expr_id() {
    let mut interner = eaml_lexer::Interner::new();
    let name = interner.intern("int");
    let mut ast = Ast::new();
    let id = ast.alloc_type_expr(TypeExpr::Named(name, 0..3));
    match &ast[id] {
        TypeExpr::Named(_, span) => assert_eq!(span, &(0..3)),
        _ => panic!("Expected Named"),
    }
}

// === Task 2: Parser struct with cursor, helpers, and error recovery ===

/// Helper to create a Parser from source text via the lexer.
fn make_parser(source: &str) -> eaml_parser::parser::Parser {
    let lex_output = eaml_lexer::lex(source);
    eaml_parser::parser::Parser::new(
        source.to_string(),
        lex_output.tokens,
        lex_output.interner,
        lex_output.diagnostics,
    )
}

#[test]
fn parser_new_initializes_at_position_zero() {
    let parser = make_parser("model Foo {}");
    assert_eq!(parser.peek(), TokenKind::KwModel);
}

#[test]
fn peek_returns_current_without_advancing() {
    let parser = make_parser("model Foo {}");
    let first = parser.peek();
    let second = parser.peek();
    assert_eq!(first, second);
    assert_eq!(first, TokenKind::KwModel);
}

#[test]
fn advance_returns_current_and_moves_forward() {
    let mut parser = make_parser("model Foo {}");
    let tok = parser.advance().clone();
    assert_eq!(tok.kind, TokenKind::KwModel);
    assert!(parser.at_ident()); // "Foo" is Ident
}

#[test]
fn at_lparen_returns_true_when_positioned_at_lparen() {
    let parser = make_parser("(x)");
    assert!(parser.at(TokenKind::LParen));
}

#[test]
fn at_ident_returns_true_for_any_ident() {
    let mut parser = make_parser("foo bar");
    assert!(parser.at_ident());
    parser.advance();
    assert!(parser.at_ident());
}

#[test]
fn eat_advances_on_match_returns_false_otherwise() {
    let mut parser = make_parser("; x");
    assert!(parser.eat(TokenKind::Semicolon));
    assert!(!parser.eat(TokenKind::Semicolon));
    assert!(parser.at_ident());
}

#[test]
fn expect_advances_on_match_emits_syn050_on_mismatch() {
    let mut parser = make_parser("{ x");
    let result = parser.expect(TokenKind::LBrace);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0..1);

    let result2 = parser.expect(TokenKind::LBrace);
    assert!(result2.is_err());
}

#[test]
fn expect_ident_returns_spur_on_ident() {
    let mut parser = make_parser("foo");
    let result = parser.expect_ident();
    assert!(result.is_ok());
    let (spur, span) = result.unwrap();
    assert_eq!(span, 0..3);
    assert_eq!(parser.resolve_spur(spur), "foo");
}

#[test]
fn expect_ident_emits_syn050_on_non_ident() {
    let mut parser = make_parser("{");
    let result = parser.expect_ident();
    assert!(result.is_err());
}

#[test]
fn at_contextual_user_matches_ident_user() {
    let mut parser = make_parser("user other");
    assert!(parser.at_contextual("user"));
    assert!(!parser.at_contextual("other"));
    parser.advance();
    assert!(parser.at_contextual("other"));
    assert!(!parser.at_contextual("user"));
}

#[test]
fn synchronize_skips_to_next_declaration_keyword() {
    let mut parser = make_parser("x y z model Foo {}");
    parser.synchronize();
    assert_eq!(parser.peek(), TokenKind::KwModel);
}

#[test]
fn synchronize_respects_brace_depth() {
    let mut parser = make_parser("{ schema Inner } model Outer {}");
    parser.synchronize();
    assert_eq!(parser.peek(), TokenKind::KwModel);
}

#[test]
fn parse_function_returns_parse_output() {
    let output = eaml_parser::parse("");
    assert!(output.program.declarations.is_empty());
    assert!(output.ast.models.is_empty());
}

#[test]
fn parse_function_with_source() {
    let output = eaml_parser::parse("model Foo {}");
    assert!(!output.program.declarations.is_empty());
}
