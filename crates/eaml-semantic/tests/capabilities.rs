//! Capability checking tests for eaml-semantic (SEM-08, SEM-09).
//!
//! Validates:
//! - CAP010: capability mismatch (FATAL) with diff message
//! - CAP001: unknown capability (warning)
//! - CAP002: duplicate capability (warning)
//! - CAP020: json_mode + string return type (warning)
//! - has_fatal flag set on CAP010
//! - Multiple prompts against respective agents' models
//! - Prompts with no requires clause skip checking

mod test_helpers;

use eaml_errors::{ErrorCode, Severity};
use test_helpers::{analyze_source, assert_has_code, assert_no_errors};

// =============================================================================
// CAP010: Capability subset mismatch (FATAL)
// =============================================================================

#[test]
fn cap_prompt_requires_subset_of_model_caps_no_error() {
    let source = r#"
        model GPT = Model(id: "gpt-4", provider: "openai", caps: [json_mode])
        schema R { result: string }
        prompt P(x: string) requires json_mode -> R {
            user: "{x}"
        }
    "#;
    let (_, analysis) = analyze_source(source);
    // Should have no capability errors
    let cap_errors: Vec<_> = analysis
        .diagnostics
        .iter()
        .filter(|d| matches!(d.code, ErrorCode::Cap010))
        .collect();
    assert!(cap_errors.is_empty(), "No CAP010 expected when caps match");
}

#[test]
fn cap_prompt_requires_missing_cap_produces_cap010() {
    let source = r#"
        model GPT = Model(id: "gpt-4", provider: "openai", caps: [json_mode])
        schema R { result: string }
        prompt P(x: string) requires [json_mode, streaming] -> R {
            user: "{x}"
        }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Cap010);
    let diag = analysis
        .diagnostics
        .iter()
        .find(|d| d.code == ErrorCode::Cap010)
        .unwrap();
    assert_eq!(diag.severity, Severity::Fatal, "CAP010 must be FATAL");
    assert!(
        diag.message.to_lowercase().contains("missing"),
        "CAP010 message should mention missing caps, got: {}",
        diag.message
    );
    assert!(
        diag.message.contains("streaming"),
        "CAP010 message should mention 'streaming' as missing, got: {}",
        diag.message
    );
}

#[test]
fn cap010_sets_has_fatal_true() {
    let source = r#"
        model GPT = Model(id: "gpt-4", provider: "openai", caps: [])
        schema R { result: string }
        prompt P(x: string) requires json_mode -> R {
            user: "{x}"
        }
    "#;
    let (_, analysis) = analyze_source(source);
    assert!(
        analysis.has_fatal,
        "has_fatal should be true when CAP010 fires"
    );
}

#[test]
fn cap_prompt_no_requires_no_capability_check() {
    let source = r#"
        model GPT = Model(id: "gpt-4", provider: "openai", caps: [])
        schema R { result: string }
        prompt P(x: string) -> R {
            user: "{x}"
        }
    "#;
    let (_, analysis) = analyze_source(source);
    let cap_errors: Vec<_> = analysis
        .diagnostics
        .iter()
        .filter(|d| matches!(d.code, ErrorCode::Cap010))
        .collect();
    assert!(
        cap_errors.is_empty(),
        "No CAP010 when prompt has no requires clause"
    );
}

#[test]
fn cap_model_empty_caps_prompt_requires_produces_cap010() {
    let source = r#"
        model GPT = Model(id: "gpt-4", provider: "openai", caps: [])
        schema R { result: string }
        prompt P(x: string) requires streaming -> R {
            user: "{x}"
        }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Cap010);
}

// =============================================================================
// CAP001: Unknown capability (warning)
// =============================================================================

#[test]
fn cap_model_unknown_capability_produces_cap001() {
    let source = r#"
        model GPT = Model(id: "gpt-4", provider: "openai", caps: [teleportation])
        schema R { result: string }
        prompt P(x: string) -> R {
            user: "{x}"
        }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Cap001);
    let diag = analysis
        .diagnostics
        .iter()
        .find(|d| d.code == ErrorCode::Cap001)
        .unwrap();
    assert_eq!(diag.severity, Severity::Warning, "CAP001 should be warning");
    assert!(
        diag.message.contains("teleportation"),
        "CAP001 should mention the unknown cap name"
    );
}

