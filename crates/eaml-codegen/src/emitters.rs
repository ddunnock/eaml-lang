//! Declaration emitters for schemas, models, let bindings, prompts, tools, and agents.
//!
//! Produces Pydantic BaseModel classes from schema declarations,
//! UPPER_SNAKE_CASE config dicts from model declarations,
//! typed variable assignments from let bindings,
//! async prompt functions with message lists and execute_prompt() calls,
//! tool bridge functions with wrappers and ToolMetadata, and
//! agent classes extending eaml_runtime.Agent.

use eaml_lexer::Interner;
use eaml_parser::ast::*;
use eaml_semantic::type_checker::{ResolvedType, TypeAnnotations};

use crate::extract_template_text;
use crate::names::{to_config_name, to_snake_case};
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
        Expr::StringLit(ts) => emit_template_as_python_string(ts, ast, interner, source),
        Expr::BoolLit(true, _) => "True".to_string(),
        Expr::BoolLit(false, _) => "False".to_string(),
        Expr::NullLit(_) => "None".to_string(),
        Expr::Ident(spur, _) => interner.resolve(spur).to_string(),
        Expr::TemplateStr(ts) => emit_template_as_python_string(ts, ast, interner, source),
        _ => "None".to_string(),
    }
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

/// Emits an UPPER_SNAKE_CASE config dict from a model declaration.
///
/// Model name converts to UPPER_SNAKE_CASE + "_CONFIG" suffix
/// per CONTEXT.md locked decision.
pub fn emit_model(model: &ModelDecl, interner: &Interner, source: &str, writer: &mut CodeWriter) {
    let config_name = to_config_name(interner.resolve(&model.name));
    let provider = extract_template_text(&model.provider, source);
    let model_id = extract_template_text(&model.model_id, source);

    let caps: Vec<String> = model
        .caps
        .iter()
        .map(|(spur, _)| format!("\"{}\"", interner.resolve(spur)))
        .collect();

    writer.writeln(&format!("{config_name} = {{"));
    writer.indent();
    writer.writeln(&format!("\"provider\": \"{provider}\","));
    writer.writeln(&format!("\"model_id\": \"{model_id}\","));

    if caps.is_empty() {
        writer.writeln("\"capabilities\": [],");
    } else {
        writer.writeln(&format!("\"capabilities\": [{}],", caps.join(", ")));
    }

    writer.dedent();
    writer.writeln("}");
}

/// Emits a template string as a Python string literal.
///
/// If the template contains no interpolations, produces a plain string.
/// If it contains interpolations, produces an f-string.
/// Literal `{` and `}` in text parts are escaped to `{{` and `}}`.
/// Multiline content uses `\n` escapes within double-quoted strings.
fn emit_template_as_python_string(
    ts: &TemplateString,
    ast: &Ast,
    interner: &Interner,
    source: &str,
) -> String {
    let has_interpolation = ts
        .parts
        .iter()
        .any(|p| matches!(p, TemplatePart::Interpolation(..)));

    let mut content = String::new();
    for part in &ts.parts {
        match part {
            TemplatePart::Text(span) => {
                let text = &source[span.clone()];
                for ch in text.chars() {
                    match ch {
                        '{' if has_interpolation => content.push_str("{{"),
                        '}' if has_interpolation => content.push_str("}}"),
                        '\n' => content.push_str("\\n"),
                        '"' => content.push_str("\\\""),
                        _ => content.push(ch),
                    }
                }
            }
            TemplatePart::Interpolation(expr_id, _) => {
                content.push('{');
                content.push_str(&emit_expr_value(*expr_id, ast, interner, source));
                content.push('}');
            }
        }
    }

    if has_interpolation {
        format!("f\"{content}\"")
    } else {
        format!("\"{content}\"")
    }
}

