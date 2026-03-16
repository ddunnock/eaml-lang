//! Snapshot tests for agent declaration emission.

mod test_helpers;

use eaml_codegen::emitters::emit_agent;
use eaml_codegen::types::ImportTracker;
use eaml_codegen::writer::CodeWriter;
use eaml_parser::ast::DeclId;

/// Helper: emits all agents from source, returns the combined output.
fn emit_agents(source: &str) -> String {
    let (parse_output, _analysis) = test_helpers::parse_and_analyze(source);
    let ast = &parse_output.ast;
    let interner = &parse_output.interner;

    let mut writer = CodeWriter::new();
    let mut imports = ImportTracker::new();

    for decl in &parse_output.program.declarations {
        if let DeclId::Agent(id) = decl {
            emit_agent(&ast[*id], ast, interner, source, &mut writer, &mut imports);
            writer.blank_line();
        }
    }

    writer.finish()
}

#[test]
fn test_agent_with_all_fields() {
    let source = r#"
model Sonnet = Model(
  id: "anthropic/claude-3-5-sonnet-20241022",
  provider: "anthropic",
  caps: [json_mode, streaming]
)

schema PageInfo {
  title: string
  length: int
}

tool fetchPage(url: string) -> PageInfo {
  python %{
    return {"title": url, "length": 0}
  }%
}

agent ResearchAssistant {
  model: Sonnet
  tools: [fetchPage]
  system: "You are a research assistant."
  max_turns: 5
  on_error: retry(3) then fail
}
"#;
    let output = emit_agents(source);
    insta::assert_snapshot!(output, @r#"
    class ResearchAssistant(Agent):
        model = SONNET_CONFIG
        tools = [fetch_page]
        system_prompt = "You are a research assistant."
        max_turns = 5
        on_error = "retry_then_fail"
        on_error_retries = 3
    "#);
}

#[test]
fn test_agent_with_on_error_fail() {
    let source = r#"
model Haiku = Model(
  id: "anthropic/claude-3-haiku-20240307",
  provider: "anthropic",
  caps: []
)

agent SimpleBot {
  model: Haiku
  system: "You are a simple bot."
  on_error: fail
}
"#;
    let output = emit_agents(source);
    insta::assert_snapshot!(output, @r#"
    class SimpleBot(Agent):
        model = HAIKU_CONFIG
        system_prompt = "You are a simple bot."
        on_error = "fail"
    "#);
}

#[test]
fn test_agent_minimal() {
    let source = r#"
model Haiku = Model(
  id: "anthropic/claude-3-haiku-20240307",
  provider: "anthropic",
  caps: []
)

agent EmptyAgent {
  model: Haiku
}
"#;
    let output = emit_agents(source);
    insta::assert_snapshot!(output, @"
    class EmptyAgent(Agent):
        model = HAIKU_CONFIG
    ");
}