#[test]
fn cap_prompt_requires_unknown_capability_produces_cap001() {
    let source = r#"
        model GPT = Model(id: "gpt-4", provider: "openai", caps: [json_mode])
        schema R { result: string }
        prompt P(x: string) requires telepathy -> R {
            user: "{x}"
        }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Cap001);
}

// =============================================================================
// CAP002: Duplicate capability (warning)
// =============================================================================

#[test]
fn cap_model_duplicate_capability_produces_cap002() {
    let source = r#"
        model GPT = Model(id: "gpt-4", provider: "openai", caps: [json_mode, json_mode])
        schema R { result: string }
        prompt P(x: string) -> R {
            user: "{x}"
        }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Cap002);
    let diag = analysis
        .diagnostics
        .iter()
        .find(|d| d.code == ErrorCode::Cap002)
        .unwrap();
    assert_eq!(diag.severity, Severity::Warning, "CAP002 should be warning");
}

#[test]
fn cap_prompt_duplicate_requires_produces_cap002() {
    let source = r#"
        model GPT = Model(id: "gpt-4", provider: "openai", caps: [json_mode])
        schema R { result: string }
        prompt P(x: string) requires [json_mode, json_mode] -> R {
            user: "{x}"
        }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Cap002);
}

// =============================================================================
// CAP020: json_mode + string return type (warning)
// =============================================================================

#[test]
fn cap_json_mode_with_string_return_produces_cap020() {
    let source = r#"
        model GPT = Model(id: "gpt-4", provider: "openai", caps: [json_mode])
        prompt P(x: string) requires json_mode -> string {
            user: "{x}"
        }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Cap020);
    let diag = analysis
        .diagnostics
        .iter()
        .find(|d| d.code == ErrorCode::Cap020)
        .unwrap();
    assert_eq!(diag.severity, Severity::Warning, "CAP020 should be warning");
}

#[test]
fn cap_json_mode_with_schema_return_no_cap020() {
    let source = r#"
        model GPT = Model(id: "gpt-4", provider: "openai", caps: [json_mode])
        schema R { result: string }
        prompt P(x: string) requires json_mode -> R {
            user: "{x}"
        }
    "#;
    let (_, analysis) = analyze_source(source);
    let cap020s: Vec<_> = analysis
        .diagnostics
        .iter()
        .filter(|d| d.code == ErrorCode::Cap020)
        .collect();
    assert!(
        cap020s.is_empty(),
        "json_mode with schema return should not produce CAP020"
    );
}

// =============================================================================
// Multiple prompts against agents' models
// =============================================================================

#[test]
fn cap_multiple_prompts_checked_against_agent_model() {
    let source = r#"
        model GPT = Model(id: "gpt-4", provider: "openai", caps: [json_mode])
        schema R { result: string }
        prompt P1(x: string) requires json_mode -> R {
            user: "{x}"
        }
        prompt P2(x: string) requires streaming -> R {
            user: "{x}"
        }
        agent A {
            model: GPT
        }
    "#;
    let (_, analysis) = analyze_source(source);
    // P1 should be fine (json_mode matches), P2 should produce CAP010 (streaming missing)
    assert_has_code(&analysis, ErrorCode::Cap010);
}

// =============================================================================
// Known capabilities produce no warnings
// =============================================================================

#[test]
fn cap_all_known_capabilities_no_warnings() {
    let source = r#"
        model GPT = Model(id: "gpt-4", provider: "openai", caps: [json_mode, streaming, function_calling, vision, code_interpreter])
        schema R { result: string }
        prompt P(x: string) -> R {
            user: "{x}"
        }
    "#;
    let (_, analysis) = analyze_source(source);
    let cap_warnings: Vec<_> = analysis
        .diagnostics
        .iter()
        .filter(|d| matches!(d.code, ErrorCode::Cap001 | ErrorCode::Cap002))
        .collect();
    assert!(
        cap_warnings.is_empty(),
        "All known caps should produce no CAP001/CAP002 warnings"
    );
}

#[test]
fn cap_no_models_no_cap_errors() {
    // Source with no models at all should not produce capability errors
    let source = r#"
        schema R { result: string }
        prompt P(x: string) -> R {
            user: "{x}"
        }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}
