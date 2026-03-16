//! Integration tests confirming all Phase 3 error codes are emittable (SEM-11).
//!
//! Each test triggers exactly one specific error code, confirming the semantic
//! analysis pipeline can produce every diagnostic code it is responsible for.
//!
//! Error code coverage map:
//! ---------------------------------------------------------------
//! Code    | Test function                           | Category
//! --------|----------------------------------------|----------
//! RES001  | sem11_res001_fires                      | Resolution
//! RES010  | sem11_res010_fires                      | Resolution
//! SEM010  | sem11_sem010_fires                      | Semantic
//! SEM020  | sem11_sem020_fires                      | Semantic
//! SEM025  | sem11_sem025_fires                      | Semantic
//! SEM030  | sem11_sem030_fires                      | Semantic
//! SEM040  | sem11_sem040_fires                      | Semantic
//! SEM060  | sem11_sem060_fires                      | Semantic
//! SEM070  | sem11_sem070_fires                      | Semantic
//! TYP001  | sem11_typ001_fires                      | Type
//! TYP010  | sem11_typ010_fires                      | Type (unused)
//! TYP030  | sem11_typ030_fires                      | Type
//! TYP031  | sem11_typ031_untestable                 | Type (note)
//! TYP032  | sem11_typ032_fires                      | Type
//! TYP040  | sem11_typ040_fires                      | Type
//! CAP001  | sem11_cap001_fires                      | Capability
//! CAP002  | sem11_cap002_fires                      | Capability
//! CAP010  | sem11_cap010_fires                      | Capability
//! CAP020  | sem11_cap020_fires                      | Capability
//! PYB010  | sem11_pyb010_fires                      | Python bridge
//! ---------------------------------------------------------------

mod test_helpers;

use eaml_errors::{ErrorCode, Severity};
use test_helpers::{analyze_source, assert_has_code, assert_no_errors};

// =============================================================================
// RES: Resolution errors
// =============================================================================

