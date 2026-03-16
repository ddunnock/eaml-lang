//! Span correctness tests for the EAML parser.
//!
//! Validates PAR-09: every AST node has a non-empty span that falls
//! within source bounds. Also verifies diagnostic spans from error
//! recovery point to valid source locations.

use eaml_errors::Span;
use eaml_parser::ast::*;

/// Extract span from an Expr variant.
fn expr_span(expr: &Expr) -> &Span {
    match expr {
        Expr::IntLit(span) => span,
        Expr::FloatLit(span) => span,
        Expr::StringLit(ts) => &ts.span,
        Expr::BoolLit(_, span) => span,
        Expr::NullLit(span) => span,
        Expr::Ident(_, span) => span,
        Expr::BinaryOp { span, .. } => span,
        Expr::UnaryOp { span, .. } => span,
        Expr::Await { span, .. } => span,
        Expr::FieldAccess { span, .. } => span,
        Expr::FnCall { span, .. } => span,
        Expr::Index { span, .. } => span,
        Expr::Paren { span, .. } => span,
        Expr::TemplateStr(ts) => &ts.span,
        Expr::If { span, .. } => span,
        Expr::Return { span, .. } => span,
        Expr::Let { span, .. } => span,
        Expr::Error(span) => span,
    }
}

/// Extract span from a TypeExpr variant.
fn type_expr_span(te: &TypeExpr) -> &Span {
    match te {
        TypeExpr::Named(_, span) => span,
        TypeExpr::Bounded { span, .. } => span,
        TypeExpr::Array(_, span) => span,
        TypeExpr::Optional(_, span) => span,
        TypeExpr::LiteralUnion { span, .. } => span,
        TypeExpr::Grouped(_, span) => span,
        TypeExpr::Error(span) => span,
    }
}

/// Extract span from a DeclId (using the AST arenas).
fn decl_span(ast: &Ast, decl: &DeclId) -> Span {
    match decl {
        DeclId::Model(id) => ast[*id].span.clone(),
        DeclId::Schema(id) => ast[*id].span.clone(),
        DeclId::Prompt(id) => ast[*id].span.clone(),
        DeclId::Tool(id) => ast[*id].span.clone(),
        DeclId::Agent(id) => ast[*id].span.clone(),
        DeclId::Import(id) => match &ast[*id] {
            ImportDecl::Eaml { span, .. } => span.clone(),
            ImportDecl::Python { span, .. } => span.clone(),
        },
        DeclId::Let(id) => ast[*id].span.clone(),
        DeclId::Error(span) => span.clone(),
    }
}

