//! Shared test helpers for eaml-parser tests.

use eaml_parser::ast::*;
use eaml_parser::parser::Parser;

/// Helper: creates a Parser from source text via the lexer.
pub fn make_parser(source: &str) -> Parser {
    let lex_output = eaml_lexer::lex(source);
    Parser::new(
        source.to_string(),
        lex_output.tokens,
        lex_output.interner,
        lex_output.diagnostics,
    )
}

/// Helper: parses a full program from source text.
pub fn parse_program(source: &str) -> eaml_parser::ParseOutput {
    eaml_parser::parse(source)
}

/// Parse an example source and assert zero error/fatal diagnostics.
pub fn parse_example(source: &str) -> eaml_parser::ParseOutput {
    let output = eaml_parser::parse(source);
    let errors: Vec<_> = output
        .diagnostics
        .iter()
        .filter(|d| {
            d.severity == eaml_errors::Severity::Error || d.severity == eaml_errors::Severity::Fatal
        })
        .collect();
    assert!(
        errors.is_empty(),
        "Expected no error diagnostics but got {}:\n{}",
        errors.len(),
        errors
            .iter()
            .map(|d| format!("  {}: {} (at {:?})", d.code, d.message, d.span))
            .collect::<Vec<_>>()
            .join("\n")
    );
    output
}

/// Count error-severity diagnostics.
pub fn error_count(output: &eaml_parser::ParseOutput) -> usize {
    output
        .diagnostics
        .iter()
        .filter(|d| {
            d.severity == eaml_errors::Severity::Error || d.severity == eaml_errors::Severity::Fatal
        })
        .count()
}

/// Check if any diagnostic has the given error code.
pub fn has_code(output: &eaml_parser::ParseOutput, code: eaml_errors::ErrorCode) -> bool {
    output.diagnostics.iter().any(|d| d.code == code)
}

/// Format an expression tree recursively for snapshot testing.
pub fn format_expr(ast: &Ast, id: ExprId, interner: &eaml_lexer::Interner) -> String {
    match &ast[id] {
        Expr::IntLit(span) => format!("IntLit({:?})", span),
        Expr::FloatLit(span) => format!("FloatLit({:?})", span),
        Expr::StringLit(ts) => format!("StringLit({})", format_template(ast, ts, interner)),
        Expr::BoolLit(val, span) => format!("BoolLit({}, {:?})", val, span),
        Expr::NullLit(span) => format!("NullLit({:?})", span),
        Expr::Ident(spur, span) => format!("Ident({}, {:?})", interner.resolve(*spur), span),
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
                interner.resolve(*field),
                span
            )
        }
        Expr::FnCall { callee, args, span } => {
            let args_str: Vec<String> = args
                .iter()
                .map(|a| {
                    let name_str = match &a.name {
                        Some(spur) => format!("{}:", interner.resolve(*spur)),
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

/// Format a template string for snapshot testing.
pub fn format_template(ast: &Ast, ts: &TemplateString, interner: &eaml_lexer::Interner) -> String {
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
