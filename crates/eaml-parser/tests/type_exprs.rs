//! Tests for type expression parsing.

use eaml_parser::ast::*;
use eaml_parser::parser::Parser;

/// Helper: parses a type expression from source text.
/// Returns the Ast and the TypeExprId of the parsed type expression.
fn parse_type_expr(source: &str) -> (Ast, TypeExprId, Vec<eaml_errors::Diagnostic>) {
    let lex_output = eaml_lexer::lex(source);
    let mut parser = Parser::new(
        source.to_string(),
        lex_output.tokens,
        lex_output.interner,
        lex_output.diagnostics,
    );
    let id = parser.parse_type_expr();
    let (ast, diagnostics) = parser.finish();
    (ast, id, diagnostics)
}

/// Helper: format a TypeExpr tree recursively for snapshot testing.
fn format_type_expr(ast: &Ast, id: TypeExprId, interner: &eaml_lexer::Interner) -> String {
    match &ast[id] {
        TypeExpr::Named(spur, span) => {
            format!("Named({}, {:?})", interner.resolve(spur), span)
        }
        TypeExpr::Bounded { base, params, span } => {
            let params_str: Vec<String> = params
                .iter()
                .map(|p| {
                    let name_str = match &p.name {
                        Some(spur) => format!("Some({})", interner.resolve(spur)),
                        None => "None".to_string(),
                    };
                    format!(
                        "BoundParam(name: {}, value_span: {:?}, span: {:?})",
                        name_str, p.value_span, p.span
                    )
                })
                .collect();
            format!(
                "Bounded(base: {}, params: [{}], span: {:?})",
                interner.resolve(base),
                params_str.join(", "),
                span
            )
        }
        TypeExpr::Array(inner, span) => {
            format!(
                "Array({}, {:?})",
                format_type_expr(ast, *inner, interner),
                span
            )
        }
        TypeExpr::Optional(inner, span) => {
            format!(
                "Optional({}, {:?})",
                format_type_expr(ast, *inner, interner),
                span
            )
        }
        TypeExpr::LiteralUnion { members, span } => {
            let members_str: Vec<String> = members.iter().map(|m| format!("{:?}", m)).collect();
            format!("LiteralUnion([{}], {:?})", members_str.join(", "), span)
        }
        TypeExpr::Grouped(inner, span) => {
            format!(
                "Grouped({}, {:?})",
                format_type_expr(ast, *inner, interner),
                span
            )
        }
        TypeExpr::Error(span) => {
            format!("Error({:?})", span)
        }
    }
}

fn parse_and_format(source: &str) -> String {
    let lex_output_for_interner = eaml_lexer::lex(source);
    let interner_copy = lex_output_for_interner.interner;
    let (ast, id, _diags) = parse_type_expr(source);
    format_type_expr(&ast, id, &interner_copy)
}

// === Primitive / Named types ===

#[test]
fn type_expr_string() {
    insta::assert_snapshot!(parse_and_format("string"), @"Named(string, 0..6)");
}

#[test]
fn type_expr_float() {
    insta::assert_snapshot!(parse_and_format("float"), @"Named(float, 0..5)");
}

#[test]
fn type_expr_named_schema() {
    insta::assert_snapshot!(parse_and_format("MySchema"), @"Named(MySchema, 0..8)");
}

// === Bounded types ===

#[test]
fn type_expr_bounded_positional() {
    let result = parse_and_format("float<0.0, 1.0>");
    insta::assert_snapshot!(result);
}

#[test]
fn type_expr_bounded_named() {
    let result = parse_and_format("float<min: 0.0, max: 1.0>");
    insta::assert_snapshot!(result);
}

#[test]
fn type_expr_bounded_single_named() {
    let result = parse_and_format("string<max: 200>");
    insta::assert_snapshot!(result);
}

#[test]
fn type_expr_bounded_int_values() {
    let result = parse_and_format("int<min: 0, max: 100>");
    insta::assert_snapshot!(result);
}

// === Type modifiers ===

#[test]
fn type_expr_array() {
    let result = parse_and_format("string[]");
    insta::assert_snapshot!(result);
}

#[test]
fn type_expr_optional() {
    let result = parse_and_format("string?");
    insta::assert_snapshot!(result);
}

#[test]
fn type_expr_array_then_optional() {
    // string[]? = Optional(Array(Named("string")))
    let result = parse_and_format("string[]?");
    insta::assert_snapshot!(result);
}

#[test]
fn type_expr_optional_then_array() {
    // string?[] = Array(Optional(Named("string")))
    let result = parse_and_format("string?[]");
    insta::assert_snapshot!(result);
}

#[test]
fn type_expr_optional_array_optional() {
    // string?[]? = Optional(Array(Optional(Named("string"))))
    let result = parse_and_format("string?[]?");
    insta::assert_snapshot!(result);
}

// === Literal unions ===

#[test]
fn type_expr_literal_union_two() {
    let result = parse_and_format(r#""positive" | "negative""#);
    insta::assert_snapshot!(result);
}

#[test]
fn type_expr_literal_union_four() {
    let result = parse_and_format(r#""low" | "medium" | "high" | "critical""#);
    insta::assert_snapshot!(result);
}

// === Grouped types ===

#[test]
fn type_expr_grouped() {
    let result = parse_and_format("(string)");
    insta::assert_snapshot!(result);
}

// === Error cases ===

#[test]
fn type_expr_multi_dim_array_syn042() {
    let (_, _, diags) = parse_type_expr("string[][]");
    assert!(
        diags
            .iter()
            .any(|d| d.code == eaml_errors::ErrorCode::Syn042),
        "expected SYN042 for multi-dimensional array, got: {:?}",
        diags.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
}