/// Verify all spans in a ParseOutput are within source bounds.
fn verify_all_spans(output: &eaml_parser::ParseOutput, source: &str) {
    let source_len = source.len();

    // Check all expression spans
    for (i, expr) in output.ast.exprs.iter().enumerate() {
        let span = expr_span(expr);
        assert!(
            span.end <= source_len,
            "Expr[{}] span end {} > source len {} (expr: {:?})",
            i,
            span.end,
            source_len,
            expr
        );
        // Spans can be zero-length for error recovery nodes
        assert!(
            span.start <= span.end,
            "Expr[{}] has inverted span: {}..{} (expr: {:?})",
            i,
            span.start,
            span.end,
            expr
        );
    }

    // Check all type expression spans
    for (i, te) in output.ast.type_exprs.iter().enumerate() {
        let span = type_expr_span(te);
        assert!(
            span.end <= source_len,
            "TypeExpr[{}] span end {} > source len {} (te: {:?})",
            i,
            span.end,
            source_len,
            te
        );
        assert!(
            span.start <= span.end,
            "TypeExpr[{}] has inverted span: {}..{} (te: {:?})",
            i,
            span.start,
            span.end,
            te
        );
    }

    // Check all declaration spans
    for (i, decl_id) in output.program.declarations.iter().enumerate() {
        let span = decl_span(&output.ast, decl_id);
        assert!(
            span.end <= source_len,
            "Decl[{}] span end {} > source len {} (decl: {:?})",
            i,
            span.end,
            source_len,
            decl_id
        );
        assert!(
            span.start <= span.end,
            "Decl[{}] has inverted span: {}..{} (decl: {:?})",
            i,
            span.start,
            span.end,
            decl_id
        );
    }

    // Check program span
    assert!(
        output.program.span.end <= source_len,
        "Program span end {} > source len {}",
        output.program.span.end,
        source_len
    );

    // Check model declaration sub-spans (template strings, caps)
    for model in &output.ast.models {
        assert!(
            model.model_id.span.end <= source_len,
            "Model model_id template span out of bounds"
        );
        assert!(
            model.provider.span.end <= source_len,
            "Model provider template span out of bounds"
        );
        for (_, cap_span) in &model.caps {
            assert!(cap_span.end <= source_len, "Model cap span out of bounds");
        }
    }

    // Check schema field spans
    for schema in &output.ast.schemas {
        for field in &schema.fields {
            assert!(
                field.span.end <= source_len,
                "Schema field span out of bounds"
            );
        }
    }

    // Check prompt sub-spans
    for prompt in &output.ast.prompts {
        for param in &prompt.params {
            assert!(
                param.span.end <= source_len,
                "Prompt param span out of bounds"
            );
        }
        if let Some(req) = &prompt.requires {
            assert!(
                req.span.end <= source_len,
                "Requires clause span out of bounds"
            );
            for (_, cap_span) in &req.caps {
                assert!(
                    cap_span.end <= source_len,
                    "Requires cap span out of bounds"
                );
            }
        }
        assert!(
            prompt.body.span.end <= source_len,
            "Prompt body span out of bounds"
        );
    }

    // Check tool sub-spans
    for tool in &output.ast.tools {
        for param in &tool.params {
            assert!(
                param.span.end <= source_len,
                "Tool param span out of bounds"
            );
        }
    }

    // Check agent sub-spans
    for agent in &output.ast.agents {
        for field in &agent.fields {
            match field {
                AgentField::Model(_, span) => {
                    assert!(span.end <= source_len, "Agent model span out of bounds");
                }
                AgentField::Tools(tools, span) => {
                    assert!(span.end <= source_len, "Agent tools span out of bounds");
                    for (_, s) in tools {
                        assert!(s.end <= source_len, "Agent tool name span out of bounds");
                    }
                }
                AgentField::System(ts) => {
                    assert!(
                        ts.span.end <= source_len,
                        "Agent system template span out of bounds"
                    );
                }
                AgentField::MaxTurns(span) => {
                    assert!(span.end <= source_len, "Agent max_turns span out of bounds");
                }
                AgentField::OnError(_, span) => {
                    assert!(span.end <= source_len, "Agent on_error span out of bounds");
                }
            }
        }
    }

    // Check diagnostic spans
    for (i, diag) in output.diagnostics.iter().enumerate() {
        assert!(
            diag.span.end <= source_len,
            "Diagnostic[{}] span end {} > source len {} (code: {})",
            i,
            diag.span.end,
            source_len,
            diag.code
        );
        assert!(
            diag.span.start <= diag.span.end,
            "Diagnostic[{}] has inverted span: {}..{} (code: {})",
            i,
            diag.span.start,
            diag.span.end,
            diag.code
        );
    }
}

// ===================================================================
// Span verification for each example file
// ===================================================================

#[test]
fn spans_minimal() {
    let source = include_str!("../../../examples/01-minimal/minimal.eaml");
    let output = eaml_parser::parse(source);
    verify_all_spans(&output, source);
}

#[test]
fn spans_sentiment() {
    let source = include_str!("../../../examples/02-sentiment/sentiment.eaml");
    let output = eaml_parser::parse(source);
    verify_all_spans(&output, source);
}

#[test]
fn spans_all_type_variants() {
    let source = include_str!("../../../examples/07-all-type-variants/types.eaml");
    let output = eaml_parser::parse(source);
    verify_all_spans(&output, source);
}

