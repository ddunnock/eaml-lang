//! Template string variable scoping tests for eaml-semantic (SEM-10).

mod test_helpers;

use eaml_errors::ErrorCode;
use test_helpers::{analyze_source, assert_has_code, assert_no_errors};

// =============================================================================
// Prompt parameter in scope for template interpolation
// =============================================================================

#[test]
fn scoping_prompt_param_in_scope() {
    let source = r#"
        schema R { result: string }
        prompt Greet(name: string) -> R {
            user: "Hello {name}"
        }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

// =============================================================================
// Top-level let binding in scope for template interpolation
// =============================================================================

#[test]
fn scoping_let_binding_in_scope() {
    let source = r#"
        let greeting: string = "hello"
        schema R { text: string }
        prompt P() -> R {
            user: "{greeting} world"
        }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

// =============================================================================
// Schema field names NOT in scope for template interpolation
// =============================================================================

#[test]
fn scoping_schema_field_not_in_scope() {
    // Schema field "result" should not be available in template scope.
    // The param x is of type R, but "result" (a field of R) is not in scope.
    let source = r#"
        schema R { result: string }
        prompt Bad(x: R) -> R {
            user: "The result is {result}"
        }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Res001);
    let diag = analysis
        .diagnostics
        .iter()
        .find(|d| d.code == ErrorCode::Res001 && d.message.contains("result"))
        .expect("Should have RES001 for 'result' not in scope");
    assert!(
        diag.message.contains("template interpolation"),
        "Error should mention template interpolation context"
    );
}

// =============================================================================
// Undefined variable in template produces RES001
// =============================================================================

#[test]
fn scoping_undefined_variable_produces_res001() {
    let source = r#"
        schema R { text: string }
        prompt P() -> R {
            user: "Hello {undefined}"
        }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Res001);
}

// =============================================================================
// Field access on param -- root object must be in scope
// =============================================================================

#[test]
fn scoping_field_access_on_param_ok() {
    let source = r#"
        schema Input { name: string }
        schema R { text: string }
        prompt P(x: Input) -> R {
            user: "Hello {x.name}"
        }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

#[test]
fn scoping_field_access_on_undefined_root_produces_res001() {
    let source = r#"
        schema R { text: string }
        prompt P() -> R {
            user: "Hello {unknown.field}"
        }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Res001);
}

// =============================================================================
// System template also validates scope
// =============================================================================

#[test]
fn scoping_system_template_validates() {
    let source = r#"
        schema R { text: string }
        prompt P(role: string) -> R {
            system: "You are a {role}"
            user: "Hello"
        }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}

#[test]
fn scoping_system_template_undefined_var() {
    let source = r#"
        schema R { text: string }
        prompt P() -> R {
            system: "You are a {missing_role}"
            user: "Hello"
        }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_has_code(&analysis, ErrorCode::Res001);
}

// =============================================================================
// Multiple params all in scope
// =============================================================================

#[test]
fn scoping_multiple_params_in_scope() {
    let source = r#"
        schema R { text: string }
        prompt P(first: string, last: string) -> R {
            user: "Hello {first} {last}"
        }
    "#;
    let (_, analysis) = analyze_source(source);
    assert_no_errors(&analysis);
}
