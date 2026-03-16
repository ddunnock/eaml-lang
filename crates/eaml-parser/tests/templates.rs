//! Tests for template string parsing.

use eaml_parser::ast::*;
use eaml_parser::parser::Parser;

/// Helper: parses a template string from source text.
fn parse_template(
    source: &str,
) -> (
    Ast,
    TemplateString,
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
    let ts = parser.parse_template_string();
    let (ast, diagnostics, interner) = parser.finish_with_interner();
    (ast, ts, diagnostics, interner)
}

/// Format a template string for snapshot testing.
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
    format!("TemplateString([{}], {:?})", parts.join(", "), ts.span)
}

fn format_expr(ast: &Ast, id: ExprId, interner: &eaml_lexer::Interner) -> String {
    match &ast[id] {
        Expr::IntLit(span) => format!("IntLit({:?})", span),
        Expr::FloatLit(span) => format!("FloatLit({:?})", span),
        Expr::BoolLit(val, span) => format!("BoolLit({}, {:?})", val, span),
        Expr::NullLit(span) => format!("NullLit({:?})", span),
        Expr::Ident(spur, span) => format!("Ident({}, {:?})", interner.resolve(spur), span),
        Expr::BinaryOp {
            left, op, right, ..
        } => {
            format!(
                "BinaryOp({}, {:?}, {})",
                format_expr(ast, *left, interner),
                op,
                format_expr(ast, *right, interner),
            )
        }
        Expr::TemplateStr(ts) => format!("TemplateStr({})", format_template(ast, ts, interner)),
        Expr::Error(span) => format!("Error({:?})", span),
        _ => format!("OtherExpr"),
    }
}

fn parse_and_format(source: &str) -> String {
    let (ast, ts, _diags, interner) = parse_template(source);
    format_template(&ast, &ts, &interner)
}

#[test]
fn template_plain_hello() {
    let result = parse_and_format(r#""hello""#);
    insta::assert_snapshot!(result);
}

#[test]
fn template_with_interpolation() {
    let result = parse_and_format(r#""hello {name}""#);
    insta::assert_snapshot!(result);
}

#[test]
fn template_expression_interpolation() {
    let result = parse_and_format(r#""{x + y}""#);
    insta::assert_snapshot!(result);
}

#[test]
fn template_no_interp() {
    let result = parse_and_format(r#""no interp""#);
    insta::assert_snapshot!(result);
}

#[test]
fn template_empty_string() {
    let result = parse_and_format(r#""""#);
    insta::assert_snapshot!(result);
}
