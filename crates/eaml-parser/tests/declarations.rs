//! Tests for declaration parsing.

#[allow(dead_code)]
mod test_helpers;

use eaml_parser::ast::*;
use test_helpers::parse_program;

/// Format a DeclId for snapshot testing.
fn format_decl(ast: &Ast, decl: &DeclId, interner: &eaml_lexer::Interner) -> String {
    match decl {
        DeclId::Model(id) => {
            let m = &ast[*id];
            let name = interner.resolve(m.name);
            let caps: Vec<String> = m
                .caps
                .iter()
                .map(|(s, _)| interner.resolve(*s).to_string())
                .collect();
            format!(
                "Model({}, id={}, provider={}, caps=[{}], span={:?})",
                name,
                format_template(ast, &m.model_id, interner),
                format_template(ast, &m.provider, interner),
                caps.join(", "),
                m.span
            )
        }
        DeclId::Schema(id) => {
            let s = &ast[*id];
            let name = interner.resolve(s.name);
            let fields: Vec<String> = s
                .fields
                .iter()
                .map(|f| {
                    let fname = interner.resolve(f.name);
                    format!("  {}: {:?}", fname, &ast[f.type_expr])
                })
                .collect();
            format!(
                "Schema({}, fields=[\n{}\n], span={:?})",
                name,
                fields.join(",\n"),
                s.span
            )
        }
        DeclId::Import(id) => {
            let imp = &ast[*id];
            match imp {
                ImportDecl::Eaml { path, alias, span } => {
                    let alias_str = alias
                        .map(|s| interner.resolve(s).to_string())
                        .unwrap_or_default();
                    format!(
                        "Import::Eaml(path={}, alias={:?}, span={:?})",
                        format_template(ast, path, interner),
                        alias_str,
                        span
                    )
                }
                ImportDecl::Python {
                    module,
                    alias,
                    span,
                } => {
                    let alias_str = alias
                        .map(|s| interner.resolve(s).to_string())
                        .unwrap_or_default();
                    format!(
                        "Import::Python(module={}, alias={:?}, span={:?})",
                        format_template(ast, module, interner),
                        alias_str,
                        span
                    )
                }
            }
        }
        DeclId::Let(id) => {
            let l = &ast[*id];
            let name = interner.resolve(l.name);
            format!(
                "Let({}, type={:?}, value={:?}, span={:?})",
                name, &ast[l.type_expr], &ast[l.value], l.span
            )
        }
        DeclId::Prompt(id) => {
            let p = &ast[*id];
            let name = interner.resolve(p.name);
            let params: Vec<String> = p
                .params
                .iter()
                .map(|param| {
                    let pname = interner.resolve(param.name);
                    format!("{}:{:?}", pname, &ast[param.type_expr])
                })
                .collect();
            let requires_str = match &p.requires {
                Some(req) => {
                    let caps: Vec<String> = req
                        .caps
                        .iter()
                        .map(|(s, _)| interner.resolve(*s).to_string())
                        .collect();
                    format!("requires [{}]", caps.join(", "))
                }
                None => "no requires".to_string(),
            };
            let fields: Vec<String> = p.body.fields.iter().map(|f| format!("  {:?}", f)).collect();
            format!(
                "Prompt({}, params=[{}], {}, return={:?}, body=[\n{}\n], span={:?})",
                name,
                params.join(", "),
                requires_str,
                &ast[p.return_type],
                fields.join(",\n"),
                p.span
            )
        }
        DeclId::Tool(id) => {
            let t = &ast[*id];
            let name = interner.resolve(t.name);
            let params: Vec<String> = t
                .params
                .iter()
                .map(|param| {
                    let pname = interner.resolve(param.name);
                    format!("{}:{:?}", pname, &ast[param.type_expr])
                })
                .collect();
            format!(
                "Tool({}, params=[{}], return={:?}, body={:?}, span={:?})",
                name,
                params.join(", "),
                &ast[t.return_type],
                t.body,
                t.span
            )
        }
        DeclId::Agent(id) => {
            let a = &ast[*id];
            let name = interner.resolve(a.name);
            format!("Agent({}, fields={:?}, span={:?})", name, a.fields, a.span)
        }
        DeclId::Error(span) => format!("Error({:?})", span),
    }
}

/// Format a template string for display.
fn format_template(_ast: &Ast, ts: &TemplateString, _interner: &eaml_lexer::Interner) -> String {
    let parts: Vec<String> = ts
        .parts
        .iter()
        .map(|p| match p {
            TemplatePart::Text(span) => format!("Text({:?})", span),
            TemplatePart::Interpolation(_, span) => format!("Interp({:?})", span),
        })
        .collect();
    format!("Template([{}])", parts.join(", "))
}

