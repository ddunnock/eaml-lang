//! Name resolution tests for eaml-semantic.

mod test_helpers;

use eaml_errors::{ErrorCode, Severity};
use test_helpers::{
    analyze_source, assert_has_code, assert_no_errors, has_secondary_label_containing,
};

// =============================================================================
// Pass 1: Declaration registration
// =============================================================================

#[test]
fn all_seven_decl_types_register() {
    let source = r#"
        import python "utils" as pyutils
        model Gpt4 = Model(id: "gpt-4", provider: "openai", caps: [])
        schema User { name: string }
        prompt greet(name: string) -> string { user: "Hello {name}" }
        tool fetch(url: string) -> string {
            python %{
                return url
            }%
        }
        agent Helper { model: Gpt4; tools: [fetch] }
        let x: int = 42
    "#;
    let (_parse, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

#[test]
fn model_registers_in_symbol_table() {
    let source = r#"model Gpt4 = Model(id: "gpt-4", provider: "openai", caps: [])"#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

#[test]
fn schema_registers_in_symbol_table() {
    let source = r#"schema User { name: string }"#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

#[test]
fn prompt_registers_in_symbol_table() {
    let source = r#"prompt greet(name: string) -> string { user: "Hello" }"#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

#[test]
fn tool_registers_in_symbol_table() {
    let source = r#"tool fetch(url: string) -> string {
        python %{
            return url
        }%
    }"#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

#[test]
fn agent_registers_in_symbol_table() {
    let source = r#"
        model Gpt4 = Model(id: "gpt-4", provider: "openai", caps: [])
        agent Helper { model: Gpt4 }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

#[test]
fn import_with_alias_registers() {
    let source = r#"import python "utils" as pyutils"#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

#[test]
fn let_registers_in_symbol_table() {
    let source = r#"let x: int = 42"#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

// =============================================================================
// Forward references
// =============================================================================

#[test]
fn forward_reference_prompt_to_later_schema() {
    // Prompt references schema declared later -- should work
    let source = r#"
        prompt greet(name: string) -> User { user: "Hello" }
        schema User { name: string }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

// =============================================================================
// Duplicate detection (RES010)
// =============================================================================

#[test]
fn duplicate_schema_produces_res010() {
    let source = r#"
        schema User { name: string }
        schema User { email: string }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Res010);
}

#[test]
fn duplicate_schema_has_first_defined_here_secondary() {
    let source = r#"
        schema User { name: string }
        schema User { email: string }
    "#;
    let (_, analysis) = analyze_source(source);
    assert!(
        has_secondary_label_containing(&analysis, ErrorCode::Res010, "first defined here"),
        "RES010 should have 'first defined here' secondary label"
    );
}

#[test]
fn duplicate_model_produces_res010() {
    let source = r#"
        model Gpt4 = Model(id: "gpt-4", provider: "openai", caps: [])
        model Gpt4 = Model(id: "gpt-4o", provider: "openai", caps: [])
    "#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Res010);
}

// =============================================================================
// Undefined references (RES001)
// =============================================================================

#[test]
fn undefined_agent_model_produces_res001() {
    let source = r#"
        agent Helper { model: NonExistent }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Res001);
}

#[test]
fn undefined_reference_with_close_match_has_did_you_mean() {
    let source = r#"
        model Gpt4 = Model(id: "gpt-4", provider: "openai", caps: [])
        agent Helper { model: Gpt5 }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Res001);
    let diag = analysis
        .diagnostics
        .iter()
        .find(|d| d.code == ErrorCode::Res001)
        .unwrap();
    assert!(
        diag.hints.iter().any(|h| h.contains("did you mean")),
        "Should have 'did you mean' hint, got hints: {:?}",
        diag.hints
    );
}

#[test]
fn schema_field_undeclared_type_produces_res001() {
    let source = r#"
        schema User { address: Address }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Res001);
}

#[test]
fn agent_tool_reference_resolves() {
    let source = r#"
        model Gpt4 = Model(id: "gpt-4", provider: "openai", caps: [])
        tool fetch(url: string) -> string {
            python %{
                return url
            }%
        }
        agent Helper { model: Gpt4; tools: [fetch] }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

#[test]
fn agent_model_reference_resolves() {
    let source = r#"
        model Gpt4 = Model(id: "gpt-4", provider: "openai", caps: [])
        agent Helper { model: Gpt4 }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

// =============================================================================
// Let binding sequential scoping
// =============================================================================

#[test]
fn let_bindings_are_sequential() {
    let source = r#"
        let x: int = 1
        let y: int = 2
    "#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

// =============================================================================
// Import ordering (SEM010)
// =============================================================================

#[test]
fn python_import_after_declaration_produces_sem010() {
    let source = r#"
        schema User { name: string }
        import python "utils"
    "#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Sem010);
}

// =============================================================================
// Python bridge bodies are NOT validated
// =============================================================================

#[test]
fn python_bridge_body_not_validated() {
    let source = r#"
        tool fetch(url: string) -> string {
            description: "Fetches a URL"
            python %{
                import requests
                return requests.get(url).text
            }%
        }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

// =============================================================================
// Error recovery nodes are skipped
// =============================================================================

#[test]
fn error_recovery_nodes_skipped() {
    // Source with an invalid token -- parser will produce Error recovery nodes
    // but semantic analysis should not crash
    let source = r#"
        schema User { name: string }
        @@@invalid
        schema Post { title: string }
    "#;
    let (_parse, _analysis) = analyze_source(source);
    // Should not crash; may have parse errors but semantic analysis completes
}

// =============================================================================
// Cycle detection (SEM070)
// =============================================================================

#[test]
fn direct_self_referencing_schema_produces_sem070() {
    let source = r#"
        schema TreeNode {
            value: string
            child: TreeNode
        }
    "#;
    let (_, analysis) = analyze_source(source);
    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|d| d.code == ErrorCode::Sem070),
        "Direct self-reference should produce SEM070 warning, got: {:?}",
        analysis
            .diagnostics
            .iter()
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect::<Vec<_>>()
    );
    let sem070 = analysis
        .diagnostics
        .iter()
        .find(|d| d.code == ErrorCode::Sem070)
        .unwrap();
    assert_eq!(sem070.severity, Severity::Warning);
}

#[test]
fn indirect_cycle_produces_sem070() {
    let source = r#"
        schema A {
            b: B
        }
        schema B {
            a: A
        }
    "#;
    let (_, analysis) = analyze_source(source);
    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|d| d.code == ErrorCode::Sem070),
        "Indirect cycle should produce SEM070 warning, got: {:?}",
        analysis
            .diagnostics
            .iter()
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect::<Vec<_>>()
    );
}

#[test]
fn non_cyclic_schema_reference_no_sem070() {
    let source = r#"
        schema A {
            b: B
        }
        schema B {
            x: string
        }
    "#;
    let (_, analysis) = analyze_source(source);
    assert!(
        !analysis
            .diagnostics
            .iter()
            .any(|d| d.code == ErrorCode::Sem070),
        "Non-cyclic reference should not produce SEM070"
    );
    assert_no_errors(&analysis);
}
