//! Snapshot tests for model declaration emission.

mod test_helpers;

use eaml_codegen::emitters::emit_model;
use eaml_codegen::writer::CodeWriter;
use eaml_parser::ast::DeclId;

/// Helper: emits all models from source, returns the combined output.
fn emit_models(source: &str) -> String {
    let (parse_output, _analysis) = test_helpers::parse_and_analyze(source);
    let ast = &parse_output.ast;
    let interner = &parse_output.interner;

    let mut writer = CodeWriter::new();

    for decl in &parse_output.program.declarations {
        if let DeclId::Model(id) = decl {
            emit_model(&ast[*id], interner, source, &mut writer);
            writer.blank_line();
        }
    }

    writer.finish()
}

#[test]
fn test_minimal_model_no_caps() {
    let source = r#"
model Haiku = Model(
  id: "anthropic/claude-3-haiku-20240307",
  provider: "anthropic",
  caps: []
)
"#;
    let output = emit_models(source);
    insta::assert_snapshot!(output, @r#"
    HAIKU_CONFIG = {
        "provider": "anthropic",
        "model_id": "anthropic/claude-3-haiku-20240307",
        "capabilities": [],
    }
    "#);
}

#[test]
fn test_model_with_capabilities() {
    let source = r#"
model Sonnet = Model(
  id: "anthropic/claude-3-5-sonnet-20241022",
  provider: "anthropic",
  caps: [json_mode, streaming]
)
"#;
    let output = emit_models(source);
    insta::assert_snapshot!(output, @r#"
    SONNET_CONFIG = {
        "provider": "anthropic",
        "model_id": "anthropic/claude-3-5-sonnet-20241022",
        "capabilities": ["json_mode", "streaming"],
    }
    "#);
}

#[test]
fn test_model_pascal_case_name() {
    let source = r#"
model ClaudeHaiku = Model(
  id: "anthropic/claude-3-haiku-20240307",
  provider: "anthropic",
  caps: []
)
"#;
    let output = emit_models(source);
    insta::assert_snapshot!(output, @r#"
    CLAUDE_HAIKU_CONFIG = {
        "provider": "anthropic",
        "model_id": "anthropic/claude-3-haiku-20240307",
        "capabilities": [],
    }
    "#);
}