// ===================================================================
// Task 1 tests: import, model, schema, let, parse_program
// ===================================================================

#[test]
fn decl_empty_program() {
    let output = parse_program("");
    assert_eq!(output.program.declarations.len(), 0);
    assert!(
        output
            .diagnostics
            .iter()
            .filter(|d| d.severity == eaml_errors::Severity::Error
                || d.severity == eaml_errors::Severity::Fatal)
            .count()
            == 0
    );
}

#[test]
fn decl_import_eaml() {
    let output = parse_program(r#"import "foo.eaml""#);
    assert_eq!(output.program.declarations.len(), 1);
    let s = format_decl(
        &output.ast,
        &output.program.declarations[0],
        &output.interner,
    );
    insta::assert_snapshot!(s);
}

#[test]
fn decl_import_eaml_with_alias() {
    let output = parse_program(r#"import "foo.eaml" as Foo"#);
    assert_eq!(output.program.declarations.len(), 1);
    let s = format_decl(
        &output.ast,
        &output.program.declarations[0],
        &output.interner,
    );
    insta::assert_snapshot!(s);
}

#[test]
fn decl_import_python() {
    let output = parse_program(r#"import python "os""#);
    assert_eq!(output.program.declarations.len(), 1);
    let s = format_decl(
        &output.ast,
        &output.program.declarations[0],
        &output.interner,
    );
    insta::assert_snapshot!(s);
}

#[test]
fn decl_import_python_with_alias() {
    let output = parse_program(r#"import python "os" as os_mod"#);
    assert_eq!(output.program.declarations.len(), 1);
    let s = format_decl(
        &output.ast,
        &output.program.declarations[0],
        &output.interner,
    );
    insta::assert_snapshot!(s);
}

#[test]
fn decl_model_empty_caps() {
    let source = r#"model Haiku = Model(id: "test-id", provider: "anthropic", caps: [])"#;
    let output = parse_program(source);
    assert_eq!(output.program.declarations.len(), 1);
    let errors: Vec<_> = output
        .diagnostics
        .iter()
        .filter(|d| {
            d.severity == eaml_errors::Severity::Error || d.severity == eaml_errors::Severity::Fatal
        })
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    let s = format_decl(
        &output.ast,
        &output.program.declarations[0],
        &output.interner,
    );
    insta::assert_snapshot!(s);
}

#[test]
fn decl_model_with_caps() {
    let source = r#"model Sonnet = Model(id: "test-id", provider: "anthropic", caps: [json_mode, streaming])"#;
    let output = parse_program(source);
    assert_eq!(output.program.declarations.len(), 1);
    let errors: Vec<_> = output
        .diagnostics
        .iter()
        .filter(|d| {
            d.severity == eaml_errors::Severity::Error || d.severity == eaml_errors::Severity::Fatal
        })
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    let s = format_decl(
        &output.ast,
        &output.program.declarations[0],
        &output.interner,
    );
    insta::assert_snapshot!(s);
}

#[test]
fn decl_schema_two_fields() {
    let source = "schema Greeting { message: string, count: int }";
    let output = parse_program(source);
    assert_eq!(output.program.declarations.len(), 1);
    let errors: Vec<_> = output
        .diagnostics
        .iter()
        .filter(|d| {
            d.severity == eaml_errors::Severity::Error || d.severity == eaml_errors::Severity::Fatal
        })
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    let s = format_decl(
        &output.ast,
        &output.program.declarations[0],
        &output.interner,
    );
    insta::assert_snapshot!(s);
}

#[test]
fn decl_schema_newline_separation() {
    let source = "schema S {\n  a: string\n  b: int\n}";
    let output = parse_program(source);
    assert_eq!(output.program.declarations.len(), 1);
    let errors: Vec<_> = output
        .diagnostics
        .iter()
        .filter(|d| {
            d.severity == eaml_errors::Severity::Error || d.severity == eaml_errors::Severity::Fatal
        })
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
}

#[test]
fn decl_let_int() {
    let source = "let x: int = 42";
    let output = parse_program(source);
    assert_eq!(output.program.declarations.len(), 1);
    let errors: Vec<_> = output
        .diagnostics
        .iter()
        .filter(|d| {
            d.severity == eaml_errors::Severity::Error || d.severity == eaml_errors::Severity::Fatal
        })
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    let s = format_decl(
        &output.ast,
        &output.program.declarations[0],
        &output.interner,
    );
    insta::assert_snapshot!(s);
}

#[test]
fn decl_let_string() {
    let source = r#"let name: string = "hello""#;
    let output = parse_program(source);
    assert_eq!(output.program.declarations.len(), 1);
    let errors: Vec<_> = output
        .diagnostics
        .iter()
        .filter(|d| {
            d.severity == eaml_errors::Severity::Error || d.severity == eaml_errors::Severity::Fatal
        })
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
}

#[test]
fn decl_program_dispatches_multiple() {
    let source = r#"import "foo.eaml"
model M = Model(id: "x", provider: "y", caps: [])
schema S { a: string }
let x: int = 1"#;
    let output = parse_program(source);
    assert_eq!(output.program.declarations.len(), 4);
    assert!(matches!(output.program.declarations[0], DeclId::Import(_)));
    assert!(matches!(output.program.declarations[1], DeclId::Model(_)));
    assert!(matches!(output.program.declarations[2], DeclId::Schema(_)));
    assert!(matches!(output.program.declarations[3], DeclId::Let(_)));
}

#[test]
fn decl_optional_semicolon() {
    let source = "schema S { a: string };";
    let output = parse_program(source);
    assert_eq!(output.program.declarations.len(), 1);
    let errors: Vec<_> = output
        .diagnostics
        .iter()
        .filter(|d| {
            d.severity == eaml_errors::Severity::Error || d.severity == eaml_errors::Severity::Fatal
        })
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
}

#[test]
fn decl_post_mvp_pipeline() {
    let source = "pipeline P { }";
    let output = parse_program(source);
    // Should produce an error DeclId and SYN080 diagnostic
    assert!(!output.program.declarations.is_empty());
    assert!(matches!(output.program.declarations[0], DeclId::Error(_)));
    let has_syn080 = output
        .diagnostics
        .iter()
        .any(|d| d.code == eaml_errors::ErrorCode::Syn080);
    assert!(has_syn080, "expected SYN080 diagnostic");
}

#[test]
fn decl_post_mvp_enum() {
    let source = "enum E { }";
    let output = parse_program(source);
    assert!(!output.program.declarations.is_empty());
    assert!(matches!(output.program.declarations[0], DeclId::Error(_)));
    let has_syn082 = output
        .diagnostics
        .iter()
        .any(|d| d.code == eaml_errors::ErrorCode::Syn082);
    assert!(has_syn082, "expected SYN082 diagnostic");
}

#[test]
fn decl_post_mvp_extends() {
    let source = "schema S extends Base { }";
    let output = parse_program(source);
    let has_syn083 = output
        .diagnostics
        .iter()
        .any(|d| d.code == eaml_errors::ErrorCode::Syn083);
    assert!(has_syn083, "expected SYN083 diagnostic");
}

#[test]
fn decl_post_mvp_annotation() {
    let source = "@deprecated schema S {}";
    let output = parse_program(source);
    let has_syn090 = output
        .diagnostics
        .iter()
        .any(|d| d.code == eaml_errors::ErrorCode::Syn090);
    assert!(has_syn090, "expected SYN090 diagnostic");
}

#[test]
fn decl_error_recovery() {
    // Invalid token followed by valid schema
    let source = "!!! schema Valid { a: string }";
    let output = parse_program(source);
    assert!(output.program.declarations.len() >= 2);
    // First should be Error, second should be Schema
    assert!(matches!(output.program.declarations[0], DeclId::Error(_)));
    assert!(matches!(output.program.declarations[1], DeclId::Schema(_)));
}

// ===================================================================
// Task 2 tests: prompt, tool, agent declarations
// ===================================================================

#[test]
fn decl_prompt_simple() {
    let source = r#"prompt Greet(name: string) -> Greeting {
  user: "Hello {name}"
}"#;
    let output = parse_program(source);
    assert_eq!(output.program.declarations.len(), 1);
    let errors: Vec<_> = output
        .diagnostics
        .iter()
        .filter(|d| {
            d.severity == eaml_errors::Severity::Error || d.severity == eaml_errors::Severity::Fatal
        })
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    assert!(matches!(output.program.declarations[0], DeclId::Prompt(_)));
    let s = format_decl(
        &output.ast,
        &output.program.declarations[0],
        &output.interner,
    );
    insta::assert_snapshot!(s);
}

#[test]
fn decl_prompt_bare_requires() {
    let source = r#"prompt P(text: string) requires json_mode -> R {
  user: "..."
}"#;
    let output = parse_program(source);
    assert_eq!(output.program.declarations.len(), 1);
    let errors: Vec<_> = output
        .diagnostics
        .iter()
        .filter(|d| {
            d.severity == eaml_errors::Severity::Error || d.severity == eaml_errors::Severity::Fatal
        })
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    // Check requires clause
    if let DeclId::Prompt(id) = &output.program.declarations[0] {
        let p = &output.ast[*id];
        assert!(p.requires.is_some());
        let req = p.requires.as_ref().unwrap();
        assert_eq!(req.caps.len(), 1);
    } else {
        panic!("expected Prompt");
    }
}

#[test]
fn decl_prompt_bracketed_requires() {
    let source = r#"prompt P(x: int) requires [json_mode, tools] -> R {
  user: "..."
}"#;
    let output = parse_program(source);
    let errors: Vec<_> = output
        .diagnostics
        .iter()
        .filter(|d| {
            d.severity == eaml_errors::Severity::Error || d.severity == eaml_errors::Severity::Fatal
        })
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    if let DeclId::Prompt(id) = &output.program.declarations[0] {
        let p = &output.ast[*id];
        let req = p.requires.as_ref().unwrap();
        assert_eq!(req.caps.len(), 2);
    } else {
        panic!("expected Prompt");
    }
}

#[test]
fn decl_prompt_empty_requires() {
    let source = r#"prompt P() requires [] -> R {
  user: "..."
}"#;
    let output = parse_program(source);
    let errors: Vec<_> = output
        .diagnostics
        .iter()
        .filter(|d| {
            d.severity == eaml_errors::Severity::Error || d.severity == eaml_errors::Severity::Fatal
        })
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    if let DeclId::Prompt(id) = &output.program.declarations[0] {
        let p = &output.ast[*id];
        let req = p.requires.as_ref().unwrap();
        assert_eq!(req.caps.len(), 0);
    } else {
        panic!("expected Prompt");
    }
}

