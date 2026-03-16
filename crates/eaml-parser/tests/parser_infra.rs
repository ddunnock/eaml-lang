//! Tests for parser infrastructure: AST types, arenas, and parser cursor.

use eaml_parser::ast::*;

// === Task 1: AST node types, typed arenas, ID newtypes ===

#[test]
fn expr_id_equality() {
    assert_ne!(ExprId(0), ExprId(1));
    assert_eq!(ExprId(0), ExprId(0));
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
fn alloc_expr_returns_correct_id() {
    let mut ast = Ast::new();
    let id = ast.alloc_expr(Expr::IntLit(0..1));
    assert_eq!(id, ExprId(0));

    let id2 = ast.alloc_expr(Expr::NullLit(1..2));
    assert_eq!(id2, ExprId(1));
}

#[test]
fn alloc_type_expr_returns_type_expr_id() {
    let mut interner = eaml_lexer::Interner::new();
    let name = interner.intern("string");
    let mut ast = Ast::new();
    let id = ast.alloc_type_expr(TypeExpr::Named(name, 0..6));
    assert_eq!(id, TypeExprId(0));
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
        cap_spans: vec![],
        span: 0..10,
    });
    assert_eq!(id, ModelDeclId(0));
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
    assert_eq!(id, SchemaDeclId(0));
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
    assert_eq!(id, PromptDeclId(0));
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
    assert_eq!(id, ToolDeclId(0));
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
    assert_eq!(id, AgentDeclId(0));
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
    assert_eq!(id, ImportDeclId(0));
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
    assert_eq!(id, LetDeclId(0));
}

#[test]
fn decl_id_preserves_typed_id() {
    let decl = DeclId::Model(ModelDeclId(0));
    match decl {
        DeclId::Model(id) => assert_eq!(id, ModelDeclId(0)),
        _ => panic!("Expected Model variant"),
    }
}

#[test]
fn every_ast_node_has_span() {
    // Verify at compile time that span fields exist by constructing each type.
    // The key assertion is that this compiles and the span values are accessible.
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
        cap_spans: vec![],
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