/// Emits an async Python function from a prompt declaration.
///
/// Produces: `async def name(params, *, model: dict) -> ReturnType:` with
/// a message list and `return await eaml_runtime.execute_prompt(...)` call.
pub fn emit_prompt(
    prompt: &PromptDecl,
    ast: &Ast,
    interner: &Interner,
    type_annotations: &TypeAnnotations,
    source: &str,
    writer: &mut CodeWriter,
    imports: &mut ImportTracker,
) {
    imports.need_execute_prompt();

    let fn_name = to_snake_case(interner.resolve(&prompt.name));

    // Build parameter list
    let mut params = Vec::new();
    for param in &prompt.params {
        let param_name = interner.resolve(&param.name);
        let resolved = &type_annotations.type_exprs[&param.type_expr];
        imports.track_type(resolved);
        let annotation = emit_type_annotation(resolved, ast, interner);
        params.push(format!("{param_name}: {annotation}"));
    }
    params.push("*, model: dict".to_string());

    // Return type
    let return_resolved = &type_annotations.type_exprs[&prompt.return_type];
    imports.track_type(return_resolved);
    let return_annotation = emit_type_annotation(return_resolved, ast, interner);

    // Emit function signature
    writer.writeln(&format!(
        "async def {fn_name}({}) -> {return_annotation}:",
        params.join(", ")
    ));
    writer.indent();

    // Emit message list
    writer.writeln("messages = [");
    writer.indent();
    for field in &prompt.body.fields {
        match field {
            PromptField::System(ts) => {
                let content = emit_template_as_python_string(ts, ast, interner, source);
                writer.writeln(&format!(
                    "{{\"role\": \"system\", \"content\": {content}}},"
                ));
            }
            PromptField::User(ts) => {
                let content = emit_template_as_python_string(ts, ast, interner, source);
                writer.writeln(&format!("{{\"role\": \"user\", \"content\": {content}}},"));
            }
            _ => {}
        }
    }
    writer.dedent();
    writer.writeln("]");

    // Build execute_prompt kwargs
    let mut kwargs = vec![
        "model=model".to_string(),
        "messages=messages".to_string(),
        format!("return_type={return_annotation}"),
    ];

    // Scan for optional kwargs (temperature, max_tokens, max_retries)
    for field in &prompt.body.fields {
        match field {
            PromptField::Temperature(span) => {
                let val = &source[span.clone()];
                kwargs.push(format!("temperature={val}"));
            }
            PromptField::MaxTokens(span) => {
                let val = &source[span.clone()];
                kwargs.push(format!("max_tokens={val}"));
            }
            PromptField::MaxRetries(span) => {
                let val = &source[span.clone()];
                kwargs.push(format!("max_retries={val}"));
            }
            _ => {}
        }
    }

    // Emit execute_prompt call
    writer.writeln("return await execute_prompt(");
    writer.indent();
    for kwarg in &kwargs {
        writer.writeln(&format!("{kwarg},"));
    }
    writer.dedent();
    writer.writeln(")");

    writer.dedent();
}

/// Dedents bridge code by stripping the minimum indentation from all non-empty lines.
fn dedent_bridge_code(code: &str) -> String {
    let lines: Vec<&str> = code.lines().collect();

    // Find minimum indentation among non-empty lines
    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);

    let dedented: Vec<&str> = lines
        .iter()
        .map(|line| {
            if line.len() >= min_indent {
                &line[min_indent..]
            } else {
                line.trim()
            }
        })
        .collect();

    // Trim leading and trailing empty lines without O(n²) remove(0)
    let start = dedented
        .iter()
        .position(|l| !l.trim().is_empty())
        .unwrap_or(dedented.len());
    let end = dedented
        .iter()
        .rposition(|l| !l.trim().is_empty())
        .map_or(start, |i| i + 1);

    dedented[start..end].join("\n")
}

/// Returns the EAML type name string for a resolved type.
///
/// Used for ToolMetadata parameters where we need EAML type names, not Python.
fn eaml_type_name(resolved: &ResolvedType, ast: &Ast, interner: &Interner) -> String {
    match resolved {
        ResolvedType::Primitive(name) => name.clone(),
        ResolvedType::Schema(id) => interner.resolve(&ast[*id].name).to_string(),
        ResolvedType::Array(inner) => format!("{}[]", eaml_type_name(inner, ast, interner)),
        ResolvedType::Optional(inner) => format!("{}?", eaml_type_name(inner, ast, interner)),
        ResolvedType::LiteralUnion(members) => {
            let items: Vec<String> = members.iter().map(|m| format!("\"{m}\"")).collect();
            items.join(" | ")
        }
        ResolvedType::Error => "any".to_string(),
    }
}

