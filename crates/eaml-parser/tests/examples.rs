//! Integration tests parsing all example .eaml files.
//!
//! Validates PAR-01: all populated example files parse into correct
//! AST structures with 0 diagnostics.

#[allow(dead_code)]
mod test_helpers;

use eaml_parser::ast::*;
use test_helpers::parse_example;

// ===================================================================
// 01-minimal/minimal.eaml
// ===================================================================

#[test]
fn example_minimal_parses() {
    let source = include_str!("../../../examples/01-minimal/minimal.eaml");
    let output = parse_example(source);

    // 3 declarations: model, schema, prompt
    assert_eq!(output.program.declarations.len(), 3);
    assert!(matches!(output.program.declarations[0], DeclId::Model(_)));
    assert!(matches!(output.program.declarations[1], DeclId::Schema(_)));
    assert!(matches!(output.program.declarations[2], DeclId::Prompt(_)));
}

#[test]
fn example_minimal_model_structure() {
    let source = include_str!("../../../examples/01-minimal/minimal.eaml");
    let output = parse_example(source);

    if let DeclId::Model(id) = &output.program.declarations[0] {
        let m = &output.ast[*id];
        assert_eq!(output.interner.resolve(&m.name), "Haiku");
        assert!(
            m.caps.is_empty(),
            "minimal model should have empty caps (got {:?})",
            m.caps
        );
    } else {
        panic!("expected Model declaration");
    }
}

#[test]
fn example_minimal_schema_structure() {
    let source = include_str!("../../../examples/01-minimal/minimal.eaml");
    let output = parse_example(source);

    if let DeclId::Schema(id) = &output.program.declarations[1] {
        let s = &output.ast[*id];
        assert_eq!(output.interner.resolve(&s.name), "Greeting");
        assert_eq!(s.fields.len(), 2);
        assert_eq!(output.interner.resolve(&s.fields[0].name), "message");
        assert_eq!(output.interner.resolve(&s.fields[1].name), "word_count");
    } else {
        panic!("expected Schema declaration");
    }
}

#[test]
fn example_minimal_prompt_structure() {
    let source = include_str!("../../../examples/01-minimal/minimal.eaml");
    let output = parse_example(source);

    if let DeclId::Prompt(id) = &output.program.declarations[2] {
        let p = &output.ast[*id];
        assert_eq!(output.interner.resolve(&p.name), "Greet");
        assert_eq!(p.params.len(), 1);
        assert_eq!(output.interner.resolve(&p.params[0].name), "name");
        assert!(p.requires.is_none(), "minimal prompt has no requires");
        // Body should have a user field
        assert!(
            p.body
                .fields
                .iter()
                .any(|f| matches!(f, PromptField::User(_))),
            "prompt should have user field"
        );
    } else {
        panic!("expected Prompt declaration");
    }
}

// ===================================================================
// 02-sentiment/sentiment.eaml
// ===================================================================

#[test]
fn example_sentiment_parses() {
    let source = include_str!("../../../examples/02-sentiment/sentiment.eaml");
    let output = parse_example(source);

    assert_eq!(output.program.declarations.len(), 3);
    assert!(matches!(output.program.declarations[0], DeclId::Model(_)));
    assert!(matches!(output.program.declarations[1], DeclId::Schema(_)));
    assert!(matches!(output.program.declarations[2], DeclId::Prompt(_)));
}

#[test]
fn example_sentiment_model_with_caps() {
    let source = include_str!("../../../examples/02-sentiment/sentiment.eaml");
    let output = parse_example(source);

    if let DeclId::Model(id) = &output.program.declarations[0] {
        let m = &output.ast[*id];
        assert_eq!(output.interner.resolve(&m.name), "Sonnet");
        assert_eq!(m.caps.len(), 2);
        let cap_names: Vec<&str> = m
            .caps
            .iter()
            .map(|(s, _)| output.interner.resolve(s))
            .collect();
        assert!(cap_names.contains(&"json_mode"));
        assert!(cap_names.contains(&"streaming"));
    } else {
        panic!("expected Model declaration");
    }
}

