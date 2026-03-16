//! Tests for template string parsing.

#[allow(dead_code)]
mod test_helpers;

use eaml_parser::ast::*;
use eaml_parser::parser::Parser;
use test_helpers::format_template;

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

/// Format a template string with span for snapshot testing.
fn format_template_with_span(
    ast: &Ast,
    ts: &TemplateString,
    interner: &eaml_lexer::Interner,
) -> String {
    let inner = format_template(ast, ts, interner);
    format!("TemplateString({}, {:?})", inner, ts.span)
}

fn parse_and_format(source: &str) -> String {
    let (ast, ts, _diags, interner) = parse_template(source);
    format_template_with_span(&ast, &ts, &interner)
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