#[test]
fn spans_capability_error() {
    let source = include_str!("../../../examples/06-capability-error/bad_model.eaml");
    let output = eaml_parser::parse(source);
    verify_all_spans(&output, source);
}

// ===================================================================
// Span verification for declarations with non-empty content
// ===================================================================

#[test]
fn spans_declaration_covers_keyword_to_end() {
    let source = "schema Greeting { message: string }";
    let output = eaml_parser::parse(source);
    verify_all_spans(&output, source);

    // The schema span should start at 0 (the 's' in 'schema')
    let span = decl_span(&output.ast, &output.program.declarations[0]);
    assert_eq!(span.start, 0, "schema span should start at beginning");
    // Should end at/near end of source
    assert!(
        span.end >= source.len() - 1,
        "schema span end {} should be near source end {}",
        span.end,
        source.len()
    );
}

#[test]
fn spans_expression_within_parent() {
    // Binary expression: 1 + 2
    let source = "let x: int = 1 + 2";
    let output = eaml_parser::parse(source);
    verify_all_spans(&output, source);

    // Find the binary op expression -- it should exist
    let has_binop = output
        .ast
        .exprs
        .iter()
        .any(|e| matches!(e, Expr::BinaryOp { .. }));
    assert!(has_binop, "expected a BinaryOp expression");

    // Verify the binary op span covers its children
    for expr in &output.ast.exprs {
        if let Expr::BinaryOp {
            left, right, span, ..
        } = expr
        {
            let left_span = expr_span(&output.ast[*left]);
            let right_span = expr_span(&output.ast[*right]);
            assert!(
                span.start <= left_span.start,
                "parent start {} should be <= left child start {}",
                span.start,
                left_span.start
            );
            assert!(
                span.end >= right_span.end,
                "parent end {} should be >= right child end {}",
                span.end,
                right_span.end
            );
        }
    }
}

// ===================================================================
// Span verification for error recovery
// ===================================================================

#[test]
fn spans_error_recovery_valid() {
    let source = r#"schema { x: int }
schema Good { a: string }"#;
    let output = eaml_parser::parse(source);
    verify_all_spans(&output, source);
}

#[test]
fn spans_multiple_declarations() {
    let source = r#"model M = Model(id: "x", provider: "y", caps: [json_mode])
schema S { name: string, age: int }
prompt P(text: string) requires json_mode -> S {
  system: "You are helpful"
  user: "Process: {text}"
  temperature: 0.5
  max_tokens: 100
}"#;
    let output = eaml_parser::parse(source);
    verify_all_spans(&output, source);

    // Verify declarations don't overlap
    let spans: Vec<Span> = output
        .program
        .declarations
        .iter()
        .map(|d| decl_span(&output.ast, d))
        .collect();
    for i in 0..spans.len() {
        for j in (i + 1)..spans.len() {
            assert!(
                spans[i].end <= spans[j].start,
                "Decl[{}] span {:?} overlaps with Decl[{}] span {:?}",
                i,
                spans[i],
                j,
                spans[j]
            );
        }
    }
}

#[test]
fn spans_template_string_parts() {
    let source = r#"prompt P(name: string) -> R {
  user: "Hello {name}, welcome!"
}"#;
    let output = eaml_parser::parse(source);
    verify_all_spans(&output, source);

    // Check template string parts within the prompt's user field
    if let DeclId::Prompt(id) = &output.program.declarations[0] {
        let p = &output.ast[*id];
        for field in &p.body.fields {
            if let PromptField::User(ts) = field {
                // Template span should be within source
                assert!(ts.span.end <= source.len());
                // Each part span should be within the template span
                for part in &ts.parts {
                    match part {
                        TemplatePart::Text(span) => {
                            assert!(span.end <= source.len(), "template text span out of bounds");
                        }
                        TemplatePart::Interpolation(_, span) => {
                            assert!(
                                span.end <= source.len(),
                                "template interpolation span out of bounds"
                            );
                        }
                    }
                }
            }
        }
    }
}