#[test]
fn example_sentiment_schema_with_literal_union() {
    let source = include_str!("../../../examples/02-sentiment/sentiment.eaml");
    let output = parse_example(source);

    if let DeclId::Schema(id) = &output.program.declarations[1] {
        let s = &output.ast[*id];
        assert_eq!(output.interner.resolve(&s.name), "SentimentResult");
        assert_eq!(s.fields.len(), 3);
        // First field has a literal union type
        let te = &output.ast[s.fields[0].type_expr];
        assert!(
            matches!(te, TypeExpr::LiteralUnion { .. }),
            "sentiment field should be literal union, got {:?}",
            te
        );
    } else {
        panic!("expected Schema declaration");
    }
}

#[test]
fn example_sentiment_prompt_with_requires() {
    let source = include_str!("../../../examples/02-sentiment/sentiment.eaml");
    let output = parse_example(source);

    if let DeclId::Prompt(id) = &output.program.declarations[2] {
        let p = &output.ast[*id];
        assert_eq!(output.interner.resolve(&p.name), "AnalyzeSentiment");
        assert!(p.requires.is_some());
        let req = p.requires.as_ref().unwrap();
        assert_eq!(req.caps.len(), 1);
        assert_eq!(output.interner.resolve(&req.caps[0].0), "json_mode");

        // Should have system, user, temperature, max_tokens fields
        let has_system = p
            .body
            .fields
            .iter()
            .any(|f| matches!(f, PromptField::System(_)));
        let has_user = p
            .body
            .fields
            .iter()
            .any(|f| matches!(f, PromptField::User(_)));
        let has_temp = p
            .body
            .fields
            .iter()
            .any(|f| matches!(f, PromptField::Temperature(_)));
        let has_max = p
            .body
            .fields
            .iter()
            .any(|f| matches!(f, PromptField::MaxTokens(_)));
        assert!(has_system, "expected system field");
        assert!(has_user, "expected user field");
        assert!(has_temp, "expected temperature field");
        assert!(has_max, "expected max_tokens field");
    } else {
        panic!("expected Prompt declaration");
    }
}

// ===================================================================
// 07-all-type-variants/types.eaml
// ===================================================================

#[test]
fn example_types_parses() {
    let source = include_str!("../../../examples/07-all-type-variants/types.eaml");
    let output = parse_example(source);

    // 1 model + 6 schemas + 3 prompts = 10 declarations
    assert_eq!(output.program.declarations.len(), 10);
}

#[test]
fn example_types_declaration_kinds() {
    let source = include_str!("../../../examples/07-all-type-variants/types.eaml");
    let output = parse_example(source);

    let model_count = output
        .program
        .declarations
        .iter()
        .filter(|d| matches!(d, DeclId::Model(_)))
        .count();
    let schema_count = output
        .program
        .declarations
        .iter()
        .filter(|d| matches!(d, DeclId::Schema(_)))
        .count();
    let prompt_count = output
        .program
        .declarations
        .iter()
        .filter(|d| matches!(d, DeclId::Prompt(_)))
        .count();

    assert_eq!(model_count, 1);
    assert_eq!(schema_count, 6);
    assert_eq!(prompt_count, 3);
}

#[test]
fn example_types_primitives_schema() {
    let source = include_str!("../../../examples/07-all-type-variants/types.eaml");
    let output = parse_example(source);

    // Second declaration should be Primitives schema
    if let DeclId::Schema(id) = &output.program.declarations[1] {
        let s = &output.ast[*id];
        assert_eq!(output.interner.resolve(&s.name), "Primitives");
        assert_eq!(s.fields.len(), 5);

        // All fields should be Named type expressions
        for field in &s.fields {
            let te = &output.ast[field.type_expr];
            assert!(
                matches!(te, TypeExpr::Named(..)),
                "field {} expected Named type, got {:?}",
                output.interner.resolve(&field.name),
                te
            );
        }
    } else {
        panic!("expected Schema declaration for Primitives");
    }
}

#[test]
fn example_types_bounded_schema() {
    let source = include_str!("../../../examples/07-all-type-variants/types.eaml");
    let output = parse_example(source);

    // Third declaration should be BoundedTypes schema
    if let DeclId::Schema(id) = &output.program.declarations[2] {
        let s = &output.ast[*id];
        assert_eq!(output.interner.resolve(&s.name), "BoundedTypes");
        assert_eq!(s.fields.len(), 6);

        // All fields should have Bounded type expressions
        for field in &s.fields {
            let te = &output.ast[field.type_expr];
            assert!(
                matches!(te, TypeExpr::Bounded { .. }),
                "field {} expected Bounded type, got {:?}",
                output.interner.resolve(&field.name),
                te
            );
        }
    } else {
        panic!("expected Schema declaration for BoundedTypes");
    }
}

