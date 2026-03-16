//! Tests for Pratt expression parsing.

use eaml_parser::ast::*;
use eaml_parser::parser::Parser;

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

/// Format an expression tree for snapshot testing.
fn format_expr(ast: &Ast, id: ExprId, interner: &eaml_lexer::Interner) -> String {
    match &ast[id] {
        Expr::IntLit(span) => format!("IntLit({:?})", span),
        Expr::FloatLit(span) => format!("FloatLit({:?})", span),
        Expr::StringLit(ts) => format!("StringLit({})", format_template(ast, ts, interner)),
        Expr::BoolLit(val, span) => format!("BoolLit({}, {:?})", val, span),
        Expr::NullLit(span) => format!("NullLit({:?})", span),
        Expr::Ident(spur, span) => format!("Ident({}, {:?})", interner.resolve(spur), span),
        Expr::BinaryOp {
            left,
            op,
            right,
            span,
        } => {
            format!(
                "BinaryOp({}, {:?}, {}, {:?})",
                format_expr(ast, *left, interner),
                op,
                format_expr(ast, *right, interner),
                span
            )
        }
        Expr::UnaryOp { op, operand, span } => {
            format!(
                "UnaryOp({:?}, {}, {:?})",
                op,
                format_expr(ast, *operand, interner),
                span
            )
        }
        Expr::Await { operand, span } => {
            format!(
                "Await({}, {:?})",
                format_expr(ast, *operand, interner),
                span
            )
        }
        Expr::FieldAccess {
            object,
            field,
            span,
        } => {
            format!(
                "FieldAccess({}, {}, {:?})",
                format_expr(ast, *object, interner),
                interner.resolve(field),
                span
            )
        }
        Expr::FnCall { callee, args, span } => {
            let args_str: Vec<String> = args
                .iter()
                .map(|a| {
                    let name_str = match &a.name {
                        Some(spur) => format!("{}:", interner.resolve(spur)),
                        None => String::new(),
                    };
                    format!("{}{}", name_str, format_expr(ast, a.value, interner))
                })
                .collect();
            format!(
                "FnCall({}, [{}], {:?})",
                format_expr(ast, *callee, interner),
                args_str.join(", "),
                span
            )
        }
        Expr::Index {
            object,
            index,
            span,
        } => {
            format!(
                "Index({}, {}, {:?})",
                format_expr(ast, *object, interner),
                format_expr(ast, *index, interner),
                span
            )
        }
        Expr::Paren { inner, span } => {
            format!("Paren({}, {:?})", format_expr(ast, *inner, interner), span)
        }
        Expr::TemplateStr(ts) => format!("TemplateStr({})", format_template(ast, ts, interner)),
        Expr::Error(span) => format!("Error({:?})", span),
        Expr::If { .. } => "If(...)".to_string(),
        Expr::Return { .. } => "Return(...)".to_string(),
        Expr::Let { .. } => "Let(...)".to_string(),
    }
}

fn format_template(ast: &Ast, ts: &TemplateString, interner: &eaml_lexer::Interner) -> String {
    let parts: Vec<String> = ts
        .parts
        .iter()
        .map(|p| match p {
            TemplatePart::Text(span) => format!("Text({:?})", span),
            TemplatePart::Interpolation(expr_id, span) => {
                format!(
                    "Interp({}, {:?})",
                    format_expr(ast, *expr_id, interner),
                    span
                )
            }
        })
        .collect();
    format!("[{}]", parts.join(", "))
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