/// Emits a Python tool consisting of bridge function, wrapper function, and ToolMetadata.
///
/// Per PYB-GEN-01 through PYB-GEN-05:
/// - Bridge function: `def name(params) -> dict:` with dedented bridge body
/// - Wrapper function: `def _eaml_call_name(params) -> ReturnType:` with validation
/// - Metadata: `_tool_name = ToolMetadata(...)` for runtime adapter registration
pub fn emit_tool(
    tool: &ToolDecl,
    ast: &Ast,
    interner: &Interner,
    type_annotations: &TypeAnnotations,
    source: &str,
    writer: &mut CodeWriter,
    imports: &mut ImportTracker,
) {
    imports.need_tool_metadata();

    let original_name = interner.resolve(&tool.name);
    let snake_name = to_snake_case(original_name);

    // Build parameter list
    let mut param_parts = Vec::new();
    let mut param_names = Vec::new();
    let mut param_metadata = Vec::new();

    for param in &tool.params {
        let param_name = interner.resolve(&param.name);
        let resolved = &type_annotations.type_exprs[&param.type_expr];
        imports.track_type(resolved);
        let annotation = emit_type_annotation(resolved, ast, interner);
        let eaml_type = eaml_type_name(resolved, ast, interner);
        param_parts.push(format!("{param_name}: {annotation}"));
        param_names.push(param_name.to_string());
        param_metadata.push((param_name.to_string(), eaml_type));
    }

    let params_str = param_parts.join(", ");

    // Return type info
    let return_resolved = &type_annotations.type_exprs[&tool.return_type];
    imports.track_type(return_resolved);
    let return_annotation = emit_type_annotation(return_resolved, ast, interner);
    let return_eaml_type = eaml_type_name(return_resolved, ast, interner);
    let is_schema_return = matches!(return_resolved, ResolvedType::Schema(_));

    match &tool.body {
        ToolBody::PythonBridge {
            description,
            code_span,
            ..
        } => {
            // (a) Bridge function -- returns dict per PYB-GEN-01
            writer.writeln(&format!("def {snake_name}({params_str}) -> dict:"));
            writer.indent();

            // Docstring from description field
            let desc_text = if let Some(ts) = description {
                let text = extract_template_text(ts, source);
                writer.writeln(&format!("\"\"\"{}\"\"\"", text));
                text
            } else {
                original_name.to_string()
            };

            // Dedent and emit bridge code
            let raw_code = &source[code_span.clone()];
            let dedented = dedent_bridge_code(raw_code);
            for line in dedented.lines() {
                writer.writeln(line);
            }

            writer.dedent();
            writer.blank_line();

            // (b) Wrapper function per PYB-GEN-01
            writer.writeln(&format!(
                "def _eaml_call_{snake_name}({params_str}) -> {return_annotation}:"
            ));
            writer.indent();

            let call_args = param_names.join(", ");
            writer.writeln(&format!("result = {snake_name}({call_args})"));

            if is_schema_return {
                writer.writeln(&format!(
                    "return {return_annotation}.model_validate(result)"
                ));
            } else {
                // Primitive type check per PYB-GEN-04 example
                let py_type = &return_annotation;
                writer.writeln(&format!("if not isinstance(result, {py_type}):"));
                writer.indent();
                writer.writeln(&format!(
                    "raise TypeError(f\"Tool '{original_name}' expected '{py_type}', got '{{type(result).__name__}}'\")"
                ));
                writer.dedent();
                writer.writeln("return result");
            }

            writer.dedent();
            writer.blank_line();

            // (c) ToolMetadata per PYB-GEN-03
            writer.writeln(&format!("_tool_{snake_name} = ToolMetadata("));
            writer.indent();
            writer.writeln(&format!("name=\"{original_name}\","));
            writer.writeln(&format!("description=\"{desc_text}\","));
            writer.writeln("parameters=[");
            writer.indent();
            for (pname, ptype) in &param_metadata {
                writer.writeln(&format!(
                    "{{\"name\": \"{pname}\", \"type\": \"{ptype}\"}},"
                ));
            }
            writer.dedent();
            writer.writeln("],");
            writer.writeln(&format!("return_type=\"{return_eaml_type}\","));
            writer.writeln(&format!("function=_eaml_call_{snake_name},"));
            writer.dedent();
            writer.writeln(")");
        }
        ToolBody::Native { .. } | ToolBody::Empty(_) => {
            // Native and empty bodies are post-MVP / caught by SEM040
        }
    }
}

/// Emits a Python class extending eaml_runtime.Agent from an agent declaration.
///
/// Agent names stay PascalCase per CONTEXT.md locked decision.
/// Produces class attributes for model, tools, system_prompt, max_turns, on_error.
pub fn emit_agent(
    agent: &AgentDecl,
    ast: &Ast,
    interner: &Interner,
    source: &str,
    writer: &mut CodeWriter,
    imports: &mut ImportTracker,
) {
    imports.need_agent();

    let name = interner.resolve(&agent.name);
    writer.writeln(&format!("class {name}(Agent):"));
    writer.indent();

    if agent.fields.is_empty() {
        writer.writeln("pass");
    } else {
        for field in &agent.fields {
            match field {
                AgentField::Model(spur, _) => {
                    let config_name = to_config_name(interner.resolve(spur));
                    writer.writeln(&format!("model = {config_name}"));
                }
                AgentField::Tools(tools, _) => {
                    let tool_names: Vec<String> = tools
                        .iter()
                        .map(|(s, _)| to_snake_case(interner.resolve(s)))
                        .collect();
                    writer.writeln(&format!("tools = [{}]", tool_names.join(", ")));
                }
                AgentField::System(ts) => {
                    let content = emit_template_as_python_string(ts, ast, interner, source);
                    writer.writeln(&format!("system_prompt = {content}"));
                }
                AgentField::MaxTurns(span) => {
                    let val = &source[span.clone()];
                    writer.writeln(&format!("max_turns = {val}"));
                }
                AgentField::OnError(ErrorPolicy::Fail, _) => {
                    writer.writeln("on_error = \"fail\"");
                }
                AgentField::OnError(ErrorPolicy::RetryThenFail { retries_span }, _) => {
                    writer.writeln("on_error = \"retry_then_fail\"");
                    let val = &source[retries_span.clone()];
                    writer.writeln(&format!("on_error_retries = {val}"));
                }
            }
        }
    }

    writer.dedent();
}