#[test]
fn decl_prompt_all_fields() {
    let source = r#"prompt P(a: string, b: int) -> R {
  system: "system msg"
  user: "user msg"
  temperature: 0.5
  max_tokens: 100
}"#;
    let output = parse_program(source);
    let errors: Vec<_> = output
        .diagnostics
        .iter()
        .filter(|d| {
            d.severity == eaml_errors::Severity::Error || d.severity == eaml_errors::Severity::Fatal
        })
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    if let DeclId::Prompt(id) = &output.program.declarations[0] {
        let p = &output.ast[*id];
        assert_eq!(p.params.len(), 2);
        assert_eq!(p.body.fields.len(), 4);
    } else {
        panic!("expected Prompt");
    }
}

#[test]
fn decl_prompt_max_retries() {
    let source = r#"prompt P() -> R {
  user: "msg"
  max_retries: 3
}"#;
    let output = parse_program(source);
    let errors: Vec<_> = output
        .diagnostics
        .iter()
        .filter(|d| {
            d.severity == eaml_errors::Severity::Error || d.severity == eaml_errors::Severity::Fatal
        })
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    if let DeclId::Prompt(id) = &output.program.declarations[0] {
        let p = &output.ast[*id];
        assert_eq!(p.body.fields.len(), 2);
    } else {
        panic!("expected Prompt");
    }
}