#[test]
fn example_types_composite_schema() {
    let source = include_str!("../../../examples/07-all-type-variants/types.eaml");
    let output = parse_example(source);

    // Fourth declaration should be CompositeTypes schema
    if let DeclId::Schema(id) = &output.program.declarations[3] {
        let s = &output.ast[*id];
        assert_eq!(output.interner.resolve(&s.name), "CompositeTypes");
        assert_eq!(s.fields.len(), 6);

        // required_name: string (Named)
        assert!(matches!(
            output.ast[s.fields[0].type_expr],
            TypeExpr::Named(..)
        ));
        // optional_name: string? (Optional)
        assert!(matches!(
            output.ast[s.fields[1].type_expr],
            TypeExpr::Optional(..)
        ));
        // tag_list: int[] (Array)
        assert!(matches!(
            output.ast[s.fields[2].type_expr],
            TypeExpr::Array(..)
        ));
        // optional_tags: string[]? (Optional(Array))
        assert!(matches!(
            output.ast[s.fields[3].type_expr],
            TypeExpr::Optional(..)
        ));
        // nullable_items: string?[] (Array(Optional))
        assert!(matches!(
            output.ast[s.fields[4].type_expr],
            TypeExpr::Array(..)
        ));
        // fully_optional: string?[]? (Optional(Array(Optional)))
        assert!(matches!(
            output.ast[s.fields[5].type_expr],
            TypeExpr::Optional(..)
        ));
    } else {
        panic!("expected Schema declaration for CompositeTypes");
    }
}

#[test]
fn example_types_union_schema() {
    let source = include_str!("../../../examples/07-all-type-variants/types.eaml");
    let output = parse_example(source);

    // Fifth declaration should be UnionTypes schema
    if let DeclId::Schema(id) = &output.program.declarations[4] {
        let s = &output.ast[*id];
        assert_eq!(output.interner.resolve(&s.name), "UnionTypes");
        assert_eq!(s.fields.len(), 3);

        // All fields should have LiteralUnion type expressions
        for field in &s.fields {
            let te = &output.ast[field.type_expr];
            assert!(
                matches!(te, TypeExpr::LiteralUnion { .. }),
                "field {} expected LiteralUnion type, got {:?}",
                output.interner.resolve(&field.name),
                te
            );
        }
    } else {
        panic!("expected Schema declaration for UnionTypes");
    }
}

// ===================================================================
// 06-capability-error/bad_model.eaml
// ===================================================================

#[test]
fn example_capability_error_parses() {
    let source = include_str!("../../../examples/06-capability-error/bad_model.eaml");
    let output = parse_example(source);

    // Parser should succeed -- capability errors are semantic
    assert_eq!(output.program.declarations.len(), 3);
    assert!(matches!(output.program.declarations[0], DeclId::Model(_)));
    assert!(matches!(output.program.declarations[1], DeclId::Schema(_)));
    assert!(matches!(output.program.declarations[2], DeclId::Prompt(_)));
}

#[test]
fn example_capability_error_model_empty_caps() {
    let source = include_str!("../../../examples/06-capability-error/bad_model.eaml");
    let output = parse_example(source);

    if let DeclId::Model(id) = &output.program.declarations[0] {
        let m = &output.ast[*id];
        assert_eq!(output.interner.resolve(&m.name), "WeakModel");
        assert!(
            m.caps.is_empty(),
            "WeakModel should have empty caps (got {:?})",
            m.caps
        );
    } else {
        panic!("expected Model declaration");
    }
}

#[test]
fn example_capability_error_prompt_requires() {
    let source = include_str!("../../../examples/06-capability-error/bad_model.eaml");
    let output = parse_example(source);

    if let DeclId::Prompt(id) = &output.program.declarations[2] {
        let p = &output.ast[*id];
        assert_eq!(output.interner.resolve(&p.name), "AnalyzeText");
        assert!(p.requires.is_some());
        let req = p.requires.as_ref().unwrap();
        assert_eq!(req.caps.len(), 1);
        assert_eq!(output.interner.resolve(&req.caps[0].0), "json_mode");
    } else {
        panic!("expected Prompt declaration");
    }
}
