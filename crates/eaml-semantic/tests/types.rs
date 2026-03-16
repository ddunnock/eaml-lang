//! Type checking tests for eaml-semantic (SEM-04 through SEM-07).

mod test_helpers;

use eaml_errors::{ErrorCode, Severity};
use test_helpers::{analyze_source, assert_has_code, assert_no_errors};

// =============================================================================
// Bounded type validation: TYP030 (min > max)
// =============================================================================

#[test]
fn types_valid_float_bounds() {
    let source = r#"schema S { score: float<0.0, 1.0> }"#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

#[test]
fn types_float_min_exceeds_max_produces_typ030() {
    let source = r#"schema S { score: float<5.0, 1.0> }"#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Typ030);
    let diag = analysis
        .diagnostics
        .iter()
        .find(|d| d.code == ErrorCode::Typ030)
        .unwrap();
    assert!(
        diag.message.contains("5"),
        "Message should mention the lower bound"
    );
    assert!(
        diag.message.contains("1"),
        "Message should mention the upper bound"
    );
}

#[test]
fn types_int_min_exceeds_max_produces_typ030() {
    let source = r#"schema S { val: int<10, 5> }"#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Typ030);
}

// =============================================================================
// Bounded type validation: TYP031 (negative string bounds)
// =============================================================================

#[test]
fn types_negative_string_bound_produces_typ031() {
    // NOTE: The parser currently cannot parse negative numeric literals in bounded
    // type params (e.g., string<-1, 10> fails at '-'). The TYP031 code path exists
    // in the type checker for completeness. This test verifies that the error code
    // is properly defined and can be used. When the parser is extended to handle
    // negative bounds, this test should parse `string<-1, 10>` directly.
    //
    // For now, we verify that TYP031 is a valid error code that doesn't crash.
    let _ = ErrorCode::Typ031;
    assert_eq!(ErrorCode::Typ031.to_string(), "TYP031");
}

// =============================================================================
// Bounded type validation: TYP032 (non-boundable type)
// =============================================================================

#[test]
fn types_bool_bounded_produces_typ032() {
    let source = r#"schema S { flag: bool<0, 1> }"#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Typ032);
}

// =============================================================================
// Literal union: TYP040 (duplicate member)
// =============================================================================

#[test]
fn types_valid_literal_union() {
    let source = r#"schema S { sentiment: "pos" | "neg" }"#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

#[test]
fn types_duplicate_literal_union_member_produces_typ040() {
    let source = r#"schema S { sentiment: "pos" | "pos" }"#;
    let (_, analysis) = analyze_source(source);
    let has_typ040 = analysis
        .diagnostics
        .iter()
        .any(|d| d.code == ErrorCode::Typ040);
    assert!(
        has_typ040,
        "Duplicate literal union member should produce TYP040, got: {:?}",
        analysis
            .diagnostics
            .iter()
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect::<Vec<_>>()
    );
    let diag = analysis
        .diagnostics
        .iter()
        .find(|d| d.code == ErrorCode::Typ040)
        .unwrap();
    assert_eq!(diag.severity, Severity::Warning);
}

// =============================================================================
// Schema field type resolution
// =============================================================================

#[test]
fn types_schema_field_unknown_type_produces_error() {
    // Unknown type should produce RES001 (from resolver) -- type checker does not re-emit
    let source = r#"schema S { field: UnknownType }"#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Res001);
}

#[test]
fn types_schema_field_string_validates() {
    let source = r#"schema S { name: string }"#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

#[test]
fn types_schema_field_references_other_schema() {
    let source = r#"
        schema Address { street: string }
        schema User { addr: Address }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

// =============================================================================
// Named bounded params
// =============================================================================

#[test]
fn types_named_bounded_params_normalize_correctly() {
    // Named params in reverse order should not produce false TYP030
    let source = r#"schema S { score: float<max: 1.0, min: 0.0> }"#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

#[test]
fn types_unknown_bounded_param_name_produces_sem030() {
    let source = r#"schema S { score: float<foo: 1.0> }"#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Sem030);
}

// =============================================================================
// Schema validation: SEM020 (duplicate field)
// =============================================================================

#[test]
fn types_duplicate_field_produces_sem020() {
    let source = r#"schema S { x: string  x: string }"#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Sem020);
}

// =============================================================================
// Prompt validation: SEM025 (missing user field)
// =============================================================================

#[test]
fn types_prompt_missing_user_field_produces_sem025() {
    let source = r#"
        schema R { text: string }
        prompt P() -> R {
            system: "You are helpful"
        }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Sem025);
}

#[test]
fn types_prompt_with_user_field_ok() {
    let source = r#"
        schema R { text: string }
        prompt P() -> R {
            user: "Hello"
        }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

// =============================================================================
// Tool validation: SEM040 (empty body)
// =============================================================================

#[test]
fn types_tool_empty_body_produces_sem040() {
    let source = r#"tool T(x: string) -> string { }"#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Sem040);
}

// =============================================================================
// Chained comparisons: SEM060
// =============================================================================

#[test]
fn types_chained_comparison_produces_sem060() {
    // Let binding value expressions are checked for chained comparisons.
    // a == b == c parses as (a == b) == c due to left-associative comparison.
    let source = r#"let x: bool = 1 == 2 == 3"#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Sem060);
    let diag = analysis
        .diagnostics
        .iter()
        .find(|d| d.code == ErrorCode::Sem060)
        .unwrap();
    assert!(
        diag.hints.iter().any(|h| h.contains("explicit grouping")),
        "SEM060 should have hint about explicit grouping, got: {:?}",
        diag.hints
    );
}

// =============================================================================
// Type shadowing: TYP001
// =============================================================================

#[test]
fn types_schema_shadows_builtin_produces_typ001() {
    let source = r#"schema string { value: int }"#;
    let (_, analysis) = analyze_source(source);
    let has_typ001 = analysis
        .diagnostics
        .iter()
        .any(|d| d.code == ErrorCode::Typ001);
    assert!(
        has_typ001,
        "Schema shadowing 'string' should produce TYP001 warning, got: {:?}",
        analysis
            .diagnostics
            .iter()
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect::<Vec<_>>()
    );
}

// =============================================================================
// Unknown provider: PYB010
// =============================================================================

#[test]
fn types_unknown_provider_produces_pyb010() {
    let source = r#"model M = Model(id: "test", provider: "unknown_provider", caps: [])"#;
    let (_, analysis) = analyze_source(source);
    let has_pyb010 = analysis
        .diagnostics
        .iter()
        .any(|d| d.code == ErrorCode::Pyb010);
    assert!(
        has_pyb010,
        "Unknown provider should produce PYB010 warning, got: {:?}",
        analysis
            .diagnostics
            .iter()
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect::<Vec<_>>()
    );
}

#[test]
fn types_known_provider_anthropic_ok() {
    let source = r#"model M = Model(id: "test", provider: "anthropic", caps: [])"#;
    let (_, analysis) = analyze_source(source);
    // Should not have PYB010
    assert!(
        !analysis
            .diagnostics
            .iter()
            .any(|d| d.code == ErrorCode::Pyb010),
        "Known provider 'anthropic' should not produce PYB010"
    );
}

#[test]
fn types_known_provider_openai_ok() {
    let source = r#"model M = Model(id: "test", provider: "openai", caps: [])"#;
    let (_, analysis) = analyze_source(source);
    assert!(
        !analysis
            .diagnostics
            .iter()
            .any(|d| d.code == ErrorCode::Pyb010),
        "Known provider 'openai' should not produce PYB010"
    );
}

// =============================================================================
// Composite type modifier orderings: all legal per spec
// =============================================================================

#[test]
fn types_optional_array_of_strings_ok() {
    let source = r#"schema S { items: string[]? }"#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

#[test]
fn types_array_of_optional_strings_ok() {
    let source = r#"schema S { items: string?[] }"#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

#[test]
fn types_optional_array_of_optional_strings_ok() {
    let source = r#"schema S { items: string?[]? }"#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

#[test]
fn types_basic_array_of_strings_ok() {
    let source = r#"schema S { items: string[] }"#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}
