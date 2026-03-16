//! Type annotation and field declaration emission for Python code generation.
//!
//! Maps EAML's `ResolvedType` to Python type annotations per TYPESYSTEM.md 10.3,
//! and emits Pydantic field declarations with bounded type constraints.

use std::collections::BTreeSet;

use eaml_lexer::Interner;
use eaml_parser::ast::{Ast, TypeExpr};
use eaml_semantic::type_checker::ResolvedType;

use crate::writer::CodeWriter;

/// Emits a Python type annotation string from a resolved EAML type.
///
/// Mapping per TYPESYSTEM.md section 10.3:
/// - `string` -> `str`
/// - `int` -> `int`
/// - `float` -> `float`
/// - `bool` -> `bool`
/// - `null` -> `None`
/// - `Schema(id)` -> schema name (PascalCase)
/// - `Array(T)` -> `List[T]`
/// - `Optional(T)` -> `Optional[T]`
/// - `LiteralUnion(members)` -> `Literal["a", "b"]`
/// - `Error` -> `Any`
pub fn emit_type_annotation(resolved: &ResolvedType, ast: &Ast, interner: &Interner) -> String {
    match resolved {
        ResolvedType::Primitive(name) => match name.as_str() {
            "string" => "str".to_string(),
            "null" => "None".to_string(),
            other => other.to_string(),
        },
        ResolvedType::Schema(id) => {
            let schema = &ast[*id];
            interner.resolve(&schema.name).to_string()
        }
        ResolvedType::Array(inner) => {
            format!("List[{}]", emit_type_annotation(inner, ast, interner))
        }
        ResolvedType::Optional(inner) => {
            format!("Optional[{}]", emit_type_annotation(inner, ast, interner))
        }
        ResolvedType::LiteralUnion(members) => {
            let items: Vec<String> = members.iter().map(|m| format!("\"{m}\"")).collect();
            format!("Literal[{}]", items.join(", "))
        }
        ResolvedType::Error => "Any".to_string(),
    }
}

/// Returns true if the outermost type is `Optional`.
pub fn is_optional(resolved: &ResolvedType) -> bool {
    matches!(resolved, ResolvedType::Optional(_))
}

/// Emits a complete Pydantic field declaration line.
///
/// Examples:
/// - `"label: str"`
/// - `"source: Optional[str] = None"`
/// - `"score: float = Field(ge=0.0, le=1.0)"`
/// - `"name: str = Field(min_length=1, max_length=200)"`
pub fn emit_field_line(
    field_name: &str,
    resolved: &ResolvedType,
    type_expr: &TypeExpr,
    ast: &Ast,
    interner: &Interner,
    source: &str,
) -> String {
    let annotation = emit_type_annotation(resolved, ast, interner);

    // Check for bounded type constraints
    if let TypeExpr::Bounded { base, params, .. } = type_expr {
        let base_name = interner.resolve(base);
        let is_string = base_name == "string";

        let mut kwargs = Vec::new();

        let all_positional = params.iter().all(|p| p.name.is_none());

        if all_positional && params.len() >= 2 {
            // Positional: first=min, second=max
            let min_val = &source[params[0].value_span.clone()];
            let max_val = &source[params[1].value_span.clone()];
            if is_string {
                kwargs.push(format!("min_length={min_val}"));
                kwargs.push(format!("max_length={max_val}"));
            } else {
                kwargs.push(format!("ge={min_val}"));
                kwargs.push(format!("le={max_val}"));
            }
        } else {
            // Named params
            for param in params {
                if let Some(name_spur) = param.name {
                    let param_name = interner.resolve(&name_spur);
                    let value = &source[param.value_span.clone()];
                    let kwarg_name = if is_string {
                        match param_name {
                            "min" | "minLen" => "min_length",
                            "max" | "maxLen" => "max_length",
                            _ => param_name,
                        }
                    } else {
                        match param_name {
                            "min" => "ge",
                            "max" => "le",
                            _ => param_name,
                        }
                    };
                    kwargs.push(format!("{kwarg_name}={value}"));
                }
            }
        }

        if !kwargs.is_empty() {
            return format!("{field_name}: {annotation} = Field({})", kwargs.join(", "));
        }
    }

    // Optional fields default to None
    if is_optional(resolved) {
        return format!("{field_name}: {annotation} = None");
    }

    format!("{field_name}: {annotation}")
}

/// Tracks which imports are needed for generated Python code.
///
/// Accumulates import requirements as code is generated, then emits
/// sorted `from X import Y` lines.
pub struct ImportTracker {
    pydantic: BTreeSet<&'static str>,
    typing: BTreeSet<&'static str>,
    eaml_runtime: BTreeSet<&'static str>,
}

impl ImportTracker {
    /// Creates a new empty import tracker.
    pub fn new() -> Self {
        Self {
            pydantic: BTreeSet::new(),
            typing: BTreeSet::new(),
            eaml_runtime: BTreeSet::new(),
        }
    }

    pub fn need_base_model(&mut self) {
        self.pydantic.insert("BaseModel");
    }

    pub fn need_field(&mut self) {
        self.pydantic.insert("Field");
    }

    pub fn need_optional(&mut self) {
        self.typing.insert("Optional");
    }

    pub fn need_list(&mut self) {
        self.typing.insert("List");
    }

    pub fn need_literal(&mut self) {
        self.typing.insert("Literal");
    }

    pub fn need_any(&mut self) {
        self.typing.insert("Any");
    }

    pub fn need_execute_prompt(&mut self) {
        self.eaml_runtime.insert("execute_prompt");
    }

    pub fn need_agent(&mut self) {
        self.eaml_runtime.insert("Agent");
    }

    pub fn need_tool_metadata(&mut self) {
        self.eaml_runtime.insert("ToolMetadata");
    }

    /// Recursively walks a resolved type and tracks needed imports.
    pub fn track_type(&mut self, resolved: &ResolvedType) {
        match resolved {
            ResolvedType::Optional(inner) => {
                self.need_optional();
                self.track_type(inner);
            }
            ResolvedType::Array(inner) => {
                self.need_list();
                self.track_type(inner);
            }
            ResolvedType::LiteralUnion(_) => {
                self.need_literal();
            }
            ResolvedType::Error => {
                self.need_any();
            }
            ResolvedType::Primitive(_) | ResolvedType::Schema(_) => {}
        }
    }

    /// Emits sorted `from X import Y` lines for all tracked imports.
    pub fn emit_imports(&self, writer: &mut CodeWriter) {
        let mut emitted = false;

        if !self.pydantic.is_empty() {
            let items: Vec<&str> = self.pydantic.iter().copied().collect();
            writer.writeln(&format!("from pydantic import {}", items.join(", ")));
            emitted = true;
        }

        if !self.typing.is_empty() {
            let items: Vec<&str> = self.typing.iter().copied().collect();
            writer.writeln(&format!("from typing import {}", items.join(", ")));
            emitted = true;
        }

        if !self.eaml_runtime.is_empty() {
            let items: Vec<&str> = self.eaml_runtime.iter().copied().collect();
            writer.writeln(&format!("from eaml_runtime import {}", items.join(", ")));
            emitted = true;
        }

        if emitted {
            writer.blank_line();
        }
    }
}

impl Default for ImportTracker {
    fn default() -> Self {
        Self::new()
    }
}
