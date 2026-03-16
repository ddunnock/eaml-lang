mod test_helpers;

use eaml_codegen::types::{emit_field_line, emit_type_annotation, is_optional, ImportTracker};
use eaml_codegen::writer::CodeWriter;
use eaml_parser::ast::*;
use eaml_semantic::type_checker::ResolvedType;

// Helper to create a minimal Ast + Interner for type annotation tests
fn make_ast_interner() -> (Ast, eaml_lexer::Interner) {
    let ast = Ast::new();
    let interner = eaml_lexer::Interner::default();
    (ast, interner)
}

// ==========================================================================
// emit_type_annotation tests
// ==========================================================================

#[test]
fn type_annotation_string() {
    let (ast, interner) = make_ast_interner();
    let resolved = ResolvedType::Primitive("string".to_string());
    assert_eq!(emit_type_annotation(&resolved, &ast, &interner), "str");
}

#[test]
fn type_annotation_int() {
    let (ast, interner) = make_ast_interner();
    let resolved = ResolvedType::Primitive("int".to_string());
    assert_eq!(emit_type_annotation(&resolved, &ast, &interner), "int");
}

#[test]
fn type_annotation_float() {
    let (ast, interner) = make_ast_interner();
    let resolved = ResolvedType::Primitive("float".to_string());
    assert_eq!(emit_type_annotation(&resolved, &ast, &interner), "float");
}

#[test]
fn type_annotation_bool() {
    let (ast, interner) = make_ast_interner();
    let resolved = ResolvedType::Primitive("bool".to_string());
    assert_eq!(emit_type_annotation(&resolved, &ast, &interner), "bool");
}

#[test]
fn type_annotation_null() {
    let (ast, interner) = make_ast_interner();
    let resolved = ResolvedType::Primitive("null".to_string());
    assert_eq!(emit_type_annotation(&resolved, &ast, &interner), "None");
}

#[test]
fn type_annotation_schema() {
    let mut ast = Ast::new();
    let mut interner = eaml_lexer::Interner::default();
    let name_spur = interner.intern("SentimentResult");
    let schema_id = ast.alloc_schema(SchemaDecl {
        name: name_spur,
        fields: vec![],
        span: 0..0,
    });
    let resolved = ResolvedType::Schema(schema_id);
    assert_eq!(
        emit_type_annotation(&resolved, &ast, &interner),
        "SentimentResult"
    );
}

#[test]
fn type_annotation_array_of_string() {
    let (ast, interner) = make_ast_interner();
    let resolved = ResolvedType::Array(Box::new(ResolvedType::Primitive("string".to_string())));
    assert_eq!(
        emit_type_annotation(&resolved, &ast, &interner),
        "List[str]"
    );
}

#[test]
fn type_annotation_optional_string() {
    let (ast, interner) = make_ast_interner();
    let resolved = ResolvedType::Optional(Box::new(ResolvedType::Primitive("string".to_string())));
    assert_eq!(
        emit_type_annotation(&resolved, &ast, &interner),
        "Optional[str]"
    );
}

#[test]
fn type_annotation_optional_array_string() {
    let (ast, interner) = make_ast_interner();
    let resolved = ResolvedType::Optional(Box::new(ResolvedType::Array(Box::new(
        ResolvedType::Primitive("string".to_string()),
    ))));
    assert_eq!(
        emit_type_annotation(&resolved, &ast, &interner),
        "Optional[List[str]]"
    );
}

#[test]
fn type_annotation_array_optional_string() {
    let (ast, interner) = make_ast_interner();
    let resolved = ResolvedType::Array(Box::new(ResolvedType::Optional(Box::new(
        ResolvedType::Primitive("string".to_string()),
    ))));
    assert_eq!(
        emit_type_annotation(&resolved, &ast, &interner),
        "List[Optional[str]]"
    );
}

#[test]
fn type_annotation_optional_array_optional_string() {
    let (ast, interner) = make_ast_interner();
    let resolved = ResolvedType::Optional(Box::new(ResolvedType::Array(Box::new(
        ResolvedType::Optional(Box::new(ResolvedType::Primitive("string".to_string()))),
    ))));
    assert_eq!(
        emit_type_annotation(&resolved, &ast, &interner),
        "Optional[List[Optional[str]]]"
    );
}

#[test]
fn type_annotation_literal_union() {
    let (ast, interner) = make_ast_interner();
    let resolved = ResolvedType::LiteralUnion(vec!["yes".to_string(), "no".to_string()]);
    assert_eq!(
        emit_type_annotation(&resolved, &ast, &interner),
        r#"Literal["yes", "no"]"#
    );
}

#[test]
fn type_annotation_error() {
    let (ast, interner) = make_ast_interner();
    let resolved = ResolvedType::Error;
    assert_eq!(emit_type_annotation(&resolved, &ast, &interner), "Any");
}

// ==========================================================================
// is_optional tests
// ==========================================================================

#[test]
fn is_optional_true_for_optional() {
    let resolved = ResolvedType::Optional(Box::new(ResolvedType::Primitive("string".to_string())));
    assert!(is_optional(&resolved));
}

#[test]
fn is_optional_false_for_primitive() {
    let resolved = ResolvedType::Primitive("string".to_string());
    assert!(!is_optional(&resolved));
}

#[test]
fn is_optional_false_for_array() {
    let resolved = ResolvedType::Array(Box::new(ResolvedType::Primitive("int".to_string())));
    assert!(!is_optional(&resolved));
}

// ==========================================================================
// emit_field_line tests
// ==========================================================================

