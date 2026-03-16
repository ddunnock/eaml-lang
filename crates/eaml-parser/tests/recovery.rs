//! Error recovery tests for the EAML parser.
//!
//! Validates PAR-08: the parser produces diagnostics for each error,
//! recovers to parse subsequent valid declarations, respects the error
//! limit (20), and never panics on any input.

#[allow(dead_code)]
mod test_helpers;

use eaml_errors::ErrorCode;
use eaml_parser::ast::*;
use test_helpers::{error_count, has_code};

/// Helper: parse source and return output.
fn parse(source: &str) -> eaml_parser::ParseOutput {
    eaml_parser::parse(source)
}

// ===================================================================
// Test 1: Missing = in model, subsequent schema still parses
// ===================================================================

#[test]
fn recovery_missing_eq_in_model() {
    // model Haiku Model(...) -- missing '=' between name and 'Model'
    // schema S { x: int } -- should still parse
    let source = r#"model Haiku Model(id: "x", provider: "y", caps: [])
schema S { x: int }"#;
    let output = parse(source);

    // At least 1 error diagnostic
    assert!(error_count(&output) >= 1, "expected at least 1 error");

    // Should have emitted SYN050 (expected '=')
    assert!(
        has_code(&output, ErrorCode::Syn050),
        "expected Syn050 for missing '='"
    );

    // Should have at least 2 declarations (Error for model, Schema for S)
    assert!(
        output.program.declarations.len() >= 2,
        "expected at least 2 declarations, got {}",
        output.program.declarations.len()
    );

    // First decl should be Error (failed model parse)
    assert!(
        matches!(output.program.declarations[0], DeclId::Error(_)),
        "expected first decl to be Error, got {:?}",
        output.program.declarations[0]
    );

    // Second decl should be Schema (recovered)
    assert!(
        matches!(output.program.declarations[1], DeclId::Schema(_)),
        "expected second decl to be Schema, got {:?}",
        output.program.declarations[1]
    );
}

// ===================================================================
// Test 2: Missing { in schema, subsequent declaration still parses
// ===================================================================

#[test]
fn recovery_missing_brace_schema() {
    // schema S x: int } -- missing '{'
    // schema T { y: string }
    let source = r#"schema S x: int }
schema T { y: string }"#;
    let output = parse(source);

    assert!(error_count(&output) >= 1);

    // S should fail, T should parse
    let schema_count = output
        .program
        .declarations
        .iter()
        .filter(|d| matches!(d, DeclId::Schema(_)))
        .count();
    assert!(
        schema_count >= 1,
        "expected at least 1 valid schema, got {}",
        schema_count
    );
}

// ===================================================================
// Test 3: Multiple errors, valid declarations interspersed
// ===================================================================

#[test]
fn recovery_multiple_errors() {
    let source = r#"schema { x: int }
model = Model(id: "x", provider: "y", caps: [])
schema Valid { name: string }
prompt { }
"#;
    let output = parse(source);

    // At least 3 errors (broken schema, broken model, broken prompt)
    assert!(
        error_count(&output) >= 3,
        "expected at least 3 errors, got {}",
        error_count(&output)
    );

    // The valid schema should parse
    let has_valid_schema = output.program.declarations.iter().any(|d| {
        if let DeclId::Schema(id) = d {
            let s = &output.ast[*id];
            output.interner.resolve(&s.name) == "Valid"
        } else {
            false
        }
    });
    assert!(has_valid_schema, "expected 'Valid' schema to parse");
}

// ===================================================================
// Test 4: Error limit (20 errors)
// ===================================================================

#[test]
fn recovery_error_limit() {
    // Generate 25 broken schema declarations (each missing name)
    let mut source = String::new();
    for _ in 0..25 {
        source.push_str("schema { }\n");
    }
    // Add a valid schema at the end
    source.push_str("schema LastValid { z: bool }\n");

    let output = parse(&source);

    // Should have at least 20 errors (the limit)
    assert!(
        error_count(&output) >= 20,
        "expected at least 20 errors, got {}",
        error_count(&output)
    );

    // Parser should have stopped before processing all 26 declarations.
    // The valid last schema should NOT be reached because the error limit
    // stops parsing at 20 errors.
    let last_valid = output.program.declarations.iter().any(|d| {
        if let DeclId::Schema(id) = d {
            let s = &output.ast[*id];
            output.interner.resolve(&s.name) == "LastValid"
        } else {
            false
        }
    });
    assert!(
        !last_valid,
        "parser should have stopped before LastValid due to error limit"
    );
}

// ===================================================================
// Test 5: Broken header skips body
// ===================================================================

#[test]
fn recovery_broken_header_skips_body() {
    // schema without name -- body should be skipped, next decl parses
    let source = r#"schema { x: int, y: string }
schema Good { a: bool }"#;
    let output = parse(source);

    assert!(error_count(&output) >= 1);

    // Good schema should parse
    let has_good = output.program.declarations.iter().any(|d| {
        if let DeclId::Schema(id) = d {
            let s = &output.ast[*id];
            output.interner.resolve(&s.name) == "Good"
        } else {
            false
        }
    });
    assert!(
        has_good,
        "expected 'Good' schema to parse after broken header"
    );
}

