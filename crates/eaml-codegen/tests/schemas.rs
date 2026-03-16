//! Snapshot tests for schema and let-binding emission.

mod test_helpers;

use eaml_codegen::emitters::{emit_let, emit_schema};
use eaml_codegen::types::ImportTracker;
use eaml_codegen::writer::CodeWriter;
use eaml_parser::ast::DeclId;

/// Helper: emits all schemas and let bindings from source, returns the combined output.
fn emit_schemas_and_lets(source: &str) -> String {
    let (parse_output, analysis) = test_helpers::parse_and_analyze(source);
    let ast = &parse_output.ast;
    let interner = &parse_output.interner;
    let type_annotations = &analysis.type_annotations;

    let mut writer = CodeWriter::new();
    let mut imports = ImportTracker::new();

    for decl in &parse_output.program.declarations {
        match decl {
            DeclId::Schema(id) => {
                emit_schema(
                    &ast[*id],
                    ast,
                    interner,
                    type_annotations,
                    source,
                    &mut writer,
                    &mut imports,
                );
                writer.blank_line();
            }
            DeclId::Let(id) => {
                emit_let(
                    &ast[*id],
                    ast,
                    interner,
                    type_annotations,
                    source,
                    &mut writer,
                    &mut imports,
                );
                writer.blank_line();
            }
            _ => {}
        }
    }

    writer.finish()
}

#[test]
fn test_minimal_schema() {
    let source = r#"
schema Greeting {
  message: string
  word_count: int
}
"#;
    let output = emit_schemas_and_lets(source);
    insta::assert_snapshot!(output, @"
    class Greeting(BaseModel):
        message: str
        word_count: int
    ");
}

#[test]
fn test_bounded_float_schema() {
    let source = r#"
schema Scored {
  score: float<0.0, 1.0>
}
"#;
    let output = emit_schemas_and_lets(source);
    insta::assert_snapshot!(output, @"
    class Scored(BaseModel):
        score: float = Field(ge=0.0, le=1.0)
    ");
}

#[test]
fn test_optional_field() {
    let source = r#"
schema WithOptional {
  name: string
  source: string?
}
"#;
    let output = emit_schemas_and_lets(source);
    insta::assert_snapshot!(output, @"
    class WithOptional(BaseModel):
        name: str
        source: Optional[str] = None
    ");
}

#[test]
fn test_literal_union_field() {
    let source = r#"
schema Sentiment {
  sentiment: "positive" | "neutral" | "negative"
}
"#;
    let output = emit_schemas_and_lets(source);
    insta::assert_snapshot!(output, @r#"
    class Sentiment(BaseModel):
        sentiment: Literal["positive", "neutral", "negative"]
    "#);
}

#[test]
fn test_array_field() {
    let source = r#"
schema Tagged {
  tags: string[]
}
"#;
    let output = emit_schemas_and_lets(source);
    insta::assert_snapshot!(output, @"
    class Tagged(BaseModel):
        tags: List[str]
    ");
}

#[test]
fn test_optional_array_field() {
    let source = r#"
schema MaybeTagged {
  items: string[]?
}
"#;
    let output = emit_schemas_and_lets(source);
    insta::assert_snapshot!(output, @"
    class MaybeTagged(BaseModel):
        items: Optional[List[str]] = None
    ");
}

#[test]
fn test_schema_reference_field() {
    let source = r#"
schema Address {
  street: string
  city: string
}

schema Person {
  name: string
  address: Address
}
"#;
    let output = emit_schemas_and_lets(source);
    insta::assert_snapshot!(output, @"
    class Address(BaseModel):
        street: str
        city: str

    class Person(BaseModel):
        name: str
        address: Address
    ");
}

#[test]
fn test_let_binding() {
    let source = r#"
let x: int = 42
"#;
    let output = emit_schemas_and_lets(source);
    insta::assert_snapshot!(output, @"x: int = 42");
}

#[test]
fn test_all_type_variants() {
    let source = r#"
schema AllTypes {
  label: string
  score: float
  count: int
  active: bool
  probability: float<0.0, 1.0>
  optional_name: string?
  tags: string[]
  sentiment: "positive" | "neutral" | "negative"
}
"#;
    let output = emit_schemas_and_lets(source);
    insta::assert_snapshot!(output, @r#"
    class AllTypes(BaseModel):
        label: str
        score: float
        count: int
        active: bool
        probability: float = Field(ge=0.0, le=1.0)
        optional_name: Optional[str] = None
        tags: List[str]
        sentiment: Literal["positive", "neutral", "negative"]
    "#);
}
