//! Declaration emitters for schemas, models, and let bindings.
//!
//! Produces Pydantic BaseModel classes from schema declarations,
//! UPPER_SNAKE_CASE config dicts from model declarations, and
//! typed variable assignments from let bindings.

use eaml_lexer::Interner;
use eaml_parser::ast::*;
use eaml_semantic::type_checker::TypeAnnotations;

use crate::names::to_config_name;
use crate::types::{emit_field_line, emit_type_annotation, is_optional, ImportTracker};
use crate::writer::CodeWriter;

/// Emits a Pydantic BaseModel class from a schema declaration.
///
/// Schema names stay PascalCase per CONTEXT.md locked decision.
/// Each field is emitted as a Pydantic field declaration with appropriate
/// type annotations and constraints (bounded types, optional defaults).
pub fn emit_schema(
    schema: &SchemaDecl,
    ast: &Ast,
    interner: &Interner,
    type_annotations: &TypeAnnotations,
    source: &str,
    writer: &mut CodeWriter,
    imports: &mut ImportTracker,
) {
    imports.need_base_model();

    let name = interner.resolve(&schema.name);
    writer.writeln(&format!("class {name}(BaseModel):"));
    writer.indent();

    if schema.fields.is_empty() {
        writer.writeln("pass");
    } else {
        for field in &schema.fields {
            let resolved = &type_annotations.type_exprs[&field.type_expr];
            imports.track_type(resolved);

            // Check if the type expression is bounded -- if so, we need Field import
            if matches!(&ast[field.type_expr], TypeExpr::Bounded { .. }) {
                imports.need_field();
            }

            let field_name = interner.resolve(&field.name);
            let type_expr = &ast[field.type_expr];
            let line = emit_field_line(field_name, resolved, type_expr, ast, interner, source);
            writer.writeln(&line);
        }
    }

    writer.dedent();
}

/// Emits a Python expression value from an AST expression.
///
/// Maps EAML literals to Python equivalents:
/// - IntLit/FloatLit: source text
/// - StringLit: Python string
/// - BoolLit: True/False
/// - NullLit: None
/// - Ident: variable name
/// - TemplateStr: f-string
fn emit_expr_value(expr_id: ExprId, ast: &Ast, interner: &Interner, source: &str) -> String {
    match &ast[expr_id] {
        Expr::IntLit(span) => source[span.clone()].to_string(),
        Expr::FloatLit(span) => source[span.clone()].to_string(),
        Expr::StringLit(ts) => emit_template_as_string(ts, ast, interner, source),
        Expr::BoolLit(true, _) => "True".to_string(),
        Expr::BoolLit(false, _) => "False".to_string(),
        Expr::NullLit(_) => "None".to_string(),
        Expr::Ident(spur, _) => interner.resolve(spur).to_string(),
        Expr::TemplateStr(ts) => emit_template_as_fstring(ts, ast, interner, source),
        _ => "None".to_string(),
    }
}

/// Emits a template string as a Python quoted string.
///
/// If there are no interpolations, emits a simple string.
/// If there are interpolations, emits an f-string.
fn emit_template_as_string(
    ts: &TemplateString,
    _ast: &Ast,
    _interner: &Interner,
    source: &str,
) -> String {
    let mut text = String::new();
    for part in &ts.parts {
        if let TemplatePart::Text(span) = part {
            text.push_str(&source[span.clone()]);
        }
    }
    format!("\"{text}\"")
}

/// Emits a template string as a Python f-string.
fn emit_template_as_fstring(
    ts: &TemplateString,
    ast: &Ast,
    interner: &Interner,
    source: &str,
) -> String {
    let mut parts = String::new();
    for part in &ts.parts {
        match part {
            TemplatePart::Text(span) => {
                parts.push_str(&source[span.clone()]);
            }
            TemplatePart::Interpolation(expr_id, _) => {
                parts.push('{');
                parts.push_str(&emit_expr_value(*expr_id, ast, interner, source));
                parts.push('}');
            }
        }
    }
    format!("f\"{parts}\"")
}

/// Emits a typed Python variable assignment from a let declaration.
pub fn emit_let(
    decl: &LetDecl,
    ast: &Ast,
    interner: &Interner,
    type_annotations: &TypeAnnotations,
    source: &str,
    writer: &mut CodeWriter,
    imports: &mut ImportTracker,
) {
    let name = interner.resolve(&decl.name);
    let resolved = &type_annotations.type_exprs[&decl.type_expr];
    imports.track_type(resolved);

    // For optional type let bindings, we need Optional import
    if is_optional(resolved) {
        imports.need_optional();
    }

    let annotation = emit_type_annotation(resolved, ast, interner);
    let value = emit_expr_value(decl.value, ast, interner, source);
    writer.writeln(&format!("{name}: {annotation} = {value}"));
}

/// Extracts plain text from a template string (ignoring interpolations).
fn extract_template_text(ts: &TemplateString, source: &str) -> String {
    let mut text = String::new();
    for part in &ts.parts {
        if let TemplatePart::Text(span) = part {
            text.push_str(&source[span.clone()]);
        }
    }
    text
}

/// Emits an UPPER_SNAKE_CASE config dict from a model declaration.
///
/// Model name converts to UPPER_SNAKE_CASE + "_CONFIG" suffix
/// per CONTEXT.md locked decision.
pub fn emit_model(
    model: &ModelDecl,
    _ast: &Ast,
    interner: &Interner,
    source: &str,
    writer: &mut CodeWriter,
) {
    let config_name = to_config_name(interner.resolve(&model.name));
    let provider = extract_template_text(&model.provider, source);
    let model_id = extract_template_text(&model.model_id, source);

    let caps: Vec<String> = model
        .caps
        .iter()
        .map(|(spur, _)| interner.resolve(spur).to_string())
        .collect();

    writer.writeln(&format!("{config_name} = {{"));
    writer.indent();
    writer.writeln(&format!("\"provider\": \"{provider}\","));
    writer.writeln(&format!("\"model_id\": \"{model_id}\","));

    if caps.is_empty() {
        writer.writeln("\"capabilities\": [],");
    } else {
        let caps_str: Vec<String> = caps.iter().map(|c| format!("\"{c}\"")).collect();
        writer.writeln(&format!("\"capabilities\": [{}],", caps_str.join(", ")));
    }

    writer.dedent();
    writer.writeln("}");
}