// ===================================================================
// Test 6: Nested brace depth recovery
// ===================================================================

#[test]
fn recovery_nested_brace_depth() {
    // Broken tool with nested braces -- should not eat the next declaration
    let source = r#"tool Broken(x: int) -> int {
  if true { { } }
}
schema After { a: string }"#;
    let output = parse(source);

    // The tool body parsing might produce errors (native body not supported),
    // but 'After' schema should parse correctly
    let has_after = output.program.declarations.iter().any(|d| {
        if let DeclId::Schema(id) = d {
            let s = &output.ast[*id];
            output.interner.resolve(&s.name) == "After"
        } else {
            false
        }
    });
    assert!(
        has_after,
        "expected 'After' schema to parse after nested braces"
    );
}

// ===================================================================
// Test 7: Post-MVP keywords interleaved with valid code
// ===================================================================

#[test]
fn recovery_postmvp_interleaved() {
    let source = r#"pipeline P { }
schema S { x: int }
enum E { }
"#;
    let output = parse(source);

    // pipeline -> SYN080, enum -> SYN082
    assert!(
        has_code(&output, ErrorCode::Syn080),
        "expected Syn080 for pipeline"
    );
    assert!(
        has_code(&output, ErrorCode::Syn082),
        "expected Syn082 for enum"
    );

    // S should parse fine
    let has_schema = output.program.declarations.iter().any(|d| {
        if let DeclId::Schema(id) = d {
            let s = &output.ast[*id];
            output.interner.resolve(&s.name) == "S"
        } else {
            false
        }
    });
    assert!(
        has_schema,
        "expected schema S to parse between post-MVP keywords"
    );
}

// ===================================================================
// Test 8: Broken prompt body -- unknown field
// ===================================================================

#[test]
fn recovery_broken_prompt_body() {
    // Unknown field 'flavor' in prompt body
    let source = r#"prompt Bad(x: string) -> R {
  flavor: "sweet"
  user: "hello"
}
schema OK { a: int }"#;
    let output = parse(source);

    // Should have a SYN061 for the unexpected field
    assert!(
        has_code(&output, ErrorCode::Syn061),
        "expected Syn061 for unexpected prompt field"
    );

    // Schema OK should still parse
    let has_ok = output.program.declarations.iter().any(|d| {
        if let DeclId::Schema(id) = d {
            let s = &output.ast[*id];
            output.interner.resolve(&s.name) == "OK"
        } else {
            false
        }
    });
    assert!(
        has_ok,
        "expected 'OK' schema to parse after broken prompt body"
    );
}

// ===================================================================
// Test 9: Garbage input doesn't panic
// ===================================================================

#[test]
fn recovery_garbage_input() {
    let source = "!@#$%^&*()_+{}[]<>,.;:'\"\\|~`";
    let output = parse(source);

    // Should not panic (if we get here, it didn't)
    // Should have at least 1 diagnostic
    assert!(
        !output.diagnostics.is_empty(),
        "garbage input should produce at least 1 diagnostic"
    );
}

// ===================================================================
// Test 10: Empty input
// ===================================================================

#[test]
fn recovery_empty_input() {
    let output = parse("");
    assert_eq!(output.program.declarations.len(), 0);
    assert_eq!(error_count(&output), 0);
}

// ===================================================================
// Test 11: Various garbage inputs to verify no panics
// ===================================================================

#[test]
fn recovery_no_panic_on_various_inputs() {
    let inputs = vec![
        "",
        "   ",
        "\n\n\n",
        "// just a comment",
        "/* block comment */",
        "schema",
        "model",
        "prompt",
        "tool",
        "agent",
        "import",
        "let",
        "schema {",
        "schema S {",
        "schema S { x: }",
        "schema S { x: int, y: }",
        "model X =",
        "model X = Model(",
        "model X = Model(id:",
        "prompt P(",
        "prompt P() ->",
        "prompt P() -> R {",
        "prompt P() -> R { user:",
        "tool T() -> R {",
        r#"tool T() -> R { python %{ code }% }"#,
        "agent A {",
        "agent A { model:",
        "agent A { tools: [",
        "let x: int =",
        "let x:",
        r#"import "#,
        "{ } { } { }",
        "))))",
        "]]]]",
        "\0\x01\x02",
    ];

    for input in inputs {
        // Should not panic for any input
        let _output = parse(input);
    }
}

// ===================================================================
// Test 12: Diagnostic spans point to valid source locations
// ===================================================================

#[test]
fn recovery_diagnostic_spans_valid() {
    let source = r#"schema { x: int }
model = bad
schema Good { a: string }"#;
    let output = parse(source);
    let source_len = source.len();

    for (i, diag) in output.diagnostics.iter().enumerate() {
        assert!(
            diag.span.end <= source_len,
            "diagnostic {} span end {} exceeds source len {}",
            i,
            diag.span.end,
            source_len
        );
        assert!(
            diag.span.start <= diag.span.end,
            "diagnostic {} has inverted span: {}..{}",
            i,
            diag.span.start,
            diag.span.end
        );
    }
}