#[test]
fn decl_tool_python_bridge() {
    let source = r#"tool Fetch(url: string) -> string {
  python %{
import requests
return requests.get(url).text
}%
}"#;
    let output = parse_program(source);
    let errors: Vec<_> = output
        .diagnostics
        .iter()
        .filter(|d| {
            d.severity == eaml_errors::Severity::Error || d.severity == eaml_errors::Severity::Fatal
        })
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    assert!(matches!(output.program.declarations[0], DeclId::Tool(_)));
    if let DeclId::Tool(id) = &output.program.declarations[0] {
        let t = &output.ast[*id];
        assert!(matches!(t.body, ToolBody::PythonBridge { .. }));
    }
}

#[test]
fn decl_tool_with_description() {
    let source = r#"tool T(x: int) -> int {
  description: "does stuff"
  python %{
return x * 2
}%
}"#;
    let output = parse_program(source);
    let errors: Vec<_> = output
        .diagnostics
        .iter()
        .filter(|d| {
            d.severity == eaml_errors::Severity::Error || d.severity == eaml_errors::Severity::Fatal
        })
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    if let DeclId::Tool(id) = &output.program.declarations[0] {
        let t = &output.ast[*id];
        match &t.body {
            ToolBody::PythonBridge { description, .. } => {
                assert!(description.is_some());
            }
            _ => panic!("expected PythonBridge body"),
        }
    }
}

