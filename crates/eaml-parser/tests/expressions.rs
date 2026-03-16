//! Tests for Pratt expression parsing.

#[allow(dead_code)]
mod test_helpers;

use eaml_parser::ast::*;
use eaml_parser::parser::Parser;
use test_helpers::format_expr;

/// Helper: parses an expression from source text.
fn parse_expr(
    source: &str,
) -> (
    Ast,
    ExprId,
    Vec<eaml_errors::Diagnostic>,
    eaml_lexer::Interner,
) {
    let lex_output = eaml_lexer::lex(source);
    let mut parser = Parser::new(
        source.to_string(),
        lex_output.tokens,
        lex_output.interner,
        lex_output.diagnostics,
    );
    let id = parser.parse_expr(0);
    let (ast, diagnostics, interner) = parser.finish_with_interner();
    (ast, id, diagnostics, interner)
}

fn parse_and_format(source: &str) -> String {
    let (ast, id, _diags, interner) = parse_expr(source);
    format_expr(&ast, id, &interner)
}

// === Literals ===

#[test]
fn expr_int_lit() {
    insta::assert_snapshot!(parse_and_format("42"), @"IntLit(0..2)");
}

#[test]
fn expr_float_lit() {
    insta::assert_snapshot!(parse_and_format("3.14"), @"FloatLit(0..4)");
}

#[test]
fn expr_bool_true() {
    insta::assert_snapshot!(parse_and_format("true"), @"BoolLit(true, 0..4)");
}

#[test]
fn expr_null() {
    insta::assert_snapshot!(parse_and_format("null"), @"NullLit(0..4)");
}

#[test]
fn expr_ident() {
    insta::assert_snapshot!(parse_and_format("x"), @"Ident(x, 0..1)");
}

// === Binary operations ===

#[test]
fn expr_add() {
    let result = parse_and_format("1 + 2");
    insta::assert_snapshot!(result);
}

#[test]
fn expr_precedence_add_mul() {
    // 1 + 2 * 3 should group multiplication tighter
    let result = parse_and_format("1 + 2 * 3");
    insta::assert_snapshot!(result);
}

#[test]
fn expr_logical_and_or() {
    // a && b || c -- AND binds tighter than OR
    let result = parse_and_format("a && b || c");
    insta::assert_snapshot!(result);
}

#[test]
fn expr_equality() {
    let result = parse_and_format("a == b");
    insta::assert_snapshot!(result);
}

// === Unary operations ===

#[test]
fn expr_neg() {
    let result = parse_and_format("-x");
    insta::assert_snapshot!(result);
}

#[test]
fn expr_not() {
    let result = parse_and_format("!flag");
    insta::assert_snapshot!(result);
}

// === Await ===

#[test]
fn expr_await_fn_call() {
    let result = parse_and_format("await f()");
    insta::assert_snapshot!(result);
}

// === Postfix ===

#[test]
fn expr_field_access() {
    let result = parse_and_format("obj.field");
    insta::assert_snapshot!(result);
}

#[test]
fn expr_fn_call() {
    let result = parse_and_format("f(x, y)");
    insta::assert_snapshot!(result);
}

#[test]
fn expr_fn_call_named_arg() {
    let result = parse_and_format("f(name: x)");
    insta::assert_snapshot!(result);
}

#[test]
fn expr_index() {
    let result = parse_and_format("arr[0]");
    insta::assert_snapshot!(result);
}

#[test]
fn expr_method_call() {
    // obj.method(x) -> FnCall(FieldAccess(obj, method), [x])
    let result = parse_and_format("obj.method(x)");
    insta::assert_snapshot!(result);
}

// === Grouping ===

#[test]
fn expr_paren() {
    let result = parse_and_format("(a + b)");
    insta::assert_snapshot!(result);
}

// === Template string as expression ===

#[test]
fn expr_template_string() {
    let result = parse_and_format(r#""hello""#);
    insta::assert_snapshot!(result);
}