#[test]
fn sem11_res001_fires() {
    let (_, output) = analyze_source(r#"schema S { x: UnknownType }"#);
    assert_has_code(&output, ErrorCode::Res001);
}

#[test]
fn sem11_res010_fires() {
    let (_, output) = analyze_source(
        r#"
        schema Dup { x: string }
        schema Dup { y: int }
    "#,
    );
    assert_has_code(&output, ErrorCode::Res010);
}

// =============================================================================
// SEM: Semantic errors
// =============================================================================

#[test]
fn sem11_sem010_fires() {
    // SEM010: python import after non-import declaration
    let (_, output) = analyze_source(
        r#"
        schema S { x: string }
        import python "os" as os_mod
    "#,
    );
    assert_has_code(&output, ErrorCode::Sem010);
}

#[test]
fn sem11_sem020_fires() {
    // SEM020: duplicate field in schema
    let (_, output) = analyze_source(r#"schema S { x: string  x: int }"#);
    assert_has_code(&output, ErrorCode::Sem020);
}

#[test]
fn sem11_sem025_fires() {
    // SEM025: prompt missing user field
    let (_, output) = analyze_source(
        r#"
        schema R { text: string }
        prompt P() -> R {
            system: "You are helpful"
        }
    "#,
    );
    assert_has_code(&output, ErrorCode::Sem025);
}

#[test]
fn sem11_sem030_fires() {
    // SEM030: unknown bounded parameter name
    let (_, output) = analyze_source(r#"schema S { score: float<foo: 1.0> }"#);
    assert_has_code(&output, ErrorCode::Sem030);
}

#[test]
fn sem11_sem040_fires() {
    // SEM040: empty tool body
    let (_, output) = analyze_source(r#"tool T(x: string) -> string { }"#);
    assert_has_code(&output, ErrorCode::Sem040);
}

#[test]
fn sem11_sem060_fires() {
    // SEM060: chained comparison (detected via let binding)
    let (_, output) = analyze_source(r#"let x: bool = 1 == 2 == 3"#);
    assert_has_code(&output, ErrorCode::Sem060);
}

#[test]
fn sem11_sem070_fires() {
    // SEM070: recursive schema reference (cycle detection)
    let (_, output) = analyze_source(
        r#"
        schema TreeNode {
            value: string
            children: TreeNode[]
        }
    "#,
    );
    assert_has_code(&output, ErrorCode::Sem070);
}

// =============================================================================
// TYP: Type errors
// =============================================================================

#[test]
fn sem11_typ001_fires() {
    // TYP001: schema name shadows built-in type
    let (_, output) = analyze_source(r#"schema string { value: int }"#);
    assert_has_code(&output, ErrorCode::Typ001);
}

#[test]
fn sem11_typ010_note() {
    // TYP010 is defined as "Incompatible types in union" in ErrorCode enum.
    // In the current semantic analysis, TYP010 is not emitted from source --
    // it's reserved for future type-level checking. Verify the code exists.
    assert_eq!(ErrorCode::Typ010.to_string(), "TYP010");
}

#[test]
fn sem11_typ030_fires() {
    // TYP030: lower bound exceeds upper bound
    let (_, output) = analyze_source(r#"schema S { score: float<5.0, 1.0> }"#);
    assert_has_code(&output, ErrorCode::Typ030);
}

#[test]
fn sem11_typ031_untestable() {
    // TYP031: negative string bound.
    // The parser cannot parse negative numeric literals in bounded type params
    // (e.g., string<-1, 10> fails at '-'). The code path exists but is
    // untestable from source in v0.1.
    assert_eq!(ErrorCode::Typ031.to_string(), "TYP031");
}

#[test]
fn sem11_typ032_fires() {
    // TYP032: bounds on non-boundable type
    let (_, output) = analyze_source(r#"schema S { flag: bool<0, 1> }"#);
    assert_has_code(&output, ErrorCode::Typ032);
}

#[test]
fn sem11_typ040_fires() {
    // TYP040: duplicate literal union member
    let (_, output) = analyze_source(r#"schema S { sentiment: "pos" | "pos" }"#);
    assert_has_code(&output, ErrorCode::Typ040);
}

// =============================================================================
// CAP: Capability errors
// =============================================================================

#[test]
fn sem11_cap001_fires() {
    // CAP001: unknown capability
    let (_, output) =
        analyze_source(r#"model M = Model(id: "x", provider: "openai", caps: [teleportation])"#);
    assert_has_code(&output, ErrorCode::Cap001);
}

#[test]
fn sem11_cap002_fires() {
    // CAP002: duplicate capability
    let (_, output) = analyze_source(
        r#"model M = Model(id: "x", provider: "openai", caps: [json_mode, json_mode])"#,
    );
    assert_has_code(&output, ErrorCode::Cap002);
}

#[test]
fn sem11_cap010_fires() {
    // CAP010: capability mismatch (FATAL)
    let (_, output) = analyze_source(
        r#"
        model M = Model(id: "x", provider: "openai", caps: [])
        schema R { result: string }
        prompt P(x: string) requires json_mode -> R {
            user: "{x}"
        }
    "#,
    );
    assert_has_code(&output, ErrorCode::Cap010);
    let diag = output
        .diagnostics
        .iter()
        .find(|d| d.code == ErrorCode::Cap010)
        .unwrap();
    assert_eq!(diag.severity, Severity::Fatal);
    assert!(output.has_fatal, "has_fatal must be true for CAP010");
}

#[test]
fn sem11_cap020_fires() {
    // CAP020: json_mode + string return type
    let (_, output) = analyze_source(
        r#"
        model M = Model(id: "x", provider: "openai", caps: [json_mode])
        prompt P(x: string) requires json_mode -> string {
            user: "{x}"
        }
    "#,
    );
    assert_has_code(&output, ErrorCode::Cap020);
}

// =============================================================================
// PYB: Python bridge errors
// =============================================================================

#[test]
fn sem11_pyb010_fires() {
    // PYB010: unknown provider
    let (_, output) =
        analyze_source(r#"model M = Model(id: "x", provider: "unknown_provider", caps: [])"#);
    assert_has_code(&output, ErrorCode::Pyb010);
}

// =============================================================================
// Example file regression tests
// =============================================================================

#[test]
fn example_01_minimal_no_errors() {
    let source = include_str!("../../../examples/01-minimal/minimal.eaml");
    let (_, output) = analyze_source(source);
    assert_no_errors(&output);
}

#[test]
fn example_02_sentiment_no_errors() {
    let source = include_str!("../../../examples/02-sentiment/sentiment.eaml");
    let (_, output) = analyze_source(source);
    assert_no_errors(&output);
}

#[test]
fn example_06_capability_error_produces_cap010() {
    let source = include_str!("../../../examples/06-capability-error/bad_model.eaml");
    let (_, output) = analyze_source(source);
    assert_has_code(&output, ErrorCode::Cap010);
    assert!(
        output.has_fatal,
        "bad_model.eaml should produce a FATAL CAP010"
    );
}

#[test]
fn example_07_all_type_variants_no_errors() {
    let source = include_str!("../../../examples/07-all-type-variants/types.eaml");
    let (_, output) = analyze_source(source);
    assert_no_errors(&output);
}