#[test]
fn field_line_plain_string() {
    let (ast, interner) = make_ast_interner();
    let resolved = ResolvedType::Primitive("string".to_string());
    let type_expr = TypeExpr::Named(lasso::Spur::default(), 0..0);
    assert_eq!(
        emit_field_line("label", &resolved, &type_expr, &ast, &interner, ""),
        "label: str"
    );
}

#[test]
fn field_line_optional_defaults_to_none() {
    let (ast, interner) = make_ast_interner();
    let resolved = ResolvedType::Optional(Box::new(ResolvedType::Primitive("string".to_string())));
    let type_expr = TypeExpr::Named(lasso::Spur::default(), 0..0);
    assert_eq!(
        emit_field_line("source", &resolved, &type_expr, &ast, &interner, ""),
        "source: Optional[str] = None"
    );
}

#[test]
fn field_line_bounded_float() {
    let mut interner = eaml_lexer::Interner::default();
    let ast = Ast::new();
    let base_spur = interner.intern("float");
    // Source: "float(0.0, 1.0)" -- positional params at specific spans
    let source = "float(0.0, 1.0)";
    let type_expr = TypeExpr::Bounded {
        base: base_spur,
        params: vec![
            BoundParam {
                name: None,
                value_span: 6..9, // "0.0"
                span: 6..9,
            },
            BoundParam {
                name: None,
                value_span: 11..14, // "1.0"
                span: 11..14,
            },
        ],
        span: 0..15,
    };
    assert_eq!(
        emit_field_line(
            "score",
            &ResolvedType::Primitive("float".to_string()),
            &type_expr,
            &ast,
            &interner,
            source
        ),
        "score: float = Field(ge=0.0, le=1.0)"
    );
}

#[test]
fn field_line_bounded_int_named_params() {
    let mut interner = eaml_lexer::Interner::default();
    let ast = Ast::new();
    let base_spur = interner.intern("int");
    let min_spur = interner.intern("min");
    let max_spur = interner.intern("max");
    // Source: "int(min=0, max=100)"
    let source = "int(min=0, max=100)";
    let type_expr = TypeExpr::Bounded {
        base: base_spur,
        params: vec![
            BoundParam {
                name: Some(min_spur),
                value_span: 8..9, // "0"
                span: 4..9,
            },
            BoundParam {
                name: Some(max_spur),
                value_span: 15..18, // "100"
                span: 11..18,
            },
        ],
        span: 0..19,
    };
    assert_eq!(
        emit_field_line(
            "count",
            &ResolvedType::Primitive("int".to_string()),
            &type_expr,
            &ast,
            &interner,
            source
        ),
        "count: int = Field(ge=0, le=100)"
    );
}

#[test]
fn field_line_bounded_string_named_params() {
    let mut interner = eaml_lexer::Interner::default();
    let ast = Ast::new();
    let base_spur = interner.intern("string");
    let min_spur = interner.intern("min");
    let max_spur = interner.intern("max");
    // Source: "string(min=1, max=200)"
    let source = "string(min=1, max=200)";
    let type_expr = TypeExpr::Bounded {
        base: base_spur,
        params: vec![
            BoundParam {
                name: Some(min_spur),
                value_span: 11..12, // "1"
                span: 7..12,
            },
            BoundParam {
                name: Some(max_spur),
                value_span: 18..21, // "200"
                span: 14..21,
            },
        ],
        span: 0..22,
    };
    assert_eq!(
        emit_field_line(
            "name",
            &ResolvedType::Primitive("string".to_string()),
            &type_expr,
            &ast,
            &interner,
            source
        ),
        "name: str = Field(min_length=1, max_length=200)"
    );
}

// ==========================================================================
// ImportTracker tests
// ==========================================================================

#[test]
fn import_tracker_empty_emits_nothing() {
    let tracker = ImportTracker::new();
    let mut w = CodeWriter::new();
    tracker.emit_imports(&mut w);
    assert_eq!(w.finish(), "");
}

#[test]
fn import_tracker_pydantic_imports() {
    let mut tracker = ImportTracker::new();
    tracker.need_base_model();
    tracker.need_field();
    let mut w = CodeWriter::new();
    tracker.emit_imports(&mut w);
    let output = w.finish();
    assert!(output.contains("from pydantic import BaseModel, Field"));
}

#[test]
fn import_tracker_typing_imports() {
    let mut tracker = ImportTracker::new();
    tracker.need_optional();
    tracker.need_list();
    tracker.need_literal();
    let mut w = CodeWriter::new();
    tracker.emit_imports(&mut w);
    let output = w.finish();
    assert!(output.contains("from typing import List, Literal, Optional"));
}

#[test]
fn import_tracker_track_type_optional_array() {
    let mut tracker = ImportTracker::new();
    let resolved = ResolvedType::Optional(Box::new(ResolvedType::Array(Box::new(
        ResolvedType::Primitive("string".to_string()),
    ))));
    tracker.track_type(&resolved);
    let mut w = CodeWriter::new();
    tracker.emit_imports(&mut w);
    let output = w.finish();
    assert!(output.contains("Optional"));
    assert!(output.contains("List"));
}

#[test]
fn import_tracker_track_type_literal_union() {
    let mut tracker = ImportTracker::new();
    let resolved = ResolvedType::LiteralUnion(vec!["a".to_string()]);
    tracker.track_type(&resolved);
    let mut w = CodeWriter::new();
    tracker.emit_imports(&mut w);
    let output = w.finish();
    assert!(output.contains("Literal"));
}

#[test]
fn import_tracker_eaml_runtime_imports() {
    let mut tracker = ImportTracker::new();
    tracker.need_execute_prompt();
    tracker.need_agent();
    let mut w = CodeWriter::new();
    tracker.emit_imports(&mut w);
    let output = w.finish();
    assert!(output.contains("from eaml_runtime import Agent, execute_prompt"));
}