#[test]
fn decl_agent_all_fields() {
    let source = r#"agent Bot {
  model: Claude
  tools: [Search, Fetch]
  system: "You are a helpful bot"
  max_turns: 10
  on_error: fail
}"#;
    let output = parse_program(source);
    let errors: Vec<_> = output
        .diagnostics
        .iter()
        .filter(|d| {
            d.severity == eaml_errors::Severity::Error || d.severity == eaml_errors::Severity::Fatal
        })
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    assert!(matches!(output.program.declarations[0], DeclId::Agent(_)));
    if let DeclId::Agent(id) = &output.program.declarations[0] {
        let a = &output.ast[*id];
        assert_eq!(a.fields.len(), 5);
    }
}

#[test]
fn decl_agent_retry_policy() {
    let source = r#"agent A {
  model: M
  tools: [T]
  on_error: retry(3) then fail
}"#;
    let output = parse_program(source);
    let errors: Vec<_> = output
        .diagnostics
        .iter()
        .filter(|d| {
            d.severity == eaml_errors::Severity::Error || d.severity == eaml_errors::Severity::Fatal
        })
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    if let DeclId::Agent(id) = &output.program.declarations[0] {
        let a = &output.ast[*id];
        // Find the on_error field
        let has_retry = a
            .fields
            .iter()
            .any(|f| matches!(f, AgentField::OnError(ErrorPolicy::RetryThenFail { .. }, _)));
        assert!(has_retry, "expected RetryThenFail policy");
    }
}

#[test]
fn decl_tool_empty_body() {
    let source = "tool T(x: int) -> int { }";
    let output = parse_program(source);
    if let DeclId::Tool(id) = &output.program.declarations[0] {
        let t = &output.ast[*id];
        assert!(matches!(t.body, ToolBody::Empty(_)));
    }
}

// ===================================================================
// Integration tests: full example files
// ===================================================================

#[test]
fn decl_example_minimal() {
    let source = include_str!("../../../examples/01-minimal/minimal.eaml");
    let output = parse_program(source);
    let errors: Vec<_> = output
        .diagnostics
        .iter()
        .filter(|d| {
            d.severity == eaml_errors::Severity::Error || d.severity == eaml_errors::Severity::Fatal
        })
        .collect();
    assert!(
        errors.is_empty(),
        "minimal.eaml had parse errors: {:?}",
        errors
    );
    assert_eq!(output.program.declarations.len(), 3); // model + schema + prompt
}

#[test]
fn decl_example_sentiment() {
    let source = include_str!("../../../examples/02-sentiment/sentiment.eaml");
    let output = parse_program(source);
    let errors: Vec<_> = output
        .diagnostics
        .iter()
        .filter(|d| {
            d.severity == eaml_errors::Severity::Error || d.severity == eaml_errors::Severity::Fatal
        })
        .collect();
    assert!(
        errors.is_empty(),
        "sentiment.eaml had parse errors: {:?}",
        errors
    );
    assert_eq!(output.program.declarations.len(), 3); // model + schema + prompt
}

#[test]
fn decl_example_types() {
    let source = include_str!("../../../examples/07-all-type-variants/types.eaml");
    let output = parse_program(source);
    let errors: Vec<_> = output
        .diagnostics
        .iter()
        .filter(|d| {
            d.severity == eaml_errors::Severity::Error || d.severity == eaml_errors::Severity::Fatal
        })
        .collect();
    assert!(
        errors.is_empty(),
        "types.eaml had parse errors: {:?}",
        errors
    );
    // model + 6 schemas + 3 prompts = 10
    assert_eq!(output.program.declarations.len(), 10);
}

#[test]
fn decl_example_bad_model() {
    let source = include_str!("../../../examples/06-capability-error/bad_model.eaml");
    let output = parse_program(source);
    let errors: Vec<_> = output
        .diagnostics
        .iter()
        .filter(|d| {
            d.severity == eaml_errors::Severity::Error || d.severity == eaml_errors::Severity::Fatal
        })
        .collect();
    assert!(
        errors.is_empty(),
        "bad_model.eaml had parse errors (cap errors are semantic, not parser): {:?}",
        errors
    );
    assert_eq!(output.program.declarations.len(), 3); // model + schema + prompt
}
